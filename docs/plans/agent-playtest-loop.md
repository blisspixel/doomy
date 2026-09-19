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
3. **The weapon triangle was untested, because only one weapon was used.** The reflex agents never swapped, so scatter and rail produced no data at all. Fixed the same day: the reflex policy now holds the shotgun inside ten units, the railgun beyond thirty, and the needle gun between. The follow-up runs are in `plans/gunfeel.md`, and they found that the rail is still never fired in any seed, because chasing agents close the distance before a fight starts.

- Report fields shipped: time to first frag, frags per minute per agent, deaths, longest gap without a frag, Host beats, spawn deaths within two seconds, stuck detection (no movement and no fire during an Active round), weapon usage, bytes per snapshot, and the combat block: time to kill as a distribution, shots and hits with a Wilson interval on accuracy, shots per kill, per-weapon accuracy, damage, kills, hit distance and kill distance, and a kill-distance histogram. Planned: pickup contention, idle ticks per agent, route metrics, tick time percentiles from the status line.
- Frustration signals become assertions with thresholds in CI: no agent stuck for more than five seconds, no spawn death rate above ten percent, at least one frag per minute at four agents. Sticky flechette/rail/scatter table TTK (0.5 to 1.2 s clean-hit) is asserted on the same `--assert` path; see [`ttk-feel-harness-proof.md`](./ttk-feel-harness-proof.md).
- Output lands under gitignored `.agents/playtest/`; the summary table for a change under test goes into that change's plan doc, except brain results, which stay in `.agents/` per TypeSafe's terms.

## The planner tier, measured (2026-09-19)

Six agents, one round, two seeds, with the weapon table from `plans/gunfeel.md`. Kill distance is the count of kills in each five unit bucket from zero.

| Tier | Accuracy | Shots per kill | Time to kill p50 / p90 | Rail shots | Kill distance buckets |
|---|---|---|---|---|---|
| reflex, seed 1 | 16.5% | 22.0 | 1.30 / 5.45 s | 17 | 16, 11, 8, 3, 0 |
| reflex, seed 42 | 15.3% | 22.6 | 0.90 / 7.60 s | 19 | 19, 7, 16, 3, 0 |
| planner, seed 1 | 16.8% | 23.1 | 2.05 / 6.25 s | 99 | 0, 5, 36, 5, 3 |
| planner, seed 42 | 14.9% | 24.7 | 1.60 / 7.65 s | 74 | 1, 3, 43, 6, 1 |
| mixed, seed 1 | 19.2% | 19.2 | 2.20 / 6.65 s | 44 | 4, 24, 15, 5, 0 |
| mixed, seed 42 | 15.3% | 25.7 | 1.20 / 6.40 s | 37 | 2, 15, 24, 2, 0 |

What it says:

- **The knife-range problem is gone.** A reflex roster puts fifteen to nineteen kills in the closest bucket; a planner roster puts nought or one there and clusters at ten to fifteen units. Holding the range a weapon wants is the whole difference.
- **The rail finally matters.** Four to five times the shots and up to seven kills in a run, against one or none for reflex agents.
- **Accuracy and shots per kill are unchanged**, which is the result that makes the first two trustworthy: the planner is not winning by shooting more or better, it is fighting somewhere else.
- **Fighting at range takes longer**, median time to kill rising from about one second to about two. That is not obviously wrong. A rail duel across a room should take longer than a shotgun in a doorway; the question the gunfeel plan now has to answer is whether the long tail is pacing or frustration.
- **A mixed roster produces the most varied distances** and the best accuracy and shots per kill of the three. It is the better default for measuring anything, because it exercises the whole triangle rather than one corner of it.

## Rungs

1. Harness boots a server on a free loopback port, connects N reflex agents, runs R rounds, writes the JSON report. CI runs it with four agents and one round.
2. **Shipped.** Planner tier: hold the range the held weapon wants in three zones (close in, strafe, back off), break off for health below 45, and collect a weapon not in hand when nobody is pressing. `--tiers reflex,planner` deals policies round robin and names agents after theirs. Waypoints from map data and the contention and route metrics remain.
3. Server exposes tick time percentiles and bytes per tick on a status line, and `--bench N M` runs N scripted bots for M ticks and prints the same JSON. This rung is rung 1 of `plans/benchmark-and-stats.md`, which specifies the histograms, the phase split, the budget headroom, and the determinism check; the hardening plan later serves the same JSON over HTTP, and the buttery-controls plan reads its correction metrics from this line. The deeper match statistics build on the same report.
4. Optional observer: an off-tick process that reads the event stream and writes free-text notes, gated behind a flag and a key.

## Verification

- Unit tests for metric computation on canned event streams.
- The CI run must finish in under two minutes and pass its thresholds.
- A recorded local run at sixteen agents committed as a table in this plan.

## Success criteria

- [x] Rung 1 in CI (#95).
- [x] Planner tier (route metrics still to come).
- [x] Status line metrics from the server (`--bench`, `--status-every-s`, the same JSON in both).
- [ ] Thresholds catch a deliberately introduced stuck bot in a test.
