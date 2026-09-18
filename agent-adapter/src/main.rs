mod protocol;

use clap::{Parser, Subcommand};
use futures_util::{SinkExt, StreamExt};
use protocol::{ClientMessage, Role, ServerMessage};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, BufRead, Write};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "doomy-agent-adapter")]
#[command(about = "Doomy agent adapter - MCP server and bot client")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Mcp {
        #[arg(long, default_value = "ws://127.0.0.1:7777")]
        server: String,
    },

    ScriptedBot {
        #[arg(long, default_value = "ws://127.0.0.1:7777")]
        server: String,

        #[arg(long, default_value = "ScriptedBot")]
        name: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct McpRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct McpResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<McpError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct McpError {
    code: i32,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,doomy_agent_adapter=debug")),
        )
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();

    match args.command {
        Commands::Mcp { server } => run_mcp_server(server).await?,
        Commands::ScriptedBot { server, name } => run_scripted_bot(server, name).await?,
    }

    Ok(())
}

async fn run_mcp_server(server_url: String) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Starting MCP server mode, connecting to {}", server_url);

    let (ws_stream, _) = connect_async(&server_url).await?;
    let (mut ws_sink, mut ws_stream) = ws_stream.split();

    let hello = ClientMessage::Hello {
        role: Role::Agent,
        name: "MCP Agent".to_string(),
    };
    ws_sink
        .send(Message::Text(serde_json::to_string(&hello)?))
        .await?;

    if let Some(Ok(Message::Text(text))) = ws_stream.next().await {
        if let Ok(ServerMessage::Welcome { player_id, role: _ }) = serde_json::from_str(&text) {
            tracing::info!("Connected to game server, player_id: {:?}", player_id);
        }
    }

    let last_snapshot = std::sync::Arc::new(tokio::sync::Mutex::new(None::<Value>));
    let last_snapshot_clone = last_snapshot.clone();

    let recent_events = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::<Value>::new()));
    let recent_events_clone = recent_events.clone();

    tokio::spawn(async move {
        while let Some(msg) = ws_stream.next().await {
            if let Ok(Message::Text(text)) = msg {
                match serde_json::from_str::<ServerMessage>(&text) {
                    Ok(ServerMessage::Snapshot(snapshot)) => {
                        let snapshot_value = serde_json::to_value(snapshot).unwrap();
                        *last_snapshot_clone.lock().await = Some(snapshot_value);
                    }
                    Ok(ServerMessage::Event(event)) => {
                        let event_value = serde_json::to_value(event).unwrap();
                        let mut events = recent_events_clone.lock().await;
                        events.push(event_value);
                        if events.len() > 50 {
                            events.remove(0);
                        }
                        tracing::info!("Game event received: {}", text);
                    }
                    Ok(ServerMessage::Welcome { .. }) => {}
                    Err(e) => {
                        tracing::warn!("Failed to parse server message: {} - error: {}", text, e);
                    }
                }
            }
        }
    });

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        let request: McpRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let response = match request.method.as_str() {
            "initialize" => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.unwrap_or(Value::Null),
                result: Some(serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "doomy-agent-adapter",
                        "version": "0.1.0"
                    }
                })),
                error: None,
            },

            "tools/list" => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.unwrap_or(Value::Null),
                result: Some(serde_json::json!({
                    "tools": [
                        {
                            "name": "observe",
                            "description": "Get current game state observation including recent events",
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "required": []
                            }
                        },
                        {
                            "name": "act",
                            "description": "Send action to the game server",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "forward": {"type": "boolean", "default": false},
                                    "back": {"type": "boolean", "default": false},
                                    "left": {"type": "boolean", "default": false},
                                    "right": {"type": "boolean", "default": false},
                                    "turn_left": {"type": "boolean", "default": false},
                                    "turn_right": {"type": "boolean", "default": false},
                                    "fire": {"type": "boolean", "default": false}
                                },
                                "required": []
                            }
                        },
                        {
                            "name": "get_events",
                            "description": "Get recent game events (frags, respawns). Includes last 50 events.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "clear": {"type": "boolean", "default": false, "description": "Clear events after retrieving"}
                                },
                                "required": []
                            }
                        }
                    ]
                })),
                error: None,
            },

            "tools/call" => {
                let tool_name = request
                    .params
                    .as_ref()
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("");

                let result = match tool_name {
                    "observe" => {
                        let snapshot_lock = last_snapshot.lock().await;
                        let events_lock = recent_events.lock().await;
                        let mut observation = snapshot_lock.clone().unwrap_or(serde_json::json!({
                            "tick": 0,
                            "players": []
                        }));

                        if let Some(obj) = observation.as_object_mut() {
                            obj.insert(
                                "recent_events".to_string(),
                                serde_json::json!(events_lock.clone()),
                            );
                        }

                        observation
                    }

                    "act" => {
                        let arguments = request
                            .params
                            .as_ref()
                            .and_then(|p| p.get("arguments"))
                            .cloned()
                            .unwrap_or(Value::Null);

                        let action = ClientMessage::Action(protocol::Action {
                            forward: arguments
                                .get("forward")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                            back: arguments
                                .get("back")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                            left: arguments
                                .get("left")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                            right: arguments
                                .get("right")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                            turn_left: arguments
                                .get("turn_left")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                            turn_right: arguments
                                .get("turn_right")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                            fire: arguments
                                .get("fire")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                        });

                        ws_sink
                            .send(Message::Text(serde_json::to_string(&action)?))
                            .await?;

                        serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": "Action sent successfully"
                            }]
                        })
                    }

                    "get_events" => {
                        let arguments = request
                            .params
                            .as_ref()
                            .and_then(|p| p.get("arguments"))
                            .cloned()
                            .unwrap_or(Value::Null);

                        let should_clear = arguments
                            .get("clear")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        let mut events_lock = recent_events.lock().await;
                        let events_copy = events_lock.clone();

                        if should_clear {
                            events_lock.clear();
                        }

                        serde_json::json!({
                            "content": [{
                                "type": "text",
                                "text": format!("Recent events: {}", serde_json::to_string_pretty(&events_copy).unwrap())
                            }]
                        })
                    }

                    _ => serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Unknown tool: {}", tool_name)
                        }]
                    }),
                };

                McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id.unwrap_or(Value::Null),
                    result: Some(result),
                    error: None,
                }
            }

            _ => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.unwrap_or(Value::Null),
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                }),
            },
        };

        let response_json = serde_json::to_string(&response)?;
        writeln!(stdout, "{}", response_json)?;
        stdout.flush()?;
    }

    Ok(())
}

async fn run_scripted_bot(
    server_url: String,
    name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        "Starting scripted bot '{}', connecting to {}",
        name,
        server_url
    );

    let (ws_stream, _) = connect_async(&server_url).await?;
    let (mut ws_sink, mut ws_stream) = ws_stream.split();

    let hello = ClientMessage::Hello {
        role: Role::Agent,
        name: name.clone(),
    };
    ws_sink
        .send(Message::Text(serde_json::to_string(&hello)?))
        .await?;

    let mut player_id = None;
    if let Some(Ok(Message::Text(text))) = ws_stream.next().await {
        if let Ok(ServerMessage::Welcome {
            player_id: pid,
            role: _,
        }) = serde_json::from_str(&text)
        {
            player_id = pid;
            tracing::info!("Bot connected, player_id: {:?}", player_id);
        }
    }

    let bot_id = player_id.unwrap();
    let mut last_snapshot: Option<protocol::Snapshot> = None;

    loop {
        tokio::select! {
            msg = ws_stream.next() => {
                if let Some(Ok(Message::Text(text))) = msg {
                    if let Ok(ServerMessage::Snapshot(snapshot)) = serde_json::from_str(&text) {
                        last_snapshot = Some(snapshot);
                    }
                } else {
                    break;
                }
            }

            _ = tokio::time::sleep(tokio::time::Duration::from_millis(50)) => {
                if let Some(ref snapshot) = last_snapshot {
                    let action = compute_bot_action(bot_id, snapshot);
                    let action_msg = ClientMessage::Action(action);

                    if ws_sink.send(Message::Text(serde_json::to_string(&action_msg)?)).await.is_err() {
                        break;
                    }
                }
            }
        }
    }

    tracing::info!("Bot disconnected");
    Ok(())
}

fn compute_bot_action(bot_id: uuid::Uuid, snapshot: &protocol::Snapshot) -> protocol::Action {
    use std::f32::consts::PI;

    let bot = snapshot.players.iter().find(|p| p.id == bot_id);

    let Some(bot) = bot else {
        return protocol::Action::default();
    };

    let mut nearest_dist = f32::MAX;
    let mut nearest_target = None;

    for target in &snapshot.players {
        if target.id == bot_id {
            continue;
        }

        let dx = target.x - bot.x;
        let dz = target.z - bot.z;
        let dist = (dx * dx + dz * dz).sqrt();

        if dist < nearest_dist {
            nearest_dist = dist;
            nearest_target = Some(target);
        }
    }

    let Some(target) = nearest_target else {
        return protocol::Action::default();
    };

    let dx = target.x - bot.x;
    let dz = target.z - bot.z;
    let target_angle = dz.atan2(dx);

    let mut angle_diff = target_angle - bot.yaw;
    while angle_diff > PI {
        angle_diff -= 2.0 * PI;
    }
    while angle_diff < -PI {
        angle_diff += 2.0 * PI;
    }

    let mut action = protocol::Action::default();

    if angle_diff.abs() > 0.3 {
        if angle_diff > 0.0 {
            action.turn_right = true;
        } else {
            action.turn_left = true;
        }
    }

    if nearest_dist > 3.0 {
        action.forward = true;
    }

    if angle_diff.abs() < 0.5 && nearest_dist < 30.0 {
        action.fire = true;
    }

    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_event_serialization() {
        let frag_event = protocol::GameEvent::Frag {
            killer: "Bot1".to_string(),
            victim: "Bot2".to_string(),
        };

        let frag_json = serde_json::to_value(&frag_event).unwrap();
        assert_eq!(frag_json["event"], "frag");
        assert_eq!(frag_json["killer"], "Bot1");
        assert_eq!(frag_json["victim"], "Bot2");

        let respawn_event = protocol::GameEvent::Respawn {
            player: "Bot2".to_string(),
        };

        let respawn_json = serde_json::to_value(&respawn_event).unwrap();
        assert_eq!(respawn_json["event"], "respawn");
        assert_eq!(respawn_json["player"], "Bot2");
    }

    #[test]
    fn test_server_message_event_parsing() {
        let frag_msg = r#"{"type":"event","event":"frag","killer":"Bot1","victim":"Bot2"}"#;
        let parsed: Result<protocol::ServerMessage, _> = serde_json::from_str(frag_msg);
        assert!(parsed.is_ok());

        match parsed.unwrap() {
            protocol::ServerMessage::Event(protocol::GameEvent::Frag { killer, victim }) => {
                assert_eq!(killer, "Bot1");
                assert_eq!(victim, "Bot2");
            }
            _ => panic!("Expected Event(Frag)"),
        }

        let respawn_msg = r#"{"type":"event","event":"respawn","player":"Bot2"}"#;
        let parsed: Result<protocol::ServerMessage, _> = serde_json::from_str(respawn_msg);
        assert!(parsed.is_ok());

        match parsed.unwrap() {
            protocol::ServerMessage::Event(protocol::GameEvent::Respawn { player }) => {
                assert_eq!(player, "Bot2");
            }
            _ => panic!("Expected Event(Respawn)"),
        }
    }
}
