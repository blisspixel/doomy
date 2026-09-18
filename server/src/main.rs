mod net;
mod protocol;
mod sim;

use clap::Parser;
use net::{GameCommand, NetServer};
use protocol::{Role, ServerMessage};
use sim::{BotController, GameState};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "doomy-server")]
#[command(about = "Doomy authoritative game server")]
struct Args {
    #[arg(long, default_value = "127.0.0.1:7777")]
    bind: String,

    #[arg(long, default_value = "2")]
    bots: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,doomy_server=debug")),
        )
        .init();

    let args = Args::parse();

    let (game_tx, mut game_rx) = mpsc::unbounded_channel();

    let net_server = NetServer::bind(&args.bind, game_tx.clone()).await?;
    let clients = net_server.clients.clone();

    tokio::spawn(async move {
        net_server.accept_loop().await;
    });

    let mut state = GameState::new();
    let mut bots = Vec::new();
    let mut client_to_player = HashMap::new();

    for i in 0..args.bots {
        let bot_id = Uuid::new_v4();
        let bot_name = format!("Bot{}", i + 1);
        state.add_player(bot_id, bot_name, Role::Agent);
        bots.push(BotController::new(bot_id));
        tracing::info!("Spawned bot: {} ({})", bot_id, i + 1);
    }

    let tick_duration = Duration::from_millis(50);
    let mut tick_interval = tokio::time::interval(tick_duration);
    tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    tracing::info!("Game loop starting (20 Hz tick)");

    loop {
        tokio::select! {
            _ = tick_interval.tick() => {
                for bot in &bots {
                    let action = bot.update(&state);
                    state.set_action(bot.player_id, action);
                }

                state.tick(tick_duration.as_secs_f32());

                let snapshot = state.snapshot();
                let snapshot_msg = ServerMessage::Snapshot(snapshot);
                
                let clients_lock = clients.lock().await;
                for client in clients_lock.iter() {
                    let _ = client.tx.send(snapshot_msg.clone());
                }
                drop(clients_lock);

                for event in state.take_events() {
                    let event_msg = ServerMessage::Event(event);
                    let clients_lock = clients.lock().await;
                    for client in clients_lock.iter() {
                        let _ = client.tx.send(event_msg.clone());
                    }
                    drop(clients_lock);
                }
            }

            Some(cmd) = game_rx.recv() => {
                match cmd {
                    GameCommand::ClientConnected { id, role, name, tx: _ } => {
                        if role != Role::Spectator {
                            let player_id = Uuid::new_v4();
                            state.add_player(player_id, name, role);
                            client_to_player.insert(id, player_id);
                            tracing::info!("Player {} joined as {:?}", player_id, role);
                        } else {
                            tracing::info!("Spectator {} joined", name);
                        }
                    }

                    GameCommand::ClientDisconnected { id } => {
                        if let Some(player_id) = client_to_player.remove(&id) {
                            state.remove_player(player_id);
                            tracing::info!("Player {} left", player_id);
                        }
                    }

                    GameCommand::ClientAction { player_id, action } => {
                        state.set_action(player_id, action);
                    }
                }
            }
        }
    }
}
