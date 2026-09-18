//! The play loop. One WebSocket to the server, a 20 Hz controller that always
//! has an action ready, and a slower decision cadence that asks the brain for
//! intent without ever blocking the controller. At most one decision is in
//! flight; a slow answer is simply late, not queued.

use crate::budget::Budget;
use crate::decision::{plan_from_answers, tactical_questions, Question};
use crate::plan::{fallback_plan, micro_action, Plan, Source, Stance};
use crate::provider::{decide, decision_request, Provider, Transport};
use crate::telemetry::{observe, RecentHits};
use crate::Error;
use fragr_server::protocol::{
    ClientMessage, GameEvent, Role, ServerMessage, SetDisplayBehavior, Snapshot,
};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
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

#[derive(Debug, Clone)]
pub struct BotConfig {
    pub server_url: String,
    pub name: String,
    pub provider: Provider,
    pub model: String,
    /// Required for paid providers; ignored for `Provider::Local`.
    pub api_key: Option<String>,
    pub decision_hz: f64,
    pub confidence_floor: f64,
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
    pub frags: u32,
    pub deaths: u32,
    pub run_usd: f64,
    pub total_usd: f64,
    pub last_plan: Option<Plan>,
    pub last_state: Option<String>,
}

enum Outcome {
    Decided(Plan),
    Refused(Error, Plan),
    Failed(Error, Plan),
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
    state: String,
    fallback: Plan,
    floor: f64,
    tx: mpsc::UnboundedSender<Outcome>,
) {
    tokio::task::spawn_blocking(move || {
        let result = decision_request(provider, &model, &api_key, &state, &questions)
            .and_then(|request| decide(transport.as_ref(), &budget, provider, &model, &request))
            .map(|decision| plan_from_answers(&decision.response.answers, floor, &fallback));
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
        let _ = tx.send(outcome);
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

    let mut published_stance: Option<Stance> = None;
    let mut plan = Plan::default();
    if let Some(wire) = display_behavior_wire(&mut published_stance, plan.stance) {
        sink.send(Message::Text(wire))
            .await
            .map_err(transport_err)?;
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
    let hz = config.decision_hz.clamp(MIN_DECISION_HZ, MAX_DECISION_HZ);
    let mut micro = tokio::time::interval(MICRO_INTERVAL);
    micro.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut macro_tick = tokio::time::interval(Duration::from_secs_f64(1.0 / hz));
    macro_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let deadline = config
        .max_seconds
        .map(|s| tokio::time::Instant::now() + Duration::from_secs(s));
    let (tx, mut rx) = mpsc::unbounded_channel::<Outcome>();
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
                    if let Some(wire) = display_behavior_wire(&mut published_stance, plan.stance) {
                        if sink.send(Message::Text(wire)).await.is_err() {
                            break;
                        }
                    }
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
                    telemetry.render(),
                    fallback_plan(&telemetry, Source::Failure),
                    config.confidence_floor,
                    tx.clone(),
                );
            },
            outcome = rx.recv() => {
                inflight = false;
                match outcome {
                    Some(Outcome::Decided(decided)) => {
                        match decided.source {
                            Source::Remote => summary.decisions_remote += 1,
                            _ => summary.decisions_low_confidence += 1,
                        }
                        plan = decided;
                        if let Some(wire) =
                            display_behavior_wire(&mut published_stance, plan.stance)
                        {
                            if sink.send(Message::Text(wire)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Outcome::Refused(err, fallback)) => {
                        summary.budget_refusals += 1;
                        if paid_enabled {
                            tracing::warn!("brain off for the rest of the run: {err}");
                        }
                        paid_enabled = false;
                        plan = fallback;
                        if let Some(wire) =
                            display_behavior_wire(&mut published_stance, plan.stance)
                        {
                            if sink.send(Message::Text(wire)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Outcome::Failed(err, fallback)) => {
                        summary.decisions_failed += 1;
                        tracing::warn!("brain call failed, local rules this cycle: {err}");
                        plan = fallback;
                        if let Some(wire) =
                            display_behavior_wire(&mut published_stance, plan.stance)
                        {
                            if sink.send(Message::Text(wire)).await.is_err() {
                                break;
                            }
                        }
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
            confidence_floor: 0.65,
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
        let sent = transport.last_request.lock().unwrap().clone().unwrap();
        assert!(sent.body.unwrap()["state"]
            .as_str()
            .unwrap()
            .contains("SELF hp="));
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
