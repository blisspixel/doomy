# Plan: Jammer dish actually unmissable (stranger eyes)

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `feat/jammer-dish-unmissable`
**Spend:** $0. Loopback. No Cloud Agent. No release tag claiming unmissable until Testy re-Casino.
**Status:** ready for PR.
**Tip base:** main at `28ea826` (includes #120 Host-per-NODS-tick). Prior tip `5c94706` (#116) sealed scrap amazement YES; dish claim stayed OPEN.

## Goal

Make the Solo Broadcast jammer dish **actually unmissable** to stranger eyes at spectator follow (~12m) and tip overview (~36m). Harness must gate something closer to stranger eyes (on-screen footprint), not only a mesh diameter constant. Capture proof stills with the dish live and aimed in-camera.

## Tip Casino finding (#116 soft prison)

| Still | What stranger eyes saw |
|---|---|
| 20 to 22 | Empty racks / floor; no dish |
| 23 | Tiny dark tripod on a white pad; not unmissable |
| 24 | Lost again behind crates |

Wire `jammer_dish` was live at origin. Headless harness `PASS diameter=5.6` ≠ stranger eyes.

## Root cause (verified)

1. **Capture aim, not spawn absence:** tip_capture never forced a dish-aimed follow / overview still. Casino stills scraped live FP during jammer while the camera faced walls / corners. Dish at `(0, 0.35, 0)` was off-frame, so stills read empty.
2. **Lit scrap materials go black in the hangar:** StandardMaterial3D with darkened albedo and modest emission under arena ambient (~0.45) and dark background makes the bowl vanish. Stranger eyes only read the light GroundPad + dark mast / antenna as a tiny tripod.
3. **Harness gap:** size gate only checks `diameter()` / `stack_height()` constants. A constant can PASS while the in-camera read fails.

Spawn / parent path in `game_manager._sync_jammer_dish` is fine (arena child, visible true, tint applied). Not a missing Solo Broadcast client path.

## Ship in this PR

0. Map-chip honesty: HUD / playlist face uses Snapshot `map_name` (Larak Lot on Solo Broadcast). Hide static Hangar Candy brand badge that strangers read as the map name.


1. Plan + plans README index row.
2. Aggressive silhouette in `jammer_dish.gd`:
   - Much larger bowl / ring / mast mass (diameter floor well above #116).
   - Thick rim; high-contrast unshaded bone / ember / rust / gunmetal palette that punches through dark hangar lighting.
   - Billboard Label3D that always faces camera and reads **SEIZE JAMMER** while live.
   - Apply live tint inside `build()` so materials exist even before the first sync tint.
3. Headless harness asserts projected on-screen footprint at follow (~12m) and overview (~36m) for a 75deg FOV 1280x720 viewport, plus structure / label gates.
4. tip_capture: force-spawn live dish (Warmup-TV pattern), pose follow + overview aimed at origin, write proof stills under `docs/screenshots/`.
5. Commit proof PNGs; wire names into tip capture and screenshots README.

## Non-goals

- Claiming unmissable in a release tag before Testy re-Casino
- Server seize-radius / protocol changes
- Host-per-NODS-tick or stance chips
- Neon flood / milsim PBR / paid assets

## Architecture impact

| Area | Change |
|---|---|
| `docs/plans/jammer-dish-unmissable.md` | This plan |
| `docs/plans/README.md` | Index row |
| `docs/plans/jammer-dish-silhouette.md` | Note superseded for the unmissable bar |
| `client/scripts/jammer_dish.gd` | Scale, unshaded tint, SEIZE JAMMER billboard, footprint helpers |
| `client/scripts/test_jammer_dish_silhouette.gd` | On-screen footprint gates |
| `client/scripts/tip_capture.gd` | Force dish + aimed follow / overview stills |
| `docs/screenshots/*jammer*` | Proof PNGs |
| `docs/screenshots/README.md` | Index rows |

## Verification

```bash
bash tools/godot_check.sh
# test_jammer_dish_silhouette: PASS with footprint floors
# Visual: tip_capture 20 follow + 22 overview show chunky SEIZE JAMMER dish
```

No cargo if Rust untouched.

## Success

- One lean PR, squash-merge when CI green
- Proof stills committed and aimed at the dish
- Harness fails if projected footprint is tip-sized
- No unmissable tag until Testy re-Casino
