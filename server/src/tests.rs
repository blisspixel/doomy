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
