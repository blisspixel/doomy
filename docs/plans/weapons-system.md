# Weapons System Implementation

**Status:** Implementation
**Owner:** blisspixel
**Spend:** $0

## Goal

Add distinct weapon roles (Flechette, Rail, Scatter) to fragr with different damage, cooldown, and spread characteristics. Players can swap weapons during gameplay, adding tactical depth to combat.

## Non-goals

- Weapon pickups or ammo system (all weapons always available)
- Visual weapon models (optional asset loading only)
- Animations or complex VFX (just muzzle flash flag)
- Projectile physics (hitscan only for Slice 1)

## Architecture impact

### Protocol changes

Add to `Action`:
- `weapon_swap: Option<WeaponType>` (validated on server)

Add to `PlayerState`:
- `weapon: String` (current weapon name)

Add new `WeaponType` enum:
- `Flechette` (default, balanced)
- `Rail` (high damage, long cooldown, tight spread)
- `Scatter` (low damage, short cooldown, wide spread)

### Server changes

Add to `sim.rs`:
- `WeaponType` enum with weapon stats (damage, cooldown, spread)
- `Player.weapon: WeaponType` field
- Weapon swap validation in action processing
- Hit detection uses weapon-specific spread angle

Constants to replace:
- Remove global `HITSCAN_DAMAGE`, `FIRE_COOLDOWN_TICKS`
- Use per-weapon stats from `WeaponType` methods

### Client changes

Add to player label display:
- Show weapon name below player name
- Optional: load `res://assets/weapons/32/<name>.png` if exists (graceful fallback)

## Weapon specifications

### Flechette (default)
- Damage: 25
- Cooldown: 10 ticks (0.5s @ 20Hz)
- Spread: 0.1 radians (tight, ~5.7 degrees)
- Role: Balanced all-purpose weapon

### Rail
- Damage: 75
- Cooldown: 40 ticks (2.0s @ 20Hz)
- Spread: 0.05 radians (very tight, ~2.9 degrees)
- Role: High-skill precision weapon

### Scatter
- Damage: 15
- Cooldown: 5 ticks (0.25s @ 20Hz)
- Spread: 0.3 radians (wide, ~17.2 degrees)
- Role: Close-range spam weapon

## Verification steps

1. Unit tests proving each weapon has different:
   - Damage values
   - Cooldown timings
   - Hit accuracy at various angles
2. Action validation rejects invalid weapon swaps
3. Defensive checks on all new fields (Option unwrapping, enum parsing)
4. cargo fmt, clippy -D warnings, test, build all green

## Success criteria

- Each weapon role behaves distinctly in combat
- Players can swap weapons via Action message
- Client displays current weapon
- All tests pass, no clippy warnings
- No regressions to rounds/scoreboard/MCP
- $0 spend

## Implementation order

1. Add `WeaponType` enum with stats methods
2. Add weapon field to Player
3. Add weapon_swap to Action and validation
4. Update hit detection to use weapon spread
5. Add weapon to PlayerState snapshot
6. Unit tests for each weapon role
7. Client weapon display (GDScript)
8. Update protocol.md

## Safety gates

- No paid assets or cloud services
- Author: Nick Seal <32712898+blisspixel@users.noreply.github.com>
- No attribution comments
- No emoji or em/en dashes
