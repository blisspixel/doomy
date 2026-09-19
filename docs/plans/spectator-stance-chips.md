# Plan: Loud spectator stance chips without Tab

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `feat/spectator-stance-chips`
**Spend:** $0. Loopback. No Cloud Agent. No release tag.
**Status:** shipped (#119).

## Goal

`PlayerState.behavior` already reaches the client (rule-bot tactics, Agent `set_display_behavior`, Continuance Compliance). Tab / left-panel scoreboard short chips exist after #103. Spectators who never open that panel still miss stance during normal spectate.

Make stance **loud and Tab-less** on the three surfaces people actually watch:

1. Warmup TV roster chips
2. Follow-cam HUD chip
3. Fighter nameplate / billboard

Brain, NODS Agents, and scrap rule bots all read the same short codes.

## Tip context

- Tip base: `main` at or after #116 (`5c94706`).
- Soft ask parked under jammer dish non-goals: Tab-less stance chips.
- Protocol unchanged: Snapshot already echoes `behavior`.

## Root cause (verified)

| Surface | Today | Gap |
|---|---|---|
| Scoreboard | `[AGG]` / `[PSH]` via `_short_behavior` | Visible only if the left panel is watched |
| Warmup TV roster | Callsigns only (`DEAD AIR DAN // ...`) | No stance |
| Follow HUD | Stance only when a known weapon is set; early clear otherwise | Empty weapon hides FOLLOWING + stance |
| Nameplate | Rule-bot shorts only; brain names stay long; chip after HP | Easy to miss; brain not shortened |

## Ship in this PR

1. Plan + plans README index row.
2. Shared client helper `stance_chip.gd`: short codes for rule bots + brain stances + Compliance; format helpers for roster / follow / nameplate.
3. Warmup TV roster: `NAME [CODE]` from Snapshot behavior (same short map).
4. Follow HUD: always show `FOLLOWING: Name [CODE]` while tracking a pawn; weapon line is additive; ember accent when stance present.
5. Nameplate: stance chip beside the callsign (before HP); shared shorts; ember tint when stance present.
6. Headless harness + `tools/godot_check.sh` wire.

## Non-goals

- Host-per-NODS-tick juice (soft later; do not block or rewrite that path)
- Jammer dish silhouette (shipped #116; leave alone)
- Server / protocol / `set_display_behavior` changes
- Tip still recapture / release tag
- New Tab UI or scoreboard redesign
- Neon flood, paid assets

## Architecture impact

| Area | Change |
|---|---|
| `docs/plans/spectator-stance-chips.md` | This plan |
| `docs/plans/README.md` | Index row |
| `client/scripts/stance_chip.gd` | Short codes + format helpers |
| `client/scripts/hud.gd` | Roster + follow + shared short |
| `client/scripts/player_pawn.gd` | Loud nameplate chip |
| `client/scripts/game_manager.gd` | Roster entries carry behavior |
| `client/scripts/test_spectator_stance_chips.gd` | Short / format gate |
| `tools/godot_check.sh` | Run new harness |

Rust / Snapshot / coverage path: untouched.

## Verification

```bash
bash tools/godot_check.sh
# Focused: test_spectator_stance_chips: PASS
```

No `cargo` if Rust untouched.

## Success

- Plan landed before code
- One lean PR `feat/spectator-stance-chips`, squash-merged when CI green
- Warmup TV, follow HUD, and nameplates show the same short stance codes without opening Tab
- Host-per-NODS and jammer dish work remain untouched
