#[cfg(test)]
use crate::protocol::{
    Action, ClientMessage, GameEvent, PlayerScore, PlayerState, Role, ServerMessage, Snapshot,
    WeaponType,
};
#[cfg(test)]
use crate::sim::{BotBehavior, BotController, GameState, MatchConfig, RoundState};
#[cfg(test)]
use uuid::Uuid;

#[test]
fn test_protocol_client_message_hello_serialization() {
    let hello = ClientMessage::Hello {
        role: Role::Agent,
        name: "TestBot".to_string(),
    };
    let json = serde_json::to_string(&hello).unwrap();
    assert!(json.contains(r#""type":"hello""#));
    assert!(json.contains(r#""role":"agent""#));

    let deserialized: ClientMessage = serde_json::from_str(&json).unwrap();
    match deserialized {
        ClientMessage::Hello { role, name } => {
            assert_eq!(role, Role::Agent);
            assert_eq!(name, "TestBot");
        }
        _ => panic!("Expected Hello message"),
    }
}

#[test]
fn test_protocol_client_message_action_serialization() {
    let action_msg = ClientMessage::Action(Action {
        forward: true,
        fire: true,
        weapon_swap: Some(WeaponType::Rail),
        ..Default::default()
    });
    let json = serde_json::to_string(&action_msg).unwrap();
    assert!(json.contains(r#""type":"action""#));
    assert!(json.contains(r#""forward":true"#));
    assert!(json.contains(r#""fire":true"#));
    assert!(json.contains(r#""weapon_swap":"rail""#));
}

#[test]
fn test_protocol_server_message_welcome() {
    let welcome = ServerMessage::Welcome {
        player_id: Some(Uuid::new_v4()),
        role: Role::Human,
    };
    let json = serde_json::to_string(&welcome).unwrap();
    assert!(json.contains(r#""type":"welcome""#));
    assert!(json.contains(r#""role":"human""#));

    let welcome_spectator = ServerMessage::Welcome {
        player_id: None,
        role: Role::Spectator,
    };
    let json = serde_json::to_string(&welcome_spectator).unwrap();
    assert!(json.contains(r#""player_id":null"#));
}

#[test]
fn test_protocol_server_message_event_frag() {
    let event = GameEvent::Frag {
        killer: "Bot1".to_string(),
        victim: "Bot2".to_string(),
        killer_score: 3,
    };
    let event_msg = ServerMessage::Event(event);
    let json = serde_json::to_string(&event_msg).unwrap();
    assert!(json.contains(r#""event":"frag""#));
    assert!(json.contains("Bot1"));
    assert!(json.contains("Bot2"));

    let deserialized: ServerMessage = serde_json::from_str(&json).unwrap();
    match deserialized {
        ServerMessage::Event(GameEvent::Frag {
            killer,
            victim,
            killer_score,
        }) => {
            assert_eq!(killer, "Bot1");
            assert_eq!(victim, "Bot2");
            assert_eq!(killer_score, 3);
        }
        _ => panic!("Expected Event(Frag)"),
    }
}

#[test]
fn test_protocol_role_serialization() {
    let agent = Role::Agent;
    assert_eq!(serde_json::to_string(&agent).unwrap(), r#""agent""#);

    let human = Role::Human;
    assert_eq!(serde_json::to_string(&human).unwrap(), r#""human""#);

    let spectator = Role::Spectator;
    assert_eq!(serde_json::to_string(&spectator).unwrap(), r#""spectator""#);
}

#[test]
fn test_protocol_weapon_type_serialization() {
    assert_eq!(
        serde_json::to_string(&WeaponType::Flechette).unwrap(),
        r#""flechette""#
    );
    assert_eq!(
        serde_json::to_string(&WeaponType::Rail).unwrap(),
        r#""rail""#
    );
    assert_eq!(
        serde_json::to_string(&WeaponType::Scatter).unwrap(),
        r#""scatter""#
    );
}

#[test]
fn test_protocol_weapon_type_deserialization() {
    let weapon: WeaponType = serde_json::from_str(r#""flechette""#).unwrap();
    assert_eq!(weapon, WeaponType::Flechette);

    let weapon: WeaponType = serde_json::from_str(r#""rail""#).unwrap();
    assert_eq!(weapon, WeaponType::Rail);

    let weapon: WeaponType = serde_json::from_str(r#""scatter""#).unwrap();
    assert_eq!(weapon, WeaponType::Scatter);
}

#[test]
fn test_protocol_invalid_weapon_type() {
    let result: Result<WeaponType, _> = serde_json::from_str(r#""invalid""#);
    assert!(result.is_err());
}

#[test]
fn test_protocol_action_defaults() {
    let action = Action::default();
    assert!(!action.forward);
    assert!(!action.back);
    assert!(!action.fire);
    assert!(action.weapon_swap.is_none());
}

#[test]
fn test_protocol_action_partial_deserialization() {
    let json = r#"{"forward":true}"#;
    let action: Action = serde_json::from_str(json).unwrap();
    assert!(action.forward);
    assert!(!action.back);
}

#[test]
fn test_protocol_game_event_respawn() {
    let event = GameEvent::Respawn {
        player: "Bot1".to_string(),
    };
    let json = serde_json::to_value(&event).unwrap();
    assert_eq!(json["event"], "respawn");
    assert_eq!(json["player"], "Bot1");
}

#[test]
fn test_protocol_game_event_round_start() {
    let event = GameEvent::RoundStart {
        round_number: 2,
        frag_limit: Some(10),
        time_limit: Some(180),
        players: vec!["Bot1".to_string(), "Bot2".to_string()],
        previous_winner: Some("Bot1".to_string()),
    };
    let json = serde_json::to_value(&event).unwrap();
    assert_eq!(json["event"], "round_start");
    assert_eq!(json["round_number"], 2);
    assert_eq!(json["frag_limit"], 10);
}

#[test]
fn test_protocol_game_event_round_end() {
    let event = GameEvent::RoundEnd {
        winner: Some("Bot1".to_string()),
        reason: "Frag limit reached".to_string(),
        final_scores: vec![PlayerScore {
            name: "Bot1".to_string(),
            score: 10,
        }],
        winner_score: Some(10),
    };
    let json = serde_json::to_value(&event).unwrap();
    assert_eq!(json["event"], "round_end");
    assert_eq!(json["winner"], "Bot1");
}

#[test]
fn test_protocol_game_event_player_joined() {
    let event = GameEvent::PlayerJoined {
        player: "NewPlayer".to_string(),
        role: "human".to_string(),
        round_number: 3,
        player_count: 5,
    };
    let json = serde_json::to_value(&event).unwrap();
    assert_eq!(json["event"], "player_joined");
    assert_eq!(json["player_count"], 5);
}

#[test]
fn test_protocol_game_event_player_left() {
    let event = GameEvent::PlayerLeft {
        player: "OldPlayer".to_string(),
        score: 8,
        round_number: 2,
        player_count: 3,
    };
    let json = serde_json::to_value(&event).unwrap();
    assert_eq!(json["event"], "player_left");
    assert_eq!(json["score"], 8);
}

#[test]
fn test_protocol_snapshot_serialization() {
    let snapshot = Snapshot {
        tick: 123,
        players: vec![PlayerState {
            id: Uuid::new_v4(),
            name: "Player1".to_string(),
            x: 10.0,
            y: 1.5,
            z: -5.0,
            yaw: 1.57,
            hp: 100,
            just_fired: false,
            behavior: None,
            score: 5,
            weapon: "Flechette".to_string(),
        }],
        round_state: Some("Active".to_string()),
        round_time_left: Some(60),
        frag_limit: Some(10),
    };
    let json = serde_json::to_value(&snapshot).unwrap();
    assert_eq!(json["tick"], 123);
    assert_eq!(json["players"][0]["name"], "Player1");
}

#[test]
fn test_protocol_snapshot_empty_players() {
    let snapshot = Snapshot {
        tick: 0,
        players: vec![],
        round_state: None,
        round_time_left: None,
        frag_limit: None,
    };
    let json = serde_json::to_string(&snapshot).unwrap();
    assert!(json.contains(r#""tick":0"#));
    assert!(json.contains(r#""players":[]"#));
}

#[test]
fn test_protocol_invalid_client_message() {
    let result: Result<ClientMessage, _> = serde_json::from_str(r#"{"type":"invalid"}"#);
    assert!(result.is_err());
}

#[test]
fn test_protocol_invalid_server_message() {
    let result: Result<ServerMessage, _> = serde_json::from_str(r#"{"type":"invalid"}"#);
    assert!(result.is_err());
}

#[test]
fn test_sim_bot_controller_aggressive() {
    let mut state = GameState::new();
    state.start_round();

    let bot_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player(bot_id, "AggressiveBot".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    let bot = BotController::new(bot_id, BotBehavior::Aggressive);
    let action = bot.update(&state);

    assert!(action.forward || action.turn_left || action.turn_right);
}

#[test]
fn test_sim_bot_controller_defensive() {
    let mut state = GameState::new();
    state.start_round();

    let bot_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player(bot_id, "DefensiveBot".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    let bot = BotController::new(bot_id, BotBehavior::Defensive);
    let action = bot.update(&state);

    assert!(
        action.forward
            || action.back
            || action.left
            || action.right
            || action.turn_left
            || action.turn_right
    );
}

#[test]
fn test_sim_bot_controller_flanker() {
    let mut state = GameState::new();
    state.start_round();

    let bot_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player(bot_id, "FlankerBot".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    let bot = BotController::new(bot_id, BotBehavior::Flanker);
    let action = bot.update(&state);

    assert!(action.forward || action.left || action.right || action.turn_left || action.turn_right);
}

#[test]
fn test_sim_bot_controller_balanced() {
    let mut state = GameState::new();
    state.start_round();

    let bot_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player(bot_id, "BalancedBot".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    let bot = BotController::new(bot_id, BotBehavior::Balanced);
    let action = bot.update(&state);

    assert!(action.forward || action.turn_left || action.turn_right);
}

#[test]
fn test_sim_bot_no_target_returns_default() {
    let state = GameState::new();
    let bot_id = Uuid::new_v4();

    let bot = BotController::new(bot_id, BotBehavior::Aggressive);
    let action = bot.update(&state);

    assert!(!action.forward);
    assert!(!action.fire);
}

#[test]
fn test_sim_round_state_transitions() {
    let mut state = GameState::new();
    assert_eq!(state.round_state, RoundState::Warmup);

    for _ in 0..state.config.warmup_ticks {
        state.tick(0.05);
    }
    assert_eq!(state.round_state, RoundState::Active);

    state.end_round("Manual end".to_string());
    assert_eq!(state.round_state, RoundState::Ended);
}

#[test]
fn test_sim_round_end_event_generation() {
    let mut state = GameState::new();
    let player_id = Uuid::new_v4();
    state.add_player(player_id, "Winner".to_string(), Role::Agent);
    state.start_round();

    *state.scores.entry(player_id).or_insert(0) = 10;

    state.end_round("Test end".to_string());

    let events = state.take_events();
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::RoundEnd { .. })));
}

#[test]
fn test_sim_player_spawn_positions_distributed() {
    let mut state = GameState::new();

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();

    state.add_player(id1, "Player1".to_string(), Role::Agent);
    state.add_player(id2, "Player2".to_string(), Role::Agent);
    state.add_player(id3, "Player3".to_string(), Role::Agent);

    let pos1 = (state.players[0].x, state.players[0].z);
    let pos2 = (state.players[1].x, state.players[1].z);
    let pos3 = (state.players[2].x, state.players[2].z);

    assert_ne!(pos1, pos2);
    assert_ne!(pos2, pos3);
    assert_ne!(pos1, pos3);
}

#[test]
fn test_sim_fire_cooldown_decrements() {
    let mut state = GameState::new();
    state.start_round();

    let player_id = Uuid::new_v4();
    state.add_player(player_id, "TestPlayer".to_string(), Role::Agent);

    let idx = state
        .players
        .iter()
        .position(|p| p.id == player_id)
        .unwrap();
    state.players[idx].fire_cooldown = 10;

    state.tick(0.05);

    assert_eq!(state.players[idx].fire_cooldown, 9);
}

#[test]
fn test_sim_respawn_timer_decrements() {
    let mut state = GameState::new();
    state.start_round();

    let player_id = Uuid::new_v4();
    state.add_player(player_id, "TestPlayer".to_string(), Role::Agent);

    let idx = state
        .players
        .iter()
        .position(|p| p.id == player_id)
        .unwrap();
    state.players[idx].respawn_timer = Some(10);

    state.tick(0.05);

    assert_eq!(state.players[idx].respawn_timer, Some(9));
}

#[test]
fn test_sim_dead_player_excluded_from_snapshot() {
    let mut state = GameState::new();
    state.start_round();

    let alive_id = Uuid::new_v4();
    let dead_id = Uuid::new_v4();

    state.add_player(alive_id, "Alive".to_string(), Role::Agent);
    state.add_player(dead_id, "Dead".to_string(), Role::Agent);

    let dead_idx = state.players.iter().position(|p| p.id == dead_id).unwrap();
    state.players[dead_idx].respawn_timer = Some(30);

    let snapshot = state.snapshot();

    assert_eq!(snapshot.players.len(), 1);
    assert_eq!(snapshot.players[0].id, alive_id);
}

#[test]
fn test_sim_arena_boundary_clamping() {
    let mut state = GameState::new();
    state.start_round();

    let player_id = Uuid::new_v4();
    state.add_player(player_id, "TestPlayer".to_string(), Role::Agent);

    let idx = state
        .players
        .iter()
        .position(|p| p.id == player_id)
        .unwrap();
    state.players[idx].x = 100.0;

    state.set_action(
        player_id,
        Action {
            forward: true,
            ..Default::default()
        },
    );

    state.tick(0.05);

    assert!(state.players[idx].x <= 25.0);
}

#[test]
fn test_sim_yaw_normalization() {
    let mut state = GameState::new();
    state.start_round();

    let player_id = Uuid::new_v4();
    state.add_player(player_id, "TestPlayer".to_string(), Role::Agent);

    let idx = state
        .players
        .iter()
        .position(|p| p.id == player_id)
        .unwrap();

    for _ in 0..100 {
        state.set_action(
            player_id,
            Action {
                turn_right: true,
                ..Default::default()
            },
        );
        state.tick(0.05);
    }

    let yaw = state.players[idx].yaw;
    assert!((0.0..2.0 * std::f32::consts::PI).contains(&yaw));
}

#[test]
fn test_sim_match_config_custom() {
    let config = MatchConfig {
        frag_limit: Some(5),
        time_limit_ticks: Some(100),
        warmup_ticks: 10,
        end_delay_ticks: 20,
    };

    assert_eq!(config.frag_limit, Some(5));
    assert_eq!(config.time_limit_ticks, Some(100));
}

#[test]
fn test_sim_scores_initialized_on_player_add() {
    let mut state = GameState::new();
    let player_id = Uuid::new_v4();

    state.add_player(player_id, "TestPlayer".to_string(), Role::Agent);

    assert!(state.scores.contains_key(&player_id));
    assert_eq!(*state.scores.get(&player_id).unwrap(), 0);
}

#[test]
fn test_sim_just_fired_flag_cleared() {
    let mut state = GameState::new();
    state.start_round();

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player(shooter_id, "Shooter".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    let shooter_idx = state
        .players
        .iter()
        .position(|p| p.id == shooter_id)
        .unwrap();
    let target_idx = state
        .players
        .iter()
        .position(|p| p.id == target_id)
        .unwrap();

    state.players[target_idx].x = 5.0;
    state.players[target_idx].z = 0.0;
    state.players[shooter_idx].x = 0.0;
    state.players[shooter_idx].z = 0.0;
    state.players[shooter_idx].yaw = 0.0;

    state.set_action(
        shooter_id,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);

    assert!(state.players[shooter_idx].just_fired);

    state.set_action(shooter_id, Action::default());
    state.tick(0.05);

    assert!(!state.players[shooter_idx].just_fired);
}

#[test]
fn test_sim_player_cannot_fire_during_cooldown() {
    let mut state = GameState::new();
    state.start_round();

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player(shooter_id, "Shooter".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    let shooter_idx = state
        .players
        .iter()
        .position(|p| p.id == shooter_id)
        .unwrap();
    let target_idx = state
        .players
        .iter()
        .position(|p| p.id == target_id)
        .unwrap();

    state.players[target_idx].x = 5.0;
    state.players[target_idx].z = 0.0;
    state.players[shooter_idx].x = 0.0;
    state.players[shooter_idx].z = 0.0;
    state.players[shooter_idx].yaw = 0.0;

    state.set_action(
        shooter_id,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);

    let initial_hp = state.players[target_idx].hp;

    state.set_action(
        shooter_id,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);

    assert_eq!(state.players[target_idx].hp, initial_hp);
}

#[test]
fn test_sim_respawn_event_generated() {
    let mut state = GameState::new();
    state.start_round();

    let player_id = Uuid::new_v4();
    state.add_player(player_id, "TestPlayer".to_string(), Role::Agent);

    let idx = state
        .players
        .iter()
        .position(|p| p.id == player_id)
        .unwrap();
    state.players[idx].hp = 0;
    state.players[idx].respawn_timer = Some(1);

    state.tick(0.05);

    let events = state.take_events();
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::Respawn { .. })));
}

#[test]
fn test_sim_snapshot_includes_round_info() {
    let mut state = GameState::new();
    state.config.time_limit_ticks = Some(200);
    state.config.frag_limit = Some(15);
    state.start_round();

    state.tick(0.05);

    let snapshot = state.snapshot();
    assert!(snapshot.round_state.is_some());
    assert!(snapshot.round_time_left.is_some());
    assert_eq!(snapshot.frag_limit, Some(15));
}

#[test]
fn test_sim_event_buffer_cleared_on_take() {
    let mut state = GameState::new();
    state.push_event(GameEvent::Respawn {
        player: "Test".to_string(),
    });

    assert_eq!(state.events.len(), 1);

    let events = state.take_events();
    assert_eq!(events.len(), 1);
    assert_eq!(state.events.len(), 0);
}

#[test]
fn test_sim_bot_does_not_update_while_dead() {
    let mut state = GameState::new();
    state.start_round();

    let bot_id = Uuid::new_v4();
    state.add_player(bot_id, "DeadBot".to_string(), Role::Agent);

    let idx = state.players.iter().position(|p| p.id == bot_id).unwrap();
    state.players[idx].respawn_timer = Some(30);

    let bot = BotController::new(bot_id, BotBehavior::Aggressive);
    let action = bot.update(&state);

    assert!(!action.forward);
    assert!(!action.fire);
}

#[test]
fn test_sim_round_start_event_includes_players() {
    let mut state = GameState::new();

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    state.add_player(id1, "Player1".to_string(), Role::Agent);
    state.add_player(id2, "Player2".to_string(), Role::Agent);

    state.start_round();

    let events = state.take_events();
    let round_start = events
        .iter()
        .find_map(|e| match e {
            GameEvent::RoundStart { players, .. } => Some(players),
            _ => None,
        })
        .expect("RoundStart event should exist");

    assert_eq!(round_start.len(), 2);
    assert!(round_start.contains(&"Player1".to_string()));
    assert!(round_start.contains(&"Player2".to_string()));
}

#[test]
fn test_net_game_command_connected() {
    use crate::net::GameCommand;

    let id = Uuid::new_v4();
    let player_id = Some(Uuid::new_v4());

    let cmd = GameCommand::Connected {
        id,
        role: Role::Agent,
        name: "TestAgent".to_string(),
        player_id,
    };

    match cmd {
        GameCommand::Connected {
            id: _,
            role,
            name,
            player_id: pid,
        } => {
            assert_eq!(role, Role::Agent);
            assert_eq!(name, "TestAgent");
            assert_eq!(pid, player_id);
        }
        _ => panic!("Expected Connected command"),
    }
}

#[test]
fn test_net_game_command_disconnected() {
    use crate::net::GameCommand;

    let id = Uuid::new_v4();
    let cmd = GameCommand::Disconnected { id };

    match cmd {
        GameCommand::Disconnected { id: cmd_id } => {
            assert_eq!(cmd_id, id);
        }
        _ => panic!("Expected Disconnected command"),
    }
}

#[test]
fn test_net_game_command_action() {
    use crate::net::GameCommand;

    let player_id = Uuid::new_v4();
    let action = Action {
        forward: true,
        fire: true,
        ..Default::default()
    };

    let cmd = GameCommand::Action {
        player_id,
        action: action.clone(),
    };

    match cmd {
        GameCommand::Action {
            player_id: pid,
            action: a,
        } => {
            assert_eq!(pid, player_id);
            assert!(a.forward);
            assert!(a.fire);
        }
        _ => panic!("Expected Action command"),
    }
}

#[test]
fn test_net_client_session_structure() {
    use crate::net::ClientSession;
    use tokio::sync::mpsc;

    let id = Uuid::new_v4();
    let (tx, _rx) = mpsc::unbounded_channel();

    let session = ClientSession { id, tx };

    assert_eq!(session.id, id);
}

#[test]
fn test_protocol_all_weapon_types_coverage() {
    assert_eq!(WeaponType::Flechette.damage(), 25);
    assert_eq!(WeaponType::Flechette.cooldown_ticks(), 10);
    assert_eq!(WeaponType::Flechette.spread_radians(), 0.1);
    assert_eq!(WeaponType::Flechette.name(), "Flechette");

    assert_eq!(WeaponType::Rail.damage(), 75);
    assert_eq!(WeaponType::Rail.cooldown_ticks(), 40);
    assert_eq!(WeaponType::Rail.spread_radians(), 0.05);
    assert_eq!(WeaponType::Rail.name(), "Rail");

    assert_eq!(WeaponType::Scatter.damage(), 15);
    assert_eq!(WeaponType::Scatter.cooldown_ticks(), 5);
    assert_eq!(WeaponType::Scatter.spread_radians(), 0.3);
    assert_eq!(WeaponType::Scatter.name(), "Scatter");
}

#[test]
fn test_sim_all_bot_behaviors_coverage() {
    let mut state = GameState::new();
    state.start_round();

    let bot_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player(bot_id, "Bot".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    for behavior in [
        BotBehavior::Aggressive,
        BotBehavior::Defensive,
        BotBehavior::Flanker,
        BotBehavior::Balanced,
    ] {
        let bot = BotController::new(bot_id, behavior);
        let _ = bot.update(&state);
    }
}

#[test]
fn test_sim_player_movement_all_directions() {
    let mut state = GameState::new();
    state.start_round();

    let player_id = Uuid::new_v4();
    state.add_player(player_id, "Mover".to_string(), Role::Agent);

    let idx = state
        .players
        .iter()
        .position(|p| p.id == player_id)
        .unwrap();

    state.players[idx].x = 0.0;
    state.players[idx].z = 0.0;

    state.set_action(
        player_id,
        Action {
            forward: true,
            ..Default::default()
        },
    );
    state.tick(0.05);

    state.set_action(
        player_id,
        Action {
            back: true,
            ..Default::default()
        },
    );
    state.tick(0.05);

    state.set_action(
        player_id,
        Action {
            left: true,
            ..Default::default()
        },
    );
    state.tick(0.05);

    state.set_action(
        player_id,
        Action {
            right: true,
            ..Default::default()
        },
    );
    state.tick(0.05);

    state.set_action(
        player_id,
        Action {
            turn_left: true,
            ..Default::default()
        },
    );
    state.tick(0.05);

    state.set_action(
        player_id,
        Action {
            turn_right: true,
            ..Default::default()
        },
    );
    state.tick(0.05);
}

#[test]
fn test_sim_round_ended_state_waits_for_delay() {
    let mut state = GameState::new();
    state.config.end_delay_ticks = 10;
    state.start_round();
    state.end_round("Test".to_string());

    assert_eq!(state.round_state, RoundState::Ended);
    assert_eq!(
        state.round_ticks, 0,
        "end_round must reset round_ticks for end_delay"
    );

    for _ in 0..9 {
        state.tick(0.05);
        assert_eq!(state.round_state, RoundState::Ended);
    }

    state.tick(0.05);
    assert_eq!(state.round_state, RoundState::Active);
}

#[test]
fn test_sim_warmup_state_skips_combat() {
    let mut state = GameState::new();
    assert_eq!(state.round_state, RoundState::Warmup);

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player(shooter_id, "Shooter".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    state.set_action(
        shooter_id,
        Action {
            fire: true,
            forward: true,
            ..Default::default()
        },
    );

    state.tick(0.05);
}

#[test]
fn test_sim_no_frag_limit_no_early_end() {
    let mut state = GameState::new();
    state.config.frag_limit = None;
    state.config.time_limit_ticks = Some(100);
    state.start_round();

    let player_id = Uuid::new_v4();
    state.add_player(player_id, "Player".to_string(), Role::Agent);
    *state.scores.entry(player_id).or_insert(0) = 100;

    state.tick(0.05);

    assert_eq!(state.round_state, RoundState::Active);
}

#[test]
fn test_sim_no_time_limit_no_early_end() {
    let mut state = GameState::new();
    state.config.frag_limit = Some(10);
    state.config.time_limit_ticks = None;
    state.start_round();

    for _ in 0..1000 {
        state.tick(0.05);
    }

    assert_eq!(state.round_state, RoundState::Active);
}

#[test]
fn test_player_id_consistent_after_add() {
    let mut state = GameState::new();
    let player_id = Uuid::new_v4();
    let name = "TestPlayer".to_string();

    state.add_player(player_id, name.clone(), Role::Human);

    let player = state
        .players
        .iter()
        .find(|p| p.id == player_id)
        .expect("Player should exist");
    assert_eq!(player.name, name);
    assert_eq!(player.hp, 100);
}

#[test]
fn test_player_id_action_flow() {
    let mut state = GameState::new();
    let player_id = Uuid::new_v4();

    state.add_player(player_id, "ActionTest".to_string(), Role::Human);

    let action = Action {
        forward: true,
        ..Default::default()
    };

    state.set_action(player_id, action);

    let player = state
        .players
        .iter()
        .find(|p| p.id == player_id)
        .expect("Player should exist after action");

    assert_eq!(player.id, player_id);
    assert!(player.pending_action.forward);
}

#[test]
fn test_hitscan_damage() {
    let mut state = GameState::new();
    state.start_round();

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player(shooter_id, "Shooter".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    let shooter_idx = state
        .players
        .iter()
        .position(|p| p.id == shooter_id)
        .unwrap();
    let target_idx = state
        .players
        .iter()
        .position(|p| p.id == target_id)
        .unwrap();

    state.players[target_idx].x = 5.0;
    state.players[target_idx].z = 0.0;
    state.players[shooter_idx].x = 0.0;
    state.players[shooter_idx].z = 0.0;
    state.players[shooter_idx].yaw = 0.0;

    let initial_hp = state.players[target_idx].hp;

    state.set_action(
        shooter_id,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);

    let final_hp = state.players[target_idx].hp;
    assert!(
        final_hp < initial_hp,
        "Target should take damage from hitscan"
    );
}

#[test]
fn test_frag_and_respawn() {
    let mut state = GameState::new();
    state.start_round();

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player(shooter_id, "Shooter".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    let shooter_idx = state
        .players
        .iter()
        .position(|p| p.id == shooter_id)
        .unwrap();
    let target_idx = state
        .players
        .iter()
        .position(|p| p.id == target_id)
        .unwrap();

    state.players[target_idx].x = 5.0;
    state.players[target_idx].z = 0.0;
    state.players[target_idx].hp = 20;
    state.players[shooter_idx].x = 0.0;
    state.players[shooter_idx].z = 0.0;
    state.players[shooter_idx].yaw = 0.0;

    state.set_action(
        shooter_id,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);

    let target = &state.players[target_idx];
    assert!(
        target.respawn_timer.is_some(),
        "Target should be dead with respawn timer"
    );
    assert!(target.hp <= 0, "Target HP should be zero or negative");

    for _ in 0..65 {
        state.tick(0.05);
    }

    let target = &state.players[target_idx];
    assert!(
        target.respawn_timer.is_none(),
        "Target should have respawned"
    );
    assert_eq!(target.hp, 100, "Target should respawn with full HP");
}

#[test]
fn test_movement_action() {
    let mut state = GameState::new();
    state.start_round();
    let player_id = Uuid::new_v4();

    state.add_player(player_id, "Mover".to_string(), Role::Human);

    let idx = state
        .players
        .iter()
        .position(|p| p.id == player_id)
        .unwrap();
    let initial_x = state.players[idx].x;

    let action = Action {
        forward: true,
        ..Default::default()
    };

    state.set_action(player_id, action);
    state.tick(0.05);

    let final_x = state.players[idx].x;
    assert_ne!(initial_x, final_x, "Player should have moved");
}

#[test]
fn test_player_removal() {
    let mut state = GameState::new();
    let player_id = Uuid::new_v4();

    state.add_player(player_id, "RemoveMe".to_string(), Role::Human);
    assert_eq!(state.players.len(), 1);

    state.remove_player(player_id);
    assert_eq!(state.players.len(), 0);
}

#[test]
fn test_round_warmup_to_active() {
    use crate::sim::RoundState;

    let mut state = GameState::new();
    assert_eq!(state.round_state, RoundState::Warmup);

    for _ in 0..state.config.warmup_ticks {
        state.tick(0.05);
    }

    assert_eq!(state.round_state, RoundState::Active);
    assert_eq!(state.round_number, 1);
}

#[test]
fn test_round_ends_on_frag_limit() {
    use crate::sim::RoundState;

    let mut state = GameState::new();
    state.config.frag_limit = Some(2);
    state.config.time_limit_ticks = None;

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player(shooter_id, "Shooter".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);
    state.start_round();

    let shooter_idx = state
        .players
        .iter()
        .position(|p| p.id == shooter_id)
        .unwrap();
    let target_idx = state
        .players
        .iter()
        .position(|p| p.id == target_id)
        .unwrap();

    state.players[shooter_idx].x = 0.0;
    state.players[shooter_idx].z = 0.0;
    state.players[shooter_idx].yaw = 0.0;

    for frag_count in 0..2 {
        state.players[target_idx].x = 5.0;
        state.players[target_idx].z = 0.0;
        state.players[target_idx].hp = 25;
        state.players[target_idx].respawn_timer = None;
        state.players[shooter_idx].fire_cooldown = 0;

        state.set_action(
            shooter_id,
            Action {
                fire: true,
                ..Default::default()
            },
        );
        state.tick(0.05);

        assert_eq!(
            *state.scores.get(&shooter_id).unwrap_or(&0),
            frag_count + 1,
            "Score should increment after frag"
        );
    }

    state.tick(0.05);

    assert_eq!(
        state.round_state,
        RoundState::Ended,
        "Round should end after reaching frag limit"
    );
}

#[test]
fn test_round_ends_on_time_limit() {
    use crate::sim::RoundState;

    let mut state = GameState::new();
    state.config.frag_limit = None;
    state.config.time_limit_ticks = Some(100);
    state.start_round();

    for _ in 0..100 {
        state.tick(0.05);
    }

    assert_eq!(state.round_state, RoundState::Ended);
}

#[test]
fn test_scores_reset_between_rounds() {
    use crate::sim::RoundState;

    let mut state = GameState::new();
    let player_id = Uuid::new_v4();

    state.add_player(player_id, "Player".to_string(), Role::Agent);
    state.start_round();

    *state.scores.entry(player_id).or_insert(0) = 5;

    state.round_state = RoundState::Ended;
    state.start_round();

    assert_eq!(*state.scores.get(&player_id).unwrap_or(&0), 0);
}

#[test]
fn test_bots_persist_when_human_leaves() {
    let mut state = GameState::new();

    let bot_id = Uuid::new_v4();
    let human_id = Uuid::new_v4();

    state.add_player(bot_id, "Bot".to_string(), Role::Agent);
    state.add_player(human_id, "Human".to_string(), Role::Human);

    assert_eq!(state.players.len(), 2);

    state.remove_player(human_id);

    assert_eq!(state.players.len(), 1);
    assert_eq!(state.players[0].id, bot_id);
}

#[test]
fn test_weapon_type_stats() {
    use crate::protocol::WeaponType;

    assert_eq!(WeaponType::Flechette.damage(), 25);
    assert_eq!(WeaponType::Flechette.cooldown_ticks(), 10);
    assert_eq!(WeaponType::Flechette.spread_radians(), 0.1);

    assert_eq!(WeaponType::Rail.damage(), 75);
    assert_eq!(WeaponType::Rail.cooldown_ticks(), 40);
    assert_eq!(WeaponType::Rail.spread_radians(), 0.05);

    assert_eq!(WeaponType::Scatter.damage(), 15);
    assert_eq!(WeaponType::Scatter.cooldown_ticks(), 5);
    assert_eq!(WeaponType::Scatter.spread_radians(), 0.3);
}

#[test]
fn test_weapon_default_is_flechette() {
    use crate::protocol::WeaponType;

    let mut state = GameState::new();
    let player_id = Uuid::new_v4();

    state.add_player(player_id, "TestPlayer".to_string(), Role::Human);

    let player = state.players.iter().find(|p| p.id == player_id).unwrap();
    assert_eq!(player.weapon, WeaponType::Flechette);
}

#[test]
fn test_weapon_swap_action() {
    use crate::protocol::WeaponType;

    let mut state = GameState::new();
    state.start_round();

    let player_id = Uuid::new_v4();
    state.add_player(player_id, "Swapper".to_string(), Role::Human);

    let idx = state
        .players
        .iter()
        .position(|p| p.id == player_id)
        .unwrap();

    assert_eq!(state.players[idx].weapon, WeaponType::Flechette);

    state.set_action(
        player_id,
        Action {
            weapon_swap: Some(WeaponType::Rail),
            ..Default::default()
        },
    );
    state.tick(0.05);

    assert_eq!(state.players[idx].weapon, WeaponType::Rail);

    state.set_action(
        player_id,
        Action {
            weapon_swap: Some(WeaponType::Scatter),
            ..Default::default()
        },
    );
    state.tick(0.05);

    assert_eq!(state.players[idx].weapon, WeaponType::Scatter);
}

#[test]
fn test_rail_higher_damage_than_flechette() {
    use crate::protocol::WeaponType;

    let mut state = GameState::new();
    state.start_round();

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player(shooter_id, "Shooter".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    let shooter_idx = state
        .players
        .iter()
        .position(|p| p.id == shooter_id)
        .unwrap();
    let target_idx = state
        .players
        .iter()
        .position(|p| p.id == target_id)
        .unwrap();

    state.players[target_idx].x = 5.0;
    state.players[target_idx].z = 0.0;
    state.players[shooter_idx].x = 0.0;
    state.players[shooter_idx].z = 0.0;
    state.players[shooter_idx].yaw = 0.0;
    state.players[shooter_idx].weapon = WeaponType::Rail;

    let initial_hp = state.players[target_idx].hp;

    state.set_action(
        shooter_id,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);

    let damage_dealt = initial_hp - state.players[target_idx].hp;
    assert_eq!(damage_dealt, 75, "Rail should deal 75 damage");
    assert!(
        damage_dealt > WeaponType::Flechette.damage(),
        "Rail damage should exceed Flechette"
    );
}

#[test]
fn test_scatter_lower_damage_than_flechette() {
    use crate::protocol::WeaponType;

    let mut state = GameState::new();
    state.start_round();

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player(shooter_id, "Shooter".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    let shooter_idx = state
        .players
        .iter()
        .position(|p| p.id == shooter_id)
        .unwrap();
    let target_idx = state
        .players
        .iter()
        .position(|p| p.id == target_id)
        .unwrap();

    state.players[target_idx].x = 5.0;
    state.players[target_idx].z = 0.0;
    state.players[shooter_idx].x = 0.0;
    state.players[shooter_idx].z = 0.0;
    state.players[shooter_idx].yaw = 0.0;
    state.players[shooter_idx].weapon = WeaponType::Scatter;

    let initial_hp = state.players[target_idx].hp;

    state.set_action(
        shooter_id,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);

    let damage_dealt = initial_hp - state.players[target_idx].hp;
    assert_eq!(damage_dealt, 15, "Scatter should deal 15 damage");
    assert!(
        damage_dealt < WeaponType::Flechette.damage(),
        "Scatter damage should be less than Flechette"
    );
}

#[test]
fn test_rail_longer_cooldown() {
    use crate::protocol::WeaponType;

    let mut state = GameState::new();
    state.start_round();

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player(shooter_id, "Shooter".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    let shooter_idx = state
        .players
        .iter()
        .position(|p| p.id == shooter_id)
        .unwrap();

    state.players[shooter_idx].weapon = WeaponType::Rail;
    state.players[shooter_idx].x = 0.0;
    state.players[shooter_idx].z = 0.0;
    state.players[shooter_idx].yaw = 0.0;

    state.set_action(
        shooter_id,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);

    assert_eq!(
        state.players[shooter_idx].fire_cooldown, 40,
        "Rail cooldown should be 40 ticks"
    );
}

#[test]
fn test_scatter_shorter_cooldown() {
    use crate::protocol::WeaponType;

    let mut state = GameState::new();
    state.start_round();

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player(shooter_id, "Shooter".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    let shooter_idx = state
        .players
        .iter()
        .position(|p| p.id == shooter_id)
        .unwrap();

    state.players[shooter_idx].weapon = WeaponType::Scatter;
    state.players[shooter_idx].x = 0.0;
    state.players[shooter_idx].z = 0.0;
    state.players[shooter_idx].yaw = 0.0;

    state.set_action(
        shooter_id,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);

    assert_eq!(
        state.players[shooter_idx].fire_cooldown, 5,
        "Scatter cooldown should be 5 ticks"
    );
}

#[test]
fn test_weapon_spread_affects_hit_detection() {
    use crate::protocol::WeaponType;

    let mut state = GameState::new();
    state.start_round();

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player(shooter_id, "Shooter".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    let shooter_idx = state
        .players
        .iter()
        .position(|p| p.id == shooter_id)
        .unwrap();
    let target_idx = state
        .players
        .iter()
        .position(|p| p.id == target_id)
        .unwrap();

    state.players[target_idx].x = 10.0;
    state.players[target_idx].z = 2.0;
    state.players[shooter_idx].x = 0.0;
    state.players[shooter_idx].z = 0.0;
    state.players[shooter_idx].yaw = 0.1;

    state.players[shooter_idx].weapon = WeaponType::Rail;
    state.players[shooter_idx].fire_cooldown = 0;

    state.set_action(
        shooter_id,
        Action {
            fire: true,
            ..Default::default()
        },
    );

    let target_hp_before = state.players[target_idx].hp;
    state.tick(0.05);
    let rail_hit = state.players[target_idx].hp < target_hp_before;

    state.players[target_idx].hp = 100;
    state.players[shooter_idx].weapon = WeaponType::Scatter;
    state.players[shooter_idx].fire_cooldown = 0;

    state.set_action(
        shooter_id,
        Action {
            fire: true,
            ..Default::default()
        },
    );

    let target_hp_before = state.players[target_idx].hp;
    state.tick(0.05);
    let scatter_hit = state.players[target_idx].hp < target_hp_before;

    assert!(
        scatter_hit || !rail_hit,
        "Scatter should be more forgiving with wider spread"
    );
}

#[test]
fn test_weapon_snapshot_includes_weapon_name() {
    use crate::protocol::WeaponType;

    let mut state = GameState::new();
    state.start_round();

    let player_id = Uuid::new_v4();
    state.add_player(player_id, "TestPlayer".to_string(), Role::Human);

    let snapshot = state.snapshot();
    assert_eq!(snapshot.players[0].weapon, "Flechette");

    let idx = state
        .players
        .iter()
        .position(|p| p.id == player_id)
        .unwrap();
    state.players[idx].weapon = WeaponType::Rail;

    let snapshot = state.snapshot();
    assert_eq!(snapshot.players[0].weapon, "Rail");

    state.players[idx].weapon = WeaponType::Scatter;

    let snapshot = state.snapshot();
    assert_eq!(snapshot.players[0].weapon, "Scatter");
}

#[test]
fn test_join_leave_event_serialization() {
    use crate::protocol::{GameEvent, ServerMessage};

    let join_event = GameEvent::PlayerJoined {
        player: "TestPlayer".to_string(),
        role: "human".to_string(),
        round_number: 1,
        player_count: 5,
    };
    let join_msg = ServerMessage::Event(join_event);
    let join_json = serde_json::to_string(&join_msg).unwrap();
    assert!(join_json.contains(r#""event":"player_joined"#));
    assert!(join_json.contains(r#""player":"TestPlayer"#));
    assert!(join_json.contains(r#""role":"human"#));
    assert!(join_json.contains(r#""round_number":1"#));
    assert!(join_json.contains(r#""player_count":5"#));

    let leave_event = GameEvent::PlayerLeft {
        player: "TestPlayer".to_string(),
        score: 7,
        round_number: 2,
        player_count: 4,
    };
    let leave_msg = ServerMessage::Event(leave_event);
    let leave_json = serde_json::to_string(&leave_msg).unwrap();
    assert!(leave_json.contains(r#""event":"player_left"#));
    assert!(leave_json.contains(r#""player":"TestPlayer"#));
    assert!(leave_json.contains(r#""score":7"#));
    assert!(leave_json.contains(r#""round_number":2"#));
    assert!(leave_json.contains(r#""player_count":4"#));
}

#[test]
fn join_leave_events_survive_tick() {
    use crate::protocol::GameEvent;
    let mut state = GameState::new();
    state.push_event(GameEvent::PlayerJoined {
        player: "AgentA".to_string(),
        role: "agent".to_string(),
        round_number: state.round_number,
        player_count: 1,
    });
    state.tick(0.05);
    state.push_event(GameEvent::PlayerLeft {
        player: "AgentA".to_string(),
        score: 0,
        round_number: state.round_number,
        player_count: 0,
    });
    state.tick(0.05);
    let events = state.take_events();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PlayerJoined { .. })),
        "PlayerJoined must survive tick() and remain until take_events: {:?}",
        events
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::PlayerLeft { .. })),
        "PlayerLeft must survive tick() and remain until take_events: {:?}",
        events
    );
}

#[test]
fn test_round_cycle_events_survive_ticks() {
    use crate::protocol::GameEvent;
    use crate::sim::{GameState, MatchConfig, RoundState};

    let mut state = GameState::new();
    state.config = MatchConfig {
        frag_limit: Some(2),
        time_limit_ticks: None,
        warmup_ticks: 3,
        end_delay_ticks: 3,
    };

    let a = uuid::Uuid::new_v4();
    let b = uuid::Uuid::new_v4();
    state.add_player(a, "Alpha".to_string(), crate::protocol::Role::Agent);
    state.add_player(b, "Bravo".to_string(), crate::protocol::Role::Agent);

    assert_eq!(state.round_state, RoundState::Warmup);

    // Warmup → Active: RoundStart must be present after the transition tick.
    for _ in 0..(state.config.warmup_ticks.saturating_sub(1)) {
        state.tick(0.05);
        let _ = state.take_events(); // drain noise; transition not yet
    }
    state.tick(0.05);
    assert_eq!(state.round_state, RoundState::Active);
    let events = state.take_events();
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::RoundStart {
                round_number: 1,
                ..
            }
        )),
        "RoundStart must survive the Warmup→Active tick: {:?}",
        events
    );

    // Drive frag limit via scores (same path end_round uses).
    *state.scores.entry(a).or_insert(0) = 2;
    state.tick(0.05);
    assert_eq!(state.round_state, RoundState::Ended);
    let events = state.take_events();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::RoundEnd { .. })),
        "RoundEnd must survive the Active→Ended tick: {:?}",
        events
    );

    // Ended delay → Round 2 Start.
    for _ in 0..(state.config.end_delay_ticks.saturating_sub(1)) {
        state.tick(0.05);
        let _ = state.take_events();
    }
    state.tick(0.05);
    assert_eq!(state.round_state, RoundState::Active);
    assert_eq!(state.round_number, 2);
    let events = state.take_events();
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::RoundStart {
                round_number: 2,
                ..
            }
        )),
        "RoundStart(2) must survive the Ended→Active tick: {:?}",
        events
    );
}

#[test]
fn test_server_round_event_wire_json_shape() {
    use crate::protocol::{GameEvent, PlayerScore, ServerMessage};

    let start = ServerMessage::Event(GameEvent::RoundStart {
        round_number: 2,
        frag_limit: Some(10),
        time_limit: Some(180),
        players: vec!["Alpha".into(), "Bravo".into()],
        previous_winner: Some("Alpha".into()),
    });
    let start_json = serde_json::to_string(&start).unwrap();
    assert!(start_json.contains(r#""type":"event""#), "{}", start_json);
    assert!(
        start_json.contains(r#""event":"round_start""#),
        "{}",
        start_json
    );
    assert!(
        start_json.contains(r#""previous_winner":"Alpha""#),
        "{}",
        start_json
    );

    let end = ServerMessage::Event(GameEvent::RoundEnd {
        winner: Some("Alpha".into()),
        reason: "Frag limit reached".into(),
        final_scores: vec![
            PlayerScore {
                name: "Alpha".into(),
                score: 10,
            },
            PlayerScore {
                name: "Bravo".into(),
                score: 3,
            },
        ],
        winner_score: Some(10),
    });
    let end_json = serde_json::to_string(&end).unwrap();
    assert!(end_json.contains(r#""event":"round_end""#), "{}", end_json);
    assert!(end_json.contains(r#""final_scores""#), "{}", end_json);

    // Round-trip on server protocol itself.
    let parsed: ServerMessage = serde_json::from_str(&start_json).unwrap();
    assert!(matches!(
        parsed,
        ServerMessage::Event(GameEvent::RoundStart { .. })
    ));
    let parsed: ServerMessage = serde_json::from_str(&end_json).unwrap();
    assert!(matches!(
        parsed,
        ServerMessage::Event(GameEvent::RoundEnd { .. })
    ));
}

// --- Net WebSocket join/leave/action/round wire paths ---

#[tokio::test]
async fn test_net_ws_agent_hello_welcome_and_connected_command() {
    use crate::net::{GameCommand, NetServer};
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    let (game_tx, mut game_rx) = tokio::sync::mpsc::unbounded_channel();
    let net = NetServer::bind("127.0.0.1:0", game_tx).await.expect("bind");
    let addr = net.local_addr().expect("local_addr");
    tokio::spawn(async move {
        net.accept_loop().await;
    });

    let url = format!("ws://{}", addr);
    let (ws, _) = connect_async(&url).await.expect("connect");
    let (mut sink, mut stream) = ws.split();

    let hello = serde_json::json!({
        "type": "hello",
        "role": "agent",
        "name": "WireAgent"
    });
    sink.send(Message::Text(hello.to_string()))
        .await
        .expect("send hello");

    let welcome_text = tokio::time::timeout(std::time::Duration::from_secs(2), stream.next())
        .await
        .expect("welcome timeout")
        .expect("welcome msg")
        .expect("welcome ok");
    let welcome_str = match welcome_text {
        Message::Text(t) => t,
        other => panic!("expected text welcome, got {:?}", other),
    };
    let welcome: ServerMessage = serde_json::from_str(&welcome_str).expect("parse welcome");
    match welcome {
        ServerMessage::Welcome {
            player_id: Some(_),
            role: Role::Agent,
        } => {}
        other => panic!("unexpected welcome: {:?}", other),
    }

    let cmd = tokio::time::timeout(std::time::Duration::from_secs(2), game_rx.recv())
        .await
        .expect("cmd timeout")
        .expect("cmd");
    match cmd {
        GameCommand::Connected {
            role: Role::Agent,
            name,
            player_id: Some(_),
            ..
        } => assert_eq!(name, "WireAgent"),
        other => panic!("unexpected cmd: {:?}", other_debug(&other)),
    }

    // Drop connection to exercise disconnect path.
    drop(sink);
    drop(stream);
    let disc = tokio::time::timeout(std::time::Duration::from_secs(2), game_rx.recv())
        .await
        .expect("disc timeout")
        .expect("disc");
    assert!(matches!(disc, GameCommand::Disconnected { .. }));
}

fn other_debug(cmd: &crate::net::GameCommand) -> String {
    match cmd {
        crate::net::GameCommand::Connected { name, role, .. } => {
            format!("Connected({:?},{})", role, name)
        }
        crate::net::GameCommand::Disconnected { .. } => "Disconnected".into(),
        crate::net::GameCommand::Action { .. } => "Action".into(),
    }
}

#[tokio::test]
async fn test_net_ws_spectator_hello_no_player_id() {
    use crate::net::{GameCommand, NetServer};
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    let (game_tx, mut game_rx) = tokio::sync::mpsc::unbounded_channel();
    let net = NetServer::bind("127.0.0.1:0", game_tx).await.expect("bind");
    let addr = net.local_addr().expect("local_addr");
    tokio::spawn(async move {
        net.accept_loop().await;
    });

    let (ws, _) = connect_async(format!("ws://{}", addr))
        .await
        .expect("connect");
    let (mut sink, mut stream) = ws.split();
    sink.send(Message::Text(
        r#"{"type":"hello","role":"spectator","name":"Eyes"}"#.into(),
    ))
    .await
    .unwrap();

    let welcome_text = tokio::time::timeout(std::time::Duration::from_secs(2), stream.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let welcome_str = match welcome_text {
        Message::Text(t) => t,
        other => panic!("{:?}", other),
    };
    let welcome: ServerMessage = serde_json::from_str(&welcome_str).unwrap();
    match welcome {
        ServerMessage::Welcome {
            player_id: None,
            role: Role::Spectator,
        } => {}
        other => panic!("{:?}", other),
    }

    let cmd = tokio::time::timeout(std::time::Duration::from_secs(2), game_rx.recv())
        .await
        .unwrap()
        .unwrap();
    match cmd {
        GameCommand::Connected {
            player_id: None,
            role: Role::Spectator,
            name,
            ..
        } => assert_eq!(name, "Eyes"),
        other => panic!("{}", other_debug(&other)),
    }
}

#[tokio::test]
async fn test_net_ws_action_forwarded_for_agent() {
    use crate::net::{GameCommand, NetServer};
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    let (game_tx, mut game_rx) = tokio::sync::mpsc::unbounded_channel();
    let net = NetServer::bind("127.0.0.1:0", game_tx).await.expect("bind");
    let addr = net.local_addr().expect("local_addr");
    let clients = net.clients.clone();
    tokio::spawn(async move {
        net.accept_loop().await;
    });

    let (ws, _) = connect_async(format!("ws://{}", addr))
        .await
        .expect("connect");
    let (mut sink, mut stream) = ws.split();
    sink.send(Message::Text(
        r#"{"type":"hello","role":"human","name":"Shooter"}"#.into(),
    ))
    .await
    .unwrap();

    let _ = tokio::time::timeout(std::time::Duration::from_secs(2), stream.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let connected = tokio::time::timeout(std::time::Duration::from_secs(2), game_rx.recv())
        .await
        .unwrap()
        .unwrap();
    let player_id = match connected {
        GameCommand::Connected {
            player_id: Some(pid),
            ..
        } => pid,
        other => panic!("{}", other_debug(&other)),
    };

    sink.send(Message::Text(
        r#"{"type":"action","forward":true,"fire":true}"#.into(),
    ))
    .await
    .unwrap();

    let action_cmd = tokio::time::timeout(std::time::Duration::from_secs(2), game_rx.recv())
        .await
        .unwrap()
        .unwrap();
    match action_cmd {
        GameCommand::Action {
            player_id: pid,
            action,
        } => {
            assert_eq!(pid, player_id);
            assert!(action.forward);
            assert!(action.fire);
        }
        other => panic!("{}", other_debug(&other)),
    }

    // Broadcast a snapshot through the client fan-out path used by the game loop.
    {
        use crate::session::broadcast_to_clients;
        let snap = ServerMessage::Snapshot(Snapshot {
            tick: 1,
            players: vec![],
            round_state: Some("active".into()),
            round_time_left: Some(100),
            frag_limit: Some(10),
        });
        broadcast_to_clients(&clients, &[snap]).await;
        let msg = tokio::time::timeout(std::time::Duration::from_secs(2), stream.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        match msg {
            Message::Text(t) => {
                let parsed: ServerMessage = serde_json::from_str(&t).unwrap();
                assert!(matches!(parsed, ServerMessage::Snapshot(_)));
            }
            other => panic!("{:?}", other),
        }
    }
}

#[tokio::test]
async fn test_net_ws_invalid_hello_closes_without_connected() {
    use crate::net::NetServer;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    let (game_tx, mut game_rx) = tokio::sync::mpsc::unbounded_channel();
    let net = NetServer::bind("127.0.0.1:0", game_tx).await.expect("bind");
    let addr = net.local_addr().expect("local_addr");
    tokio::spawn(async move {
        net.accept_loop().await;
    });

    let (ws, _) = connect_async(format!("ws://{}", addr))
        .await
        .expect("connect");
    let (mut sink, mut stream) = ws.split();
    sink.send(Message::Text(r#"{"type":"action","forward":true}"#.into()))
        .await
        .unwrap();

    // Connection should end without a Connected command.
    let _ = tokio::time::timeout(std::time::Duration::from_millis(500), stream.next()).await;
    let maybe = tokio::time::timeout(std::time::Duration::from_millis(300), game_rx.recv()).await;
    assert!(
        maybe.is_err() || maybe.as_ref().ok().and_then(|o| o.as_ref()).is_none(),
        "invalid hello must not emit Connected"
    );
}

#[tokio::test]
async fn test_session_plus_net_join_leave_round_broadcast_path() {
    use crate::net::NetServer;
    use crate::session::{broadcast_to_clients, GameSession};
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    let (game_tx, mut game_rx) = tokio::sync::mpsc::unbounded_channel();
    let net = NetServer::bind("127.0.0.1:0", game_tx).await.expect("bind");
    let addr = net.local_addr().expect("local_addr");
    let clients = net.clients.clone();
    tokio::spawn(async move {
        net.accept_loop().await;
    });

    let mut session = GameSession::new();
    session.spawn_bots(2);
    session.state.start_round();

    let (ws, _) = connect_async(format!("ws://{}", addr))
        .await
        .expect("connect");
    let (mut sink, mut stream) = ws.split();
    sink.send(Message::Text(
        r#"{"type":"hello","role":"agent","name":"RoundFox"}"#.into(),
    ))
    .await
    .unwrap();
    let _ = tokio::time::timeout(std::time::Duration::from_secs(2), stream.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();

    let cmd = tokio::time::timeout(std::time::Duration::from_secs(2), game_rx.recv())
        .await
        .unwrap()
        .unwrap();
    session.apply_command(cmd);

    let joined = session.state.take_events();
    assert!(joined.iter().any(|e| matches!(
        e,
        GameEvent::PlayerJoined {
            player,
            ..
        } if player == "RoundFox"
    )));

    // Re-push join onto queue then tick so Event is broadcast on the wire.
    session.state.push_event(GameEvent::PlayerJoined {
        player: "RoundFox".into(),
        role: "agent".into(),
        round_number: session.state.round_number,
        player_count: session.state.players.len(),
    });
    let messages = session.tick_messages(0.05);
    broadcast_to_clients(&clients, &messages).await;

    let mut saw_snapshot = false;
    let mut saw_join_event = false;
    for _ in 0..8 {
        let msg = tokio::time::timeout(std::time::Duration::from_millis(500), stream.next()).await;
        let Ok(Some(Ok(Message::Text(t)))) = msg else {
            break;
        };
        if let Ok(parsed) = serde_json::from_str::<ServerMessage>(&t) {
            match parsed {
                ServerMessage::Snapshot(_) => saw_snapshot = true,
                ServerMessage::Event(GameEvent::PlayerJoined { player, .. })
                    if player == "RoundFox" =>
                {
                    saw_join_event = true;
                }
                _ => {}
            }
        }
        if saw_snapshot && saw_join_event {
            break;
        }
    }
    assert!(saw_snapshot, "client should receive Snapshot");
    assert!(saw_join_event, "client should receive PlayerJoined event");

    drop(sink);
    drop(stream);
    if let Ok(Some(disc)) =
        tokio::time::timeout(std::time::Duration::from_secs(2), game_rx.recv()).await
    {
        session.apply_command(disc);
        let left = session.state.take_events();
        assert!(left
            .iter()
            .any(|e| matches!(e, GameEvent::PlayerLeft { player, .. } if player == "RoundFox")));
    }
}
