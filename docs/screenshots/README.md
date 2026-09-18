# fragr docs/screenshots

## Current status: mood plates (not tip captures yet)

Files in this directory are still **mood / concept plates**, not live Godot tip captures. They show intended visual direction. Do not treat them as proof of the current playable HUD or arena.

Live tip captures are the goal (see [`../plans/tip-screenshots.md`](../plans/tip-screenshots.md) and `tools/capture_tip_screenshots.sh`). Until those land, README must say these are mood plates, not "live gameplay."

## Standing rule

README screenshots MUST match the current playable UI once live captures exist. Stale mood stubs that misrepresent the build as tip gameplay are forbidden. Carve-outs that hide missing tip shots behind vague wording are also forbidden.

## Current mood plates

| File | Aspect | Purpose |
|------|--------|---------|
| `01_arena_overview_16x9.png` | 16:9 | Hub/choke arena layout concept |
| `02_spectator_hud_16x9.png` | 16:9 | Spectator HUD concept |
| `03_fighters_cyanex_kragge_1x1.png` | 1:1 | Fighter characters concept (Cyanex courier + Kragge scrap-mech) |
| `04_muzzle_juice_16x9.png` | 16:9 | Weapon feedback/muzzle juice concept |
| `05_weapon_icons_1x1.png` | 1:1 | Weapon icon concepts |
| `06_map_callouts_16x9.png` | 16:9 | Map callout concepts |

Boomer-arena energy. **No Doom IP.**

## How tip captures replace these

1. Launch server: `cargo run -p fragr-server -- --bots 4`
2. Run `tools/capture_tip_screenshots.sh` (Xvfb + Godot 4.7.2-stable + opengl3) or capture manually in the editor
3. Wait for bots to engage, HUD/killfeed populated, no pink shader frames
4. Replace mood plate files with live captures (same filenames) when stills are honest
5. Update this README to drop the mood-only status and say "live tip captures"
6. Update main README from "Mood art" to "Live gameplay"

Live captures must show what currently runs. No aspirational features. No concept art labeled as gameplay.

## Tip screenshot capture

For automated capture of genuine tip-of-tree gameplay screenshots, see [`../plans/tip-screenshots.md`](../plans/tip-screenshots.md).
