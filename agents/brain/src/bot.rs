//! The play loop. One WebSocket to the server, a 20 Hz controller that always
//! has an action ready, and a slower decision cadence that asks the brain for
//! intent without ever blocking the controller. At most one decision is in
//! flight; a slow answer is simply late, not queued.

use crate::budget::Budget;
use crate::decision::{plan_from_answers, tactical_questions, Gate, Question};
use crate::plan::{fallback_plan, micro_action, Plan, Source};
use crate::provider::{decide, decision_request, Provider, Transport};
use crate::telemetry::{observe, RecentHits};
use crate::Error;
use fragr_server::protocol::{ClientMessage, GameEvent, Role, ServerMessage, Snapshot};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

/// The controller cadence: one action per server tick.
pub const MICRO_INTERVAL: Duration = Duration::from_millis(50);
/// Decisions per second are clamped to this range.
pub const MIN_DECISION_HZ: f64 = 0.1;
pub const MAX_DECISION_HZ: f64 = 5.0;
/// Each retryable failure doubles the decision interval, up to this many times.
pub const MAX_BACKOFF_LEVEL: u32 = 4;

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
    tx: mpsc::UnboundedSender<(Outcome, u64)>,
) {
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
    });
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
    let (ws, _) = connect_async(&config.server_url)
        .await
        .map_err(transport_err)?;
    let (mut sink, mut stream) = ws.split();
    let hello = ClientMessage::Hello {
        role: Role::Agent,
        name: config.name.clone(),
    };
    sink.send(Message::Text(
        serde_json::to_string(&hello).map_err(transport_err)?,
    ))
    .await
    .map_err(transport_err)?;

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
    let mut plan = Plan::default();
    let mut paid_enabled = config.provider.is_paid();
    let questions = Arc::new(tactical_questions());
    let hz = config.decision_hz.clamp(MIN_DECISION_HZ, MAX_DECISION_HZ);
    let mut micro = tokio::time::interval(MICRO_INTERVAL);
    micro.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let base_interval = Duration::from_secs_f64(1.0 / hz);
    let mut backoff_level = 0u32;
    let mut macro_tick = tokio::time::interval(base_interval);
    macro_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let deadline = config
        .max_seconds
        .map(|s| tokio::time::Instant::now() + Duration::from_secs(s));
    let (tx, mut rx) = mpsc::unbounded_channel::<(Outcome, u64)>();
    let mut inflight = false;

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
                    if sink.send(Message::Text(text)).await.is_err() {
                        break;
                    }
                    summary.actions_sent += 1;
                }
            },
            _ = macro_tick.tick(), if !inflight => {
                let Some(id) = me else { continue };
                let Some(snapshot) = last.as_ref() else { continue };
                let Some(telemetry) = observe(id, snapshot, &mut hits) else { continue };
                summary.last_state = Some(telemetry.render());
                if !paid_enabled {
                    let source = if config.provider.is_paid() { Source::Budget } else { Source::Local };
                    plan = fallback_plan(&telemetry, source);
                    summary.decisions_local += 1;
                    continue;
                }
                inflight = true;
                spawn_decision(
                    transport.clone(),
                    budget.clone(),
                    questions.clone(),
                    config.provider,
                    config.model.clone(),
                    config.api_key.clone().unwrap_or_default(),
                    telemetry.state_object(),
                    fallback_plan(&telemetry, Source::Failure),
                    config.gate,
                    tx.clone(),
                );
            },
            outcome = rx.recv() => {
                inflight = false;
                if let Some((Outcome::Decided(_) | Outcome::Failed(..), ms)) = &outcome {
                    summary.decision_latency.push(*ms);
                }
                match outcome.map(|(outcome, _)| outcome) {
                    Some(Outcome::Decided(decided)) => {
                        match decided.source {
                            Source::Remote => summary.decisions_remote += 1,
                            _ => summary.decisions_low_confidence += 1,
                        }
                        plan = decided;
                        if backoff_level > 0 {
                            backoff_level = 0;
                            macro_tick = tokio::time::interval(base_interval);
                            macro_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                        }
                    }
                    Some(Outcome::Refused(err, fallback)) => {
                        summary.budget_refusals += 1;
                        if paid_enabled {
                            tracing::warn!("brain off for the rest of the run: {err}");
                        }
                        paid_enabled = false;
                        plan = fallback;
                    }
                    Some(Outcome::Failed(err, fallback)) => {
                        summary.decisions_failed += 1;
                        if is_retryable(&err) && backoff_level < MAX_BACKOFF_LEVEL {
                            backoff_level += 1;
                            summary.backoffs += 1;
                            macro_tick = tokio::time::interval(backoff_interval(base_interval, backoff_level));
                            macro_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                        }
                        tracing::warn!("brain call failed, local rules this cycle: {err}");
                        plan = fallback;
                    }
                    None => break,
                }
            },
        }
    }
    let _ = sink.close().await;
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
            match_config: Some(MatchConfig {
                warmup_ticks: 1,
                boss_spawn_ticks: None,
                compliance_ping_ticks: None,
                ..MatchConfig::default()
            }),
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
            summary.decisions_failed <= 6,
            "backoff bounds the failures: {summary:?}"
        );
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
        let bad = config("ws://127.0.0.1:9", Provider::Local, 1);
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
        let _ = shutdown.send(());
        let _ = Refusal::NoCap;
    }
}
