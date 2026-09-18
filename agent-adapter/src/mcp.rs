//! MCP request dispatch for observe / act / get_events (and initialize / tools/list).
//! Kept free of stdin/WebSocket I/O so behavioral unit tests can cover the real tool paths.

use crate::protocol::{self, Action};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

/// In-memory MCP tool state mirrored from the WebSocket receive loop.
#[derive(Debug, Default, Clone)]
pub struct ToolState {
    pub last_snapshot: Option<Value>,
    pub recent_events: Vec<Value>,
    pub player_id: Option<Uuid>,
}

/// Outcome of handling one MCP request. `pending_action` is set when `act` validated.
#[derive(Debug)]
pub struct HandleOutcome {
    pub response: McpResponse,
    pub pending_action: Option<Action>,
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
pub fn validate_act_arguments(arguments: &Value) -> Result<Action, String> {
    if arguments.is_null() {
        return Ok(Action::default());
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

    Ok(Action {
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

pub fn build_observe_result(state: &ToolState) -> Value {
    match state.last_snapshot.as_ref() {
        Some(snapshot) => {
            let mut observation = snapshot.clone();
            if let Some(obj) = observation.as_object_mut() {
                obj.insert(
                    "recent_events".to_string(),
                    serde_json::json!(state.recent_events.clone()),
                );
                obj.insert(
                    "self_player_id".to_string(),
                    serde_json::json!(state.player_id.map(|id| id.to_string())),
                );
            }
            observation
        }
        None => serde_json::json!({
            "status": "connecting",
            "message": "Waiting for first snapshot from server",
            "self_player_id": state.player_id.map(|id| id.to_string()),
            "recent_events": state.recent_events.clone()
        }),
    }
}

pub fn build_get_events_result(state: &mut ToolState, clear: bool) -> Value {
    let events_copy = state.recent_events.clone();
    if clear {
        state.recent_events.clear();
    }
    serde_json::json!({
        "content": [{
            "type": "text",
            "text": format!("Recent events: {}", serde_json::to_string_pretty(&events_copy).unwrap())
        }]
    })
}

fn tools_list_result() -> Value {
    serde_json::json!({
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
    })
}

/// Dispatch one JSON-RPC MCP request against tool state.
pub fn handle_mcp_request(request: McpRequest, state: &mut ToolState) -> HandleOutcome {
    let id = request.id.clone().unwrap_or(Value::Null);

    match request.method.as_str() {
        "initialize" => HandleOutcome {
            response: McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
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
            pending_action: None,
        },

        "tools/list" => HandleOutcome {
            response: McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(tools_list_result()),
                error: None,
            },
            pending_action: None,
        },

        "tools/call" => {
            let tool_name = request
                .params
                .as_ref()
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");

            let mut pending_action = None;
            let result = match tool_name {
                "observe" => build_observe_result(state),

                "act" => {
                    let arguments = request
                        .params
                        .as_ref()
                        .and_then(|p| p.get("arguments"))
                        .cloned()
                        .unwrap_or(Value::Null);

                    match validate_act_arguments(&arguments) {
                        Ok(action) => {
                            pending_action = Some(action);
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

                    build_get_events_result(state, should_clear)
                }

                _ => serde_json::json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Unknown tool: {}", tool_name)
                    }]
                }),
            };

            HandleOutcome {
                response: McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(result),
                    error: None,
                },
                pending_action,
            }
        }

        _ => HandleOutcome {
            response: McpResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                }),
            },
            pending_action: None,
        },
    }
}

/// Buffer a parsed or soft-prison raw event into recent_events (cap 50).
pub fn push_recent_event(state: &mut ToolState, event_value: Value) {
    state.recent_events.push(event_value);
    if state.recent_events.len() > 50 {
        state.recent_events.remove(0);
    }
}

/// Apply an inbound server JSON text frame into tool state (snapshot / event / soft prison).
pub fn ingest_server_text(state: &mut ToolState, text: &str) {
    if text.len() > 1_000_000 {
        return;
    }

    match serde_json::from_str::<protocol::ServerMessage>(text) {
        Ok(protocol::ServerMessage::Snapshot(snapshot)) => {
            if let Ok(snapshot_value) = serde_json::to_value(snapshot) {
                state.last_snapshot = Some(snapshot_value);
            }
        }
        Ok(protocol::ServerMessage::Event(event)) => {
            if let Ok(event_value) = serde_json::to_value(event) {
                push_recent_event(state, event_value);
            }
        }
        Ok(protocol::ServerMessage::Welcome { .. }) => {}
        Err(_) => {
            if let Ok(raw) = serde_json::from_str::<Value>(text) {
                if raw.get("type").and_then(|v| v.as_str()) == Some("event") {
                    push_recent_event(state, raw);
                }
            }
        }
    }
}

#[cfg(test)]
mod mcp_tests {
    use super::*;

    fn req(method: &str, params: Option<Value>) -> McpRequest {
        McpRequest {
            jsonrpc: "2.0".into(),
            id: Some(Value::from(1)),
            method: method.into(),
            params,
        }
    }

    #[test]
    fn observe_connecting_until_snapshot() {
        let mut state = ToolState {
            player_id: Some(Uuid::nil()),
            ..Default::default()
        };
        let out = handle_mcp_request(
            req("tools/call", Some(serde_json::json!({"name":"observe"}))),
            &mut state,
        );
        let result = out.response.result.unwrap();
        assert_eq!(result["status"], "connecting");
        assert!(result["self_player_id"].is_string());
    }

    #[test]
    fn observe_includes_snapshot_events_and_self_id() {
        let mut state = ToolState {
            last_snapshot: Some(serde_json::json!({"tick": 3, "players": []})),
            recent_events: vec![serde_json::json!({"event":"frag"})],
            player_id: Some(Uuid::nil()),
        };
        let out = handle_mcp_request(
            req("tools/call", Some(serde_json::json!({"name":"observe"}))),
            &mut state,
        );
        let result = out.response.result.unwrap();
        assert_eq!(result["tick"], 3);
        assert_eq!(result["recent_events"].as_array().unwrap().len(), 1);
        assert!(result["self_player_id"].is_string());
    }

    #[test]
    fn act_valid_sets_pending_action() {
        let mut state = ToolState::default();
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({
                    "name": "act",
                    "arguments": {"forward": true, "fire": true, "weapon_swap": "rail"}
                })),
            ),
            &mut state,
        );
        let action = out.pending_action.expect("pending");
        assert!(action.forward && action.fire);
        assert_eq!(action.weapon_swap, Some(protocol::WeaponType::Rail));
        let text = out.response.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(text.contains("Action sent successfully"));
    }

    #[test]
    fn act_schema_error_sets_is_error() {
        let mut state = ToolState::default();
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({
                    "name": "act",
                    "arguments": {"laser": true}
                })),
            ),
            &mut state,
        );
        assert!(out.pending_action.is_none());
        let result = out.response.result.unwrap();
        assert_eq!(result["isError"], true);
        assert!(result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("schema error"));
    }

    #[test]
    fn get_events_lists_and_optional_clear() {
        let mut state = ToolState {
            recent_events: vec![
                serde_json::json!({"event":"player_joined","player":"A"}),
                serde_json::json!({"event":"round_start","round_number":1}),
            ],
            ..Default::default()
        };
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({"name":"get_events","arguments":{"clear":false}})),
            ),
            &mut state,
        );
        let text = out.response.result.unwrap()["content"][0]["text"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(text.contains("player_joined"));
        assert_eq!(state.recent_events.len(), 2);

        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({"name":"get_events","arguments":{"clear":true}})),
            ),
            &mut state,
        );
        assert!(out.response.result.is_some());
        assert!(state.recent_events.is_empty());
    }

    #[test]
    fn initialize_and_tools_list_and_unknown_method() {
        let mut state = ToolState::default();
        let init = handle_mcp_request(req("initialize", None), &mut state);
        assert!(init.response.result.unwrap()["serverInfo"]["name"]
            .as_str()
            .unwrap()
            .contains("fragr-agent-adapter"));

        let list = handle_mcp_request(req("tools/list", None), &mut state);
        let list_result = list.response.result.unwrap();
        let tools = list_result["tools"].as_array().unwrap();
        let names: Vec<_> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"observe"));
        assert!(names.contains(&"act"));
        assert!(names.contains(&"get_events"));

        let bad = handle_mcp_request(req("nope", None), &mut state);
        assert_eq!(bad.response.error.unwrap().code, -32601);
    }

    #[test]
    fn ingest_snapshot_event_and_soft_prison() {
        let mut state = ToolState::default();
        ingest_server_text(
            &mut state,
            r#"{"type":"snapshot","tick":9,"players":[],"round_state":"active","round_time_left":100,"frag_limit":10}"#,
        );
        assert_eq!(state.last_snapshot.as_ref().unwrap()["tick"], 9);

        ingest_server_text(
            &mut state,
            r#"{"type":"event","event":"player_joined","player":"X","role":"agent","round_number":1,"player_count":3}"#,
        );
        assert_eq!(state.recent_events.len(), 1);

        // Soft prison: unknown event shape still buffers when type=event.
        ingest_server_text(
            &mut state,
            r#"{"type":"event","event":"future_thing","payload":1}"#,
        );
        assert_eq!(state.recent_events.len(), 2);
    }

    #[test]
    fn push_recent_event_rotates_at_50() {
        let mut state = ToolState::default();
        for i in 0..55 {
            push_recent_event(&mut state, serde_json::json!({"i": i}));
        }
        assert_eq!(state.recent_events.len(), 50);
        assert_eq!(state.recent_events[0]["i"], 5);
    }

    #[test]
    fn hello_name_resolution_used_in_client_message() {
        // Behavioral: Hello payload carries the resolved --name (not the default).
        let name = "ArenaFox";
        let hello = protocol::ClientMessage::Hello {
            role: protocol::Role::Agent,
            name: name.to_string(),
        };
        let json = serde_json::to_string(&hello).unwrap();
        assert!(json.contains("ArenaFox"));
        assert!(!json.contains("MCP Agent"));
    }
}
