# fragr docs/screenshots

## Current Status: MOOD ART ONLY

**All screenshots in this directory are MOOD ART, not live client captures.** These are concept images showing the intended visual direction and feature set. They do not represent the current playable state of the game.

## Standing Rule

README screenshots MUST match the current playable UI. When the Godot client has weapons, sprites, HUD elements, or arena features that differ from these mood plates, regenerate captures to show what actually runs. Stale mood stubs that misrepresent the build are forbidden.

## Current Mood Plates

| File | Aspect | Purpose |
|------|--------|---------|
| `01_arena_overview_16x9.png` | 16:9 | Hub/choke arena layout concept |
| `02_spectator_hud_16x9.png` | 16:9 | Spectator HUD concept |
| `03_fighters_cyanex_kragge_1x1.png` | 1:1 | Fighter characters concept (Cyanex courier + Kragge scrap-mech) |
| `04_muzzle_juice_16x9.png` | 16:9 | Weapon feedback/muzzle juice concept |
| `05_weapon_icons_1x1.png` | 1:1 | Weapon icon concepts |
| `06_map_callouts_16x9.png` | 16:9 | Map callout concepts |

Boomer-arena energy. **No Doom IP.** 

## Live Capture Instructions (when ready)

When the Godot client is playable and ready for honest screenshots:

1. Launch server: `cargo run -p fragr-server -- --bots 4`
2. Open `client/` in Godot 4.7.2-stable editor
3. Run scene (F5) in spectator mode
4. Wait for bots to engage (30s)
5. Capture screenshots showing actual gameplay
6. Replace mood art files with live captures (same filenames)
7. Update this README to remove "MOOD ART ONLY" warning
8. Update main README note from "Mood art" to "Live gameplay"

Live captures must show what currently runs. No aspirational features. No concept art labeled as gameplay.
