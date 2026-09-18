use crate::protocol::{Action, GameEvent, PlayerState, Role, Snapshot, WeaponType};
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
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            frag_limit: Some(10),
            time_limit_ticks: Some(20 * 60 * 3),
            warmup_ticks: 20 * 3,
            end_delay_ticks: 20 * 5,
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
}

pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub hp: i32,
    pub pending_action: Action,
    pub fire_cooldown: u32,
    pub respawn_timer: Option<u32>,
    pub just_fired: bool,
    pub role: Role,
    pub weapon: WeaponType,
}

impl GameState {
    pub fn new() -> Self {
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
        }
    }

    pub fn start_round(&mut self) {
        self.round_number += 1;
        self.round_state = RoundState::Active;
        self.round_ticks = 0;
        self.scores.clear();

        for player in &mut self.players {
            self.scores.insert(player.id, 0);
        }

        self.events.push(GameEvent::RoundStart {
            round_number: self.round_number,
            frag_limit: self.config.frag_limit,
            time_limit: self.config.time_limit_ticks.map(|t| t / 20),
        });

        tracing::info!(
            "Round {} started (frag_limit: {:?}, time_limit: {:?}s)",
            self.round_number,
            self.config.frag_limit,
            self.config.time_limit_ticks.map(|t| t / 20)
        );
    }

    pub fn end_round(&mut self, reason: String) {
        let winner = self
            .scores
            .iter()
            .max_by_key(|(_, &score)| score)
            .and_then(|(id, _)| self.players.iter().find(|p| p.id == *id))
            .map(|p| p.name.clone());

        self.round_state = RoundState::Ended;

        self.events.push(GameEvent::RoundEnd {
            winner: winner.clone(),
            reason: reason.clone(),
        });

        tracing::info!(
            "Round {} ended: {} (winner: {:?})",
            self.round_number,
            reason,
            winner
        );
    }

    pub fn add_player(&mut self, id: Uuid, name: String, role: Role) {
        let angle = (self.players.len() as f32) * (2.0 * PI / 8.0);
        let spawn_radius = ARENA_SIZE * 0.3;

        self.players.push(Player {
            id,
            name,
            x: angle.cos() * spawn_radius,
            y: 1.5,
            z: angle.sin() * spawn_radius,
            yaw: angle + PI,
            hp: PLAYER_MAX_HP,
            pending_action: Action::default(),
            fire_cooldown: 0,
            respawn_timer: None,
            just_fired: false,
            role,
            weapon: WeaponType::default(),
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
        self.events.clear();

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

            player.x += dx * MOVE_SPEED * dt;
            player.z += dz * MOVE_SPEED * dt;

            player.x = player.x.clamp(
                -ARENA_SIZE / 2.0 + PLAYER_RADIUS,
                ARENA_SIZE / 2.0 - PLAYER_RADIUS,
            );
            player.z = player.z.clamp(
                -ARENA_SIZE / 2.0 + PLAYER_RADIUS,
                ARENA_SIZE / 2.0 - PLAYER_RADIUS,
            );

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
            let shooter = &mut self.players[shooter_idx];
            shooter.fire_cooldown = weapon.cooldown_ticks();
            shooter.just_fired = true;

            if let Some(victim_idx) = maybe_victim_idx {
                let shooter_name = self.players[shooter_idx].name.clone();
                let shooter_id = self.players[shooter_idx].id;
                let victim = &mut self.players[victim_idx];

                victim.hp -= weapon.damage();
                if victim.hp <= 0 {
                    let victim_name = victim.name.clone();
                    victim.respawn_timer = Some(RESPAWN_DELAY_TICKS);

                    *self.scores.entry(shooter_id).or_insert(0) += 1;

                    self.events.push(GameEvent::Frag {
                        killer: shooter_name.clone(),
                        victim: victim_name.clone(),
                    });
                    tracing::info!(
                        "FRAG: {} → {} (score: {})",
                        shooter_name,
                        victim_name,
                        self.scores[&shooter_id]
                    );
                }
            }
        }

        for id in respawn_ids {
            self.do_respawn(id);
        }
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
            let spawn_radius = ARENA_SIZE * 0.3;

            player.x = angle.cos() * spawn_radius;
            player.y = 1.5;
            player.z = angle.sin() * spawn_radius;
            player.yaw = angle + PI;
            player.hp = PLAYER_MAX_HP;
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
        }
    }

    pub fn take_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.events)
    }
}

#[derive(Clone)]
pub struct BotController {
    pub player_id: Uuid,
    pub behavior: BotBehavior,
}

#[derive(Debug, Clone, Copy)]
pub enum BotBehavior {
    Aggressive,
    Defensive,
    Flanker,
    Balanced,
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
        }

        action
    }
}
