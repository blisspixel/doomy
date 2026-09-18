use crate::protocol::{
    boss_down_host_line, boss_host_line, compliance_host_line, default_host_line,
    default_mode_name, default_playlist, Action, GameEvent, PickupState, PlayerScore, PlayerState,
    Role, ShotResult, Snapshot, WeaponType, BOSS_NAME, MODE_NAME, PLAYLIST_NAME,
};
use std::collections::HashMap;
use std::f32::consts::PI;
use uuid::Uuid;

const MOVE_SPEED: f32 = 5.0;
const TURN_SPEED: f32 = 2.0;
const ARENA_SIZE: f32 = 50.0;
const PLAYER_RADIUS: f32 = 0.5;
const RESPAWN_DELAY_TICKS: u32 = 60;
const HITSCAN_RANGE: f32 = 100.0;
const PLAYER_MAX_HP: i32 = 100;
/// Max Unicode scalars in a speak/taunt line (after trim).
pub const SPEAK_MAX_CHARS: usize = 80;
/// Min ticks between successful speaks for one player (~3s at 20 Hz).
pub const SPEAK_COOLDOWN_TICKS: u64 = 60;
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

/// Axis-aligned scrap solid in XZ (Godot props mirrored for authoritative cover).
#[derive(Debug, Clone, Copy)]
struct Aabb2 {
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

    const fn expand(self, r: f32) -> Self {
        Self {
            min_x: self.min_x - r,
            max_x: self.max_x + r,
            min_z: self.min_z - r,
            max_z: self.max_z + r,
        }
    }

    fn contains(self, x: f32, z: f32) -> bool {
        x >= self.min_x && x <= self.max_x && z >= self.min_z && z <= self.max_z
    }
}

/// Scrap chokes matching `client/scenes/arena.tscn` (pillars, low walls, crates).
fn arena_obstacles() -> [Aabb2; 19] {
    [
        // Pillars at (±7, ±7), mesh 2.5x2.5
        Aabb2::from_center(7.0, -7.0, 1.25, 1.25),
        Aabb2::from_center(-7.0, -7.0, 1.25, 1.25),
        Aabb2::from_center(7.0, 7.0, 1.25, 1.25),
        Aabb2::from_center(-7.0, 7.0, 1.25, 1.25),
        // Low walls N/S/E/W (half 4.0 along long axis, 0.4 thick)
        Aabb2::from_center(0.0, -10.0, 4.0, 0.4),
        Aabb2::from_center(0.0, 10.0, 4.0, 0.4),
        Aabb2::from_center(10.0, 0.0, 0.4, 4.0),
        Aabb2::from_center(-10.0, 0.0, 0.4, 4.0),
        // Crates (mesh 2x2 xz): existing + flank clusters
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
    ]
}

fn circle_blocked(x: f32, z: f32) -> bool {
    for obs in arena_obstacles() {
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
fn resolve_move(old_x: f32, old_z: f32, new_x: f32, new_z: f32) -> (f32, f32) {
    let (nx, nz) = clamp_arena(new_x, new_z);
    if !circle_blocked(nx, nz) {
        return (nx, nz);
    }
    let (sx, _) = clamp_arena(new_x, old_z);
    if !circle_blocked(sx, old_z) {
        return (sx, old_z);
    }
    let (_, sz) = clamp_arena(old_x, new_z);
    if !circle_blocked(old_x, sz) {
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

fn ray_blocked_by_cover(ox: f32, oz: f32, dx: f32, dz: f32, max_dist: f32) -> bool {
    for obs in arena_obstacles() {
        if let Some(t) = ray_aabb_hit(ox, oz, dx, dz, obs) {
            if t < max_dist {
                return true;
            }
        }
    }
    false
}

fn spawn_on_ring(angle: f32) -> (f32, f32, f32) {
    let spawn_radius = ARENA_SIZE * 0.3;
    let mut a = angle;
    for _ in 0..16 {
        let x = a.cos() * spawn_radius;
        let z = a.sin() * spawn_radius;
        if !circle_blocked(x, z) {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundState {
    Warmup,
    Active,
    Ended,
}

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
            end_delay_ticks: 20 * 5,
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

fn default_arena_pickups() -> Vec<ArenaPickup> {
    vec![
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
    ]
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
}

impl GameState {
    pub fn new() -> Self {
        Self::default()
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
        self.reset_pickups();

        for player in &mut self.players {
            self.scores.insert(player.id, 0);
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
            host_line: default_host_line(),
        });

        tracing::info!(
            "Round {} started (frag_limit: {:?}, time_limit: {:?}s)",
            self.round_number,
            self.config.frag_limit,
            self.config.time_limit_ticks.map(|t| t / 20)
        );
    }

    pub fn end_round(&mut self, reason: String) {
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

        self.round_state = RoundState::Ended;
        self.round_ticks = 0;

        self.events.push(GameEvent::RoundEnd {
            winner: winner.clone(),
            reason: reason.clone(),
            final_scores,
            winner_score,
        });

        tracing::info!(
            "Round {} ended: {} (winner: {:?}, score: {:?})",
            self.round_number,
            reason,
            winner,
            winner_score
        );
    }

    pub fn add_player(&mut self, id: Uuid, name: String, role: Role) {
        let angle = (self.players.len() as f32) * (2.0 * PI / 8.0);
        let (sx, sz, yaw) = spawn_on_ring(angle);

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

    pub fn tick(&mut self, dt: f32) {
        self.tick += 1;
        self.shot_results.clear();
        // Do not clear events here. Join/leave are pushed from the net loop between
        // ticks; clearing would drop them before main broadcasts take_events().
        // take_events() in the game loop is the drain.

        match self.round_state {
            RoundState::Warmup => {
                self.round_ticks += 1;
                if self.round_ticks >= self.config.warmup_ticks {
                    self.start_round();
                }
                return;
            }
            RoundState::Active => {
                self.round_ticks += 1;

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
                if !self.boss_spawned {
                    if let Some(at) = self.config.boss_spawn_ticks {
                        if self.round_ticks >= at {
                            self.spawn_compliance_drone();
                        }
                    }
                }

                if let Some(time_limit) = self.config.time_limit_ticks {
                    if self.round_ticks >= time_limit {
                        self.end_round("Time limit reached".to_string());
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
            let (rx, rz) = resolve_move(old_x, old_z, new_x, new_z);
            player.x = rx;
            player.z = rz;

            if action.turn_left {
                player.yaw -= TURN_SPEED * dt;
            }
            if action.turn_right {
                player.yaw += TURN_SPEED * dt;
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
                let victim = &mut self.players[victim_idx];
                let target_id = victim.id;
                let target_name = victim.name.clone();
                let absorbed = damage.min(victim.armor);
                victim.armor -= absorbed;
                victim.hp -= damage - absorbed;
                let target_hp_after = victim.hp;

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

                if victim.hp <= 0 {
                    let victim_was_boss = victim.is_boss;
                    if victim_was_boss {
                        // Boss does not respawn; removed after this hit batch.
                        victim.respawn_timer = None;
                    } else {
                        victim.respawn_timer = Some(RESPAWN_DELAY_TICKS);
                    }

                    *self.scores.entry(shooter_id).or_insert(0) += 1;
                    let killer_score = self.scores[&shooter_id];

                    self.events.push(GameEvent::Frag {
                        killer: shooter_name.clone(),
                        victim: target_name.clone(),
                        killer_score,
                    });
                    if victim_was_boss {
                        let boss_id = victim.id;
                        self.events.push(GameEvent::BossDown {
                            name: target_name.clone(),
                            boss_id,
                            killer: Some(shooter_name.clone()),
                            message: boss_down_host_line(),
                        });
                        tracing::info!(
                            "BOSS DOWN: {} fragged {} (score: {})",
                            shooter_name,
                            target_name,
                            killer_score
                        );
                    } else {
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
        let ray_dx = shooter.yaw.cos();
        let ray_dz = shooter.yaw.sin();

        let mut closest_dist = HITSCAN_RANGE;
        let mut closest_idx = None;

        for (i, target) in self.players.iter().enumerate() {
            if i == shooter_idx || target.respawn_timer.is_some() {
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
                    if ray_blocked_by_cover(shooter.x, shooter.z, ray_dx, ray_dz, dist) {
                        continue;
                    }
                    closest_dist = dist;
                    closest_idx = Some(i);
                }
            }
        }

        closest_idx
    }

    fn do_respawn(&mut self, player_id: Uuid) {
        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            let angle = rand::random::<f32>() * 2.0 * PI;
            let (sx, sz, yaw) = spawn_on_ring(angle);

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
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        let round_time_left = if self.round_state == RoundState::Active {
            self.config
                .time_limit_ticks
                .map(|limit| (limit.saturating_sub(self.round_ticks)) / 20)
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
                        .map(|b| format!("{:?}", b.behavior));

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
            host_line: if self.boss_id.is_some() {
                boss_host_line()
            } else if self.compliance_ticks_left > 0 {
                compliance_host_line()
            } else {
                default_host_line()
            },
            pickups: self.pickups.iter().map(|p| p.to_state()).collect(),
        }
    }

    fn reset_pickups(&mut self) {
        self.pickups = default_arena_pickups();
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
            pickups: default_arena_pickups(),
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

        // Quake chase: Flechette bots divert toward a live Rail/Scatter pad.
        if bot.weapon == WeaponType::Flechette && self.behavior != BotBehavior::Compliance {
            let mut best: Option<(f32, f32, f32)> = None; // dist, x, z
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
                    let take = match best {
                        Some((d, _, _)) => pdist < d,
                        None => true,
                    };
                    if take {
                        best = Some((pdist, pad.x, pad.z));
                    }
                }
            }
            if let Some((pdist, px, pz)) = best {
                if nearest_dist > 10.0 || pdist < nearest_dist * 0.7 {
                    let pdx = px - bot.x;
                    let pdz = pz - bot.z;
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

        match self.behavior {
            BotBehavior::Aggressive => {
                // Always chase, fire when close
                if angle_diff.abs() > 0.2 {
                    if angle_diff > 0.0 {
                        action.turn_right = true;
                    } else {
                        action.turn_left = true;
                    }
                }
                action.forward = true;
                if angle_diff.abs() < 0.6 && nearest_dist < 35.0 {
                    action.fire = true;
                }
            }

            BotBehavior::Defensive => {
                // Keep distance, strafe, precise shooting
                if angle_diff.abs() > 0.15 {
                    if angle_diff > 0.0 {
                        action.turn_right = true;
                    } else {
                        action.turn_left = true;
                    }
                }

                if nearest_dist < 8.0 {
                    action.back = true;
                } else if nearest_dist > 15.0 {
                    action.forward = true;
                } else {
                    // Strafe at optimal range
                    if (state.tick % 40) < 20 {
                        action.left = true;
                    } else {
                        action.right = true;
                    }
                }

                if angle_diff.abs() < 0.3 && nearest_dist < 25.0 {
                    action.fire = true;
                }
            }

            BotBehavior::Flanker => {
                // Circle around target, fire from sides
                if angle_diff.abs() > 0.25 {
                    if angle_diff > 0.0 {
                        action.turn_right = true;
                    } else {
                        action.turn_left = true;
                    }
                }

                if nearest_dist > 10.0 {
                    action.forward = true;
                } else {
                    // Circle strafe
                    action.forward = true;
                    if (state.tick % 60) < 30 {
                        action.left = true;
                        action.turn_left = true;
                    } else {
                        action.right = true;
                        action.turn_right = true;
                    }
                }

                if angle_diff.abs() < 0.5 && nearest_dist < 30.0 {
                    action.fire = true;
                }
            }

            BotBehavior::Balanced => {
                // Standard chase and shoot
                if angle_diff.abs() > 0.3 {
                    if angle_diff > 0.0 {
                        action.turn_right = true;
                    } else {
                        action.turn_left = true;
                    }
                }

                if nearest_dist > 5.0 {
                    action.forward = true;
                }

                if angle_diff.abs() < 0.5 && nearest_dist < 30.0 {
                    action.fire = true;
                }
            }

            BotBehavior::Compliance => {
                // Mid-range Rail enforcer: hold orbit near center, precise shots.
                if angle_diff.abs() > 0.12 {
                    if angle_diff > 0.0 {
                        action.turn_right = true;
                    } else {
                        action.turn_left = true;
                    }
                }
                if nearest_dist < 10.0 {
                    action.back = true;
                } else if nearest_dist > 22.0 {
                    action.forward = true;
                } else if (state.tick % 50) < 25 {
                    action.left = true;
                } else {
                    action.right = true;
                }
                if angle_diff.abs() < 0.25 && nearest_dist < 40.0 {
                    action.fire = true;
                }
            }
        }

        action
    }
}
