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
#[command(name = "fragr-agent-adapter")]
#[command(about = "fragr agent adapter - MCP server and bot client")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Mcp {
        #[arg(long, default_value = "ws://127.0.0.1:6767")]
        server: String,
    },

    ScriptedBot {
        #[arg(long, default_value = "ws://127.0.0.1:6767")]
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
                .unwrap_or_else(|_| EnvFilter::new("info,fragr_agent_adapter=debug")),
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

    let player_id = std::sync::Arc::new(tokio::sync::Mutex::new(None::<uuid::Uuid>));
    let player_id_clone = player_id.clone();

    if let Some(Ok(Message::Text(text))) = ws_stream.next().await {
        if let Ok(ServerMessage::Welcome {
            player_id: pid,
            role: _,
        }) = serde_json::from_str(&text)
        {
            *player_id.lock().await = pid;
            tracing::info!("Connected to game server, player_id: {:?}", pid);
        }
    }

    let last_snapshot = std::sync::Arc::new(tokio::sync::Mutex::new(None::<Value>));
    let last_snapshot_clone = last_snapshot.clone();

    let recent_events = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::<Value>::new()));
    let recent_events_clone = recent_events.clone();

    tokio::spawn(async move {
        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if text.len() > 1_000_000 {
                        tracing::warn!(
                            "Oversized message received ({} bytes), ignoring",
                            text.len()
                        );
                        continue;
                    }

                    match serde_json::from_str::<ServerMessage>(&text) {
                        Ok(ServerMessage::Snapshot(snapshot)) => {
                            match serde_json::to_value(snapshot) {
                                Ok(snapshot_value) => {
                                    *last_snapshot_clone.lock().await = Some(snapshot_value);
                                }
                                Err(e) => {
                                    tracing::error!("Failed to serialize snapshot: {}", e);
                                }
                            }
                        }
                        Ok(ServerMessage::Event(event)) => match serde_json::to_value(event) {
                            Ok(event_value) => {
                                let mut events = recent_events_clone.lock().await;
                                events.push(event_value);
                                if events.len() > 50 {
                                    events.remove(0);
                                }
                                tracing::info!("Game event received: {}", text);
                            }
                            Err(e) => {
                                tracing::error!("Failed to serialize event: {}", e);
                            }
                        },
                        Ok(ServerMessage::Welcome { .. }) => {}
                        Err(e) => {
                            tracing::warn!(
                                "Failed to parse server message: {} - error: {}",
                                text,
                                e
                            );
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    tracing::info!("Server closed connection");
                    break;
                }
                Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {}
                Ok(Message::Binary(_)) => {
                    tracing::warn!("Unexpected binary message, ignoring");
                }
                Ok(Message::Frame(_)) => {}
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;

        if line.len() > 100_000 {
            tracing::warn!("Oversized MCP request ({} bytes), ignoring", line.len());
            continue;
        }

        let request: McpRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Malformed MCP request: {} - error: {}", line, e);
                let error_response = McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Value::Null,
                    result: None,
                    error: Some(McpError {
                        code: -32700,
                        message: "Parse error".to_string(),
                    }),
                };
                if let Ok(response_json) = serde_json::to_string(&error_response) {
                    let _ = writeln!(stdout, "{}", response_json);
                    let _ = stdout.flush();
                }
                continue;
            }
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
                        "name": "fragr-agent-adapter",
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
                            "description": "Get current game state observation including self_player_id and recent events. Returns connecting state until first snapshot arrives.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "required": []
                            }
                        },
                        {
                            "name": "act",
                            "description": "Send action to the game server. Actions are level-held (sticky) within each tick window. Set true to activate, false to deactivate. Weapon swap changes loadout.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "forward": {"type": "boolean", "default": false, "description": "Move forward"},
                                    "back": {"type": "boolean", "default": false, "description": "Move backward"},
                                    "left": {"type": "boolean", "default": false, "description": "Strafe left"},
                                    "right": {"type": "boolean", "default": false, "description": "Strafe right"},
                                    "turn_left": {"type": "boolean", "default": false, "description": "Turn left"},
                                    "turn_right": {"type": "boolean", "default": false, "description": "Turn right"},
                                    "fire": {"type": "boolean", "default": false, "description": "Fire weapon"},
                                    "weapon_swap": {"type": "string", "enum": ["flechette", "rail", "scatter"], "description": "Switch to weapon type"}
                                },
                                "required": []
                            }
                        },
                        {
                            "name": "get_events",
                            "description": "Get recent game events (player joins/leaves, frags, respawns, round start/end). Includes last 50 events.",
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
                        let pid_lock = player_id_clone.lock().await;

                        match snapshot_lock.as_ref() {
                            Some(snapshot) => {
                                let mut observation = snapshot.clone();
                                if let Some(obj) = observation.as_object_mut() {
                                    obj.insert(
                                        "recent_events".to_string(),
                                        serde_json::json!(events_lock.clone()),
                                    );
                                    obj.insert(
                                        "self_player_id".to_string(),
                                        serde_json::json!(pid_lock.map(|id| id.to_string())),
                                    );
                                }
                                observation
                            }
                            None => serde_json::json!({
                                "status": "connecting",
                                "message": "Waiting for first snapshot from server",
                                "self_player_id": pid_lock.map(|id| id.to_string()),
                                "recent_events": events_lock.clone()
                            }),
                        }
                    }

                    "act" => {
                        let arguments = request
                            .params
                            .as_ref()
                            .and_then(|p| p.get("arguments"))
                            .cloned()
                            .unwrap_or(Value::Null);

                        match validate_act_arguments(&arguments) {
                            Ok(action) => {
                                let action_msg = ClientMessage::Action(action);
                                ws_sink
                                    .send(Message::Text(serde_json::to_string(&action_msg)?))
                                    .await?;

                                serde_json::json!({
                                    "content": [{
                                        "type": "text",
                                        "text": "Action sent successfully"
                                    }]
                                })
                            }
                            Err(msg) => serde_json::json!({
                                "content": [{
                                    "type": "text",
                                    "text": msg
                                }],
                                "isError": true
                            }),
                        }
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

const ACT_ALLOWED_KEYS: &[&str] = &[
    "forward",
    "back",
    "left",
    "right",
    "turn_left",
    "turn_right",
    "fire",
    "weapon_swap",
];

/// Validate MCP `act` arguments. Empty/missing args are OK (all defaults).
/// Unknown keys and bad weapon_swap values are schema errors (do not coerce).
fn validate_act_arguments(arguments: &Value) -> Result<protocol::Action, String> {
    if arguments.is_null() {
        return Ok(protocol::Action::default());
    }

    let obj = match arguments.as_object() {
        Some(o) => o,
        None => return Err("schema error: act arguments must be an object".to_string()),
    };

    let mut unknowns: Vec<&str> = obj
        .keys()
        .filter(|k| !ACT_ALLOWED_KEYS.contains(&k.as_str()))
        .map(|k| k.as_str())
        .collect();
    unknowns.sort();
    if !unknowns.is_empty() {
        let listed = unknowns
            .iter()
            .map(|k| format!("'{}'", k))
            .collect::<Vec<_>>()
            .join(", ");
        if unknowns.len() == 1 {
            return Err(format!("schema error: unknown act field {}", listed));
        }
        return Err(format!("schema error: unknown act fields {}", listed));
    }

    let weapon_swap = if let Some(v) = obj.get("weapon_swap") {
        if v.is_null() {
            None
        } else {
            let s = v.as_str().ok_or_else(|| {
                "schema error: weapon_swap must be a string (flechette|rail|scatter)".to_string()
            })?;
            match s {
                "flechette" => Some(protocol::WeaponType::Flechette),
                "rail" => Some(protocol::WeaponType::Rail),
                "scatter" => Some(protocol::WeaponType::Scatter),
                other => {
                    return Err(format!(
                        "schema error: weapon_swap must be flechette|rail|scatter, got '{}'",
                        other
                    ))
                }
            }
        }
    } else {
        None
    };

    let bool_field =
        |key: &str| -> bool { obj.get(key).and_then(|v| v.as_bool()).unwrap_or(false) };

    Ok(protocol::Action {
        forward: bool_field("forward"),
        back: bool_field("back"),
        left: bool_field("left"),
        right: bool_field("right"),
        turn_left: bool_field("turn_left"),
        turn_right: bool_field("turn_right"),
        fire: bool_field("fire"),
        weapon_swap,
    })
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
            killer_score: 5,
        };

        let frag_json = serde_json::to_value(&frag_event).unwrap();
        assert_eq!(frag_json["event"], "frag");
        assert_eq!(frag_json["killer"], "Bot1");
        assert_eq!(frag_json["victim"], "Bot2");
        assert_eq!(frag_json["killer_score"], 5);

        let respawn_event = protocol::GameEvent::Respawn {
            player: "Bot2".to_string(),
        };

        let respawn_json = serde_json::to_value(&respawn_event).unwrap();
        assert_eq!(respawn_json["event"], "respawn");
        assert_eq!(respawn_json["player"], "Bot2");

        let round_start_event = protocol::GameEvent::RoundStart {
            round_number: 1,
            frag_limit: Some(10),
            time_limit: Some(180),
            players: vec!["Bot1".to_string(), "Bot2".to_string()],
            previous_winner: None,
        };

        let round_start_json = serde_json::to_value(&round_start_event).unwrap();
        assert_eq!(round_start_json["event"], "round_start");
        assert_eq!(round_start_json["round_number"], 1);
        assert_eq!(round_start_json["frag_limit"], 10);
        assert_eq!(round_start_json["time_limit"], 180);
        assert_eq!(round_start_json["players"].as_array().unwrap().len(), 2);

        let round_end_event = protocol::GameEvent::RoundEnd {
            winner: Some("Bot1".to_string()),
            reason: "Frag limit reached".to_string(),
            final_scores: vec![
                protocol::PlayerScore {
                    name: "Bot1".to_string(),
                    score: 10,
                },
                protocol::PlayerScore {
                    name: "Bot2".to_string(),
                    score: 3,
                },
            ],
            winner_score: Some(10),
        };

        let round_end_json = serde_json::to_value(&round_end_event).unwrap();
        assert_eq!(round_end_json["event"], "round_end");
        assert_eq!(round_end_json["winner"], "Bot1");
        assert_eq!(round_end_json["reason"], "Frag limit reached");
        assert_eq!(round_end_json["winner_score"], 10);
        assert_eq!(round_end_json["final_scores"].as_array().unwrap().len(), 2);

        let player_joined_event = protocol::GameEvent::PlayerJoined {
            player: "NewPlayer".to_string(),
            role: "agent".to_string(),
            round_number: 2,
            player_count: 5,
        };

        let player_joined_json = serde_json::to_value(&player_joined_event).unwrap();
        assert_eq!(player_joined_json["event"], "player_joined");
        assert_eq!(player_joined_json["player"], "NewPlayer");
        assert_eq!(player_joined_json["role"], "agent");
        assert_eq!(player_joined_json["round_number"], 2);
        assert_eq!(player_joined_json["player_count"], 5);

        let player_left_event = protocol::GameEvent::PlayerLeft {
            player: "OldPlayer".to_string(),
            score: 7,
            round_number: 2,
            player_count: 4,
        };

        let player_left_json = serde_json::to_value(&player_left_event).unwrap();
        assert_eq!(player_left_json["event"], "player_left");
        assert_eq!(player_left_json["player"], "OldPlayer");
        assert_eq!(player_left_json["score"], 7);
        assert_eq!(player_left_json["round_number"], 2);
        assert_eq!(player_left_json["player_count"], 4);
    }

    #[test]
    fn test_server_message_event_parsing() {
        let frag_msg =
            r#"{"type":"event","event":"frag","killer":"Bot1","victim":"Bot2","killer_score":3}"#;
        let parsed: Result<protocol::ServerMessage, _> = serde_json::from_str(frag_msg);
        assert!(parsed.is_ok());

        match parsed.unwrap() {
            protocol::ServerMessage::Event(protocol::GameEvent::Frag {
                killer,
                victim,
                killer_score,
            }) => {
                assert_eq!(killer, "Bot1");
                assert_eq!(victim, "Bot2");
                assert_eq!(killer_score, 3);
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

        let round_start_msg = r#"{"type":"event","event":"round_start","round_number":2,"frag_limit":10,"time_limit":180,"players":["Bot1","Bot2"],"previous_winner":"Bot1"}"#;
        let parsed: Result<protocol::ServerMessage, _> = serde_json::from_str(round_start_msg);
        assert!(parsed.is_ok());

        match parsed.unwrap() {
            protocol::ServerMessage::Event(protocol::GameEvent::RoundStart {
                round_number,
                frag_limit,
                time_limit,
                players,
                previous_winner,
            }) => {
                assert_eq!(round_number, 2);
                assert_eq!(frag_limit, Some(10));
                assert_eq!(time_limit, Some(180));
                assert_eq!(players.len(), 2);
                assert_eq!(previous_winner, Some("Bot1".to_string()));
            }
            _ => panic!("Expected Event(RoundStart)"),
        }

        let round_end_msg = r#"{"type":"event","event":"round_end","winner":"Bot1","reason":"Frag limit reached","final_scores":[{"name":"Bot1","score":10},{"name":"Bot2","score":5}],"winner_score":10}"#;
        let parsed: Result<protocol::ServerMessage, _> = serde_json::from_str(round_end_msg);
        assert!(parsed.is_ok());

        match parsed.unwrap() {
            protocol::ServerMessage::Event(protocol::GameEvent::RoundEnd {
                winner,
                reason,
                final_scores,
                winner_score,
            }) => {
                assert_eq!(winner, Some("Bot1".to_string()));
                assert_eq!(reason, "Frag limit reached");
                assert_eq!(final_scores.len(), 2);
                assert_eq!(final_scores[0].name, "Bot1");
                assert_eq!(final_scores[0].score, 10);
                assert_eq!(winner_score, Some(10));
            }
            _ => panic!("Expected Event(RoundEnd)"),
        }

        let player_joined_msg = r#"{"type":"event","event":"player_joined","player":"NewPlayer","role":"agent","round_number":1,"player_count":3}"#;
        let parsed: Result<protocol::ServerMessage, _> = serde_json::from_str(player_joined_msg);
        assert!(parsed.is_ok());

        match parsed.unwrap() {
            protocol::ServerMessage::Event(protocol::GameEvent::PlayerJoined {
                player,
                role,
                round_number,
                player_count,
            }) => {
                assert_eq!(player, "NewPlayer");
                assert_eq!(role, "agent");
                assert_eq!(round_number, 1);
                assert_eq!(player_count, 3);
            }
            _ => panic!("Expected Event(PlayerJoined)"),
        }

        let player_left_msg = r#"{"type":"event","event":"player_left","player":"OldPlayer","score":5,"round_number":2,"player_count":4}"#;
        let parsed: Result<protocol::ServerMessage, _> = serde_json::from_str(player_left_msg);
        assert!(parsed.is_ok());

        match parsed.unwrap() {
            protocol::ServerMessage::Event(protocol::GameEvent::PlayerLeft {
                player,
                score,
                round_number,
                player_count,
            }) => {
                assert_eq!(player, "OldPlayer");
                assert_eq!(score, 5);
                assert_eq!(round_number, 2);
                assert_eq!(player_count, 4);
            }
            _ => panic!("Expected Event(PlayerLeft)"),
        }
    }

    #[test]
    fn test_welcome_message_parsing() {
        let welcome_msg = r#"{"type":"welcome","player_id":"550e8400-e29b-41d4-a716-446655440000","role":"agent"}"#;
        let parsed: Result<protocol::ServerMessage, _> = serde_json::from_str(welcome_msg);
        assert!(parsed.is_ok());

        match parsed.unwrap() {
            protocol::ServerMessage::Welcome { player_id, role } => {
                assert!(player_id.is_some());
                assert_eq!(role, protocol::Role::Agent);
            }
            _ => panic!("Expected Welcome"),
        }

        let spectator_welcome = r#"{"type":"welcome","player_id":null,"role":"spectator"}"#;
        let parsed: Result<protocol::ServerMessage, _> = serde_json::from_str(spectator_welcome);
        assert!(parsed.is_ok());

        match parsed.unwrap() {
            protocol::ServerMessage::Welcome { player_id, role } => {
                assert!(player_id.is_none());
                assert_eq!(role, protocol::Role::Spectator);
            }
            _ => panic!("Expected Welcome"),
        }
    }

    #[test]
    fn test_event_buffer_rotation() {
        let mut buffer = Vec::new();

        for i in 0..60 {
            let event = serde_json::json!({"event": "test", "index": i});
            buffer.push(event);
            if buffer.len() > 50 {
                buffer.remove(0);
            }
        }

        assert_eq!(buffer.len(), 50);
        assert_eq!(buffer.first().unwrap()["index"], 10);
        assert_eq!(buffer.last().unwrap()["index"], 59);
    }

    #[test]
    fn test_action_defaults() {
        let action = protocol::Action::default();
        assert!(!action.forward);
        assert!(!action.back);
        assert!(!action.left);
        assert!(!action.right);
        assert!(!action.turn_left);
        assert!(!action.turn_right);
        assert!(!action.fire);
        assert!(action.weapon_swap.is_none());
    }

    #[test]
    fn test_action_serialization() {
        let action = protocol::Action {
            forward: true,
            fire: true,
            ..Default::default()
        };

        let serialized = serde_json::to_string(&ClientMessage::Action(action)).unwrap();
        assert!(serialized.contains(r#""forward":true"#));
        assert!(serialized.contains(r#""fire":true"#));
        assert!(serialized.contains(r#""back":false"#));
    }

    #[test]
    fn test_weapon_swap_serialization() {
        let action = protocol::Action {
            weapon_swap: Some(protocol::WeaponType::Rail),
            ..Default::default()
        };

        let serialized = serde_json::to_string(&ClientMessage::Action(action)).unwrap();
        assert!(serialized.contains(r#""weapon_swap":"rail""#));

        let action_flechette = protocol::Action {
            weapon_swap: Some(protocol::WeaponType::Flechette),
            ..Default::default()
        };

        let serialized = serde_json::to_string(&ClientMessage::Action(action_flechette)).unwrap();
        assert!(serialized.contains(r#""weapon_swap":"flechette""#));

        let action_scatter = protocol::Action {
            weapon_swap: Some(protocol::WeaponType::Scatter),
            ..Default::default()
        };

        let serialized = serde_json::to_string(&ClientMessage::Action(action_scatter)).unwrap();
        assert!(serialized.contains(r#""weapon_swap":"scatter""#));
    }

    #[test]
    fn test_weapon_swap_none_serialization() {
        let action = protocol::Action {
            forward: true,
            weapon_swap: None,
            ..Default::default()
        };

        let serialized = serde_json::to_string(&ClientMessage::Action(action)).unwrap();
        assert!(serialized.contains(r#""forward":true"#));
    }

    #[test]
    fn test_weapon_type_parsing() {
        let rail_json = r#""rail""#;
        let parsed: protocol::WeaponType = serde_json::from_str(rail_json).unwrap();
        assert_eq!(parsed, protocol::WeaponType::Rail);

        let flechette_json = r#""flechette""#;
        let parsed: protocol::WeaponType = serde_json::from_str(flechette_json).unwrap();
        assert_eq!(parsed, protocol::WeaponType::Flechette);

        let scatter_json = r#""scatter""#;
        let parsed: protocol::WeaponType = serde_json::from_str(scatter_json).unwrap();
        assert_eq!(parsed, protocol::WeaponType::Scatter);
    }

    #[test]
    fn test_malformed_event_handling() {
        let bad_json = r#"{"type":"event","event":"invalid_event_type"}"#;
        let parsed: Result<protocol::ServerMessage, _> = serde_json::from_str(bad_json);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_snapshot_with_players() {
        let snapshot = protocol::Snapshot {
            tick: 100,
            players: vec![protocol::PlayerState {
                id: uuid::Uuid::new_v4(),
                name: "TestBot".to_string(),
                x: 10.0,
                y: 1.5,
                z: -5.0,
                yaw: 1.57,
                hp: 75,
                just_fired: false,
                behavior: Some("Aggressive".to_string()),
                score: 3,
                weapon: "Rail".to_string(),
            }],
            round_state: Some("Active".to_string()),
            round_time_left: Some(120),
            frag_limit: Some(10),
        };

        let json = serde_json::to_value(&snapshot).unwrap();
        assert_eq!(json["tick"], 100);
        assert_eq!(json["players"][0]["name"], "TestBot");
        assert_eq!(json["players"][0]["hp"], 75);
        assert_eq!(json["players"][0]["x"], 10.0);
        assert_eq!(json["players"][0]["weapon"], "Rail");
        assert_eq!(json["players"][0]["score"], 3);
        assert_eq!(json["players"][0]["behavior"], "Aggressive");
        assert_eq!(json["round_state"], "Active");
        assert_eq!(json["round_time_left"], 120);
        assert_eq!(json["frag_limit"], 10);
    }

    #[test]
    fn test_snapshot_includes_weapon_in_observe() {
        let snapshot = protocol::Snapshot {
            tick: 50,
            players: vec![
                protocol::PlayerState {
                    id: uuid::Uuid::new_v4(),
                    name: "Agent1".to_string(),
                    x: 5.0,
                    y: 1.5,
                    z: 5.0,
                    yaw: 0.0,
                    hp: 100,
                    just_fired: true,
                    behavior: None,
                    score: 5,
                    weapon: "Flechette".to_string(),
                },
                protocol::PlayerState {
                    id: uuid::Uuid::new_v4(),
                    name: "Agent2".to_string(),
                    x: -5.0,
                    y: 1.5,
                    z: -5.0,
                    yaw: std::f32::consts::PI,
                    hp: 50,
                    just_fired: false,
                    behavior: None,
                    score: 2,
                    weapon: "Scatter".to_string(),
                },
            ],
            round_state: Some("Active".to_string()),
            round_time_left: Some(90),
            frag_limit: Some(10),
        };

        let json = serde_json::to_value(&snapshot).unwrap();
        assert_eq!(json["players"][0]["weapon"], "Flechette");
        assert_eq!(json["players"][1]["weapon"], "Scatter");
        assert_eq!(json["players"][0]["score"], 5);
        assert_eq!(json["players"][1]["score"], 2);
    }

    #[test]
    fn test_mcp_request_parsing() {
        let valid_req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
        let parsed: Result<McpRequest, _> = serde_json::from_str(valid_req);
        assert!(parsed.is_ok());
        let req = parsed.unwrap();
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "tools/list");
    }

    #[test]
    fn test_mcp_error_response() {
        let error = McpError {
            code: -32700,
            message: "Parse error".to_string(),
        };
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            id: Value::Null,
            result: None,
            error: Some(error),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("-32700"));
        assert!(json.contains("Parse error"));
    }

    #[test]
    fn test_client_action_message_structure() {
        let hello = ClientMessage::Hello {
            role: protocol::Role::Agent,
            name: "TestAgent".to_string(),
        };
        let json = serde_json::to_string(&hello).unwrap();
        assert!(json.contains(r#""type":"hello""#));
        assert!(json.contains(r#""role":"agent""#));
        assert!(json.contains("TestAgent"));

        let action = ClientMessage::Action(protocol::Action::default());
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains(r#""type":"action""#));
    }

    #[test]
    fn test_join_leave_event_parsing() {
        let join_msg = r#"{"type":"event","event":"player_joined","player":"Agent1","role":"agent","round_number":1,"player_count":5}"#;
        let parsed: Result<protocol::ServerMessage, _> = serde_json::from_str(join_msg);
        assert!(parsed.is_ok(), "Failed to parse join event: {:?}", parsed);

        match parsed.unwrap() {
            protocol::ServerMessage::Event(protocol::GameEvent::PlayerJoined {
                player,
                role,
                round_number,
                player_count,
            }) => {
                assert_eq!(player, "Agent1");
                assert_eq!(role, "agent");
                assert_eq!(round_number, 1);
                assert_eq!(player_count, 5);
            }
            _ => panic!("Expected Event(PlayerJoined)"),
        }

        let leave_msg = r#"{"type":"event","event":"player_left","player":"Agent1","score":7,"round_number":2,"player_count":4}"#;
        let parsed: Result<protocol::ServerMessage, _> = serde_json::from_str(leave_msg);
        assert!(parsed.is_ok(), "Failed to parse leave event: {:?}", parsed);

        match parsed.unwrap() {
            protocol::ServerMessage::Event(protocol::GameEvent::PlayerLeft {
                player,
                score,
                round_number,
                player_count,
            }) => {
                assert_eq!(player, "Agent1");
                assert_eq!(score, 7);
                assert_eq!(round_number, 2);
                assert_eq!(player_count, 4);
            }
            _ => panic!("Expected Event(PlayerLeft)"),
        }
    }

    #[test]
    fn test_join_leave_events_in_buffer() {
        let mut buffer = Vec::new();

        let join_event = serde_json::json!({
            "event": "player_joined",
            "player": "TestAgent",
            "role": "agent",
            "round_number": 1,
            "player_count": 5
        });
        buffer.push(join_event);

        let leave_event = serde_json::json!({
            "event": "player_left",
            "player": "TestAgent",
            "score": 3,
            "round_number": 1,
            "player_count": 4
        });
        buffer.push(leave_event);

        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer[0]["event"], "player_joined");
        assert_eq!(buffer[0]["player"], "TestAgent");
        assert_eq!(buffer[0]["role"], "agent");
        assert_eq!(buffer[1]["event"], "player_left");
        assert_eq!(buffer[1]["player"], "TestAgent");
        assert_eq!(buffer[1]["score"], 3);
    }

    #[test]
    fn test_action_unknown_field_fails_deserialize() {
        let json = r#"{"type":"action","forward":true,"laser":true}"#;
        let parsed: Result<ClientMessage, _> = serde_json::from_str(json);
        assert!(
            parsed.is_err(),
            "unknown Action field must fail deserialize: {:?}",
            parsed
        );
    }

    #[test]
    fn test_action_valid_deserializes() {
        let json = r#"{"type":"action","forward":true,"fire":true,"weapon_swap":"rail"}"#;
        let parsed: Result<ClientMessage, _> = serde_json::from_str(json);
        assert!(parsed.is_ok(), "{:?}", parsed);
        match parsed.unwrap() {
            ClientMessage::Action(a) => {
                assert!(a.forward);
                assert!(a.fire);
                assert_eq!(a.weapon_swap, Some(protocol::WeaponType::Rail));
            }
            other => panic!("expected Action, got {:?}", other),
        }
    }

    #[test]
    fn test_validate_act_arguments_unknown_key() {
        let args = serde_json::json!({"laser": true, "forward": true});
        let err = validate_act_arguments(&args).unwrap_err();
        assert!(err.contains("schema error"));
        assert!(err.contains("laser"), "err={}", err);
        assert!(!err.contains("forward") || err.contains("unknown"));
    }

    #[test]
    fn test_validate_act_arguments_bad_weapon_swap() {
        let args = serde_json::json!({"weapon_swap": "potato"});
        let err = validate_act_arguments(&args).unwrap_err();
        assert!(err.contains("schema error"));
        assert!(err.contains("weapon_swap"), "err={}", err);
        assert!(err.contains("potato"), "err={}", err);
    }

    #[test]
    fn test_validate_act_arguments_valid_fire_forward() {
        let args = serde_json::json!({"fire": true, "forward": true});
        let action = validate_act_arguments(&args).expect("valid act");
        assert!(action.fire);
        assert!(action.forward);
        assert!(!action.back);
        assert!(action.weapon_swap.is_none());
    }

    #[test]
    fn test_validate_act_arguments_empty_ok() {
        assert!(validate_act_arguments(&Value::Null).is_ok());
        assert!(validate_act_arguments(&serde_json::json!({})).is_ok());
    }
}
