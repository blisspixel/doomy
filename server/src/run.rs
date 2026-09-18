//! The authoritative tick loop, shared by the `fragr-server` binary and by
//! in-process harnesses such as the playtest tool. Binding on port 0 and the
//! `ready` channel let a caller learn the real address; `shutdown` ends the loop.

use crate::net::NetServer;
use crate::session::{broadcast_to_clients, send_unicasts_to_players, GameSession};
use crate::sim::{MapKind, MatchConfig};
use std::future::Future;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::mpsc;

/// Fixed simulation step: 20 Hz.
pub const TICK: Duration = Duration::from_millis(50);

/// Everything the loop needs besides the shutdown signal.
#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub bind: String,
    pub bots: usize,
    pub map: MapKind,
    pub map_rotate: bool,
    /// Match rules override (frag limit, timers). `None` keeps the defaults.
    pub match_config: Option<MatchConfig>,
}

impl Default for ServerOptions {
    fn default() -> Self {
        ServerOptions {
            bind: "0.0.0.0:6767".to_string(),
            bots: 4,
            map: MapKind::default(),
            map_rotate: false,
            match_config: None,
        }
    }
}

/// Bind, accept clients, and tick the session until `shutdown` resolves.
/// When `ready` is Some, send the bound address once accept is live.
pub async fn run_server(
    options: ServerOptions,
    shutdown: impl Future<Output = ()>,
    ready: Option<tokio::sync::oneshot::Sender<SocketAddr>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (game_tx, mut game_rx) = mpsc::unbounded_channel();

    let net_server = NetServer::bind(&options.bind, game_tx.clone()).await?;
    if let Some(tx) = ready {
        let _ = tx.send(net_server.local_addr()?);
    }
    let clients = net_server.clients.clone();

    tokio::spawn(async move {
        net_server.accept_loop().await;
    });

    let mut session = GameSession::with_map(options.map, options.map_rotate);
    if let Some(config) = options.match_config {
        session.state.config = config;
    }
    session.spawn_bots(options.bots);
    tracing::info!(
        "Map: {} (id {}){}",
        options.map.name(),
        options.map.id(),
        if options.map_rotate {
            ", rotate each round"
        } else {
            ""
        }
    );

    let mut tick_interval = tokio::time::interval(TICK);
    tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    tracing::info!("Game loop starting (20 Hz tick)");

    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            _ = tick_interval.tick() => {
                let messages = session.tick_messages(TICK.as_secs_f32());
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
