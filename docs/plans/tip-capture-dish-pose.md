# Plan: Tip capture dish pose lock (stranger re-run)

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `feat/tip-capture-dish-pose`
**Spend:** $0. Loopback. No Cloud Agent. HOLD tag (no dish-claim until Testy re-Casino).
**Status:** in flight (live gate PASS; awaiting merge).
**Tip base:** main `c5afe5d` (#129). Soft Prison on #129: amazement YES, orange gate CLEAR on empty hangar, live tip_capture dish still OPEN/missable (frames racks/soldier on stranger re-run; committed docs stills showed dish). Park #128 MapInfo. Soft louder #119 / Auditor finish not this PR.

## Goal

Force spectator / free-fly cam at dish origin during tip_capture jammer stills so orange gate PASSES on stranger re-run with chunky orange bowl + SEIZE JAMMER filling the view. Strengthen tip_pose_lock so frag-follow / soldier FP / rack chase cannot override after latch.

## Root cause

1. tip_pose_lock only early-returns `_process`. `set_fp_mode` still teleports the camera onto a soldier; under lock that yank sticks for the still.
2. `lock_on_frag` still arms frag yank while locked.
3. No held transform re-assert; pose is set once then awaited.
4. Follow / overview / label poses sit too far and leave racks / soldiers dominating the frame even when orange barely clears.
5. Hangar Join FP aims pawn yaw but does not lock spectator `fp_yaw` / `fp_pitch` at the dish.

Mesh absence is not the bug (#129 orange gate already fails empty hangar).

## Ship

1. Plan + plans README index; mark tip-capture-dish-gate shipped (#129).
2. spectator_cam: held tip pose transform re-applied every frame; set_fp_mode / lock_on_frag no-op while locked.
3. tip_capture: chunkier look-at poses at dish origin (0, ~2 to 4, 0); re-assert pose immediately before each jammer save; FP look lock sets fp_yaw / fp_pitch at dish.
4. Live `./tools/capture_tip_screenshots.sh` against `--solo-broadcast`; commit stills only if orange gate PASS.
5. One lean PR; squash-merge if green. HOLD tag.

## Non-goals

- Dish-claim release tag
- Soft louder stance chips / Auditor finish
- MapInfo (#128)
- Studio void harness path

## Architecture impact

| Area | Change |
|---|---|
| `docs/plans/tip-capture-dish-pose.md` | This plan |
| `docs/plans/README.md` | Index; gate plan shipped |
| `client/scripts/spectator_cam.gd` | Held tip pose; block FP/frag yank |
| `client/scripts/tip_capture.gd` | Chunky dish poses + re-assert + FP look |
| `docs/screenshots/*jammer*` | Recapture if live PASS |
| `docs/screenshots/README.md` | Pose note |

## Verification

```bash
bash tools/godot_check.sh
# tip_capture against --solo-broadcast: 20/22/23 orange floors PASS; dish fills frame
python3 tools/gate_tip_jammer_orange.py docs/screenshots
```

## Success

- Stranger live re-run orange gate PASS with chunky SEIZE JAMMER hangar stills
- tip_pose_lock cannot be overridden by set_fp_mode / lock_on_frag after latch
- No dish-claim tag
