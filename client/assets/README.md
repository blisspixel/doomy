# client/assets - fragr Godot pack (sprites-as-code v2)

RGBA pixel sprites. Import: **Nearest**, mipmaps **off**. Use base sizes (32/64/16).

## Import settings

All PNG assets are configured with `.import` files for pixel-perfect rendering:
- Filter mode: Nearest (0) - no blurring/smoothing
- Mipmaps: disabled - prevents mipmap blurring at distance
- Compression: mode 0 (lossless or uncompressed)
- detect_3d/compress_to: 0 - preserves quality for 3D billboard sprites

When adding new assets, ensure import settings match existing .import files.

## weapons/32
- `flechette.png` `rail.png` `scatter.png` - WeaponType v1
- `weapons_atlas_flechette_rail_scatter.png`
- `_future/` - rocket_tube, shock_pistol, gravity_baton (do not wire)

## characters/64
- `cyanex_idle.png` `kragge_idle.png` - skin labels
- `*_idle_0..3.png` + `*_idle_strip.png` - 4-frame idle bob
- `fighters_idle_sheet.png`

## tiles/16
- `floor` `wall` `hazard` `spawn_a` `spawn_b` + `tileset_16x80.png`

## vfx/32
- `muzzle_flash.png` `rail_beam_tip.png`

## ui
- `on_air.png` `contested_frequency.png` `hangar_candy.png` `chrome-strip.png` - full plates
- `on_air_badge.png` `contested_frequency_badge.png` `hangar_candy_badge.png` `chrome_strip_hud.png` - HUD crops (nearest)
- Contested Frequency / Hangar Candy / ON AIR broadcast grit. Dull, not neon.

Boomer-arena. No Doom IP.
