use crate::protocol::{ClientMessage, Role, ServerMessage};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use uuid::Uuid;

pub type WsTx = mpsc::UnboundedSender<ServerMessage>;
pub type WsRx = mpsc::UnboundedReceiver<ServerMessage>;

pub struct ClientSession {
    pub id: Uuid,
    pub player_id: Option<Uuid>,
    pub role: Role,
    pub tx: WsTx,
}

pub struct NetServer {
    listener: TcpListener,
    pub clients: Arc<Mutex<Vec<ClientSession>>>,
    game_tx: mpsc::UnboundedSender<GameCommand>,
}

pub enum GameCommand {
    ClientConnected {
        id: Uuid,
        role: Role,
        name: String,
        tx: WsTx,
    },
    ClientDisconnected {
        id: Uuid,
    },
    ClientAction {
        player_id: Uuid,
        action: crate::protocol::Action,
    },
}

impl NetServer {
    pub async fn bind(addr: &str, game_tx: mpsc::UnboundedSender<GameCommand>) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        tracing::info!("WebSocket server listening on {}", addr);

        Ok(Self {
            listener,
            clients: Arc::new(Mutex::new(Vec::new())),
            game_tx,
        })
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

    pub async fn broadcast(&self, msg: ServerMessage) {
        let clients = self.clients.lock().await;
        for client in clients.iter() {
            let _ = client.tx.send(msg.clone());
        }
    }

    pub async fn send_to(&self, player_id: Uuid, msg: ServerMessage) {
        let clients = self.clients.lock().await;
        if let Some(client) = clients.iter().find(|c| c.player_id == Some(player_id)) {
            let _ = client.tx.send(msg);
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

    let mut role = None;

    if let Some(Ok(Message::Text(text))) = ws_stream.next().await {
        match serde_json::from_str::<ClientMessage>(&text) {
            Ok(ClientMessage::Hello { role: r, name }) => {
                role = Some(r);
                
                let player_id = if r != Role::Spectator {
                    Some(Uuid::new_v4())
                } else {
                    None
                };

                let welcome = ServerMessage::Welcome {
                    player_id,
                    role: r,
                };

                ws_sink
                    .send(Message::Text(serde_json::to_string(&welcome)?))
                    .await?;

                let mut clients_lock = clients.lock().await;
                clients_lock.push(ClientSession {
                    id: client_id,
                    player_id,
                    role: r,
                    tx: tx.clone(),
                });
                drop(clients_lock);

                let cmd_player_id = player_id;

                game_tx.send(GameCommand::ClientConnected {
                    id: client_id,
                    role: r,
                    name,
                    tx,
                })?;

                tracing::info!("Client {:?} connected as {:?} (player_id: {:?})", client_id, r, cmd_player_id);
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
    let player_id = if role != Role::Spectator {
        Some(Uuid::new_v4())
    } else {
        None
    };

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if ws_sink.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if role != Role::Spectator {
                    if let Ok(ClientMessage::Action(action)) = serde_json::from_str(&text) {
                        if let Some(pid) = player_id {
                            let _ = game_tx.send(GameCommand::ClientAction {
                                player_id: pid,
                                action,
                            });
                        }
                    }
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    send_task.abort();

    game_tx.send(GameCommand::ClientDisconnected { id: client_id })?;

    let mut clients_lock = clients.lock().await;
    clients_lock.retain(|c| c.id != client_id);

    tracing::info!("Client {:?} disconnected", client_id);

    Ok(())
}
