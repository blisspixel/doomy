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

1. **Render target and dither.** Godot 4.7's `Viewport.SCALING_3D_MODE_NEAREST` with `rendering/scaling_3d/scale` at 0.25 under the `canvas_items` stretch mode: the 3D pass renders at a quarter of the window (480 by 270 at 1080p), upscales by an exact integer with nearest filtering, and the HUD stays native. The renderer switches to Compatibility (OpenGL 3.3, low base cost, keeps depth fog, glow, and baked lighting; loses nothing a Doom look needs). An optional ordered-dither post shader (4 by 4 Bayer) on a `ColorRect` in a canvas layer below the HUD, reading the upscaled 3D buffer and indexing the threshold matrix by the screen position divided by the scale factor so the pattern aligns to the 3D pixel grid (2D is never rendered at low resolution), quantising to `docs/palette.json`, toggleable in the boot menu ("Pixel: on/off") and off in the tip capture harness until the stills are re-approved. Evidence: before and after stills, the far-cam harness still passes, frame time unchanged within noise.
2. **Surface atlas.** One 16 by 16 tile atlas per map family (floor, wall, trim, hazard, spawn, crate, pillar) in the locked palette with baked highlight and shadow rows; `StandardMaterial3D` with nearest filtering, no mipmaps, triplanar off, UV scaled so one tile is one metre. Arena Duel and Compliance Yard re-dressed as they stand (geometry is still hardcoded in the server); when the campaign moves geometry to `.map` files, both arenas are re-exported with the same atlas and texel constant. Evidence: stills from both maps, the palette validator in the art bible run over the atlas.
3. **Fighter sprites.** Eight facing directions per fighter (Cyanex and Kragge skins), frames for idle, walk (4), fire (2), pain (1), death (4), at 64 pixels; a `Sprite3D` billboard aligned to the camera with a normal map so lights hit it; direction chosen from the fighter's yaw relative to the camera; enemy tint forced per team colour. Evidence: a headless harness that resolves direction and frame from yaw and state; stills.
4. **Weapon view models and effects.** Flechette, Rail, and Scatter hand sprites with idle bob, fire (2 frames), and a reload beat; muzzle flash frames; impact sprites by surface; sprite gibs on overkill with the DENIED watermark. Evidence: stills, the existing fire-juice path still fires the HUD kick.
5. **HUD grid, sprites over text.** The screen is not a readout. Health, armour, ammo, weapon, and pickups read as sprites and icons the way Vampire Survivors and the boomer shooters do: a status face or bar, a weapon sprite with an ammo count in a pixel numeral, an icon for a pickup in view, station art for the radio card, and one line of text at most on screen at any moment outside menus and the killfeed. Text is for names in the killfeed, the Host line, and menus. All HUD elements sit on an 8 pixel grid with fixed regions: top left match state, top right killfeed, bottom left health and armour, bottom right weapon with the radio station card and track toast above it (the radio plan keeps the card out of the killfeed corner); no overlapping labels at 640 by 360; the ON AIR booth portrait replaces the mode label block. Evidence: a headless layout check that asserts no two HUD rectangles intersect, stills at 16:9 and 16:10.

## Research notes (2026-09-18)

- **Low resolution the 4.7 way.** `SCALING_3D_MODE_NEAREST` landed in 4.7 for all three renderers and is 3D only; the docs say to use a scale equal to one over an integer (0.5, 0.3333, 0.25, 0.2). 2D is never scaled, so the HUD stays crisp for free. `canvas_items` stretch is the 4.7 default. The older `SubViewportContainer` recipe with a fixed 480 by 270 base and `scale_mode = integer` gives a strict pixel grid for the HUD at the cost of black bars on odd window sizes; recipe one above is simpler and is the default, recipe two is the fallback if the HUD grid stage needs it. Never `viewport` stretch at low resolution (text gets downsampled), never bilinear (blur), never FSR (Forward Plus only). The engine default for stretch mode is still `disabled`; only projects created in 4.7 get `canvas_items` written in, so fragr sets it explicitly. Post shaders in recipe one run at native resolution over the upscaled buffer, hence the grid-aligned threshold above; recipe two is the path if a shader truly must run inside the low-res pass.
- **Renderer.** Compatibility lacks SDFGI, VoxelGI, volumetric fog, SSR, TAA, FSR2, CompositorEffects, and HDR 2D (it does keep SSAO, MSAA 3D, depth fog, glow, and lightmap rendering). None of the missing features serve this look. It runs on old integrated GPUs without Vulkan. Keep Mobile as a switch only if a CompositorEffect is ever needed.
- **Shaders.** Screen-reading shaders use `hint_screen_texture` with `filter_nearest` and `textureLod`; stacked effects need a `BackBufferCopy` between them. Order: posterise and dither inside the low-res pass, CRT or scanlines at native resolution after the upscale. Sources with permissive licences: Bayer dithering (CC0), retro post-processing with bit depth and 4 by 4 dither (CC0), dither gradient in the Obra Dinn style (MIT), PixelDither palette quantiser (MIT), the godot-color-dither collection (CC0 shaders), CRT Lottes (CC0), all on godotshaders.com or GitHub. The Obra Dinn devlog on stable dithering is the primary write-up.
- **Eight-direction billboards.** `Sprite3D` with `BILLBOARD_FIXED_Y`, `TEXTURE_FILTER_NEAREST` on every sprite (the default is linear with mipmaps), `ALPHA_CUT_DISCARD` to dodge transparency sorting, `shaded = false`. Frame from `wrapf(atan2(to_camera.x, to_camera.z) - yaw, -PI, PI)` divided into eight; store five directions and flip the other three, the Doom trick. Billboards face the camera when rendering shadows, so use a blob shadow sprite on the floor instead.
- **Weapon view models.** Under recipe one the view model is a `Sprite3D` child of the camera (no depth test, alpha cut, unshaded) so it shares the low-res pass and the dither; bob is a sine on speed, sway lerps toward the negated mouse delta, muzzle flash is a sheet frame plus a one-frame `OmniLight3D`. Lighting: unshaded sprites plus depth fog is the Doom look; arenas can go vertex-lit with the force-vertex-shading override; sector light through `vertex_color_use_as_albedo`.
- **Import and texel density.** Lossless import for pixel art, VRAM compression off even in 3D, mipmaps off, Detect 3D off. Filtering is per node in 4.x. One atlas with `region_rect` or `hframes` and `vframes`. There is no texel density feature; fix one constant (16 texels per metre, Doom's one texel per unit scaled) and keep it with `uv1_scale` or, once maps come from TrenchBroom, func_godot's inverse scale factor.
- **References.** bearlikelion's BoomerShooter (MIT, GDScript, 4.7 Forward Plus, not low-res), Zorochase's ultimate-retro-shader-collection (MIT, 4.2 to 4.7, PSX and N64 spatial shaders, billboard sprite shaders, dither on a container), psx_visuals_gd4 (MIT port). The Game Engine Black Book: DOOM covers sprite rotations and colormap light diminishing.
- **Asset production.** Pixelorama (MIT, built in Godot) and LibreSprite (GPL, output unencumbered). Godot has no palette quantise call, so the palette validator in the art bible becomes a GDScript `EditorImportPlugin` that snaps pixels to the palette with an optional Bayer threshold and packs sheets with `blit_rect`, or a Rust CLI with the `image` crate doing the same offline. Blender direction renderers exist as Python add-ons; they run inside Blender and are not repo code, but prefer the Rust packer path.

## Verification

- `tools/godot_check.sh` (import, parse, harnesses) on every stage.
- `tools/capture_tip_screenshots.sh` regenerated stills committed with each stage; `docs/screenshots/README.md` updated to describe what is live.
- A frame-time note in the PR: 60 frames per second at 1280 by 720 on the reference machine with four bots, measured with the engine's monitor.

## Success criteria

- [ ] Stage 1 lands with the toggle and stills.
- [ ] Both maps re-dressed with the atlas.
- [ ] Fighters read at thirty metres: the far-cam harness asserts sprite height in pixels at 30 m, and a still shows direction and state.
- [ ] Every weapon is identifiable by its view model silhouette alone: a silhouette contact sheet of the three view models committed with the stills.
- [ ] No overlapping HUD elements in any tip still, and no still outside menus with more than one line of HUD text plus the killfeed.
