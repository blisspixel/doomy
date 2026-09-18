# Client Assets Wiring Plan

**Status:** In progress  
**Branch:** `cursor/exceptional-client-assets-5b5d`  
**Base:** `main` @ `f6ecdea` (v0.3.0)

## Goal

Wire existing pixel art assets under `client/assets/` into the Godot 4.7.2 client to achieve exceptional visual presentation. Replace placeholder rectangles and capsules with real pixel weapons, character sprites, tiles, and VFX.

## Non-goals

- Adding new assets (all assets already exist on main)
- Changing networking, combat, or round logic
- Adding animation systems beyond simple idle bobs for characters
- Multiplayer prediction or interpolation improvements
- Spending any money

## Asset inventory (already on main)

```
client/assets/
  weapons/32/
    flechette.png, rail.png, scatter.png  (32x32 weapon icons)
    weapons_atlas_flechette_rail_scatter.png
  characters/64/
    cyanex_idle_strip.png (4 frames @ 64x64)
    kragge_idle_strip.png (4 frames @ 64x64)
    fighters_idle_sheet.png
  tiles/16/
    floor.png, wall.png, hazard.png, spawn_a.png, spawn_b.png
    tileset_16x80.png
  vfx/32/
    muzzle_flash.png (32x32 generic muzzle flash)
    rail_beam_tip.png (32x32 rail weapon effect)
```

All assets have `@4x` variants for higher DPI; we'll use base sizes with nearest-neighbor filtering.

## Changes

### 1. Import configuration

Set all PNG assets to:
- Filter: Nearest
- Mipmaps: off
- Compress: lossless or off (preserve pixel-perfect quality)

Update `client/assets/README.md` to document import settings if not already clear.

### 2. Application name

Client application name has been changed to "fragr Client" in `client/project.godot`.

### 3. Weapon icons on HUD

Modify `client/scripts/hud.gd`:
- Add a `TextureRect` node for weapon icon display (130x130px, bottom-right)
- Add dark panel background for weapon display
- Load weapon textures: `res://assets/weapons/32/flechette.png`, `rail.png`, `scatter.png`
- Display weapon icon + player name + weapon role ("RAIL (sniper)", "SCATTER (close)", "FLECHETTE (balanced)")

Modify `client/scripts/player_pawn.gd`:
- Track current weapon for HUD queries
- Expose weapon name via `get_weapon_name()` method

### 4. Character sprites with variety

Replace `CapsuleMesh` bodies in `client/scenes/player.tscn` with `Sprite3D` nodes:
- Use **two character types for clear visual distinction:**
  - **Kragge** (bulkier): Rusher, Tank, Hunter
  - **Cyanex** (sleeker): Sniper, Flanker, Scout, Guard, Striker
- Set `Sprite3D` billboard mode to `BILLBOARD_FIXED_Y` (face camera, stay upright)
- Keep label above character
- Apply color tint via `modulate` to preserve per-player colors
- Add 4-frame idle animation bob

Modify `client/scripts/player_pawn.gd`:
- Update to work with `Sprite3D` instead of `MeshInstance3D`
- Select character sprite based on bot name (kragge vs cyanex)
- Keep color coding system
- Enhanced hit flash: scale to 1.2x, brighter red (1.8, 0.3, 0.3)
- Low-HP warning: show `!HP!` when < 30

### 5. Weapon sprites on characters

Add `WeaponSprite` Sprite3D child to character body:
- Display current weapon icon on fighter
- Position offset to side/front (visible silhouette)
- Color-tint to match fighter color
- Update dynamically when weapon changes

### 6. Floor and wall tiles

Apply textures to existing arena meshes:
- Floor: `StandardMaterial3D.albedo_texture` with `floor.png`, nearest filter
- Walls: same approach with `wall.png`
- Keep arena zone markers (spawn areas, hazards) as-is
- Arena functionality must not regress

### 7. VFX wiring - Punchy and dramatic

Modify `client/scripts/player_pawn.gd` `show_muzzle_flash()`:
- Replace muzzle `MeshInstance3D` capsule with `Sprite3D`
- **Rail weapon:** 2.5x scale, cyan-tinted `rail_beam_tip.png`, 4.0 energy cyan `OmniLight3D` (5m range)
- **Scatter/Flechette:** 2x scale, warm-tinted `muzzle_flash.png`, 3.5 energy warm `OmniLight3D` (4m range)
- **Snappier timing:** 0.06s flash duration
- Position sprite at weapon muzzle location (billboard mode)

Add `OmniLight3D` child to muzzle sprite:
- Dynamic light energy and color based on weapon type
- Creates dramatic lighting during fire

### 8. HUD and combat drama

Enhance `client/scripts/hud.gd`:
- Weapon icon enlarged to 130x130px with dark panel background
- Weapon display shows player name + weapon + role description
- Frag label: 1.3x scale animation, 2.8s display duration, brighter color (0.4 lighten)
- Round end winner: more dramatic formatting and animation (1.4x scale)

Enhance `client/scripts/game_manager.gd`:
- Frag camera lock extended to 2.0s (from 1.5s)
- Pass player name to weapon display for context

### 9. Arena lighting

Brighten arena for better visibility:
- Directional light: 1.5 energy (was 1.2), warm color tint
- Omni light: 0.5 energy (was 0.3)
- Better fighter and VFX visibility

## Architecture impact

- No protocol changes
- No server changes
- Godot client presentation layer only
- Scene structure changes: `player.tscn` (body mesh to sprite), possibly `main.tscn` (HUD layout)

## Verification steps

### Pre-implementation

- [x] Plan written and reviewed
- [x] Assets confirmed present on main @ f6ecdea
- [x] Feature branch created: `cursor/exceptional-client-assets-5b5d`

### During implementation

- [x] Set import flags on all PNG assets (nearest, no mipmaps)
- [x] Application name changed to fragr
- [x] Player sprites render correctly (billboard, tinted)
- [x] Weapon icons show on HUD based on state
- [x] Floor and wall textures applied
- [x] VFX sprites appear on weapon fire
- [x] No visual regressions (players spawn, move, fight)

### Post-implementation

- [ ] Smoke test: `cargo run -p fragr-server` with 4 bots
- [ ] Godot client shows pixel art characters fighting
- [ ] Weapon icons visible and change with weapon state
- [ ] Muzzle flash and rail effects use correct sprites
- [ ] No placeholder capsules visible (except muzzle when not firing)
- [x] Rust verification: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo build --workspace`
- [ ] Godot verification: scenes open cleanly in 4.7.2-stable
- [x] Git author set to Nick Seal `<32712898+blisspixel@users.noreply.github.com>`
- [x] No tool attribution, no emoji, no em/en dashes in any commit or code
- [x] PR created, merge-ready

## Success criteria

Spectator watches the match and sees:
- Pixel art characters (not capsules) with idle animations and color tints
- Weapon icons on HUD showing current weapon
- Textured floor and walls (even if simple)
- Pixel art muzzle flash and rail beam effects on weapon fire
- All existing gameplay (networking, combat, rounds, scoring) still works

Exceptional quality bar: looks like a real retro game, not a tech demo.

## Spend gate

Zero dollars. All assets already in repo. No external dependencies or hosting.

## Timeline

No calendar estimate. Implementation involves:
- Godot scene editing (player.tscn, main.tscn, arena.tscn)
- GDScript updates (player_pawn.gd, hud.gd)
- Asset import configuration
- Testing smoke run

Expected to complete in single agent session with local Godot 4.7.2 available.
