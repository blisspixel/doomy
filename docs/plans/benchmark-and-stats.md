# Plan: benchmark mode and deep statistics

**Status:** planned (2026-09-18)
**Branch:** `feat/bench-*` and `feat/stats-*`
**Spend:** $0. Everything here is local computation over data the server already has.

## Goal

Two audiences, one pipeline. The benchmark answers "did this change make the game slower or heavier", in numbers that gate CI. The statistics layer answers "what actually happened in that match", at a depth a person who enjoys the mathematics can dig into: distributions rather than averages, intervals rather than point estimates, and enough raw data exported that someone can do their own analysis without asking us for it.

The rule that keeps both honest: a single number is a headline, never a conclusion. Every figure that matters ships with its spread, its sample size, and, where a claim is being made, an interval.

## Non-goals

- Telemetry that leaves the machine. Everything is local files and a local endpoint; a public server may expose aggregate status only, and player names in exports are opt-in.
- A dashboard product. The export format is the deliverable; charts are whatever the reader likes.
- Replacing human judgement about fun. The numbers find regressions and imbalance; people still decide what feels good.

## Part one: benchmark mode

`fragr-server --bench N M` runs N scripted fighters on a fixed map and seed for M ticks with no network clients, then prints one JSON object and exits. The same JSON is what the status line serves while a real server runs, so a benchmark and a live server are read the same way.

Measured per tick, reported as distributions:

- **Tick time**: the wall clock the whole tick took, split into phases (input apply, movement, combat resolve, bots, pickups and rounds, interest sets, snapshot encode). Each phase reports count, mean, standard deviation, p50, p90, p99, p99.9, max, and the tick index of the max. Percentiles come from a logarithmic histogram (an HDR histogram), not a sorted vector, so a long run costs constant memory and the numbers are exact to a stated precision.
- **Budget headroom**: tick time as a fraction of the tick period, and the count of ticks over 50, 80, and 100 percent of budget. An overrun count of zero is the pass; the p99 fraction is the headline.
- **Bytes**: snapshot bytes per client per tick (mean and p99), total bytes out per second, and the same after delta encoding and interest management land, so the effect of each is a before and after in the same units.
- **Allocation and cache proxies**: allocations per tick if a counting allocator is enabled in the bench build, and the number of entities touched per phase, which is the portable stand-in for cache behaviour.
- **Determinism check**: the run is seeded, so the bench asserts that two runs with the same seed produce identical frag counts and final positions. A failure here is a correctness bug, not a performance one.

CI runs a small configuration on every pull request and fails on a pass-threshold breach; the full ladder (16, 64, 128, 256 fighters, from `massive-arenas.md`) runs locally and lands as a table in that plan.

## Part two: match statistics

The playtest harness already computes frags, gaps, spawn deaths, and weapon usage. This turns that into a real analysis layer, computed from the event stream and the snapshot stream, over one match or many.

**Combat**

- Time to kill: the distribution of elapsed time from first damage to death per victim, by weapon and by attacker type (human, rule bot, agent). Reported as a histogram with median and interquartile range, because time to kill is skewed and a mean lies.
- Damage per engagement, shots fired per kill, and accuracy by range bucket, each with a Wilson interval on the hit rate so a fighter with nine shots is not compared naively against one with nine hundred.
- Engagement distance histogram, which is the honest test of whether the three weapons occupy different ranges. The weapon triangle is working when each weapon's kill distribution peaks in its own band and the overlaps are small; the report prints each weapon's median kill distance and the overlap coefficient between pairs.
- Time to first shot and reaction time from target visible to first hit, which is also the machine and human separation the fair-play profiler uses.
- Killstreak and revenge structure: how often a death is avenged within thirty seconds, which is the "is this a fight or a farm" measure.

**Balance and skill**

- Per-weapon kill share against usage share, with a chi-square test for whether the difference is more than noise at the sample size in hand. A weapon that is picked twice as often but kills at the same rate is fine; a weapon that kills at twice the rate per engagement is not.
- Map balance: spawn to first contact time by spawn point, kill heat by cell on the same grid the sim uses, and pad control share. A spawn that dies ten percent more often than the mean is a map bug.
- Skill rating across policies and players: TrueSkill, which carries an uncertainty per rating, rather than a bare Elo number, so a new fighter's rating comes with an honest error bar. Paired-round design and sample-size arithmetic for a given effect size live in `decision-brain.md` and are reused here.
- Rating convergence: how many rounds until the interval on a rating is smaller than the gap being tested. This is what stops us claiming a policy is better after nine rounds.

**Flow and feel**

- Dead time: the fraction of a round with no damage anywhere, and the longest such gap.
- Time between deaths per fighter, and the fraction of a round spent respawning.
- Pickup contention: how often two fighters reach a pad within two seconds, which is the measure of whether the map's item timing creates fights.
- Correction magnitude and snapshot age from the client, once prediction lands, as the objective half of "does it feel smooth".

**Nerd mode surface**

- A client overlay (a settings toggle, off by default) showing frame time, tick time, snapshot age, correction error, ping, bytes per second, and the current match's running statistics, in a monospace corner panel. Everything on it comes from the same JSON as the export, so there is no second source of truth.
- An export: `--report-full` writes one JSON per match with the event stream, per-fighter series, and every distribution as a histogram, plus a CSV of the event stream for anyone who prefers a spreadsheet. Schema versioned, documented, and stable.
- A results card that stays simple for everyone else. Depth is opt-in.

## Method notes, so the numbers mean something

- Percentiles from histograms with a stated precision, never from means and standard deviations of a skewed quantity.
- Proportions with Wilson intervals; differences between proportions with a two-proportion test and the sample size stated.
- Distributions compared with a two-sample test appropriate to the shape, not by eyeballing means.
- Every reported number carries its sample size. A statistic computed from fewer than thirty observations is printed with a marker that says so.
- Seeded runs and paired designs wherever two things are being compared, so variance from spawns and item timing cancels instead of drowning the effect.
- The bench prints the seed, the build, the map, the tick rate, and the configuration in the same object, so a number can always be traced to the run that produced it.

## Verification

- Unit tests on every statistic against canned event streams with known answers, including the interval arithmetic.
- A determinism test: same seed, same numbers, twice.
- A regression test: an artificially slowed tick phase makes the benchmark fail its threshold.
- The export schema documented in `docs/protocol.md` and validated by a test.

## Rungs

1. Phase timing and the histogram plumbing; `--bench N M`; the status line JSON; CI threshold on tick time and bytes. This is playtest rung 3.
2. Combat statistics (time to kill, accuracy with intervals, engagement distance) in the playtest report.
3. Balance and map statistics; TrueSkill across policies with convergence reporting.
4. Nerd overlay in the client and the full export with its schema.
5. The scale ladder table from `massive-arenas.md` produced by the bench.

## Success criteria

- [ ] `--bench` prints phase percentiles, budget headroom, and bytes, and CI fails on a regression.
- [ ] Two seeded runs are identical.
- [ ] Every proportion in the report carries an interval and a sample size.
- [ ] The weapon triangle is demonstrated by the engagement distance histogram, not asserted.
- [ ] The nerd overlay and the full export read from the same JSON.
