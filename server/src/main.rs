use clap::Parser;
use fragr_server::run::{run_server, ServerOptions};
use tracing_subscriber::EnvFilter;

// Eq is out because a threshold is a float; PartialEq still serves the tests.
#[derive(Parser, Debug, PartialEq)]
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

    /// Benchmark instead of serving: run this many scripted fighters with no
    /// network, print one JSON report, and exit. The ruler for every change.
    #[arg(long)]
    bench: Option<usize>,

    /// Ticks to run in benchmark mode (20 per second of match time).
    #[arg(long, default_value_t = 1200)]
    bench_ticks: u64,

    /// Run the benchmark twice and report whether the two matches agreed.
    #[arg(long, default_value_t = false)]
    bench_check: bool,

    /// Seed for the simulation's random stream. The same seed gives the same
    /// match, which is what makes two runs comparable.
    #[arg(long, default_value_t = 1)]
    seed: u64,

    /// Print the status report (tick time, bytes, budget use) this often, in
    /// seconds. Zero turns it off.
    #[arg(long, default_value_t = 60)]
    status_every_s: u64,

    /// Exit non-zero when a benchmark crosses a threshold below. This is what
    /// CI runs, so a regression is a failed build rather than a note nobody
    /// reads.
    #[arg(long, default_value_t = false)]
    bench_assert: bool,

    /// Largest share of the tick budget the p99 tick may use before
    /// `--bench-assert` fails. One tick in a hundred over half the budget
    /// means the next change has nowhere to go.
    #[arg(long, default_value_t = 0.5)]
    bench_max_budget_p99: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    // In benchmark mode the JSON report is the only thing on stdout, so logs
    // go to stderr and only warnings survive.
    init_tracing(args.bench.is_some());
    let map = fragr_server::sim::MapKind::from_cli(&args.map).ok_or_else(|| {
        format!(
            "invalid --map {:?}; expected 1/arena or 2/compliance-yard",
            args.map
        )
    })?;
    if let Some(bots) = args.bench {
        let report = if args.bench_check {
            fragr_server::bench::run_bench_checked(bots, args.bench_ticks, map, args.seed)
        } else {
            fragr_server::bench::run_bench(bots, args.bench_ticks, map, args.seed)
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
        // A run that is not reproducible is a correctness failure, not a slow one.
        if report.deterministic == Some(false) {
            return Err("benchmark was not deterministic for this seed".into());
        }
        if args.bench_assert {
            for complaint in
                fragr_server::bench::check_thresholds(&report, args.bench_max_budget_p99)
            {
                eprintln!("benchmark: {complaint}");
            }
            if !fragr_server::bench::check_thresholds(&report, args.bench_max_budget_p99).is_empty()
            {
                return Err("benchmark crossed a threshold".into());
            }
        }
        return Ok(());
    }

    let options = ServerOptions {
        bind: args.bind,
        bots: args.bots,
        map,
        map_rotate: args.map_rotate,
        match_config: None,
        solo_broadcast: args.solo_broadcast,
        seed: args.seed,
        status_every_s: args.status_every_s,
    };
    run_server(options, std::future::pending::<()>(), None).await
}

fn init_tracing(quiet: bool) {
    let default = if quiet {
        "warn"
    } else {
        "info,fragr_server=debug"
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default)),
        )
        .with_writer(std::io::stderr)
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
                    seed: 1,
                    status_every_s: 0,
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

        // A joiner is told the arena's shape once, then the tick broadcasts
        // begin. Read a few messages so the test does not depend on the order.
        let mut saw_map = false;
        let mut saw_tick = false;
        for _ in 0..8 {
            let msg = tokio::time::timeout(Duration::from_secs(2), stream.next())
                .await
                .expect("tick timeout")
                .expect("tick msg")
                .expect("tick ok");
            let Message::Text(text) = msg else {
                panic!("expected text broadcast, got {msg:?}");
            };
            let v: serde_json::Value = serde_json::from_str(&text).expect("json");
            match v["type"].as_str().unwrap_or("") {
                "map_info" => {
                    assert!(v["solids"].is_array(), "map_info carries solids: {text}");
                    assert!(v["half_extent"].as_f64().unwrap_or(0.0) > 0.0);
                    saw_map = true;
                }
                "snapshot" | "event" => saw_tick = true,
                other => panic!("unexpected message type {other}: {text}"),
            }
            if saw_map && saw_tick {
                break;
            }
        }
        assert!(saw_map, "a joining fighter should be told the map");
        assert!(saw_tick, "and then receive tick broadcasts");

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
