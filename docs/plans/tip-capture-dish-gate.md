# Plan: Tip capture hangar dish gate (live re-run)

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `feat/tip-capture-dish-gate`
**Spend:** $0. Loopback. No Cloud Agent. No dish-claim tag until Testy re-Casino.
**Status:** shipped (#129).
**Tip base:** main `3f86a27` (#127 README embeds on #125 stills). Soft Prison on #125 / `cc4eff9`: amazement YES, Hangar Candy CLEAR, live tip_capture dish still OPEN/missable (orange≈0.01 on live re-run; committed docs stills showed dish).

## Goal

Make `tools/capture_tip_screenshots.sh` + `client/scripts/tip_capture.gd` reliably produce hangar dish stills on a live re-run that match committed quality: jammer-live bowl + SEIZE JAMMER under dark hangar and killfeed, with CI failing if orange footprint goes empty again.

## Root cause (verified)

1. **tip_force latch only covered Snapshot null.** After NODS clear, bots seize the dish; Snapshot sends `{live:false, seized:true}` (green JAMMER OK). That is not null, so #125 latch did not re-apply ember live tint. Soft Prison orange filter then read ≈0.01 (pickups only). Committed stills won the race when the last `_force_live_jammer_dish` landed after the seized Snapshot.
2. **Spectator frag-follow can yank aim.** `SpectatorCamera.lock_on_frag` ignores `follow_mode=false` for up to 1.5s, so a late FRAG during the pose wait looks at fighters / racks instead of origin.
3. **No orange / on-screen gate on live capture.** Diameter and studio footprint harnesses PASS while tip stills can still be empty racks. Capture saved whatever the viewport showed.

Studio void `capture_jammer_dish_proof` path stays harness-only. Tip face is live tip_capture against `--solo-broadcast`.

## Ship

1. Plan + plans README index.
2. `game_manager._sync_jammer_dish`: while `tip_force_jammer_dish` is latched, always keep forced live dish at origin (override null and seized / non-live Snapshots).
3. `spectator_cam.tip_pose_lock`: tip_capture freezes follow / frag-follow / free-fly while posing at dish origin.
4. `tip_capture`: latch early for jammer proof; pose lock + aim at origin; assert ember orange pixel ratio / footprint before saving 20 / 22 / 23 (fail hard if empty).
5. `tools/capture_tip_screenshots.sh`: post-run orange gate on jammer stills so CI / operators cannot commit empty hangar frames.
6. Recapture live tip stills only when the gate passes; update screenshots README. No dish-claim tag.

## Non-goals

- Dish-claim release tag
- TTK proof / louder stance chips
- Server phase skip cheats (client tip force is the Warmup-TV pattern)
- Studio void path changes

## Architecture impact

| Area | Change |
|---|---|
| `docs/plans/tip-capture-dish-gate.md` | This plan |
| `docs/plans/README.md` | Index row; note #125 gap |
| `client/scripts/game_manager.gd` | tip_force always forces live |
| `client/scripts/spectator_cam.gd` | tip_pose_lock |
| `client/scripts/tip_capture.gd` | Lock + orange assert on jammer stills |
| `tools/capture_tip_screenshots.sh` | Post-run orange gate |
| `docs/screenshots/*jammer*` | Recapture if live gate passes |
| `docs/screenshots/README.md` | Gate note |

## Verification

```bash
bash tools/godot_check.sh
# tip_capture against --solo-broadcast: 20/22/23 orange floors pass; SEIZE JAMMER in hangar
```

## Success

- One lean PR on `3f86a27`, squash-merge when CI green
- Live re-run cannot silently save empty hangar dish stills
- No dish-claim tag until Testy re-Casino
