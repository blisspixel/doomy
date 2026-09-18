#[cfg(test)]
use crate::protocol::{Action, Role};
#[cfg(test)]
use crate::sim::GameState;
#[cfg(test)]
use uuid::Uuid;

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
    use crate::protocol::{GameEvent, Role};
    let mut state = GameState::new(GameConfig::default());
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
        events.iter().any(|e| matches!(e, GameEvent::PlayerJoined { .. })),
        "PlayerJoined must survive tick() and remain until take_events: {:?}",
        events
    );
    assert!(
        events.iter().any(|e| matches!(e, GameEvent::PlayerLeft { .. })),
        "PlayerLeft must survive tick() and remain until take_events: {:?}",
        events
    );
}

