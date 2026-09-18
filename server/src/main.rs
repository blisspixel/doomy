use clap::Parser;
use fragr_server::run::{run_server, ServerOptions};
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

    /// Contested Frequency Solo Broadcast Episode 0 (Calibration / Larak Lot).
    /// NODS clear + jammer dish + Auditor. MP unchanged when off.
    #[arg(long, default_value_t = false)]
    solo_broadcast: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_tracing();
    let args = Args::parse();
    let map = fragr_server::sim::MapKind::from_cli(&args.map).ok_or_else(|| {
        format!(
            "invalid --map {:?}; expected 1/arena or 2/compliance-yard",
            args.map
        )
    })?;
    let options = ServerOptions {
        bind: args.bind,
        bots: args.bots,
        map,
        map_rotate: args.map_rotate,
        match_config: None,
        solo_broadcast: args.solo_broadcast,
    };
    run_server(options, std::future::pending::<()>(), None).await
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,fragr_server=debug")),
        )
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use std::net::SocketAddr;
    use std::time::Duration;
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    #[test]
    fn args_default_bind_and_bots() {
        let args = Args::try_parse_from(["fragr-server"]).expect("defaults");
        assert_eq!(args.bind, "0.0.0.0:6767");
        assert_eq!(args.bots, 4);
        assert_eq!(args.map, "1");
        assert!(!args.map_rotate);
        assert!(!args.solo_broadcast);
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

    #[test]
    fn args_solo_broadcast() {
        let args = Args::try_parse_from(["fragr-server", "--solo-broadcast"]).expect("solo");
        assert!(args.solo_broadcast);
    }

    #[tokio::test]
    async fn run_server_ws_hello_welcome_tick_then_shutdown() {
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<SocketAddr>();

        let server = tokio::spawn(async move {
            run_server(
                ServerOptions {
                    bind: "127.0.0.1:0".to_string(),
                    bots: 1,
                    map: fragr_server::sim::MapKind::ArenaDuel,
                    map_rotate: false,
                    match_config: None,
                    solo_broadcast: false,
                },
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

    #[test]
    fn server_options_default_matches_cli_defaults() {
        let options = ServerOptions::default();
        let args = Args::try_parse_from(["fragr-server"]).expect("defaults");
        assert_eq!(options.bind, args.bind);
        assert_eq!(options.bots, args.bots);
        assert!(options.match_config.is_none());
    }
}
