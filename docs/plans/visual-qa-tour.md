# Plan: visual QA tour and feel probes

**Status:** planned (2026-09-18)
**Branch:** `feat/qa-tour`
**Spend:** $0.

## Goal

Polish needs eyes on every state, every time. A tour script drives the client through every player-facing state and every map, captures a dated set of stills and short metrics, and hands them to the agent developer for critique. The critique becomes concrete plan items. The same run feeds `docs/screenshots/` when the tip changes. Feel is covered by probes that print numbers (same-frame aim, time to first shot, acceleration curve), not by opinion alone.

## Non-goals

- Replacing human play. The tour catches wrong, ugly, overlapping, missing, and slow; a person still says whether it is fun.
- A pixel-diff gate in CI. Stills are for critique and the plan docs, not a golden-image test that flakes on a font.

## The tour

One manifest, `client/qa/tour.json`, lists states in order. Each state names the setup (mode, map, script), the trigger (a key, a wire message, a timer), the still to save, and the metrics to record. The client already captures timed stills through `tip_capture.gd`; the tour generalises it into a state machine driven by the manifest so a new state is a JSON entry, not a new function.

States, in the order a player meets them:

1. Boot menu, idle and with each option focused (keyboard and gamepad glyphs).
2. Settings menu (planned; captured once it exists), each tab.
3. Spectator warmup: the Warmup TV bumper, the roster, the countdown.
4. Human join: first-person on spawn, crosshair, weapon face at rest, HUD with no pickup in view.
5. HUD with a pickup in view and on the pad (weapon, health, armour prompts).
6. Radio: station card on switch for all eight stations, the track toast, ducking under a Host line.
7. Shooting: each weapon at rest, on fire (muzzle flash frame), on hit confirm, on frag stinger, with the killfeed line.
8. Movement: strafe, back-pedal, spawn shield, and a death and respawn cycle, as a short frame strip.
9. Round end: scoreboard, MVP podium, the Ended linger, the next-round bumper.
10. Boss beat and compliance ping.
11. Each map from the director camera at three fixed poses.
12. Benchmark overlay when it exists, and the status line numbers.
13. Agents on the scoreboard: a brain fighter's stance chip, a rule bot's behaviour chip, an MCP agent's name.

Output lands under gitignored `.agents/qa/<stamp>/`: one PNG per state, `manifest.json` with each still's state name, resolution, frame time at capture, and the metrics below, and `contact.png`, a contact sheet for a one-glance pass. The tip-screenshot path stays: `tools/capture_tip_screenshots.sh` becomes a thin wrapper that runs the tour with the tip subset and copies the approved stills into `docs/screenshots/`.

## Feel probes

Headless, repeatable, printed as numbers alongside the stills:

- **Same-frame aim:** inject a mouse motion event, assert the camera yaw changed on that frame, report the delta in degrees per count.
- **Time to first shot:** from join to the first fire action accepted by the server, in milliseconds.
- **Acceleration curve:** hold forward for one second, sample speed every tick, report the time to 90 percent of top speed and any overshoot.
- **Stop distance:** release forward at top speed, report the distance to rest.
- **Snapshot age:** for sixty seconds, the age of the snapshot being rendered, p50 and p99.
- **Frame time:** p50, p99, and max during the tour, at 1280 by 720 with the Compatibility renderer once the look pass lands.

These start as a report. Thresholds move into CI as `plans/buttery-controls.md` lands its stages, the same way the playtest thresholds did.

## The critique loop

1. Run `tools/qa_tour.sh` (Windows: `tools/qa_tour.ps1` is not added; Git Bash runs the same script) against a local server with bots.
2. The agent developer reads every still and the manifest, writes `.agents/qa/<stamp>/critique.md` with one line per finding (state, what is wrong, severity, the plan it belongs to), and promotes findings into the relevant plan as checklist items in the same session. Standing critique criteria: text density (sprites and icons over words, at most one line of HUD text outside menus and the killfeed), overlap, contrast against every map, readability at 480 by 270, and whether the weapon in hand is identifiable by silhouette.
3. A finding is closed by a later tour whose still shows the fix; the plan links the stamp.

## Verification

- The tour runs on Windows and Linux with the pinned Godot; a missing state fails loudly rather than skipping.
- Every still is non-empty and the manifest lists every state in the manifest order.
- The feel probes print all six numbers on a local run.
- The first critique is filed and at least three findings are promoted into plans.

## Rungs

1. Manifest and tour driver replacing the hand-written capture functions; boot, spectator, human join, HUD, radio, round end states; contact sheet.
2. Shooting and movement frame strips; map poses; agent chips.
3. Feel probes as a report.
4. Settings menu and benchmark overlay states once those exist.

## Success criteria

- [ ] Rung 1 runs locally and the first critique is filed.
- [ ] Rung 2 covers every weapon and both maps.
- [ ] Feel probes print six numbers.
- [ ] Tip screenshots are produced by the tour.
