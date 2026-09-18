//! Game session: join/leave/action command application and tick broadcast.
//! Extracted from the binary so join/leave/round wire paths are unit-testable.

use crate::net::{ClientSession, GameCommand};
use crate::protocol::{self, Role, ServerMessage};
use crate::sim::{BotController, GameState, SpeakOutcome};
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
}

impl GameSession {
    pub fn new() -> Self {
        Self {
            state: GameState::new(),
            bots: Vec::new(),
            client_to_player: HashMap::new(),
            pending_unicasts: Vec::new(),
        }
    }

    /// Spawn named bots into the arena (same configs as the production binary).
    pub fn spawn_bots(&mut self, count: usize) {
        let bot_configs = [
            ("Rusher", crate::sim::BotBehavior::Aggressive),
            ("Sniper", crate::sim::BotBehavior::Defensive),
            ("Flanker", crate::sim::BotBehavior::Flanker),
            ("Tank", crate::sim::BotBehavior::Balanced),
            ("Scout", crate::sim::BotBehavior::Flanker),
            ("Guard", crate::sim::BotBehavior::Defensive),
            ("Hunter", crate::sim::BotBehavior::Aggressive),
            ("Striker", crate::sim::BotBehavior::Balanced),
        ];

        for i in 0..count {
            let bot_id = Uuid::new_v4();
            let (bot_name, behavior) = bot_configs
                .get(i)
                .unwrap_or(&("Bot", crate::sim::BotBehavior::Balanced));
            self.state
                .add_player(bot_id, bot_name.to_string(), Role::Agent);
            let bot_controller = BotController::new(bot_id, *behavior);
            self.bots.push(bot_controller.clone());
            self.state.bots.push(bot_controller);
            tracing::info!("Spawned bot: {} ({:?}, {})", bot_name, behavior, bot_id);
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
        for bot in &self.bots {
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
        let names: Vec<_> = session
            .state
            .players
            .iter()
            .map(|p| p.name.as_str())
            .collect();
        assert!(names.contains(&"Rusher"));
        assert!(names.contains(&"Sniper"));
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
}
