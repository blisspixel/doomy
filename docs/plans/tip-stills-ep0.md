# Tip stills after Solo Broadcast Episode 0

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `feat/tip-face-ep0-stills`
**Spend:** $0. No ElevenLabs, no GCP apply, no renet, no soft spectator chips, no gunfeel reopen.
**Status:** Shipped (#110).

## Goal

Recapture README tip stills so embeds match tip face after Solo Broadcast Episode 0 (`8a85895` / #106). Surface Solo Broadcast / `--solo-broadcast` in README What runs today. Keep README a product front door (no lore sludge). Episode chrome depth stays in VISION and plans.

## Tip priorities

Episode 0 Calibration / Larak Lot is on main. **Tip face stills + README Solo Broadcast surfacing is the NOW tip.** Soft spectator chips and gunfeel are not this PR.

## Non-goals

- Soft spectator chips
- Gunfeel / look_at / hit reopen
- Release tag
- GCP apply, ElevenLabs, new gameplay
- Relabeling mood art as tip proof

## Locks

- Contested Frequency pixel grit
- Port 6767
- Coverage fail-under 80
- Attribution lock, no emoji, no em/en dashes
- Nick Seal / blisspixel noreply authorship only

## Stills to refresh

| File | Why |
|---|---|
| `12_tip_calibration_larak_16x9.png` | New: Calibration title + objective + Larak Lot |
| `01_arena_overview_16x9.png` | Contested Frequency HUD after Episode 0 |
| `02_spectator_hud_16x9.png` | Host / scoreboard face on Solo Broadcast |
| `07_tip_compliance_pressure_16x9.png` | Mid-scrap killfeed + Contested Frequency |
| `08_tip_host_flash_midjoin_16x9.png` | Mid-join Host flash |
| `09_tip_weapons_frags_16x9.png` | Weapons / muzzle / frags |
| `10_tip_human_join_fp_16x9.png` | Join FP crosshair + weapon face |
| `11_tip_warmup_tv_bumper_16x9.png` | Warmup TV with Larak Lot face |

## Capture path

1. `cargo run -p fragr-server -- --bind 127.0.0.1:6767 --bots 4 --solo-broadcast`
2. Godot 4.7.2-stable on PATH as `godot`
3. `tools/capture_tip_screenshots.sh` (Xvfb + opengl3)
4. Driver: `client/scripts/tip_capture.gd` (Calibration still + Warmup TV + timed scrap + mid-join + Join FP)

## Docs updates

- `README.md`: What runs today surfaces Solo Broadcast; Quick start and server options mention `--solo-broadcast`; embeds add Calibration still
- `docs/screenshots/README.md`: tip table includes `12_tip_calibration_larak`; regenerate notes use `--solo-broadcast`
- `docs/plans/README.md`: this plan indexed; Episode 0 story plan marked shipped (#106)

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p fragr-server -- --bind 127.0.0.1:6767 --bots 4 --solo-broadcast &
tools/capture_tip_screenshots.sh
# Inspect PNGs: Calibration / Larak Lot, Contested Frequency, no pink frames
```

## Success criteria

- [x] Plan in tree; index row present
- [x] README surfaces Solo Broadcast / `--solo-broadcast`
- [x] Tip stills refreshed (or honest blocker documented)
- [x] PR on `feat/tip-face-ep0-stills` (#110)
- [x] Squash-merge when CI green; no release tag

## Spend and safety

- $0 local only
- No cloud apply, no paid APIs
- No tool attribution, emoji, or em/en dashes
