# Tip screenshots

Plan for capturing genuine tip-of-tree screenshots of the fragr Godot client.

## Goal

Produce automated screenshot captures of the running Godot 4.7.2-stable client showing real gameplay (spawned bots, combat, spectator HUD, killfeed, scoreboard) to replace mood-stub art in docs and README tip gallery.

## Non-goals

- Mood art replacement (mood stills remain valid for vision direction under `docs/screenshots/mood/`, distinct from tip proof)
- Protocol changes (WS to renet or other transport rewrites)
- Changing archived mood art files without clear labeling as mood-only
- CI integration gated on screenshot pixel diffs (future polish, not Slice 1 bar)

## Hard technical constraints

**Godot headless rendering limitation:** True `--headless` mode cannot capture pixels. The headless display driver has no framebuffer.

**Required approach for CI or non-GUI capture:**

1. **Xvfb** (X Virtual Framebuffer) to provide a display server without physical screen
2. **OpenGL renderer** (`--rendering-driver opengl3`) because Vulkan on Xvfb requires lavapipe and introduces complexity
3. **Pixel extraction via Viewport API:**
   ```gdscript
   await RenderingServer.frame_post_draw
   var img = get_viewport().get_texture().get_image()
   img.save_png("res://output/screenshot.png")
   ```
4. **Alternative: Movie Maker mode** (`--write-movie output/` with PNG format) if project is configured for it

**Throw-outs (do not pursue):**

- Bare `--headless` screenshot pipelines (framebuffer unavailable)
- Vulkan with lavapipe on Xvfb as default CI path (complexity, maintenance risk)
- Labeling mood-stub art as current tip (misleading)
- Reopening WS to renet transport in this effort (separate concern)
- Inventing `TipCapture.tscn` scene paths or structures without real spike and working proof

## Thin-client hazards requiring spike

The fragr client is a thin presenter. The following conditions may produce misleading or empty screenshots if not handled:

1. **Empty world until network seed:** Bots, arena geometry, and game state arrive over WebSocket. A screenshot taken before the first `Spawn` and `WorldState` messages will show an empty scene.
2. **SubViewport vs root composite:** If using a SubViewport for the 3D scene, ensure the capture targets the correct node (root or SubViewport texture).
3. **Late HUD/frags/killfeed:** UI elements (killfeed, scoreboard, intent chips) may not be populated immediately. Wait for combat (first frag within ~5s of round start) to capture meaningful gameplay.
4. **Splash/shader pink frames:** Godot may render pink placeholders for missing textures or shaders still compiling on first frames. Delay capture until shaders are warm.
5. **Compliance timing:** Continuance compliance ping fires ~15s into Active (after ~2s warmup). Timed multi-shot capture is required to land the pressure chip.

**Spike checklist (required before trusting automated stills):**

- [x] Xvfb + opengl3 + Viewport.get_texture().get_image() produces non-empty arena with bots
  - Evidence: `docs/screenshots/01_arena_overview_16x9.png` (1280x720, colored zone pads + Contested Frequency HUD); Godot log Welcome Mode Contested Frequency/Arena Duel; server spawned Rusher/Sniper/Flanker/Tank
- [x] Killfeed and scoreboard visible after first frag
  - Evidence: `02_spectator_hud_16x9.png` SCRAP LEAGUE scoreboard with behavior chips; `07_tip_compliance_pressure_16x9.png` killfeed `Sniper SCRAPPED Tank`; server FRAG log lines
- [x] No pink placeholder frames in capture
  - Evidence: tip_capture pink heuristic did not trip; inspected PNGs show grey floor + zone colors, no magenta missing-material fill
- [x] Bot intent chips and names visible
  - Evidence: scoreboard `[DEF]/` `[AGG]`/`[FLK]`/`[BAL]`; FOLLOWING lines with role chips; floating nameplates
- [x] Server logs confirm bots spawned and fighting before capture
  - Evidence: `/tmp/fragr-server.log` Spawned bot x4, Round 1 started, multiple FRAG lines, Compliance ping fired
- [x] Contested Frequency mode title + Host bumper + compliance pressure chip
  - Evidence: `02_spectator_hud_16x9.png` shows `CONTESTED FREQUENCY // ARENA DUEL`, `HOST: CONTINUANCE COMPLIANCE PING. APPROVED LANES ONLY.`, `PRESSURE: CONTINUANCE COMPLIANCE`, `APPROVED LANES ONLY`

## Architecture impact

- Optional capture mode: `tools/capture_tip_screenshots.sh` + `client/scripts/tip_capture.gd` (Xvfb + opengl3 + Viewport API, timed multi-shot)
- No changes to server or protocol required for capture
- Mood plates archived under `docs/screenshots/mood/`
- Client scene/script fixes required for honest tip stills (see Status note)

## Verification

1. Run script or capture flow on loopback (server with 4 bots, client with Xvfb)
2. Inspect output PNG: arena visible, bots present, HUD populated, no pink frames
3. Confirm file size and resolution reasonable (1280x720 tip stills)
4. Repeat capture shows consistent Contested Frequency HUD wiring

## Success criteria

- Documented, reproducible capture pipeline using Xvfb + opengl3 + Viewport API
- Genuine tip screenshots showing Contested Frequency HUD, scoreboard, killfeed, Host bumper, compliance pressure
- Spike checklist completed with evidence (logs, output files)
- Plan doc tracks throw-outs and hazards so future contributors do not repeat dead ends

## Spend and safety

- $0 (local only, Xvfb is free)
- No cloud CI apply without approval (CI integration is future polish)

## Godot pin

**4.7.2-stable only.** Do not assume APIs or behaviors from newer or older versions.

## References

- Researcher HOLD brief on Godot tip screenshots (commits 71290cb, ad78d41)
- Godot docs: Viewport.get_texture(), Image.save_png(), Movie Maker mode
- Xvfb manpage and OpenGL vs Vulkan renderer trade-offs

## Status note (tip seal)

Live tip PNGs landed under `docs/screenshots/` and README tip gallery updated. Capture path:

```bash
export PATH=/path/to/godot-4.7.2:$PATH   # binary named godot or Godot_v4.7.2-stable_linux.x86_64
cargo run -p fragr-server --release -- --bots 4 --bind 127.0.0.1:6767
tools/capture_tip_screenshots.sh
```

Client fixes required to load and present Contested Frequency HUD for capture:

- `client/project.godot`: restored corrupted `move_right` InputEventKey `resource_name`
- `client/scenes/arena.tscn`: moved zone `sub_resource` blocks above nodes (Godot format requirement); `emission_energy_multiplier`
- `client/scenes/player.tscn`: added missing `Highlight` MeshInstance3D referenced by `player_pawn.gd`
- `client/scenes/main.tscn`: Contested Frequency / SCRAP LEAGUE label defaults; Host bumper font size fit for 1280 width
- `client/scripts/tip_capture.gd`: timed multi-shot + typed GDScript for warnings-as-errors
