# Plan: agent playtest loop

**Status:** planned (2026-09-18)
**Branch:** `feat/playtest-harness`
**Spend:** $0 for the scripted levels. An LLM-driven observer is optional and off by default.

## Goal

Local agents play the game and file structured feedback so most iteration does not need human testers. A `playtest` harness boots the server, connects N scripted agents through the adapter, runs a fixed number of rounds, and writes a report. The same harness runs in CI on a small configuration so regressions in feel show up as a diff, and locally at larger sizes to feed the scale ladder.

## Non-goals

- Replacing human judgement about fun. Agents catch stuck states, dead time, and imbalance; people still decide what is funny.
- Any model on the combat tick. The optional observer reads the event stream after the fact.

## Shape

- Crate: `tools/playtest` (`fragr-playtest`), Rust, workspace member, reuses the adapter's WebSocket client and the server's wire types (the shared protocol crate lands first if it is ready; otherwise the adapter's mirror).
- Command: `fragr-playtest --agents 8 --rounds 3 --map 1 --tiers reflex,planner --report .agents/playtest/<stamp>.json`.
- Agent tiers: `reflex` (chase nearest, fire when facing), `planner` (observe every few ticks, pick a pickup or a target, path by waypoints), later `llm` (off-tick, optional).
- Report fields: time to first frag, frags per minute per agent, deaths per minute, weapon usage spread, pickup contention, idle ticks per agent, stuck detection (no movement for N ticks while alive), spawn deaths within two seconds, longest gap without a frag, Host beats per minute, snapshot bytes per tick, tick time percentiles from the server log.
- Frustration signals become assertions with thresholds in CI: no agent stuck for more than five seconds, no spawn death rate above ten percent, at least one frag per minute at four agents.
- Output lands under gitignored `.agents/playtest/`; the summary table for a change under test goes into that change's plan doc.

## Rungs

1. Harness boots a server on a free loopback port, connects N reflex agents, runs R rounds, writes the JSON report. CI runs it with four agents and one round.
2. Planner tier with waypoints read from the map data; pickup seeking; the report gains contention and route metrics.
3. Server exposes tick time and bytes per tick on a status line so the report can read them without parsing logs (ties into the benchmark mode item).
4. Optional observer: an off-tick process that reads the event stream and writes free-text notes, gated behind a flag and a key.

## Verification

- Unit tests for metric computation on canned event streams.
- The CI run must finish in under two minutes and pass its thresholds.
- A recorded local run at sixteen agents committed as a table in this plan.

## Success criteria

- [ ] Rung 1 in CI.
- [ ] Planner tier with route metrics.
- [ ] Status line metrics from the server.
- [ ] Thresholds catch a deliberately introduced stuck bot in a test.
