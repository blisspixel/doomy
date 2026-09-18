# Plan: boomer shooter look pass

**Status:** planned (2026-09-18)
**Branch:** `feat/look-pass-*` (one PR per stage below)
**Spend:** $0. Art is authored in-repo (pixel plates, palette, shaders). No paid assets.

## Goal

Make the tip look like a modern pixel-art boomer shooter instead of a textured graybox: a low internal resolution with nearest upscaling, surfaces limited to the locked palette with ordered dithering, fighters as readable eight-direction sprites with walk, fire, pain, and death frames, weapon view models with idle, fire, and bob frames, muzzle and impact frames, a HUD laid out on a grid, and level surfaces built from a coherent tile atlas with baked lighting and trim. Reference points and the reasons behind each choice are in `docs/DESIGN-REFERENCES.md` (Dusk, Amid Evil, Prodeus, Cultic, Nightmare Reaper rows) and the palette lock in `docs/ART_STORY_BIBLE.md`.

## Non-goals

- Photoreal or milsim rendering, post-process bloom floods, neon.
- New weapons or maps (this pass re-dresses what exists).
- Changing the wire protocol or the server. Presentation only.
- Doom or id lookalikes in sprites, weapons, or HUD.

## Stages (each a PR that passes the Godot CI job and ships regenerated tip screenshots)

1. **Render target and dither.** A `SubViewport` at 640 by 360 with nearest-neighbour integer upscale to the window, `Forward Plus` kept, the HUD in a native-resolution `CanvasLayer`. An optional ordered-dither post shader (4 by 4 Bayer) quantising to `docs/palette.json`, toggleable in the boot menu ("Pixel: on/off") and off in the tip capture harness until the stills are re-approved. Evidence: before and after stills, the far-cam harness still passes, frame time unchanged within noise.
2. **Surface atlas.** One 16 by 16 tile atlas per map family (floor, wall, trim, hazard, spawn, crate, pillar) in the locked palette with baked highlight and shadow rows; `StandardMaterial3D` with nearest filtering, no mipmaps, triplanar off, UV scaled so one tile is one metre. Arena Duel and Compliance Yard re-dressed. Evidence: stills from both maps, the palette validator in the art bible run over the atlas.
3. **Fighter sprites.** Eight facing directions per fighter (Cyanex and Kragge skins), frames for idle, walk (4), fire (2), pain (1), death (4), at 64 pixels; a `Sprite3D` billboard aligned to the camera with a normal map so lights hit it; direction chosen from the fighter's yaw relative to the camera; enemy tint forced per team colour. Evidence: a headless harness that resolves direction and frame from yaw and state; stills.
4. **Weapon view models and effects.** Flechette, Rail, and Scatter hand sprites with idle bob, fire (2 frames), and a reload beat; muzzle flash frames; impact sprites by surface; sprite gibs on overkill with the DENIED watermark. Evidence: stills, the existing fire-juice path still fires the HUD kick.
5. **HUD grid.** All HUD text on an 8 pixel grid with fixed regions: top left match state, top right radio and killfeed, bottom left health and armour, bottom right weapon; no overlapping labels at 640 by 360; the ON AIR booth portrait replaces the mode label block. Evidence: a headless layout check that asserts no two HUD rectangles intersect, stills at 16:9 and 16:10.

## Verification

- `tools/godot_check.sh` (import, parse, harnesses) on every stage.
- `tools/capture_tip_screenshots.sh` regenerated stills committed with each stage; `docs/screenshots/README.md` updated to describe what is live.
- A frame-time note in the PR: 60 frames per second at 1280 by 720 on the reference machine with four bots, measured with the engine's monitor.

## Success criteria

- [ ] Stage 1 lands with the toggle and stills.
- [ ] Both maps re-dressed with the atlas.
- [ ] Fighters read at thirty metres with direction and state visible.
- [ ] Every weapon is identifiable by its view model silhouette alone.
- [ ] No overlapping HUD text in any tip still.
