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
