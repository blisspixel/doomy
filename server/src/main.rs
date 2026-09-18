use clap::Parser;
use fragr_server::net::NetServer;
use fragr_server::session::{broadcast_to_clients, GameSession};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "fragr-server")]
#[command(about = "fragr authoritative game server")]
struct Args {
    #[arg(long, default_value = "0.0.0.0:6767")]
    bind: String,

    #[arg(long, default_value = "4")]
    bots: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,fragr_server=debug")),
        )
        .init();

    let args = Args::parse();

    let (game_tx, mut game_rx) = mpsc::unbounded_channel();

    let net_server = NetServer::bind(&args.bind, game_tx.clone()).await?;
    let clients = net_server.clients.clone();

    tokio::spawn(async move {
        net_server.accept_loop().await;
    });

    let mut session = GameSession::new();
    session.spawn_bots(args.bots);

    let tick_duration = Duration::from_millis(50);
    let mut tick_interval = tokio::time::interval(tick_duration);
    tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    tracing::info!("Game loop starting (20 Hz tick)");

    loop {
        tokio::select! {
            _ = tick_interval.tick() => {
                let messages = session.tick_messages(tick_duration.as_secs_f32());
                broadcast_to_clients(&clients, &messages).await;
            }

            Some(cmd) = game_rx.recv() => {
                session.apply_command(cmd);
            }
        }
    }
}
