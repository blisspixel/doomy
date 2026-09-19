# Plan: agent playtest loop

**Status:** in flight (rung 1 shipped in #95, 2026-09-18)
**Branch:** `feat/playtest-harness`
**Spend:** $0 for the scripted levels. An LLM-driven observer is optional and off by default.

## Goal

Local agents play the game and file structured feedback so most iteration does not need human testers. A `playtest` harness boots the server, connects N scripted agents through the adapter, runs a fixed number of rounds, and writes a report. The same harness runs in CI on a small configuration so regressions in feel show up as a diff, and locally at larger sizes to feed the scale ladder.

## Non-goals

- Replacing human judgement about fun. Agents catch stuck states, dead time, and imbalance; people still decide what is funny.
- Any model on the combat tick. The optional observer reads the event stream after the fact.

## Shape

- Crate: `tools/playtest` (`fragr-playtest`), Rust, workspace member, takes wire types straight from `fragr-server` and boots the server in-process through `run_server`.
- Command today: `fragr-playtest --agents 4 --rounds 1 --map 1 --frag-limit 3 --time-limit-seconds 45 --assert --report .agents/playtest/ci.json`. Planned: `--tiers reflex,planner,brain`.
- Agent policies (all the same agent on the wire): `reflex` (chase nearest, fire when facing; shipped), `planner` (observe every few ticks, pick a pickup or a target, path by waypoints), and `brain` (the decision-brain client under a cap, from `plans/decision-brain.md`).
## First combat numbers (2026-09-19)

Six reflex agents, one round, 76.5 seconds, Arena Duel. The combat report earns its place immediately, because three of these were invisible before it:

| Measure | Value |
|---|---|
| Shots, hits | 605, 203 |
| Accuracy | 33.6 percent (interval 29.9 to 37.4) |
| Shots per kill | 11.6 |
| Time to kill | p50 1.50 s, p90 6.00 s, max 15.15 s, n=52 |
| Kill distance | p50 3.0 units; buckets of five units: 30, 9, 11, 2, then nothing |

Three findings:

1. **Time to kill is far outside the target.** `plans/gunfeel.md` asks for 0.6 to 1.2 seconds bare; the median is 1.5 and the ninetieth percentile is six. That is the Quake-slow pacing the research flagged, now measured rather than argued.
2. **Everything happens at knife range.** Three quarters of kills land inside ten units, in an arena fifty across, with a weapon that reaches forty two. Either the reflex policy has no reason to hold range or the map funnels fighters together. The planner tier and the look pass both have a stake in this.
3. **The weapon triangle is untested, because only one weapon was used.** The reflex agents never swap, so scatter and rail produced no data at all. The harness cannot measure a triangle its agents refuse to use: the planner tier must seek and use pickups, or the triangle stays an assertion.

- Report fields shipped: time to first frag, frags per minute per agent, deaths, longest gap without a frag, Host beats, spawn deaths within two seconds, stuck detection (no movement and no fire during an Active round), weapon usage, bytes per snapshot, and the combat block: time to kill as a distribution, shots and hits with a Wilson interval on accuracy, shots per kill, per-weapon accuracy, damage, kills, hit distance and kill distance, and a kill-distance histogram. Planned: pickup contention, idle ticks per agent, route metrics, tick time percentiles from the status line.
- Frustration signals become assertions with thresholds in CI: no agent stuck for more than five seconds, no spawn death rate above ten percent, at least one frag per minute at four agents.
- Output lands under gitignored `.agents/playtest/`; the summary table for a change under test goes into that change's plan doc, except brain results, which stay in `.agents/` per TypeSafe's terms.

## Rungs

1. Harness boots a server on a free loopback port, connects N reflex agents, runs R rounds, writes the JSON report. CI runs it with four agents and one round.
2. Planner tier with waypoints read from the map data; pickup seeking; the report gains contention and route metrics.
3. Server exposes tick time percentiles and bytes per tick on a status line, and `--bench N M` runs N scripted bots for M ticks and prints the same JSON. This rung owns the roadmap's benchmark mode; the hardening plan later serves the same JSON over HTTP, and the buttery-controls plan reads its correction metrics from this line.
4. Optional observer: an off-tick process that reads the event stream and writes free-text notes, gated behind a flag and a key.

## Verification

- Unit tests for metric computation on canned event streams.
- The CI run must finish in under two minutes and pass its thresholds.
- A recorded local run at sixteen agents committed as a table in this plan.

## Success criteria

- [x] Rung 1 in CI (#95).
- [ ] Planner tier with route metrics.
- [ ] Status line metrics from the server.
- [ ] Thresholds catch a deliberately introduced stuck bot in a test.
