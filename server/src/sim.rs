use crate::protocol::{
    boss_down_host_line, boss_host_line, boss_round_wipe_host_line, compliance_host_line,
    default_host_line, default_mode_name, default_playlist, empty_mvp_host_line,
    episode0_host_line_auditor, episode0_host_line_cold_open, episode0_host_line_fail,
    episode0_host_line_jammer, episode0_host_line_nods, episode0_host_line_win,
    episode0_objective_chip, episode0_unlock_teaser, killstreak_host_line, mvp_host_line,
    roster_host_line, round_open_host_line, rule_bot_taunt_line, warmup_host_line, Action,
    BotTauntKind, GameEvent, PickupState, PlayerScore, PlayerState, Role, ServerMessage,
    ShotResult, Snapshot, WeaponType, AUDITOR_NAME, BOSS_NAME, EPISODE_ID_EP0,
    EPISODE_MAP_LARAK_LOT, EPISODE_TITLE_EP0, MODE_NAME, PLAYLIST_NAME,
};
use std::collections::HashMap;
use std::f32::consts::PI;
use uuid::Uuid;

const MOVE_SPEED: f32 = 5.0;
const TURN_SPEED: f32 = 2.0;
const ARENA_SIZE: f32 = 50.0;
const PLAYER_RADIUS: f32 = 0.5;
const RESPAWN_DELAY_TICKS: u32 = 60;
/// Ticks after a respawn during which a fighter cannot be hit (one second).
pub const SPAWN_SHIELD_TICKS: u32 = 20;
const HITSCAN_RANGE: f32 = 100.0;
const PLAYER_MAX_HP: i32 = 100;
/// Max Unicode scalars in a speak/taunt line (after trim).
pub const SPEAK_MAX_CHARS: usize = 80;
/// Min ticks between successful speaks for one player (~3s at 20 Hz).
pub const SPEAK_COOLDOWN_TICKS: u64 = 60;
/// Chance (percent) a rule-bot killer speaks after a frag (not every scrap).
const RULE_BOT_TAUNT_FRAG_PCT: u64 = 35;
/// Chance a rule-bot victim speaks after death.
const RULE_BOT_TAUNT_DEATH_PCT: u64 = 25;
/// Chance a rule-bot speaks on killstreak tier 2/3/5.
const RULE_BOT_TAUNT_STREAK_PCT: u64 = 55;
/// Chance the Warmup cadence speaker actually talks.
const RULE_BOT_TAUNT_WARMUP_PCT: u64 = 40;
/// Warmup ticks between attempts to pick one rule bot to speak.
const RULE_BOT_WARMUP_TAUNT_EVERY: u32 = 20;
/// Continuance Compliance Drone hit points (tankier than scrap fighters).
pub const BOSS_MAX_HP: i32 = 200;
/// Touch radius for mid-map pickup pads.
pub const PICKUP_CLAIM_RADIUS: f32 = 1.75;
/// Ticks until a claimed weapon pad respawns (~12s at 20 Hz).
pub const PICKUP_RESPAWN_TICKS: u32 = 20 * 12;
/// Ticks until a claimed health/armor pad respawns (~15s at 20 Hz).
pub const HEALTH_PICKUP_RESPAWN_TICKS: u32 = 20 * 15;
/// Max scrap armor (simple absorb-before-HP).
pub const PLAYER_MAX_ARMOR: i32 = 100;
/// Health pad heal amount (capped at PLAYER_MAX_HP).
pub const HEALTH_PAD_AMOUNT: i32 = 40;
/// Armor scrap grant amount (capped at PLAYER_MAX_ARMOR).
pub const ARMOR_PAD_AMOUNT: i32 = 25;

/// Solo Broadcast Episode 0: NODS frags the meatbag must clear.
pub const EP0_NODS_GOAL: u32 = 5;
/// Soft-touch radius for the jammer dish (arena center).
pub const EP0_JAMMER_RADIUS: f32 = 3.0;

/// Axis-aligned scrap solid in XZ (Godot props mirrored for authoritative cover).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Aabb2 {
    min_x: f32,
    max_x: f32,
    min_z: f32,
    max_z: f32,
}

impl Aabb2 {
    const fn from_center(cx: f32, cz: f32, half_x: f32, half_z: f32) -> Self {
        Self {
            min_x: cx - half_x,
            max_x: cx + half_x,
            min_z: cz - half_z,
            max_z: cz + half_z,
        }
    }

    pub(crate) const fn expand(self, r: f32) -> Self {
        Self {
            min_x: self.min_x - r,
            max_x: self.max_x + r,
            min_z: self.min_z - r,
            max_z: self.max_z + r,
        }
    }

    pub(crate) fn contains(self, x: f32, z: f32) -> bool {
        x >= self.min_x && x <= self.max_x && z >= self.min_z && z <= self.max_z
    }
}

/// Contested Frequency scrap layouts (1 = Arena Duel default, 2 = Compliance Yard).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MapKind {
    #[default]
    ArenaDuel = 1,
    ComplianceYard = 2,
}

impl MapKind {
    pub fn from_cli(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "1" | "arena" | "duel" | "arena-duel" | "arena_duel" => Some(Self::ArenaDuel),
            "2" | "compliance" | "yard" | "compliance-yard" | "compliance_yard" => {
                Some(Self::ComplianceYard)
            }
            _ => None,
        }
    }

    pub fn id(self) -> u32 {
        match self {
            Self::ArenaDuel => 1,
            Self::ComplianceYard => 2,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::ArenaDuel => "Arena Duel",
            Self::ComplianceYard => "Compliance Yard",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::ArenaDuel => Self::ComplianceYard,
            Self::ComplianceYard => Self::ArenaDuel,
        }
    }

    pub fn spawn_radius(self) -> f32 {
        match self {
            Self::ArenaDuel => ARENA_SIZE * 0.3,
            Self::ComplianceYard => ARENA_SIZE * 0.25,
        }
    }

    /// Scrap chokes matching Godot arena scenes.
    pub(crate) fn obstacles(self) -> Vec<Aabb2> {
        match self {
            Self::ArenaDuel => vec![
                // Pillars at (±7, ±7), mesh 2.5x2.5
                Aabb2::from_center(7.0, -7.0, 1.25, 1.25),
                Aabb2::from_center(-7.0, -7.0, 1.25, 1.25),
                Aabb2::from_center(7.0, 7.0, 1.25, 1.25),
                Aabb2::from_center(-7.0, 7.0, 1.25, 1.25),
                // Low walls N/S/E/W
                Aabb2::from_center(0.0, -10.0, 4.0, 0.4),
                Aabb2::from_center(0.0, 10.0, 4.0, 0.4),
                Aabb2::from_center(10.0, 0.0, 0.4, 4.0),
                Aabb2::from_center(-10.0, 0.0, 0.4, 4.0),
                // Crates
                Aabb2::from_center(4.0, 16.0, 1.0, 1.0),
                Aabb2::from_center(-15.0, 3.0, 1.0, 1.0),
                Aabb2::from_center(16.0, -4.0, 1.0, 1.0),
                Aabb2::from_center(-3.5, -14.0, 1.0, 1.0),
                Aabb2::from_center(3.5, -14.0, 1.0, 1.0),
                Aabb2::from_center(-3.5, 14.0, 1.0, 1.0),
                Aabb2::from_center(3.5, 14.0, 1.0, 1.0),
                Aabb2::from_center(-14.0, -5.0, 1.0, 1.0),
                Aabb2::from_center(-14.0, 5.0, 1.0, 1.0),
                Aabb2::from_center(14.0, 5.0, 1.0, 1.0),
                Aabb2::from_center(14.0, -5.0, 1.0, 1.0),
            ],
            Self::ComplianceYard => vec![
                // Inner yard posts (±5, ±5)
                Aabb2::from_center(5.0, -5.0, 1.0, 1.0),
                Aabb2::from_center(-5.0, -5.0, 1.0, 1.0),
                Aabb2::from_center(5.0, 5.0, 1.0, 1.0),
                Aabb2::from_center(-5.0, 5.0, 1.0, 1.0),
                // Split corridor bars (gaps at axes for hub lanes)
                Aabb2::from_center(-7.0, -8.0, 3.5, 0.45),
                Aabb2::from_center(7.0, -8.0, 3.5, 0.45),
                Aabb2::from_center(-7.0, 8.0, 3.5, 0.45),
                Aabb2::from_center(7.0, 8.0, 3.5, 0.45),
                Aabb2::from_center(8.0, -7.0, 0.45, 3.5),
                Aabb2::from_center(8.0, 7.0, 0.45, 3.5),
                Aabb2::from_center(-8.0, -7.0, 0.45, 3.5),
                Aabb2::from_center(-8.0, 7.0, 0.45, 3.5),
                // Outer scrap crates
                Aabb2::from_center(0.0, -16.0, 1.0, 1.0),
                Aabb2::from_center(0.0, 16.0, 1.0, 1.0),
                Aabb2::from_center(-16.0, 0.0, 1.0, 1.0),
                Aabb2::from_center(16.0, 0.0, 1.0, 1.0),
                Aabb2::from_center(14.0, 14.0, 1.0, 1.0),
                Aabb2::from_center(-14.0, -14.0, 1.0, 1.0),
            ],
        }
    }

    pub(crate) fn pickups(self) -> Vec<ArenaPickup> {
        match self {
            Self::ArenaDuel => vec![
                ArenaPickup {
                    id: "pad_rail".to_string(),
                    kind: PickupKind::Weapon(WeaponType::Rail),
                    amount: 0,
                    x: 12.0,
                    y: 0.4,
                    z: 12.0,
                    available: true,
                    respawn_timer: None,
                },
                ArenaPickup {
                    id: "pad_scatter".to_string(),
                    kind: PickupKind::Weapon(WeaponType::Scatter),
                    amount: 0,
                    x: -12.0,
                    y: 0.4,
                    z: -12.0,
                    available: true,
                    respawn_timer: None,
                },
                ArenaPickup {
                    id: "pad_flechette".to_string(),
                    kind: PickupKind::Weapon(WeaponType::Flechette),
                    amount: 0,
                    x: -12.0,
                    y: 0.4,
                    z: 12.0,
                    available: true,
                    respawn_timer: None,
                },
                ArenaPickup {
                    id: "pad_health_n".to_string(),
                    kind: PickupKind::Health,
                    amount: HEALTH_PAD_AMOUNT,
                    x: 0.0,
                    y: 0.4,
                    z: 8.0,
                    available: true,
                    respawn_timer: None,
                },
                ArenaPickup {
                    id: "pad_health_s".to_string(),
                    kind: PickupKind::Health,
                    amount: HEALTH_PAD_AMOUNT,
                    x: 0.0,
                    y: 0.4,
                    z: -8.0,
                    available: true,
                    respawn_timer: None,
                },
                ArenaPickup {
                    id: "pad_armor".to_string(),
                    kind: PickupKind::Armor,
                    amount: ARMOR_PAD_AMOUNT,
                    x: 8.0,
                    y: 0.4,
                    z: 0.0,
                    available: true,
                    respawn_timer: None,
                },
            ],
            Self::ComplianceYard => vec![
                ArenaPickup {
                    id: "pad_rail".to_string(),
                    kind: PickupKind::Weapon(WeaponType::Rail),
                    amount: 0,
                    x: 12.0,
                    y: 0.4,
                    z: 12.0,
                    available: true,
                    respawn_timer: None,
                },
                ArenaPickup {
                    id: "pad_scatter".to_string(),
                    kind: PickupKind::Weapon(WeaponType::Scatter),
                    amount: 0,
                    x: -12.0,
                    y: 0.4,
                    z: -12.0,
                    available: true,
                    respawn_timer: None,
                },
                ArenaPickup {
                    id: "pad_flechette".to_string(),
                    kind: PickupKind::Weapon(WeaponType::Flechette),
                    amount: 0,
                    x: -12.0,
                    y: 0.4,
                    z: 12.0,
                    available: true,
                    respawn_timer: None,
                },
                ArenaPickup {
                    id: "pad_health_n".to_string(),
                    kind: PickupKind::Health,
                    amount: HEALTH_PAD_AMOUNT,
                    x: 0.0,
                    y: 0.4,
                    z: 11.0,
                    available: true,
                    respawn_timer: None,
                },
                ArenaPickup {
                    id: "pad_health_s".to_string(),
                    kind: PickupKind::Health,
                    amount: HEALTH_PAD_AMOUNT,
                    x: 0.0,
                    y: 0.4,
                    z: -11.0,
                    available: true,
                    respawn_timer: None,
                },
                ArenaPickup {
                    id: "pad_armor".to_string(),
                    kind: PickupKind::Armor,
                    amount: ARMOR_PAD_AMOUNT,
                    x: 11.0,
                    y: 0.4,
                    z: 0.0,
                    available: true,
                    respawn_timer: None,
                },
            ],
        }
    }
}

fn circle_blocked(map: MapKind, x: f32, z: f32) -> bool {
    for obs in map.obstacles() {
        if obs.expand(PLAYER_RADIUS).contains(x, z) {
            return true;
        }
    }
    false
}

fn clamp_arena(x: f32, z: f32) -> (f32, f32) {
    let half = ARENA_SIZE / 2.0 - PLAYER_RADIUS;
    (x.clamp(-half, half), z.clamp(-half, half))
}

/// Quake-style slide: try full move, then axis slides, then stay.
fn resolve_move(map: MapKind, old_x: f32, old_z: f32, new_x: f32, new_z: f32) -> (f32, f32) {
    let (nx, nz) = clamp_arena(new_x, new_z);
    if !circle_blocked(map, nx, nz) {
        return (nx, nz);
    }
    let (sx, _) = clamp_arena(new_x, old_z);
    if !circle_blocked(map, sx, old_z) {
        return (sx, old_z);
    }
    let (_, sz) = clamp_arena(old_x, new_z);
    if !circle_blocked(map, old_x, sz) {
        return (old_x, sz);
    }
    clamp_arena(old_x, old_z)
}

/// Slab ray vs AABB. Returns entry distance along unit (dx,dz) when hit ahead.
fn ray_aabb_hit(ox: f32, oz: f32, dx: f32, dz: f32, obs: Aabb2) -> Option<f32> {
    let (tmin_x, tmax_x) = if dx.abs() < 1e-8 {
        if ox < obs.min_x || ox > obs.max_x {
            return None;
        }
        (f32::NEG_INFINITY, f32::INFINITY)
    } else {
        let inv = 1.0 / dx;
        let t1 = (obs.min_x - ox) * inv;
        let t2 = (obs.max_x - ox) * inv;
        (t1.min(t2), t1.max(t2))
    };
    let (tmin_z, tmax_z) = if dz.abs() < 1e-8 {
        if oz < obs.min_z || oz > obs.max_z {
            return None;
        }
        (f32::NEG_INFINITY, f32::INFINITY)
    } else {
        let inv = 1.0 / dz;
        let t1 = (obs.min_z - oz) * inv;
        let t2 = (obs.max_z - oz) * inv;
        (t1.min(t2), t1.max(t2))
    };
    let t_enter = tmin_x.max(tmin_z);
    let t_exit = tmax_x.min(tmax_z);
    if t_exit < t_enter || t_exit < 0.0 {
        None
    } else {
        Some(t_enter.max(0.0))
    }
}

fn ray_blocked_by_cover(map: MapKind, ox: f32, oz: f32, dx: f32, dz: f32, max_dist: f32) -> bool {
    for obs in map.obstacles() {
        if let Some(t) = ray_aabb_hit(ox, oz, dx, dz, obs) {
            if t < max_dist {
                return true;
            }
        }
    }
    false
}

fn spawn_on_ring(map: MapKind, angle: f32) -> (f32, f32, f32) {
    let spawn_radius = map.spawn_radius();
    let mut a = angle;
    for _ in 0..16 {
        let x = a.cos() * spawn_radius;
        let z = a.sin() * spawn_radius;
        if !circle_blocked(map, x, z) {
            return (x, z, a + PI);
        }
        a += PI / 8.0;
    }
    // Hub is clear of solids (drone spawn + fallback).
    (0.0, 0.0, angle + PI)
}

/// Result of attempting an off-tick speak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeakOutcome {
    Sent,
    RateLimited,
    Rejected,
}

/// Solo Broadcast Episode 0 phase (Calibration on Larak Lot).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpisodePhase {
    Nods,
    Jammer,
    Auditor,
    Won,
    Failed,
}

impl EpisodePhase {
    pub fn wire(self) -> &'static str {
        match self {
            Self::Nods => "nods",
            Self::Jammer => "jammer",
            Self::Auditor => "auditor",
            Self::Won => "won",
            Self::Failed => "failed",
        }
    }
}

/// Contested Frequency Solo Broadcast Episode 0 runtime (off = MP unchanged).
#[derive(Debug, Clone)]
pub struct SoloBroadcastEp0 {
    pub enabled: bool,
    pub started: bool,
    pub nods_cleared: u32,
    pub nods_goal: u32,
    pub jammer_seized: bool,
    pub phase: EpisodePhase,
    /// Sticky Host line override while Solo Broadcast is live.
    pub host_line: Option<String>,
}

impl Default for SoloBroadcastEp0 {
    fn default() -> Self {
        Self {
            enabled: false,
            started: false,
            nods_cleared: 0,
            nods_goal: EP0_NODS_GOAL,
            jammer_seized: false,
            phase: EpisodePhase::Nods,
            host_line: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundState {
    Warmup,
    Active,
    Ended,
}

#[derive(Debug, Clone)]
pub struct MatchConfig {
    pub frag_limit: Option<u32>,
    pub time_limit_ticks: Option<u32>,
    pub warmup_ticks: u32,
    pub end_delay_ticks: u32,
    /// Tick into Active when Continuance compliance ping fires once (None = off).
    pub compliance_ping_ticks: Option<u32>,
    /// How long approved-lanes slow lasts after the ping.
    pub compliance_duration_ticks: u32,
    /// Tick into Active when Compliance Drone spawns once (None = off).
    pub boss_spawn_ticks: Option<u32>,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            frag_limit: Some(10),
            time_limit_ticks: Some(20 * 60 * 3),
            warmup_ticks: 20 * 2,
            end_delay_ticks: 20 * 8,
            // ~15s into Active so one round of play feels the pressure beat.
            compliance_ping_ticks: Some(20 * 15),
            compliance_duration_ticks: 20 * 6,
            // ~20s into Active: Continuance escalates with a killable drone.
            boss_spawn_ticks: Some(20 * 20),
        }
    }
}

/// What a mid-map pad grants on claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickupKind {
    Weapon(WeaponType),
    Health,
    Armor,
}

impl PickupKind {
    pub fn wire_name(self) -> &'static str {
        match self {
            PickupKind::Weapon(_) => "weapon",
            PickupKind::Health => "health",
            PickupKind::Armor => "armor",
        }
    }

    pub fn weapon(self) -> Option<WeaponType> {
        match self {
            PickupKind::Weapon(w) => Some(w),
            _ => None,
        }
    }
}

/// Authoritative mid-map pad (weapon / health / armor; Quake chase energy).
#[derive(Debug, Clone)]
pub struct ArenaPickup {
    pub id: String,
    pub kind: PickupKind,
    pub amount: i32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub available: bool,
    pub respawn_timer: Option<u32>,
}

impl ArenaPickup {
    fn to_state(&self) -> PickupState {
        let weapon = self
            .kind
            .weapon()
            .map(|w| w.name().to_string())
            .unwrap_or_default();
        let amount = match self.kind {
            PickupKind::Weapon(_) => None,
            PickupKind::Health | PickupKind::Armor => Some(self.amount),
        };
        PickupState {
            id: self.id.clone(),
            kind: self.kind.wire_name().to_string(),
            weapon,
            amount,
            x: self.x,
            y: self.y,
            z: self.z,
            available: self.available,
            respawn_in: if self.available {
                None
            } else {
                self.respawn_timer
            },
        }
    }

    fn respawn_ticks(&self) -> u32 {
        match self.kind {
            PickupKind::Weapon(_) => PICKUP_RESPAWN_TICKS,
            PickupKind::Health | PickupKind::Armor => HEALTH_PICKUP_RESPAWN_TICKS,
        }
    }
}

pub struct GameState {
    pub tick: u64,
    pub players: Vec<Player>,
    pub events: Vec<GameEvent>,
    pub bots: Vec<BotController>,
    pub scores: HashMap<Uuid, u32>,
    pub round_state: RoundState,
    pub round_ticks: u32,
    pub round_number: u32,
    pub config: MatchConfig,
    /// Cleared each tick; filled when weapons fire this tick.
    pub shot_results: Vec<ShotResult>,
    /// Remaining ticks of Continuance compliance slow (0 = none).
    pub compliance_ticks_left: u32,
    /// Whether this Active round already fired its compliance ping.
    pub compliance_fired: bool,
    /// Living Compliance Drone player id (None if not spawned or already down).
    pub boss_id: Option<Uuid>,
    /// Whether this Active round already spawned its drone.
    pub boss_spawned: bool,
    /// Mid-map pads: weapons, health, armor (Solo Scrap + MP).
    pub pickups: Vec<ArenaPickup>,
    /// Sticky Host line while RoundState::Ended (MVP podium bumper for mid-join).
    pub ended_host_line: Option<String>,
    /// Sticky MVP name while Ended (structured mid-join rehydrate).
    pub ended_mvp: Option<String>,
    /// Sticky MVP frag count while Ended.
    pub ended_mvp_frags: Option<u32>,
    /// Sticky Warmup / mid-join Host line naming dialed-in rule bots (None = default league line).
    pub roster_host_line: Option<String>,
    /// Dialed-in rule-bot callsigns for Warmup / RoundStart Host drama.
    pub roster_names: Vec<String>,
    /// Active Contested Frequency scrap layout.
    pub map: MapKind,
    /// When true, alternate map each start_round.
    pub map_rotate: bool,
    /// Fighters that just respawned and their remaining shield ticks.
    pub spawn_shields: HashMap<Uuid, u32>,
    /// Solo Broadcast Episode 0 (Calibration / Larak Lot). Off for MP.
    pub solo_broadcast: SoloBroadcastEp0,
}

pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub hp: i32,
    /// Scrap armor; absorbs damage before HP (0 on spawn/respawn).
    pub armor: i32,
    pub pending_action: Action,
    pub fire_cooldown: u32,
    pub respawn_timer: Option<u32>,
    pub just_fired: bool,
    pub role: Role,
    pub weapon: WeaponType,
    /// Tick of last successful speak (rate limit).
    pub last_speak_tick: Option<u64>,
    /// Continuance Compliance Drone (no respawn, distinct silhouette).
    pub is_boss: bool,
    /// Within-round killstreak (resets on death and round boundaries).
    pub killstreak: u32,
    /// Agent-set observe chip (never used for combat). Rule bots use BotController.
    pub display_behavior: Option<String>,
    /// Newest input sequence applied to this fighter, echoed in the Ack.
    pub last_input_seq: Option<u32>,
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_map(map: MapKind, map_rotate: bool) -> Self {
        Self {
            map,
            map_rotate,
            pickups: map.pickups(),
            ..Self::default()
        }
    }

    pub fn start_round(&mut self) {
        let previous_winner = if self.round_number > 0 {
            self.scores
                .iter()
                .max_by_key(|(_, &score)| score)
                .and_then(|(id, _)| self.players.iter().find(|p| p.id == *id))
                .map(|p| p.name.clone())
        } else {
            None
        };

        self.round_number += 1;
        self.round_state = RoundState::Active;
        self.round_ticks = 0;
        self.scores.clear();
        self.compliance_fired = false;
        self.compliance_ticks_left = 0;
        self.clear_boss();
        if self.solo_broadcast.enabled {
            self.solo_broadcast.nods_cleared = 0;
            self.solo_broadcast.jammer_seized = false;
            self.solo_broadcast.phase = EpisodePhase::Nods;
            self.solo_broadcast.started = false;
            self.solo_broadcast.host_line = Some(episode0_host_line_cold_open());
            self.relabel_rule_bots_as_nods();
        }
        if self.map_rotate && self.round_number > 1 {
            self.map = self.map.next();
            tracing::info!("Map rotate -> {} ({})", self.map.name(), self.map.id());
        }
        self.reset_pickups();
        self.ended_host_line = None;
        self.ended_mvp = None;
        self.ended_mvp_frags = None;

        for player in &mut self.players {
            self.scores.insert(player.id, 0);
            player.killstreak = 0;
        }

        let players: Vec<String> = self.players.iter().map(|p| p.name.clone()).collect();

        self.events.push(GameEvent::RoundStart {
            round_number: self.round_number,
            frag_limit: self.config.frag_limit,
            time_limit: self.config.time_limit_ticks.map(|t| t / 20),
            players,
            previous_winner,
            mode_name: default_mode_name(),
            playlist: default_playlist(),
            host_line: if self.solo_broadcast.enabled {
                episode0_host_line_cold_open()
            } else {
                round_open_host_line(self.map.name(), &self.roster_names)
            },
        });

        tracing::info!(
            "Round {} started (frag_limit: {:?}, time_limit: {:?}s)",
            self.round_number,
            self.config.frag_limit,
            self.config.time_limit_ticks.map(|t| t / 20)
        );
    }

    pub fn end_round(&mut self, reason: String) {
        // Dismiss a live drone before podium so Ended mid-join is not soft-prisoned
        // with compliance_drone pressure and MCP gets boss_down (killer null).
        self.wipe_boss_for_round_end();

        let (winner, winner_score) = self
            .scores
            .iter()
            .max_by_key(|(_, &score)| score)
            .and_then(|(id, score)| {
                self.players
                    .iter()
                    .find(|p| p.id == *id)
                    .map(|p| (p.name.clone(), *score))
            })
            .unzip();

        let mut final_scores: Vec<PlayerScore> = self
            .scores
            .iter()
            .filter_map(|(id, &score)| {
                self.players
                    .iter()
                    .find(|p| p.id == *id)
                    .map(|p| PlayerScore {
                        name: p.name.clone(),
                        score,
                    })
            })
            .collect();

        final_scores.sort_by_key(|a| std::cmp::Reverse(a.score));

        // MVP is top score / frags (same selection as winner).
        let mvp = winner.clone();
        let mvp_frags = winner_score;
        let host_line = match (&mvp, mvp_frags) {
            (Some(name), Some(frags)) => mvp_host_line(name, frags),
            _ => empty_mvp_host_line(),
        };
        self.ended_host_line = Some(host_line.clone());
        self.ended_mvp = mvp.clone();
        self.ended_mvp_frags = mvp_frags;

        self.round_state = RoundState::Ended;
        self.round_ticks = 0;
        for player in &mut self.players {
            player.killstreak = 0;
        }

        self.events.push(GameEvent::RoundEnd {
            winner: winner.clone(),
            reason: reason.clone(),
            final_scores,
            winner_score,
            mvp: mvp.clone(),
            mvp_frags,
            host_line: host_line.clone(),
        });

        tracing::info!(
            "Round {} ended: {} (mvp: {:?}, frags: {:?})",
            self.round_number,
            reason,
            mvp,
            mvp_frags
        );
    }

    pub fn add_player(&mut self, id: Uuid, name: String, role: Role) {
        let angle = (self.players.len() as f32) * (2.0 * PI / 8.0);
        let (sx, sz, yaw) = spawn_on_ring(self.map, angle);

        self.players.push(Player {
            id,
            name: name.clone(),
            x: sx,
            y: 1.5,
            z: sz,
            yaw,
            hp: PLAYER_MAX_HP,
            armor: 0,
            pending_action: Action::default(),
            fire_cooldown: 0,
            respawn_timer: None,
            just_fired: false,
            role,
            weapon: WeaponType::default(),
            last_speak_tick: None,
            is_boss: false,
            killstreak: 0,
            display_behavior: None,
            last_input_seq: None,
        });

        self.scores.entry(id).or_insert(0);
    }

    pub fn remove_player(&mut self, id: Uuid) {
        if let Some(player) = self.players.iter().find(|p| p.id == id) {
            if player.role == Role::Human {
                tracing::info!("Human player left, bots keep fighting");
            }
        }
        self.players.retain(|p| p.id != id);
        self.scores.remove(&id);
    }

    pub fn set_action(&mut self, id: Uuid, action: Action) {
        if let Some(player) = self.players.iter_mut().find(|p| p.id == id) {
            player.pending_action = action;
        }
    }

    /// Max Unicode scalars for an Agent display-behavior chip.
    pub const DISPLAY_BEHAVIOR_MAX_CHARS: usize = 32;

    /// Set observe-only behavior label for an Agent. Ignores humans, spectators,
    /// rule bots (players with a BotController), empty/overlong/control text.
    /// Never influences combat.
    pub fn set_display_behavior(&mut self, id: Uuid, raw: &str) -> bool {
        if self.bots.iter().any(|b| b.player_id == id) {
            return false;
        }
        let trimmed: String = raw
            .trim()
            .chars()
            .take(Self::DISPLAY_BEHAVIOR_MAX_CHARS + 1)
            .collect();
        if trimmed.is_empty() || trimmed.chars().count() > Self::DISPLAY_BEHAVIOR_MAX_CHARS {
            return false;
        }
        if trimmed.chars().any(|c| c.is_control()) {
            return false;
        }
        let Some(player) = self.players.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        if player.role != Role::Agent {
            return false;
        }
        player.display_behavior = Some(trimmed);
        true
    }

    /// One Ack per fighter whose client numbers its inputs. Built after a
    /// tick so the state it carries is the state that input produced.
    pub fn input_acks(&self) -> Vec<(Uuid, ServerMessage)> {
        self.players
            .iter()
            .filter(|p| p.role == Role::Human)
            .filter_map(|p| {
                p.last_input_seq.map(|seq| {
                    (
                        p.id,
                        ServerMessage::Ack {
                            seq,
                            tick: self.tick,
                            x: p.x,
                            z: p.z,
                            yaw: p.yaw,
                        },
                    )
                })
            })
            .collect()
    }

    pub fn tick(&mut self, dt: f32) {
        self.tick += 1;
        self.shot_results.clear();
        self.spawn_shields.retain(|_, ticks| {
            *ticks = ticks.saturating_sub(1);
            *ticks > 0
        });
        // Do not clear events here. Join/leave are pushed from the net loop between
        // ticks; clearing would drop them before main broadcasts take_events().
        // take_events() in the game loop is the drain.

        match self.round_state {
            RoundState::Warmup => {
                self.round_ticks += 1;
                self.maybe_warmup_rule_bot_taunts();
                self.tick_solo_broadcast();
                if self.round_ticks >= self.config.warmup_ticks {
                    self.start_round();
                }
                return;
            }
            RoundState::Active => {
                self.round_ticks += 1;
                self.tick_solo_broadcast();

                if self.compliance_ticks_left > 0 {
                    self.compliance_ticks_left -= 1;
                }
                if !self.compliance_fired {
                    if let Some(at) = self.config.compliance_ping_ticks {
                        if self.round_ticks >= at {
                            self.fire_compliance_ping();
                        }
                    }
                }
                if !self.boss_spawned && !self.solo_broadcast.enabled {
                    if let Some(at) = self.config.boss_spawn_ticks {
                        if self.round_ticks >= at {
                            self.spawn_compliance_drone();
                        }
                    }
                }

                if let Some(time_limit) = self.config.time_limit_ticks {
                    if self.round_ticks >= time_limit {
                        if self.solo_broadcast.enabled {
                            self.fail_episode("Calibration timed out".to_string());
                        } else {
                            self.end_round("Time limit reached".to_string());
                        }
                        return;
                    }
                }

                if let Some(frag_limit) = self.config.frag_limit {
                    if let Some(&max_score) = self.scores.values().max() {
                        if max_score >= frag_limit {
                            self.end_round("Frag limit reached".to_string());
                            return;
                        }
                    }
                }
            }
            RoundState::Ended => {
                self.round_ticks += 1;
                if self.round_ticks >= self.config.end_delay_ticks {
                    self.start_round();
                }
                return;
            }
        }

        let mut respawn_ids = Vec::new();
        let move_speed = if self.compliance_ticks_left > 0 {
            MOVE_SPEED * 0.5
        } else {
            MOVE_SPEED
        };

        for player in &mut self.players {
            player.just_fired = false;

            if player.fire_cooldown > 0 {
                player.fire_cooldown -= 1;
            }

            if let Some(timer) = player.respawn_timer.as_mut() {
                *timer = timer.saturating_sub(1);
                if *timer == 0 {
                    respawn_ids.push(player.id);
                }
                continue;
            }

            let action = &player.pending_action;

            if let Some(new_weapon) = action.weapon_swap {
                player.weapon = new_weapon;
            }

            if let Some(seq) = action.seq {
                player.last_input_seq = Some(seq);
            }

            // Client-owned yaw wins and is applied before the move, so the
            // fighter travels in the direction the client predicted for this
            // same input. Without it, the turn bits still turn at a fixed rate
            // after the move, which is how agents and older clients aim.
            let client_yaw = action.yaw.filter(|y| y.is_finite());
            if let Some(yaw) = client_yaw {
                player.yaw = crate::movement::normalize_yaw(yaw);
            }

            let mut dx = 0.0;
            let mut dz = 0.0;
            if action.forward {
                dx += player.yaw.cos();
                dz += player.yaw.sin();
            }
            if action.back {
                dx -= player.yaw.cos();
                dz -= player.yaw.sin();
            }
            if action.left {
                dx += (player.yaw - PI / 2.0).cos();
                dz += (player.yaw - PI / 2.0).sin();
            }
            if action.right {
                dx += (player.yaw + PI / 2.0).cos();
                dz += (player.yaw + PI / 2.0).sin();
            }

            let len = (dx * dx + dz * dz).sqrt();
            if len > 0.0 {
                dx /= len;
                dz /= len;
            }

            let old_x = player.x;
            let old_z = player.z;
            let new_x = old_x + dx * move_speed * dt;
            let new_z = old_z + dz * move_speed * dt;
            let (rx, rz) = resolve_move(self.map, old_x, old_z, new_x, new_z);
            player.x = rx;
            player.z = rz;

            if client_yaw.is_none() {
                if action.turn_left {
                    player.yaw -= TURN_SPEED * dt;
                }
                if action.turn_right {
                    player.yaw += TURN_SPEED * dt;
                }
            }

            while player.yaw < 0.0 {
                player.yaw += 2.0 * PI;
            }
            while player.yaw >= 2.0 * PI {
                player.yaw -= 2.0 * PI;
            }
        }

        // Authoritative look_at: snap yaw toward player_id (preferred) or world x/z.
        // Applied after movement/turn so agents can still strafe while locking aim.
        let look_intents: Vec<(Uuid, crate::protocol::LookAt)> = self
            .players
            .iter()
            .filter(|p| p.respawn_timer.is_none())
            .filter_map(|p| p.pending_action.look_at.clone().map(|look| (p.id, look)))
            .collect();
        for (aimer_id, look) in look_intents {
            let target_xz: Option<(f32, f32)> = if let Some(pid) = look.player_id {
                self.players
                    .iter()
                    .find(|p| p.id == pid && p.respawn_timer.is_none())
                    .map(|p| (p.x, p.z))
            } else if let (Some(x), Some(z)) = (look.x, look.z) {
                Some((x, z))
            } else {
                None
            };
            if let Some((tx, tz)) = target_xz {
                if let Some(aimer) = self.players.iter_mut().find(|p| p.id == aimer_id) {
                    let dx = tx - aimer.x;
                    let dz = tz - aimer.z;
                    if dx * dx + dz * dz > 1e-8 {
                        let mut yaw = dz.atan2(dx);
                        while yaw < 0.0 {
                            yaw += 2.0 * PI;
                        }
                        while yaw >= 2.0 * PI {
                            yaw -= 2.0 * PI;
                        }
                        aimer.yaw = yaw;
                    }
                }
            }
        }

        self.tick_pickups();

        let mut hits = Vec::new();
        for i in 0..self.players.len() {
            let player = &self.players[i];

            if player.respawn_timer.is_some() {
                continue;
            }

            if player.pending_action.fire && player.fire_cooldown == 0 {
                hits.push((i, self.check_hitscan(i)));
            }
        }

        for (shooter_idx, maybe_victim_idx) in hits {
            let weapon = self.players[shooter_idx].weapon;
            let damage = weapon.damage();
            let shooter_name = self.players[shooter_idx].name.clone();
            let shooter_id = self.players[shooter_idx].id;

            {
                let shooter = &mut self.players[shooter_idx];
                shooter.fire_cooldown = weapon.cooldown_ticks();
                shooter.just_fired = true;
            }

            if let Some(victim_idx) = maybe_victim_idx {
                let (target_id, target_name, target_hp_after, died, victim_was_boss, boss_id) = {
                    let victim = &mut self.players[victim_idx];
                    let target_id = victim.id;
                    let target_name = victim.name.clone();
                    let absorbed = damage.min(victim.armor);
                    victim.armor -= absorbed;
                    victim.hp -= damage - absorbed;
                    let target_hp_after = victim.hp;
                    let died = victim.hp <= 0;
                    let victim_was_boss = victim.is_boss;
                    if died {
                        // Victim streak dies with them; boss does not respawn.
                        victim.killstreak = 0;
                        if victim_was_boss {
                            victim.respawn_timer = None;
                        } else {
                            victim.respawn_timer = Some(RESPAWN_DELAY_TICKS);
                        }
                    }
                    (
                        target_id,
                        target_name,
                        target_hp_after,
                        died,
                        victim_was_boss,
                        target_id,
                    )
                };

                self.shot_results.push(ShotResult {
                    shooter_id,
                    shooter: shooter_name.clone(),
                    hit: true,
                    target_id: Some(target_id),
                    target: Some(target_name.clone()),
                    damage,
                    target_hp_after: Some(target_hp_after),
                });
                self.events.push(GameEvent::Hit {
                    shooter: shooter_name.clone(),
                    shooter_id,
                    target: target_name.clone(),
                    target_id,
                    damage,
                    target_hp_after,
                });

                if died {
                    *self.scores.entry(shooter_id).or_insert(0) += 1;
                    let killer_score = self.scores[&shooter_id];

                    self.events.push(GameEvent::Frag {
                        killer: shooter_name.clone(),
                        victim: target_name.clone(),
                        killer_score,
                    });

                    // Killer streak (victim already reset). Host callouts at 2/3/5.
                    let killer_streak = {
                        let killer = &mut self.players[shooter_idx];
                        killer.killstreak = killer.killstreak.saturating_add(1);
                        killer.killstreak
                    };
                    if let Some((tier, message)) =
                        killstreak_host_line(killer_streak, &shooter_name)
                    {
                        self.events.push(GameEvent::Killstreak {
                            player: shooter_name.clone(),
                            player_id: shooter_id,
                            streak: killer_streak,
                            tier,
                            message,
                        });
                    }

                    // Named rule bots: occasional Contested Frequency speak (off-tick).
                    if matches!(killer_streak, 2 | 3 | 5) {
                        self.maybe_rule_bot_taunt(
                            shooter_id,
                            BotTauntKind::Killstreak,
                            RULE_BOT_TAUNT_STREAK_PCT,
                        );
                    } else {
                        self.maybe_rule_bot_taunt(
                            shooter_id,
                            BotTauntKind::Frag,
                            RULE_BOT_TAUNT_FRAG_PCT,
                        );
                    }
                    self.maybe_rule_bot_taunt(
                        target_id,
                        BotTauntKind::Death,
                        RULE_BOT_TAUNT_DEATH_PCT,
                    );

                    if victim_was_boss {
                        let boss_msg = if self.solo_broadcast.enabled {
                            episode0_host_line_win()
                        } else {
                            boss_down_host_line()
                        };
                        self.events.push(GameEvent::BossDown {
                            name: target_name.clone(),
                            boss_id,
                            killer: Some(shooter_name.clone()),
                            message: boss_msg,
                        });
                        tracing::info!(
                            "BOSS DOWN: {} fragged {} (score: {})",
                            shooter_name,
                            target_name,
                            killer_score
                        );
                        if self.solo_broadcast.enabled
                            && self.solo_broadcast.phase == EpisodePhase::Auditor
                        {
                            self.complete_episode(
                                "Auditor down. Frequency stays unmetered.".to_string(),
                            );
                        }
                    } else {
                        self.note_nods_frag(shooter_id);
                        tracing::info!(
                            "FRAG: {} -> {} (score: {})",
                            shooter_name,
                            target_name,
                            killer_score
                        );
                    }
                }
            } else {
                self.shot_results.push(ShotResult {
                    shooter_id,
                    shooter: shooter_name,
                    hit: false,
                    target_id: None,
                    target: None,
                    damage: 0,
                    target_hp_after: None,
                });
            }
        }

        for id in respawn_ids {
            self.do_respawn(id);
        }

        self.reap_dead_boss();
    }

    fn check_hitscan(&self, shooter_idx: usize) -> Option<usize> {
        let shooter = &self.players[shooter_idx];
        let spread = shooter.weapon.spread_radians();
        let weapon_range = shooter.weapon.range_units().min(HITSCAN_RANGE);
        let ray_dx = shooter.yaw.cos();
        let ray_dz = shooter.yaw.sin();

        let mut closest_dist = weapon_range;
        let mut closest_idx = None;

        for (i, target) in self.players.iter().enumerate() {
            if i == shooter_idx || target.respawn_timer.is_some() {
                continue;
            }
            if self.spawn_shields.get(&target.id).is_some_and(|t| *t > 0) {
                continue;
            }

            let dx = target.x - shooter.x;
            let dz = target.z - shooter.z;
            let dist = (dx * dx + dz * dz).sqrt();

            if dist > closest_dist {
                continue;
            }

            let dot = dx * ray_dx + dz * ray_dz;
            if dot <= 0.0 {
                continue;
            }

            let target_angle = dz.atan2(dx);
            let mut angle_diff = target_angle - shooter.yaw;
            while angle_diff > PI {
                angle_diff -= 2.0 * PI;
            }
            while angle_diff < -PI {
                angle_diff += 2.0 * PI;
            }

            if angle_diff.abs() <= spread {
                let proj_dist = dot;
                let perp_x = dx - ray_dx * proj_dist;
                let perp_z = dz - ray_dz * proj_dist;
                let perp_dist = (perp_x * perp_x + perp_z * perp_z).sqrt();

                if perp_dist <= PLAYER_RADIUS * 2.0 {
                    if ray_blocked_by_cover(self.map, shooter.x, shooter.z, ray_dx, ray_dz, dist) {
                        continue;
                    }
                    closest_dist = dist;
                    closest_idx = Some(i);
                }
            }
        }

        closest_idx
    }

    /// Ring slot farthest from every living fighter, so a respawn never lands in a fight.
    fn farthest_spawn_angle(&self, player_id: Uuid) -> f32 {
        let others: Vec<(f32, f32)> = self
            .players
            .iter()
            .filter(|p| p.id != player_id && p.respawn_timer.is_none())
            .map(|p| (p.x, p.z))
            .collect();
        if others.is_empty() {
            return rand::random::<f32>() * 2.0 * PI;
        }
        let mut best_angle = 0.0;
        let mut best_gap = f32::MIN;
        for slot in 0..16 {
            let angle = slot as f32 * (PI / 8.0);
            let (sx, sz, _) = spawn_on_ring(self.map, angle);
            let nearest = others
                .iter()
                .map(|(ox, oz)| ((ox - sx).powi(2) + (oz - sz).powi(2)).sqrt())
                .fold(f32::MAX, f32::min);
            if nearest > best_gap {
                best_gap = nearest;
                best_angle = angle;
            }
        }
        best_angle
    }

    fn do_respawn(&mut self, player_id: Uuid) {
        let angle = self.farthest_spawn_angle(player_id);
        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            let (sx, sz, yaw) = spawn_on_ring(self.map, angle);

            player.x = sx;
            player.y = 1.5;
            player.z = sz;
            player.yaw = yaw;
            player.hp = PLAYER_MAX_HP;
            player.armor = 0;
            player.respawn_timer = None;
            player.fire_cooldown = 0;

            self.events.push(GameEvent::Respawn {
                player: player.name.clone(),
            });
            self.spawn_shields.insert(player_id, SPAWN_SHIELD_TICKS);
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        let round_time_left = if self.round_state == RoundState::Active {
            self.config
                .time_limit_ticks
                .map(|limit| (limit.saturating_sub(self.round_ticks)) / 20)
        } else if self.round_state == RoundState::Warmup {
            let remaining = self.config.warmup_ticks.saturating_sub(self.round_ticks);
            Some(remaining.div_ceil(20).max(1))
        } else {
            None
        };

        Snapshot {
            tick: self.tick,
            players: self
                .players
                .iter()
                .filter(|p| p.respawn_timer.is_none())
                .map(|p| {
                    let behavior = self
                        .bots
                        .iter()
                        .find(|b| b.player_id == p.id)
                        .map(|b| format!("{:?}", b.behavior))
                        .or_else(|| p.display_behavior.clone());

                    PlayerState {
                        id: p.id,
                        name: p.name.clone(),
                        x: p.x,
                        y: p.y,
                        z: p.z,
                        yaw: p.yaw,
                        hp: p.hp,
                        armor: p.armor,
                        just_fired: p.just_fired,
                        behavior,
                        score: *self.scores.get(&p.id).unwrap_or(&0),
                        weapon: p.weapon.name().to_string(),
                    }
                })
                .collect(),
            round_state: Some(format!("{:?}", self.round_state)),
            round_time_left,
            frag_limit: self.config.frag_limit,
            shot_results: self.shot_results.clone(),
            mode_name: MODE_NAME.to_string(),
            playlist: PLAYLIST_NAME.to_string(),
            pressure: if self.boss_id.is_some() {
                Some("compliance_drone".to_string())
            } else if self.compliance_ticks_left > 0 {
                Some("compliance".to_string())
            } else {
                None
            },
            host_line: if self.round_state == RoundState::Ended {
                self.ended_host_line
                    .clone()
                    .unwrap_or_else(default_host_line)
            } else if let Some(line) = self.solo_broadcast.host_line.as_ref() {
                line.clone()
            } else if self.boss_id.is_some() {
                boss_host_line()
            } else if self.compliance_ticks_left > 0 {
                compliance_host_line()
            } else if self.round_state == RoundState::Warmup {
                let remaining = self.config.warmup_ticks.saturating_sub(self.round_ticks);
                let secs = remaining.div_ceil(20).max(1);
                warmup_host_line(&self.display_map_name(), &self.roster_names, secs)
            } else {
                default_host_line()
            },
            mvp: if self.round_state == RoundState::Ended {
                self.ended_mvp.clone()
            } else {
                None
            },
            mvp_frags: if self.round_state == RoundState::Ended {
                self.ended_mvp_frags
            } else {
                None
            },
            pickups: self.pickups.iter().map(|p| p.to_state()).collect(),
            map_id: self.map.id(),
            map_name: self.display_map_name(),
            episode_id: if self.solo_broadcast.enabled {
                Some(EPISODE_ID_EP0.to_string())
            } else {
                None
            },
            episode_title: if self.solo_broadcast.enabled {
                Some(EPISODE_TITLE_EP0.to_string())
            } else {
                None
            },
            episode_objective: if self.solo_broadcast.enabled {
                Some(episode0_objective_chip())
            } else {
                None
            },
            episode_progress: if self.solo_broadcast.enabled {
                Some(self.episode_progress_chip())
            } else {
                None
            },
            episode_phase: if self.solo_broadcast.enabled {
                Some(self.solo_broadcast.phase.wire().to_string())
            } else {
                None
            },
        }
    }

    /// Refresh sticky Warmup Host roster line from current rule-bot display names.
    /// Clears when `names` is empty so mid-join falls back to the default league line.
    pub fn set_roster_host_line_from_names(&mut self, names: &[String]) {
        self.roster_names = names.to_vec();
        if names.is_empty() {
            self.roster_host_line = None;
        } else {
            self.roster_host_line = Some(roster_host_line(names));
        }
    }

    /// Enable Contested Frequency Solo Broadcast Episode 0 (Calibration / Larak Lot).
    /// Relabels rule bots as NODS, disables timed Compliance Drone (Auditor is gated),
    /// and arms episode chrome. Safe to call once at boot.
    pub fn enable_solo_broadcast_ep0(&mut self) {
        self.solo_broadcast = SoloBroadcastEp0 {
            enabled: true,
            started: false,
            nods_cleared: 0,
            nods_goal: EP0_NODS_GOAL,
            jammer_seized: false,
            phase: EpisodePhase::Nods,
            host_line: Some(episode0_host_line_cold_open()),
        };
        // Auditor spawns after jammer seize, not on a timed mid-round drone.
        self.config.boss_spawn_ticks = None;
        // Insanely-fun: time-to-first-frag under ~30s. Short Warmup, keep guns loud.
        self.config.warmup_ticks = 20; // 1s
                                       // Slightly earlier compliance ping as Continuance probe-van flavor.
        self.config.compliance_ping_ticks = Some(20 * 8);
        self.relabel_rule_bots_as_nods();
    }

    pub(crate) fn relabel_rule_bots_as_nods(&mut self) {
        let mut n = 0u32;
        for player in &mut self.players {
            if player.is_boss || player.role == Role::Human {
                continue;
            }
            if self.bots.iter().any(|b| b.player_id == player.id) {
                n += 1;
                player.name = format!("NODS-{n:02}");
            }
        }
        let names: Vec<String> = self
            .players
            .iter()
            .filter(|p| self.bots.iter().any(|b| b.player_id == p.id) && !p.is_boss)
            .map(|p| p.name.clone())
            .collect();
        self.set_roster_host_line_from_names(&names);
    }

    fn display_map_name(&self) -> String {
        if self.solo_broadcast.enabled {
            EPISODE_MAP_LARAK_LOT.to_string()
        } else {
            self.map.name().to_string()
        }
    }

    fn episode_progress_chip(&self) -> String {
        let sb = &self.solo_broadcast;
        let nods = format!(
            "NODS {}/{}",
            sb.nods_cleared.min(sb.nods_goal),
            sb.nods_goal
        );
        let jammer = if sb.jammer_seized {
            "JAMMER OK"
        } else if sb.phase == EpisodePhase::Jammer {
            "SEIZE JAMMER"
        } else {
            "JAMMER"
        };
        let auditor = match sb.phase {
            EpisodePhase::Auditor => "AUDITOR LIVE",
            EpisodePhase::Won => "AUDITOR DOWN",
            EpisodePhase::Failed => "FAILED",
            _ => "AUDITOR",
        };
        format!("{nods} | {jammer} | {auditor}")
    }

    pub(crate) fn maybe_start_episode(&mut self) {
        if !self.solo_broadcast.enabled || self.solo_broadcast.started {
            return;
        }
        self.solo_broadcast.started = true;
        self.solo_broadcast.host_line = Some(episode0_host_line_cold_open());
        self.events.push(GameEvent::EpisodeStart {
            id: EPISODE_ID_EP0.to_string(),
            title: EPISODE_TITLE_EP0.to_string(),
            objective: episode0_objective_chip(),
            host_line: episode0_host_line_cold_open(),
            map_name: EPISODE_MAP_LARAK_LOT.to_string(),
        });
        tracing::info!("Solo Broadcast Episode 0 started (Calibration / Larak Lot)");
    }

    pub(crate) fn note_nods_frag(&mut self, killer_id: Uuid) {
        if !self.solo_broadcast.enabled {
            return;
        }
        if self.solo_broadcast.phase != EpisodePhase::Nods {
            return;
        }
        let Some(killer) = self.players.iter().find(|p| p.id == killer_id) else {
            return;
        };
        if killer.role != Role::Human {
            return;
        }
        self.solo_broadcast.nods_cleared = self.solo_broadcast.nods_cleared.saturating_add(1);
        if self.solo_broadcast.nods_cleared == 1 {
            self.solo_broadcast.host_line = Some(episode0_host_line_nods());
        }
        if self.solo_broadcast.nods_cleared >= self.solo_broadcast.nods_goal {
            self.solo_broadcast.phase = EpisodePhase::Jammer;
            self.solo_broadcast.host_line = Some(episode0_host_line_jammer());
            tracing::info!("Episode 0: NODS cleared, jammer dish is live");
        }
    }

    pub(crate) fn try_seize_jammer(&mut self) {
        if !self.solo_broadcast.enabled || self.solo_broadcast.phase != EpisodePhase::Jammer {
            return;
        }
        let human_near = self.players.iter().any(|p| {
            p.role == Role::Human
                && p.respawn_timer.is_none()
                && (p.x * p.x + p.z * p.z).sqrt() <= EP0_JAMMER_RADIUS
        });
        if !human_near {
            return;
        }
        self.solo_broadcast.jammer_seized = true;
        self.solo_broadcast.phase = EpisodePhase::Auditor;
        self.solo_broadcast.host_line = Some(episode0_host_line_auditor());
        self.spawn_auditor();
        tracing::info!("Episode 0: jammer seized, Auditor inbound");
    }

    /// Continuance Auditor elite (clipboard shield). Reuses Compliance AI path.
    pub fn spawn_auditor(&mut self) -> Option<Uuid> {
        if self.boss_spawned || self.boss_id.is_some() {
            return None;
        }
        if self.round_state != RoundState::Active {
            return None;
        }
        let id = Uuid::new_v4();
        let name = AUDITOR_NAME.to_string();
        self.players.push(Player {
            id,
            name: name.clone(),
            x: 0.0,
            y: 2.2,
            z: 0.0,
            yaw: 0.0,
            hp: BOSS_MAX_HP,
            armor: 50,
            pending_action: Action::default(),
            fire_cooldown: 0,
            respawn_timer: None,
            just_fired: false,
            role: Role::Agent,
            weapon: WeaponType::Rail,
            last_speak_tick: None,
            is_boss: true,
            killstreak: 0,
            display_behavior: None,
            last_input_seq: None,
        });
        self.bots
            .push(BotController::new(id, BotBehavior::Compliance));
        self.scores.insert(id, 0);
        self.boss_id = Some(id);
        self.boss_spawned = true;
        self.events.push(GameEvent::BossSpawn {
            name: name.clone(),
            boss_id: id,
            message: episode0_host_line_auditor(),
            hp: BOSS_MAX_HP,
        });
        tracing::info!("Auditor spawned ({})", id);
        Some(id)
    }

    pub(crate) fn complete_episode(&mut self, reason: String) {
        if !self.solo_broadcast.enabled || self.solo_broadcast.phase == EpisodePhase::Won {
            return;
        }
        self.solo_broadcast.phase = EpisodePhase::Won;
        let host = episode0_host_line_win();
        self.solo_broadcast.host_line = Some(host.clone());
        self.events.push(GameEvent::EpisodeComplete {
            id: EPISODE_ID_EP0.to_string(),
            reason: reason.clone(),
            host_line: host,
            unlock_teaser: episode0_unlock_teaser(),
        });
        tracing::info!("Episode 0 complete: {reason}");
        self.end_round(reason);
    }

    pub(crate) fn fail_episode(&mut self, reason: String) {
        if !self.solo_broadcast.enabled {
            self.end_round(reason);
            return;
        }
        if self.solo_broadcast.phase == EpisodePhase::Won
            || self.solo_broadcast.phase == EpisodePhase::Failed
        {
            return;
        }
        self.solo_broadcast.phase = EpisodePhase::Failed;
        let host = episode0_host_line_fail();
        self.solo_broadcast.host_line = Some(host.clone());
        self.events.push(GameEvent::EpisodeFail {
            id: EPISODE_ID_EP0.to_string(),
            reason: reason.clone(),
            host_line: host,
        });
        tracing::info!("Episode 0 fail: {reason}");
        self.end_round("Citizen Handle assigned".to_string());
    }

    pub(crate) fn tick_solo_broadcast(&mut self) {
        if !self.solo_broadcast.enabled {
            return;
        }
        self.maybe_start_episode();
        if self.round_state == RoundState::Active {
            self.try_seize_jammer();
        }
    }

    fn reset_pickups(&mut self) {
        self.pickups = self.map.pickups();
    }

    /// Decrement pad respawn timers and claim available pads on touch.
    fn tick_pickups(&mut self) {
        for pad in &mut self.pickups {
            if let Some(timer) = pad.respawn_timer.as_mut() {
                *timer = timer.saturating_sub(1);
                if *timer == 0 {
                    pad.respawn_timer = None;
                    pad.available = true;
                }
            }
        }

        // Collect claims (player_id, pad index) without holding dual borrows.
        let mut claims: Vec<(Uuid, usize)> = Vec::new();
        for player in &self.players {
            if player.respawn_timer.is_some() || player.is_boss {
                continue;
            }
            for (pi, pad) in self.pickups.iter().enumerate() {
                if !pad.available {
                    continue;
                }
                let useful = match pad.kind {
                    PickupKind::Weapon(_) => true,
                    PickupKind::Health => player.hp < PLAYER_MAX_HP,
                    PickupKind::Armor => player.armor < PLAYER_MAX_ARMOR,
                };
                if !useful {
                    continue;
                }
                let dx = player.x - pad.x;
                let dz = player.z - pad.z;
                if dx * dx + dz * dz <= PICKUP_CLAIM_RADIUS * PICKUP_CLAIM_RADIUS {
                    claims.push((player.id, pi));
                    break; // one pad per player per tick
                }
            }
        }

        for (player_id, pad_idx) in claims {
            let pad = &mut self.pickups[pad_idx];
            if !pad.available {
                continue; // raced / already taken this tick
            }
            let kind = pad.kind;
            let amount = pad.amount;
            let pickup_id = pad.id.clone();
            let respawn = pad.respawn_ticks();
            pad.available = false;
            pad.respawn_timer = Some(respawn);

            let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) else {
                continue;
            };
            let (weapon_wire, amount_wire, label) = match kind {
                PickupKind::Weapon(w) => {
                    player.weapon = w;
                    (w.name().to_string(), None, w.name().to_string())
                }
                PickupKind::Health => {
                    let before = player.hp;
                    player.hp = (player.hp + amount).min(PLAYER_MAX_HP);
                    let gained = player.hp - before;
                    (String::new(), Some(gained), format!("+{} HP", gained))
                }
                PickupKind::Armor => {
                    let before = player.armor;
                    player.armor = (player.armor + amount).min(PLAYER_MAX_ARMOR);
                    let gained = player.armor - before;
                    (String::new(), Some(gained), format!("+{} armor", gained))
                }
            };
            let name = player.name.clone();
            self.events.push(GameEvent::Pickup {
                player: name.clone(),
                player_id,
                kind: kind.wire_name().to_string(),
                weapon: weapon_wire,
                amount: amount_wire,
                pickup_id: pickup_id.clone(),
            });
            tracing::info!("PICKUP: {} claimed {} ({})", name, label, pickup_id);
        }
    }

    fn fire_compliance_ping(&mut self) {
        self.compliance_fired = true;
        self.compliance_ticks_left = self.config.compliance_duration_ticks;
        self.events.push(GameEvent::CompliancePing {
            message: compliance_host_line(),
            duration_ticks: self.config.compliance_duration_ticks,
        });
        tracing::info!(
            "Compliance ping fired (duration {} ticks)",
            self.config.compliance_duration_ticks
        );
    }

    /// Drop the Continuance Compliance Drone into the Active scrap (once per round).
    pub fn spawn_compliance_drone(&mut self) -> Option<Uuid> {
        if self.boss_spawned || self.boss_id.is_some() {
            return None;
        }
        if self.round_state != RoundState::Active {
            return None;
        }
        let id = Uuid::new_v4();
        self.players.push(Player {
            id,
            name: BOSS_NAME.to_string(),
            x: 0.0,
            y: 2.2,
            z: 0.0,
            yaw: 0.0,
            hp: BOSS_MAX_HP,
            armor: 0,
            pending_action: Action::default(),
            fire_cooldown: 0,
            respawn_timer: None,
            just_fired: false,
            role: Role::Agent,
            weapon: WeaponType::Rail,
            last_speak_tick: None,
            is_boss: true,
            killstreak: 0,
            display_behavior: None,
            last_input_seq: None,
        });
        self.bots
            .push(BotController::new(id, BotBehavior::Compliance));
        self.scores.insert(id, 0);
        self.boss_id = Some(id);
        self.boss_spawned = true;
        self.events.push(GameEvent::BossSpawn {
            name: BOSS_NAME.to_string(),
            boss_id: id,
            message: boss_host_line(),
            hp: BOSS_MAX_HP,
        });
        tracing::info!("Compliance Drone spawned ({})", id);
        Some(id)
    }

    /// Round-end dismiss: emit boss_down (no killer) then scrub the drone.
    fn wipe_boss_for_round_end(&mut self) {
        let Some(id) = self.boss_id else {
            return;
        };
        let alive = self
            .players
            .iter()
            .any(|p| p.id == id && p.hp > 0 && p.respawn_timer.is_none());
        if !alive {
            self.clear_boss();
            return;
        }
        let name = self
            .players
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| BOSS_NAME.to_string());
        self.events.push(GameEvent::BossDown {
            name,
            boss_id: id,
            killer: None,
            message: boss_round_wipe_host_line(),
        });
        self.clear_boss();
        tracing::info!("Boss wiped on round end ({})", id);
    }

    fn clear_boss(&mut self) {
        if let Some(id) = self.boss_id.take() {
            self.players.retain(|p| p.id != id);
            self.bots.retain(|b| b.player_id != id);
            self.scores.remove(&id);
        }
        self.boss_spawned = false;
        // Also scrub any orphaned boss players (round recycle safety).
        let orphan_ids: Vec<Uuid> = self
            .players
            .iter()
            .filter(|p| p.is_boss)
            .map(|p| p.id)
            .collect();
        for id in orphan_ids {
            self.players.retain(|p| p.id != id);
            self.bots.retain(|b| b.player_id != id);
            self.scores.remove(&id);
        }
    }

    fn reap_dead_boss(&mut self) {
        let Some(boss_id) = self.boss_id else {
            return;
        };
        let dead = self
            .players
            .iter()
            .find(|p| p.id == boss_id)
            .map(|p| p.hp <= 0)
            .unwrap_or(true);
        if !dead {
            return;
        }
        self.players.retain(|p| p.id != boss_id);
        self.bots.retain(|b| b.player_id != boss_id);
        self.scores.remove(&boss_id);
        self.boss_id = None;
    }

    /// True when `id` is a named scrap rule bot (not Continuance Compliance).
    pub fn is_named_rule_bot(&self, id: Uuid) -> bool {
        self.bots
            .iter()
            .any(|b| b.player_id == id && b.behavior != BotBehavior::Compliance)
    }

    fn taunt_roll(tick: u64, player_id: Uuid, salt: u64) -> u64 {
        tick.wrapping_mul(2654435761)
            .wrapping_add(player_id.as_u128() as u64)
            .wrapping_add(salt)
            % 100
    }

    /// Probabilistic Contested Frequency speak for a named rule bot.
    /// Silent on miss / RateLimited / Rejected (no Error unicast spam for bots).
    pub fn maybe_rule_bot_taunt(&mut self, player_id: Uuid, kind: BotTauntKind, chance_pct: u64) {
        if !self.is_named_rule_bot(player_id) {
            return;
        }
        let salt = match kind {
            BotTauntKind::Frag => 11,
            BotTauntKind::Death => 22,
            BotTauntKind::Killstreak => 33,
            BotTauntKind::Warmup => 44,
        };
        if Self::taunt_roll(self.tick, player_id, salt) >= chance_pct {
            return;
        }
        let _ = self.try_rule_bot_taunt(player_id, kind);
    }

    /// Always attempt a callsign-flavored speak (still respects SPEAK_COOLDOWN).
    pub fn try_rule_bot_taunt(&mut self, player_id: Uuid, kind: BotTauntKind) -> SpeakOutcome {
        if !self.is_named_rule_bot(player_id) {
            return SpeakOutcome::Rejected;
        }
        let name = match self.players.iter().find(|p| p.id == player_id) {
            Some(p) => p.name.clone(),
            None => return SpeakOutcome::Rejected,
        };
        let salt = self.tick ^ (player_id.as_u128() as u64);
        let line = rule_bot_taunt_line(&name, kind, salt);
        self.try_speak(player_id, &line)
    }

    /// During Warmup, occasionally one dialed-in rule bot speaks a tuning-in line.
    fn maybe_warmup_rule_bot_taunts(&mut self) {
        if self.round_ticks == 0 || !self.round_ticks.is_multiple_of(RULE_BOT_WARMUP_TAUNT_EVERY) {
            return;
        }
        let rule_bots: Vec<Uuid> = self
            .bots
            .iter()
            .filter(|b| b.behavior != BotBehavior::Compliance)
            .map(|b| b.player_id)
            .collect();
        if rule_bots.is_empty() {
            return;
        }
        let idx = (self.tick as usize) % rule_bots.len();
        self.maybe_rule_bot_taunt(
            rule_bots[idx],
            BotTauntKind::Warmup,
            RULE_BOT_TAUNT_WARMUP_PCT,
        );
    }

    /// Validate and emit an off-tick speak event.
    pub fn try_speak(&mut self, player_id: Uuid, raw_text: &str) -> SpeakOutcome {
        let trimmed: String = raw_text.trim().chars().take(SPEAK_MAX_CHARS + 1).collect();
        if trimmed.is_empty() {
            return SpeakOutcome::Rejected;
        }
        if trimmed.chars().count() > SPEAK_MAX_CHARS {
            return SpeakOutcome::Rejected;
        }
        if trimmed.chars().any(|c| c.is_control()) {
            return SpeakOutcome::Rejected;
        }

        let tick = self.tick;
        let player = match self.players.iter_mut().find(|p| p.id == player_id) {
            Some(p) => p,
            None => return SpeakOutcome::Rejected,
        };

        if let Some(last) = player.last_speak_tick {
            if tick.saturating_sub(last) < SPEAK_COOLDOWN_TICKS {
                return SpeakOutcome::RateLimited;
            }
        }

        let name = player.name.clone();
        player.last_speak_tick = Some(tick);
        self.events.push(GameEvent::Speak {
            player: name,
            player_id,
            text: trimmed,
        });
        SpeakOutcome::Sent
    }

    pub fn take_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn push_event(&mut self, event: GameEvent) {
        self.events.push(event);
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            tick: 0,
            players: Vec::new(),
            events: Vec::new(),
            bots: Vec::new(),
            scores: HashMap::new(),
            round_state: RoundState::Warmup,
            round_ticks: 0,
            round_number: 0,
            config: MatchConfig::default(),
            shot_results: Vec::new(),
            compliance_ticks_left: 0,
            compliance_fired: false,
            boss_id: None,
            boss_spawned: false,
            pickups: MapKind::ArenaDuel.pickups(),
            ended_host_line: None,
            ended_mvp: None,
            ended_mvp_frags: None,
            roster_host_line: None,
            roster_names: Vec::new(),
            map: MapKind::ArenaDuel,
            map_rotate: false,
            spawn_shields: HashMap::new(),
            solo_broadcast: SoloBroadcastEp0::default(),
        }
    }
}

#[derive(Clone)]
pub struct BotController {
    pub player_id: Uuid,
    pub behavior: BotBehavior,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotBehavior {
    Aggressive,
    Defensive,
    Flanker,
    Balanced,
    /// Continuance Compliance Drone: mid-range Rail enforcer.
    Compliance,
}

impl BotController {
    pub fn new(player_id: Uuid, behavior: BotBehavior) -> Self {
        Self {
            player_id,
            behavior,
        }
    }

    pub fn update(&self, state: &GameState) -> Action {
        let Some(bot) = state.players.iter().find(|p| p.id == self.player_id) else {
            return Action::default();
        };

        if bot.respawn_timer.is_some() {
            return Action::default();
        }

        let mut nearest_dist = f32::MAX;
        let mut nearest_target: Option<&Player> = None;

        for target in &state.players {
            if target.id == self.player_id || target.respawn_timer.is_some() {
                continue;
            }

            let dx = target.x - bot.x;
            let dz = target.z - bot.z;
            let dist = (dx * dx + dz * dz).sqrt();

            if dist < nearest_dist {
                nearest_dist = dist;
                nearest_target = Some(target);
            }
        }

        let Some(target) = nearest_target else {
            return Action::default();
        };

        let dx = target.x - bot.x;
        let dz = target.z - bot.z;
        let target_angle = dz.atan2(dx);

        let mut angle_diff = target_angle - bot.yaw;
        while angle_diff > PI {
            angle_diff -= 2.0 * PI;
        }
        while angle_diff < -PI {
            angle_diff += 2.0 * PI;
        }

        let mut action = Action::default();

        // Seek a role pad when Flechette and a better weapon is nearby.
        // Prefer Scatter when the fight is close; Rail when it is long.
        if bot.weapon == WeaponType::Flechette && self.behavior != BotBehavior::Compliance {
            let want_scatter = nearest_dist < 12.0;
            let mut best: Option<(f32, f32, f32, WeaponType)> = None;
            for pad in &state.pickups {
                let Some(pad_weapon) = pad.kind.weapon() else {
                    continue;
                };
                if !pad.available || pad_weapon == WeaponType::Flechette {
                    continue;
                }
                let pdx = pad.x - bot.x;
                let pdz = pad.z - bot.z;
                let pdist = (pdx * pdx + pdz * pdz).sqrt();
                if pdist < 28.0 {
                    // Soft preference: matching role is closer in score space.
                    let role_bonus = match (want_scatter, pad_weapon) {
                        (true, WeaponType::Scatter) => 0.0,
                        (false, WeaponType::Rail) => 0.0,
                        _ => 6.0,
                    };
                    let score = pdist + role_bonus;
                    let take = match best {
                        Some((s, _, _, _)) => score < s,
                        None => true,
                    };
                    if take {
                        best = Some((score, pad.x, pad.z, pad_weapon));
                    }
                }
            }
            if let Some((_, px, pz, _)) = best {
                let pdx = px - bot.x;
                let pdz = pz - bot.z;
                let pdist = (pdx * pdx + pdz * pdz).sqrt();
                if nearest_dist > 10.0 || pdist < nearest_dist * 0.7 {
                    let pad_angle = pdz.atan2(pdx);
                    let mut pad_diff = pad_angle - bot.yaw;
                    while pad_diff > PI {
                        pad_diff -= 2.0 * PI;
                    }
                    while pad_diff < -PI {
                        pad_diff += 2.0 * PI;
                    }
                    if pad_diff.abs() > 0.2 {
                        if pad_diff > 0.0 {
                            action.turn_right = true;
                        } else {
                            action.turn_left = true;
                        }
                    }
                    action.forward = true;
                    if pad_diff.abs() < 0.5 && nearest_dist < 18.0 {
                        action.fire = true;
                    }
                    return action;
                }
            }
        }

        // Hold the role lane for the weapon in hand.
        let (prefer_min, prefer_max) = bot.weapon.preferred_range();
        let fire_range = bot.weapon.range_units() * 0.95;
        let aim_slack = match bot.weapon {
            WeaponType::Rail => 0.22,
            WeaponType::Scatter => 0.55,
            WeaponType::Flechette => 0.40,
        };

        match self.behavior {
            BotBehavior::Aggressive => {
                // Rush toward preferred band; Scatter push-in, Rail less so.
                if angle_diff.abs() > 0.2 {
                    if angle_diff > 0.0 {
                        action.turn_right = true;
                    } else {
                        action.turn_left = true;
                    }
                }
                if nearest_dist > prefer_min {
                    action.forward = true;
                } else if nearest_dist < prefer_min * 0.6 && bot.weapon != WeaponType::Scatter {
                    action.back = true;
                } else {
                    action.forward = true;
                }
                if angle_diff.abs() < aim_slack && nearest_dist < fire_range {
                    action.fire = true;
                }
            }

            BotBehavior::Defensive => {
                // Hold preferred band, strafe, precise shots (Rail-friendly).
                if angle_diff.abs() > 0.15 {
                    if angle_diff > 0.0 {
                        action.turn_right = true;
                    } else {
                        action.turn_left = true;
                    }
                }

                if nearest_dist < prefer_min {
                    action.back = true;
                } else if nearest_dist > prefer_max {
                    action.forward = true;
                } else if (state.tick % 40) < 20 {
                    action.left = true;
                } else {
                    action.right = true;
                }

                if angle_diff.abs() < aim_slack.min(0.3) && nearest_dist < fire_range {
                    action.fire = true;
                }
            }

            BotBehavior::Flanker => {
                if angle_diff.abs() > 0.25 {
                    if angle_diff > 0.0 {
                        action.turn_right = true;
                    } else {
                        action.turn_left = true;
                    }
                }

                if nearest_dist > prefer_max {
                    action.forward = true;
                } else {
                    action.forward = nearest_dist > prefer_min;
                    if (state.tick % 60) < 30 {
                        action.left = true;
                        action.turn_left = true;
                    } else {
                        action.right = true;
                        action.turn_right = true;
                    }
                }

                if angle_diff.abs() < aim_slack && nearest_dist < fire_range {
                    action.fire = true;
                }
            }

            BotBehavior::Balanced => {
                if angle_diff.abs() > 0.3 {
                    if angle_diff > 0.0 {
                        action.turn_right = true;
                    } else {
                        action.turn_left = true;
                    }
                }

                if nearest_dist > prefer_min {
                    action.forward = true;
                } else if nearest_dist < prefer_min * 0.7 {
                    action.back = true;
                }

                if angle_diff.abs() < aim_slack && nearest_dist < fire_range {
                    action.fire = true;
                }
            }

            BotBehavior::Compliance => {
                // Mid-range Rail enforcer: hold orbit, precise shots.
                if angle_diff.abs() > 0.12 {
                    if angle_diff > 0.0 {
                        action.turn_right = true;
                    } else {
                        action.turn_left = true;
                    }
                }
                if nearest_dist < prefer_min {
                    action.back = true;
                } else if nearest_dist > prefer_max {
                    action.forward = true;
                } else if (state.tick % 50) < 25 {
                    action.left = true;
                } else {
                    action.right = true;
                }
                if angle_diff.abs() < 0.25 && nearest_dist < fire_range {
                    action.fire = true;
                }
            }
        }

        action
    }
}
