use clap::Parser;
use fragr_server::net::NetServer;
use fragr_server::session::{broadcast_to_clients, send_unicasts_to_players, GameSession};
use std::future::Future;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug, PartialEq, Eq)]
#[command(name = "fragr-server")]
#[command(about = "fragr authoritative game server")]
struct Args {
    #[arg(long, default_value = "0.0.0.0:6767")]
    bind: String,

    #[arg(long, default_value = "4")]
    bots: usize,

    /// Scrap map: 1/arena (Arena Duel) or 2/compliance-yard (Compliance Yard).
    #[arg(long, default_value = "1")]
    map: String,

    /// Alternate Arena Duel and Compliance Yard each round.
    #[arg(long, default_value_t = false)]
    map_rotate: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();
    let args = Args::parse();
    let map = fragr_server::sim::MapKind::from_cli(&args.map).ok_or_else(|| {
        format!(
            "invalid --map {:?}; expected 1/arena or 2/compliance-yard",
            args.map
        )
    })?;
    run_server(
        args.bind,
        args.bots,
        map,
        args.map_rotate,
        std::future::pending::<()>(),
        None,
    )
    .await
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,fragr_server=debug")),
        )
        .init();
}

/// Bind, accept clients, and tick the session until `shutdown` resolves.
/// When `ready` is Some, send the bound address once accept is live.
async fn run_server(
    bind: String,
    bots: usize,
    map: fragr_server::sim::MapKind,
    map_rotate: bool,
    shutdown: impl Future<Output = ()>,
    ready: Option<tokio::sync::oneshot::Sender<SocketAddr>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (game_tx, mut game_rx) = mpsc::unbounded_channel();

    let net_server = NetServer::bind(&bind, game_tx.clone()).await?;
    if let Some(tx) = ready {
        let _ = tx.send(net_server.local_addr()?);
    }
    let clients = net_server.clients.clone();

    tokio::spawn(async move {
        net_server.accept_loop().await;
    });

    let mut session = GameSession::with_map(map, map_rotate);
    session.spawn_bots(bots);
    tracing::info!(
        "Map: {} (id {}){}",
        map.name(),
        map.id(),
        if map_rotate {
            ", rotate each round"
        } else {
            ""
        }
    );

    let tick_duration = Duration::from_millis(50);
    let mut tick_interval = tokio::time::interval(tick_duration);
    tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    tracing::info!("Game loop starting (20 Hz tick)");

    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            _ = tick_interval.tick() => {
                let messages = session.tick_messages(tick_duration.as_secs_f32());
                broadcast_to_clients(&clients, &messages).await;
            }

            Some(cmd) = game_rx.recv() => {
                session.apply_command(cmd);
                let unicasts = session.take_unicasts();
                send_unicasts_to_players(&clients, &session.client_to_player, &unicasts).await;
            }

            _ = &mut shutdown => {
                tracing::info!("Server shutdown requested");
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    #[test]
    fn args_default_bind_and_bots() {
        let args = Args::try_parse_from(["fragr-server"]).expect("defaults");
        assert_eq!(args.bind, "0.0.0.0:6767");
        assert_eq!(args.bots, 4);
        assert_eq!(args.map, "1");
        assert!(!args.map_rotate);
    }

    #[test]
    fn args_custom_bind_and_bots() {
        let args = Args::try_parse_from(["fragr-server", "--bind", "127.0.0.1:0", "--bots", "2"])
            .expect("custom");
        assert_eq!(args.bind, "127.0.0.1:0");
        assert_eq!(args.bots, 2);
    }

    #[test]
    fn args_map_and_rotate() {
        let args =
            Args::try_parse_from(["fragr-server", "--map", "compliance-yard", "--map-rotate"])
                .expect("map args");
        assert_eq!(args.map, "compliance-yard");
        assert!(args.map_rotate);
        assert!(fragr_server::sim::MapKind::from_cli(&args.map).is_some());
    }

    #[tokio::test]
    async fn run_server_ws_hello_welcome_tick_then_shutdown() {
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<SocketAddr>();

        let server = tokio::spawn(async move {
            run_server(
                "127.0.0.1:0".to_string(),
                1,
                fragr_server::sim::MapKind::ArenaDuel,
                false,
                async move {
                    let _ = shutdown_rx.await;
                },
                Some(ready_tx),
            )
            .await
            .map_err(|e| e.to_string())
        });

        let addr = tokio::time::timeout(Duration::from_secs(2), ready_rx)
            .await
            .expect("ready timeout")
            .expect("ready addr");

        let url = format!("ws://{}", addr);
        let (ws, _) = connect_async(&url).await.expect("connect");
        let (mut sink, mut stream) = ws.split();

        let hello = serde_json::json!({
            "type": "hello",
            "role": "agent",
            "name": "CovClimb"
        });
        sink.send(Message::Text(hello.to_string()))
            .await
            .expect("send hello");

        let welcome = tokio::time::timeout(Duration::from_secs(2), stream.next())
            .await
            .expect("welcome timeout")
            .expect("welcome msg")
            .expect("welcome ok");
        let Message::Text(text) = welcome else {
            panic!("expected text welcome, got {welcome:?}");
        };
        let v: serde_json::Value = serde_json::from_str(&text).expect("welcome json");
        assert_eq!(v["type"], "welcome");
        assert!(v["player_id"].is_string(), "player_id={}", v["player_id"]);

        // Session tick should broadcast at least one snapshot/event after join.
        let tick_msg = tokio::time::timeout(Duration::from_secs(2), stream.next())
            .await
            .expect("tick timeout")
            .expect("tick msg")
            .expect("tick ok");
        let Message::Text(tick_text) = tick_msg else {
            panic!("expected text tick broadcast, got {tick_msg:?}");
        };
        let tick_v: serde_json::Value = serde_json::from_str(&tick_text).expect("tick json");
        let tick_type = tick_v["type"].as_str().unwrap_or("");
        assert!(
            tick_type == "snapshot" || tick_type == "event",
            "unexpected tick type: {tick_text}"
        );

        let _ = shutdown_tx.send(());
        let result = tokio::time::timeout(Duration::from_secs(2), server)
            .await
            .expect("server join timeout")
            .expect("server task");
        assert!(result.is_ok(), "{result:?}");
    }
}
