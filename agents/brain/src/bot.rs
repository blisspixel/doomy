//! The play loop. One WebSocket to the server, a 20 Hz controller that always
//! has an action ready, and a slower decision cadence that asks the brain for
//! intent without ever blocking the controller. At most one decision is in
//! flight; a slow answer is simply late, not queued.

use crate::budget::Budget;
use crate::decision::{plan_from_answers, tactical_questions, Gate, Question};
use crate::plan::{fallback_plan, micro_action, Plan, Source, Stance};
use crate::provider::{decide, decision_request, Provider, Transport};
use crate::telemetry::{observe, RecentHits, Telemetry};
use crate::Error;
use fragr_server::protocol::{
    ClientMessage, GameEvent, Role, ServerMessage, SetDisplayBehavior, Snapshot,
};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

/// The controller cadence: one action per server tick.
pub const MICRO_INTERVAL: Duration = Duration::from_millis(50);
/// Decisions per second are clamped to this range.
pub const MIN_DECISION_HZ: f64 = 0.1;
pub const MAX_DECISION_HZ: f64 = 5.0;
/// Each retryable failure doubles the decision interval, up to this many times.
pub const MAX_BACKOFF_LEVEL: u32 = 4;
/// Consecutive unreadable answers before the brain is switched off for the run.
pub const MAX_CONSECUTIVE_MALFORMED: u32 = 3;
/// How long to wait for the server to accept the socket.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
/// How long one outbound message may take before the socket is given up on.
pub const SEND_TIMEOUT: Duration = Duration::from_secs(2);
/// How long to wait for an in-flight decision after the loop ends, so its
/// charge lands in the ledger before the summary is read.
pub const DRAIN_TIMEOUT: Duration = Duration::from_secs(5);

/// The decision interval after `level` consecutive retryable failures.
pub fn backoff_interval(base: Duration, level: u32) -> Duration {
    base * 2u32.pow(level.min(MAX_BACKOFF_LEVEL))
}

/// Failures worth slowing down for: transport trouble, timeouts, rate limits,
/// and server errors. TypeSafe says to back off rather than retry at once; the
/// bot never retries inside a cycle, it just asks less often until one lands.
pub fn is_retryable(err: &Error) -> bool {
    matches!(
        err,
        Error::Transport(_)
            | Error::Api {
                status: 408 | 429 | 500..=599,
                ..
            }
    )
}

/// Failures that will not get better by asking again: a bad key, a wrong
/// model id, a rejected request shape. One of these switches the brain off
/// for the rest of the run instead of billing a phantom charge every cycle.
pub fn is_fatal(err: &Error) -> bool {
    matches!(
        err,
        Error::Api {
            status: 400 | 401 | 403 | 404 | 422,
            ..
        } | Error::MissingApiKey(_)
            | Error::InvalidArgument(_)
    )
}

/// Whether a paid decision is worth asking for right now. Dead fighters and
/// fighters outside an active round only need local rules.
pub fn brain_worth_asking(telemetry: &Telemetry) -> bool {
    telemetry.hp > 0 && telemetry.round_state == "active"
}

/// Send one text frame with a bound on how long the socket may stall.
async fn send_text<S>(sink: &mut S, text: String) -> bool
where
    S: futures_util::Sink<Message> + Unpin,
{
    matches!(
        tokio::time::timeout(SEND_TIMEOUT, sink.send(Message::Text(text))).await,
        Ok(Ok(()))
    )
}

#[derive(Debug, Clone)]
pub struct BotConfig {
    pub server_url: String,
    pub name: String,
    pub provider: Provider,
    pub model: String,
    /// Required for paid providers; ignored for `Provider::Local`.
    pub api_key: Option<String>,
    pub decision_hz: f64,
    pub gate: Gate,
    /// Leave the match after this long; `None` plays until the socket closes.
    pub max_seconds: Option<u64>,
}

/// What one run did, printed as JSON when it ends.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct BotSummary {
    pub name: String,
    pub provider: String,
    pub model: String,
    pub snapshots: u64,
    pub actions_sent: u64,
    pub decisions_remote: u64,
    pub decisions_low_confidence: u64,
    pub decisions_failed: u64,
    pub decisions_local: u64,
    pub budget_refusals: u64,
    /// Times the decision interval was doubled after a retryable failure.
    pub backoffs: u64,
    /// Failures that switched the brain off for the rest of the run.
    pub fatal_failures: u64,
    /// Why the brain was switched off, if it was.
    pub brain_disabled: Option<String>,
    pub frags: u32,
    pub deaths: u32,
    pub run_usd: f64,
    pub total_usd: f64,
    pub last_plan: Option<Plan>,
    pub last_state: Option<String>,
    pub decision_latency: LatencyStats,
}

enum Outcome {
    Decided(Plan),
    Refused(Error, Plan),
    Failed(Error, Plan),
}

/// One decision on its way back: the answer channel and the task behind it.
type InFlight = (oneshot::Receiver<(Outcome, u64)>, JoinHandle<()>);

/// Round-trip time of sent decisions, in milliseconds.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct LatencyStats {
    pub samples: u64,
    pub min_ms: u64,
    pub max_ms: u64,
    pub mean_ms: f64,
}

impl LatencyStats {
    pub fn push(&mut self, ms: u64) {
        if self.samples == 0 {
            self.min_ms = ms;
            self.max_ms = ms;
        } else {
            self.min_ms = self.min_ms.min(ms);
            self.max_ms = self.max_ms.max(ms);
        }
        let total = self.mean_ms * self.samples as f64 + ms as f64;
        self.samples += 1;
        self.mean_ms = total / self.samples as f64;
    }
}

fn transport_err<E: std::fmt::Display>(err: E) -> Error {
    Error::Transport(err.to_string())
}

/// Wire JSON for SetDisplayBehavior when stance changes (including first publish).
fn display_behavior_wire(published: &mut Option<Stance>, stance: Stance) -> Option<String> {
    if published.as_ref() == Some(&stance) {
        return None;
    }
    *published = Some(stance);
    serde_json::to_string(&ClientMessage::SetDisplayBehavior(SetDisplayBehavior {
        behavior: stance.name().to_string(),
    }))
    .ok()
}

fn lock(budget: &Mutex<Budget>) -> std::sync::MutexGuard<'_, Budget> {
    budget
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[allow(clippy::too_many_arguments)]
fn spawn_decision(
    transport: Arc<dyn Transport>,
    budget: Arc<Mutex<Budget>>,
    questions: Arc<BTreeMap<String, Question>>,
    provider: Provider,
    model: String,
    api_key: String,
    state: Value,
    fallback: Plan,
    gate: Gate,
    tx: oneshot::Sender<(Outcome, u64)>,
) -> JoinHandle<()> {
    tokio::task::spawn_blocking(move || {
        let started = std::time::Instant::now();
        let result = decision_request(provider, &model, &api_key, &state, &questions)
            .and_then(|request| decide(transport.as_ref(), &budget, provider, &model, &request))
            .map(|decision| plan_from_answers(&decision.response.answers, &gate, &fallback));
        let outcome = match result {
            Ok(plan) => Outcome::Decided(plan),
            Err(err @ Error::Budget(_)) => Outcome::Refused(
                err,
                Plan {
                    source: Source::Budget,
                    ..fallback
                },
            ),
            Err(err) => Outcome::Failed(err, fallback),
        };
        let _ = tx.send((outcome, started.elapsed().as_millis() as u64));
    })
}

/// Join the server as an agent and play until the socket closes, `stop` is
/// set, or `max_seconds` passes. Never returns early on brain trouble.
pub async fn run_bot(
    config: BotConfig,
    transport: Arc<dyn Transport>,
    budget: Arc<Mutex<Budget>>,
    stop: Arc<AtomicBool>,
) -> Result<BotSummary, Error> {
    if config.provider.is_paid() && config.api_key.as_deref().unwrap_or("").trim().is_empty() {
        return Err(Error::MissingApiKey(config.provider.key_names().join(", ")));
    }
    let (ws, _) = tokio::time::timeout(CONNECT_TIMEOUT, connect_async(&config.server_url))
        .await
        .map_err(|_| Error::Transport(format!("connect to {} timed out", config.server_url)))?
        .map_err(transport_err)?;
    let (mut sink, mut stream) = ws.split();
    let hello = ClientMessage::Hello {
        role: Role::Agent,
        name: config.name.clone(),
    };
    if !send_text(
        &mut sink,
        serde_json::to_string(&hello).map_err(transport_err)?,
    )
    .await
    {
        return Err(Error::Transport(
            "hello was not accepted in time".to_string(),
        ));
    }

    let mut published_stance: Option<Stance> = None;
    let mut plan = Plan::default();
    if let Some(wire) = display_behavior_wire(&mut published_stance, plan.stance) {
        if !send_text(&mut sink, wire).await {
            return Err(Error::Transport(
                "first stance was not accepted in time".to_string(),
            ));
        }
    }

    let mut summary = BotSummary {
        name: config.name.clone(),
        provider: config.provider.name().to_string(),
        model: config.model.clone(),
        ..BotSummary::default()
    };
    let mut me: Option<Uuid> = None;
    let mut my_name = config.name.clone();
    let mut last: Option<Snapshot> = None;
    let mut hits = RecentHits::default();
    let mut paid_enabled = config.provider.is_paid();
    let questions = Arc::new(tactical_questions());
    let hz = if config.decision_hz.is_finite() {
        config.decision_hz.clamp(MIN_DECISION_HZ, MAX_DECISION_HZ)
    } else {
        3.0
    };
    let mut micro = tokio::time::interval(MICRO_INTERVAL);
    micro.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let base_interval = Duration::from_secs_f64(1.0 / hz);
    let mut backoff_level = 0u32;
    let mut macro_tick = tokio::time::interval(base_interval);
    macro_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let deadline = config
        .max_seconds
        .and_then(|s| tokio::time::Instant::now().checked_add(Duration::from_secs(s)));
    let mut inflight: Option<InFlight> = None;
    let mut consecutive_malformed = 0u32;

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        if let Some(deadline) = deadline {
            if tokio::time::Instant::now() >= deadline {
                break;
            }
        }
        tokio::select! {
            msg = stream.next() => match msg {
                Some(Ok(Message::Text(text))) => match serde_json::from_str::<ServerMessage>(&text) {
                    Ok(ServerMessage::Welcome { player_id, .. }) => me = player_id,
                    Ok(ServerMessage::Snapshot(snapshot)) => {
                        summary.snapshots += 1;
                        if let Some(id) = me {
                            if let Some(p) = snapshot.players.iter().find(|p| p.id == id) {
                                my_name = p.name.clone();
                            }
                        }
                        last = Some(snapshot);
                    }
                    Ok(ServerMessage::Event(event)) => {
                        let tick = last.as_ref().map(|s| s.tick).unwrap_or(0);
                        if let Some(id) = me {
                            hits.ingest(id, tick, &event);
                        }
                        if let GameEvent::Frag { killer, victim, .. } = &event {
                            if *killer == my_name {
                                summary.frags += 1;
                            }
                            if *victim == my_name {
                                summary.deaths += 1;
                            }
                        }
                    }
                    // The brain does not predict, so an ack is nothing to act on.
                    Ok(ServerMessage::Ack { .. }) => {}
                    Ok(ServerMessage::Error { code, message }) => {
                        tracing::warn!("server rejected: {code}: {message}");
                    }
                    Err(_) => {}
                },
                Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                Some(Ok(_)) => {}
            },
            _ = micro.tick() => {
                if let (Some(id), Some(snapshot)) = (me, last.as_ref()) {
                    let action = micro_action(&plan, id, snapshot);
                    let text = serde_json::to_string(&ClientMessage::Action(action))
                        .map_err(transport_err)?;
                    if !send_text(&mut sink, text).await {
                        break;
                    }
                    summary.actions_sent += 1;
                }
            },
            _ = macro_tick.tick(), if inflight.is_none() => {
                let Some(id) = me else { continue };
                let Some(snapshot) = last.as_ref() else { continue };
                let Some(telemetry) = observe(id, snapshot, &mut hits) else { continue };
                summary.last_state = Some(telemetry.render());
                if !paid_enabled || !brain_worth_asking(&telemetry) {
                    let source = if config.provider.is_paid() && !paid_enabled {
                        Source::Budget
                    } else {
                        Source::Local
                    };
                    plan = fallback_plan(&telemetry, source);
                    summary.decisions_local += 1;
                    if let Some(wire) = display_behavior_wire(&mut published_stance, plan.stance) {
                        if !send_text(&mut sink, wire).await {
                            break;
                        }
                    }
                    continue;
                }
                let (tx, rx) = oneshot::channel();
                let handle = spawn_decision(
                    transport.clone(),
                    budget.clone(),
                    questions.clone(),
                    config.provider,
                    config.model.clone(),
                    config.api_key.clone().unwrap_or_default(),
                    telemetry.state_object(),
                    fallback_plan(&telemetry, Source::Failure),
                    config.gate,
                    tx,
                );
                inflight = Some((rx, handle));
            },
            answer = async { (&mut inflight.as_mut().expect("guarded by the branch condition").0).await }, if inflight.is_some() => {
                inflight = None;
                let outcome = match answer {
                    Ok((outcome, ms)) => {
                        if matches!(outcome, Outcome::Decided(_) | Outcome::Failed(..)) {
                            summary.decision_latency.push(ms);
                        }
                        outcome
                    }
                    Err(_) => {
                        // The blocking task ended without answering (a panic).
                        summary.decisions_failed += 1;
                        tracing::warn!("decision task ended without an answer; local rules this cycle");
                        continue;
                    }
                };
                match outcome {
                    Outcome::Decided(decided) => {
                        consecutive_malformed = 0;
                        match decided.source {
                            Source::Remote => summary.decisions_remote += 1,
                            _ => summary.decisions_low_confidence += 1,
                        }
                        plan = decided;
                        if backoff_level > 0 {
                            backoff_level = 0;
                            macro_tick = tokio::time::interval_at(
                                tokio::time::Instant::now() + base_interval,
                                base_interval,
                            );
                            macro_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                        }
                        if let Some(wire) =
                            display_behavior_wire(&mut published_stance, plan.stance)
                        {
                            if !send_text(&mut sink, wire).await {
                                break;
                            }
                        }
                    }
                    Outcome::Refused(err, fallback) => {
                        summary.budget_refusals += 1;
                        if paid_enabled {
                            tracing::warn!("brain off for the rest of the run: {err}");
                            summary.brain_disabled = Some(err.to_string());
                        }
                        paid_enabled = false;
                        plan = fallback;
                        if let Some(wire) =
                            display_behavior_wire(&mut published_stance, plan.stance)
                        {
                            if !send_text(&mut sink, wire).await {
                                break;
                            }
                        }
                    }
                    Outcome::Failed(err, fallback) => {
                        summary.decisions_failed += 1;
                        let unreadable = matches!(err, Error::Malformed(_) | Error::Io(_));
                        consecutive_malformed = if unreadable { consecutive_malformed + 1 } else { 0 };
                        if is_fatal(&err) || consecutive_malformed >= MAX_CONSECUTIVE_MALFORMED {
                            summary.fatal_failures += 1;
                            summary.brain_disabled = Some(err.to_string());
                            paid_enabled = false;
                            tracing::warn!("brain off for the rest of the run: {err}");
                        } else if is_retryable(&err) && backoff_level < MAX_BACKOFF_LEVEL {
                            backoff_level += 1;
                            summary.backoffs += 1;
                            let period = backoff_interval(base_interval, backoff_level);
                            macro_tick = tokio::time::interval_at(tokio::time::Instant::now() + period, period);
                            macro_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                            tracing::warn!("brain call failed, backing off to {period:?}: {err}");
                        } else {
                            tracing::warn!("brain call failed, local rules this cycle: {err}");
                        }
                        plan = fallback;
                        if let Some(wire) =
                            display_behavior_wire(&mut published_stance, plan.stance)
                        {
                            if !send_text(&mut sink, wire).await {
                                break;
                            }
                        }
                    }
                }
            },
        }
    }
    let _ = sink.close().await;
    // Let an in-flight decision finish so its charge is in the ledger before
    // the totals are read; a panic or a hang is bounded, not fatal.
    if let Some((_, handle)) = inflight.take() {
        let _ = tokio::time::timeout(DRAIN_TIMEOUT, handle).await;
    }
    {
        let guard = lock(&budget);
        summary.run_usd = guard.run_usd();
        summary.total_usd = guard.total_usd();
    }
    summary.last_plan = Some(plan);
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::{Caps, Pricing, Refusal};
    use crate::provider::fakes::{push_answers, FakeTransport};
    use crate::provider::HttpResponse;
    use fragr_server::run::{run_server, ServerOptions};
    use fragr_server::sim::{MapKind, MatchConfig};

    async fn boot_server(bots: usize) -> (String, tokio::sync::oneshot::Sender<()>) {
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
        let options = ServerOptions {
            bind: "127.0.0.1:0".to_string(),
            bots,
            map: MapKind::default(),
            map_rotate: false,
            solo_broadcast: false,
            match_config: Some(MatchConfig {
                warmup_ticks: 1,
                boss_spawn_ticks: None,
                compliance_ping_ticks: None,
                ..MatchConfig::default()
            }),
            seed: 1,
            status_every_s: 0,
        };
        tokio::spawn(async move {
            let _ = run_server(
                options,
                async move {
                    let _ = shutdown_rx.await;
                },
                Some(ready_tx),
            )
            .await;
        });
        let addr = tokio::time::timeout(Duration::from_secs(5), ready_rx)
            .await
            .expect("bind in time")
            .expect("server bound");
        (format!("ws://{addr}"), shutdown_tx)
    }

    fn config(url: &str, provider: Provider, seconds: u64) -> BotConfig {
        BotConfig {
            server_url: url.to_string(),
            name: "Brain-1".to_string(),
            provider,
            model: provider.default_model().to_string(),
            api_key: provider.is_paid().then(|| "sk_test".to_string()),
            decision_hz: 5.0,
            gate: Gate::default(),
            max_seconds: Some(seconds),
        }
    }

    fn budget(run_usd: f64) -> Arc<Mutex<Budget>> {
        Arc::new(Mutex::new(Budget::new(
            Caps {
                run_usd,
                total_usd: None,
                run_calls: None,
            },
            Pricing::default(),
        )))
    }

    #[tokio::test]
    async fn a_bad_key_switches_the_brain_off_after_one_call() {
        let (url, shutdown) = boot_server(1).await;
        let transport = Arc::new(FakeTransport::new(vec![Ok(HttpResponse {
            status: 401,
            body: br#"{"error":{"message":"No auth"}}"#.to_vec(),
        })]));
        let budget = budget(1.0);
        let summary = run_bot(
            config(&url, Provider::OpenRouter, 2),
            transport.clone(),
            budget.clone(),
            Arc::new(AtomicBool::new(false)),
        )
        .await
        .expect("bot runs");
        assert_eq!(summary.decisions_failed, 1, "{summary:?}");
        assert_eq!(summary.fatal_failures, 1);
        assert_eq!(transport.calls(), 1, "no phantom charges after a 401");
        assert!(summary.brain_disabled.as_deref().unwrap().contains("401"));
        assert!(
            summary.decisions_local >= 2,
            "local rules take over: {summary:?}"
        );
        assert_eq!(budget.lock().unwrap().run_calls(), 1);
        let _ = shutdown.send(());
    }

    #[tokio::test]
    async fn unreadable_answers_switch_the_brain_off_after_three() {
        let (url, shutdown) = boot_server(1).await;
        let transport = Arc::new(FakeTransport::new(vec![Ok(HttpResponse {
            status: 200,
            body: b"not json".to_vec(),
        })]));
        let summary = run_bot(
            config(&url, Provider::Typesafe, 3),
            transport.clone(),
            budget(1.0),
            Arc::new(AtomicBool::new(false)),
        )
        .await
        .expect("bot runs");
        assert_eq!(
            transport.calls(),
            MAX_CONSECUTIVE_MALFORMED as usize,
            "{summary:?}"
        );
        assert_eq!(summary.fatal_failures, 1);
        assert!(summary.brain_disabled.is_some());
        let _ = shutdown.send(());
    }

    #[test]
    fn brain_is_only_asked_while_alive_and_active() {
        use crate::telemetry::fixtures::{player, snapshot};
        let me = Uuid::new_v4();
        let mut hits = RecentHits::default();
        let alive = snapshot(1, vec![player("me", me, 0.0, 0.0, 50, "rail")], vec![]);
        assert!(brain_worth_asking(&observe(me, &alive, &mut hits).unwrap()));
        let dead = snapshot(1, vec![player("me", me, 0.0, 0.0, 0, "rail")], vec![]);
        assert!(!brain_worth_asking(&observe(me, &dead, &mut hits).unwrap()));
        let mut warmup = alive.clone();
        warmup.round_state = Some("Warmup".to_string());
        assert!(!brain_worth_asking(
            &observe(me, &warmup, &mut hits).unwrap()
        ));
        let mut ended = alive.clone();
        ended.round_state = Some("Ended".to_string());
        assert!(!brain_worth_asking(
            &observe(me, &ended, &mut hits).unwrap()
        ));
        assert!(is_fatal(&Error::Api {
            status: 401,
            message: String::new()
        }));
        assert!(is_fatal(&Error::Api {
            status: 400,
            message: String::new()
        }));
        assert!(!is_fatal(&Error::Api {
            status: 503,
            message: String::new()
        }));
        assert!(!is_fatal(&Error::Malformed(String::new())));
    }

    #[test]
    fn backoff_doubles_and_caps() {
        let base = Duration::from_millis(200);
        assert_eq!(backoff_interval(base, 0), base);
        assert_eq!(backoff_interval(base, 1), Duration::from_millis(400));
        assert_eq!(backoff_interval(base, 4), Duration::from_millis(3200));
        assert_eq!(backoff_interval(base, 9), Duration::from_millis(3200));
        assert!(is_retryable(&Error::Transport("timeout".into())));
        assert!(is_retryable(&Error::Api {
            status: 429,
            message: String::new()
        }));
        assert!(is_retryable(&Error::Api {
            status: 529,
            message: String::new()
        }));
        assert!(!is_retryable(&Error::Api {
            status: 401,
            message: String::new()
        }));
        assert!(!is_retryable(&Error::Malformed(String::new())));
    }

    #[test]
    fn latency_stats_track_min_max_mean() {
        let mut stats = LatencyStats::default();
        assert_eq!(stats.samples, 0);
        stats.push(400);
        stats.push(200);
        stats.push(600);
        assert_eq!(stats.samples, 3);
        assert_eq!(stats.min_ms, 200);
        assert_eq!(stats.max_ms, 600);
        assert!((stats.mean_ms - 400.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn local_provider_plays_for_free() {
        let (url, shutdown) = boot_server(2).await;
        let transport = Arc::new(FakeTransport::ok(push_answers()));
        let stop = Arc::new(AtomicBool::new(false));
        let summary = run_bot(
            config(&url, Provider::Local, 3),
            transport.clone(),
            budget(0.0),
            stop,
        )
        .await
        .expect("bot runs");
        assert!(summary.snapshots > 10, "{summary:?}");
        assert!(summary.actions_sent > 10, "{summary:?}");
        assert!(summary.decisions_local >= 3, "{summary:?}");
        assert_eq!(summary.decisions_remote, 0);
        assert_eq!(transport.calls(), 0, "local never touches the transport");
        assert_eq!(summary.run_usd, 0.0);
        assert_eq!(summary.provider, "local");
        assert_eq!(summary.last_plan.as_ref().unwrap().source, Source::Local);
        assert!(summary.last_state.as_ref().unwrap().starts_with("SELF hp="));
        let _ = shutdown.send(());
    }

    #[tokio::test]
    async fn remote_answers_drive_the_plan_and_the_ledger() {
        let (url, shutdown) = boot_server(2).await;
        let transport = Arc::new(FakeTransport::ok(push_answers()));
        let budget = budget(1.0);
        let stop = Arc::new(AtomicBool::new(false));
        let summary = run_bot(
            config(&url, Provider::Typesafe, 3),
            transport.clone(),
            budget.clone(),
            stop,
        )
        .await
        .expect("bot runs");
        assert!(summary.decisions_remote >= 3, "{summary:?}");
        assert_eq!(summary.decisions_local, 0);
        assert_eq!(summary.budget_refusals, 0);
        assert!(transport.calls() >= 3);
        assert!(summary.run_usd > 0.0);
        let plan = summary.last_plan.as_ref().unwrap();
        assert_eq!(plan.source, Source::Remote);
        assert_eq!(plan.stance, crate::plan::Stance::PushEnemy);
        assert!(budget.lock().unwrap().run_calls() as usize >= 3);
        assert!(summary.decision_latency.samples >= 3);
        assert!(summary.decision_latency.max_ms >= summary.decision_latency.min_ms);
        let sent = transport.last_request.lock().unwrap().clone().unwrap();
        let state = sent.body.unwrap()["state"].clone();
        assert!(state.is_object(), "the brain gets an object: {state}");
        assert!(state["self"]["health"].is_string());
        let _ = shutdown.send(());
    }

    #[tokio::test]
    async fn budget_refusal_switches_to_local_rules_once() {
        let (url, shutdown) = boot_server(1).await;
        let transport = Arc::new(FakeTransport::ok(push_answers()));
        let stop = Arc::new(AtomicBool::new(false));
        let summary = run_bot(
            config(&url, Provider::OpenRouter, 3),
            transport.clone(),
            budget(0.0),
            stop,
        )
        .await
        .expect("bot runs");
        assert_eq!(summary.budget_refusals, 1, "{summary:?}");
        assert!(summary.decisions_local >= 2, "local rules take over");
        assert_eq!(transport.calls(), 0, "nothing is sent without a cap");
        assert_eq!(summary.last_plan.as_ref().unwrap().source, Source::Budget);
        assert_eq!(summary.run_usd, 0.0);
        let _ = shutdown.send(());
    }

    #[tokio::test]
    async fn failed_calls_fall_back_and_still_count() {
        let (url, shutdown) = boot_server(1).await;
        let transport = Arc::new(FakeTransport::new(vec![Ok(HttpResponse {
            status: 503,
            body: br#"{"error":{"message":"overloaded"}}"#.to_vec(),
        })]));
        let budget = budget(1.0);
        let stop = Arc::new(AtomicBool::new(false));
        let summary = run_bot(
            config(&url, Provider::Typesafe, 2),
            transport.clone(),
            budget.clone(),
            stop,
        )
        .await
        .expect("bot runs");
        assert!(summary.decisions_failed >= 2, "{summary:?}");
        assert_eq!(summary.decisions_remote, 0);
        assert_eq!(summary.last_plan.as_ref().unwrap().source, Source::Failure);
        assert!(
            summary.backoffs >= 1,
            "a 503 slows the cadence: {summary:?}"
        );
        assert!(
            summary.decisions_failed <= 4,
            "backoff never fires at once: {summary:?}"
        );
        assert_eq!(summary.fatal_failures, 0);
        assert!(summary.brain_disabled.is_none());
        assert!(
            summary.run_usd > 0.0,
            "sent calls are charged at the estimate"
        );
        let _ = shutdown.send(());
    }

    #[tokio::test]
    async fn stop_flag_and_missing_key_and_bad_url() {
        let (url, shutdown) = boot_server(0).await;
        let transport = Arc::new(FakeTransport::ok(push_answers()));
        let stop = Arc::new(AtomicBool::new(true));
        let summary = run_bot(
            config(&url, Provider::Local, 30),
            transport.clone(),
            budget(0.0),
            stop,
        )
        .await
        .expect("bot exits on stop");
        assert_eq!(summary.actions_sent, 0);
        let mut no_key = config(&url, Provider::Typesafe, 1);
        no_key.api_key = None;
        assert!(matches!(
            run_bot(
                no_key,
                transport.clone(),
                budget(1.0),
                Arc::new(AtomicBool::new(false))
            )
            .await,
            Err(Error::MissingApiKey(_))
        ));
        // A port nobody listens on: bind, learn the number, release it.
        let free = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = free.local_addr().unwrap().port();
        drop(free);
        let bad = config(&format!("ws://127.0.0.1:{port}"), Provider::Local, 1);
        let started = std::time::Instant::now();
        assert!(matches!(
            run_bot(
                bad,
                transport,
                budget(0.0),
                Arc::new(AtomicBool::new(false))
            )
            .await,
            Err(Error::Transport(_))
        ));
        assert!(started.elapsed() < CONNECT_TIMEOUT + Duration::from_secs(1));
        let _ = shutdown.send(());
        let _ = Refusal::NoCap;
    }

    #[test]
    fn display_behavior_wire_only_on_stance_change() {
        let mut published = None;
        let first = display_behavior_wire(&mut published, Stance::HoldAngle).unwrap();
        assert!(first.contains("set_display_behavior"));
        assert!(first.contains("hold_angle"));
        assert!(display_behavior_wire(&mut published, Stance::HoldAngle).is_none());
        let next = display_behavior_wire(&mut published, Stance::PushEnemy).unwrap();
        assert!(next.contains("push_enemy"));
        assert_eq!(published, Some(Stance::PushEnemy));
    }

    #[tokio::test]
    async fn brain_publishes_stance_chip_on_join() {
        let (url, shutdown) = boot_server(0).await;
        let transport = Arc::new(FakeTransport::ok(push_answers()));
        let stop = Arc::new(AtomicBool::new(false));
        let bot = tokio::spawn(run_bot(
            config(&url, Provider::Local, 2),
            transport,
            budget(0.0),
            stop.clone(),
        ));
        // Spectator reads snapshots until Brain-1 shows a stance chip.
        let (ws, _) = connect_async(&url).await.expect("spec connect");
        let (mut sink, mut stream) = ws.split();
        sink.send(Message::Text(
            serde_json::to_string(&ClientMessage::Hello {
                role: Role::Spectator,
                name: "Spec".into(),
            })
            .unwrap(),
        ))
        .await
        .unwrap();
        let mut saw = false;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        while tokio::time::Instant::now() < deadline {
            let Some(Ok(Message::Text(text))) = stream.next().await else {
                break;
            };
            let Ok(ServerMessage::Snapshot(snap)) = serde_json::from_str(&text) else {
                continue;
            };
            if let Some(p) = snap.players.iter().find(|p| p.name == "Brain-1") {
                if p.behavior.as_deref() == Some("hold_angle")
                    || p.behavior.as_deref() == Some("push_enemy")
                    || p.behavior.as_deref() == Some("fall_back_heal")
                    || p.behavior.as_deref() == Some("kite_distance")
                {
                    saw = true;
                    break;
                }
            }
        }
        stop.store(true, Ordering::Relaxed);
        let _ = bot.await;
        let _ = shutdown.send(());
        assert!(saw, "spectator never saw Brain-1 stance chip");
    }
}
