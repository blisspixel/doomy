//! MCP request dispatch for observe / act / get_events / speak / join / leave / round_state
//! (and initialize / tools/list).
//! Kept free of stdin/WebSocket I/O so behavioral unit tests can cover the real tool paths.

use crate::protocol::{self, Action, Speak};
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

/// Mirrored speak cooldown (~3s at 20 Hz). Same as server SPEAK_COOLDOWN_TICKS.
pub const SPEAK_COOLDOWN_TICKS: u64 = 60;

/// In-memory MCP tool state mirrored from the WebSocket receive loop.
#[derive(Debug, Clone)]
pub struct ToolState {
    pub last_snapshot: Option<Value>,
    pub recent_events: Vec<Value>,
    pub player_id: Option<Uuid>,
    /// Tick of last MCP speak that was accepted for send (rate-limit honesty).
    pub last_speak_tick: Option<u64>,
    /// True after Welcome / successful join; false after leave or before join.
    pub connected: bool,
    /// Display name from `--name` / `FRAGR_AGENT_NAME` (join reuses when args omit name).
    pub default_name: String,
    /// Name used for the current joined session.
    pub session_name: Option<String>,
}

impl Default for ToolState {
    fn default() -> Self {
        Self {
            last_snapshot: None,
            recent_events: Vec::new(),
            player_id: None,
            last_speak_tick: None,
            connected: false,
            default_name: "MCP Agent".to_string(),
            session_name: None,
        }
    }
}

/// Outcome of handling one MCP request.
/// `pending_action` is set when `act` validated; `pending_speak` when `speak` validated.
/// `pending_join` / `pending_leave` drive WebSocket Hello reconnect / clean disconnect in main.
#[derive(Debug)]
pub struct HandleOutcome {
    pub response: McpResponse,
    pub pending_action: Option<Action>,
    pub pending_speak: Option<Speak>,
    pub pending_join: Option<String>,
    pub pending_leave: bool,
}

fn empty_outcome(response: McpResponse) -> HandleOutcome {
    HandleOutcome {
        response,
        pending_action: None,
        pending_speak: None,
        pending_join: None,
        pending_leave: false,
    }
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
    "look_at",
];

const LOOK_AT_ALLOWED_KEYS: &[&str] = &["x", "z", "player_id"];

const JOIN_ALLOWED_KEYS: &[&str] = &["name"];

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

    let look_at = if let Some(v) = obj.get("look_at") {
        if v.is_null() {
            None
        } else {
            let look_obj = v
                .as_object()
                .ok_or_else(|| "schema error: look_at must be an object".to_string())?;
            let mut unknowns: Vec<&str> = look_obj
                .keys()
                .filter(|k| !LOOK_AT_ALLOWED_KEYS.contains(&k.as_str()))
                .map(|k| k.as_str())
                .collect();
            unknowns.sort();
            if !unknowns.is_empty() {
                let listed = unknowns
                    .iter()
                    .map(|k| format!("'{}'", k))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(format!("schema error: unknown look_at field(s) {}", listed));
            }

            let num_field = |key: &str| -> Result<Option<f32>, String> {
                match look_obj.get(key) {
                    None => Ok(None),
                    Some(val) if val.is_null() => Ok(None),
                    Some(val) => val
                        .as_f64()
                        .map(|n| Some(n as f32))
                        .ok_or_else(|| format!("schema error: look_at.{} must be a number", key)),
                }
            };
            let player_id = match look_obj.get("player_id") {
                None => None,
                Some(val) if val.is_null() => None,
                Some(val) => {
                    let s = val.as_str().ok_or_else(|| {
                        "schema error: look_at.player_id must be a uuid string".to_string()
                    })?;
                    Some(Uuid::parse_str(s).map_err(|_| {
                        format!(
                            "schema error: look_at.player_id must be a valid uuid, got '{}'",
                            s
                        )
                    })?)
                }
            };
            let x = num_field("x")?;
            let z = num_field("z")?;
            if player_id.is_none() && (x.is_none() || z.is_none()) {
                return Err("schema error: look_at requires player_id or both x and z".to_string());
            }
            Some(protocol::LookAt { x, z, player_id })
        }
    } else {
        None
    };

    Ok(Action {
        forward: bool_field("forward"),
        back: bool_field("back"),
        left: bool_field("left"),
        right: bool_field("right"),
        turn_left: bool_field("turn_left"),
        turn_right: bool_field("turn_right"),
        fire: bool_field("fire"),
        weapon_swap,
        look_at,
    })
}

pub const SPEAK_MAX_CHARS: usize = 80;

/// Validate MCP `speak` arguments. Returns a Speak payload or a clear schema error.
pub fn validate_speak_arguments(arguments: &Value) -> Result<Speak, String> {
    let obj = match arguments.as_object() {
        Some(o) => o,
        None => return Err("schema error: speak arguments must be an object".to_string()),
    };

    let mut unknowns: Vec<&str> = obj
        .keys()
        .filter(|k| k.as_str() != "text")
        .map(|k| k.as_str())
        .collect();
    unknowns.sort();
    if !unknowns.is_empty() {
        let listed = unknowns
            .iter()
            .map(|k| format!("'{}'", k))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!("schema error: unknown speak field(s) {}", listed));
    }

    let text_val = obj
        .get("text")
        .ok_or_else(|| "schema error: speak requires 'text'".to_string())?;
    let raw = text_val
        .as_str()
        .ok_or_else(|| "schema error: speak.text must be a string".to_string())?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("schema error: speak.text must be non-empty".to_string());
    }
    if trimmed.chars().count() > SPEAK_MAX_CHARS {
        return Err(format!(
            "schema error: speak.text max length is {} characters",
            SPEAK_MAX_CHARS
        ));
    }
    if trimmed.chars().any(|c| c.is_control()) {
        return Err("schema error: speak.text must not contain control characters".to_string());
    }

    Ok(Speak {
        text: trimmed.to_string(),
    })
}

/// Validate MCP `join` arguments. Optional `name`; otherwise reuse default_name.
pub fn validate_join_arguments(arguments: &Value, default_name: &str) -> Result<String, String> {
    if arguments.is_null() {
        return Ok(default_name.to_string());
    }

    let obj = match arguments.as_object() {
        Some(o) => o,
        None => return Err("schema error: join arguments must be an object".to_string()),
    };

    let mut unknowns: Vec<&str> = obj
        .keys()
        .filter(|k| !JOIN_ALLOWED_KEYS.contains(&k.as_str()))
        .map(|k| k.as_str())
        .collect();
    unknowns.sort();
    if !unknowns.is_empty() {
        let listed = unknowns
            .iter()
            .map(|k| format!("'{}'", k))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!("schema error: unknown join field(s) {}", listed));
    }

    match obj.get("name") {
        None => Ok(default_name.to_string()),
        Some(v) if v.is_null() => Ok(default_name.to_string()),
        Some(v) => {
            let s = v
                .as_str()
                .ok_or_else(|| "schema error: join.name must be a string".to_string())?;
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return Err("schema error: join.name must be non-empty".to_string());
            }
            Ok(trimmed.to_string())
        }
    }
}

/// Validate MCP `leave` / `round_state` arguments: empty object or null only.
pub fn validate_no_arg_tool(tool: &str, arguments: &Value) -> Result<(), String> {
    if arguments.is_null() {
        return Ok(());
    }
    let obj = match arguments.as_object() {
        Some(o) => o,
        None => {
            return Err(format!(
                "schema error: {} arguments must be an object",
                tool
            ))
        }
    };
    if !obj.is_empty() {
        let mut keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
        keys.sort();
        let listed = keys
            .iter()
            .map(|k| format!("'{}'", k))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "schema error: {} takes no fields; unknown field(s) {}",
            tool, listed
        ));
    }
    Ok(())
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

fn snap_field(snap: Option<&Value>, key: &str) -> Value {
    snap.and_then(|s| s.get(key))
        .cloned()
        .unwrap_or(Value::Null)
}

/// Round summary from last snapshot + recent round_start / round_end (no observe scrape).
pub fn build_round_state_result(state: &ToolState) -> Value {
    let snap = state.last_snapshot.as_ref();
    let mut round_number = Value::Null;
    let mut last_round_start = Value::Null;
    let mut last_round_end = Value::Null;

    for ev in state.recent_events.iter().rev() {
        match ev.get("event").and_then(|e| e.as_str()) {
            Some("round_start") if last_round_start.is_null() => {
                last_round_start = ev.clone();
                if let Some(n) = ev.get("round_number") {
                    round_number = n.clone();
                }
            }
            Some("round_end") if last_round_end.is_null() => {
                last_round_end = ev.clone();
            }
            _ => {}
        }
    }

    // Prefer live snapshot mode/host/pressure; fall back to last round_start fields.
    let mode_name = match snap_field(snap, "mode_name") {
        Value::Null => last_round_start
            .get("mode_name")
            .cloned()
            .unwrap_or(Value::Null),
        other => other,
    };
    let host_line = match snap_field(snap, "host_line") {
        Value::Null => last_round_start
            .get("host_line")
            .cloned()
            .unwrap_or(Value::Null),
        other => other,
    };
    let frag_limit = match snap_field(snap, "frag_limit") {
        Value::Null => last_round_start
            .get("frag_limit")
            .cloned()
            .unwrap_or(Value::Null),
        other => other,
    };

    serde_json::json!({
        "connected": state.connected,
        "self_player_id": state.player_id.map(|id| id.to_string()),
        "session_name": state.session_name,
        "round_state": snap_field(snap, "round_state"),
        "round_number": round_number,
        "round_time_left": snap_field(snap, "round_time_left"),
        "frag_limit": frag_limit,
        "mode_name": mode_name,
        "playlist": snap_field(snap, "playlist"),
        "host_line": host_line,
        "pressure": snap_field(snap, "pressure"),
        "last_round_start": last_round_start,
        "last_round_end": last_round_end
    })
}

fn tools_list_result() -> Value {
    serde_json::json!({
        "tools": [
            {
                "name": "observe",
                "description": "Get current game state observation including self_player_id, recent events, shot_results (hit-confirm), and pickups (mid-map weapon / health / armor pads; pickup events carry kind). Returns connecting state until first snapshot arrives.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            },
            {
                "name": "act",
                "description": "Send action to the game server. Actions are level-held (sticky) within each tick window. Set true to activate, false to deactivate. Weapon swap changes loadout. look_at aims (server applies yaw).",
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
                        "weapon_swap": {"type": "string", "enum": ["flechette", "rail", "scatter"], "description": "Switch to weapon type"},
                        "look_at": {
                            "type": "object",
                            "description": "Aim: server sets yaw toward player_id (preferred) or world x/z",
                            "properties": {
                                "player_id": {"type": "string", "description": "Target player UUID"},
                                "x": {"type": "number", "description": "World X target"},
                                "z": {"type": "number", "description": "World Z target"}
                            },
                            "additionalProperties": false
                        }
                    },
                    "required": []
                }
            },
            {
                "name": "get_events",
                "description": "Get recent game events (player joins/leaves, frags, respawns, round start/end, speaks). Includes last 50 events.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "clear": {"type": "boolean", "default": false, "description": "Clear events after retrieving"}
                    },
                    "required": []
                }
            },
            {
                "name": "speak",
                "description": "Send a short off-tick taunt/callout (rate-limited, max 80 chars). Not on the combat Action tick. Spectators see it; it appears in recent_events/get_events.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": {"type": "string", "description": "Callout text (trimmed, max 80 chars, no control characters)"}
                    },
                    "required": ["text"],
                    "additionalProperties": false
                }
            },
            {
                "name": "join",
                "description": "Join the arena as an agent (Hello/Welcome). Optional name reuses --name / FRAGR_AGENT_NAME when omitted. Idempotent if already joined.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string", "description": "Display name for Hello (optional; defaults to adapter --name)"}
                    },
                    "required": [],
                    "additionalProperties": false
                }
            },
            {
                "name": "leave",
                "description": "Leave the arena with a clean WebSocket disconnect. Returns isError if not connected.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }
            },
            {
                "name": "round_state",
                "description": "Current round summary (state, number, time left, frag limit, mode_name, host_line, pressure) from last snapshot plus recent round_start/round_end. Prefer this over scraping observe.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
                }
            }
        ]
    })
}

fn snapshot_tick(state: &ToolState) -> u64 {
    state
        .last_snapshot
        .as_ref()
        .and_then(|s| s.get("tick"))
        .and_then(|t| t.as_u64())
        .unwrap_or(0)
}

fn speak_rate_limited(state: &ToolState, tick: u64) -> bool {
    match state.last_speak_tick {
        Some(last) => tick.saturating_sub(last) < SPEAK_COOLDOWN_TICKS,
        None => false,
    }
}

fn tool_error_result(message: &str) -> Value {
    serde_json::json!({
        "content": [{
            "type": "text",
            "text": message
        }],
        "isError": true
    })
}

fn tool_ok_text(message: &str) -> Value {
    serde_json::json!({
        "content": [{
            "type": "text",
            "text": message
        }]
    })
}

/// Dispatch one JSON-RPC MCP request against tool state.
pub fn handle_mcp_request(request: McpRequest, state: &mut ToolState) -> HandleOutcome {
    let id = request.id.clone().unwrap_or(Value::Null);

    match request.method.as_str() {
        "initialize" => empty_outcome(McpResponse {
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
        }),

        "tools/list" => empty_outcome(McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(tools_list_result()),
            error: None,
        }),

        "tools/call" => {
            let tool_name = request
                .params
                .as_ref()
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");

            let mut pending_action = None;
            let mut pending_speak = None;
            let mut pending_join = None;
            let mut pending_leave = false;
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

                "speak" => {
                    let arguments = request
                        .params
                        .as_ref()
                        .and_then(|p| p.get("arguments"))
                        .cloned()
                        .unwrap_or(Value::Null);

                    match validate_speak_arguments(&arguments) {
                        Ok(speak) => {
                            if state.player_id.is_none() {
                                tool_error_result(
                                    "speak rejected: not connected as a player (spectators cannot speak)",
                                )
                            } else {
                                let tick = snapshot_tick(state);
                                if speak_rate_limited(state, tick) {
                                    tool_error_result(
                                        "speak rate limited; try again in a few seconds",
                                    )
                                } else {
                                    state.last_speak_tick = Some(tick);
                                    pending_speak = Some(speak);
                                    tool_ok_text("Speak sent successfully")
                                }
                            }
                        }
                        Err(msg) => tool_error_result(&msg),
                    }
                }

                "join" => {
                    let arguments = request
                        .params
                        .as_ref()
                        .and_then(|p| p.get("arguments"))
                        .cloned()
                        .unwrap_or(Value::Null);

                    match validate_join_arguments(&arguments, &state.default_name) {
                        Ok(join_name) => {
                            if state.connected {
                                let who = state
                                    .session_name
                                    .clone()
                                    .unwrap_or_else(|| join_name.clone());
                                tool_ok_text(&format!("Already joined as '{}'", who))
                            } else {
                                pending_join = Some(join_name.clone());
                                tool_ok_text(&format!("Joining as '{}'; Hello pending", join_name))
                            }
                        }
                        Err(msg) => tool_error_result(&msg),
                    }
                }

                "leave" => {
                    let arguments = request
                        .params
                        .as_ref()
                        .and_then(|p| p.get("arguments"))
                        .cloned()
                        .unwrap_or(Value::Null);

                    match validate_no_arg_tool("leave", &arguments) {
                        Ok(()) => {
                            if !state.connected {
                                tool_error_result("leave rejected: not connected")
                            } else {
                                state.connected = false;
                                state.player_id = None;
                                state.session_name = None;
                                state.last_snapshot = None;
                                state.last_speak_tick = None;
                                pending_leave = true;
                                tool_ok_text("Left arena; disconnecting")
                            }
                        }
                        Err(msg) => tool_error_result(&msg),
                    }
                }

                "round_state" => {
                    let arguments = request
                        .params
                        .as_ref()
                        .and_then(|p| p.get("arguments"))
                        .cloned()
                        .unwrap_or(Value::Null);

                    match validate_no_arg_tool("round_state", &arguments) {
                        Ok(()) => build_round_state_result(state),
                        Err(msg) => tool_error_result(&msg),
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
                pending_speak,
                pending_join,
                pending_leave,
            }
        }

        _ => empty_outcome(McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(McpError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
            }),
        }),
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
        Ok(protocol::ServerMessage::Welcome { player_id, .. }) => {
            state.player_id = player_id;
            state.connected = true;
        }
        Ok(protocol::ServerMessage::Error { .. }) => {
            // Unicast speak rejection; MCP speak path already mirrors cooldown as isError.
        }
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
            last_speak_tick: None,
            ..Default::default()
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

    #[test]
    fn look_at_player_id_ok() {
        let id = Uuid::new_v4();
        let args = serde_json::json!({"look_at": {"player_id": id.to_string()}, "fire": true});
        let action = validate_act_arguments(&args).expect("valid look_at");
        assert!(action.fire);
        let look = action.look_at.expect("look_at");
        assert_eq!(look.player_id, Some(id));
    }

    #[test]
    fn look_at_xz_ok() {
        let args = serde_json::json!({"look_at": {"x": 1.5, "z": -2.0}});
        let action = validate_act_arguments(&args).expect("valid look_at xz");
        let look = action.look_at.expect("look_at");
        assert_eq!(look.x, Some(1.5));
        assert_eq!(look.z, Some(-2.0));
    }

    #[test]
    fn look_at_unknown_field_errors() {
        let args = serde_json::json!({"look_at": {"x": 1.0, "z": 2.0, "laser": true}});
        let err = validate_act_arguments(&args).unwrap_err();
        assert!(err.contains("look_at"), "{err}");
        assert!(err.contains("laser") || err.contains("unknown"), "{err}");
    }

    #[test]
    fn look_at_incomplete_xz_errors() {
        let args = serde_json::json!({"look_at": {"x": 1.0}});
        let err = validate_act_arguments(&args).unwrap_err();
        assert!(err.contains("look_at"), "{err}");
    }

    #[test]
    fn speak_valid_sets_pending_speak() {
        let mut state = ToolState {
            player_id: Some(Uuid::nil()),
            last_snapshot: Some(serde_json::json!({"tick": 10})),
            ..Default::default()
        };
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({
                    "name": "speak",
                    "arguments": {"text": "  nice scrap  "}
                })),
            ),
            &mut state,
        );
        let speak = out.pending_speak.expect("pending speak");
        assert_eq!(speak.text, "nice scrap");
        assert!(out.pending_action.is_none());
        let result = out.response.result.unwrap();
        assert!(result.get("isError").is_none() || result["isError"] == false);
        assert!(result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Speak sent successfully"));
        assert_eq!(state.last_speak_tick, Some(10));
    }

    #[test]
    fn speak_rate_limited_sets_is_error() {
        let mut state = ToolState {
            player_id: Some(Uuid::nil()),
            last_snapshot: Some(serde_json::json!({"tick": 100})),
            last_speak_tick: Some(90),
            ..Default::default()
        };
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({
                    "name": "speak",
                    "arguments": {"text": "again"}
                })),
            ),
            &mut state,
        );
        assert!(
            out.pending_speak.is_none(),
            "rate-limited must not queue speak"
        );
        let result = out.response.result.unwrap();
        assert_eq!(result["isError"], true);
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("rate limited"), "{text}");
        assert!(!text.contains("Speak sent successfully"), "{text}");
    }

    #[test]
    fn speak_after_cooldown_ok() {
        let mut state = ToolState {
            player_id: Some(Uuid::nil()),
            last_snapshot: Some(serde_json::json!({"tick": 200})),
            last_speak_tick: Some(100),
            ..Default::default()
        };
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({
                    "name": "speak",
                    "arguments": {"text": "back"}
                })),
            ),
            &mut state,
        );
        assert!(out.pending_speak.is_some());
        let result = out.response.result.unwrap();
        assert!(result.get("isError").is_none() || result["isError"] == false);
    }

    #[test]
    fn speak_overlong_sets_is_error() {
        let mut state = ToolState {
            player_id: Some(Uuid::nil()),
            last_snapshot: Some(serde_json::json!({"tick": 1})),
            ..Default::default()
        };
        let long = "x".repeat(SPEAK_MAX_CHARS + 1);
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({
                    "name": "speak",
                    "arguments": {"text": long}
                })),
            ),
            &mut state,
        );
        assert!(out.pending_speak.is_none());
        let result = out.response.result.unwrap();
        assert_eq!(result["isError"], true);
        assert!(state.last_speak_tick.is_none());
    }

    #[test]
    fn speak_spectator_sets_is_error() {
        let mut state = ToolState {
            player_id: None,
            last_snapshot: Some(serde_json::json!({"tick": 1})),
            ..Default::default()
        };
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({
                    "name": "speak",
                    "arguments": {"text": "hi"}
                })),
            ),
            &mut state,
        );
        assert!(out.pending_speak.is_none());
        let result = out.response.result.unwrap();
        assert_eq!(result["isError"], true);
        assert!(
            result["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("spectator")
                || result["content"][0]["text"]
                    .as_str()
                    .unwrap()
                    .contains("not connected")
        );
    }

    #[test]
    fn speak_schema_errors() {
        let mut state = ToolState::default();
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({
                    "name": "speak",
                    "arguments": {"text": "hi", "laser": true}
                })),
            ),
            &mut state,
        );
        assert!(out.pending_speak.is_none());
        let result = out.response.result.unwrap();
        assert_eq!(result["isError"], true);
        assert!(result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("schema error"));

        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({
                    "name": "speak",
                    "arguments": {"text": ""}
                })),
            ),
            &mut state,
        );
        assert!(out.pending_speak.is_none());

        let long = "x".repeat(SPEAK_MAX_CHARS + 1);
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({
                    "name": "speak",
                    "arguments": {"text": long}
                })),
            ),
            &mut state,
        );
        assert!(out.pending_speak.is_none());
    }

    #[test]
    fn tools_list_includes_speak() {
        let mut state = ToolState::default();
        let list = handle_mcp_request(req("tools/list", None), &mut state);
        let result = list.response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        let names: Vec<_> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"speak"), "names={:?}", names);
    }

    #[test]
    fn tools_list_includes_join_leave_round_state() {
        let mut state = ToolState::default();
        let list = handle_mcp_request(req("tools/list", None), &mut state);
        let result = list.response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        let names: Vec<_> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"join"), "names={:?}", names);
        assert!(names.contains(&"leave"), "names={:?}", names);
        assert!(names.contains(&"round_state"), "names={:?}", names);
    }

    #[test]
    fn join_idempotent_when_connected() {
        let mut state = ToolState {
            connected: true,
            player_id: Some(Uuid::nil()),
            session_name: Some("ArenaFox".into()),
            default_name: "ArenaFox".into(),
            ..Default::default()
        };
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({"name":"join","arguments":{}})),
            ),
            &mut state,
        );
        assert!(out.pending_join.is_none());
        assert!(!out.pending_leave);
        let result = out.response.result.unwrap();
        assert!(result.get("isError").is_none() || result["isError"] == false);
        assert!(result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Already joined"));
    }

    #[test]
    fn join_pending_when_not_connected() {
        let mut state = ToolState {
            default_name: "MCP Agent".into(),
            ..Default::default()
        };
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({
                    "name": "join",
                    "arguments": {"name": "  ScrapFox  "}
                })),
            ),
            &mut state,
        );
        assert_eq!(out.pending_join.as_deref(), Some("ScrapFox"));
        let result = out.response.result.unwrap();
        assert!(result.get("isError").is_none() || result["isError"] == false);
        assert!(result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Joining as 'ScrapFox'"));
    }

    #[test]
    fn join_reuses_default_name() {
        let mut state = ToolState {
            default_name: "FromFlag".into(),
            ..Default::default()
        };
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({"name":"join","arguments":{}})),
            ),
            &mut state,
        );
        assert_eq!(out.pending_join.as_deref(), Some("FromFlag"));
    }

    #[test]
    fn join_schema_unknown_field_errors() {
        let mut state = ToolState::default();
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({
                    "name": "join",
                    "arguments": {"name": "A", "laser": true}
                })),
            ),
            &mut state,
        );
        assert!(out.pending_join.is_none());
        let result = out.response.result.unwrap();
        assert_eq!(result["isError"], true);
        assert!(result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("schema error"));
    }

    #[test]
    fn leave_when_connected_sets_pending_leave() {
        let mut state = ToolState {
            connected: true,
            player_id: Some(Uuid::nil()),
            session_name: Some("ArenaFox".into()),
            last_snapshot: Some(serde_json::json!({"tick": 1})),
            ..Default::default()
        };
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({"name":"leave","arguments":{}})),
            ),
            &mut state,
        );
        assert!(out.pending_leave);
        assert!(!state.connected);
        assert!(state.player_id.is_none());
        assert!(state.last_snapshot.is_none());
        let result = out.response.result.unwrap();
        assert!(result.get("isError").is_none() || result["isError"] == false);
    }

    #[test]
    fn leave_when_not_connected_is_error() {
        let mut state = ToolState::default();
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({"name":"leave","arguments":{}})),
            ),
            &mut state,
        );
        assert!(!out.pending_leave);
        let result = out.response.result.unwrap();
        assert_eq!(result["isError"], true);
        assert!(result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("not connected"));
    }

    #[test]
    fn leave_schema_unknown_field_errors() {
        let mut state = ToolState {
            connected: true,
            player_id: Some(Uuid::nil()),
            ..Default::default()
        };
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({
                    "name": "leave",
                    "arguments": {"force": true}
                })),
            ),
            &mut state,
        );
        assert!(!out.pending_leave);
        assert!(state.connected);
        let result = out.response.result.unwrap();
        assert_eq!(result["isError"], true);
        assert!(result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("schema error"));
    }

    #[test]
    fn round_state_from_snapshot_and_events() {
        let mut state = ToolState {
            connected: true,
            player_id: Some(Uuid::nil()),
            last_snapshot: Some(serde_json::json!({
                "tick": 42,
                "round_state": "Active",
                "round_time_left": 90,
                "frag_limit": 10,
                "mode_name": "Contested Frequency",
                "playlist": "Arena Duel",
                "host_line": "HOST: LIVE.",
                "pressure": "compliance"
            })),
            recent_events: vec![
                serde_json::json!({
                    "event": "round_start",
                    "round_number": 3,
                    "frag_limit": 10,
                    "mode_name": "Contested Frequency",
                    "host_line": "HOST: START."
                }),
                serde_json::json!({
                    "event": "frag",
                    "killer": "A",
                    "victim": "B"
                }),
            ],
            ..Default::default()
        };
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({"name":"round_state","arguments":{}})),
            ),
            &mut state,
        );
        let result = out.response.result.unwrap();
        assert_eq!(result["round_state"], "Active");
        assert_eq!(result["round_number"], 3);
        assert_eq!(result["round_time_left"], 90);
        assert_eq!(result["frag_limit"], 10);
        assert_eq!(result["mode_name"], "Contested Frequency");
        assert_eq!(result["host_line"], "HOST: LIVE.");
        assert_eq!(result["pressure"], "compliance");
        assert_eq!(result["connected"], true);
        assert_eq!(result["last_round_start"]["round_number"], 3);
    }

    #[test]
    fn round_state_schema_unknown_field_errors() {
        let mut state = ToolState::default();
        let out = handle_mcp_request(
            req(
                "tools/call",
                Some(serde_json::json!({
                    "name": "round_state",
                    "arguments": {"extra": 1}
                })),
            ),
            &mut state,
        );
        let result = out.response.result.unwrap();
        assert_eq!(result["isError"], true);
        assert!(result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("schema error"));
    }

    #[test]
    fn ingest_welcome_sets_connected() {
        let mut state = ToolState::default();
        assert!(!state.connected);
        ingest_server_text(
            &mut state,
            r#"{"type":"welcome","player_id":"00000000-0000-0000-0000-000000000000","role":"agent","mode_name":"Contested Frequency","playlist":"Arena Duel"}"#,
        );
        assert!(state.connected);
        assert!(state.player_id.is_some());
    }
}
