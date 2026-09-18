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

/// Danger levels, safest first. The score answer indexes into this list.
pub const DANGER_LEVELS: [&str; 5] = ["safe", "watchful", "pressured", "critical", "dying"];

/// The fixed question set for arena play. Stable across calls so the provider
/// can cache and so estimates stay honest.
pub fn tactical_questions() -> BTreeMap<String, Question> {
    let mut questions = BTreeMap::new();
    questions.insert(
        Q_STANCE.to_string(),
        Question::Choice {
            instructions: "Pick the stance for the next second of an arena deathmatch. \
Distances are in world units. Health pads restore HP. A fighter respawns after death \
with full HP, so trading is fine when ahead."
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
                    "Shotgun, 15 damage per pellet, reaches 14 units: best inside 10 units"
                        .to_string(),
                ),
                (
                    "flechette".to_string(),
                    "Needle gun, 25 damage, reaches 42 units: best from 10 to 40 units".to_string(),
                ),
                (
                    "rail".to_string(),
                    "Railgun, 75 damage, slow, reaches 100 units: best beyond 30 units with a clear line"
                        .to_string(),
                ),
            ]),
        },
    );
    questions.insert(
        Q_DANGER.to_string(),
        Question::Score {
            instructions: "Rate how close this fighter is to dying in the next few seconds."
                .to_string(),
            criteria: DANGER_LEVELS.iter().map(|s| s.to_string()).collect(),
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

/// Turn answers into a plan. Anything below the confidence floor, unparseable,
/// or missing falls back to the local plan for that field. The stance decides
/// the plan's `source`; weapon and danger are best effort.
pub fn plan_from_answers(answers: &BTreeMap<String, Answer>, floor: f64, fallback: &Plan) -> Plan {
    let mut plan = fallback.clone();
    match answers.get(Q_STANCE) {
        Some(Answer::Choice {
            choice, confidence, ..
        }) => {
            let confidence = confidence.unwrap_or(0.0);
            match Stance::parse(choice) {
                Some(stance) if confidence >= floor => {
                    plan.stance = stance;
                    plan.confidence = confidence;
                    plan.source = Source::Remote;
                }
                _ => {
                    plan.source = Source::LowConfidence;
                    plan.confidence = confidence;
                }
            }
        }
        _ => {
            plan.source = Source::LowConfidence;
        }
    }
    if let Some(Answer::Choice {
        choice, confidence, ..
    }) = answers.get(Q_WEAPON)
    {
        if confidence.unwrap_or(0.0) >= floor {
            if let Some(weapon) = parse_weapon(choice) {
                plan.weapon = Some(weapon);
            }
        }
    }
    if let Some(Answer::Score { score, .. }) = answers.get(Q_DANGER) {
        if score.is_finite() {
            let level = score.round().clamp(1.0, DANGER_LEVELS.len() as f64);
            plan.danger = level as u8;
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
        let plan = plan_from_answers(&answers, 0.65, &local());
        assert_eq!(plan.stance, Stance::PushEnemy);
        assert_eq!(plan.weapon, Some(WeaponType::Rail));
        assert_eq!(plan.danger, 4);
        assert_eq!(plan.source, Source::Remote);
        assert!((plan.confidence - 0.9).abs() < 1e-9);

        answers.insert(Q_STANCE.to_string(), choice("push_enemy", 0.5));
        answers.insert(Q_WEAPON.to_string(), choice("rail", 0.5));
        let plan = plan_from_answers(&answers, 0.65, &local());
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
        let plan = plan_from_answers(&answers, 0.65, &local());
        assert_eq!(
            plan.stance,
            Stance::HoldAngle,
            "unknown options never leak in"
        );
        assert_eq!(plan.weapon, None);
        assert_eq!(plan.source, Source::LowConfidence);

        let plan = plan_from_answers(&BTreeMap::new(), 0.65, &local());
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
        let plan = plan_from_answers(&odd, 0.0, &local());
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
            plan_from_answers(&odd, 0.0, &local()).danger,
            5,
            "scores clamp"
        );
    }
}
