use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Named scrap-league identity (Contested Frequency denies it exists).
pub const MODE_NAME: &str = "Contested Frequency";
/// Playlist label under the league lie.
pub const PLAYLIST_NAME: &str = "Arena Duel";

pub fn default_mode_name() -> String {
    MODE_NAME.to_string()
}

pub fn default_playlist() -> String {
    PLAYLIST_NAME.to_string()
}

pub fn default_host_line() -> String {
    "HOST: CONTESTED FREQUENCY. LEAGUE DENIES EXISTENCE. ARENA DUEL IS LIVE.".to_string()
}

/// Host line while Continuance compliance pressure is live.
pub fn compliance_host_line() -> String {
    "HOST: CONTINUANCE COMPLIANCE PING. APPROVED LANES ONLY.".to_string()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WeaponType {
    #[default]
    Flechette,
    Rail,
    Scatter,
}

impl WeaponType {
    pub fn damage(self) -> i32 {
        match self {
            WeaponType::Flechette => 25,
            WeaponType::Rail => 75,
            WeaponType::Scatter => 15,
        }
    }

    pub fn cooldown_ticks(self) -> u32 {
        match self {
            WeaponType::Flechette => 10,
            WeaponType::Rail => 40,
            WeaponType::Scatter => 5,
        }
    }

    pub fn spread_radians(self) -> f32 {
        match self {
            WeaponType::Flechette => 0.1,
            WeaponType::Rail => 0.05,
            WeaponType::Scatter => 0.3,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            WeaponType::Flechette => "Flechette",
            WeaponType::Rail => "Rail",
            WeaponType::Scatter => "Scatter",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Hello { role: Role, name: String },
    Action(Action),
    Speak(Speak),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Welcome {
        player_id: Option<Uuid>,
        role: Role,
        #[serde(default = "default_mode_name")]
        mode_name: String,
        #[serde(default = "default_playlist")]
        playlist: String,
    },
    Snapshot(Snapshot),
    Event(GameEvent),
    /// Unicast control-plane rejection (e.g. speak rate limit). Not broadcast.
    Error {
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Spectator,
    Human,
    Agent,
}

/// World-point or player-id aim target. Server applies yaw toward the target.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LookAt {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub z: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_id: Option<Uuid>,
}

/// Off-tick callout / taunt (control plane, not sticky Action).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Speak {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Action {
    #[serde(default)]
    pub forward: bool,
    #[serde(default)]
    pub back: bool,
    #[serde(default)]
    pub left: bool,
    #[serde(default)]
    pub right: bool,
    #[serde(default)]
    pub turn_left: bool,
    #[serde(default)]
    pub turn_right: bool,
    #[serde(default)]
    pub fire: bool,
    #[serde(default)]
    pub weapon_swap: Option<WeaponType>,
    /// Authoritative aim: yaw snaps toward player_id (preferred) or world x/z.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub look_at: Option<LookAt>,
}

/// Per-tick fire outcome for observe (hit-confirm without vision).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShotResult {
    pub shooter_id: Uuid,
    pub shooter: String,
    pub hit: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    pub damage: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_hp_after: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub tick: u64,
    pub players: Vec<PlayerState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub round_time_left: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frag_limit: Option<u32>,
    /// Shots resolved on this tick (empty omitted on wire).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shot_results: Vec<ShotResult>,
    /// Contested Frequency (scrap league that denies it exists).
    #[serde(default = "default_mode_name")]
    pub mode_name: String,
    /// Arena Duel under the league lie.
    #[serde(default = "default_playlist")]
    pub playlist: String,
    /// Live pressure beat id when Continuance is squeezing the round.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pressure: Option<String>,
    /// Sticky Contested Frequency Host chrome (mid-join / observe).
    #[serde(default = "default_host_line")]
    pub host_line: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: Uuid,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub hp: i32,
    pub just_fired: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior: Option<String>,
    pub score: u32,
    pub weapon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerScore {
    pub name: String,
    pub score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum GameEvent {
    Frag {
        killer: String,
        victim: String,
        killer_score: u32,
    },
    /// Non-lethal or pre-frag damage. Structured hit-confirm for agents.
    Hit {
        shooter: String,
        shooter_id: Uuid,
        target: String,
        target_id: Uuid,
        damage: i32,
        target_hp_after: i32,
    },
    Respawn {
        player: String,
    },
    RoundStart {
        round_number: u32,
        frag_limit: Option<u32>,
        time_limit: Option<u32>,
        players: Vec<String>,
        previous_winner: Option<String>,
        #[serde(default = "default_mode_name")]
        mode_name: String,
        #[serde(default = "default_playlist")]
        playlist: String,
        #[serde(default = "default_host_line")]
        host_line: String,
    },
    RoundEnd {
        winner: Option<String>,
        reason: String,
        final_scores: Vec<PlayerScore>,
        winner_score: Option<u32>,
    },
    PlayerJoined {
        player: String,
        role: String,
        round_number: u32,
        player_count: usize,
    },
    PlayerLeft {
        player: String,
        score: u32,
        round_number: u32,
        player_count: usize,
    },
    /// Mid-round Continuance compliance pressure (Host bumper + move slow).
    CompliancePing {
        message: String,
        duration_ticks: u32,
    },
    /// Off-tick agent/human callout (rate-limited, length-capped).
    Speak {
        player: String,
        player_id: Uuid,
        text: String,
    },
}

#[cfg(test)]
mod protocol_tests {
    use super::*;

    #[test]
    fn unknown_action_field_fails_deserialize() {
        let json = r#"{"type":"action","forward":true,"laser":true}"#;
        let parsed: Result<ClientMessage, _> = serde_json::from_str(json);
        assert!(
            parsed.is_err(),
            "unknown Action field must fail: {:?}",
            parsed
        );
    }

    #[test]
    fn valid_action_deserializes() {
        let json = r#"{"type":"action","forward":true,"fire":true}"#;
        let parsed: Result<ClientMessage, _> = serde_json::from_str(json);
        assert!(parsed.is_ok(), "{:?}", parsed);
        match parsed.unwrap() {
            ClientMessage::Action(a) => {
                assert!(a.forward);
                assert!(a.fire);
                assert!(!a.back);
                assert!(a.look_at.is_none());
            }
            other => panic!("expected Action, got {:?}", other),
        }
    }

    #[test]
    fn look_at_player_id_deserializes() {
        let id = Uuid::new_v4();
        let json = format!(r#"{{"type":"action","look_at":{{"player_id":"{}"}}}}"#, id);
        let parsed: ClientMessage = serde_json::from_str(&json).expect("look_at action");
        match parsed {
            ClientMessage::Action(a) => {
                let look = a.look_at.expect("look_at present");
                assert_eq!(look.player_id, Some(id));
                assert!(look.x.is_none());
                assert!(look.z.is_none());
            }
            other => panic!("expected Action, got {:?}", other),
        }
    }

    #[test]
    fn look_at_world_xz_deserializes() {
        let json = r#"{"type":"action","look_at":{"x":10.0,"z":-5.0}}"#;
        let parsed: ClientMessage = serde_json::from_str(json).expect("look_at xz");
        match parsed {
            ClientMessage::Action(a) => {
                let look = a.look_at.expect("look_at present");
                assert_eq!(look.x, Some(10.0));
                assert_eq!(look.z, Some(-5.0));
                assert!(look.player_id.is_none());
            }
            other => panic!("expected Action, got {:?}", other),
        }
    }

    #[test]
    fn unknown_look_at_field_fails_deserialize() {
        let json = r#"{"type":"action","look_at":{"x":1.0,"z":2.0,"laser":true}}"#;
        let parsed: Result<ClientMessage, _> = serde_json::from_str(json);
        assert!(
            parsed.is_err(),
            "unknown LookAt field must fail: {:?}",
            parsed
        );
    }

    #[test]
    fn shot_result_and_hit_event_round_trip() {
        let shooter_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        let shot = ShotResult {
            shooter_id,
            shooter: "A".into(),
            hit: true,
            target_id: Some(target_id),
            target: Some("B".into()),
            damage: 25,
            target_hp_after: Some(75),
        };
        let snap = Snapshot {
            tick: 1,
            players: vec![],
            round_state: None,
            round_time_left: None,
            frag_limit: None,
            shot_results: vec![shot.clone()],
            mode_name: default_mode_name(),
            playlist: default_playlist(),
            pressure: None,
            host_line: default_host_line(),
        };
        let v = serde_json::to_value(&snap).unwrap();
        assert_eq!(v["shot_results"][0]["hit"], true);
        assert_eq!(v["shot_results"][0]["damage"], 25);
        let back: Snapshot = serde_json::from_value(v).unwrap();
        assert_eq!(back.shot_results, vec![shot]);

        let hit = GameEvent::Hit {
            shooter: "A".into(),
            shooter_id,
            target: "B".into(),
            target_id,
            damage: 25,
            target_hp_after: 75,
        };
        let ev = serde_json::to_value(&hit).unwrap();
        assert_eq!(ev["event"], "hit");
        assert_eq!(ev["target_hp_after"], 75);
        let back: GameEvent = serde_json::from_value(ev).unwrap();
        match back {
            GameEvent::Hit {
                damage,
                target_hp_after,
                ..
            } => {
                assert_eq!(damage, 25);
                assert_eq!(target_hp_after, 75);
            }
            other => panic!("expected Hit, got {:?}", other),
        }
    }

    #[test]
    fn speak_deserializes_and_deny_unknown() {
        let ok: ClientMessage =
            serde_json::from_str(r#"{"type":"speak","text":"nice scrap"}"#).expect("speak");
        match ok {
            ClientMessage::Speak(s) => assert_eq!(s.text, "nice scrap"),
            other => panic!("expected Speak, got {:?}", other),
        }
        let bad: Result<ClientMessage, _> =
            serde_json::from_str(r#"{"type":"speak","text":"x","laser":true}"#);
        assert!(bad.is_err(), "unknown Speak field must fail: {:?}", bad);
    }

    #[test]
    fn speak_event_round_trip() {
        let id = Uuid::new_v4();
        let ev = GameEvent::Speak {
            player: "ArenaFox".into(),
            player_id: id,
            text: "frequency contested".into(),
        };
        let v = serde_json::to_value(&ev).unwrap();
        assert_eq!(v["event"], "speak");
        assert_eq!(v["text"], "frequency contested");
        assert_eq!(v["player"], "ArenaFox");
        let back: GameEvent = serde_json::from_value(v).unwrap();
        match back {
            GameEvent::Speak { text, player, .. } => {
                assert_eq!(text, "frequency contested");
                assert_eq!(player, "ArenaFox");
            }
            other => panic!("expected Speak, got {:?}", other),
        }
    }
}
