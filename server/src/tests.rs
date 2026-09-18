#[cfg(test)]
use crate::protocol::{Action, Role, WeaponType};
#[cfg(test)]
use crate::sim::{GameState, WeaponStats};
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

    state.add_player_with_weapon(shooter_id, "Shooter".to_string(), WeaponType::Rail);
    state.add_player_with_weapon(target_id, "Target".to_string(), WeaponType::Flechette);

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
    state.players[target_idx].hp = 40;
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

#[test]
fn test_weapon_stats_flechette() {
    let stats = WeaponStats::for_weapon(WeaponType::Flechette);
    assert_eq!(stats.damage, 15);
    assert_eq!(stats.cooldown_ticks, 6);
    assert_eq!(stats.range, 100.0);
    assert_eq!(stats.pellets, 1);
    assert_eq!(stats.spread, 0.0);
}

#[test]
fn test_weapon_stats_rail() {
    let stats = WeaponStats::for_weapon(WeaponType::Rail);
    assert_eq!(stats.damage, 50);
    assert_eq!(stats.cooldown_ticks, 25);
    assert_eq!(stats.range, 120.0);
    assert_eq!(stats.pellets, 1);
    assert_eq!(stats.spread, 0.0);
}

#[test]
fn test_weapon_stats_scatter() {
    let stats = WeaponStats::for_weapon(WeaponType::Scatter);
    assert_eq!(stats.damage, 8);
    assert_eq!(stats.cooldown_ticks, 15);
    assert_eq!(stats.range, 30.0);
    assert_eq!(stats.pellets, 5);
    assert!(stats.spread > 0.0);
}

#[test]
fn test_blaster_damage_and_cooldown() {
    let mut state = GameState::new();

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player_with_weapon(shooter_id, "Shooter".to_string(), WeaponType::Flechette);
    state.add_player_with_weapon(target_id, "Target".to_string(), WeaponType::Flechette);

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
    assert_eq!(initial_hp - final_hp, 15, "Flechette should deal 15 damage");

    let cooldown = state.players[shooter_idx].fire_cooldown;
    assert_eq!(cooldown, 6, "Flechette cooldown should be 6 ticks");
}

#[test]
fn test_cannon_damage_and_cooldown() {
    let mut state = GameState::new();

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player_with_weapon(shooter_id, "Shooter".to_string(), WeaponType::Rail);
    state.add_player_with_weapon(target_id, "Target".to_string(), WeaponType::Flechette);

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
    assert_eq!(initial_hp - final_hp, 50, "Rail should deal 50 damage");

    let cooldown = state.players[shooter_idx].fire_cooldown;
    assert_eq!(cooldown, 25, "Rail cooldown should be 25 ticks");
}

#[test]
fn test_scatter_close_range_damage() {
    let mut state = GameState::new();

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player_with_weapon(shooter_id, "Shooter".to_string(), WeaponType::Scatter);
    state.add_player_with_weapon(target_id, "Target".to_string(), WeaponType::Flechette);

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

    state.players[target_idx].x = 2.0;
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
    let damage = initial_hp - final_hp;

    assert!(
        damage >= 32,
        "Scatter at close range should hit with 4+ pellets (32+ damage), got {}",
        damage
    );
    assert!(
        damage <= 40,
        "Scatter max damage is 40 (5 pellets × 8), got {}",
        damage
    );

    let cooldown = state.players[shooter_idx].fire_cooldown;
    assert_eq!(cooldown, 15, "Scatter cooldown should be 15 ticks");
}

#[test]
fn test_scatter_out_of_range() {
    let mut state = GameState::new();

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player_with_weapon(shooter_id, "Shooter".to_string(), WeaponType::Scatter);
    state.add_player(target_id, "Target".to_string(), Role::Human);

    let shooter_idx = 0;
    let target_idx = 1;

    assert_eq!(state.players[shooter_idx].id, shooter_id);
    assert_eq!(state.players[target_idx].id, target_id);

    state.players[shooter_idx].x = 10.0;
    state.players[shooter_idx].z = 10.0;
    state.players[shooter_idx].yaw = 0.0;
    state.players[target_idx].x = 10.0;
    state.players[target_idx].z = -30.0;

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
    assert_eq!(
        final_hp, initial_hp,
        "Scatter behind shooter should not hit"
    );
}

#[test]
fn test_weapon_swap() {
    let mut state = GameState::new();
    let player_id = Uuid::new_v4();

    state.add_player_with_weapon(player_id, "Swapper".to_string(), WeaponType::Flechette);

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
}

#[test]
fn test_cannon_two_shot_frag() {
    let mut state = GameState::new();

    let shooter_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    state.add_player_with_weapon(shooter_id, "Shooter".to_string(), WeaponType::Rail);
    state.add_player_with_weapon(target_id, "Target".to_string(), WeaponType::Flechette);

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

    let target_hp = state.players[target_idx].hp;
    assert_eq!(
        target_hp, 50,
        "First cannon shot should leave target at 50 HP"
    );

    for _ in 0..25 {
        state.tick(0.05);
    }

    state.set_action(
        shooter_id,
        Action {
            fire: true,
            ..Default::default()
        },
    );
    state.tick(0.05);

    let target = &state.players[target_idx];
    assert!(target.hp <= 0, "Second cannon shot should kill target");
    assert!(
        target.respawn_timer.is_some(),
        "Target should be respawning"
    );
}
