#[cfg(test)]
use crate::protocol::{
    boss_down_host_line, boss_host_line, boss_round_wipe_host_line, compliance_host_line,
    default_host_line, default_map_id, default_map_name, default_mode_name, default_playlist,
    mvp_host_line, round_open_host_line, warmup_host_line, Action, ClientMessage, GameEvent,
    PlayerScore, PlayerState, Role, ServerMessage, Snapshot, WeaponType, AUDITOR_NAME, BOSS_NAME,
    EPISODE_ID_EP0, EPISODE_MAP_LARAK_LOT,
};
#[cfg(test)]
use crate::session::GameSession;
#[cfg(test)]
use crate::sim::{
    BotBehavior, BotController, EpisodePhase, GameState, MapKind, MatchConfig, RoundState,
    BOSS_MAX_HP, HEALTH_PICKUP_RESPAWN_TICKS, PICKUP_RESPAWN_TICKS,
};

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
        mode_name: default_mode_name(),
        playlist: default_playlist(),
    };
    let json = serde_json::to_string(&welcome).unwrap();
    assert!(json.contains(r#""type":"welcome""#));
    assert!(json.contains(r#""role":"human""#));

    let welcome_spectator = ServerMessage::Welcome {
        player_id: None,
        role: Role::Spectator,
        mode_name: default_mode_name(),
        playlist: default_playlist(),
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
        mode_name: default_mode_name(),
        playlist: default_playlist(),
        host_line: default_host_line(),
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
        mvp: Some("Bot1".to_string()),
        mvp_frags: Some(10),
        host_line: mvp_host_line("Bot1", 10),
    };
    let json = serde_json::to_value(&event).unwrap();
    assert_eq!(json["event"], "round_end");
    assert_eq!(json["winner"], "Bot1");
    assert_eq!(json["mvp"], "Bot1");
    assert_eq!(json["mvp_frags"], 10);
    assert!(json["host_line"].as_str().unwrap().contains("ROUND MVP"));
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
            armor: 0,
            just_fired: false,
            behavior: None,
            score: 5,
            weapon: "Flechette".to_string(),
        }],
        round_state: Some("Active".to_string()),
        round_time_left: Some(60),
        frag_limit: Some(10),
        shot_results: vec![],
        mode_name: default_mode_name(),
        playlist: default_playlist(),
        pressure: None,
        host_line: default_host_line(),
        mvp: None,
        mvp_frags: None,
        pickups: vec![],
        map_id: default_map_id(),
        map_name: default_map_name(),
        episode_id: None,
        episode_title: None,
        episode_objective: None,
        episode_progress: None,
        episode_phase: None,
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
        shot_results: vec![],
        mode_name: default_mode_name(),
        playlist: default_playlist(),
        pressure: None,
        host_line: default_host_line(),
        mvp: None,
        mvp_frags: None,
        pickups: vec![],
        map_id: default_map_id(),
        map_name: default_map_name(),
        episode_id: None,
        episode_title: None,
        episode_objective: None,
        episode_progress: None,
        episode_phase: None,
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
        compliance_ping_ticks: None,
        compliance_duration_ticks: 20 * 6,
        boss_spawn_ticks: None,
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
    assert_eq!(WeaponType::Flechette.spread_radians(), 0.10);
    assert_eq!(WeaponType::Flechette.range_units(), 42.0);
    assert_eq!(WeaponType::Flechette.name(), "Flechette");

    assert_eq!(WeaponType::Rail.damage(), 75);
    assert_eq!(WeaponType::Rail.cooldown_ticks(), 40);
    assert_eq!(WeaponType::Rail.spread_radians(), 0.04);
    assert_eq!(WeaponType::Rail.range_units(), 100.0);
    assert_eq!(WeaponType::Rail.name(), "Rail");

    assert_eq!(WeaponType::Scatter.damage(), 15);
    assert_eq!(WeaponType::Scatter.cooldown_ticks(), 5);
    assert_eq!(WeaponType::Scatter.spread_radians(), 0.38);
    assert_eq!(WeaponType::Scatter.range_units(), 14.0);
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
        BotBehavior::Compliance,
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
    assert_eq!(WeaponType::Flechette.spread_radians(), 0.10);
    assert_eq!(WeaponType::Flechette.range_units(), 42.0);

    assert_eq!(WeaponType::Rail.damage(), 75);
    assert_eq!(WeaponType::Rail.cooldown_ticks(), 40);
    assert_eq!(WeaponType::Rail.spread_radians(), 0.04);
    assert_eq!(WeaponType::Rail.range_units(), 100.0);

    assert_eq!(WeaponType::Scatter.damage(), 15);
    assert_eq!(WeaponType::Scatter.cooldown_ticks(), 5);
    assert_eq!(WeaponType::Scatter.spread_radians(), 0.38);
    assert_eq!(WeaponType::Scatter.range_units(), 14.0);
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
        compliance_ping_ticks: None,
        compliance_duration_ticks: 20 * 6,
        boss_spawn_ticks: None,
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
        mode_name: default_mode_name(),
        playlist: default_playlist(),
        host_line: default_host_line(),
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
    assert!(
        start_json.contains(r#""mode_name":"Contested Frequency""#),
        "{}",
        start_json
    );
    assert!(
        start_json.contains(r#""playlist":"Arena Duel""#),
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
        mvp: Some("Alpha".into()),
        mvp_frags: Some(10),
        host_line: mvp_host_line("Alpha", 10),
    });
    let end_json = serde_json::to_string(&end).unwrap();
    assert!(end_json.contains(r#""event":"round_end""#), "{}", end_json);
    assert!(end_json.contains(r#""final_scores""#), "{}", end_json);
    assert!(end_json.contains(r#""mvp":"Alpha""#), "{}", end_json);
    assert!(end_json.contains("ROUND MVP"), "{}", end_json);

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
            mode_name,
            playlist,
        } => {
            assert_eq!(mode_name, default_mode_name());
            assert_eq!(playlist, default_playlist());
        }
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
        crate::net::GameCommand::Speak { .. } => "Speak".into(),
        crate::net::GameCommand::SetDisplayBehavior { .. } => "SetDisplayBehavior".into(),
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
            mode_name,
            playlist,
        } => {
            assert_eq!(mode_name, default_mode_name());
            assert_eq!(playlist, default_playlist());
        }
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
            shot_results: vec![],
            mode_name: default_mode_name(),
            playlist: default_playlist(),
            pressure: None,
            host_line: default_host_line(),
            mvp: None,
            mvp_frags: None,
            pickups: vec![],
            map_id: default_map_id(),
            map_name: default_map_name(),
            episode_id: None,
            episode_title: None,
            episode_objective: None,
            episode_progress: None,
            episode_phase: None,
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

#[test]
fn test_look_at_player_id_sets_yaw() {
    use std::f32::consts::PI;
    let mut state = GameState::new();
    state.start_round();

    let aimer_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();
    state.add_player(aimer_id, "Aimer".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    let aimer_idx = state.players.iter().position(|p| p.id == aimer_id).unwrap();
    let target_idx = state
        .players
        .iter()
        .position(|p| p.id == target_id)
        .unwrap();
    state.players[aimer_idx].x = 0.0;
    state.players[aimer_idx].z = 0.0;
    state.players[aimer_idx].yaw = 0.0;
    state.players[target_idx].x = 0.0;
    state.players[target_idx].z = 10.0;

    state.set_action(
        aimer_id,
        Action {
            look_at: Some(crate::protocol::LookAt {
                player_id: Some(target_id),
                x: None,
                z: None,
            }),
            ..Default::default()
        },
    );
    state.tick(0.05);

    let yaw = state.players[aimer_idx].yaw;
    let expected = PI / 2.0;
    let mut diff = (yaw - expected).abs();
    if diff > PI {
        diff = 2.0 * PI - diff;
    }
    assert!(
        diff < 0.05,
        "look_at player_id should face target, yaw={yaw} expected~{expected}"
    );
}

#[test]
fn test_look_at_world_xz_sets_yaw() {
    use std::f32::consts::PI;
    let mut state = GameState::new();
    state.start_round();

    let aimer_id = Uuid::new_v4();
    state.add_player(aimer_id, "Aimer".to_string(), Role::Agent);
    let aimer_idx = state.players.iter().position(|p| p.id == aimer_id).unwrap();
    state.players[aimer_idx].x = 0.0;
    state.players[aimer_idx].z = 0.0;
    state.players[aimer_idx].yaw = 0.0;

    state.set_action(
        aimer_id,
        Action {
            look_at: Some(crate::protocol::LookAt {
                player_id: None,
                x: Some(-10.0),
                z: Some(0.0),
            }),
            ..Default::default()
        },
    );
    state.tick(0.05);

    let yaw = state.players[aimer_idx].yaw;
    let expected = PI;
    let mut diff = (yaw - expected).abs();
    if diff > PI {
        diff = 2.0 * PI - diff;
    }
    assert!(
        diff < 0.05,
        "look_at x/z should face world point, yaw={yaw} expected~{expected}"
    );
}

#[test]
fn test_shot_results_hit_and_hit_event() {
    let mut state = GameState::new();
    state.start_round();

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();
    state.add_player(shooter_id, "Shooter".to_string(), Role::Agent);
    state.add_player(target_id, "Victim".to_string(), Role::Agent);

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
    state.players[target_idx].x = 5.0;
    state.players[target_idx].z = 0.0;

    let hp_before = state.players[target_idx].hp;

    state.set_action(
        shooter_id,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);

    let snap = state.snapshot();
    assert_eq!(snap.shot_results.len(), 1, "one shot_result expected");
    let shot = &snap.shot_results[0];
    assert_eq!(shot.shooter_id, shooter_id);
    assert!(shot.hit, "aligned flechette should hit");
    assert_eq!(shot.target_id, Some(target_id));
    assert_eq!(shot.damage, WeaponType::Flechette.damage());
    assert_eq!(
        shot.target_hp_after,
        Some(hp_before - WeaponType::Flechette.damage())
    );

    let hit_events: Vec<_> = state
        .events
        .iter()
        .filter(|e| matches!(e, GameEvent::Hit { .. }))
        .collect();
    assert_eq!(hit_events.len(), 1, "Hit event on damage");
    match &hit_events[0] {
        GameEvent::Hit {
            damage,
            target_hp_after,
            target_id: tid,
            ..
        } => {
            assert_eq!(*damage, WeaponType::Flechette.damage());
            assert_eq!(*target_hp_after, hp_before - WeaponType::Flechette.damage());
            assert_eq!(*tid, target_id);
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_shot_results_miss() {
    use std::f32::consts::PI;
    let mut state = GameState::new();
    state.start_round();

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();
    state.add_player(shooter_id, "Shooter".to_string(), Role::Agent);
    state.add_player(target_id, "Victim".to_string(), Role::Agent);

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
    state.players[shooter_idx].yaw = PI / 2.0;
    state.players[target_idx].x = 5.0;
    state.players[target_idx].z = 0.0;

    state.set_action(
        shooter_id,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);

    let snap = state.snapshot();
    assert_eq!(snap.shot_results.len(), 1);
    let shot = &snap.shot_results[0];
    assert!(!shot.hit);
    assert!(shot.target_id.is_none());
    assert_eq!(shot.damage, 0);
    assert!(
        !state
            .events
            .iter()
            .any(|e| matches!(e, GameEvent::Hit { .. })),
        "no Hit event on miss"
    );
}

#[test]
fn test_snapshot_carries_contested_frequency_mode_identity() {
    let state = GameState::new();
    let snap = state.snapshot();
    assert_eq!(snap.mode_name, "Contested Frequency");
    assert_eq!(snap.playlist, "Arena Duel");
    assert!(snap.pressure.is_none());
    let expected_warm = warmup_host_line("Arena Duel", &[], 2);
    assert_eq!(snap.host_line, expected_warm);
    assert_eq!(snap.round_time_left, Some(2));
    let json = serde_json::to_value(&snap).unwrap();
    assert_eq!(json["mode_name"], "Contested Frequency");
    assert_eq!(json["playlist"], "Arena Duel");
    assert!(json.get("pressure").is_none() || json["pressure"].is_null());
    assert_eq!(json["host_line"], expected_warm);
}

#[test]
fn test_compliance_ping_fires_once_and_sets_pressure() {
    let mut state = GameState::new();
    state.config = MatchConfig {
        frag_limit: Some(99),
        time_limit_ticks: Some(20 * 60),
        warmup_ticks: 2,
        end_delay_ticks: 5,
        compliance_ping_ticks: Some(5),
        compliance_duration_ticks: 10,
        boss_spawn_ticks: None,
    };
    let id = Uuid::new_v4();
    state.add_player(id, "Scrap".to_string(), Role::Human);

    // Drain warmup into Active.
    state.tick(0.05);
    state.tick(0.05);
    let _ = state.take_events();
    assert_eq!(state.round_state, RoundState::Active);
    assert!(!state.compliance_fired);

    // Ticks 1..4: no ping yet.
    for _ in 0..4 {
        state.tick(0.05);
        let ev = state.take_events();
        assert!(
            !ev.iter()
                .any(|e| matches!(e, GameEvent::CompliancePing { .. })),
            "too early: {:?}",
            ev
        );
    }

    // Tick 5: compliance fires.
    state.tick(0.05);
    let ev = state.take_events();
    let ping = ev
        .iter()
        .find_map(|e| match e {
            GameEvent::CompliancePing {
                message,
                duration_ticks,
            } => Some((message.clone(), *duration_ticks)),
            _ => None,
        })
        .expect("compliance_ping event");
    assert!(ping.0.contains("CONTINUANCE"));
    assert_eq!(ping.1, 10);
    assert!(state.compliance_fired);
    // Fire sets ticks_left after the decrement check, so the fire tick keeps full duration.
    assert_eq!(state.compliance_ticks_left, 10);

    let snap = state.snapshot();
    assert_eq!(snap.pressure.as_deref(), Some("compliance"));
    assert_eq!(snap.host_line, compliance_host_line());

    // Second fire must not happen.
    for _ in 0..20 {
        state.tick(0.05);
        let ev = state.take_events();
        assert!(
            !ev.iter()
                .any(|e| matches!(e, GameEvent::CompliancePing { .. })),
            "duplicate ping: {:?}",
            ev
        );
    }
    assert!(state.compliance_ticks_left == 0);
    assert!(state.snapshot().pressure.is_none());
}

#[test]
fn test_compliance_pressure_slows_movement() {
    let mut state = GameState::new();
    state.config = MatchConfig {
        frag_limit: Some(99),
        time_limit_ticks: Some(20 * 60),
        warmup_ticks: 1,
        end_delay_ticks: 5,
        compliance_ping_ticks: None,
        compliance_duration_ticks: 20,
        boss_spawn_ticks: None,
    };
    let id = Uuid::new_v4();
    state.add_player(id, "Runner".to_string(), Role::Human);
    state.tick(0.05); // warmup -> active
    let _ = state.take_events();

    let start_x = state.players[0].x;
    state.set_action(
        id,
        Action {
            forward: true,
            ..Default::default()
        },
    );
    state.tick(0.05);
    let normal_dx = (state.players[0].x - start_x).abs();

    // Reset position-ish by measuring another normal step
    let x2 = state.players[0].x;
    state.tick(0.05);
    let normal_dx2 = (state.players[0].x - x2).abs();

    state.compliance_ticks_left = 20;
    let x3 = state.players[0].x;
    state.tick(0.05);
    let slow_dx = (state.players[0].x - x3).abs();

    assert!(
        slow_dx < normal_dx2 * 0.75,
        "compliance should slow move: normal={} slow={}",
        normal_dx2,
        slow_dx
    );
    assert!(normal_dx > 0.0);
}

#[test]
fn test_compliance_ping_wire_json_shape() {
    let event = GameEvent::CompliancePing {
        message: compliance_host_line(),
        duration_ticks: 120,
    };
    let msg = ServerMessage::Event(event);
    let json = serde_json::to_value(&msg).unwrap();
    assert_eq!(json["type"], "event");
    assert_eq!(json["event"], "compliance_ping");
    assert_eq!(json["duration_ticks"], 120);
    assert!(json["message"].as_str().unwrap().contains("CONTINUANCE"));
}

#[test]
fn test_welcome_includes_mode_identity() {
    let welcome = ServerMessage::Welcome {
        player_id: None,
        role: Role::Spectator,
        mode_name: default_mode_name(),
        playlist: default_playlist(),
    };
    let json = serde_json::to_value(&welcome).unwrap();
    assert_eq!(json["mode_name"], "Contested Frequency");
    assert_eq!(json["playlist"], "Arena Duel");
}

#[test]
fn test_snapshot_host_line_sticky_for_mid_join() {
    // Mid-join / observe must see Host chrome without waiting for RoundStart.
    let mut state = GameState::new();
    state.config = MatchConfig {
        frag_limit: Some(99),
        time_limit_ticks: Some(20 * 60),
        warmup_ticks: 2,
        end_delay_ticks: 5,
        compliance_ping_ticks: Some(5),
        compliance_duration_ticks: 8,
        boss_spawn_ticks: None,
    };
    state.add_player(Uuid::new_v4(), "Late".to_string(), Role::Human);

    // Warmup: Contested Frequency countdown Host drama sticky on Snapshot.
    let warm = state.snapshot();
    assert!(warm.pressure.is_none());
    let expected_warm = warmup_host_line("Arena Duel", &[], 1); // warmup_ticks=2 -> 1s ceil
    assert_eq!(warm.host_line, expected_warm);
    assert_eq!(warm.round_time_left, Some(1));
    let warm_json = serde_json::to_value(&warm).unwrap();
    assert_eq!(warm_json["host_line"], expected_warm);

    // Enter Active and fire compliance.
    state.tick(0.05);
    state.tick(0.05);
    let _ = state.take_events();
    for _ in 0..5 {
        state.tick(0.05);
    }
    let _ = state.take_events();
    assert!(state.compliance_ticks_left > 0);

    let during = state.snapshot();
    assert_eq!(during.pressure.as_deref(), Some("compliance"));
    assert_eq!(during.host_line, compliance_host_line());
    let during_json = serde_json::to_value(&during).unwrap();
    assert_eq!(during_json["host_line"], compliance_host_line());

    // After pressure ends, Snapshot returns to league Host line.
    while state.compliance_ticks_left > 0 {
        state.tick(0.05);
    }
    let after = state.snapshot();
    assert!(after.pressure.is_none());
    assert_eq!(after.host_line, default_host_line());
}

#[test]
fn test_snapshot_host_line_defaults_when_absent_on_wire() {
    // Old Snapshot without host_line still deserializes (mid-join clients / adapters).
    let raw = r#"{
        "tick": 1,
        "players": [],
        "mode_name": "Contested Frequency",
        "playlist": "Arena Duel"
    }"#;
    let snap: Snapshot = serde_json::from_str(raw).expect("legacy snapshot");
    assert_eq!(snap.host_line, default_host_line());
}

#[test]
fn test_compliance_drone_spawns_once_with_pressure_and_host() {
    let mut state = GameState::new();
    let a = Uuid::new_v4();
    state.add_player(a, "Rusher".into(), Role::Agent);
    state.config = MatchConfig {
        frag_limit: Some(99),
        time_limit_ticks: Some(20 * 60),
        warmup_ticks: 1,
        end_delay_ticks: 20,
        compliance_ping_ticks: None,
        compliance_duration_ticks: 10,
        boss_spawn_ticks: Some(3),
    };
    // Warmup -> Active
    state.tick(0.05);
    assert_eq!(state.round_state, RoundState::Active);
    assert!(!state.boss_spawned);

    state.tick(0.05); // round_ticks 1
    state.tick(0.05); // 2
    let _ = state.take_events();
    state.tick(0.05); // 3: spawn
    let events = state.take_events();
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::BossSpawn { name, hp, .. }
            if name == BOSS_NAME && *hp == BOSS_MAX_HP
        )),
        "expected BossSpawn, got {:?}",
        events
    );
    assert!(state.boss_id.is_some());
    assert!(state.boss_spawned);
    let snap = state.snapshot();
    assert_eq!(snap.pressure.as_deref(), Some("compliance_drone"));
    assert_eq!(snap.host_line, boss_host_line());
    assert!(snap.players.iter().any(|p| p.name == BOSS_NAME
        && p.behavior.as_deref() == Some("Compliance")
        && p.hp == BOSS_MAX_HP));

    // Second spawn attempt is a no-op.
    let before = state.players.len();
    assert!(state.spawn_compliance_drone().is_none());
    assert_eq!(state.players.len(), before);
}

#[test]
fn test_boss_wiped_on_round_end_emits_boss_down_no_killer() {
    let mut state = GameState::new();
    let a = Uuid::new_v4();
    state.add_player(a, "Rusher".into(), Role::Agent);
    state.config = MatchConfig {
        frag_limit: Some(99),
        time_limit_ticks: Some(5),
        warmup_ticks: 1,
        end_delay_ticks: 20,
        compliance_ping_ticks: None,
        compliance_duration_ticks: 10,
        boss_spawn_ticks: Some(1),
    };
    state.tick(0.05); // Warmup -> Active
    let _ = state.take_events();
    state.tick(0.05); // spawn boss
    let _ = state.take_events();
    assert!(
        state.boss_id.is_some(),
        "boss should be alive before round end"
    );
    let boss_id = state.boss_id.unwrap();

    // Drain Active time limit into Ended.
    for _ in 0..10 {
        state.tick(0.05);
        if state.round_state == RoundState::Ended {
            break;
        }
    }
    assert_eq!(state.round_state, RoundState::Ended);
    let events = state.take_events();
    let wipe = events.iter().find(|e| {
        matches!(
            e,
            GameEvent::BossDown {
                killer: None,
                message,
                ..
            } if message == &boss_round_wipe_host_line()
        )
    });
    assert!(
        wipe.is_some(),
        "expected BossDown wipe (killer null) on round end, got {:?}",
        events
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::RoundEnd { .. })),
        "expected RoundEnd after wipe, got {:?}",
        events
    );
    assert!(state.boss_id.is_none());
    let snap = state.snapshot();
    assert!(
        snap.pressure.is_none(),
        "Ended must not keep compliance_drone pressure"
    );
    assert!(
        !snap.players.iter().any(|p| p.name == BOSS_NAME),
        "drone must be absent from Ended snapshot"
    );
    let _ = boss_id;
}

#[test]
fn test_compliance_drone_killable_emits_boss_down_no_respawn() {
    let mut state = GameState::new();
    let shooter = Uuid::new_v4();
    state.add_player(shooter, "Rusher".into(), Role::Agent);
    state.config = MatchConfig {
        frag_limit: Some(99),
        time_limit_ticks: Some(20 * 60),
        warmup_ticks: 1,
        end_delay_ticks: 20,
        compliance_ping_ticks: None,
        compliance_duration_ticks: 10,
        boss_spawn_ticks: Some(1),
    };
    state.tick(0.05); // warmup -> Active
    let _ = state.take_events();
    state.tick(0.05); // spawn at tick 1
    let _ = state.take_events();
    let boss_id = state.boss_id.expect("boss alive");

    // Point Rusher at boss and dump damage with Rail-level hits.
    {
        let boss = state.players.iter().find(|p| p.id == boss_id).unwrap();
        let bx = boss.x;
        let bz = boss.z;
        let rusher = state.players.iter_mut().find(|p| p.id == shooter).unwrap();
        rusher.x = bx - 5.0;
        rusher.z = bz;
        rusher.yaw = 0.0; // facing +x toward boss
        rusher.weapon = WeaponType::Rail;
        rusher.pending_action = Action {
            fire: true,
            ..Default::default()
        };
    }
    // Keep firing until boss is down (Rail 75 dmg; 200 HP => 3 hits).
    let mut saw_down = false;
    for _ in 0..20 {
        if let Some(r) = state.players.iter_mut().find(|p| p.id == shooter) {
            r.fire_cooldown = 0;
            r.pending_action.fire = true;
        }
        state.tick(0.05);
        let events = state.take_events();
        if events
            .iter()
            .any(|e| matches!(e, GameEvent::BossDown { .. }))
        {
            saw_down = true;
            assert!(
                events.iter().any(|e| matches!(
                    e,
                    GameEvent::BossDown {
                        name,
                        killer: Some(k),
                        message,
                        ..
                    } if name == BOSS_NAME && k == "Rusher" && message == &boss_down_host_line()
                )),
                "boss_down shape bad: {:?}",
                events
            );
            break;
        }
    }
    assert!(saw_down, "expected BossDown after Rail volleys");
    assert!(state.boss_id.is_none());
    assert!(!state.players.iter().any(|p| p.name == BOSS_NAME));
    assert_eq!(state.snapshot().pressure, None);
    // Advance past normal respawn window: drone must not return.
    for _ in 0..80 {
        state.tick(0.05);
        let _ = state.take_events();
    }
    assert!(!state
        .players
        .iter()
        .any(|p| p.name == BOSS_NAME || p.is_boss));
}

#[test]
fn test_boss_down_wire_json_and_compliance_ai_acts() {
    let mut state = GameState::new();
    let target = Uuid::new_v4();
    state.add_player(target, "Sniper".into(), Role::Agent);
    state.start_round();
    state.spawn_compliance_drone().expect("spawn");
    let boss_id = state.boss_id.unwrap();
    let bot = state
        .bots
        .iter()
        .find(|b| b.player_id == boss_id)
        .cloned()
        .expect("compliance controller");
    let action = bot.update(&state);
    // Should turn and/or fire toward the only scrap fighter.
    assert!(
        action.fire
            || action.turn_left
            || action.turn_right
            || action.forward
            || action.back
            || action.left
            || action.right,
        "Compliance AI should produce intent, got {:?}",
        action
    );

    let down = GameEvent::BossDown {
        name: BOSS_NAME.into(),
        boss_id,
        killer: None,
        message: boss_down_host_line(),
    };
    let v = serde_json::to_value(&down).unwrap();
    assert_eq!(v["event"], "boss_down");
    assert!(v.get("killer").is_none());
}

#[test]
fn test_sim_pickups_present_in_snapshot() {
    let state = GameState::new();
    let snap = state.snapshot();
    assert_eq!(snap.pickups.len(), 6);
    let ids: Vec<_> = snap.pickups.iter().map(|p| p.id.as_str()).collect();
    assert!(ids.contains(&"pad_rail"));
    assert!(ids.contains(&"pad_scatter"));
    assert!(ids.contains(&"pad_flechette"));
    assert!(ids.contains(&"pad_health_n"));
    assert!(ids.contains(&"pad_health_s"));
    assert!(ids.contains(&"pad_armor"));
    assert!(snap.pickups.iter().all(|p| p.available));
    let health = snap
        .pickups
        .iter()
        .find(|p| p.id == "pad_health_n")
        .unwrap();
    assert_eq!(health.kind, "health");
    assert_eq!(health.amount, Some(40));
    let armor = snap.pickups.iter().find(|p| p.id == "pad_armor").unwrap();
    assert_eq!(armor.kind, "armor");
    assert_eq!(armor.amount, Some(25));
}

#[test]
fn test_sim_pickup_claim_changes_weapon_and_emits_event() {
    let mut state = GameState::new();
    state.start_round();
    let id = Uuid::new_v4();
    state.add_player(id, "Rusher".into(), Role::Human);
    // Place player on rail pad.
    if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
        p.x = 12.0;
        p.z = 12.0;
        p.weapon = WeaponType::Flechette;
    }
    state.tick(0.05);
    let player = state.players.iter().find(|p| p.id == id).unwrap();
    assert_eq!(player.weapon, WeaponType::Rail);
    let events = state.take_events();
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::Pickup {
                kind,
                weapon,
                pickup_id,
                ..
            } if kind == "weapon" && weapon == "Rail" && pickup_id == "pad_rail"
        )),
        "expected pickup event, got {:?}",
        events
    );
    let rail = state.pickups.iter().find(|p| p.id == "pad_rail").unwrap();
    assert!(!rail.available);
    assert_eq!(rail.respawn_timer, Some(PICKUP_RESPAWN_TICKS));
}

#[test]
fn test_sim_pickup_respawns_after_timer() {
    let mut state = GameState::new();
    state.start_round();
    let id = Uuid::new_v4();
    state.add_player(id, "Rusher".into(), Role::Human);
    if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
        p.x = 12.0;
        p.z = 12.0;
    }
    state.tick(0.05);
    assert!(
        !state
            .pickups
            .iter()
            .find(|p| p.id == "pad_rail")
            .unwrap()
            .available
    );
    // Move off pad so we do not re-claim instantly.
    if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
        p.x = 0.0;
        p.z = 0.0;
    }
    for _ in 0..PICKUP_RESPAWN_TICKS {
        state.tick(0.05);
    }
    let rail = state.pickups.iter().find(|p| p.id == "pad_rail").unwrap();
    assert!(rail.available, "rail pad should respawn");
    assert!(rail.respawn_timer.is_none());
}

#[test]
fn test_sim_pickup_reset_on_round_start() {
    let mut state = GameState::new();
    state.start_round();
    let id = Uuid::new_v4();
    state.add_player(id, "Rusher".into(), Role::Human);
    if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
        p.x = 12.0;
        p.z = 12.0;
    }
    state.tick(0.05);
    assert!(
        !state
            .pickups
            .iter()
            .find(|p| p.id == "pad_rail")
            .unwrap()
            .available
    );
    state.end_round("test".into());
    state.start_round();
    assert!(state.pickups.iter().all(|p| p.available));
}

#[test]
fn test_sim_health_pad_heals_and_emits_kind() {
    let mut state = GameState::new();
    state.start_round();
    let id = Uuid::new_v4();
    state.add_player(id, "Rusher".into(), Role::Human);
    if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
        p.x = 0.0;
        p.z = 8.0;
        p.hp = 40;
    }
    state.tick(0.05);
    let player = state.players.iter().find(|p| p.id == id).unwrap();
    assert_eq!(player.hp, 80);
    let events = state.take_events();
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::Pickup {
                kind,
                amount: Some(40),
                pickup_id,
                ..
            } if kind == "health" && pickup_id == "pad_health_n"
        )),
        "expected health pickup event, got {:?}",
        events
    );
    let pad = state
        .pickups
        .iter()
        .find(|p| p.id == "pad_health_n")
        .unwrap();
    assert!(!pad.available);
    assert_eq!(pad.respawn_timer, Some(HEALTH_PICKUP_RESPAWN_TICKS));
}

#[test]
fn test_sim_health_pad_skipped_at_full_hp() {
    let mut state = GameState::new();
    state.start_round();
    let id = Uuid::new_v4();
    state.add_player(id, "Rusher".into(), Role::Human);
    if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
        p.x = 0.0;
        p.z = 8.0;
        p.hp = 100;
    }
    state.tick(0.05);
    let pad = state
        .pickups
        .iter()
        .find(|p| p.id == "pad_health_n")
        .unwrap();
    assert!(pad.available, "full HP should not claim health pad");
}

#[test]
fn test_sim_armor_pad_grants_armor_and_absorbs_damage() {
    let mut state = GameState::new();
    state.start_round();
    let id = Uuid::new_v4();
    state.add_player(id, "Rusher".into(), Role::Human);
    if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
        p.x = 8.0;
        p.z = 0.0;
    }
    state.tick(0.05);
    let player = state.players.iter().find(|p| p.id == id).unwrap();
    assert_eq!(player.armor, 25);
    let events = state.take_events();
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::Pickup {
                kind,
                amount: Some(25),
                pickup_id,
                ..
            } if kind == "armor" && pickup_id == "pad_armor"
        )),
        "expected armor pickup event, got {:?}",
        events
    );

    let shooter = Uuid::new_v4();
    state.add_player(shooter, "Shooter".into(), Role::Agent);
    if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
        p.x = 0.0;
        p.z = 0.0;
        p.yaw = 0.0;
        p.armor = 10;
        p.hp = 100;
    }
    if let Some(p) = state.players.iter_mut().find(|p| p.id == shooter) {
        p.x = -5.0;
        p.z = 0.0;
        p.yaw = 0.0;
        p.weapon = WeaponType::Flechette;
        p.pending_action.fire = true;
    }
    state.tick(0.05);
    let victim = state.players.iter().find(|p| p.id == id).unwrap();
    assert_eq!(victim.armor, 0);
    assert_eq!(victim.hp, 85);
}

#[test]
fn test_sim_health_pad_respawns_after_timer() {
    let mut state = GameState::new();
    state.start_round();
    let id = Uuid::new_v4();
    state.add_player(id, "Rusher".into(), Role::Human);
    if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
        p.x = 0.0;
        p.z = 8.0;
        p.hp = 50;
    }
    state.tick(0.05);
    assert!(
        !state
            .pickups
            .iter()
            .find(|p| p.id == "pad_health_n")
            .unwrap()
            .available
    );
    if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
        p.x = 20.0;
        p.z = 20.0;
    }
    for _ in 0..HEALTH_PICKUP_RESPAWN_TICKS {
        state.tick(0.05);
    }
    let pad = state
        .pickups
        .iter()
        .find(|p| p.id == "pad_health_n")
        .unwrap();
    assert!(pad.available, "health pad should respawn");
}

#[test]
fn test_sim_weapon_pads_still_claim_with_health_pads_present() {
    let mut state = GameState::new();
    state.start_round();
    let id = Uuid::new_v4();
    state.add_player(id, "Rusher".into(), Role::Human);
    if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
        p.x = -12.0;
        p.z = -12.0;
        p.weapon = WeaponType::Flechette;
    }
    state.tick(0.05);
    let player = state.players.iter().find(|p| p.id == id).unwrap();
    assert_eq!(player.weapon, WeaponType::Scatter);
}

#[test]
fn test_sim_choke_blocks_move_into_pillar() {
    let mut state = GameState::new();
    state.start_round();
    let id = Uuid::new_v4();
    state.add_player(id, "Rusher".into(), Role::Human);
    if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
        // South of pillar NE at (7, -7); walk north into it.
        p.x = 7.0;
        p.z = -9.5;
        p.yaw = std::f32::consts::FRAC_PI_2; // face +z
    }
    for _ in 0..40 {
        state.set_action(
            id,
            Action {
                forward: true,
                ..Default::default()
            },
        );
        state.tick(0.05);
    }
    let player = state.players.iter().find(|p| p.id == id).unwrap();
    // Expanded pillar face is at z = -8.25 - 0.5 = -8.75; must not penetrate.
    assert!(
        player.z <= -8.7,
        "player should be stopped by pillar, z={}",
        player.z
    );
    assert!(
        (player.x - 7.0).abs() < 0.2,
        "x should stay near 7, got {}",
        player.x
    );
}

#[test]
fn test_sim_choke_hitscan_blocked_by_low_wall() {
    let mut state = GameState::new();
    state.start_round();
    let shooter = Uuid::new_v4();
    let target = Uuid::new_v4();
    state.add_player(shooter, "Shooter".into(), Role::Agent);
    state.add_player(target, "Victim".into(), Role::Agent);
    if let Some(p) = state.players.iter_mut().find(|p| p.id == shooter) {
        // South of low wall north (z=-10); shoot toward +z through wall.
        p.x = 0.0;
        p.z = -15.0;
        p.yaw = std::f32::consts::FRAC_PI_2;
        p.weapon = WeaponType::Rail;
    }
    if let Some(p) = state.players.iter_mut().find(|p| p.id == target) {
        p.x = 0.0;
        p.z = -5.0;
        p.hp = 100;
    }
    let hp_before = state.players.iter().find(|p| p.id == target).unwrap().hp;
    state.set_action(
        shooter,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);
    let hp_after = state.players.iter().find(|p| p.id == target).unwrap().hp;
    assert_eq!(hp_before, hp_after, "low wall must block hitscan");
}

#[test]
fn test_sim_choke_hitscan_clear_lane_still_hits() {
    let mut state = GameState::new();
    state.start_round();
    let shooter = Uuid::new_v4();
    let target = Uuid::new_v4();
    state.add_player(shooter, "Shooter".into(), Role::Agent);
    state.add_player(target, "Victim".into(), Role::Agent);
    if let Some(p) = state.players.iter_mut().find(|p| p.id == shooter) {
        // Open lane along +x through hub (no wall on x axis at z=0 between -5 and 5).
        p.x = -5.0;
        p.z = 0.0;
        p.yaw = 0.0;
        p.weapon = WeaponType::Flechette;
    }
    if let Some(p) = state.players.iter_mut().find(|p| p.id == target) {
        p.x = 5.0;
        p.z = 0.0;
        p.hp = 100;
    }
    let hp_before = state.players.iter().find(|p| p.id == target).unwrap().hp;
    state.set_action(
        shooter,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);
    let hp_after = state.players.iter().find(|p| p.id == target).unwrap().hp;
    assert!(hp_after < hp_before, "clear hub lane should still hit");
}

#[test]
fn test_sim_choke_health_pad_still_claimable_from_hub() {
    let mut state = GameState::new();
    state.start_round();
    let id = Uuid::new_v4();
    state.add_player(id, "Rusher".into(), Role::Human);
    if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
        p.x = 0.0;
        p.z = 8.0;
        p.hp = 40;
    }
    state.tick(0.05);
    let player = state.players.iter().find(|p| p.id == id).unwrap();
    assert_eq!(player.hp, 80, "health pad north must remain claimable");
}

#[test]
fn test_sim_choke_spawn_points_clear_of_solids() {
    let mut state = GameState::new();
    for i in 0..8 {
        let id = Uuid::new_v4();
        state.add_player(id, format!("P{i}"), Role::Agent);
    }
    for p in &state.players {
        // Same predicate as move: inflated AABB must not contain spawn.
        // Probe by trying a zero move resolve: if blocked at spawn, circle_blocked is true.
        // We assert players are not inside the raw expanded solids by checking distance
        // to known pillar centers exceeds half+radius.
        for (cx, cz) in [(7.0_f32, -7.0), (-7.0, -7.0), (7.0, 7.0), (-7.0, 7.0)] {
            let dx = (p.x - cx).abs();
            let dz = (p.z - cz).abs();
            let inside = dx <= 1.25 + 0.5 && dz <= 1.25 + 0.5;
            assert!(
                !inside,
                "spawn ({}, {}) inside pillar at ({}, {})",
                p.x, p.z, cx, cz
            );
        }
    }
}

#[test]
fn test_sim_choke_compliance_drone_center_clear() {
    let mut state = GameState::new();
    state.config.boss_spawn_ticks = Some(1);
    state.start_round();
    // Advance into Active and to boss spawn tick.
    while state.round_state != RoundState::Active {
        state.tick(0.05);
    }
    for _ in 0..5 {
        state.tick(0.05);
        if state.boss_id.is_some() {
            break;
        }
    }
    assert!(state.boss_id.is_some(), "drone should spawn");
    let boss = state
        .players
        .iter()
        .find(|p| p.is_boss)
        .expect("boss player");
    assert!(boss.x.abs() < 0.1 && boss.z.abs() < 0.1, "drone at hub");
}

fn force_hitscan_frag(state: &mut GameState, shooter_id: Uuid, target_id: Uuid) {
    let shooter_idx = state
        .players
        .iter()
        .position(|p| p.id == shooter_id)
        .expect("shooter");
    let target_idx = state
        .players
        .iter()
        .position(|p| p.id == target_id)
        .expect("target");
    state.players[target_idx].x = 5.0;
    state.players[target_idx].z = 0.0;
    state.players[target_idx].y = 1.5;
    state.players[target_idx].hp = 20;
    state.players[target_idx].armor = 0;
    state.players[target_idx].respawn_timer = None;
    state.players[shooter_idx].x = 0.0;
    state.players[shooter_idx].z = 0.0;
    state.players[shooter_idx].y = 1.5;
    state.players[shooter_idx].yaw = 0.0;
    state.players[shooter_idx].fire_cooldown = 0;
    state.set_action(
        shooter_id,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);
}

#[test]
fn test_killstreak_emits_at_2_3_5_and_resets_on_death() {
    let mut state = GameState::new();
    state.config.compliance_ping_ticks = None;
    state.config.boss_spawn_ticks = None;
    state.config.frag_limit = Some(99);
    state.start_round();

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();
    state.add_player(shooter_id, "Rusher".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);
    let _ = state.take_events();

    let mut seen_tiers: Vec<String> = Vec::new();
    for n in 1..=5 {
        // Wait out respawn if target died last frag.
        for _ in 0..100 {
            let tidx = state
                .players
                .iter()
                .position(|p| p.id == target_id)
                .unwrap();
            if state.players[tidx].respawn_timer.is_none()
                && state.players[tidx].hp > 0
                && !state.spawn_shields.contains_key(&target_id)
            {
                break;
            }
            state.tick(0.05);
        }
        let _ = state.take_events();
        force_hitscan_frag(&mut state, shooter_id, target_id);
        let events = state.take_events();
        assert!(
            events.iter().any(|e| matches!(
                e,
                GameEvent::Frag {
                    killer,
                    ..
                } if killer == "Rusher"
            )),
            "expected frag on kill {}",
            n
        );
        let sidx = state
            .players
            .iter()
            .position(|p| p.id == shooter_id)
            .unwrap();
        assert_eq!(state.players[sidx].killstreak, n, "streak after kill {}", n);
        for e in &events {
            if let GameEvent::Killstreak {
                streak,
                tier,
                message,
                player,
                ..
            } = e
            {
                assert_eq!(player, "Rusher");
                assert_eq!(*streak, n);
                assert!(message.starts_with("HOST:"));
                seen_tiers.push(tier.clone());
            }
        }
        if n == 1 || n == 4 {
            assert!(
                !events
                    .iter()
                    .any(|e| matches!(e, GameEvent::Killstreak { .. })),
                "no killstreak event at streak {}",
                n
            );
        }
    }
    assert_eq!(seen_tiers, vec!["double", "triple", "rampage"]);

    // Victim death resets victim streak; kill Rusher with Target to reset Rusher.
    let sidx = state
        .players
        .iter()
        .position(|p| p.id == shooter_id)
        .unwrap();
    assert_eq!(state.players[sidx].killstreak, 5);
    for _ in 0..70 {
        let tidx = state
            .players
            .iter()
            .position(|p| p.id == target_id)
            .unwrap();
        if state.players[tidx].respawn_timer.is_none() && state.players[tidx].hp > 0 {
            break;
        }
        state.tick(0.05);
    }
    let _ = state.take_events();
    force_hitscan_frag(&mut state, target_id, shooter_id);
    let sidx = state
        .players
        .iter()
        .position(|p| p.id == shooter_id)
        .unwrap();
    assert_eq!(state.players[sidx].killstreak, 0, "death resets streak");
}

#[test]
fn test_killstreak_resets_on_round_start() {
    let mut state = GameState::new();
    state.config.compliance_ping_ticks = None;
    state.config.boss_spawn_ticks = None;
    state.start_round();
    let a = Uuid::new_v4();
    state.add_player(a, "A".to_string(), Role::Agent);
    let idx = state.players.iter().position(|p| p.id == a).unwrap();
    state.players[idx].killstreak = 4;
    state.start_round();
    let idx = state.players.iter().position(|p| p.id == a).unwrap();
    assert_eq!(state.players[idx].killstreak, 0);
}

#[test]
fn test_weapon_range_units_distinct() {
    use crate::protocol::WeaponType;
    assert!(WeaponType::Scatter.range_units() < WeaponType::Flechette.range_units());
    assert!(WeaponType::Flechette.range_units() < WeaponType::Rail.range_units());
    assert_eq!(WeaponType::Flechette.preferred_range(), (8.0, 28.0));
    assert_eq!(WeaponType::Rail.preferred_range(), (18.0, 45.0));
    assert_eq!(WeaponType::Scatter.preferred_range(), (2.0, 10.0));
}

#[test]
fn test_scatter_misses_beyond_range() {
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

    // Clear north lane (z=20): beyond Scatter 14u, inside Rail. Avoids mid choke walls.
    state.players[shooter_idx].x = 0.0;
    state.players[shooter_idx].z = 20.0;
    state.players[shooter_idx].yaw = 0.0;
    state.players[target_idx].x = 16.0;
    state.players[target_idx].z = 20.0;

    state.players[shooter_idx].weapon = WeaponType::Scatter;
    state.players[shooter_idx].fire_cooldown = 0;
    state.set_action(
        shooter_id,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    let hp_before = state.players[target_idx].hp;
    state.tick(0.05);
    assert_eq!(
        state.players[target_idx].hp, hp_before,
        "Scatter must miss beyond range_units"
    );

    state.players[shooter_idx].weapon = WeaponType::Rail;
    state.players[shooter_idx].fire_cooldown = 0;
    state.set_action(
        shooter_id,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);
    assert!(
        state.players[target_idx].hp < hp_before,
        "Rail must still hit at 20u"
    );
}

#[test]
fn test_bot_rail_holds_long_lane() {
    use crate::protocol::WeaponType;

    let mut state = GameState::new();
    state.start_round();

    let bot_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();
    state.add_player(bot_id, "Bot".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    let bot_idx = state.players.iter().position(|p| p.id == bot_id).unwrap();
    let target_idx = state
        .players
        .iter()
        .position(|p| p.id == target_id)
        .unwrap();

    state.players[bot_idx].weapon = WeaponType::Rail;
    state.players[bot_idx].x = 0.0;
    state.players[bot_idx].z = 0.0;
    state.players[bot_idx].yaw = 0.0;
    // Target inside Rail preferred band (~25u).
    state.players[target_idx].x = 25.0;
    state.players[target_idx].z = 0.0;

    let bot = BotController::new(bot_id, BotBehavior::Defensive);
    let action = bot.update(&state);
    // At preferred band, Defensive should strafe (not rush) and be willing to fire.
    assert!(
        action.fire || action.left || action.right || action.back || action.forward,
        "Rail bot at preferred range should act"
    );
    // Not closing hard when already in band.
    assert!(
        !(action.forward && !action.left && !action.right && !action.back),
        "Rail defensive bot should not only rush when already mid-long"
    );
}

#[test]
fn test_bot_scatter_pushes_close() {
    use crate::protocol::WeaponType;

    let mut state = GameState::new();
    state.start_round();

    let bot_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();
    state.add_player(bot_id, "Bot".to_string(), Role::Agent);
    state.add_player(target_id, "Target".to_string(), Role::Agent);

    let bot_idx = state.players.iter().position(|p| p.id == bot_id).unwrap();
    let target_idx = state
        .players
        .iter()
        .position(|p| p.id == target_id)
        .unwrap();

    state.players[bot_idx].weapon = WeaponType::Scatter;
    state.players[bot_idx].x = 0.0;
    state.players[bot_idx].z = 0.0;
    state.players[bot_idx].yaw = 0.0;
    // Target outside Scatter preferred band but inside fire range stretch.
    state.players[target_idx].x = 16.0;
    state.players[target_idx].z = 0.0;

    let bot = BotController::new(bot_id, BotBehavior::Aggressive);
    let action = bot.update(&state);
    assert!(
        action.forward,
        "Scatter aggressive bot should push toward close range"
    );
}

#[test]
fn test_round_end_emits_mvp_host_line_and_sticky_snapshot() {
    let mut state = GameState::new();
    state.config.frag_limit = Some(2);
    state.config.time_limit_ticks = None;
    state.config.compliance_ping_ticks = None;
    state.config.boss_spawn_ticks = None;
    state.config.warmup_ticks = 1;
    state.config.end_delay_ticks = 20 * 10;

    let a = uuid::Uuid::new_v4();
    let b = uuid::Uuid::new_v4();
    state.add_player(a, "Rusher".to_string(), Role::Agent);
    state.add_player(b, "Anchor".to_string(), Role::Agent);
    // Leave warmup
    state.tick(0.05);
    assert_eq!(state.round_state, RoundState::Active);

    *state.scores.get_mut(&a).unwrap() = 2;
    *state.scores.get_mut(&b).unwrap() = 1;
    state.end_round("Frag limit reached".to_string());

    let events = state.take_events();
    let end = events
        .iter()
        .find_map(|e| match e {
            GameEvent::RoundEnd {
                mvp,
                mvp_frags,
                host_line,
                winner,
                ..
            } => Some((mvp.clone(), *mvp_frags, host_line.clone(), winner.clone())),
            _ => None,
        })
        .expect("RoundEnd event");
    assert_eq!(end.0.as_deref(), Some("Rusher"));
    assert_eq!(end.1, Some(2));
    assert!(end.2.contains("ROUND MVP"));
    assert!(end.2.contains("Rusher"));
    assert_eq!(end.3.as_deref(), Some("Rusher"));

    assert_eq!(state.round_state, RoundState::Ended);
    let snap = state.snapshot();
    assert!(snap.host_line.contains("ROUND MVP"));
    assert!(snap.host_line.contains("Rusher"));
    assert_eq!(snap.mvp.as_deref(), Some("Rusher"));
    assert_eq!(snap.mvp_frags, Some(2));
    assert_eq!(
        state.ended_host_line.as_deref(),
        Some(snap.host_line.as_str())
    );
    assert_eq!(state.ended_mvp.as_deref(), Some("Rusher"));
    assert_eq!(state.ended_mvp_frags, Some(2));

    // Next round clears sticky MVP Host line and structured mvp fields.
    state.start_round();
    assert!(state.ended_host_line.is_none());
    assert!(state.ended_mvp.is_none());
    assert!(state.ended_mvp_frags.is_none());
    let warm = state.snapshot();
    assert_eq!(warm.host_line, default_host_line());
    assert!(warm.mvp.is_none());
    assert!(warm.mvp_frags.is_none());
}

#[test]
fn test_default_ended_linger_is_eight_seconds() {
    let cfg = MatchConfig::default();
    assert_eq!(
        cfg.end_delay_ticks,
        20 * 8,
        "Ended linger should be 8s at 20 Hz so podium/Host bumper can be read"
    );
}

#[test]
fn test_snapshot_omits_mvp_while_active() {
    let mut state = GameState::new();
    state.config.compliance_ping_ticks = None;
    state.config.boss_spawn_ticks = None;
    state.start_round();
    let snap = state.snapshot();
    assert_eq!(snap.round_state.as_deref(), Some("Active"));
    assert!(snap.mvp.is_none());
    assert!(snap.mvp_frags.is_none());
    let json = serde_json::to_value(&snap).unwrap();
    assert!(json.get("mvp").is_none());
    assert!(json.get("mvp_frags").is_none());
}

#[test]
fn test_snapshot_mvp_wire_round_trip_while_ended() {
    let mut state = GameState::new();
    state.config.compliance_ping_ticks = None;
    state.config.boss_spawn_ticks = None;
    let id = uuid::Uuid::new_v4();
    state.add_player(id, "Ghost".to_string(), Role::Agent);
    state.start_round();
    *state.scores.get_mut(&id).unwrap() = 3;
    state.end_round("Frag limit reached".to_string());
    let snap = state.snapshot();
    let json = serde_json::to_value(&snap).unwrap();
    assert_eq!(json["round_state"], "Ended");
    assert_eq!(json["mvp"], "Ghost");
    assert_eq!(json["mvp_frags"], 3);
    assert!(json["host_line"].as_str().unwrap().contains("Ghost"));
    let back: Snapshot = serde_json::from_value(json).unwrap();
    assert_eq!(back.mvp.as_deref(), Some("Ghost"));
    assert_eq!(back.mvp_frags, Some(3));
}

#[test]
fn test_round_end_empty_mvp_host_line() {
    let mut state = GameState::new();
    state.config.compliance_ping_ticks = None;
    state.config.boss_spawn_ticks = None;
    state.end_round("Time limit reached".to_string());
    let events = state.take_events();
    let host = events
        .iter()
        .find_map(|e| match e {
            GameEvent::RoundEnd { mvp, host_line, .. } => Some((mvp.clone(), host_line.clone())),
            _ => None,
        })
        .expect("RoundEnd");
    assert!(host.0.is_none());
    assert!(host.1.contains("NO MVP"));
    let snap = state.snapshot();
    assert!(snap.host_line.contains("NO MVP"));
    assert!(snap.mvp.is_none());
    assert!(snap.mvp_frags.is_none());
}

#[test]
fn test_map_kind_cli_and_names() {
    assert_eq!(MapKind::from_cli("1"), Some(MapKind::ArenaDuel));
    assert_eq!(MapKind::from_cli("arena"), Some(MapKind::ArenaDuel));
    assert_eq!(MapKind::from_cli("2"), Some(MapKind::ComplianceYard));
    assert_eq!(
        MapKind::from_cli("compliance-yard"),
        Some(MapKind::ComplianceYard)
    );
    assert_eq!(MapKind::from_cli("nope"), None);
    assert_eq!(MapKind::ArenaDuel.id(), 1);
    assert_eq!(MapKind::ComplianceYard.id(), 2);
    assert_eq!(MapKind::ArenaDuel.name(), "Arena Duel");
    assert_eq!(MapKind::ComplianceYard.name(), "Compliance Yard");
    assert_eq!(MapKind::ArenaDuel.next(), MapKind::ComplianceYard);
    assert_eq!(MapKind::ComplianceYard.next(), MapKind::ArenaDuel);
}

#[test]
fn test_sim_compliance_yard_blocks_move_into_post() {
    let mut state = GameState::with_map(MapKind::ComplianceYard, false);
    state.start_round();
    let id = Uuid::new_v4();
    state.add_player(id, "Rusher".into(), Role::Human);
    if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
        // South of NE post at (5, -5); walk north into it.
        p.x = 5.0;
        p.z = -7.5;
        p.yaw = std::f32::consts::FRAC_PI_2;
    }
    for _ in 0..40 {
        state.set_action(
            id,
            Action {
                forward: true,
                ..Default::default()
            },
        );
        state.tick(0.05);
    }
    let player = state.players.iter().find(|p| p.id == id).unwrap();
    assert!(
        player.z <= -6.4,
        "player should be stopped by yard post, z={}",
        player.z
    );
}

#[test]
fn test_sim_compliance_yard_pad_claim_and_hub_clear() {
    let mut state = GameState::with_map(MapKind::ComplianceYard, false);
    state.start_round();
    let snap = state.snapshot();
    assert_eq!(snap.map_id, 2);
    assert_eq!(snap.map_name, "Compliance Yard");
    assert_eq!(state.pickups.len(), 6);
    let armor = state.pickups.iter().find(|p| p.id == "pad_armor").unwrap();
    assert!((armor.x - 11.0).abs() < 0.01);
    assert!((armor.z - 0.0).abs() < 0.01);

    // Hub must stay clear for drone (circle at 0,0 not blocked).
    for obs in MapKind::ComplianceYard.obstacles() {
        assert!(
            !obs.expand(0.5).contains(0.0, 0.0),
            "hub blocked by {:?}",
            obs
        );
    }

    let id = Uuid::new_v4();
    state.add_player(id, "Scrapper".into(), Role::Human);
    if let Some(p) = state.players.iter_mut().find(|p| p.id == id) {
        p.x = 11.0;
        p.z = 0.0;
        p.armor = 0;
        p.hp = 100;
    }
    state.tick(0.05);
    let player = state.players.iter().find(|p| p.id == id).unwrap();
    assert!(player.armor > 0, "armor pad should claim on yard");
}

#[test]
fn test_sim_map_rotate_each_start_round() {
    let mut state = GameState::with_map(MapKind::ArenaDuel, true);
    assert_eq!(state.map, MapKind::ArenaDuel);
    state.start_round(); // round 1, stay Arena Duel
    assert_eq!(state.round_number, 1);
    assert_eq!(state.map, MapKind::ArenaDuel);
    state.end_round("test".into());
    state.start_round(); // round 2, rotate to yard
    assert_eq!(state.round_number, 2);
    assert_eq!(state.map, MapKind::ComplianceYard);
    assert_eq!(state.snapshot().map_id, 2);
    state.end_round("test".into());
    state.start_round(); // round 3, back to arena
    assert_eq!(state.map, MapKind::ArenaDuel);
}

#[test]
fn test_protocol_snapshot_map_defaults_round_trip() {
    let json = serde_json::json!({
        "tick": 1,
        "players": [],
        "mode_name": "Contested Frequency",
        "playlist": "Arena Duel",
        "host_line": "HOST: CONTESTED FREQUENCY. PLAY VS COMPLIANCE. ARENA DUEL IS LIVE."
    });
    let snap: Snapshot = serde_json::from_value(json).unwrap();
    assert_eq!(snap.map_id, default_map_id());
    assert_eq!(snap.map_name, default_map_name());
}

#[test]
fn test_warmup_host_drama_roster_map_countdown() {
    let mut state = GameState::new();
    state.config.warmup_ticks = 40; // 2s
    state.set_roster_host_line_from_names(&["Dead Air Dan".into(), "Nightfall".into()]);
    assert_eq!(state.round_state, RoundState::Warmup);

    let snap = state.snapshot();
    assert_eq!(snap.round_time_left, Some(2));
    assert!(snap.host_line.contains("CONTESTED FREQUENCY"));
    assert!(snap.host_line.contains("ARENA DUEL"));
    assert!(snap.host_line.contains("DEAD AIR DAN"));
    assert!(snap.host_line.contains("ON THE SCRAP"));
    assert!(snap.host_line.contains("2."));
    assert_eq!(
        snap.host_line,
        warmup_host_line("Arena Duel", &state.roster_names, 2)
    );

    // Advance one second of Warmup; countdown should drop.
    for _ in 0..20 {
        state.tick(0.05);
    }
    assert_eq!(state.round_state, RoundState::Warmup);
    let mid = state.snapshot();
    assert_eq!(mid.round_time_left, Some(1));
    assert!(mid.host_line.ends_with("1."));

    // Finish Warmup -> RoundStart fight bumper with map + roster.
    for _ in 0..20 {
        state.tick(0.05);
    }
    assert_eq!(state.round_state, RoundState::Active);
    let start = state
        .take_events()
        .into_iter()
        .find_map(|e| match e {
            GameEvent::RoundStart { host_line, .. } => Some(host_line),
            _ => None,
        })
        .expect("RoundStart");
    assert_eq!(
        start,
        round_open_host_line("Arena Duel", &["Dead Air Dan".into(), "Nightfall".into()])
    );
    assert!(start.contains("FIGHT!"));
}

#[test]
fn rule_bot_taunt_emits_speak_and_respects_cooldown() {
    use crate::protocol::BotTauntKind;
    use crate::sim::{SpeakOutcome, SPEAK_COOLDOWN_TICKS};

    let mut state = GameState::new();
    let bot_id = Uuid::new_v4();
    state.add_player(bot_id, "Dead Air Dan".to_string(), Role::Agent);
    state
        .bots
        .push(BotController::new(bot_id, BotBehavior::Aggressive));

    assert_eq!(
        state.try_rule_bot_taunt(bot_id, BotTauntKind::Frag),
        SpeakOutcome::Sent
    );
    let speaks: Vec<_> = state
        .take_events()
        .into_iter()
        .filter_map(|e| match e {
            GameEvent::Speak { player, text, .. } => Some((player, text)),
            _ => None,
        })
        .collect();
    assert_eq!(speaks.len(), 1);
    assert_eq!(speaks[0].0, "Dead Air Dan");
    assert!(!speaks[0].1.is_empty());
    assert!(speaks[0].1.chars().count() <= 80);

    assert_eq!(
        state.try_rule_bot_taunt(bot_id, BotTauntKind::Death),
        SpeakOutcome::RateLimited
    );
    assert!(
        state
            .take_events()
            .iter()
            .all(|e| !matches!(e, GameEvent::Speak { .. })),
        "rate-limited taunt must not emit Speak"
    );

    state.tick += SPEAK_COOLDOWN_TICKS;
    assert_eq!(
        state.try_rule_bot_taunt(bot_id, BotTauntKind::Killstreak),
        SpeakOutcome::Sent
    );
    assert!(state
        .take_events()
        .iter()
        .any(|e| matches!(e, GameEvent::Speak { .. })));
}

#[test]
fn compliance_boss_does_not_get_scrap_radio_taunts() {
    use crate::protocol::BotTauntKind;
    use crate::sim::SpeakOutcome;

    let mut state = GameState::new();
    let boss_id = Uuid::new_v4();
    state.add_player(boss_id, "COMPLIANCE-DRONE".to_string(), Role::Agent);
    state
        .bots
        .push(BotController::new(boss_id, BotBehavior::Compliance));

    assert!(!state.is_named_rule_bot(boss_id));
    assert_eq!(
        state.try_rule_bot_taunt(boss_id, BotTauntKind::Frag),
        SpeakOutcome::Rejected
    );
    assert!(state
        .take_events()
        .iter()
        .all(|e| !matches!(e, GameEvent::Speak { .. })));
}

#[test]
fn warmup_rule_bot_taunt_path_can_emit_speak() {
    use crate::protocol::BotTauntKind;
    use crate::sim::SpeakOutcome;

    let mut state = GameState::new();
    state.config.warmup_ticks = 200;
    let bot_id = Uuid::new_v4();
    state.add_player(bot_id, "Static Kid".to_string(), Role::Agent);
    state
        .bots
        .push(BotController::new(bot_id, BotBehavior::Flanker));
    assert_eq!(state.round_state, RoundState::Warmup);

    assert_eq!(
        state.try_rule_bot_taunt(bot_id, BotTauntKind::Warmup),
        SpeakOutcome::Sent
    );
    let line = state
        .take_events()
        .into_iter()
        .find_map(|e| match e {
            GameEvent::Speak { text, .. } => Some(text),
            _ => None,
        })
        .expect("Warmup Speak");
    assert!(
        line.contains("glitch") || line.contains("static") || line.contains("tuning"),
        "{line}"
    );

    state.tick += 1;
    assert_eq!(
        state.try_rule_bot_taunt(bot_id, BotTauntKind::Warmup),
        SpeakOutcome::RateLimited
    );
}

#[test]
fn test_sim_respawn_lands_far_from_living_fighters() {
    use crate::sim::SPAWN_SHIELD_TICKS;
    let mut state = GameState::new();
    state.start_round();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    state.add_player(a, "A".into(), Role::Agent);
    state.add_player(b, "B".into(), Role::Agent);
    // Park B on the east ring slot and let A's respawn timer run out.
    for p in state.players.iter_mut() {
        if p.id == b {
            p.x = 15.0;
            p.z = 0.0;
        }
        if p.id == a {
            p.respawn_timer = Some(1);
        }
    }
    state.tick(0.05);
    let a_p = state.players.iter().find(|p| p.id == a).unwrap();
    assert!(a_p.respawn_timer.is_none(), "A should have respawned");
    let dist = ((a_p.x - 15.0).powi(2) + a_p.z.powi(2)).sqrt();
    assert!(dist > 20.0, "respawn lands on the far side, got {dist}");
    assert_eq!(
        state.spawn_shields.get(&a).copied(),
        Some(SPAWN_SHIELD_TICKS)
    );
    assert!(
        !state.spawn_shields.contains_key(&b),
        "joins are not shielded"
    );
}

#[test]
fn test_sim_spawn_shield_blocks_damage_for_one_second() {
    use crate::sim::SPAWN_SHIELD_TICKS;
    let mut state = GameState::new();
    state.start_round();
    let shooter = Uuid::new_v4();
    let target = Uuid::new_v4();
    state.add_player(shooter, "Shooter".into(), Role::Agent);
    state.add_player(target, "Victim".into(), Role::Agent);
    // Kill the victim and let it respawn: the shield is keyed by id, so it can
    // then be moved into the clear hub lane the choke tests use.
    for p in state.players.iter_mut() {
        if p.id == target {
            p.respawn_timer = Some(1);
        }
    }
    state.tick(0.05);
    assert!(state.spawn_shields.contains_key(&target));
    for p in state.players.iter_mut() {
        if p.id == shooter {
            p.x = -5.0;
            p.z = 0.0;
            p.yaw = 0.0;
            p.weapon = WeaponType::Flechette;
            p.fire_cooldown = 0;
        }
        if p.id == target {
            p.x = 5.0;
            p.z = 0.0;
        }
    }
    state.set_action(
        shooter,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);
    let hp = state.players.iter().find(|p| p.id == target).unwrap().hp;
    assert_eq!(hp, 100, "a shielded fighter takes no damage");
    for _ in 0..SPAWN_SHIELD_TICKS {
        state.set_action(shooter, Action::default());
        state.tick(0.05);
    }
    assert!(!state.spawn_shields.contains_key(&target), "shield expires");
    for p in state.players.iter_mut() {
        if p.id == shooter {
            p.fire_cooldown = 0;
        }
    }
    state.set_action(
        shooter,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);
    let hp = state.players.iter().find(|p| p.id == target).unwrap().hp;
    assert!(hp < 100, "the shot lands once the shield is down, hp {hp}");
}

#[test]
fn solo_broadcast_ep0_win_path() {
    let mut session = GameSession::new();
    session.spawn_bots(4);
    session.enable_solo_broadcast_ep0();
    assert!(session.state.solo_broadcast.enabled);
    assert_eq!(session.state.solo_broadcast.phase, EpisodePhase::Nods);

    let human = Uuid::new_v4();
    session
        .state
        .add_player(human, "Meatbag".to_string(), Role::Human);

    // Cold open during Warmup.
    for _ in 0..25 {
        session.state.tick(0.05);
    }
    let events = session.state.take_events();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::EpisodeStart { .. })),
        "expected EpisodeStart, got {events:?}"
    );

    if session.state.round_state != RoundState::Active {
        session.state.start_round();
    }

    let nods: Vec<_> = session
        .state
        .players
        .iter()
        .filter(|p| p.name.starts_with("NODS-") && !p.is_boss)
        .map(|p| p.id)
        .collect();
    assert!(!nods.is_empty(), "expected NODS labels after enable");

    while session.state.solo_broadcast.nods_cleared < session.state.solo_broadcast.nods_goal {
        session.state.note_nods_frag(human);
    }
    assert_eq!(session.state.solo_broadcast.phase, EpisodePhase::Jammer);

    if let Some(p) = session.state.players.iter_mut().find(|p| p.id == human) {
        p.x = 0.0;
        p.z = 0.0;
        p.respawn_timer = None;
    }
    session.state.try_seize_jammer();
    assert_eq!(session.state.solo_broadcast.phase, EpisodePhase::Auditor);
    assert!(session.state.boss_id.is_some());
    assert_eq!(
        session
            .state
            .players
            .iter()
            .find(|p| p.is_boss)
            .map(|p| p.name.as_str()),
        Some(AUDITOR_NAME)
    );

    session
        .state
        .complete_episode("Auditor down. Frequency stays unmetered.".to_string());
    assert_eq!(session.state.solo_broadcast.phase, EpisodePhase::Won);
    let events = session.state.take_events();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::EpisodeComplete { .. })),
        "expected EpisodeComplete, got {events:?}"
    );

    let snap = session.state.snapshot();
    assert_eq!(snap.map_name, EPISODE_MAP_LARAK_LOT);
    assert_eq!(snap.episode_id.as_deref(), Some(EPISODE_ID_EP0));
    assert_eq!(snap.episode_phase.as_deref(), Some("won"));
}

#[test]
fn solo_broadcast_ep0_fail_on_timeout() {
    let mut session = GameSession::new();
    session.spawn_bots(2);
    session.enable_solo_broadcast_ep0();
    session.state.config.warmup_ticks = 1;
    session.state.config.time_limit_ticks = Some(5);
    let human = Uuid::new_v4();
    session
        .state
        .add_player(human, "Meatbag".to_string(), Role::Human);
    for _ in 0..40 {
        session.state.tick(0.05);
        if session.state.solo_broadcast.phase == EpisodePhase::Failed {
            break;
        }
    }
    assert_eq!(session.state.solo_broadcast.phase, EpisodePhase::Failed);
    let events = session.state.take_events();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::EpisodeFail { .. })),
        "expected EpisodeFail, got {events:?}"
    );
}

#[test]
fn solo_broadcast_off_leaves_mp_snapshot_clean() {
    let mut session = GameSession::new();
    session.spawn_bots(2);
    let snap = session.state.snapshot();
    assert!(snap.episode_id.is_none());
    assert!(snap.episode_title.is_none());
    assert!(snap.episode_objective.is_none());
    assert!(snap.episode_progress.is_none());
    assert!(snap.episode_phase.is_none());
    assert_ne!(snap.map_name, EPISODE_MAP_LARAK_LOT);
}
