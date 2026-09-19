# Plan: Soft first Calibration jammer seize stall

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `fix/first-calib-seize-stall`
**Spend:** $0. Loopback. No Cloud Agent. No release tag. No dish chrome.
**Status:** shipped (#131).

## Goal

Solo Broadcast Episode 0 jammer seize must not soft lock the meatbag on first Calibration. Standing on the visible dish pad must seize reliably and read as the phase transition.

## Root cause (verified)

Playtest tip `679b774` (Casino): calib1 stalled 220s / 440 acts on `SEIZE JAMMER`; calib2 cleared in ~39s. Dump `calib_phase_jammer_218.json` places CalibFox at `(0.13, 4.62)` (~4.6m from origin) while `EP0_JAMMER_RADIUS = 3.0`. Client ground ring is `RING_RADIUS = 6.2`. Meatbag stood on the pad silhouette and never entered the soft touch cylinder. Wire seize itself HOLDs when the body is inside the radius.

## Ship in this PR

1. Raise `EP0_JAMMER_RADIUS` to `6.5` so soft touch matches the visible pad (ring 6.2 plus player radius slack). Keep dish at origin; no client silhouette or tip_capture chrome.
2. Regression tests: meatbag at playtest pad stance (~4.6m) seizes; meatbag at north health (~8.0m) does not; existing win path and snapshot seize still pass.
3. Host jammer line names the pad step so the beat reads.
4. Sync the jammer_dish.gd comment that claimed server radius stays 3.0.
5. Short plan + README index row.

## Non-goals

- Louder #119 stance chips (separate non-blocking PR / after sticky TTK)
- Tip-face dish pose, orange gate, tip stills
- Auditor AI / TTK harness
- Release tag

## Verification

- `cargo fmt`, `cargo clippy -D warnings`, focused `try_seize` / `jammer` / `solo_broadcast_ep0` / `note_nods` tests
- No Godot tip_capture re-run required (no dish chrome)

## Success

1. Meatbag on the visible pad seizes without hunting a 3m hub.
2. Outside the pad still requires a deliberate step in.
3. First Calibration jammer phase is readable: Host line + SEIZE JAMMER chip + soft touch that matches eyes.
