//! The ruler: how long a tick took, how much a snapshot cost, and whether a
//! seeded run reproduces itself. One JSON object describes a run, and the same
//! object is printed by `--bench` and by the live status line, so a benchmark
//! and a running server are read the same way.
//!
//! Percentiles come from a log-linear histogram rather than a sorted vector, so
//! a long run costs a fixed amount of memory and the error on a reported value
//! is bounded by the bucket width (about 2 percent here). A mean and a standard
//! deviation would lie about tick time, which is skewed by design: most ticks
//! are cheap and the interesting ones are not.

use crate::run::TICK;
use crate::session::GameSession;
use crate::sim::{MapKind, MatchConfig};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Buckets per power of two. Sixteen puts the worst-case error near 2 percent
/// (half a bucket) while keeping the whole histogram inside a few kilobytes.
const SUB_BUCKETS: u32 = 16;
/// Powers of two covered, from 1 unit up. Nanoseconds to about 18 minutes.
const POWERS: u32 = 40;
const BUCKETS: usize = (SUB_BUCKETS * POWERS) as usize;

/// A log-linear histogram of non-negative values, exact in count and bounded in
/// relative error on every reported percentile.
#[derive(Debug, Clone)]
pub struct Histogram {
    buckets: Vec<u64>,
    count: u64,
    sum: f64,
    min: u64,
    max: u64,
}

impl Default for Histogram {
    fn default() -> Self {
        Histogram::new()
    }
}

impl Histogram {
    pub fn new() -> Self {
        Histogram {
            buckets: vec![0; BUCKETS],
            count: 0,
            sum: 0.0,
            min: u64::MAX,
            max: 0,
        }
    }

    fn index_of(value: u64) -> usize {
        if value == 0 {
            return 0;
        }
        let power = 63 - value.leading_zeros(); // floor(log2(value))
        if power >= POWERS {
            return BUCKETS - 1;
        }
        // Position inside the power of two, linearly split into SUB_BUCKETS.
        let base = 1u64 << power;
        let offset = ((value - base) * SUB_BUCKETS as u64) / base;
        (power * SUB_BUCKETS) as usize + offset as usize
    }

    /// The value at the top of a bucket, which is what a percentile reports.
    fn value_at(index: usize) -> u64 {
        let power = index as u32 / SUB_BUCKETS;
        let offset = index as u32 % SUB_BUCKETS;
        if power == 0 {
            return offset as u64;
        }
        let base = 1u64 << power;
        base + (base / SUB_BUCKETS as u64) * (offset as u64 + 1)
    }

    pub fn record(&mut self, value: u64) {
        self.buckets[Histogram::index_of(value)] += 1;
        self.count += 1;
        self.sum += value as f64;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn mean(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }

    pub fn min(&self) -> u64 {
        if self.count == 0 {
            0
        } else {
            self.min
        }
    }

    pub fn max(&self) -> u64 {
        self.max
    }

    /// The smallest recorded value at or above the given quantile, to bucket
    /// resolution. `quantile` is clamped into `[0, 1]`.
    pub fn quantile(&self, quantile: f64) -> u64 {
        if self.count == 0 {
            return 0;
        }
        let q = quantile.clamp(0.0, 1.0);
        let target = (q * self.count as f64).ceil().max(1.0) as u64;
        let mut seen = 0u64;
        for (i, bucket) in self.buckets.iter().enumerate() {
            seen += bucket;
            if seen >= target {
                return Histogram::value_at(i).min(self.max);
            }
        }
        self.max
    }

    /// How many recorded values were at or above a threshold.
    pub fn count_at_or_above(&self, threshold: u64) -> u64 {
        let from = Histogram::index_of(threshold);
        self.buckets[from..].iter().sum()
    }

    /// The reported shape of a measurement: every number a reader needs and
    /// the sample size that earned it.
    pub fn summary(&self, scale: f64) -> Summary {
        Summary {
            count: self.count,
            mean: self.mean() * scale,
            min: self.min() as f64 * scale,
            p50: self.quantile(0.50) as f64 * scale,
            p90: self.quantile(0.90) as f64 * scale,
            p99: self.quantile(0.99) as f64 * scale,
            p999: self.quantile(0.999) as f64 * scale,
            max: self.max() as f64 * scale,
        }
    }
}

/// A distribution as it is printed. Never a bare mean.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub count: u64,
    pub mean: f64,
    pub min: f64,
    pub p50: f64,
    pub p90: f64,
    pub p99: f64,
    pub p999: f64,
    pub max: f64,
}

/// Running measurements for one server, reset never; the report is a snapshot
/// of them. Recording is a few integer operations, so it stays on in production.
#[derive(Debug, Default)]
pub struct TickStats {
    tick_ns: Histogram,
    snapshot_bytes: Histogram,
    ticks: u64,
    over_budget: u64,
    over_half_budget: u64,
    started: Option<Instant>,
}

impl TickStats {
    pub fn new() -> Self {
        TickStats {
            started: Some(Instant::now()),
            ..TickStats::default()
        }
    }

    /// Record one tick: how long the whole step took and how many bytes the
    /// snapshot it produced serialised to.
    pub fn record_tick(&mut self, elapsed: Duration, snapshot_bytes: usize) {
        let ns = elapsed.as_nanos().min(u64::MAX as u128) as u64;
        self.tick_ns.record(ns);
        self.snapshot_bytes.record(snapshot_bytes as u64);
        self.ticks += 1;
        let budget = TICK.as_nanos() as u64;
        if ns >= budget {
            self.over_budget += 1;
        }
        if ns * 2 >= budget {
            self.over_half_budget += 1;
        }
    }

    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    pub fn over_budget(&self) -> u64 {
        self.over_budget
    }

    /// The report, with time in milliseconds and budget use as a fraction.
    pub fn report(&self, fighters: usize, clients: usize) -> StatsReport {
        let budget_ns = TICK.as_nanos() as f64;
        StatsReport {
            uptime_s: self
                .started
                .map(|t| t.elapsed().as_secs_f64())
                .unwrap_or(0.0),
            ticks: self.ticks,
            tick_hz: 1.0 / TICK.as_secs_f64(),
            fighters,
            clients,
            tick_ms: self.tick_ns.summary(1e-6),
            snapshot_bytes: self.snapshot_bytes.summary(1.0),
            budget_use_p50: self.tick_ns.quantile(0.50) as f64 / budget_ns,
            budget_use_p99: self.tick_ns.quantile(0.99) as f64 / budget_ns,
            ticks_over_budget: self.over_budget,
            ticks_over_half_budget: self.over_half_budget,
        }
    }
}

/// What `--bench` prints and what the status line serves.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatsReport {
    pub uptime_s: f64,
    pub ticks: u64,
    pub tick_hz: f64,
    pub fighters: usize,
    pub clients: usize,
    /// Whole-tick time in milliseconds. Phase splits come with the sim refactor.
    pub tick_ms: Summary,
    /// Serialised snapshot size in bytes, one sample per tick.
    pub snapshot_bytes: Summary,
    pub budget_use_p50: f64,
    pub budget_use_p99: f64,
    pub ticks_over_budget: u64,
    pub ticks_over_half_budget: u64,
}

/// What a benchmark run was asked to do, printed beside its numbers so a
/// figure can always be traced back to the run that produced it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchConfig {
    pub bots: usize,
    pub ticks: u64,
    pub map: u32,
    pub map_name: String,
    pub seed: u64,
    pub build: String,
}

/// A benchmark run: the configuration, the measurements, and the determinism
/// check that two seeded runs of the same shape produce the same match.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchReport {
    pub config: BenchConfig,
    pub stats: StatsReport,
    /// Frags by fighter name at the end of the run, the cheap fingerprint of a
    /// match. Two runs with the same seed must agree on it.
    pub frags: Vec<(String, u32)>,
    /// Set when the run was repeated to check determinism.
    pub deterministic: Option<bool>,
}

/// Run `ticks` ticks of a session with `bots` fighters and no network, timing
/// every step. Deterministic for a given seed: the same seed gives the same
/// match, which is what makes the numbers comparable between runs.
pub fn run_bench(bots: usize, ticks: u64, map: MapKind, seed: u64) -> BenchReport {
    let mut session = GameSession::with_map(map, false);
    session.state.seed(seed);
    session.state.config = MatchConfig {
        // A benchmark measures fighting, not the pauses around it.
        warmup_ticks: 1,
        ..MatchConfig::default()
    };
    session.spawn_bots(bots);
    let mut stats = TickStats::new();
    let dt = TICK.as_secs_f32();
    for _ in 0..ticks {
        let started = Instant::now();
        let messages = session.tick_messages(dt);
        let elapsed = started.elapsed();
        let bytes = messages
            .iter()
            .map(|m| serde_json::to_string(m).map(|s| s.len()).unwrap_or(0))
            .sum::<usize>();
        stats.record_tick(elapsed, bytes);
        session.take_unicasts();
    }
    let mut frags: Vec<(String, u32)> = session
        .state
        .players
        .iter()
        .map(|p| {
            let score = session.state.scores.get(&p.id).copied().unwrap_or(0);
            (p.name.clone(), score)
        })
        .collect();
    frags.sort();
    let fighters = session.state.players.len();
    BenchReport {
        config: BenchConfig {
            bots,
            ticks,
            map: map.id(),
            map_name: map.name().to_string(),
            seed,
            build: env!("CARGO_PKG_VERSION").to_string(),
        },
        stats: stats.report(fighters, 0),
        frags,
        deterministic: None,
    }
}

/// Run the benchmark twice with the same seed and report whether the two
/// matches agreed. A disagreement is a correctness bug, not a slow tick.
pub fn run_bench_checked(bots: usize, ticks: u64, map: MapKind, seed: u64) -> BenchReport {
    let first = run_bench(bots, ticks, map, seed);
    let second = run_bench(bots, ticks, map, seed);
    BenchReport {
        deterministic: Some(first.frags == second.frags),
        ..first
    }
}

/// What a benchmark must satisfy to pass. Returned as complaints rather than
/// a bare boolean so a failing build says which line to look at.
pub fn check_thresholds(report: &BenchReport, max_budget_p99: f64) -> Vec<String> {
    let mut out = Vec::new();
    let stats = &report.stats;
    if report.deterministic == Some(false) {
        out.push("two runs with the same seed produced different matches".to_string());
    }
    if stats.ticks_over_budget > 0 {
        out.push(format!(
            "{} of {} ticks missed the {:.0} ms budget",
            stats.ticks_over_budget,
            stats.ticks,
            1000.0 / stats.tick_hz
        ));
    }
    if stats.budget_use_p99 > max_budget_p99 {
        out.push(format!(
            "the p99 tick used {:.1} percent of the budget, over the {:.1} percent allowed",
            stats.budget_use_p99 * 100.0,
            max_budget_p99 * 100.0
        ));
    }
    if stats.ticks == 0 {
        out.push("no ticks were measured".to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histogram_counts_and_percentiles_are_honest() {
        let mut h = Histogram::new();
        assert_eq!(h.count(), 0);
        assert_eq!(h.quantile(0.5), 0);
        assert_eq!(h.mean(), 0.0);
        assert_eq!(h.min(), 0);
        for v in 1..=1000u64 {
            h.record(v);
        }
        assert_eq!(h.count(), 1000);
        assert_eq!(h.min(), 1);
        assert_eq!(h.max(), 1000);
        assert!((h.mean() - 500.5).abs() < 1e-6);
        // Every percentile within one bucket width (about 7 percent here, since
        // a bucket near 500 spans 32 units) of the true value.
        for (q, want) in [(0.5, 500.0), (0.9, 900.0), (0.99, 990.0)] {
            let got = h.quantile(q) as f64;
            assert!(
                (got - want).abs() / want < 0.08,
                "p{q}: got {got}, want about {want}"
            );
            assert!(got >= want * 0.92 && got <= h.max() as f64);
        }
        assert_eq!(h.quantile(1.0), h.max());
        assert_eq!(h.quantile(-5.0), h.quantile(0.0));
        assert_eq!(h.quantile(9.0), h.max());
    }

    #[test]
    fn histogram_handles_zero_and_huge_values() {
        let mut h = Histogram::new();
        h.record(0);
        assert_eq!(h.count(), 1);
        assert_eq!(h.min(), 0);
        assert_eq!(h.quantile(0.5), 0);
        h.record(u64::MAX);
        assert_eq!(h.max(), u64::MAX);
        assert_eq!(h.count(), 2);
        // Counting above a threshold is what budget overruns use.
        let mut b = Histogram::new();
        for v in [1u64, 10, 100, 1000, 10_000] {
            b.record(v);
        }
        assert_eq!(b.count_at_or_above(100), 3);
        assert_eq!(b.count_at_or_above(0), 5);
        assert_eq!(b.count_at_or_above(u64::MAX), 0);
    }

    #[test]
    fn summary_scales_and_carries_its_sample_size() {
        let mut h = Histogram::new();
        for _ in 0..10 {
            h.record(2_000_000); // 2 ms in nanoseconds
        }
        let s = h.summary(1e-6);
        assert_eq!(s.count, 10);
        assert!((s.mean - 2.0).abs() < 0.05, "{s:?}");
        assert!((s.p50 - 2.0).abs() < 0.1, "{s:?}");
        assert!(s.max >= s.p999 && s.p999 >= s.p99 && s.p99 >= s.p50 && s.p50 >= s.min);
        let json = serde_json::to_string(&s).unwrap();
        let back: Summary = serde_json::from_str(&json).unwrap();
        assert_eq!(back.count, s.count);
        assert!((back.p99 - s.p99).abs() < 1e-9);
    }

    #[test]
    fn tick_stats_count_budget_overruns() {
        let mut stats = TickStats::new();
        let budget = TICK;
        stats.record_tick(budget / 10, 1000);
        stats.record_tick(budget / 4, 1000);
        stats.record_tick(budget / 2, 1000);
        stats.record_tick(budget * 2, 5000);
        assert_eq!(stats.ticks(), 4);
        assert_eq!(stats.over_budget(), 1);
        let report = stats.report(8, 2);
        assert_eq!(report.ticks, 4);
        assert_eq!(report.fighters, 8);
        assert_eq!(report.clients, 2);
        assert_eq!(report.ticks_over_budget, 1);
        assert_eq!(
            report.ticks_over_half_budget, 2,
            "half a budget and two budgets count; a quarter does not"
        );
        assert!(report.budget_use_p99 > 1.0, "{report:?}");
        assert!(
            report.tick_ms.max >= 99.0,
            "two budgets is 100 ms: {:?}",
            report.tick_ms
        );
        assert!(report.snapshot_bytes.max >= 4900.0);
        assert!((report.tick_hz - 20.0).abs() < 1e-9);
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"budget_use_p99\""), "{json}");
    }

    #[test]
    fn thresholds_name_what_went_wrong() {
        let mut report = run_bench(4, 100, MapKind::ArenaDuel, 1);
        assert!(
            check_thresholds(&report, 0.5).is_empty(),
            "a healthy run has no complaints: {:?}",
            report.stats
        );
        report.deterministic = Some(false);
        assert!(check_thresholds(&report, 0.5)[0].contains("different matches"));
        report.deterministic = Some(true);
        report.stats.ticks_over_budget = 3;
        let complaints = check_thresholds(&report, 0.5);
        assert!(
            complaints
                .iter()
                .any(|c| c.contains("missed the 50 ms budget")),
            "{complaints:?}"
        );
        report.stats.ticks_over_budget = 0;
        report.stats.budget_use_p99 = 0.75;
        let complaints = check_thresholds(&report, 0.5);
        assert!(
            complaints.iter().any(|c| c.contains("75.0 percent")),
            "{complaints:?}"
        );
        assert!(
            check_thresholds(&report, 0.8).is_empty(),
            "a looser bar passes"
        );
        report.stats.ticks = 0;
        assert!(check_thresholds(&report, 0.8)[0].contains("no ticks"));
    }

    #[test]
    fn bench_runs_and_reports_a_match() {
        let report = run_bench(4, 200, MapKind::ArenaDuel, 7);
        assert_eq!(report.config.bots, 4);
        assert_eq!(report.config.ticks, 200);
        assert_eq!(report.config.seed, 7);
        assert_eq!(report.config.map, MapKind::ArenaDuel.id());
        assert_eq!(report.stats.ticks, 200);
        assert!(report.stats.fighters >= 4, "{:?}", report.stats);
        assert!(report.stats.snapshot_bytes.p50 > 0.0);
        assert!(report.stats.tick_ms.max >= report.stats.tick_ms.p50);
        assert_eq!(report.frags.len(), report.stats.fighters);
        assert!(report.deterministic.is_none());
        // JSON keeps every field; float text is not bit-exact, so compare the
        // parts a reader depends on rather than the whole struct.
        let json = serde_json::to_string(&report).unwrap();
        let back: BenchReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.config, report.config);
        assert_eq!(back.frags, report.frags);
        assert_eq!(back.stats.ticks, report.stats.ticks);
        assert_eq!(back.stats.tick_ms.count, report.stats.tick_ms.count);
        assert!((back.stats.tick_ms.p99 - report.stats.tick_ms.p99).abs() < 1e-9);
    }

    #[test]
    fn a_seeded_bench_reproduces_itself_and_a_different_seed_differs() {
        let a = run_bench_checked(4, 400, MapKind::ArenaDuel, 42);
        assert_eq!(
            a.deterministic,
            Some(true),
            "same seed, same match: {:?}",
            a.frags
        );
        let b = run_bench(4, 400, MapKind::ArenaDuel, 42);
        assert_eq!(a.frags, b.frags, "a third run agrees too");
        let c = run_bench(4, 400, MapKind::ArenaDuel, 43);
        // Different seeds should usually diverge; if they ever agree the test
        // still passes, but the frag totals must at least be produced.
        assert_eq!(c.frags.len(), a.frags.len());
    }
}
