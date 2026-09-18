//! What the brain is told. A compact, deterministic text rendering of the
//! fighter's situation: itself, the nearest enemy, the pads worth reaching, and
//! the round. Small on purpose: decision models read literally and are thrown
//! by irrelevant state, and every character is billed.

use fragr_server::protocol::{GameEvent, Snapshot};
use serde_json::{json, Value};
use std::collections::{BTreeMap, VecDeque};
use uuid::Uuid;

/// Damage taken inside this many ticks counts as being under fire (two seconds).
pub const UNDER_FIRE_TICKS: u64 = 40;
/// Enemy range buckets, aligned with the weapon ranges the questions describe.
pub const CLOSE_RANGE_UNITS: f32 = 10.0;
pub const FAR_RANGE_UNITS: f32 = 30.0;
/// A pad inside this distance is "near".
pub const NEAR_PAD_UNITS: f32 = 12.0;
/// A round clock at or under this is "ending_soon".
pub const ENDING_SOON_SECONDS: u32 = 30;

/// Enemy distance as a word. The brain is not a calculator; comparisons happen here.
pub fn range_bucket(dist: f32) -> &'static str {
    if dist < CLOSE_RANGE_UNITS {
        "close"
    } else if dist <= FAR_RANGE_UNITS {
        "mid"
    } else {
        "far"
    }
}

/// Pad distance as a word.
pub fn pad_bucket(dist: Option<f32>) -> &'static str {
    match dist {
        None => "none",
        Some(d) if d <= NEAR_PAD_UNITS => "near",
        Some(_) => "far",
    }
}

/// Hits landed on this fighter, stamped with the tick they were seen at.
#[derive(Debug, Default, Clone)]
pub struct RecentHits {
    hits: VecDeque<(u64, i32)>,
}

impl RecentHits {
    /// Record a hit event if it targets `me`. Returns true when it did.
    pub fn ingest(&mut self, me: Uuid, tick: u64, event: &GameEvent) -> bool {
        if let GameEvent::Hit {
            target_id, damage, ..
        } = event
        {
            if *target_id == me {
                self.hits.push_back((tick, *damage));
                return true;
            }
        }
        false
    }

    /// Total damage taken within `window` ticks before `tick`, dropping older hits.
    pub fn damage_within(&mut self, tick: u64, window: u64) -> i32 {
        let floor = tick.saturating_sub(window);
        while matches!(self.hits.front(), Some((t, _)) if *t < floor) {
            self.hits.pop_front();
        }
        // A hit stamped in the future means the server restarted its clock.
        self.hits.retain(|(t, _)| *t <= tick);
        self.hits.iter().map(|(_, d)| *d).sum()
    }

    pub fn len(&self) -> usize {
        self.hits.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hits.is_empty()
    }
}

/// The nearest living opponent as the brain sees it.
#[derive(Debug, Clone, PartialEq)]
pub struct EnemyView {
    pub id: Uuid,
    pub name: String,
    pub dist: f32,
    pub hp: i32,
    pub weapon: String,
}

/// One fighter's situation on one tick.
#[derive(Debug, Clone, PartialEq)]
pub struct Telemetry {
    pub tick: u64,
    pub name: String,
    pub hp: i32,
    pub armor: i32,
    pub weapon: String,
    pub under_fire: bool,
    pub recent_damage: i32,
    pub score: u32,
    pub top_rival_score: u32,
    pub round_state: String,
    pub round_time_left: Option<u32>,
    pub fighters: usize,
    pub enemy: Option<EnemyView>,
    /// Distance to the nearest available health pad, if any.
    pub health_pad: Option<f32>,
    pub armor_pad: Option<f32>,
    /// Nearest available pad per weapon name, sorted by name.
    pub weapon_pads: BTreeMap<String, f32>,
}

/// Coarse HP band; the brain does not need the number.
pub fn hp_tier(hp: i32) -> &'static str {
    if hp < 35 {
        "low"
    } else if hp < 70 {
        "mid"
    } else {
        "high"
    }
}

fn dist2d(ax: f32, az: f32, bx: f32, bz: f32) -> f32 {
    ((ax - bx).powi(2) + (az - bz).powi(2)).sqrt()
}

/// Build telemetry for `me` from a snapshot. `None` when `me` is not in it.
pub fn observe(me: Uuid, snapshot: &Snapshot, hits: &mut RecentHits) -> Option<Telemetry> {
    let mine = snapshot.players.iter().find(|p| p.id == me)?;
    let mut enemy: Option<EnemyView> = None;
    let mut top_rival_score = 0;
    let mut fighters = 0;
    for other in &snapshot.players {
        if other.id != me {
            // A respawning leader is still the leader.
            top_rival_score = top_rival_score.max(other.score);
        }
        if other.hp <= 0 {
            continue;
        }
        fighters += 1;
        if other.id == me {
            continue;
        }
        let dist = dist2d(mine.x, mine.z, other.x, other.z);
        if enemy.as_ref().is_none_or(|e| dist < e.dist) {
            enemy = Some(EnemyView {
                id: other.id,
                name: other.name.clone(),
                dist,
                hp: other.hp,
                weapon: other.weapon.to_ascii_lowercase(),
            });
        }
    }
    let mut health_pad = None;
    let mut armor_pad = None;
    let mut weapon_pads: BTreeMap<String, f32> = BTreeMap::new();
    for pad in snapshot.pickups.iter().filter(|p| p.available) {
        let dist = dist2d(mine.x, mine.z, pad.x, pad.z);
        match pad.kind.as_str() {
            "health" => {
                if health_pad.is_none_or(|d| dist < d) {
                    health_pad = Some(dist);
                }
            }
            "armor" => {
                if armor_pad.is_none_or(|d| dist < d) {
                    armor_pad = Some(dist);
                }
            }
            _ => {
                let name = pad.weapon.to_ascii_lowercase();
                if name.is_empty() {
                    continue;
                }
                let entry = weapon_pads.entry(name).or_insert(dist);
                if dist < *entry {
                    *entry = dist;
                }
            }
        }
    }
    let recent_damage = hits.damage_within(snapshot.tick, UNDER_FIRE_TICKS);
    Some(Telemetry {
        tick: snapshot.tick,
        name: mine.name.clone(),
        hp: mine.hp,
        armor: mine.armor,
        weapon: mine.weapon.to_ascii_lowercase(),
        under_fire: recent_damage > 0,
        recent_damage,
        score: mine.score,
        top_rival_score,
        round_state: snapshot
            .round_state
            .clone()
            .unwrap_or_else(|| "unknown".to_string())
            .to_ascii_lowercase(),
        round_time_left: snapshot.round_time_left,
        fighters,
        enemy,
        health_pad,
        armor_pad,
        weapon_pads,
    })
}

fn dist_text(dist: Option<f32>) -> String {
    match dist {
        Some(d) => format!("{d:.1}"),
        None => "none".to_string(),
    }
}

impl Telemetry {
    fn score_edge(&self) -> &'static str {
        if self.score > self.top_rival_score {
            "ahead"
        } else if self.score < self.top_rival_score {
            "behind"
        } else {
            "even"
        }
    }

    fn clock_bucket(&self) -> &'static str {
        match self.round_time_left {
            None => "unknown",
            Some(t) if t <= ENDING_SOON_SECONDS => "ending_soon",
            Some(_) => "plenty",
        }
    }

    /// The state object sent to the brain: named fields, words instead of
    /// numbers, and only what the three questions use. TypeSafe's guidance is
    /// an object with descriptive names, comparisons done in code, and nothing
    /// unrelated to the questions (unrelated detail measurably hurts accuracy).
    pub fn state_object(&self) -> Value {
        let enemy = match &self.enemy {
            Some(enemy) => json!({
                "present": true,
                "range": range_bucket(enemy.dist),
                "health": hp_tier(enemy.hp),
                "weapon": enemy.weapon,
            }),
            None => json!({ "present": false }),
        };
        json!({
            "self": {
                "health": hp_tier(self.hp),
                "armor": if self.armor > 0 { "some" } else { "none" },
                "weapon": self.weapon,
                "taking_damage": self.under_fire,
                "score": self.score_edge(),
            },
            "enemy": enemy,
            "pads": {
                "health": pad_bucket(self.health_pad),
                "armor": pad_bucket(self.armor_pad),
            },
            "clock": self.clock_bucket(),
        })
    }

    /// The human-readable state for logs and the `ask` command. Stable field
    /// order, one line per group, distances to a tenth.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "SELF hp={} armor={} weapon={} under_fire={} recent_damage={} score={} top_rival={}\n",
            self.hp,
            self.armor,
            self.weapon,
            if self.under_fire { "yes" } else { "no" },
            self.recent_damage,
            self.score,
            self.top_rival_score
        ));
        match &self.enemy {
            Some(enemy) => out.push_str(&format!(
                "ENEMY name={} dist={:.1} hp={} weapon={}\n",
                enemy.name,
                enemy.dist,
                hp_tier(enemy.hp),
                enemy.weapon
            )),
            None => out.push_str("ENEMY none\n"),
        }
        out.push_str(&format!(
            "PADS health={} armor={}",
            dist_text(self.health_pad),
            dist_text(self.armor_pad)
        ));
        for (name, dist) in &self.weapon_pads {
            out.push_str(&format!(" weapon.{name}={dist:.1}"));
        }
        out.push('\n');
        out.push_str(&format!(
            "ROUND state={} time_left={} fighters={}\n",
            self.round_state,
            match self.round_time_left {
                Some(t) => t.to_string(),
                None => "none".to_string(),
            },
            self.fighters
        ));
        out
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    use fragr_server::protocol::{PickupState, PlayerState, Snapshot};
    use uuid::Uuid;

    pub fn player(name: &str, id: Uuid, x: f32, z: f32, hp: i32, weapon: &str) -> PlayerState {
        PlayerState {
            id,
            name: name.to_string(),
            x,
            y: 1.0,
            z,
            yaw: 0.0,
            hp,
            armor: 0,
            just_fired: false,
            behavior: None,
            score: 0,
            weapon: weapon.to_string(),
        }
    }

    pub fn pad(kind: &str, weapon: &str, x: f32, z: f32, available: bool) -> PickupState {
        PickupState {
            id: format!("{kind}-{weapon}-{x}-{z}"),
            kind: kind.to_string(),
            weapon: weapon.to_string(),
            amount: None,
            x,
            y: 0.0,
            z,
            available,
            respawn_in: None,
        }
    }

    pub fn snapshot(tick: u64, players: Vec<PlayerState>, pickups: Vec<PickupState>) -> Snapshot {
        Snapshot {
            tick,
            players,
            round_state: Some("Active".to_string()),
            round_time_left: Some(90),
            frag_limit: Some(10),
            shot_results: vec![],
            mode_name: "Contested Frequency".to_string(),
            playlist: "Arena Duel".to_string(),
            pressure: None,
            host_line: String::new(),
            mvp: None,
            mvp_frags: None,
            pickups,
            map_id: 1,
            map_name: "Arena Duel".to_string(),
        episode_id: None,
        episode_title: None,
        episode_objective: None,
        episode_progress: None,
        episode_phase: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::{pad, player, snapshot};
    use super::*;

    #[test]
    fn recent_hits_track_only_me_and_expire() {
        let me = Uuid::new_v4();
        let other = Uuid::new_v4();
        let mut hits = RecentHits::default();
        let on_me = GameEvent::Hit {
            shooter: "A".into(),
            shooter_id: other,
            target: "me".into(),
            target_id: me,
            damage: 25,
            target_hp_after: 75,
        };
        let on_other = GameEvent::Hit {
            shooter: "me".into(),
            shooter_id: me,
            target: "A".into(),
            target_id: other,
            damage: 25,
            target_hp_after: 75,
        };
        assert!(hits.ingest(me, 10, &on_me));
        assert!(!hits.ingest(me, 12, &on_other));
        assert!(!hits.ingest(me, 12, &GameEvent::Respawn { player: "x".into() }));
        assert!(hits.ingest(me, 30, &on_me));
        assert_eq!(hits.len(), 2);
        assert!(!hits.is_empty());
        assert_eq!(hits.damage_within(40, UNDER_FIRE_TICKS), 50);
        assert_eq!(hits.damage_within(60, UNDER_FIRE_TICKS), 25);
        assert_eq!(hits.damage_within(200, UNDER_FIRE_TICKS), 0);
        assert!(hits.is_empty());
        hits.ingest(me, 500, &on_me);
        assert_eq!(
            hits.damage_within(100, UNDER_FIRE_TICKS),
            0,
            "future hits drop"
        );
        assert!(hits.is_empty());
    }

    #[test]
    fn observe_picks_nearest_living_enemy_and_pads() {
        let me = Uuid::new_v4();
        let near = Uuid::new_v4();
        let far = Uuid::new_v4();
        let dead = Uuid::new_v4();
        let mut players = vec![
            player("me", me, 0.0, 0.0, 80, "Flechette"),
            player("far", far, 30.0, 0.0, 20, "Rail"),
            player("near", near, 0.0, 5.0, 60, "Scatter"),
            player("dead", dead, 1.0, 1.0, 0, "Rail"),
        ];
        players[1].score = 7;
        players[3].score = 9;
        let pickups = vec![
            pad("health", "", 3.0, 4.0, true),
            pad("health", "", 1.0, 0.0, false),
            pad("armor", "", 0.0, 8.0, true),
            pad("weapon", "Rail", 6.0, 8.0, true),
            pad("weapon", "Rail", 3.0, 0.0, true),
            pad("weapon", "", 2.0, 0.0, true),
        ];
        let snap = snapshot(100, players, pickups);
        let mut hits = RecentHits::default();
        let t = observe(me, &snap, &mut hits).expect("me is in the snapshot");
        assert_eq!(t.name, "me");
        assert_eq!(t.weapon, "flechette");
        assert_eq!(t.fighters, 3, "dead fighters do not count");
        let enemy = t.enemy.as_ref().unwrap();
        assert_eq!(enemy.name, "near");
        assert!((enemy.dist - 5.0).abs() < 1e-5);
        assert_eq!(enemy.weapon, "scatter");
        assert_eq!(t.top_rival_score, 9, "a respawning leader still leads");
        assert!((t.health_pad.unwrap() - 5.0).abs() < 1e-5);
        assert!((t.armor_pad.unwrap() - 8.0).abs() < 1e-5);
        assert_eq!(t.weapon_pads.len(), 1);
        assert!((t.weapon_pads["rail"] - 3.0).abs() < 1e-5);
        assert!(!t.under_fire);
        assert_eq!(t.round_state, "active");
        assert_eq!(t.round_time_left, Some(90));
        assert!(observe(Uuid::new_v4(), &snap, &mut hits).is_none());
    }

    #[test]
    fn observe_alone_and_under_fire() {
        let me = Uuid::new_v4();
        let mut snap = snapshot(50, vec![player("me", me, 0.0, 0.0, 30, "rail")], vec![]);
        snap.round_state = None;
        snap.round_time_left = None;
        let mut hits = RecentHits::default();
        let hit = GameEvent::Hit {
            shooter: "A".into(),
            shooter_id: Uuid::new_v4(),
            target: "me".into(),
            target_id: me,
            damage: 40,
            target_hp_after: 30,
        };
        hits.ingest(me, 45, &hit);
        let t = observe(me, &snap, &mut hits).unwrap();
        assert!(t.enemy.is_none());
        assert!(t.under_fire);
        assert_eq!(t.recent_damage, 40);
        assert_eq!(t.round_state, "unknown");
        assert_eq!(t.fighters, 1);
        let text = t.render();
        assert!(text.contains("ENEMY none"));
        assert!(text.contains("under_fire=yes"));
        assert!(text.contains("time_left=none"));
        assert!(text.contains("PADS health=none armor=none\n"));
    }

    #[test]
    fn render_is_compact_and_deterministic() {
        let me = Uuid::new_v4();
        let foe = Uuid::new_v4();
        let snap = snapshot(
            7,
            vec![
                player("me", me, 0.0, 0.0, 75, "Flechette"),
                player("Probe-2", foe, 12.3, 0.0, 50, "Rail"),
            ],
            vec![
                pad("health", "", 8.1, 0.0, true),
                pad("weapon", "Scatter", 0.0, 2.0, true),
            ],
        );
        let mut hits = RecentHits::default();
        let t = observe(me, &snap, &mut hits).unwrap();
        let text = t.render();
        let expected = "SELF hp=75 armor=0 weapon=flechette under_fire=no recent_damage=0 score=0 top_rival=0\n\
ENEMY name=Probe-2 dist=12.3 hp=mid weapon=rail\n\
PADS health=8.1 armor=none weapon.scatter=2.0\n\
ROUND state=active time_left=90 fighters=2\n";
        assert_eq!(text, expected);
        assert_eq!(t.render(), text);
        assert!(text.len() < 260, "state stays small: {}", text.len());
        assert_eq!(hp_tier(0), "low");
        assert_eq!(hp_tier(34), "low");
        assert_eq!(hp_tier(35), "mid");
        assert_eq!(hp_tier(70), "high");
    }

    #[test]
    fn buckets_turn_numbers_into_words() {
        assert_eq!(range_bucket(0.0), "close");
        assert_eq!(range_bucket(9.9), "close");
        assert_eq!(range_bucket(10.0), "mid");
        assert_eq!(range_bucket(30.0), "mid");
        assert_eq!(range_bucket(30.1), "far");
        assert_eq!(pad_bucket(None), "none");
        assert_eq!(pad_bucket(Some(12.0)), "near");
        assert_eq!(pad_bucket(Some(12.1)), "far");
    }

    #[test]
    fn state_object_is_words_only_and_minimal() {
        let me = Uuid::new_v4();
        let foe = Uuid::new_v4();
        let mut snap = snapshot(
            7,
            vec![
                player("me", me, 0.0, 0.0, 75, "Flechette"),
                player("Probe-2", foe, 12.3, 0.0, 30, "Rail"),
            ],
            vec![
                pad("health", "", 8.1, 0.0, true),
                pad("weapon", "Scatter", 0.0, 2.0, true),
            ],
        );
        snap.players[1].score = 3;
        snap.round_time_left = Some(20);
        let mut hits = RecentHits::default();
        let t = observe(me, &snap, &mut hits).unwrap();
        let state = t.state_object();
        assert_eq!(
            state,
            serde_json::json!({
                "self": {"health": "high", "armor": "none", "weapon": "flechette", "taking_damage": false, "score": "behind"},
                "enemy": {"present": true, "range": "mid", "health": "low", "weapon": "rail"},
                "pads": {"health": "near", "armor": "none"},
                "clock": "ending_soon",
            })
        );
        let text = state.to_string();
        assert!(
            !text.contains("12.3") && !text.contains("Probe-2"),
            "no numbers, no names: {text}"
        );
        assert!(text.len() < 260, "{}", text.len());
        let alone = snapshot(1, vec![player("me", me, 0.0, 0.0, 100, "rail")], vec![]);
        let t = observe(me, &alone, &mut hits).unwrap();
        let state = t.state_object();
        assert_eq!(state["enemy"], serde_json::json!({"present": false}));
        assert_eq!(state["self"]["score"], "even");
        assert_eq!(state["clock"], "plenty");
        let mut unknown = alone.clone();
        unknown.round_time_left = None;
        unknown.players[0].score = 2;
        let t = observe(me, &unknown, &mut hits).unwrap();
        assert_eq!(t.state_object()["clock"], "unknown");
        assert_eq!(t.state_object()["self"]["score"], "ahead");
    }
}
