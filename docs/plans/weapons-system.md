# Weapons System Design

## Goal
Add combat depth through distinct weapon roles with different TTK profiles, making bot behavior more readable and combat more engaging.

## Non-goals
- Projectile weapons (complex physics, collision detection)
- Complex inventory UI (keep spectator-first simple)
- Weapon pickups as map entities (defer to later slice)
- Rebalancing existing netcode

## Weapon Roles (3 distinct hitscan variants)

### 1. Blaster (default peashooter)
- Damage: 15 HP/hit
- Fire rate: 6 tick cooldown (300ms, ~3.3 shots/sec)
- Range: 100 units
- TTK: 7 hits = 100 HP (2.1s optimal)
- Role: High-uptime harassment, forgiving aim

### 2. Cannon (hard hitter)
- Damage: 50 HP/hit
- Fire rate: 25 tick cooldown (1250ms, 0.8 shots/sec)
- Range: 120 units
- TTK: 2 hits = 100 HP (1.25s optimal)
- Role: Punish positioning mistakes, precision rewarded

### 3. Scattergun (close range)
- Damage: 8 HP per pellet × 5 pellets = 40 HP max at point-blank
- Fire rate: 15 tick cooldown (750ms, ~1.3 shots/sec)
- Range: 30 units (falloff curve)
- Spread: 0.3 radian cone (5 raycasts)
- TTK: 3 hits point-blank = 120 HP (1.5s optimal), 5+ hits at range
- Role: Close-quarters area denial, flanker weapon

## Protocol Extension

Backward-compatible additions to existing messages:

```json
// PlayerState gains weapon field
{
  "weapon": "blaster" | "cannon" | "scattergun"
}

// Action gains optional weapon_swap
{
  "weapon_swap": "blaster" | "cannon" | "scattergun"  // optional
}
```

## Bot Weapon Selection Strategy

Bots choose weapons based on behavior personality:
- **Aggressive**: Scattergun (rush in, high close DPS)
- **Defensive**: Cannon (range advantage, punish approach)
- **Flanker**: Scattergun (circle-strafe synergy)
- **Balanced**: Blaster (reliable, works at all ranges)

Bots swap weapons during respawn to match their intended role. No mid-combat swaps in initial implementation.

## Implementation Plan

### Phase 1: Core weapon types (server)
- [x] Define `WeaponType` enum
- [x] Add weapon stats struct (damage, cooldown, range, spread)
- [x] Extend `Player` with `weapon: WeaponType`
- [x] Update hitscan logic for single-ray and multi-ray (scattergun)
- [x] Weapon-specific cooldown tracking

### Phase 2: Protocol updates
- [x] Add `weapon` to `PlayerState` serialization
- [x] Add optional `weapon_swap` to `Action`
- [x] Handle weapon swap in tick logic (instant for now)

### Phase 3: Bot integration
- [x] Assign weapons based on `BotBehavior` on spawn
- [x] Bots request weapon on respawn
- [x] Update bot firing logic to account for weapon range/cooldown

### Phase 4: Client updates (Godot)
- [x] Parse weapon from PlayerState
- [x] Display weapon name on nameplate or HUD element
- [x] Optional: different muzzle flash color per weapon

### Phase 5: Testing
- [x] Unit tests for weapon damage calculations
- [x] Unit tests for scattergun spread/falloff
- [x] Unit tests for cooldown enforcement
- [x] Integration test: bots with different weapons fight

## Success Criteria

1. Three distinct weapons implemented with different TTK profiles
2. Bots use weapons matching their behavior (visible in logs/HUD)
3. Unit tests cover damage, cooldown, range for each weapon
4. Godot spectator shows weapon name per player
5. `cargo fmt`, `clippy -D warnings`, `test`, `build` all pass
6. Protocol remains backward-compatible (old clients see default weapon)

## Risks & Mitigations

**Risk**: Scattergun multi-raycast too expensive at 20 Hz × 4 bots
**Mitigation**: Profile; reduce pellet count or optimize raycast if needed

**Risk**: Weapon balance makes one weapon dominant
**Mitigation**: TTK designed for role tradeoffs; iterate values if playtest shows imbalance

**Risk**: Breaking existing agent-adapter or client
**Mitigation**: New fields optional; old Action/Snapshot still valid JSON
