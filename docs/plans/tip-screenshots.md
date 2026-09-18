# Tip screenshots

Plan for capturing genuine tip-of-tree screenshots of the fragr Godot client.

## Goal

Produce automated screenshot captures of the running Godot 4.7.2-stable client showing real gameplay (spawned bots, combat, spectator HUD, killfeed, scoreboard) to replace or supplement current mood-stub art in docs and README.

## Non-goals

- Mood art replacement (mood stills remain valid for vision direction, distinct from tip proof)
- Protocol changes (WS to renet or other transport rewrites)
- Changing existing binary mood art files without clear labeling as mood-only
- CI integration gated on screenshot pixel diffs (future polish, not Slice 1 bar)

## Hard technical constraints

**Godot headless rendering limitation:** True `--headless` mode cannot capture pixels. The headless display driver has no framebuffer.

**Required approach for CI or non-GUI capture:**

1. **Xvfb** (X Virtual Framebuffer) to provide a display server without physical screen
2. **OpenGL renderer** (`--rendering-driver opengl3` or `gl_compatibility` project setting) because Vulkan on Xvfb requires lavapipe and introduces complexity
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

**Spike checklist (required before trusting automated stills):**

- [ ] Xvfb + opengl3 + Viewport.get_texture().get_image() produces non-empty arena with bots
- [ ] Killfeed and scoreboard visible after first frag
- [ ] No pink placeholder frames in capture
- [ ] Bot intent chips and names visible
- [ ] Server logs confirm bots spawned and fighting before capture

## Architecture impact

- New optional capture mode (script or Movie Maker config) for client
- No changes to server or protocol
- No changes to existing mood art files (unless explicitly re-labeled as mood-only)
- Potential new `scripts/capture_tip_screenshot.sh` or similar wrapper for Xvfb + Godot

## Verification

1. Run script or capture flow on loopback (server with 4 bots, client with Xvfb)
2. Inspect output PNG: arena visible, bots present, HUD populated, no pink frames
3. Confirm file size and resolution reasonable (e.g. 1920x1080 or 1280x720)
4. Repeat capture shows consistent results (not fluke)

## Success criteria

- Documented, reproducible capture pipeline using Xvfb + opengl3 + Viewport API or Movie Maker
- At least one genuine tip screenshot showing bots fighting, HUD, killfeed
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
