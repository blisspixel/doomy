# fragr docs/screenshots

## Current status: live tip captures + archived mood plates

`01_arena_overview_16x9.png`, `02_spectator_hud_16x9.png`, `07_tip_compliance_pressure_16x9.png`, `08_tip_host_flash_midjoin_16x9.png`, `09_tip_weapons_frags_16x9.png`, and `10_tip_human_join_fp_16x9.png` are **live tip captures** from Godot 4.7.2-stable under Xvfb + opengl3 against a loopback `fragr-server --bots 4`. They show Contested Frequency spectator HUD, Host flash on mid-join, weapons/frags combat, and Join FP scrap juice. Regenerated after weapon roles excellence (#76) so choke + FP + killstreak + role face match tip.

Archived mood / concept plates remain under `mood/` for vision direction only. Do not treat mood plates as tip gameplay proof.

## Standing rule

README screenshots MUST match the current playable UI. Stale mood stubs labeled as tip gameplay are forbidden.

## Tip captures (current)

| File | Aspect | Purpose |
|------|--------|---------|
| `01_arena_overview_16x9.png` | 16:9 | Live tip: arena + Contested Frequency HUD / scoreboard |
| `02_spectator_hud_16x9.png` | 16:9 | Live tip: Host compliance bumper + pressure chip + scoreboard |
| `07_tip_compliance_pressure_16x9.png` | 16:9 | Live tip: mid-round scrap with killfeed + Contested Frequency |
| `08_tip_host_flash_midjoin_16x9.png` | 16:9 | Live tip: mid-join Host bumper flash from first Active Snapshot |
| `09_tip_weapons_frags_16x9.png` | 16:9 | Live tip: weapons / muzzle / killfeed combat still |
| `10_tip_human_join_fp_16x9.png` | 16:9 | Live tip: Join FP scrap juice (crosshair + weapon face) |

Recaptured on tip after weapon roles (#76) via `tools/capture_tip_screenshots.sh`. `10_tip_human_join_fp` remains a first-class tip embed.

## Mood plates (archived)

| File | Aspect | Purpose |
|------|--------|---------|
| `mood/01_arena_overview_16x9.png` | 16:9 | Hub/choke arena layout concept |
| `mood/02_spectator_hud_16x9.png` | 16:9 | Spectator HUD concept |
| `mood/03_fighters_cyanex_kragge_1x1.png` | 1:1 | Fighter characters concept |
| `mood/04_muzzle_juice_16x9.png` | 16:9 | Weapon feedback concept |
| `mood/05_weapon_icons_1x1.png` | 1:1 | Weapon icon concepts |
| `mood/06_map_callouts_16x9.png` | 16:9 | Map callout concepts |

Boomer-arena energy. **No Doom IP.**

## How to regenerate tip captures

1. Launch server: `cargo run -p fragr-server -- --bots 4`
2. Put Godot 4.7.2-stable on PATH as `godot` (or `Godot_v4.7.2-stable_linux.x86_64`)
3. Run `tools/capture_tip_screenshots.sh` (Xvfb + opengl3; sets `LIBGL_ALWAYS_SOFTWARE=1` by default)
4. Inspect PNGs: Contested Frequency title, scoreboard/killfeed/Host bumpers, compliance pressure when timing hits ~15s into Active; no pink frames
5. Keep mood plates under `mood/`; do not re-label them as tip

Live captures must show what currently runs. No aspirational features. No concept art labeled as gameplay.

## Tip screenshot capture plan

See [`../plans/tip-screenshots.md`](../plans/tip-screenshots.md).

## Human join FP juice

![Human join FP](10_tip_human_join_fp_16x9.png)

Join-mode first-person scrap juice: crosshair, held-weapon face, spawn/damage flash overlays on Contested Frequency.
