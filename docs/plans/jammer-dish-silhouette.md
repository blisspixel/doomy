# Plan: Unmissable jammer dish silhouette (in-camera)

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `feat/jammer-dish-silhouette`
**Spend:** $0. Loopback. No Cloud Agent. No release tag.
**Status:** shipped (#116).

## Goal

Solo Broadcast jammer phase already exposes `jammer_dish` on Snapshot (#111). Make the Godot client draw an **unmissable** center dish silhouette so phase two reads from any normal spectator follow (~12m) and tip overview (~36m): not a tiny speck, not HUD-only.

## Tip context

- Main tip at plan start: `d9dc615` (#111 sealed; NODS progress FIXED).
- Server `jammer_dish` is honest: present in Jammer (`live`), and after seize into Auditor/Won (`seized`). Cleared otherwise.
- Soft-touch seize radius is `EP0_JAMMER_RADIUS = 3.0`. Client silhouette should match that world scale.

## Root cause (verified)

`game_manager.gd` already syncs and builds a pedestal + flattened sphere + label. Geometry is undersized for camera distance:

| Part | Current | Problem at follow / overview |
|---|---|---|
| Pedestal | r ~0.55 to 0.7, h 0.45 | Lost against scrap floor grit |
| Dish | SphereMesh r 1.1, h 0.55 | ~2.2m wide speck at 12m; invisible noise at 36m |
| Label | font 48 at y 1.35 | Reads as HUD-adjacent chrome, not a world objective |

## Ship in this PR

1. Plan + plans README index row.
2. Client-only chunky silhouette (Rock & Roll Racing saturated scrap palette: bone / gunmetal / rust / ember accents; no neon flood):
   - Ground hazard ring ~seize radius (~3.1m)
   - Tall gunmetal mast
   - Wide dish bowl (~5.5m+ diameter) with rim for crisp edge
   - Vertical antenna spike so the stack clears fighter billboards
   - Large billboard Label3D with outline
3. Extract build/tint/extent helpers into `client/scripts/jammer_dish.gd` so `game_manager.gd` stays lean and a headless harness can gate size.
4. Headless harness `test_jammer_dish_silhouette.gd` + wire into `tools/godot_check.sh`.
5. Clear / hide when Snapshot drops `jammer_dish` (phase left jammer without seize chrome). Keep seized tint into Auditor.

## Non-goals

- Host-per-NODS-tick juice (soft optional later)
- Tab-less stance chips (soft optional later)
- Docs PR #112 (park; do not block)
- Tip still recapture / release tag
- Server protocol or seize-radius changes
- Neon flood, milsim PBR, new paid assets

## Architecture impact

| Area | Change |
|---|---|
| `docs/plans/jammer-dish-silhouette.md` | This plan |
| `docs/plans/README.md` | Index row |
| `client/scripts/jammer_dish.gd` | Build + tint + extent constants |
| `client/scripts/game_manager.gd` | Call builder / tint; drop inline tiny meshes |
| `client/scripts/test_jammer_dish_silhouette.gd` | Size / structure gate |
| `tools/godot_check.sh` | Run new harness |

Rust / Snapshot / coverage path: untouched. Honest 80% fail-under stays as-is.

## Verification

```bash
bash tools/godot_check.sh
# Focused: test_jammer_dish_silhouette: PASS
# Visual: Solo Broadcast to jammer; follow cam and free overview both show a chunky center dish
```

No `cargo` required if Rust untouched. If Rust is touched: `cargo fmt`, `cargo clippy -D warnings`, `cargo test --workspace`, `cargo llvm-cov --workspace --fail-under-lines 80`.

## Success

- Plan landed before code
- One lean PR `feat/jammer-dish-silhouette`, squash-merged when CI green
- Jammer phase: unmissable in-world dish from follow and overview
- Seized chrome remains; dish clears when Snapshot omits it
