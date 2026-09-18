# Pixel-3D look bar

## Goal

Raise the Godot tip face from graybox + neon zone plates to a **chunky pixel-3D scrap-league arena**: Unreal+CS readability in a real 3D world, Doom-sprite-level silhouettes on billboards (feeling, not IP). Palette locked to art bible: bone-white / gunmetal / rust / blood / ember (muted cyan/magenta accents only).

## Tip priorities

MCP session tools shipped on tip (`8185f1c`, #65). **Pixel-3D look bar is now the tip priority.** look_at / hit HOLD. Speak rate-limit closed.

## Non-goals

- New maps, navmesh, or sim geometry changes on the Rust server
- Photoreal / milsim materials, PBR scrap piles, or neon flood
- New Midjourney / paid asset burn
- look_at / hit-confirm reopen
- Protocol or MCP tool changes
- Coverage work (Godot-only; Rust untouched)

## Architecture impact

| Area | Change |
|---|---|
| `client/scenes/arena.tscn` | Palette materials, hazard/spawn zone treatment, scrap props, WorldEnvironment fog/ambient, scrap-league lighting |
| `client/scenes/player.tscn` | Larger readable Sprite3D silhouettes; weapon billboard scale |
| `client/scripts/player_pawn.gd` | Muted brand tints; preserve sprite art (no neon wash); weapon readability |
| `client/scenes/main.tscn` | HUD chrome tint toward gunmetal / bone (light touch) |
| `docs/plans/README.md` | Tip priorities: look bar in flight; MCP session tools shipped |
| `docs/screenshots/*` + README gallery | Recapture tip stills if they beat graybox shots |

Assets: reuse `client/assets` (and `/workspace/fragr-game-assets` if needed). Prefer tip art already in tree over placeholders.

## Look rules (KAPU)

- **Do:** chunky silhouettes, nearest-filter tiles, scrap hangar mood, readable weapons at spectator distance
- **Don't:** flat 2D-only, gray forever, milsim photoreal, neon puke, Doom/id branding

## Verification

```bash
# Godot-only (no Rust touch; coverage fail-under 80 unchanged)
/workspace/godot/Godot_v4.7.2-stable_linux.x86_64 --path client --headless --import --quit-after 120
# Tip stills
cargo run -p fragr-server -- --bots 4 &
OUT_DIR=docs/screenshots tools/capture_tip_screenshots.sh
```

Inspect stills: floor/wall grit reads, fighters are billboard sprites not white cubes, zones are muted scrap (not candy neon), weapons readable.

## Success

- Plan landed; tip priorities updated
- Playable tip face looks like pixel-3D Contested Frequency scrap, not graybox
- PR open on `cursor/pixel-3d-look-bar`; CI-ready (Godot-only diff)
