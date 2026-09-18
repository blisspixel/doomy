use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WeaponType {
    #[default]
    Flechette,
    Rail,
    Scatter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Hello { role: Role, name: String },
    Action(Action),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Welcome { player_id: Option<Uuid>, role: Role },
    Snapshot(Snapshot),
    Event(GameEvent),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Spectator,
    Human,
    Agent,
}

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub look_at: Option<LookAt>,
}

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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shot_results: Vec<ShotResult>,
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
}
