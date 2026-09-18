use crate::protocol::{ClientMessage, Role, ServerMessage};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

pub type WsTx = mpsc::UnboundedSender<ServerMessage>;
pub type WsRx = mpsc::UnboundedReceiver<ServerMessage>;

pub struct ClientSession {
    pub id: Uuid,
    pub tx: WsTx,
}

pub struct NetServer {
    listener: TcpListener,
    pub clients: Arc<Mutex<Vec<ClientSession>>>,
    game_tx: mpsc::UnboundedSender<GameCommand>,
}

pub enum GameCommand {
    Connected {
        id: Uuid,
        role: Role,
        name: String,
        player_id: Option<Uuid>,
    },
    Disconnected {
        id: Uuid,
    },
    Action {
        player_id: Uuid,
        action: crate::protocol::Action,
    },
    Speak {
        player_id: Uuid,
        text: String,
    },
    SetDisplayBehavior {
        player_id: Uuid,
        behavior: String,
    },
}

impl NetServer {
    pub async fn bind(
        addr: &str,
        game_tx: mpsc::UnboundedSender<GameCommand>,
    ) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        tracing::info!("WebSocket server listening on {}", addr);

        Ok(Self {
            listener,
            clients: Arc::new(Mutex::new(Vec::new())),
            game_tx,
        })
    }

    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }

    pub async fn accept_loop(self) {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    tracing::debug!("New connection from {}", addr);
                    let game_tx = self.game_tx.clone();
                    let clients = self.clients.clone();

                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, game_tx, clients).await {
                            tracing::warn!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Accept error: {}", e);
                }
            }
        }
    }
}

async fn handle_connection(
    stream: TcpStream,
    game_tx: mpsc::UnboundedSender<GameCommand>,
    clients: Arc<Mutex<Vec<ClientSession>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sink, mut ws_stream) = ws_stream.split();

    let (tx, mut rx): (WsTx, WsRx) = mpsc::unbounded_channel();
    let client_id = Uuid::new_v4();

    let role;
    let player_id;

    if let Some(Ok(Message::Text(text))) = ws_stream.next().await {
        match serde_json::from_str::<ClientMessage>(&text) {
            Ok(ClientMessage::Hello { role: r, name }) => {
                role = Some(r);

                player_id = if r != Role::Spectator {
                    Some(Uuid::new_v4())
                } else {
                    None
                };

                let welcome = ServerMessage::Welcome {
                    player_id,
                    role: r,
                    mode_name: crate::protocol::default_mode_name(),
                    playlist: crate::protocol::default_playlist(),
                };

                ws_sink
                    .send(Message::Text(serde_json::to_string(&welcome)?))
                    .await?;

                let mut clients_lock = clients.lock().await;
                clients_lock.push(ClientSession {
                    id: client_id,
                    tx: tx.clone(),
                });
                drop(clients_lock);

                game_tx.send(GameCommand::Connected {
                    id: client_id,
                    role: r,
                    name,
                    player_id,
                })?;

                tracing::info!(
                    "Client {:?} connected as {:?} (player_id: {:?})",
                    client_id,
                    r,
                    player_id
                );
            }
            _ => {
                tracing::warn!("Invalid hello message");
                return Ok(());
            }
        }
    } else {
        return Ok(());
    }

    let role = role.unwrap();

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let Ok(json) = serde_json::to_string(&msg) else {
                tracing::warn!("Failed to serialize outbound server message; dropping client send");
                break;
            };
            if ws_sink.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if role != Role::Spectator {
                    match serde_json::from_str::<ClientMessage>(&text) {
                        Ok(ClientMessage::Action(action)) => {
                            if let Some(pid) = player_id {
                                let _ = game_tx.send(GameCommand::Action {
                                    player_id: pid,
                                    action,
                                });
                            }
                        }
                        Ok(ClientMessage::Speak(speak)) => {
                            if let Some(pid) = player_id {
                                let _ = game_tx.send(GameCommand::Speak {
                                    player_id: pid,
                                    text: speak.text,
                                });
                            }
                        }
                        Ok(ClientMessage::SetDisplayBehavior(msg)) if role == Role::Agent => {
                            // Further gated in sim (rule bots / humans ignored).
                            if let Some(pid) = player_id {
                                let _ = game_tx.send(GameCommand::SetDisplayBehavior {
                                    player_id: pid,
                                    behavior: msg.behavior,
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    send_task.abort();

    game_tx.send(GameCommand::Disconnected { id: client_id })?;

    let mut clients_lock = clients.lock().await;
    clients_lock.retain(|c| c.id != client_id);

    tracing::info!("Client {:?} disconnected", client_id);

    Ok(())
}
