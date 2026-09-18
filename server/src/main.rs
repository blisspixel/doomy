use clap::Parser;
use fragr_server::net::{GameCommand, NetServer};
use fragr_server::protocol::{self, Role, ServerMessage};
use fragr_server::sim::{BotController, GameState};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "fragr-server")]
#[command(about = "fragr authoritative game server")]
struct Args {
    #[arg(long, default_value = "0.0.0.0:7777")]
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

    let mut state = GameState::new();
    let mut bots = Vec::new();
    let mut client_to_player = HashMap::new();

    let bot_configs = [
        ("Rusher", fragr_server::sim::BotBehavior::Aggressive),
        ("Sniper", fragr_server::sim::BotBehavior::Defensive),
        ("Flanker", fragr_server::sim::BotBehavior::Flanker),
        ("Tank", fragr_server::sim::BotBehavior::Balanced),
        ("Scout", fragr_server::sim::BotBehavior::Flanker),
        ("Guard", fragr_server::sim::BotBehavior::Defensive),
        ("Hunter", fragr_server::sim::BotBehavior::Aggressive),
        ("Striker", fragr_server::sim::BotBehavior::Balanced),
    ];

    for i in 0..args.bots {
        let bot_id = Uuid::new_v4();
        let (bot_name, behavior) = bot_configs
            .get(i)
            .unwrap_or(&("Bot", fragr_server::sim::BotBehavior::Balanced));
        state.add_player(bot_id, bot_name.to_string(), Role::Agent);
        let bot_controller = BotController::new(bot_id, *behavior);
        bots.push(bot_controller.clone());
        state.bots.push(bot_controller);
        tracing::info!("Spawned bot: {} ({:?}, {})", bot_name, behavior, bot_id);
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
                    GameCommand::Connected { id, role, name, player_id } => {
                        if let Some(pid) = player_id {
                            state.add_player(pid, name.clone(), role);
                            client_to_player.insert(id, pid);
                            let player_count = state.players.len();
                            state.push_event(protocol::GameEvent::PlayerJoined {
                                player: name.clone(),
                                role: format!("{:?}", role).to_lowercase(),
                                round_number: state.round_number,
                                player_count,
                            });
                            tracing::info!("Player {} joined as {:?} (round {}, {} players)", pid, role, state.round_number, player_count);
                        } else {
                            tracing::info!("Spectator {} joined", name);
                        }
                    }

                    GameCommand::Disconnected { id } => {
                        if let Some(player_id) = client_to_player.remove(&id) {
                            let (player_name, player_score) = state.players.iter()
                                .find(|p| p.id == player_id)
                                .map(|p| (p.name.clone(), *state.scores.get(&p.id).unwrap_or(&0)))
                                .unwrap_or_else(|| ("Unknown".to_string(), 0));
                            let player_count_before = state.players.len();
                            state.remove_player(player_id);
                            state.push_event(protocol::GameEvent::PlayerLeft {
                                player: player_name.clone(),
                                score: player_score,
                                round_number: state.round_number,
                                player_count: player_count_before - 1,
                            });
                            tracing::info!("Player {} left (score: {}, round {}, {} players remain)", player_name, player_score, state.round_number, player_count_before - 1);
                        }
                    }

                    GameCommand::Action { player_id, action } => {
                        state.set_action(player_id, action);
                    }
                }
            }
        }
    }
}
