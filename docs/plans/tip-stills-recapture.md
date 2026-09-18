# Tip stills recapture (post weapon-roles)

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/tip-stills-recapture`
**Spend:** $0. No ElevenLabs, no GCP apply, no renet, no look_at/hit reopen.
**Status:** In flight. Live Xvfb+Godot recapture completed on box (01/02/07/08/09/10).

## Goal

Recapture live Godot tip stills so choke geometry, Join FP juice, killstreak Host chrome, and weapon-roles face match current tip. Keep `10_tip_human_join_fp` in the docs/screenshots tip table. Live captures only (Xvfb + opengl3); mood plates stay archived under `mood/`.

## Tip priorities

Weapon roles excellence shipped (#76 / `9734faf`). **Tip stills recapture is the NOW tip.** look_at / hit HOLD. Port 6767. Coverage fail-under 80.

## Non-goals

- GCP apply
- ElevenLabs
- look_at / hit reopen (HOLD)
- New gameplay features or protocol changes
- Relabeling mood art as tip proof
- Changing coverage floor

## Locks

- Contested Frequency pixel grit (bone / gunmetal / blood / ember)
- Port 6767
- Coverage fail-under 80 (unfiltered)
- look_at / hit HOLD
- No GCP apply

## Stills to refresh

| File | Why |
|---|---|
| `01_arena_overview_16x9.png` | Arena + Contested Frequency HUD after choke + roles |
| `02_spectator_hud_16x9.png` | Host compliance bumper / scoreboard face |
| `07_tip_compliance_pressure_16x9.png` | Mid-scrap killfeed + pressure chip |
| `08_tip_host_flash_midjoin_16x9.png` | Mid-join Host flash (killstreak chrome path) |
| `09_tip_weapons_frags_16x9.png` | Weapons / muzzle / frags with role face |
| `10_tip_human_join_fp_16x9.png` | Join FP crosshair + weapon face after roles |

## Capture path (existing)

1. `cargo run -p fragr-server -- --bind 127.0.0.1:6767 --bots 4`
2. Godot 4.7.2-stable on PATH as `godot`
3. `tools/capture_tip_screenshots.sh` (Xvfb + opengl3; `LIBGL_ALWAYS_SOFTWARE=1`)
4. Driver: `client/scripts/tip_capture.gd` timed multi-shot + mid-join Host flash + human Join FP

If Godot/Xvfb fails on the box, document the exact blocker and still ship docs/table fixes plus any stills produced.

## Docs updates

- `docs/screenshots/README.md`: tip table keeps `10_tip_human_join_fp`; status text reflects recapture after weapon roles
- `README.md`: embeds unchanged unless paths change
- `docs/plans/README.md`: tip priorities: weapon roles shipped; this NOW

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
# Capture (when display path works):
cargo run -p fragr-server -- --bind 127.0.0.1:6767 --bots 4 &
tools/capture_tip_screenshots.sh
# Inspect PNGs: Contested Frequency, scoreboard/killfeed/Host, FP crosshair, no pink frames
```

Rust tests only need to stay green if this PR touches Rust (expected: docs + PNGs only). Coverage floor unchanged.

## Success criteria

- Plan in tree; tip priorities list this as NOW (weapon roles shipped)
- Tip table includes `10_tip_human_join_fp`
- Tip face stills current, or honest blocker documented
- PR open on `cursor/tip-stills-recapture`
- CI-ready (no Rust regressions)

## Spend and safety

- $0 local only
- No cloud apply, no paid APIs
- No tool attribution, emoji, or em/en dashes
