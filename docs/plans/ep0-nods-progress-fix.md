# Plan: Episode 0 NODS progress + jammer dish + map honesty

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `fix/ep0-nods-progress`
**Spend:** $0. Loopback. No Cloud Agent. No release tag.
**Status:** shipped (#111).

## Goal

Make Solo Broadcast Calibration progress real in normal scrap play, make the jammer phase readable in-world, and stop lying about map face when geometry is Compliance Yard.

## Root cause (verified)

`GameState::note_nods_frag` only credited `Role::Human`. Agent meatbags (MCP / playtest) never advanced NODS 0/5, so jammer / Auditor / win never fired. Unit win-path called `note_nods_frag` directly and skipped the lethal hitscan path. `display_map_name` always returned Larak Lot while solo broadcast was on, even on map 2 Compliance Yard geometry.

`start_round()` resets `nods_cleared` on Warmup→Active once per round (expected). No extra mid-Active wipe found.

## Ship in this PR

1. Credit NODS clears when the killer is a meatbag on the scrap path: `Role::Human` **or** `Role::Agent` that is **not** a server rule bot and not the Auditor (`is_boss`). Only count when the victim is a NODS rule bot (`NODS-*` name or bot controller, not boss).
2. Prove with tests: direct `note_nods_frag` for Human and Agent meatbag; rule-bot frag does not credit; non-NODS victim does not credit; one lethal Rail hitscan path test.
3. After 5 credited clears → phase Jammer → seize → Auditor → win still works.
4. Snapshot `jammer_dish` world marker while phase is jammer (and seized chrome after). Godot renders a pedestal + dish silhouette at arena center.
5. Map face honesty: Larak Lot only when Solo Broadcast runs on Arena Duel (map 1). Compliance Yard keeps `Compliance Yard`.

## Non-goals

- Soft Tab-less stance chips
- Release tag / v0.12
- Tip still recapture
- New maps or Episode 1

## Verification

- `cargo fmt`, `cargo clippy -D warnings`, `cargo test --workspace`
- `cargo llvm-cov --workspace --fail-under-lines 80`
- Focused: `note_nods_frag_*`, `solo_broadcast_ep0_win_path`, `jammer_dish_*`, `solo_broadcast_map_name_matches_map_kind`

## Success

NODS chip ticks in Agent and Human Solo Broadcast play. Jammer phase shows a dish in the world. map_id=2 never labels Larak Lot.
