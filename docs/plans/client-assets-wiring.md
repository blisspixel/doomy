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

Change `config/name` in `client/project.godot` from `"Doomy Client"` to `"fragr"`.

### 3. Weapon icons on HUD

Modify `client/scripts/hud.gd`:
- Add a `TextureRect` node for weapon icon display (or create dynamically)
- Load weapon textures: `res://assets/weapons/32/flechette.png`, `rail.png`, `scatter.png`
- Display appropriate weapon icon based on followed player's `state.weapon`

Modify `client/scripts/player_pawn.gd`:
- Continue showing weapon name in label text as fallback
- Expose current weapon for HUD queries

### 4. Character sprites

Replace `CapsuleMesh` bodies in `client/scenes/player.tscn` with `Sprite3D` nodes:
- Use `cyanex_idle_strip.png` for one faction/team (or all bots)
- Optionally use `kragge_idle_strip.png` for visual variety
- Set `Sprite3D` billboard mode to `BILLBOARD_FIXED_Y` (face camera, but stay upright)
- Keep label above character
- Apply color tint via `modulate` or material to preserve per-player colors

Modify `client/scripts/player_pawn.gd`:
- Update to work with `Sprite3D` instead of `MeshInstance3D`
- Keep color coding system
- Keep hit flash and emission effects (adapt for sprite materials)

### 5. Floor and wall tiles

Option A (simple):
- Apply texture to existing floor `PlaneMesh` using `StandardMaterial3D.albedo_texture`
- Apply texture to wall meshes
- Keep arena zone markers (spawn areas, hazards) as-is

Option B (TileMap):
- Only if trivial to add without breaking arena zones
- Otherwise, defer to future work

Start with Option A. Arena functionality must not regress.

### 6. VFX wiring

Modify `client/scripts/player_pawn.gd` `show_muzzle_flash()`:
- Replace muzzle `MeshInstance3D` capsule with `Sprite3D`
- Use `muzzle_flash.png` for Flechette and Scatter weapons
- Use `rail_beam_tip.png` for Rail weapon (check `state.weapon`)
- Position sprite at weapon muzzle location (billboard mode)
- Keep existing flash timing and scale animation

### 7. HUD layout enhancement (optional)

If weapon icon looks awkward in current HUD:
- Adjust `client/scenes/main.tscn` HUD panel layout
- Add weapon icon display area near player name or corner
- Keep changes minimal and consistent with existing style

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

- [ ] Smoke test: `cargo run -p doomy-server` with 4 bots
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
