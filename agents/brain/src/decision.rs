//! The questions the brain is asked and the answers it gives back. Both
//! providers speak the same shape: a `questions` map keyed by name, each with a
//! `type` of `choice`, `noul` (a yes-or-no probability), or `score`, and an
//! `answers` map keyed the same way whose values carry the type-specific field
//! (`choice`, `noul`, `score`) plus `confidence` and `probabilities`.

use crate::budget::Pricing;
use crate::plan::{parse_weapon, Plan, Source, Stance};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Descriptions for the two sides of a yes-or-no question.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoulCriteria {
    #[serde(rename = "true")]
    pub yes: String,
    #[serde(rename = "false")]
    pub no: String,
}

/// One typed question. Serializes to the provider's wire shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Question {
    Noul {
        instructions: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        criteria: Option<NoulCriteria>,
    },
    Choice {
        instructions: String,
        criteria: BTreeMap<String, String>,
    },
    Score {
        instructions: String,
        criteria: Vec<String>,
    },
}

pub const Q_STANCE: &str = "stance";
pub const Q_WEAPON: &str = "weapon";
pub const Q_DANGER: &str = "danger";

/// Danger level names, safest first. The score answer's probabilities are
/// keyed by zero-based level index; `score` is the expectation over indices.
pub const DANGER_LEVELS: [&str; 5] = ["safe", "watchful", "pressured", "critical", "dying"];

/// When a remote answer is trusted. A choice passes when the top option leads
/// the runner-up by `margin_floor`, or when the provider's own `confidence`
/// statistic reaches `confidence_floor`. TypeSafe's worked example calls a
/// 0.60 versus 0.38 split "clear enough to act on" at a confidence of 0.39, so
/// the margin is the primary test and the confidence number the backup.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gate {
    pub confidence_floor: f64,
    pub margin_floor: f64,
}

impl Default for Gate {
    fn default() -> Self {
        Gate {
            confidence_floor: 0.65,
            margin_floor: 0.2,
        }
    }
}

/// Top probability minus the runner-up; `None` when no probabilities came back.
pub fn choice_margin(probabilities: &BTreeMap<String, f64>) -> Option<f64> {
    let mut sorted: Vec<f64> = probabilities
        .values()
        .copied()
        .filter(|p| p.is_finite())
        .collect();
    if sorted.is_empty() {
        return None;
    }
    sorted.sort_by(|a, b| b.total_cmp(a));
    Some(sorted[0] - sorted.get(1).copied().unwrap_or(0.0))
}

impl Gate {
    /// Whether a choice answer is trusted, and the trust figure to report
    /// (the margin when probabilities exist, else the confidence).
    pub fn accepts(
        &self,
        confidence: Option<f64>,
        probabilities: &BTreeMap<String, f64>,
    ) -> (bool, f64) {
        let margin = choice_margin(probabilities);
        let confidence = confidence.unwrap_or(0.0);
        let by_margin = margin.is_some_and(|m| m >= self.margin_floor);
        let by_confidence = confidence >= self.confidence_floor;
        (by_margin || by_confidence, margin.unwrap_or(confidence))
    }
}

/// The most likely level index from a score answer's probabilities, if keyed by index.
pub fn score_argmax(probabilities: &BTreeMap<String, f64>) -> Option<usize> {
    probabilities
        .iter()
        .filter_map(|(key, p)| key.parse::<usize>().ok().map(|i| (i, *p)))
        .filter(|(_, p)| p.is_finite())
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// The fixed question set for arena play. Stable across calls so estimates
/// stay honest. Written to TypeSafe's guidance: short atomic questions,
/// criteria that say what belongs and what belongs to a neighbour, score
/// levels that describe situations rather than degrees, no numbers.
pub fn tactical_questions() -> BTreeMap<String, Question> {
    let mut questions = BTreeMap::new();
    questions.insert(
        Q_STANCE.to_string(),
        Question::Choice {
            instructions:
                "Arena deathmatch. Pick the stance for the next second. Death only costs a respawn."
                    .to_string(),
            criteria: Stance::ALL
                .into_iter()
                .map(|s| (s.name().to_string(), s.criteria().to_string()))
                .collect(),
        },
    );
    questions.insert(
        Q_WEAPON.to_string(),
        Question::Choice {
            instructions: "Pick the weapon to hold for the next second. Swapping is instant."
                .to_string(),
            criteria: BTreeMap::from([
                (
                    "scatter".to_string(),
                    "Shotgun. For a close enemy. Not for mid or far range.".to_string(),
                ),
                (
                    "flechette".to_string(),
                    "Needle gun. For a mid-range enemy. Not for close or far.".to_string(),
                ),
                (
                    "rail".to_string(),
                    "Railgun, one heavy slow shot. For a far enemy. Not for close range."
                        .to_string(),
                ),
            ]),
        },
    );
    questions.insert(
        Q_DANGER.to_string(),
        Question::Score {
            instructions: "How close is this fighter to dying in the next few seconds?".to_string(),
            criteria: vec![
                "Healthy, not taking damage, no enemy close.".to_string(),
                "Healthy, an enemy at mid or far range, not taking damage.".to_string(),
                "Taking damage or an enemy close, with health to spare.".to_string(),
                "Low health with an enemy in range.".to_string(),
                "Low health, taking damage, enemy close, no health pad near.".to_string(),
            ],
        },
    );
    questions
}

/// One answer, keyed by the question's type.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Answer {
    Noul {
        /// Probability that the answer is yes.
        noul: f64,
    },
    Choice {
        choice: String,
        #[serde(default)]
        confidence: Option<f64>,
        #[serde(default)]
        probabilities: BTreeMap<String, f64>,
    },
    Score {
        score: f64,
        #[serde(default)]
        confidence: Option<f64>,
        #[serde(default)]
        legend: Value,
        #[serde(default)]
        probabilities: BTreeMap<String, f64>,
    },
}

/// Tokens billed, plus the dollar cost when the provider reports it (OpenRouter does).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cost: Option<f64>,
}

/// A decision response from either provider.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DecisionResponse {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub provider: Option<String>,
    pub answers: BTreeMap<String, Answer>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

impl DecisionResponse {
    /// Dollars this call cost: the provider's figure when given, else tokens times price.
    pub fn cost_usd(&self, pricing: &Pricing) -> Option<f64> {
        let usage = self.usage.as_ref()?;
        Some(
            usage
                .cost
                .unwrap_or_else(|| pricing.cost(usage.input_tokens, usage.output_tokens)),
        )
    }
}

/// Turn answers into a plan. Anything the gate rejects, unparseable, or
/// missing falls back to the local plan for that field. The stance decides
/// the plan's `source`; weapon and danger are best effort. Danger takes the
/// most likely level, never the interpolated expectation, because TypeSafe
/// documents the score's numerical calibration as weak.
/// Draw an option from the distribution the model returned, rather than always
/// taking the largest.
///
/// A decision model trained to be calibrated returns a probability for every
/// option, and that distribution is the policy. Taking the argmax every tick
/// throws the calibration away and makes the fighter deterministic: the same
/// situation always produces the same move, so a fighter that has walked into
/// a corner walks into it again, forever. Sampling keeps the model's own
/// ordering (the option it favours is still the one it usually gets) while
/// letting the rest of the distribution break a loop.
///
/// `roll` is a value in [0, 1) from a seeded stream, so a run reproduces.
pub fn sample_choice(probabilities: &BTreeMap<String, f64>, roll: f64) -> Option<&str> {
    let total: f64 = probabilities
        .values()
        .filter(|p| p.is_finite() && **p > 0.0)
        .sum();
    if !total.is_finite() || total <= 0.0 {
        return None;
    }
    let target = roll.clamp(0.0, 1.0) * total;
    let mut seen = 0.0;
    let mut last: Option<&str> = None;
    for (option, weight) in probabilities {
        if !weight.is_finite() || *weight <= 0.0 {
            continue;
        }
        seen += weight;
        last = Some(option.as_str());
        if seen > target {
            return last;
        }
    }
    // Floating point can leave the last bucket just short of the target.
    last
}

/// Turn the model's answers into a plan.
///
/// `roll` is a value in [0, 1) from a seeded stream. The stance is drawn from
/// the distribution the model returned rather than taken as its largest entry,
/// so the same situation does not always produce the same move and a fighter
/// wedged in a corner has a way out of it. Pass 0.0 for the model's favourite.
pub fn plan_from_answers(
    answers: &BTreeMap<String, Answer>,
    gate: &Gate,
    fallback: &Plan,
    roll: f64,
) -> Plan {
    let mut plan = fallback.clone();
    match answers.get(Q_STANCE) {
        Some(Answer::Choice {
            choice,
            confidence,
            probabilities,
        }) => {
            let (trusted, figure) = gate.accepts(*confidence, probabilities);
            plan.confidence = figure;
            // The distribution is the policy. Fall back to the named choice
            // when the model sent no probabilities to draw from.
            let drawn = sample_choice(probabilities, roll).unwrap_or(choice.as_str());
            match Stance::parse(drawn) {
                Some(stance) if trusted => {
                    plan.stance = stance;
                    plan.source = Source::Remote;
                }
                _ => {
                    plan.source = Source::LowConfidence;
                }
            }
        }
        _ => {
            plan.source = Source::LowConfidence;
        }
    }
    if let Some(Answer::Choice {
        choice,
        confidence,
        probabilities,
    }) = answers.get(Q_WEAPON)
    {
        if gate.accepts(*confidence, probabilities).0 {
            if let Some(weapon) = parse_weapon(choice) {
                plan.weapon = Some(weapon);
            }
        }
    }
    if let Some(Answer::Score {
        score,
        probabilities,
        ..
    }) = answers.get(Q_DANGER)
    {
        let level = match score_argmax(probabilities) {
            Some(index) => Some(index as i64 + 1),
            None if score.is_finite() => Some(score.round() as i64 + 1),
            None => None,
        };
        if let Some(level) = level {
            plan.danger = level.clamp(1, DANGER_LEVELS.len() as i64) as u8;
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragr_server::protocol::WeaponType;

    fn choice(choice: &str, confidence: f64) -> Answer {
        Answer::Choice {
            choice: choice.to_string(),
            confidence: Some(confidence),
            probabilities: BTreeMap::new(),
        }
    }

    fn loose() -> Gate {
        Gate {
            confidence_floor: 0.0,
            margin_floor: 0.0,
        }
    }

    fn local() -> Plan {
        Plan {
            stance: Stance::HoldAngle,
            weapon: None,
            danger: 1,
            confidence: 1.0,
            source: Source::Local,
        }
    }

    #[test]
    fn questions_serialize_to_the_wire_shape() {
        let questions = tactical_questions();
        assert_eq!(questions.len(), 3);
        let json = serde_json::to_value(&questions).unwrap();
        assert_eq!(json[Q_STANCE]["type"], "choice");
        assert!(
            json[Q_STANCE]["criteria"]["push_enemy"]
                .as_str()
                .unwrap()
                .len()
                > 10
        );
        assert!(json[Q_WEAPON]["criteria"]["rail"].as_str().is_some());
        assert_eq!(json[Q_DANGER]["type"], "score");
        assert_eq!(json[Q_DANGER]["criteria"].as_array().unwrap().len(), 5);
        let noul = Question::Noul {
            instructions: "Reload now?".into(),
            criteria: Some(NoulCriteria {
                yes: "clip low and no enemy".into(),
                no: "enemy in view".into(),
            }),
        };
        let json = serde_json::to_value(&noul).unwrap();
        assert_eq!(json["type"], "noul");
        assert_eq!(json["criteria"]["true"], "clip low and no enemy");
        let bare = Question::Noul {
            instructions: "x".into(),
            criteria: None,
        };
        assert!(serde_json::to_value(&bare)
            .unwrap()
            .get("criteria")
            .is_none());
        let back: Question = serde_json::from_value(json).unwrap();
        assert_eq!(back, noul);
    }

    #[test]
    fn answers_parse_both_provider_shapes() {
        let native = serde_json::json!({
            "model": "jev-1.13.0",
            "answers": {
                "stance": {"type": "choice", "choice": "push_enemy", "probabilities": {"push_enemy": 0.8, "hold_angle": 0.2}, "confidence": 0.8},
                "reload": {"type": "noul", "noul": 0.93},
                "danger": {"type": "score", "score": 2.4, "legend": {"1": "safe"}, "probabilities": {"2": 0.6}, "confidence": 0.5}
            },
            "usage": {"input_tokens": 300, "output_tokens": 0}
        });
        let parsed: DecisionResponse = serde_json::from_value(native).unwrap();
        assert_eq!(parsed.model, "jev-1.13.0");
        assert_eq!(parsed.provider, None);
        assert!(
            matches!(parsed.answers["reload"], Answer::Noul { noul } if (noul - 0.93).abs() < 1e-9)
        );
        let cost = parsed.cost_usd(&Pricing::default()).unwrap();
        assert!((cost - 300.0 * 0.042 / 1e6).abs() < 1e-15);

        let routed = serde_json::json!({
            "id": "gen-123",
            "model": "typesafe/jev-1.13",
            "provider": "TypeSafe",
            "answers": {
                "stance": {"type": "choice", "choice": "kite_distance", "confidence": 0.7}
            },
            "usage": {"input_tokens": 300, "output_tokens": 0, "cost": 0.00002}
        });
        let parsed: DecisionResponse = serde_json::from_value(routed).unwrap();
        assert_eq!(parsed.id.as_deref(), Some("gen-123"));
        assert_eq!(parsed.provider.as_deref(), Some("TypeSafe"));
        assert_eq!(parsed.cost_usd(&Pricing::default()), Some(0.00002));

        let no_usage: DecisionResponse =
            serde_json::from_value(serde_json::json!({"answers": {}})).unwrap();
        assert_eq!(no_usage.cost_usd(&Pricing::default()), None);
        assert!(
            serde_json::from_value::<DecisionResponse>(serde_json::json!({"model": "x"})).is_err()
        );
    }

    #[test]
    fn plan_from_answers_gates_on_confidence() {
        let mut answers = BTreeMap::new();
        answers.insert(Q_STANCE.to_string(), choice("push_enemy", 0.9));
        answers.insert(Q_WEAPON.to_string(), choice("rail", 0.9));
        answers.insert(
            Q_DANGER.to_string(),
            Answer::Score {
                score: 3.6,
                confidence: Some(0.5),
                legend: Value::Null,
                probabilities: BTreeMap::new(),
            },
        );
        let plan = plan_from_answers(&answers, &Gate::default(), &local(), 0.0);
        assert_eq!(plan.stance, Stance::PushEnemy);
        assert_eq!(plan.weapon, Some(WeaponType::Rail));
        assert_eq!(plan.danger, 5, "3.6 rounds to index 4, the last level");
        assert_eq!(plan.source, Source::Remote);
        assert!((plan.confidence - 0.9).abs() < 1e-9);

        answers.insert(Q_STANCE.to_string(), choice("push_enemy", 0.5));
        answers.insert(Q_WEAPON.to_string(), choice("rail", 0.5));
        let plan = plan_from_answers(&answers, &Gate::default(), &local(), 0.0);
        assert_eq!(
            plan.stance,
            Stance::HoldAngle,
            "low confidence keeps the local stance"
        );
        assert_eq!(plan.weapon, None, "low confidence keeps the local weapon");
        assert_eq!(plan.source, Source::LowConfidence);
        assert!((plan.confidence - 0.5).abs() < 1e-9);

        answers.insert(Q_STANCE.to_string(), choice("teleport", 0.99));
        answers.insert(Q_WEAPON.to_string(), choice("bfg", 0.99));
        let plan = plan_from_answers(&answers, &Gate::default(), &local(), 0.0);
        assert_eq!(
            plan.stance,
            Stance::HoldAngle,
            "unknown options never leak in"
        );
        assert_eq!(plan.weapon, None);
        assert_eq!(plan.source, Source::LowConfidence);

        let plan = plan_from_answers(&BTreeMap::new(), &Gate::default(), &local(), 0.0);
        assert_eq!(plan.source, Source::LowConfidence);
        assert_eq!(plan.danger, 1);

        let mut odd = BTreeMap::new();
        odd.insert(
            Q_STANCE.to_string(),
            Answer::Choice {
                choice: "hold_angle".into(),
                confidence: None,
                probabilities: BTreeMap::new(),
            },
        );
        odd.insert(
            Q_DANGER.to_string(),
            Answer::Score {
                score: f64::NAN,
                confidence: None,
                legend: Value::Null,
                probabilities: BTreeMap::new(),
            },
        );
        let plan = plan_from_answers(&odd, &loose(), &local(), 0.0);
        assert_eq!(
            plan.source,
            Source::Remote,
            "a zero floor accepts a missing confidence"
        );
        assert_eq!(plan.danger, 1, "a NaN score is ignored");
        odd.insert(
            Q_DANGER.to_string(),
            Answer::Score {
                score: 99.0,
                confidence: None,
                legend: Value::Null,
                probabilities: BTreeMap::new(),
            },
        );
        assert_eq!(
            plan_from_answers(&odd, &loose(), &local(), 0.0).danger,
            5,
            "scores clamp"
        );
        for (score, level) in [(0.29, 1u8), (0.5, 2), (1.4, 2), (2.6, 4), (3.67, 5)] {
            odd.insert(
                Q_DANGER.to_string(),
                Answer::Score {
                    score,
                    confidence: None,
                    legend: Value::Null,
                    probabilities: BTreeMap::new(),
                },
            );
            assert_eq!(
                plan_from_answers(&odd, &loose(), &local(), 0.0).danger,
                level,
                "score {score}"
            );
        }
    }
    #[test]
    fn margin_gate_accepts_clear_splits_below_the_confidence_floor() {
        let gate = Gate::default();
        let split = BTreeMap::from([
            ("push_enemy".to_string(), 0.6),
            ("hold_angle".to_string(), 0.38),
            ("kite_distance".to_string(), 0.02),
        ]);
        assert!((choice_margin(&split).unwrap() - 0.22).abs() < 1e-9);
        assert!(gate.accepts(Some(0.39), &split).0);
        let tight = BTreeMap::from([
            ("push_enemy".to_string(), 0.5),
            ("hold_angle".to_string(), 0.4),
        ]);
        assert!(!gate.accepts(Some(0.3), &tight).0);
        assert!(
            gate.accepts(Some(0.7), &tight).0,
            "confidence floor still counts"
        );
        assert_eq!(choice_margin(&BTreeMap::new()), None);
        assert_eq!(gate.accepts(None, &BTreeMap::new()), (false, 0.0));
        let single = BTreeMap::from([("x".to_string(), 0.9)]);
        assert!((choice_margin(&single).unwrap() - 0.9).abs() < 1e-9);
        let nan = BTreeMap::from([("x".to_string(), f64::NAN)]);
        assert_eq!(choice_margin(&nan), None);
        let mut answers = BTreeMap::new();
        answers.insert(
            Q_STANCE.to_string(),
            Answer::Choice {
                choice: "push_enemy".into(),
                confidence: Some(0.39),
                probabilities: split,
            },
        );
        let plan = plan_from_answers(&answers, &gate, &local(), 0.0);
        assert_eq!(plan.source, Source::Remote);
        assert!(
            (plan.confidence - 0.22).abs() < 1e-9,
            "the margin is reported"
        );
    }

    #[test]
    fn danger_takes_the_most_likely_level() {
        let probs = BTreeMap::from([
            ("0".to_string(), 0.0),
            ("1".to_string(), 0.01),
            ("2".to_string(), 0.03),
            ("3".to_string(), 0.25),
            ("4".to_string(), 0.71),
        ]);
        assert_eq!(score_argmax(&probs), Some(4));
        let mut answers = BTreeMap::new();
        answers.insert(
            Q_DANGER.to_string(),
            Answer::Score {
                score: 3.67,
                confidence: Some(0.72),
                legend: Value::Null,
                probabilities: probs,
            },
        );
        assert_eq!(
            plan_from_answers(&answers, &Gate::default(), &local(), 0.0).danger,
            5
        );
        let named = BTreeMap::from([("safe".to_string(), 0.9)]);
        assert_eq!(score_argmax(&named), None);
        answers.insert(
            Q_DANGER.to_string(),
            Answer::Score {
                score: 1.2,
                confidence: None,
                legend: Value::Null,
                probabilities: named,
            },
        );
        assert_eq!(
            plan_from_answers(&answers, &Gate::default(), &local(), 0.0).danger,
            2,
            "falls back to the rounded expectation"
        );
    }
}

#[cfg(test)]
mod sampling_tests {
    use super::*;

    fn dist(pairs: &[(&str, f64)]) -> BTreeMap<String, f64> {
        pairs.iter().map(|(k, v)| ((*k).to_string(), *v)).collect()
    }

    #[test]
    fn a_roll_lands_in_the_option_that_owns_it() {
        // Sorted by key: hold 0.1, push 0.7, retreat 0.2 -> cuts at .1 and .8.
        let d = dist(&[("push", 0.7), ("hold", 0.1), ("retreat", 0.2)]);
        assert_eq!(sample_choice(&d, 0.0), Some("hold"));
        assert_eq!(sample_choice(&d, 0.05), Some("hold"));
        assert_eq!(sample_choice(&d, 0.5), Some("push"));
        assert_eq!(sample_choice(&d, 0.79), Some("push"));
        assert_eq!(sample_choice(&d, 0.9), Some("retreat"));
        assert_eq!(sample_choice(&d, 1.0), Some("retreat"));
    }

    #[test]
    fn the_favoured_option_is_still_the_usual_one() {
        let d = dist(&[("push", 0.7), ("hold", 0.1), ("retreat", 0.2)]);
        let mut pushes = 0;
        let steps = 1000;
        for i in 0..steps {
            if sample_choice(&d, i as f64 / steps as f64) == Some("push") {
                pushes += 1;
            }
        }
        // Sweeping the whole range reproduces the distribution it came from.
        assert!(
            (pushes as f64 / steps as f64 - 0.7).abs() < 0.02,
            "push drawn {pushes} times in {steps}"
        );
    }

    #[test]
    fn a_distribution_that_does_not_add_up_is_still_usable() {
        // Unnormalised weights are normalised by their own total.
        let d = dist(&[("a", 2.0), ("b", 2.0)]);
        assert_eq!(sample_choice(&d, 0.1), Some("a"));
        assert_eq!(sample_choice(&d, 0.9), Some("b"));
    }

    #[test]
    fn nothing_to_draw_from_draws_nothing() {
        assert_eq!(sample_choice(&BTreeMap::new(), 0.5), None);
        assert_eq!(sample_choice(&dist(&[("a", 0.0)]), 0.5), None);
        assert_eq!(sample_choice(&dist(&[("a", f64::NAN)]), 0.5), None);
        assert_eq!(sample_choice(&dist(&[("a", -1.0)]), 0.5), None);
    }
}
