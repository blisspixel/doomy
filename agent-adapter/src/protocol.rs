use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MODE_NAME: &str = "Contested Frequency";
pub const PLAYLIST_NAME: &str = "Arena Duel";

pub fn default_mode_name() -> String {
    MODE_NAME.to_string()
}

pub fn default_playlist() -> String {
    PLAYLIST_NAME.to_string()
}

pub fn default_pickup_kind() -> String {
    "weapon".to_string()
}

pub fn default_host_line() -> String {
    "HOST: CONTESTED FREQUENCY. LEAGUE DENIES EXISTENCE. ARENA DUEL IS LIVE.".to_string()
}

pub const MAP_ID_ARENA_DUEL: u32 = 1;
pub const MAP_NAME_ARENA_DUEL: &str = "Arena Duel";

pub fn default_map_id() -> u32 {
    MAP_ID_ARENA_DUEL
}

pub fn default_map_name() -> String {
    MAP_NAME_ARENA_DUEL.to_string()
}

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
    Hello {
        role: Role,
        name: String,
    },
    Action(Action),
    Speak(Speak),
    /// Agent-only display label echoed into Snapshot PlayerState.behavior.
    SetDisplayBehavior(SetDisplayBehavior),
}

#[allow(clippy::large_enum_variant)] // Snapshot carries round chrome; boxing churns every tick.
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
    /// Unicast acknowledgement of the newest numbered input the server applied.
    /// Only clients that predict receive it; mirrored so the adapter never
    /// chokes on a message it is not interested in.
    Ack {
        seq: u32,
        tick: u64,
        x: f32,
        z: f32,
        yaw: f32,
    },
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

/// Observe-only stance / tactics chip for Agent clients (control plane).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SetDisplayBehavior {
    pub behavior: String,
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
    /// Client-owned absolute facing in radians; wins over the turn bits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yaw: Option<f32>,
    /// Input sequence the server acknowledges for predicting clients.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq: Option<u32>,
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

/// Floor pickup pad state mirrored from server Snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PickupState {
    pub id: String,
    #[serde(default = "default_pickup_kind")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub weapon: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<i32>,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub respawn_in: Option<u32>,
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
    #[serde(default = "default_mode_name")]
    pub mode_name: String,
    #[serde(default = "default_playlist")]
    pub playlist: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pressure: Option<String>,
    #[serde(default = "default_host_line")]
    pub host_line: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mvp: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mvp_frags: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pickups: Vec<PickupState>,
    #[serde(default = "default_map_id")]
    pub map_id: u32,
    #[serde(default = "default_map_name")]
    pub map_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_objective: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_progress: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episode_phase: Option<String>,
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
    #[serde(default)]
    pub armor: i32,
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mvp: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mvp_frags: Option<u32>,
        #[serde(default = "default_host_line")]
        host_line: String,
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
    CompliancePing {
        message: String,
        duration_ticks: u32,
    },
    BossSpawn {
        name: String,
        boss_id: Uuid,
        message: String,
        hp: i32,
    },
    BossDown {
        name: String,
        boss_id: Uuid,
        #[serde(skip_serializing_if = "Option::is_none")]
        killer: Option<String>,
        message: String,
    },
    Pickup {
        player: String,
        player_id: Uuid,
        #[serde(default = "default_pickup_kind")]
        kind: String,
        #[serde(default, skip_serializing_if = "String::is_empty")]
        weapon: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        amount: Option<i32>,
        pickup_id: String,
    },
    Killstreak {
        player: String,
        player_id: Uuid,
        streak: u32,
        tier: String,
        message: String,
    },
    Speak {
        player: String,
        player_id: Uuid,
        text: String,
    },
    EpisodeStart {
        id: String,
        title: String,
        objective: String,
        host_line: String,
        map_name: String,
    },
    EpisodeComplete {
        id: String,
        reason: String,
        host_line: String,
        unlock_teaser: String,
    },
    EpisodeFail {
        id: String,
        reason: String,
        host_line: String,
    },
}
