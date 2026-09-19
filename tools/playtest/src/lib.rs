//! Agent playtest harness: boots the authoritative server in-process on a free
//! loopback port, connects scripted agents over the real wire, watches the match
//! as a spectator, and turns what it saw into a metrics report. Humans still
//! judge fun; this catches stuck agents, dead time, spawn deaths, and regressions
//! in the numbers that make a round feel alive.

use fragr_server::protocol::{
    Action, ClientMessage, GameEvent, LookAt, Role, ServerMessage, Snapshot, WeaponType,
};
use fragr_server::run::{run_server, ServerOptions, TICK};
use fragr_server::sim::{MapKind, MatchConfig};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

/// Ticks per second of the authoritative loop.
pub const TICKS_PER_SECOND: f64 = 20.0;
/// A death this soon after a spawn counts as a spawn death.
pub const SPAWN_DEATH_WINDOW_TICKS: u64 = 40;
/// Reflex agents fire inside this range and walk toward targets beyond three units.
const FIRE_RANGE: f32 = 20.0;
const CLOSE_RANGE: f32 = 3.0;
/// Hold the shotgun inside this distance, the railgun beyond the next one,
/// and the needle gun between. The first combat report showed the reflex
/// agents never swapping, which left two of the three weapons unmeasured and
/// the weapon triangle an assertion rather than a finding.
const SCATTER_RANGE: f32 = 10.0;
const RAIL_RANGE: f32 = 30.0;

/// The weapon a fighter should be holding at this distance.
pub fn weapon_for_distance(dist: f32) -> WeaponType {
    if dist < SCATTER_RANGE {
        WeaponType::Scatter
    } else if dist > RAIL_RANGE {
        WeaponType::Rail
    } else {
        WeaponType::Flechette
    }
}

/// Parse a wire weapon name; unknown names keep whatever is held.
fn weapon_from_wire(name: &str) -> Option<WeaponType> {
    match name.to_ascii_lowercase().as_str() {
        "flechette" => Some(WeaponType::Flechette),
        "rail" => Some(WeaponType::Rail),
        "scatter" => Some(WeaponType::Scatter),
        _ => None,
    }
}
/// Movement smaller than this between snapshots counts as idle.
const IDLE_EPSILON: f32 = 0.01;

#[derive(Debug, Clone)]
pub struct Config {
    pub agents: usize,
    pub rounds: u32,
    pub map: MapKind,
    pub frag_limit: u32,
    pub time_limit_ticks: u32,
    /// Hard stop for the whole run, in ticks of the observed clock.
    pub max_ticks: u64,
    /// Simulation seed. The same seed gives the same match, which is what lets
    /// two harness runs be compared rather than merely averaged.
    pub seed: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            agents: 4,
            rounds: 1,
            map: MapKind::ArenaDuel,
            frag_limit: 5,
            time_limit_ticks: 20 * 60,
            max_ticks: 20 * 120,
            seed: 1,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Server(String),
    Transport(String),
    Timeout(&'static str),
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Server(msg) => write!(f, "server error: {msg}"),
            Error::Transport(msg) => write!(f, "transport error: {msg}"),
            Error::Timeout(what) => write!(f, "timed out waiting for {what}"),
            Error::Io(err) => write!(f, "io error: {err}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

/// One event with the observer's clock (the last snapshot tick seen before it).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedEvent {
    pub tick: u64,
    pub event: GameEvent,
}

/// Width of an engagement-distance bucket, in world units. Five units is
/// about a third of the scatter gun's reach and a twelfth of the rail's, so
/// the three weapons land in visibly different buckets.
pub const DISTANCE_BUCKET: f32 = 5.0;
/// Buckets kept; the last one is everything beyond fifty units, which is a
/// corner-to-corner shot in a fifty unit arena.
pub const DISTANCE_BUCKETS: usize = 11;

/// A distribution reported the honest way: the shape, not just the middle,
/// and always with the sample size that earned it.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Quantiles {
    pub count: u64,
    pub min: f64,
    pub p50: f64,
    pub p90: f64,
    pub max: f64,
    pub mean: f64,
}

impl Quantiles {
    /// From values in any order. Empty gives zeros and a count of zero, which
    /// is how a reader knows there is nothing to read.
    pub fn from_values(values: &[f64]) -> Self {
        let mut sorted: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
        if sorted.is_empty() {
            return Quantiles::default();
        }
        sorted.sort_by(|a, b| a.total_cmp(b));
        let pick = |q: f64| -> f64 {
            let idx = ((sorted.len() as f64 - 1.0) * q).round() as usize;
            sorted[idx.min(sorted.len() - 1)]
        };
        Quantiles {
            count: sorted.len() as u64,
            min: sorted[0],
            p50: pick(0.5),
            p90: pick(0.9),
            max: sorted[sorted.len() - 1],
            mean: sorted.iter().sum::<f64>() / sorted.len() as f64,
        }
    }
}

/// Wilson score interval for a proportion at about ninety five percent
/// confidence. A hit rate from nine shots and one from nine hundred are not
/// the same claim, and this is what says so. Zero trials gives the full range.
pub fn wilson_interval(hits: u64, trials: u64) -> (f64, f64) {
    if trials == 0 {
        return (0.0, 1.0);
    }
    let z = 1.96f64;
    let n = trials as f64;
    let p = hits as f64 / n;
    let denom = 1.0 + z * z / n;
    let centre = p + z * z / (2.0 * n);
    let margin = z * ((p * (1.0 - p) / n) + (z * z / (4.0 * n * n))).sqrt();
    (
        ((centre - margin) / denom).clamp(0.0, 1.0),
        ((centre + margin) / denom).clamp(0.0, 1.0),
    )
}

/// What one weapon did: how often it fired, how often that landed, how far
/// away, and how much damage it dealt.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WeaponReport {
    pub shots: u64,
    pub hits: u64,
    pub accuracy: f64,
    /// The interval on that accuracy, so a small sample admits it.
    pub accuracy_lo: f64,
    pub accuracy_hi: f64,
    pub damage: i64,
    pub kills: u64,
    /// Distance at which its shots landed.
    pub hit_distance: Quantiles,
    /// Distance at which it killed. The weapon triangle works when these peak
    /// in different places.
    pub kill_distance: Quantiles,
}

/// The combat picture: how long a kill takes, how often shots land, and at
/// what range each weapon does its work.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CombatReport {
    /// Seconds from the first damage on a victim to their death.
    pub time_to_kill_s: Quantiles,
    pub shots: u64,
    pub hits: u64,
    pub accuracy: f64,
    pub accuracy_lo: f64,
    pub accuracy_hi: f64,
    pub shots_per_kill: f64,
    /// Kills per five unit bucket, the last holding everything beyond.
    pub kill_distance_buckets: Vec<u64>,
    pub by_weapon: BTreeMap<String, WeaponReport>,
}

/// Running tallies for one weapon while a run is in flight.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WeaponTally {
    pub shots: u64,
    pub hits: u64,
    pub damage: i64,
    pub kills: u64,
    pub hit_distances: Vec<f64>,
    pub kill_distances: Vec<f64>,
}

/// Per-fighter movement and fire bookkeeping from snapshots.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AgentTrack {
    pub ticks_present: u64,
    /// Ticks without movement while a round is active.
    pub idle_ticks: u64,
    /// Longest run of active-round ticks with no movement and no fire.
    pub stuck_max_ticks: u64,
    stuck_run: u64,
    last_pos: Option<(f32, f32)>,
    pub fire_ticks: u64,
    pub weapon_fire_ticks: BTreeMap<String, u64>,
    /// Weapon held on the newest snapshot, for attributing a frag.
    pub last_weapon: Option<String>,
}

/// Everything the observer keeps. Snapshots are folded in as they arrive so a
/// long run does not hold every frame in memory.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Observation {
    pub snapshots_seen: u64,
    pub first_tick: Option<u64>,
    pub last_tick: u64,
    pub snapshot_bytes: u64,
    pub events: Vec<TimedEvent>,
    pub tracks: BTreeMap<String, AgentTrack>,
    /// Per weapon, what it fired and what landed.
    pub weapons: BTreeMap<String, WeaponTally>,
    /// Victim name to the tick their engagement started, so a death can be
    /// timed from the first damage that led to it.
    pub engagement_start: BTreeMap<String, u64>,
    /// Seconds from first damage to death, one per kill that had an opening hit.
    pub time_to_kill_s: Vec<f64>,
    /// Distance of every kill, for the histogram.
    pub kill_distances: Vec<f64>,
}

impl Observation {
    pub fn ingest_snapshot(&mut self, snapshot: &Snapshot, bytes: usize) {
        self.snapshots_seen += 1;
        self.snapshot_bytes += bytes as u64;
        if self.first_tick.is_none() {
            self.first_tick = Some(snapshot.tick);
        }
        self.last_tick = self.last_tick.max(snapshot.tick);
        let active = snapshot.round_state.as_deref() == Some("Active");
        for player in &snapshot.players {
            let track = self.tracks.entry(player.name.clone()).or_default();
            track.ticks_present += 1;
            let pos = (player.x, player.z);
            if let Some((lx, lz)) = track.last_pos {
                let moved = ((pos.0 - lx).powi(2) + (pos.1 - lz).powi(2)).sqrt();
                if !active || moved >= IDLE_EPSILON || player.just_fired {
                    // Moving, fighting, or between rounds is not stuck.
                    track.stuck_run = 0;
                } else {
                    track.stuck_run += 1;
                    track.stuck_max_ticks = track.stuck_max_ticks.max(track.stuck_run);
                }
                if active && moved < IDLE_EPSILON {
                    track.idle_ticks += 1;
                }
            }
            track.last_pos = Some(pos);
            track.last_weapon = Some(player.weapon.clone());
            if player.just_fired {
                track.fire_ticks += 1;
                *track
                    .weapon_fire_ticks
                    .entry(player.weapon.clone())
                    .or_default() += 1;
            }
        }
        self.ingest_shots(snapshot);
    }

    /// Every shot resolved on this tick, with the distance it travelled. The
    /// server publishes hits and misses, so accuracy is exact rather than
    /// inferred from fire ticks.
    fn ingest_shots(&mut self, snapshot: &Snapshot) {
        if snapshot.shot_results.is_empty() {
            return;
        }
        let by_id: BTreeMap<Uuid, (String, f32, f32)> = snapshot
            .players
            .iter()
            .map(|p| (p.id, (p.weapon.clone(), p.x, p.z)))
            .collect();
        for shot in &snapshot.shot_results {
            let Some((weapon, sx, sz)) = by_id.get(&shot.shooter_id).cloned() else {
                continue;
            };
            let tally = self.weapons.entry(weapon).or_default();
            tally.shots += 1;
            if !shot.hit {
                continue;
            }
            tally.hits += 1;
            tally.damage += shot.damage as i64;
            if let Some((_, tx, tz)) = shot.target_id.and_then(|id| by_id.get(&id)).cloned() {
                let distance = ((tx - sx).powi(2) + (tz - sz).powi(2)).sqrt() as f64;
                tally.hit_distances.push(distance);
            }
        }
    }

    pub fn ingest_event(&mut self, event: GameEvent) {
        match &event {
            // The clock on a death starts at the first damage that led to it.
            GameEvent::Hit { target, .. } => {
                self.engagement_start
                    .entry(target.clone())
                    .or_insert(self.last_tick);
            }
            GameEvent::Frag { killer, victim, .. } => {
                if let Some(start) = self.engagement_start.remove(victim) {
                    let ticks = self.last_tick.saturating_sub(start);
                    self.time_to_kill_s.push(ticks as f64 / TICKS_PER_SECOND);
                }
                let killer_track = self.tracks.get(killer);
                let killer_pos = killer_track.and_then(|t| t.last_pos);
                let killer_weapon = killer_track.and_then(|t| t.last_weapon.clone());
                let victim_pos = self.tracks.get(victim).and_then(|t| t.last_pos);
                if let (Some((kx, kz)), Some((vx, vz))) = (killer_pos, victim_pos) {
                    let distance = ((vx - kx).powi(2) + (vz - kz).powi(2)).sqrt() as f64;
                    self.kill_distances.push(distance);
                    if let Some(weapon) = killer_weapon {
                        let tally = self.weapons.entry(weapon).or_default();
                        tally.kills += 1;
                        tally.kill_distances.push(distance);
                    }
                }
            }
            // A fighter who respawned is not still in their last engagement.
            GameEvent::Respawn { player } => {
                self.engagement_start.remove(player);
            }
            _ => {}
        }
        self.events.push(TimedEvent {
            tick: self.last_tick,
            event,
        });
    }

    /// Fold the running tallies into the combat picture.
    pub fn combat_report(&self) -> CombatReport {
        let mut shots = 0u64;
        let mut hits = 0u64;
        let mut kills = 0u64;
        let mut by_weapon = BTreeMap::new();
        for (weapon, tally) in &self.weapons {
            shots += tally.shots;
            hits += tally.hits;
            kills += tally.kills;
            let accuracy = if tally.shots == 0 {
                0.0
            } else {
                tally.hits as f64 / tally.shots as f64
            };
            let (lo, hi) = wilson_interval(tally.hits, tally.shots);
            by_weapon.insert(
                weapon.clone(),
                WeaponReport {
                    shots: tally.shots,
                    hits: tally.hits,
                    accuracy,
                    accuracy_lo: lo,
                    accuracy_hi: hi,
                    damage: tally.damage,
                    kills: tally.kills,
                    hit_distance: Quantiles::from_values(&tally.hit_distances),
                    kill_distance: Quantiles::from_values(&tally.kill_distances),
                },
            );
        }
        let mut buckets = vec![0u64; DISTANCE_BUCKETS];
        for distance in &self.kill_distances {
            let index = ((distance / DISTANCE_BUCKET as f64) as usize).min(DISTANCE_BUCKETS - 1);
            buckets[index] += 1;
        }
        let (lo, hi) = wilson_interval(hits, shots);
        CombatReport {
            time_to_kill_s: Quantiles::from_values(&self.time_to_kill_s),
            shots,
            hits,
            accuracy: if shots == 0 {
                0.0
            } else {
                hits as f64 / shots as f64
            },
            accuracy_lo: lo,
            accuracy_hi: hi,
            shots_per_kill: if kills == 0 {
                0.0
            } else {
                shots as f64 / kills as f64
            },
            kill_distance_buckets: buckets,
            by_weapon,
        }
    }

    pub fn rounds_completed(&self) -> u32 {
        self.events
            .iter()
            .filter(|e| matches!(e.event, GameEvent::RoundEnd { .. }))
            .count() as u32
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AgentReport {
    pub frags: u64,
    pub deaths: u64,
    pub spawn_deaths: u64,
    pub frags_per_minute: f64,
    pub deaths_per_minute: f64,
    pub idle_ratio: f64,
    pub stuck_max_s: f64,
    pub fire_ticks: u64,
    pub weapon_fire_ticks: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Report {
    pub agents: usize,
    pub rounds_completed: u32,
    pub ticks: u64,
    pub seconds: f64,
    pub frags: u64,
    pub frags_per_minute: f64,
    pub time_to_first_frag_s: Option<f64>,
    pub longest_gap_without_frag_s: f64,
    pub host_beats_per_minute: f64,
    pub pickups: u64,
    pub spawn_deaths: u64,
    pub snapshot_bytes_per_tick: f64,
    /// How the fighting actually went: time to kill, accuracy with intervals,
    /// and where each weapon does its work.
    pub combat: CombatReport,
    pub per_agent: BTreeMap<String, AgentReport>,
}

fn seconds(ticks: u64) -> f64 {
    ticks as f64 / TICKS_PER_SECOND
}

fn per_minute(count: u64, ticks: u64) -> f64 {
    if ticks == 0 {
        return 0.0;
    }
    count as f64 / (seconds(ticks) / 60.0)
}

/// Fold an observation into the report. Pure, so canned observations test it.
pub fn compute_report(obs: &Observation, agents: usize) -> Report {
    let first = obs.first_tick.unwrap_or(0);
    let ticks = obs.last_tick.saturating_sub(first).max(1);
    let mut per_agent: BTreeMap<String, AgentReport> = BTreeMap::new();
    for (name, track) in &obs.tracks {
        per_agent.insert(
            name.clone(),
            AgentReport {
                idle_ratio: if track.ticks_present == 0 {
                    0.0
                } else {
                    track.idle_ticks as f64 / track.ticks_present as f64
                },
                stuck_max_s: seconds(track.stuck_max_ticks),
                fire_ticks: track.fire_ticks,
                weapon_fire_ticks: track.weapon_fire_ticks.clone(),
                ..AgentReport::default()
            },
        );
    }

    let mut last_spawn: BTreeMap<String, u64> = BTreeMap::new();
    let mut frag_ticks: Vec<u64> = Vec::new();
    let mut host_beats = 0u64;
    let mut pickups = 0u64;
    let mut spawn_deaths = 0u64;
    for timed in &obs.events {
        match &timed.event {
            GameEvent::RoundStart { players, .. } => {
                host_beats += 1;
                for player in players {
                    last_spawn.insert(player.clone(), timed.tick);
                }
            }
            GameEvent::Respawn { player } => {
                last_spawn.insert(player.clone(), timed.tick);
            }
            GameEvent::Frag { killer, victim, .. } => {
                frag_ticks.push(timed.tick);
                per_agent.entry(killer.clone()).or_default().frags += 1;
                let victim_report = per_agent.entry(victim.clone()).or_default();
                victim_report.deaths += 1;
                if let Some(spawned) = last_spawn.get(victim) {
                    if timed.tick.saturating_sub(*spawned) <= SPAWN_DEATH_WINDOW_TICKS {
                        victim_report.spawn_deaths += 1;
                        spawn_deaths += 1;
                    }
                }
            }
            GameEvent::RoundEnd { .. }
            | GameEvent::Killstreak { .. }
            | GameEvent::CompliancePing { .. }
            | GameEvent::BossSpawn { .. }
            | GameEvent::BossDown { .. } => host_beats += 1,
            GameEvent::Pickup { .. } => pickups += 1,
            GameEvent::Hit { .. }
            | GameEvent::PlayerJoined { .. }
            | GameEvent::PlayerLeft { .. }
            | GameEvent::Speak { .. } => {}
            GameEvent::EpisodeStart { .. }
            | GameEvent::EpisodeComplete { .. }
            | GameEvent::EpisodeFail { .. } => {}
        }
    }
    for report in per_agent.values_mut() {
        report.frags_per_minute = per_minute(report.frags, ticks);
        report.deaths_per_minute = per_minute(report.deaths, ticks);
    }

    let frags = frag_ticks.len() as u64;
    let time_to_first_frag_s = frag_ticks.first().map(|t| seconds(t.saturating_sub(first)));
    let mut longest_gap = 0u64;
    let mut previous = first;
    for t in &frag_ticks {
        longest_gap = longest_gap.max(t.saturating_sub(previous));
        previous = *t;
    }
    longest_gap = longest_gap.max(obs.last_tick.saturating_sub(previous));

    Report {
        agents,
        rounds_completed: obs.rounds_completed(),
        ticks,
        seconds: seconds(ticks),
        frags,
        frags_per_minute: per_minute(frags, ticks),
        time_to_first_frag_s,
        longest_gap_without_frag_s: seconds(longest_gap),
        host_beats_per_minute: per_minute(host_beats, ticks),
        pickups,
        spawn_deaths,
        snapshot_bytes_per_tick: if obs.snapshots_seen == 0 {
            0.0
        } else {
            obs.snapshot_bytes as f64 / obs.snapshots_seen as f64
        },
        combat: obs.combat_report(),
        per_agent,
    }
}

/// Frustration signals that block a merge. Empty means the run is acceptable.
pub fn check_thresholds(report: &Report) -> Vec<String> {
    let mut problems = Vec::new();
    if report.rounds_completed == 0 {
        problems.push("no round completed".to_string());
    }
    for (name, agent) in &report.per_agent {
        if agent.stuck_max_s > 5.0 {
            problems.push(format!("{name} stuck for {:.1} s", agent.stuck_max_s));
        }
    }
    if report.frags >= 10 && report.spawn_deaths as f64 / report.frags as f64 > 0.10 {
        problems.push(format!(
            "spawn deaths {} of {} frags",
            report.spawn_deaths, report.frags
        ));
    }
    if report.agents >= 4 && report.frags_per_minute < 1.0 {
        problems.push(format!(
            "only {:.2} frags per minute with {} agents",
            report.frags_per_minute, report.agents
        ));
    }
    problems
}

/// Reflex tier: face the nearest fighter, close in, fire in range. Same policy as
/// the adapter's scripted bot, driven straight from the server's wire types.
pub fn reflex_action(bot_id: Uuid, snapshot: &Snapshot) -> Action {
    let Some(me) = snapshot.players.iter().find(|p| p.id == bot_id) else {
        return Action::default();
    };
    let mut nearest: Option<(f32, Uuid)> = None;
    for other in &snapshot.players {
        if other.id == bot_id {
            continue;
        }
        let dist = ((other.x - me.x).powi(2) + (other.z - me.z).powi(2)).sqrt();
        if nearest.is_none_or(|(d, _)| dist < d) {
            nearest = Some((dist, other.id));
        }
    }
    let Some((dist, target)) = nearest else {
        return Action::default();
    };
    // Swap only when the right weapon is not already in hand, so the report
    // does not fill with pointless swaps.
    let wanted = weapon_for_distance(dist);
    let weapon_swap = (weapon_from_wire(&me.weapon) != Some(wanted)).then_some(wanted);
    Action {
        look_at: Some(LookAt {
            player_id: Some(target),
            x: None,
            z: None,
        }),
        forward: dist > CLOSE_RANGE,
        fire: dist < FIRE_RANGE,
        weapon_swap,
        ..Action::default()
    }
}

fn transport<E: fmt::Display>(err: E) -> Error {
    Error::Transport(err.to_string())
}

async fn agent_task(url: String, name: String, stop: Arc<AtomicBool>) -> Result<(), Error> {
    let (ws, _) = connect_async(&url).await.map_err(transport)?;
    let (mut sink, mut stream) = ws.split();
    let hello = ClientMessage::Hello {
        role: Role::Agent,
        name,
    };
    sink.send(Message::Text(
        serde_json::to_string(&hello).map_err(transport)?,
    ))
    .await
    .map_err(transport)?;
    let mut player_id: Option<Uuid> = None;
    while let Some(msg) = stream.next().await {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        let Ok(Message::Text(text)) = msg else {
            continue;
        };
        match serde_json::from_str::<ServerMessage>(&text) {
            Ok(ServerMessage::Welcome { player_id: pid, .. }) => player_id = pid,
            Ok(ServerMessage::Snapshot(snapshot)) => {
                let Some(id) = player_id else {
                    continue;
                };
                let action = ClientMessage::Action(reflex_action(id, &snapshot));
                if sink
                    .send(Message::Text(
                        serde_json::to_string(&action).map_err(transport)?,
                    ))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            _ => {}
        }
    }
    let _ = sink.close().await;
    Ok(())
}

/// Run one playtest: server plus agents plus observer, then the report.
pub async fn run(config: Config) -> Result<(Report, Observation), Error> {
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    // Controlled rounds: no mid-round boss or compliance beat, so the numbers
    // describe the fighters and nothing else.
    let match_config = MatchConfig {
        frag_limit: Some(config.frag_limit),
        time_limit_ticks: Some(config.time_limit_ticks),
        boss_spawn_ticks: None,
        compliance_ping_ticks: None,
        ..MatchConfig::default()
    };
    let options = ServerOptions {
        bind: "127.0.0.1:0".to_string(),
        bots: 0,
        map: config.map,
        map_rotate: false,
        match_config: Some(match_config),
        // A harness run is reproducible and quiet: the report is the output.
        seed: config.seed,
        status_every_s: 0,
        solo_broadcast: false,
    };
    let server = tokio::spawn(async move {
        run_server(
            options,
            async move {
                let _ = shutdown_rx.await;
            },
            Some(ready_tx),
        )
        .await
        .map_err(|e| e.to_string())
    });
    let addr = tokio::time::timeout(Duration::from_secs(5), ready_rx)
        .await
        .map_err(|_| Error::Timeout("server bind"))?
        .map_err(|_| Error::Server("server exited before binding".to_string()))?;
    let url = format!("ws://{addr}");

    let stop = Arc::new(AtomicBool::new(false));
    let mut agents = Vec::with_capacity(config.agents);
    for i in 0..config.agents {
        agents.push(tokio::spawn(agent_task(
            url.clone(),
            format!("Probe-{}", i + 1),
            stop.clone(),
        )));
    }

    let (ws, _) = connect_async(&url).await.map_err(transport)?;
    let (mut sink, mut stream) = ws.split();
    let hello = ClientMessage::Hello {
        role: Role::Spectator,
        name: "Observer".to_string(),
    };
    sink.send(Message::Text(
        serde_json::to_string(&hello).map_err(transport)?,
    ))
    .await
    .map_err(transport)?;

    let mut observation = Observation::default();
    let deadline = Duration::from_secs_f64(config.max_ticks as f64 / TICKS_PER_SECOND + 15.0);
    let watch = async {
        while let Some(msg) = stream.next().await {
            let Ok(Message::Text(text)) = msg else {
                continue;
            };
            match serde_json::from_str::<ServerMessage>(&text) {
                Ok(ServerMessage::Snapshot(snapshot)) => {
                    observation.ingest_snapshot(&snapshot, text.len());
                }
                Ok(ServerMessage::Event(event)) => observation.ingest_event(event),
                _ => {}
            }
            if observation.rounds_completed() >= config.rounds {
                break;
            }
            let elapsed = observation
                .last_tick
                .saturating_sub(observation.first_tick.unwrap_or(0));
            if elapsed >= config.max_ticks {
                break;
            }
        }
    };
    let _ = tokio::time::timeout(deadline, watch).await;

    stop.store(true, Ordering::Relaxed);
    let _ = sink.close().await;
    let _ = shutdown_tx.send(());
    let _ = tokio::time::timeout(Duration::from_secs(5), server).await;
    for agent in agents {
        let _ = tokio::time::timeout(Duration::from_secs(2), agent).await;
    }
    let _ = TICK;
    let report = compute_report(&observation, config.agents);
    Ok((report, observation))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fragr_server::protocol::PlayerState;

    fn player(name: &str, id: Uuid, x: f32, z: f32, fired: bool) -> PlayerState {
        PlayerState {
            id,
            name: name.to_string(),
            x,
            y: 0.0,
            z,
            yaw: 0.0,
            hp: 100,
            armor: 0,
            just_fired: fired,
            behavior: None,
            score: 0,
            weapon: "Flechette".to_string(),
        }
    }

    fn snapshot(tick: u64, players: Vec<PlayerState>) -> Snapshot {
        Snapshot {
            tick,
            players,
            round_state: Some("Active".to_string()),
            round_time_left: None,
            frag_limit: Some(5),
            shot_results: Vec::new(),
            mode_name: "Contested Frequency".to_string(),
            playlist: "Arena Duel".to_string(),
            pressure: None,
            host_line: String::new(),
            mvp: None,
            mvp_frags: None,
            pickups: Vec::new(),
            map_id: 1,
            map_name: "Arena Duel".to_string(),
            episode_id: None,
            episode_title: None,
            episode_objective: None,
            episode_progress: None,
            episode_phase: None,
            jammer_dish: None,
        }
    }

    fn round_start(players: &[&str]) -> GameEvent {
        GameEvent::RoundStart {
            round_number: 1,
            frag_limit: Some(5),
            time_limit: None,
            players: players.iter().map(|p| p.to_string()).collect(),
            previous_winner: None,
            mode_name: String::new(),
            playlist: String::new(),
            host_line: String::new(),
        }
    }

    fn frag(killer: &str, victim: &str) -> GameEvent {
        GameEvent::Frag {
            killer: killer.to_string(),
            victim: victim.to_string(),
            killer_score: 1,
        }
    }

    #[test]
    fn reflex_action_targets_the_nearest_fighter() {
        let me = Uuid::new_v4();
        let near = Uuid::new_v4();
        let far = Uuid::new_v4();
        let snap = snapshot(
            1,
            vec![
                player("me", me, 0.0, 0.0, false),
                player("far", far, 30.0, 0.0, false),
                player("near", near, 5.0, 0.0, false),
            ],
        );
        let action = reflex_action(me, &snap);
        assert_eq!(action.look_at.unwrap().player_id, Some(near));
        assert!(action.forward);
        assert!(action.fire);
        let alone = snapshot(1, vec![player("me", me, 0.0, 0.0, false)]);
        assert!(reflex_action(me, &alone).look_at.is_none());
        assert!(reflex_action(Uuid::new_v4(), &snap).look_at.is_none());
        let close = snapshot(
            1,
            vec![
                player("me", me, 0.0, 0.0, false),
                player("near", near, 1.0, 0.0, false),
            ],
        );
        let action = reflex_action(me, &close);
        assert!(!action.forward);
        assert!(action.fire);
    }

    #[test]
    fn observation_tracks_idle_stuck_and_fire() {
        let a = Uuid::new_v4();
        let mut obs = Observation::default();
        obs.ingest_snapshot(&snapshot(10, vec![player("a", a, 0.0, 0.0, true)]), 100);
        obs.ingest_snapshot(&snapshot(11, vec![player("a", a, 0.0, 0.0, false)]), 100);
        obs.ingest_snapshot(&snapshot(12, vec![player("a", a, 0.0, 0.0, true)]), 100);
        obs.ingest_snapshot(&snapshot(13, vec![player("a", a, 1.0, 0.0, false)]), 100);
        obs.ingest_snapshot(&snapshot(14, vec![player("a", a, 1.0, 0.0, false)]), 100);
        let track = &obs.tracks["a"];
        assert_eq!(track.ticks_present, 5);
        assert_eq!(track.idle_ticks, 3);
        assert_eq!(track.stuck_max_ticks, 1, "firing or moving resets the run");
        assert_eq!(track.fire_ticks, 2);
        assert_eq!(track.weapon_fire_ticks["Flechette"], 2);
        assert_eq!(obs.first_tick, Some(10));
        assert_eq!(obs.last_tick, 14);
        assert_eq!(obs.snapshot_bytes, 500);
        // Standing still between rounds is not stuck and not idle.
        for tick in 15..30 {
            let mut ended = snapshot(tick, vec![player("a", a, 1.0, 0.0, false)]);
            ended.round_state = Some("Ended".to_string());
            obs.ingest_snapshot(&ended, 100);
        }
        let track = &obs.tracks["a"];
        assert_eq!(track.stuck_max_ticks, 1);
        assert_eq!(track.idle_ticks, 3);
        assert_eq!(track.ticks_present, 20);
    }

    #[test]
    fn report_counts_frags_spawn_deaths_and_gaps() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let mut obs = Observation::default();
        obs.ingest_snapshot(
            &snapshot(
                0,
                vec![
                    player("a", a, 0.0, 0.0, false),
                    player("b", b, 5.0, 0.0, false),
                ],
            ),
            200,
        );
        obs.ingest_event(round_start(&["a", "b"]));
        obs.ingest_snapshot(&snapshot(20, vec![player("a", a, 1.0, 0.0, false)]), 200);
        obs.ingest_event(frag("a", "b"));
        obs.ingest_snapshot(&snapshot(80, vec![player("a", a, 2.0, 0.0, false)]), 200);
        obs.ingest_event(GameEvent::Respawn {
            player: "b".to_string(),
        });
        obs.ingest_snapshot(&snapshot(200, vec![player("a", a, 3.0, 0.0, false)]), 200);
        obs.ingest_event(frag("b", "a"));
        obs.ingest_event(GameEvent::Pickup {
            player: "b".to_string(),
            player_id: b,
            kind: "weapon".to_string(),
            weapon: "Rail".to_string(),
            amount: None,
            pickup_id: "p1".to_string(),
        });
        obs.ingest_snapshot(&snapshot(1200, vec![player("a", a, 4.0, 0.0, false)]), 200);
        obs.ingest_event(GameEvent::RoundEnd {
            winner: Some("a".to_string()),
            reason: "frag_limit".to_string(),
            final_scores: Vec::new(),
            winner_score: Some(1),
            mvp: None,
            mvp_frags: None,
            host_line: String::new(),
        });
        let report = compute_report(&obs, 2);
        assert_eq!(report.rounds_completed, 1);
        assert_eq!(report.ticks, 1200);
        assert_eq!(report.frags, 2);
        assert_eq!(report.spawn_deaths, 1);
        assert_eq!(report.pickups, 1);
        assert_eq!(report.time_to_first_frag_s, Some(1.0));
        assert!((report.longest_gap_without_frag_s - 50.0).abs() < 1e-9);
        assert!((report.frags_per_minute - 2.0).abs() < 1e-9);
        assert!((report.host_beats_per_minute - 2.0).abs() < 1e-9);
        assert!((report.snapshot_bytes_per_tick - 200.0).abs() < 1e-9);
        assert_eq!(report.per_agent["a"].frags, 1);
        assert_eq!(report.per_agent["a"].deaths, 1);
        assert_eq!(report.per_agent["b"].spawn_deaths, 1);
        assert_eq!(report.per_agent["b"].deaths, 1);
        let empty = compute_report(&Observation::default(), 0);
        assert_eq!(empty.frags, 0);
        assert_eq!(empty.time_to_first_frag_s, None);
        assert_eq!(empty.snapshot_bytes_per_tick, 0.0);
    }

    #[test]
    fn thresholds_flag_the_frustrations() {
        let mut report = Report {
            agents: 4,
            rounds_completed: 1,
            frags: 20,
            frags_per_minute: 4.0,
            spawn_deaths: 1,
            ..Report::default()
        };
        assert!(check_thresholds(&report).is_empty());
        report.rounds_completed = 0;
        report.frags_per_minute = 0.5;
        report.spawn_deaths = 5;
        report.per_agent.insert(
            "Probe-1".to_string(),
            AgentReport {
                stuck_max_s: 6.0,
                ..AgentReport::default()
            },
        );
        let problems = check_thresholds(&report);
        assert_eq!(problems.len(), 4, "{problems:?}");
        assert!(problems.iter().any(|p| p.contains("no round completed")));
        assert!(problems
            .iter()
            .any(|p| p.contains("Probe-1 stuck for 6.0 s")));
        assert!(problems.iter().any(|p| p.contains("spawn deaths 5 of 20")));
        assert!(problems.iter().any(|p| p.contains("0.50 frags per minute")));
    }

    #[tokio::test]
    async fn reflex_agents_finish_a_round_in_process() {
        let config = Config {
            agents: 4,
            rounds: 1,
            frag_limit: 2,
            time_limit_ticks: 20 * 40,
            max_ticks: 20 * 90,
            ..Config::default()
        };
        let (report, observation) = run(config).await.expect("playtest run");
        assert!(observation.snapshots_seen > 0);
        assert_eq!(report.agents, 4);
        // The round ends by frag limit or time limit; how many frags land before that
        // depends on the machine (coverage builds run slower), so the thresholds are
        // enforced by the CI smoke step, not here.
        assert!(report.rounds_completed >= 1, "{report:?}");
        assert!(
            !report.per_agent.is_empty(),
            "{:?}",
            report.per_agent.keys()
        );
        assert!(
            report.per_agent.len() <= 4,
            "only the four probes should appear: {:?}",
            report.per_agent.keys()
        );
        assert!(report.snapshot_bytes_per_tick > 0.0);
    }
}

#[cfg(test)]
mod combat_tests {
    use super::*;
    use fragr_server::protocol::{PlayerState, ShotResult};

    fn player(name: &str, id: Uuid, x: f32, z: f32, weapon: &str) -> PlayerState {
        PlayerState {
            id,
            name: name.to_string(),
            x,
            y: 1.0,
            z,
            yaw: 0.0,
            hp: 100,
            armor: 0,
            just_fired: false,
            behavior: None,
            score: 0,
            weapon: weapon.to_string(),
        }
    }

    fn frame(tick: u64, players: Vec<PlayerState>, shots: Vec<ShotResult>) -> Snapshot {
        Snapshot {
            tick,
            players,
            round_state: Some("Active".to_string()),
            round_time_left: Some(60),
            frag_limit: Some(10),
            shot_results: shots,
            mode_name: "Contested Frequency".to_string(),
            playlist: "Arena Duel".to_string(),
            pressure: None,
            host_line: String::new(),
            mvp: None,
            mvp_frags: None,
            pickups: Vec::new(),
            map_id: 1,
            map_name: "Arena Duel".to_string(),
            episode_id: None,
            episode_title: None,
            episode_objective: None,
            episode_progress: None,
            episode_phase: None,
            jammer_dish: None,
        }
    }

    fn shot(shooter: Uuid, hit: bool, target: Option<Uuid>, damage: i32) -> ShotResult {
        ShotResult {
            shooter_id: shooter,
            shooter: "S".to_string(),
            hit,
            target_id: target,
            target: target.map(|_| "T".to_string()),
            damage,
            target_hp_after: hit.then_some(75),
        }
    }

    #[test]
    fn quantiles_describe_a_distribution_and_survive_nothing() {
        let empty = Quantiles::from_values(&[]);
        assert_eq!(empty.count, 0);
        assert_eq!(empty.p50, 0.0);
        let all_bad = Quantiles::from_values(&[f64::NAN, f64::INFINITY]);
        assert_eq!(all_bad.count, 0, "nonsense values never become a statistic");
        let q = Quantiles::from_values(&[5.0, 1.0, 3.0, 2.0, 4.0]);
        assert_eq!(q.count, 5);
        assert_eq!((q.min, q.p50, q.max), (1.0, 3.0, 5.0));
        assert!((q.mean - 3.0).abs() < 1e-9);
        assert!(q.p90 >= q.p50 && q.p90 <= q.max);
        let one = Quantiles::from_values(&[7.5]);
        assert_eq!((one.count, one.min, one.p50, one.max), (1, 7.5, 7.5, 7.5));
    }

    #[test]
    fn wilson_says_how_little_a_small_sample_means() {
        assert_eq!(wilson_interval(0, 0), (0.0, 1.0), "no trials, no claim");
        let (lo, hi) = wilson_interval(1, 1);
        assert!(
            lo > 0.0 && hi == 1.0,
            "one hit from one shot is not certainty: {lo} {hi}"
        );
        assert!(lo < 0.3, "and it is a weak claim: {lo}");
        let (lo_small, hi_small) = wilson_interval(5, 10);
        let (lo_big, hi_big) = wilson_interval(500, 1000);
        assert!(
            (hi_small - lo_small) > (hi_big - lo_big) * 5.0,
            "ten shots say far less than a thousand: {lo_small}..{hi_small} vs {lo_big}..{hi_big}"
        );
        for (lo, hi) in [wilson_interval(0, 20), wilson_interval(20, 20)] {
            assert!((0.0..=1.0).contains(&lo) && (0.0..=1.0).contains(&hi));
        }
    }

    #[test]
    fn shots_are_counted_by_weapon_with_the_distance_they_travelled() {
        let mut obs = Observation::default();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        obs.ingest_snapshot(
            &frame(
                1,
                vec![
                    player("A", a, 0.0, 0.0, "rail"),
                    player("B", b, 12.0, 0.0, "scatter"),
                ],
                vec![shot(a, true, Some(b), 75), shot(a, false, None, 0)],
            ),
            100,
        );
        let report = obs.combat_report();
        assert_eq!(report.shots, 2);
        assert_eq!(report.hits, 1);
        assert!((report.accuracy - 0.5).abs() < 1e-9);
        assert!(
            report.accuracy_lo < 0.5 && report.accuracy_hi > 0.5,
            "{report:?}"
        );
        let rail = &report.by_weapon["rail"];
        assert_eq!((rail.shots, rail.hits, rail.damage), (2, 1, 75));
        assert!(
            (rail.hit_distance.p50 - 12.0).abs() < 1e-4,
            "{:?}",
            rail.hit_distance
        );
        assert!(!report.by_weapon.contains_key("scatter"), "B never fired");
        obs.ingest_snapshot(
            &frame(
                2,
                vec![player("A", a, 0.0, 0.0, "rail")],
                vec![shot(Uuid::new_v4(), true, None, 10)],
            ),
            100,
        );
        assert_eq!(
            obs.combat_report().shots,
            2,
            "a shot from someone absent is dropped, not guessed at"
        );
    }

    #[test]
    fn time_to_kill_runs_from_the_first_damage_to_the_death() {
        let mut obs = Observation::default();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let scene = |tick: u64| {
            frame(
                tick,
                vec![
                    player("A", a, 0.0, 0.0, "flechette"),
                    player("B", b, 6.0, 0.0, "scatter"),
                ],
                vec![],
            )
        };
        let hit = || GameEvent::Hit {
            shooter: "A".into(),
            shooter_id: a,
            target: "B".into(),
            target_id: b,
            damage: 25,
            target_hp_after: 75,
        };
        obs.ingest_snapshot(&scene(100), 50);
        obs.ingest_event(hit());
        obs.ingest_snapshot(&scene(110), 50);
        obs.ingest_event(hit());
        obs.ingest_snapshot(&scene(120), 50);
        obs.ingest_event(GameEvent::Frag {
            killer: "A".into(),
            victim: "B".into(),
            killer_score: 1,
        });
        let report = obs.combat_report();
        assert_eq!(report.time_to_kill_s.count, 1);
        assert!(
            (report.time_to_kill_s.p50 - 1.0).abs() < 1e-6,
            "twenty ticks at twenty a second is one second, and the second hit did not restart it: {:?}",
            report.time_to_kill_s
        );
        let flechette = &report.by_weapon["flechette"];
        assert_eq!(flechette.kills, 1);
        assert!((flechette.kill_distance.p50 - 6.0).abs() < 1e-4);
        assert_eq!(
            report.kill_distance_buckets[1], 1,
            "six units falls in the second bucket"
        );
        assert_eq!(report.kill_distance_buckets.iter().sum::<u64>(), 1);

        obs.ingest_event(GameEvent::Respawn { player: "B".into() });
        obs.ingest_snapshot(&scene(200), 50);
        obs.ingest_event(GameEvent::Frag {
            killer: "A".into(),
            victim: "B".into(),
            killer_score: 2,
        });
        let report = obs.combat_report();
        assert_eq!(
            report.time_to_kill_s.count, 1,
            "a death with no opening hit is left untimed rather than timed wrongly"
        );
        assert_eq!(report.by_weapon["flechette"].kills, 2);
    }

    #[test]
    fn distance_buckets_hold_the_far_tail() {
        let obs = Observation {
            kill_distances: vec![0.0, 4.9, 5.0, 49.9, 50.0, 500.0],
            ..Observation::default()
        };
        let report = obs.combat_report();
        assert_eq!(report.kill_distance_buckets.len(), DISTANCE_BUCKETS);
        assert_eq!(report.kill_distance_buckets[0], 2, "under five units");
        assert_eq!(report.kill_distance_buckets[1], 1);
        assert_eq!(report.kill_distance_buckets[9], 1, "forty five to fifty");
        assert_eq!(
            report.kill_distance_buckets[DISTANCE_BUCKETS - 1],
            2,
            "fifty and beyond share the last bucket"
        );
    }

    #[test]
    fn an_empty_run_reports_nothing_rather_than_nonsense() {
        let report = Observation::default().combat_report();
        assert_eq!((report.shots, report.hits), (0, 0));
        assert_eq!(report.accuracy, 0.0);
        assert_eq!((report.accuracy_lo, report.accuracy_hi), (0.0, 1.0));
        assert_eq!(report.shots_per_kill, 0.0);
        assert_eq!(report.time_to_kill_s.count, 0);
        assert!(report.by_weapon.is_empty());
        assert_eq!(report.kill_distance_buckets.len(), DISTANCE_BUCKETS);
    }
}

#[cfg(test)]
mod weapon_choice_tests {
    use super::*;

    #[test]
    fn the_weapon_follows_the_range() {
        assert_eq!(weapon_for_distance(0.0), WeaponType::Scatter);
        assert_eq!(weapon_for_distance(9.9), WeaponType::Scatter);
        assert_eq!(weapon_for_distance(10.0), WeaponType::Flechette);
        assert_eq!(weapon_for_distance(30.0), WeaponType::Flechette);
        assert_eq!(weapon_for_distance(30.1), WeaponType::Rail);
        assert_eq!(weapon_from_wire("Rail"), Some(WeaponType::Rail));
        assert_eq!(weapon_from_wire("scatter"), Some(WeaponType::Scatter));
        assert_eq!(weapon_from_wire("bfg"), None);
    }
}
