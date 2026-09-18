# Far-cam fighter scale

## Goal

Keep Cyanex / Kragge billboard silhouettes chunky and readable when the spectator camera is far (free-fly or tip overview). Close follow cam must stay unchanged in feel.

## Tip priorities

Pixel-3D look bar sealed on tip (`5d16f32`, #66). **Far-cam fighter scale is the tip priority.** look_at / hit HOLD. join / leave / host-flash HOLD.

## Non-goals

- Camera path or follow-offset retune
- look_at / hit-confirm reopen
- Server / protocol / MCP changes
- New art or paid assets
- Photoreal LOD meshes

## Architecture impact

| Area | Change |
|---|---|
| `client/scripts/player_pawn.gd` | Distance-aware Sprite3D scale curve from active Camera3D; compose with hit flash |
| `client/scripts/test_far_cam_scale.gd` | Headless asserts on the scale curve |
| `docs/plans/README.md` | Tip priorities: far-cam NOW; pixel-3D look bar shipped |
| Tip stills (optional) | Recapture overview if far silhouettes read clearly better |

## Scale rules

- Follow distance is ~12m (`spectator_cam` offset `(0, 5.5, 10.5)`). At or below REF (12m), scale multiplier is `1.0` (close follow unchanged).
- Beyond REF, scale is `min(distance / REF, 3.5)` so overview / free-fly (~36m) lands near 3x, not tiny gray dots.
- Hit flash boost multiplies on top of far-cam scale (does not reset it to `Vector3.ONE`).
- Body children (weapon / muzzle) inherit body scale. Label gets the same far-cam multiplier for name readability.

## Verification

```bash
/workspace/godot/Godot_v4.7.2-stable_linux.x86_64 --path client --headless --script res://scripts/test_far_cam_scale.gd
# Optional tip still (needs fragr-server + Xvfb)
cargo run -p fragr-server -- --bots 4 &
PATH="/workspace/godot:$PATH" OUT_DIR=docs/screenshots tools/capture_tip_screenshots.sh
```

Inspect overview still: fighters remain chunky at far spectator distance; follow-cam combat stills do not look oversized.

## Success

- Plan landed; tip priorities updated
- Far spectators can still read fighter silhouettes
- Close follow unchanged
- PR open on `cursor/far-cam-fighter-scale`; CI-ready (Godot-only diff)
