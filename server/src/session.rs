//! Game session: join/leave/action command application and tick broadcast.
//! Extracted from the binary so join/leave/round wire paths are unit-testable.

use crate::net::{ClientSession, GameCommand};
use crate::protocol::{self, Role, ServerMessage};
use crate::sim::{BotController, GameState, MapKind, SpeakOutcome};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Owns sim state, bot controllers, and client-to-player mapping for the arena.
pub struct GameSession {
    pub state: GameState,
    pub bots: Vec<BotController>,
    pub client_to_player: HashMap<Uuid, Uuid>,
    /// Player-targeted control messages (e.g. speak rate-limit Error). Drained by the game loop.
    pub pending_unicasts: Vec<(Uuid, ServerMessage)>,
    /// Target rule-bot count. Solo scrap and empty-arena recovery refill up to this.
    pub min_bots: usize,
}

impl GameSession {
    pub fn new() -> Self {
        Self::with_map(MapKind::ArenaDuel, false)
    }

    pub fn with_map(map: MapKind, map_rotate: bool) -> Self {
        Self {
            state: GameState::with_map(map, map_rotate),
            bots: Vec::new(),
            client_to_player: HashMap::new(),
            pending_unicasts: Vec::new(),
            min_bots: 0,
        }
    }

    /// Contested Frequency scrap-league rule-bot roster (callsigns + sticky behaviors).
    pub fn rule_bot_roster() -> &'static [(&'static str, crate::sim::BotBehavior)] {
        &[
            ("Dead Air Dan", crate::sim::BotBehavior::Aggressive),
            ("Nightfall", crate::sim::BotBehavior::Defensive),
            ("Static Kid", crate::sim::BotBehavior::Flanker),
            ("Aunt Linda", crate::sim::BotBehavior::Balanced),
            ("Scout Ant", crate::sim::BotBehavior::Flanker),
            ("Crackpot", crate::sim::BotBehavior::Defensive),
            ("Buzzkill", crate::sim::BotBehavior::Aggressive),
            ("Tin Foil Tina", crate::sim::BotBehavior::Balanced),
        ]
    }

    /// Spawn named scrap bots into the arena (same configs as the production binary).
    /// Raises `min_bots` to at least the resulting rule-bot count so solo stays stocked.
    /// Refreshes sticky Warmup Host roster intro for mid-join.
    pub fn spawn_bots(&mut self, count: usize) {
        let bot_configs = Self::rule_bot_roster();

        let start_index = self.bots.len();
        for i in 0..count {
            let bot_id = Uuid::new_v4();
            let config_index = start_index + i;
            let (bot_name, behavior) = bot_configs
                .get(config_index % bot_configs.len())
                .copied()
                .unwrap_or(("Scrap Bot", crate::sim::BotBehavior::Balanced));
            let display_name = if config_index < bot_configs.len() {
                bot_name.to_string()
            } else {
                format!("{bot_name}-{}", config_index / bot_configs.len() + 1)
            };
            self.state
                .add_player(bot_id, display_name.clone(), Role::Agent);
            let bot_controller = BotController::new(bot_id, behavior);
            self.bots.push(bot_controller.clone());
            self.state.bots.push(bot_controller);
            tracing::info!("Spawned bot: {} ({:?}, {})", display_name, behavior, bot_id);
        }
        if self.bots.len() > self.min_bots {
            self.min_bots = self.bots.len();
        }
        self.refresh_roster_host_line();
    }

    /// Rebuild sticky Warmup Host line from current rule-bot display names.
    fn refresh_roster_host_line(&mut self) {
        let names: Vec<String> = self
            .bots
            .iter()
            .filter_map(|b| {
                self.state
                    .players
                    .iter()
                    .find(|p| p.id == b.player_id)
                    .map(|p| p.name.clone())
            })
            .collect();
        self.state.set_roster_host_line_from_names(&names);
    }

    /// Set the floor for rule-bot count and refill immediately if below it.
    pub fn set_min_bots(&mut self, min_bots: usize) {
        self.min_bots = min_bots;
        self.ensure_min_bots();
    }

    /// Spawn rule bots until `bots.len() >= min_bots`. No-op when already stocked or min is 0.
    pub fn ensure_min_bots(&mut self) {
        if self.min_bots == 0 {
            return;
        }
        let have = self.bots.len();
        if have >= self.min_bots {
            return;
        }
        let need = self.min_bots - have;
        tracing::info!(
            "Arena below min_bots ({have}/{}); spawning {need} rule bot(s)",
            self.min_bots
        );
        self.spawn_bots(need);
    }

    /// Remove a client-mapped player that still holds `name` (ghost reconnect reclaim).
    /// Rule bots are not in `client_to_player` and are never evicted by name.
    fn evict_client_player_by_name(&mut self, name: &str) {
        let ghost_ids: Vec<Uuid> = self
            .state
            .players
            .iter()
            .filter(|p| p.name == name)
            .filter(|p| self.client_to_player.values().any(|pid| *pid == p.id))
            .map(|p| p.id)
            .collect();

        for player_id in ghost_ids {
            self.client_to_player.retain(|_, pid| *pid != player_id);
            let (player_name, player_score) = self
                .state
                .players
                .iter()
                .find(|p| p.id == player_id)
                .map(|p| (p.name.clone(), *self.state.scores.get(&p.id).unwrap_or(&0)))
                .unwrap_or_else(|| (name.to_string(), 0));
            let player_count_before = self.state.players.len();
            self.state.remove_player(player_id);
            self.state.push_event(protocol::GameEvent::PlayerLeft {
                player: player_name.clone(),
                score: player_score,
                round_number: self.state.round_number,
                player_count: player_count_before.saturating_sub(1),
            });
            tracing::info!(
                "Evicted ghost player {} on reconnect (score: {}, {} players remain)",
                player_name,
                player_score,
                player_count_before.saturating_sub(1)
            );
        }
    }

    /// Apply a net-layer game command (join, leave, or action).
    /// Join/leave push PlayerJoined / PlayerLeft events onto the sim event queue.
    pub fn apply_command(&mut self, cmd: GameCommand) {
        match cmd {
            GameCommand::Connected {
                id,
                role,
                name,
                player_id,
            } => {
                if let Some(pid) = player_id {
                    // Re-Hello same name: drop prior client-mapped ghost before add.
                    self.evict_client_player_by_name(&name);
                    self.state.add_player(pid, name.clone(), role);
                    self.client_to_player.insert(id, pid);
                    let player_count = self.state.players.len();
                    self.state.push_event(protocol::GameEvent::PlayerJoined {
                        player: name.clone(),
                        role: format!("{:?}", role).to_lowercase(),
                        round_number: self.state.round_number,
                        player_count,
                    });
                    tracing::info!(
                        "Player {} joined as {:?} (round {}, {} players)",
                        pid,
                        role,
                        self.state.round_number,
                        player_count
                    );
                } else {
                    tracing::info!("Spectator {} joined", name);
                }
            }

            GameCommand::Disconnected { id } => {
                if let Some(player_id) = self.client_to_player.remove(&id) {
                    let (player_name, player_score) = self
                        .state
                        .players
                        .iter()
                        .find(|p| p.id == player_id)
                        .map(|p| (p.name.clone(), *self.state.scores.get(&p.id).unwrap_or(&0)))
                        .unwrap_or_else(|| ("Unknown".to_string(), 0));
                    let player_count_before = self.state.players.len();
                    self.state.remove_player(player_id);
                    self.state.push_event(protocol::GameEvent::PlayerLeft {
                        player: player_name.clone(),
                        score: player_score,
                        round_number: self.state.round_number,
                        player_count: player_count_before.saturating_sub(1),
                    });
                    tracing::info!(
                        "Player {} left (score: {}, round {}, {} players remain)",
                        player_name,
                        player_score,
                        self.state.round_number,
                        player_count_before.saturating_sub(1)
                    );
                }
            }

            GameCommand::Action { player_id, action } => {
                self.state.set_action(player_id, action);
            }

            GameCommand::Speak { player_id, text } => {
                match self.state.try_speak(player_id, &text) {
                    SpeakOutcome::Sent => {}
                    SpeakOutcome::RateLimited => {
                        self.pending_unicasts.push((
                            player_id,
                            ServerMessage::Error {
                                code: "speak_rate_limited".to_string(),
                                message: "speak rate limited; try again in a few seconds"
                                    .to_string(),
                            },
                        ));
                    }
                    SpeakOutcome::Rejected => {
                        self.pending_unicasts.push((
                            player_id,
                            ServerMessage::Error {
                                code: "speak_rejected".to_string(),
                                message: "speak rejected".to_string(),
                            },
                        ));
                    }
                }
            }
        }
    }

    /// Run one sim tick: bot AI, physics, then collect snapshot + event messages to broadcast.
    pub fn tick_messages(&mut self, dt: f32) -> Vec<ServerMessage> {
        self.ensure_min_bots();
        let mut driven = std::collections::HashSet::new();
        for bot in &self.bots {
            let action = bot.update(&self.state);
            self.state.set_action(bot.player_id, action);
            driven.insert(bot.player_id);
        }
        // Continuance boss lives on GameState.bots only (not min_bots roster).
        let state_only: Vec<_> = self
            .state
            .bots
            .iter()
            .filter(|b| !driven.contains(&b.player_id))
            .cloned()
            .collect();
        for bot in &state_only {
            let action = bot.update(&self.state);
            self.state.set_action(bot.player_id, action);
        }

        self.state.tick(dt);

        let mut out = Vec::new();
        out.push(ServerMessage::Snapshot(self.state.snapshot()));
        for event in self.state.take_events() {
            out.push(ServerMessage::Event(event));
        }
        out
    }

    /// Drain player-targeted unicast messages queued by apply_command.
    pub fn take_unicasts(&mut self) -> Vec<(Uuid, ServerMessage)> {
        std::mem::take(&mut self.pending_unicasts)
    }
}

impl Default for GameSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Fan-out server messages to every connected client session.
pub async fn broadcast_to_clients(
    clients: &Arc<Mutex<Vec<ClientSession>>>,
    messages: &[ServerMessage],
) {
    for msg in messages {
        let clients_lock = clients.lock().await;
        for client in clients_lock.iter() {
            let _ = client.tx.send(msg.clone());
        }
        drop(clients_lock);
    }
}

/// Send queued unicast messages to the client session that owns each player_id.
pub async fn send_unicasts_to_players(
    clients: &Arc<Mutex<Vec<ClientSession>>>,
    client_to_player: &HashMap<Uuid, Uuid>,
    unicasts: &[(Uuid, ServerMessage)],
) {
    if unicasts.is_empty() {
        return;
    }
    let clients_lock = clients.lock().await;
    for (player_id, msg) in unicasts {
        let client_id = client_to_player
            .iter()
            .find(|(_, pid)| *pid == player_id)
            .map(|(cid, _)| *cid);
        let Some(client_id) = client_id else {
            continue;
        };
        if let Some(client) = clients_lock.iter().find(|c| c.id == client_id) {
            let _ = client.tx.send(msg.clone());
        }
    }
}

#[cfg(test)]
mod session_tests {
    use super::*;
    use crate::protocol::Action;

    #[test]
    fn join_agent_pushes_player_joined_event() {
        let mut session = GameSession::new();
        let client_id = Uuid::new_v4();
        let player_id = Uuid::new_v4();

        session.apply_command(GameCommand::Connected {
            id: client_id,
            role: Role::Agent,
            name: "ArenaFox".to_string(),
            player_id: Some(player_id),
        });

        assert_eq!(session.state.players.len(), 1);
        assert_eq!(session.client_to_player.get(&client_id), Some(&player_id));

        let events = session.state.take_events();
        assert!(
            events.iter().any(|e| matches!(
                e,
                protocol::GameEvent::PlayerJoined {
                    player,
                    role,
                    player_count: 1,
                    ..
                } if player == "ArenaFox" && role == "agent"
            )),
            "expected PlayerJoined, got {:?}",
            events
        );
    }

    #[test]
    fn join_spectator_does_not_add_player() {
        let mut session = GameSession::new();
        session.apply_command(GameCommand::Connected {
            id: Uuid::new_v4(),
            role: Role::Spectator,
            name: "Watcher".to_string(),
            player_id: None,
        });
        assert!(session.state.players.is_empty());
        assert!(session.state.take_events().is_empty());
    }

    #[test]
    fn leave_pushes_player_left_with_score_and_count() {
        let mut session = GameSession::new();
        let client_id = Uuid::new_v4();
        let player_id = Uuid::new_v4();

        session.apply_command(GameCommand::Connected {
            id: client_id,
            role: Role::Human,
            name: "Joiner".to_string(),
            player_id: Some(player_id),
        });
        let _ = session.state.take_events();
        *session.state.scores.get_mut(&player_id).unwrap() = 7;

        session.apply_command(GameCommand::Disconnected { id: client_id });

        assert!(session.state.players.is_empty());
        assert!(!session.client_to_player.contains_key(&client_id));

        let events = session.state.take_events();
        assert!(
            events.iter().any(|e| matches!(
                e,
                protocol::GameEvent::PlayerLeft {
                    player,
                    score: 7,
                    player_count: 0,
                    ..
                } if player == "Joiner"
            )),
            "expected PlayerLeft, got {:?}",
            events
        );
    }

    #[test]
    fn disconnect_unknown_client_is_noop() {
        let mut session = GameSession::new();
        session.apply_command(GameCommand::Disconnected { id: Uuid::new_v4() });
        assert!(session.state.take_events().is_empty());
    }

    #[test]
    fn action_command_sets_player_intent() {
        let mut session = GameSession::new();
        let client_id = Uuid::new_v4();
        let player_id = Uuid::new_v4();
        session.apply_command(GameCommand::Connected {
            id: client_id,
            role: Role::Agent,
            name: "Shooter".to_string(),
            player_id: Some(player_id),
        });
        let _ = session.state.take_events();

        session.apply_command(GameCommand::Action {
            player_id,
            action: Action {
                forward: true,
                fire: true,
                ..Default::default()
            },
        });

        let player = session
            .state
            .players
            .iter()
            .find(|p| p.id == player_id)
            .expect("player present");
        assert!(player.pending_action.forward);
        assert!(player.pending_action.fire);
    }

    #[test]
    fn tick_messages_emit_snapshot_and_round_events() {
        let mut session = GameSession::new();
        session.spawn_bots(2);
        session.state.start_round();

        let msgs = session.tick_messages(0.05);
        assert!(
            msgs.iter().any(|m| matches!(m, ServerMessage::Snapshot(_))),
            "expected Snapshot"
        );
        // Round start was pushed by start_round; tick drains remaining events if any.
        assert!(!msgs.is_empty());
    }

    #[test]
    fn spawn_bots_adds_named_agents() {
        let mut session = GameSession::new();
        session.spawn_bots(4);
        assert_eq!(session.state.players.len(), 4);
        assert_eq!(session.bots.len(), 4);
        assert_eq!(session.min_bots, 4);
        let names: Vec<_> = session
            .state
            .players
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert!(names.contains(&"Dead Air Dan"));
        assert!(names.contains(&"Nightfall"));
        assert!(names.contains(&"Static Kid"));
        assert!(names.contains(&"Aunt Linda"));
        assert!(session
            .state
            .roster_host_line
            .as_ref()
            .is_some_and(|h| h.contains("DEAD AIR DAN") && h.contains("ON THE SCRAP")));
    }

    #[test]
    fn spawn_bots_roster_is_contested_frequency_callsigns() {
        let roster: Vec<_> = GameSession::rule_bot_roster()
            .iter()
            .map(|(n, _)| *n)
            .collect();
        assert_eq!(roster.len(), 8);
        assert!(!roster
            .iter()
            .any(|n| n.starts_with("Bot") || *n == "Rusher"));
        assert!(roster.contains(&"Buzzkill"));
        assert!(roster.contains(&"Tin Foil Tina"));
    }

    #[test]
    fn warmup_snapshot_host_line_names_scrap_roster() {
        let mut session = GameSession::new();
        session.spawn_bots(4);
        assert_eq!(session.state.round_state, crate::sim::RoundState::Warmup);
        let snap = session.state.snapshot();
        assert!(
            snap.host_line.contains("ON THE SCRAP"),
            "Warmup mid-join should name dialed-in scrap bots: {}",
            snap.host_line
        );
        assert!(snap.host_line.contains("DEAD AIR DAN"));
        // RoundStart should carry the same roster Host flavor.
        session.state.start_round();
        let start = session
            .state
            .events
            .iter()
            .find_map(|e| match e {
                protocol::GameEvent::RoundStart { host_line, .. } => Some(host_line.clone()),
                _ => None,
            })
            .expect("RoundStart");
        assert!(start.contains("ON THE SCRAP"));
    }

    #[test]
    fn ensure_min_bots_refills_empty_arena() {
        let mut session = GameSession::new();
        session.set_min_bots(4);
        assert_eq!(session.bots.len(), 4);
        assert_eq!(session.state.players.len(), 4);

        // Simulate emptied rule-bot roster (solo must not stay empty).
        session.bots.clear();
        session.state.bots.clear();
        session.state.players.clear();
        session.ensure_min_bots();
        assert_eq!(session.bots.len(), 4);
        assert_eq!(session.state.players.len(), 4);
    }

    #[test]
    fn ensure_min_bots_noop_when_stocked_or_zero() {
        let mut session = GameSession::new();
        session.ensure_min_bots();
        assert!(session.bots.is_empty());

        session.spawn_bots(2);
        let before = session.bots.len();
        session.ensure_min_bots();
        assert_eq!(session.bots.len(), before);

        session.set_min_bots(2);
        session.ensure_min_bots();
        assert_eq!(session.bots.len(), 2);
    }

    #[test]
    fn tick_messages_ensures_min_bots_before_ai() {
        let mut session = GameSession::new();
        session.min_bots = 3;
        assert!(session.bots.is_empty());
        let _ = session.tick_messages(0.05);
        assert_eq!(session.bots.len(), 3);
        assert!(session.state.players.len() >= 3);
    }

    #[test]
    fn join_then_leave_round_trip_updates_player_count_on_events() {
        let mut session = GameSession::new();
        session.spawn_bots(2);
        let c1 = Uuid::new_v4();
        let p1 = Uuid::new_v4();
        let c2 = Uuid::new_v4();
        let p2 = Uuid::new_v4();

        session.apply_command(GameCommand::Connected {
            id: c1,
            role: Role::Agent,
            name: "A".to_string(),
            player_id: Some(p1),
        });
        session.apply_command(GameCommand::Connected {
            id: c2,
            role: Role::Agent,
            name: "B".to_string(),
            player_id: Some(p2),
        });
        let events = session.state.take_events();
        let join_counts: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                protocol::GameEvent::PlayerJoined { player_count, .. } => Some(*player_count),
                _ => None,
            })
            .collect();
        assert_eq!(join_counts, vec![3, 4]);

        session.apply_command(GameCommand::Disconnected { id: c1 });
        let events = session.state.take_events();
        assert!(events.iter().any(|e| matches!(
            e,
            protocol::GameEvent::PlayerLeft {
                player_count: 3,
                ..
            }
        )));
    }

    #[test]
    fn speak_command_pushes_rate_limited_event() {
        let mut session = GameSession::new();
        let client_id = Uuid::new_v4();
        let player_id = Uuid::new_v4();
        session.apply_command(GameCommand::Connected {
            id: client_id,
            role: Role::Agent,
            name: "ArenaFox".to_string(),
            player_id: Some(player_id),
        });
        let _ = session.state.take_events();

        session.apply_command(GameCommand::Speak {
            player_id,
            text: "  nice scrap  ".to_string(),
        });
        let events = session.state.take_events();
        assert!(
            events.iter().any(|e| matches!(
                e,
                protocol::GameEvent::Speak {
                    player,
                    text,
                    ..
                } if player == "ArenaFox" && text == "nice scrap"
            )),
            "expected Speak, got {:?}",
            events
        );

        // Rate limit: immediate second speak is dropped + Error unicast.
        session.apply_command(GameCommand::Speak {
            player_id,
            text: "again".to_string(),
        });
        assert!(
            session.state.take_events().is_empty(),
            "rate-limited speak must not emit"
        );
        let unicasts = session.take_unicasts();
        assert_eq!(unicasts.len(), 1, "expected one speak Error unicast");
        assert_eq!(unicasts[0].0, player_id);
        match &unicasts[0].1 {
            ServerMessage::Error { code, message } => {
                assert_eq!(code, "speak_rate_limited");
                assert!(message.contains("rate limited"), "{message}");
            }
            other => panic!("expected Error, got {:?}", other),
        }

        // Advance ticks past cooldown.
        for _ in 0..crate::sim::SPEAK_COOLDOWN_TICKS {
            session.state.tick(0.05);
            let _ = session.state.take_events();
        }
        session.apply_command(GameCommand::Speak {
            player_id,
            text: "again".to_string(),
        });
        assert!(
            session.state.take_events().iter().any(|e| matches!(
                e,
                protocol::GameEvent::Speak { text, .. } if text == "again"
            )),
            "speak after cooldown must emit"
        );
    }

    #[test]
    fn speak_rejects_empty_and_overlong() {
        let mut session = GameSession::new();
        let player_id = Uuid::new_v4();
        session.apply_command(GameCommand::Connected {
            id: Uuid::new_v4(),
            role: Role::Agent,
            name: "Talker".to_string(),
            player_id: Some(player_id),
        });
        let _ = session.state.take_events();

        session.apply_command(GameCommand::Speak {
            player_id,
            text: "   ".to_string(),
        });
        assert!(session.state.take_events().is_empty());
        let u = session.take_unicasts();
        assert!(
            matches!(
                &u[..],
                [(pid, ServerMessage::Error { code, .. })]
                    if *pid == player_id && code == "speak_rejected"
            ),
            "empty speak must Error unicast, got {:?}",
            u
        );

        let long = "x".repeat(crate::sim::SPEAK_MAX_CHARS + 1);
        session.apply_command(GameCommand::Speak {
            player_id,
            text: long,
        });
        assert!(session.state.take_events().is_empty());
        let u = session.take_unicasts();
        assert!(
            matches!(
                &u[..],
                [(pid, ServerMessage::Error { code, .. })]
                    if *pid == player_id && code == "speak_rejected"
            ),
            "overlong speak must Error unicast, got {:?}",
            u
        );
    }

    #[test]
    fn compliance_drone_spawn_does_not_inflate_min_bots() {
        let mut session = GameSession::new();
        session.spawn_bots(2);
        assert_eq!(session.min_bots, 2);
        assert_eq!(session.bots.len(), 2);

        session.state.start_round();
        session.state.config.boss_spawn_ticks = Some(1);
        session.state.config.compliance_ping_ticks = None;
        // Advance Active to spawn tick.
        while !session.state.boss_spawned {
            let _ = session.tick_messages(0.05);
            if session.state.round_ticks > 20 {
                panic!("boss should have spawned");
            }
        }
        assert!(session.state.boss_id.is_some());
        assert_eq!(
            session.bots.len(),
            2,
            "rule-bot roster must stay at min_bots"
        );
        assert_eq!(session.min_bots, 2);
        // Boss is on state.bots for AI + behavior chip.
        assert!(session
            .state
            .bots
            .iter()
            .any(|b| b.behavior == crate::sim::BotBehavior::Compliance));
        // Snapshot must list the drone.
        let snap = session.state.snapshot();
        assert!(snap
            .players
            .iter()
            .any(|p| p.name == protocol::BOSS_NAME && p.behavior.as_deref() == Some("Compliance")));
        assert_eq!(snap.pressure.as_deref(), Some("compliance_drone"));
    }

    #[test]
    fn reconnect_same_name_after_leave_is_single_player() {
        let mut session = GameSession::new();
        let c1 = Uuid::new_v4();
        let p1 = Uuid::new_v4();
        session.apply_command(GameCommand::Connected {
            id: c1,
            role: Role::Human,
            name: "Human Player".to_string(),
            player_id: Some(p1),
        });
        let _ = session.state.take_events();

        session.apply_command(GameCommand::Disconnected { id: c1 });
        assert!(session.state.players.is_empty());
        let _ = session.state.take_events();

        let c2 = Uuid::new_v4();
        let p2 = Uuid::new_v4();
        session.apply_command(GameCommand::Connected {
            id: c2,
            role: Role::Human,
            name: "Human Player".to_string(),
            player_id: Some(p2),
        });

        assert_eq!(session.state.players.len(), 1);
        assert_eq!(session.state.players[0].id, p2);
        assert_eq!(session.state.players[0].name, "Human Player");
        assert_eq!(session.client_to_player.get(&c2), Some(&p2));
        assert!(!session.client_to_player.contains_key(&c1));
    }

    #[test]
    fn overlapping_reconnect_same_name_evicts_ghost_without_stuck_session() {
        let mut session = GameSession::new();
        let c_old = Uuid::new_v4();
        let p_old = Uuid::new_v4();
        session.apply_command(GameCommand::Connected {
            id: c_old,
            role: Role::Agent,
            name: "ArenaFox".to_string(),
            player_id: Some(p_old),
        });
        *session.state.scores.get_mut(&p_old).unwrap() = 3;
        let _ = session.state.take_events();

        // New Hello before old Disconnect (soft prison without eviction).
        let c_new = Uuid::new_v4();
        let p_new = Uuid::new_v4();
        session.apply_command(GameCommand::Connected {
            id: c_new,
            role: Role::Agent,
            name: "ArenaFox".to_string(),
            player_id: Some(p_new),
        });

        assert_eq!(
            session.state.players.len(),
            1,
            "ghost must be evicted; only one ArenaFox"
        );
        assert_eq!(session.state.players[0].id, p_new);
        assert_eq!(session.client_to_player.get(&c_new), Some(&p_new));
        assert!(
            !session.client_to_player.contains_key(&c_old),
            "old client mapping must be cleared"
        );

        let events = session.state.take_events();
        assert!(
            events.iter().any(|e| matches!(
                e,
                protocol::GameEvent::PlayerLeft {
                    player,
                    score: 3,
                    ..
                } if player == "ArenaFox"
            )),
            "expected ghost PlayerLeft, got {:?}",
            events
        );
        assert!(
            events.iter().any(|e| matches!(
                e,
                protocol::GameEvent::PlayerJoined {
                    player,
                    player_count: 1,
                    ..
                } if player == "ArenaFox"
            )),
            "expected PlayerJoined after reclaim, got {:?}",
            events
        );

        // Late Disconnect for the old socket must not remove the new player.
        session.apply_command(GameCommand::Disconnected { id: c_old });
        assert_eq!(session.state.players.len(), 1);
        assert_eq!(session.state.players[0].id, p_new);
        assert!(session.state.take_events().is_empty());
    }

    #[test]
    fn reconnect_new_name_is_new_session_without_ghost() {
        let mut session = GameSession::new();
        let c1 = Uuid::new_v4();
        let p1 = Uuid::new_v4();
        session.apply_command(GameCommand::Connected {
            id: c1,
            role: Role::Human,
            name: "Alpha".to_string(),
            player_id: Some(p1),
        });
        session.apply_command(GameCommand::Disconnected { id: c1 });
        let _ = session.state.take_events();

        let c2 = Uuid::new_v4();
        let p2 = Uuid::new_v4();
        session.apply_command(GameCommand::Connected {
            id: c2,
            role: Role::Human,
            name: "Bravo".to_string(),
            player_id: Some(p2),
        });

        assert_eq!(session.state.players.len(), 1);
        assert_eq!(session.state.players[0].name, "Bravo");
        assert_eq!(session.client_to_player.len(), 1);
    }

    #[test]
    fn reconnect_same_name_does_not_evict_rule_bot() {
        let mut session = GameSession::new();
        session.spawn_bots(1);
        assert_eq!(session.state.players.len(), 1);
        let bot_name = session.state.players[0].name.clone();
        let bot_id = session.state.players[0].id;

        let c1 = Uuid::new_v4();
        let p1 = Uuid::new_v4();
        session.apply_command(GameCommand::Connected {
            id: c1,
            role: Role::Human,
            name: bot_name.clone(),
            player_id: Some(p1),
        });

        assert_eq!(session.state.players.len(), 2);
        assert!(session.state.players.iter().any(|p| p.id == bot_id));
        assert!(session.state.players.iter().any(|p| p.id == p1));
    }
}
