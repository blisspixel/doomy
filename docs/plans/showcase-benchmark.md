# Plan: a benchmark that pushes the machine and shows off the game

**Status:** spec (2026-09-19)
**Branch:** `feat/showcase-bench-*` (one PR per rung)
**Spend:** $0. Local only, no service, no telemetry.

## What exists, stated plainly

The benchmark that ships today is a headless, server-side simulation timer. It builds a session, seeds it, runs rule bots for a set number of ticks, and records two things per tick: how long the tick took and how many bytes the broadcast would have been. It never opens a window, never loads a scene, never presents a frame.

It is good at its job. Log-linear histogram, exact count and mean, thresholds that fail a build on a single overrun, a determinism check that runs the whole thing twice, and the same statistics structure running inside the live server so a benchmark and a running server are read the same way. It should not change.

But it measures the part of fragr that has already been proven not to be the bottleneck. The scale ladder found tick time at the 256-fighter rung sitting at 2.884 ms against a 50 ms budget, 5.77 percent of it, with zero overruns in twenty-four thousand ticks, and concluded that the wall is bandwidth rather than simulation. So as a regression gate it is exactly right, and as a thing that pushes a machine it is the wrong instrument entirely. No amount of extending it server-side makes it one.

The client side is not thin, it is absent. Nothing in the client times a frame.

## The prerequisite nobody names

A benchmark that renders a different world on every run is not a benchmark.

The client renders whatever a live server sends over a socket, so scheduling jitter, connection timing and roster order mean two runs show different matches. Before any camera path or overlay matters, **the content has to come from a file**. The server already serialises exactly the right bytes every tick and then throws them away; writing them to a trace file is a small change to code that exists, and it is what makes everything else possible.

Two more determinism requirements sit alongside it:

**Everything advances on wall time, never per frame.** The camera and the trace cursor are both functions of scene time in seconds. If either steps per frame, a fast machine walks a different path than a slow one and there is nothing to compare.

**A real bug has to be fixed first.** The pawn smooths its position with a lerp scaled by delta, which is frame-rate dependent: a machine at 240 frames a second and one at 40 converge toward the same target at genuinely different rates and therefore render different positions from identical inputs. It needs exponential smoothing instead. That is a correctness fix for the shipped game, not just for the benchmark.

## The flow

Nine scenes, two hundred seconds of scored time, escalating monotonically so a machine that falls over does so late and visibly, and the per-scene split says why.

| Scene | Time | Stresses |
|---|---|---|
| 0. Prime, discarded | 10 s | Nothing. It compiles every pipeline so a shader hitch is not recorded as stutter |
| 1. Cold open, empty arena | 20 s | Geometry, one shadowed light, fog, tonemap. The floor |
| 2. The scrap, 16 fighters | 25 s | Draw calls, nameplates, HUD, audio. The number a player actually cares about |
| 3. Muzzle storm, 32 firing | 20 s | Clustered dynamic lights and alpha overdraw |
| 4. Full server, 64 fighters | 25 s | Node count and per-node script cost. Where CPU-bound separates from GPU-bound |
| 5. Overdraw pit | 20 s | Fill rate, from a far pose where every billboard scales up and overlaps, then a push to point blank |
| 6. Compliance pressure | 20 s | The 2D layer, runtime node construction, and the only scene with a story in it |
| 7. Massive arena, 256 | 30 s | Everything at once, beyond anything retail does. The shot people screenshot |
| 8. Resolution ladder | 20 s | Scene 2 replayed at three render scales |
| 9. Score screen | static | |

Scene 8 is the most diagnostic thing in the flow and is not optional. If frame time is flat across the ladder the machine is CPU or draw-call bound; if it scales with pixel count it is GPU bound. That is what turns a score into an explanation.

**What this renderer actually is**, so the flow stresses the real thing rather than borrowing from benchmarks for other games: about thirty meshes, one shadowed directional light, four static point lights, fighters as alpha-scissor billboards with a point light on each muzzle, a canvas layer for the HUD, and no particle systems anywhere. So the honest stressors are draw calls, dynamic lights in the cluster, alpha overdraw, canvas throughput, render scale, the shadow atlas, and per-node script cost. Tessellation, ray tracing, probe-based global illumination and destruction are not realistic here, and adding a particle system purely to make the benchmark look hard would measure something the game does not ship. When muzzle sparks and impact debris become real content, the benchmark gains a particle scene then and not before.

## The statistics

**The rule everyone gets wrong, first: compute in frame time and convert to frames per second only at print.** The mean of per-frame rates is not the reciprocal of the mean frame time, and it flatters fast frames. Average is frame count over elapsed time, and nothing else.

Per scene and overall: count, min, the first and fifth percentiles, median, mean, ninety-fifth, ninety-ninth, ninety-nine-point-nine, max, standard deviation, and median absolute deviation, which a single hitch does not move.

**The one percent and point one percent lows, all three conventions, each labelled.** Three definitions circulate and people quietly compare across them: the plain percentile, the mean of the worst one percent of frames by count, and the mean of the worst frames by accumulated time. Publish all three and name each. The composite uses the time-weighted one, because it is the one that penalises what actually ruins a session.

All three need the raw frame-time array rather than a histogram. Twelve thousand frames is forty-eight kilobytes. Keep the array.

**Stutter, which is where the nerd analysis is earned.** Consecutive-frame delta at median, ninety-ninth and max, which separates slow-but-smooth from fast-but-juddery. A windowed stutter count, where a frame counts when it exceeds twice the median of the trailing second, so a scene transition is not counted but an in-scene hitch is. Total time spent in frames over 33 and over 50 milliseconds, as a share of the run, which is how much of it felt bad. And frames over budget against a declared target, mirroring the server's vocabulary so both reports read alike.

The smoothness headline is the ratio of the ninety-ninth percentile to the median, reported directly. It needs no clamping and no explanation, unlike an invented index.

**The composite score** is a weighted geometric mean of per-scene median frame rates, multiplied by a penalty derived from the ratio of median to ninety-ninth percentile, calibrated so a named reference machine scores exactly a thousand. Geometric because it is unit-invariant and stops one enormous scene dominating. Median per scene because it does not chase the tail, which the penalty handles separately. The raw score, the penalty and the final score all print separately, alongside every per-scene figure and every weight, so a reader can watch a machine averaging a higher rate lose to one that holds a lower rate flat.

**Confidence intervals, honestly.** Frame times are strongly autocorrelated, so a naive interval on twelve thousand frames is far too narrow and would be rigour-shaped nonsense. Proportions use the Wilson interval that already exists in the playtest harness, reused rather than reimplemented. Within a run, batch means: split each scene into ten equal-time blocks and put an interval on those ten block means, because blocks are long enough to be near-independent. Across runs, run the flow three times and report the median with the range. And never put an interval on a maximum, which the output should say out loud.

## Comparability

A result is comparable only if it carries its environment: engine version and rendering driver, adapter name, vendor and API version, processor and core count, operating system, memory, peak video memory, display resolution and refresh, whether the machine was on battery, the preset, and hashes of the flow manifest and the trace.

A result missing any field is flagged unverified and the comparison tool refuses it. That is mechanical rather than a note in a document.

Four presets, and only same-preset results compare. Pinned per preset: resolution, render scale, rendering method, anti-aliasing, shadow atlas, fog, texture filter, frame cap off, vertical sync off, and window mode, because exclusive fullscreen and windowed do not compare.

The prime scene is discarded, and a declared number of frames after each transition is discarded too, so pipeline compilation is never recorded as stutter. The number goes in the result.

## Zero cost, and one honest limit

Nothing here touches a paid service. The engine is free and already pinned, the trace is a local file, results are local JSON and CSV, the published table is a committed markdown file whose history is the git log, and comparison is a subcommand of the server binary. No telemetry leaves the machine, which the stats plan already forbids.

The limit worth stating before anyone tries: **the scored flow cannot run in CI.** The runner has no GPU and the existing capture path already falls back to software rendering, which would take hours for two hundred seconds of sixty-four-fighter rendering. CI gates the pure-function statistics and the determinism checks. The score is a local artifact published as a table. Nobody should gate a pull request on a frame rate.

## Rungs

1. **Trace export.** The server writes the per-tick broadcast it already serialises to a file, with a header carrying the config and a content hash. While in there, count the unicast bytes instead of discarding them, and make the determinism check compare the trace hash rather than only the frag list, which finally delivers what the stats plan promised.
2. **A frame-time meter as pure functions**, with a headless harness asserting exact answers on a canned array. No rendering, so it runs in the existing CI job.
3. **Trace playback, and the smoothing fix.** The client feeds itself from the trace file instead of a socket. Accepted when a headless replay ends with a scoreboard matching the server's own frags for that seed, which means client and server agree on the match with no socket between them.
4. **Camera path and flow manifest.** Accepted when the camera transform at three points is identical across two runs driven at deliberately different simulated frame rates.
5. **The run itself:** overlay, per-scene banner, score screen, and export. Every number on screen read from the same structure the file is serialised from.
6. **Determinism, presets, environment stamping,** including the refusal to compare unverified results. Accepted when three consecutive runs show a composite coefficient of variation under two percent.
7. **Score, intervals, and the comparison subcommand.** Accepted when comparing a run against itself reports zero delta and an interval containing zero.
8. **The escalation scenes and the published table,** with at least two machines in it.
9. **Trailer capture,** optional and never scored.

Three small corrections to fold in along the way: the threshold check is called twice where once would do, the documented command line disagrees with the actual flags in two places, and the snapshot byte figure silently becomes wrong the moment interest management lands, because it counts broadcast only.

## Related

- `benchmark-and-stats.md`: the statistics this implements, and the overlay it describes.
- `massive-arenas.md`: the measured table proving the simulation is not the wall.
- `visual-qa-tour.md`: the tour already measures frame time per state and is the natural place the flow manifest borrows from.
- `look-pass-boomer.md`: the renderer switch that changes every number here, so the first published table should predate it and the second should follow it.
