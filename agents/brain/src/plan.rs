//! Intent and the local controller. A `Plan` is what the brain (or the local
//! rules standing in for it) wants for the next second or so. `micro_action`
//! turns the plan into a wire action on every tick from the latest snapshot,
//! so aim, spacing, and fire never wait on the network.

use crate::telemetry::Telemetry;
use fragr_server::protocol::{Action, LookAt, Snapshot, WeaponType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stop closing in when this near the target.
pub const CLOSE_RANGE: f32 = 3.0;
/// Back away while kiting inside this distance.
pub const KITE_RANGE: f32 = 8.0;
/// Push when the enemy is inside this distance; hold beyond it.
pub const PUSH_RANGE: f32 = 25.0;
/// Head for health below this HP when a pad is available.
pub const LOW_HP: i32 = 40;
/// Prefer scatter inside this distance.
pub const SCATTER_RANGE: f32 = 10.0;
/// Prefer rail beyond this distance.
pub const RAIL_RANGE: f32 = 30.0;
/// Ticks between strafe direction changes while holding or kiting.
pub const STRAFE_PERIOD_TICKS: u64 = 20;

/// What the fighter is trying to do right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stance {
    PushEnemy,
    FallBackHeal,
    HoldAngle,
    KiteDistance,
}

impl Stance {
    pub const ALL: [Stance; 4] = [
        Stance::PushEnemy,
        Stance::FallBackHeal,
        Stance::HoldAngle,
        Stance::KiteDistance,
    ];

    /// The option name the brain chooses between.
    pub fn name(self) -> &'static str {
        match self {
            Stance::PushEnemy => "push_enemy",
            Stance::FallBackHeal => "fall_back_heal",
            Stance::HoldAngle => "hold_angle",
            Stance::KiteDistance => "kite_distance",
        }
    }

    pub fn parse(text: &str) -> Option<Stance> {
        Stance::ALL.into_iter().find(|s| s.name() == text)
    }

    /// The description the brain is given for this option.
    pub fn criteria(self) -> &'static str {
        match self {
            Stance::PushEnemy => {
                "Own HP is high or the enemy is low: close the distance and keep firing"
            }
            Stance::FallBackHeal => {
                "Own HP is low or damage is pouring in and a health pad is reachable: break off and go heal"
            }
            Stance::HoldAngle => {
                "No enemy near or the position is good: hold, strafe, and shoot what comes"
            }
            Stance::KiteDistance => {
                "The enemy is close with a short-range weapon: back off while firing"
            }
        }
    }
}

/// Where the current plan came from; reported so a run can be audited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// Nothing decided yet.
    Initial,
    /// Local rules by choice (`--provider local`).
    Local,
    /// The remote brain answered with enough confidence.
    Remote,
    /// The remote brain answered below the confidence floor; local rules used.
    LowConfidence,
    /// The remote call failed or timed out; local rules used.
    Failure,
    /// A spend cap stopped the call; local rules used.
    Budget,
}

/// The macro intent for the next stretch of play.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub stance: Stance,
    /// Swap to this weapon; `None` keeps the current one.
    pub weapon: Option<WeaponType>,
    /// 1 (safe) to 5 (dying).
    pub danger: u8,
    /// Confidence reported by the brain, or 1.0 for local rules.
    pub confidence: f64,
    pub source: Source,
}

impl Default for Plan {
    fn default() -> Self {
        Plan {
            stance: Stance::HoldAngle,
            weapon: None,
            danger: 1,
            confidence: 0.0,
            source: Source::Initial,
        }
    }
}

/// Parse a wire weapon name (`flechette`, `Rail`, ...).
pub fn parse_weapon(name: &str) -> Option<WeaponType> {
    match name.to_ascii_lowercase().as_str() {
        "flechette" => Some(WeaponType::Flechette),
        "rail" => Some(WeaponType::Rail),
        "scatter" => Some(WeaponType::Scatter),
        _ => None,
    }
}

pub fn weapon_name(weapon: WeaponType) -> &'static str {
    match weapon {
        WeaponType::Flechette => "flechette",
        WeaponType::Rail => "rail",
        WeaponType::Scatter => "scatter",
    }
}

/// The weapon local rules would hold at this distance.
pub fn weapon_for_distance(dist: f32) -> WeaponType {
    if dist < SCATTER_RANGE {
        WeaponType::Scatter
    } else if dist > RAIL_RANGE {
        WeaponType::Rail
    } else {
        WeaponType::Flechette
    }
}

/// Local rules: the plan the fighter follows when the brain is absent, slow,
/// unsure, or out of budget. Deterministic in the telemetry.
pub fn fallback_plan(t: &Telemetry, source: Source) -> Plan {
    let current = parse_weapon(&t.weapon);
    let (stance, danger, weapon) = match &t.enemy {
        _ if t.hp < LOW_HP && t.health_pad.is_some() => (Stance::FallBackHeal, 4, None),
        Some(enemy)
            if enemy.dist < KITE_RANGE
                && enemy.weapon == "scatter"
                && current != Some(WeaponType::Scatter) =>
        {
            (
                Stance::KiteDistance,
                3,
                Some(weapon_for_distance(enemy.dist)),
            )
        }
        Some(enemy) if enemy.dist < PUSH_RANGE => (
            Stance::PushEnemy,
            if t.under_fire { 3 } else { 2 },
            Some(weapon_for_distance(enemy.dist)),
        ),
        Some(enemy) => (Stance::HoldAngle, 1, Some(weapon_for_distance(enemy.dist))),
        None => (Stance::HoldAngle, 1, None),
    };
    Plan {
        stance,
        weapon: weapon.filter(|w| Some(*w) != current),
        danger,
        confidence: 1.0,
        source,
    }
}

fn strafe(tick: u64) -> (bool, bool) {
    if (tick / STRAFE_PERIOD_TICKS).is_multiple_of(2) {
        (true, false)
    } else {
        (false, true)
    }
}

/// Every tick: turn the plan into a wire action from the latest snapshot.
pub fn micro_action(plan: &Plan, me: Uuid, snapshot: &Snapshot) -> Action {
    let Some(mine) = snapshot.players.iter().find(|p| p.id == me) else {
        return Action::default();
    };
    let held = parse_weapon(&mine.weapon).unwrap_or_default();
    let weapon_swap = plan.weapon.filter(|w| *w != held);
    let fire_range = weapon_swap.unwrap_or(held).range_units();
    let mut nearest: Option<(f32, Uuid, f32, f32)> = None;
    for other in &snapshot.players {
        if other.id == me || other.hp <= 0 {
            continue;
        }
        let dist = ((other.x - mine.x).powi(2) + (other.z - mine.z).powi(2)).sqrt();
        if nearest.is_none_or(|(d, _, _, _)| dist < d) {
            nearest = Some((dist, other.id, other.x, other.z));
        }
    }
    let (left, right) = strafe(snapshot.tick);
    let mut action = Action {
        weapon_swap,
        ..Action::default()
    };
    if plan.stance == Stance::FallBackHeal {
        let pad = snapshot
            .pickups
            .iter()
            .filter(|p| p.available && p.kind == "health")
            .map(|p| {
                let dist = ((p.x - mine.x).powi(2) + (p.z - mine.z).powi(2)).sqrt();
                (dist, p.x, p.z)
            })
            .min_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((dist, x, z)) = pad {
            action.look_at = Some(LookAt {
                x: Some(x),
                z: Some(z),
                player_id: None,
            });
            action.forward = dist > 1.0;
            return action;
        }
        // No pad to run to: behave like a kiter.
    }
    let Some((dist, target, _, _)) = nearest else {
        return action;
    };
    action.look_at = Some(LookAt {
        player_id: Some(target),
        x: None,
        z: None,
    });
    action.fire = dist <= fire_range;
    match plan.stance {
        Stance::PushEnemy => {
            action.forward = dist > CLOSE_RANGE;
        }
        Stance::HoldAngle => {
            action.left = left;
            action.right = right;
        }
        Stance::KiteDistance | Stance::FallBackHeal => {
            action.back = dist < KITE_RANGE;
            action.left = left;
            action.right = right;
        }
    }
    action
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::fixtures::{pad, player, snapshot};
    use crate::telemetry::{observe, RecentHits};

    fn telemetry_for(snapshot: &Snapshot, me: Uuid) -> Telemetry {
        let mut hits = RecentHits::default();
        observe(me, snapshot, &mut hits).unwrap()
    }

    #[test]
    fn stance_names_roundtrip_and_have_criteria() {
        for stance in Stance::ALL {
            assert_eq!(Stance::parse(stance.name()), Some(stance));
            assert!(stance.criteria().len() > 20);
            let json = serde_json::to_string(&stance).unwrap();
            assert_eq!(json, format!("\"{}\"", stance.name()));
        }
        assert_eq!(Stance::parse("teleport"), None);
        assert_eq!(Plan::default().stance, Stance::HoldAngle);
        assert_eq!(Plan::default().source, Source::Initial);
    }

    #[test]
    fn weapon_helpers() {
        assert_eq!(parse_weapon("Rail"), Some(WeaponType::Rail));
        assert_eq!(parse_weapon("scatter"), Some(WeaponType::Scatter));
        assert_eq!(parse_weapon("FLECHETTE"), Some(WeaponType::Flechette));
        assert_eq!(parse_weapon("bfg"), None);
        for weapon in [WeaponType::Flechette, WeaponType::Rail, WeaponType::Scatter] {
            assert_eq!(parse_weapon(weapon_name(weapon)), Some(weapon));
        }
        assert_eq!(weapon_for_distance(2.0), WeaponType::Scatter);
        assert_eq!(weapon_for_distance(20.0), WeaponType::Flechette);
        assert_eq!(weapon_for_distance(50.0), WeaponType::Rail);
    }

    #[test]
    fn fallback_rules_cover_each_stance() {
        let me = Uuid::new_v4();
        let foe = Uuid::new_v4();
        // Low HP with a pad: heal.
        let snap = snapshot(
            1,
            vec![
                player("me", me, 0.0, 0.0, 20, "flechette"),
                player("foe", foe, 4.0, 0.0, 90, "scatter"),
            ],
            vec![pad("health", "", 10.0, 0.0, true)],
        );
        let plan = fallback_plan(&telemetry_for(&snap, me), Source::Local);
        assert_eq!(plan.stance, Stance::FallBackHeal);
        assert_eq!(plan.danger, 4);
        assert_eq!(plan.weapon, None);
        assert_eq!(plan.source, Source::Local);
        assert_eq!(plan.confidence, 1.0);
        // Low HP, no pad, scatter enemy in the face: kite.
        let snap = snapshot(
            1,
            vec![
                player("me", me, 0.0, 0.0, 20, "flechette"),
                player("foe", foe, 4.0, 0.0, 90, "scatter"),
            ],
            vec![],
        );
        let plan = fallback_plan(&telemetry_for(&snap, me), Source::Failure);
        assert_eq!(plan.stance, Stance::KiteDistance);
        assert_eq!(plan.weapon, Some(WeaponType::Scatter));
        assert_eq!(plan.source, Source::Failure);
        // Healthy, enemy in push range: push, keep flechette (already held).
        let snap = snapshot(
            1,
            vec![
                player("me", me, 0.0, 0.0, 90, "flechette"),
                player("foe", foe, 15.0, 0.0, 90, "rail"),
            ],
            vec![],
        );
        let plan = fallback_plan(&telemetry_for(&snap, me), Source::Local);
        assert_eq!(plan.stance, Stance::PushEnemy);
        assert_eq!(plan.danger, 2);
        assert_eq!(plan.weapon, None, "already holding the right weapon");
        // Far enemy: hold and swap to rail.
        let snap = snapshot(
            1,
            vec![
                player("me", me, 0.0, 0.0, 90, "flechette"),
                player("foe", foe, 40.0, 0.0, 90, "rail"),
            ],
            vec![],
        );
        let plan = fallback_plan(&telemetry_for(&snap, me), Source::Local);
        assert_eq!(plan.stance, Stance::HoldAngle);
        assert_eq!(plan.weapon, Some(WeaponType::Rail));
        // Alone: hold, no swap.
        let snap = snapshot(1, vec![player("me", me, 0.0, 0.0, 90, "rail")], vec![]);
        let plan = fallback_plan(&telemetry_for(&snap, me), Source::Local);
        assert_eq!(plan.stance, Stance::HoldAngle);
        assert_eq!(plan.weapon, None);
        assert_eq!(plan.danger, 1);
    }

    #[test]
    fn fallback_marks_danger_under_fire() {
        let me = Uuid::new_v4();
        let foe = Uuid::new_v4();
        let snap = snapshot(
            100,
            vec![
                player("me", me, 0.0, 0.0, 90, "flechette"),
                player("foe", foe, 15.0, 0.0, 90, "rail"),
            ],
            vec![],
        );
        let mut hits = RecentHits::default();
        hits.ingest(
            me,
            99,
            &fragr_server::protocol::GameEvent::Hit {
                shooter: "foe".into(),
                shooter_id: foe,
                target: "me".into(),
                target_id: me,
                damage: 10,
                target_hp_after: 90,
            },
        );
        let t = observe(me, &snap, &mut hits).unwrap();
        assert_eq!(fallback_plan(&t, Source::Local).danger, 3);
    }

    #[test]
    fn micro_push_hold_and_kite() {
        let me = Uuid::new_v4();
        let foe = Uuid::new_v4();
        let snap = snapshot(
            0,
            vec![
                player("me", me, 0.0, 0.0, 90, "flechette"),
                player("foe", foe, 10.0, 0.0, 90, "rail"),
            ],
            vec![],
        );
        let push = Plan {
            stance: Stance::PushEnemy,
            weapon: Some(WeaponType::Rail),
            danger: 2,
            confidence: 0.9,
            source: Source::Remote,
        };
        let action = micro_action(&push, me, &snap);
        assert_eq!(action.look_at.as_ref().unwrap().player_id, Some(foe));
        assert!(action.forward);
        assert!(action.fire, "rail reaches 10 units");
        assert_eq!(action.weapon_swap, Some(WeaponType::Rail));
        assert!(!action.left && !action.right && !action.back);

        let hold = Plan {
            stance: Stance::HoldAngle,
            weapon: None,
            ..push.clone()
        };
        let action = micro_action(&hold, me, &snap);
        assert!(!action.forward);
        assert!(action.left && !action.right, "tick 0 strafes left");
        assert_eq!(action.weapon_swap, None);
        let mut later = snap.clone();
        later.tick = STRAFE_PERIOD_TICKS;
        let action = micro_action(&hold, me, &later);
        assert!(!action.left && action.right, "next period strafes right");

        let kite = Plan {
            stance: Stance::KiteDistance,
            ..hold.clone()
        };
        let action = micro_action(&kite, me, &snap);
        assert!(!action.back, "10 units is outside kite range");
        let mut close = snap.clone();
        close.players[1].x = 4.0;
        let action = micro_action(&kite, me, &close);
        assert!(action.back);
        assert!(action.fire);

        let swap_same = Plan {
            weapon: Some(WeaponType::Flechette),
            ..push.clone()
        };
        assert_eq!(
            micro_action(&swap_same, me, &snap).weapon_swap,
            None,
            "no swap to the held weapon"
        );
    }

    #[test]
    fn micro_fire_range_follows_the_weapon() {
        let me = Uuid::new_v4();
        let foe = Uuid::new_v4();
        let snap = snapshot(
            0,
            vec![
                player("me", me, 0.0, 0.0, 90, "scatter"),
                player("foe", foe, 20.0, 0.0, 90, "rail"),
            ],
            vec![],
        );
        let push = Plan {
            stance: Stance::PushEnemy,
            weapon: None,
            danger: 2,
            confidence: 1.0,
            source: Source::Local,
        };
        assert!(
            !micro_action(&push, me, &snap).fire,
            "scatter is a 14 unit weapon"
        );
        let with_rail = Plan {
            weapon: Some(WeaponType::Rail),
            ..push
        };
        assert!(
            micro_action(&with_rail, me, &snap).fire,
            "the swapped weapon sets range"
        );
    }

    #[test]
    fn micro_heal_runs_to_the_nearest_pad_or_kites() {
        let me = Uuid::new_v4();
        let foe = Uuid::new_v4();
        let snap = snapshot(
            0,
            vec![
                player("me", me, 0.0, 0.0, 20, "flechette"),
                player("foe", foe, 5.0, 0.0, 90, "scatter"),
            ],
            vec![
                pad("health", "", 0.0, 12.0, true),
                pad("health", "", 0.0, 6.0, true),
                pad("health", "", 0.0, 1.0, false),
                pad("armor", "", 0.5, 0.0, true),
            ],
        );
        let heal = Plan {
            stance: Stance::FallBackHeal,
            weapon: None,
            danger: 4,
            confidence: 1.0,
            source: Source::Local,
        };
        let action = micro_action(&heal, me, &snap);
        let look = action.look_at.unwrap();
        assert_eq!(look.player_id, None);
        assert_eq!(look.z, Some(6.0), "nearest available health pad");
        assert!(action.forward);
        assert!(!action.fire);
        let mut on_pad = snap.clone();
        on_pad.players[0].z = 5.5;
        assert!(!micro_action(&heal, me, &on_pad).forward, "stop on the pad");
        let mut no_pads = snap.clone();
        no_pads.pickups.clear();
        let action = micro_action(&heal, me, &no_pads);
        assert_eq!(action.look_at.unwrap().player_id, Some(foe));
        assert!(action.back, "kite when there is nowhere to heal");
    }

    #[test]
    fn micro_handles_missing_self_and_no_enemies() {
        let me = Uuid::new_v4();
        let plan = Plan::default();
        let empty = snapshot(0, vec![], vec![]);
        let action = micro_action(&plan, me, &empty);
        assert!(action.look_at.is_none() && !action.fire && !action.forward);
        let alone = snapshot(
            0,
            vec![
                player("me", me, 0.0, 0.0, 90, "rail"),
                player("corpse", Uuid::new_v4(), 1.0, 0.0, 0, "rail"),
            ],
            vec![],
        );
        let action = micro_action(&plan, me, &alone);
        assert!(action.look_at.is_none(), "dead fighters are not targets");
        assert!(!action.fire);
    }
}
