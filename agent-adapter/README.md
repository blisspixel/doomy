# Agent Adapter

MCP-compatible control plane for external agents to observe and act in the fragr arena. Runs separate from the hot-path combat tick.

## What this is

The agent-adapter bridges external AI agents (LLMs, scripted bots, MCP clients) to the authoritative game server. It provides:

1. **MCP Server mode**: JSON-RPC 2.0 over stdio, compatible with MCP clients
2. **Scripted bot mode**: Standalone bot client for testing without MCP

Agents and humans share the same discrete action channel into the server. The adapter operates at slow control-plane rates (observe/act at ~1-10 Hz), not the combat tick (20 Hz).

## Quick Start

### MCP Server (for external agents)

```bash
cd agent-adapter
cargo run -- mcp --server ws://127.0.0.1:6767 --name ArenaFox
# or: FRAGR_AGENT_NAME=ArenaFox cargo run -- mcp
```

Connect via MCP client (stdio) and use the `observe` and `act` tools.
Hello joins with `--name` / `FRAGR_AGENT_NAME` (default `MCP Agent`). There is no separate `session_join` tool; join is the WebSocket Hello on adapter start.

### Scripted Bot (standalone test)

```bash
cd agent-adapter
cargo run -- scripted-bot --server ws://127.0.0.1:6767 --name MyBot
```

Connects as an agent role, observes snapshots, computes simple chase-and-shoot actions at ~20 Hz.

## MCP Tools

### `observe`

Get the current game state snapshot including self player ID and recent events.

**Input schema:**
```json
{}
```

**Output (after first snapshot):**
```json
{
  "tick": 12345,
  "players": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Bot1",
      "x": 10.5,
      "y": 1.5,
      "z": -5.2,
      "yaw": 1.57,
      "hp": 75,
      "just_fired": false,
      "behavior": "Aggressive",
      "score": 3,
      "weapon": "Flechette"
    }
  ],
  "round_state": "Active",
  "round_time_left": 120,
  "frag_limit": 10,
  "self_player_id": "550e8400-e29b-41d4-a716-446655440000",
  "recent_events": [
    {"event": "player_joined", "player": "Bot1", "role": "Agent"},
    {"event": "round_start", "round_number": 1, "frag_limit": 10, "time_limit": 180},
    {"event": "frag", "killer": "Bot1", "victim": "Bot2"},
    {"event": "respawn", "player": "Bot2"},
    {"event": "round_end", "winner": "Bot1", "reason": "Frag limit reached"},
    {"event": "player_left", "player": "Bot2"}
  ]
}
```

**Output (connecting state, before first snapshot):**
```json
{
  "status": "connecting",
  "message": "Waiting for first snapshot from server",
  "self_player_id": "550e8400-e29b-41d4-a716-446655440000",
  "recent_events": []
}
```

**Notes:**
- Returns connecting state until first snapshot arrives from server
- `self_player_id`: UUID of your agent's player (null for spectators)
- `recent_events`: Last 50 game events (player joins/leaves, frags, respawns, round start/end) in chronological order
- Dead players (HP <= 0) are omitted from players array
- `behavior` field is present only for server-side bots
- `weapon` field shows current weapon: "Flechette" (balanced), "Rail" (precision), or "Scatter" (close-range)
- Call rate: 1-10 Hz is typical; faster is allowed but returns cached data between server ticks

### `act`

Send an action to control your agent's pawn.

**Input schema:**
```json
{
  "forward": false,
  "back": false,
  "left": false,
  "right": false,
  "turn_left": false,
  "turn_right": false,
  "fire": false,
  "weapon_swap": null
}
```

All fields are optional. Movement and fire are booleans (default `false`). `weapon_swap` is an optional string: `"flechette"`, `"rail"`, or `"scatter"`.

**Output:**
```json
{
  "content": [{
    "type": "text",
    "text": "Action sent successfully"
  }]
}
```

**Notes:**
- Actions are **level-held (sticky)** within each server tick window, not edge-triggered
- Each `act` call overwrites the previous pending action state
- All `true` fields are applied together on the next server tick
- Movement keys combine (e.g., `forward + left` = diagonal)
- `weapon_swap` is processed immediately on the next tick
- Server enforces weapon-specific cooldowns (Flechette: 500ms, Rail: 2.0s, Scatter: 250ms)
- Call rate: 1-10 Hz typical for MCP agents; faster allowed but limited by server tick rate

### `get_events`

Get recent game events (player joins/leaves, frags, respawns, round start/end) explicitly.

**Input schema:**
```json
{
  "clear": false
}
```

All fields are optional. `clear` (boolean, default false): clear event buffer after retrieving.

**Output:**
```json
{
  "content": [{
    "type": "text",
    "text": "Recent events: [{\"event\":\"player_joined\",\"player\":\"Bot1\",\"role\":\"Agent\"},{\"event\":\"frag\",\"killer\":\"Bot1\",\"victim\":\"Bot2\"},{\"event\":\"respawn\",\"player\":\"Bot2\"}]"
  }]
}
```

**Notes:**
- Returns last 50 events in chronological order
- Event types: `player_joined`, `player_left`, `frag` (kill), `respawn`, `round_start`, `round_end`
- Events are also included in `observe` output under `recent_events`
- Set `clear: true` to acknowledge events and reset buffer

## Architecture

```
MCP Client (LLM/script)
    |
    | stdio (JSON-RPC 2.0)
    v
agent-adapter (this crate)
    |
    | WebSocket JSON
    v
game server (fragr-server)
```

**Roles:**
- `spectator`: Read-only, no player ID, cannot send actions
- `human`: Keyboard/mouse player, sends actions at input rate
- `agent`: Bot/MCP agent, sends actions via adapter or direct WS

## Protocol

The adapter speaks the same WebSocket JSON protocol as the Godot client and human players. See `docs/protocol.md` for full message schemas.

**Connection flow:**
1. Adapter connects to game server WebSocket
2. Sends `Hello` with `role=agent` and name
3. Receives `Welcome` with assigned `player_id`
4. Begins receiving `Snapshot` messages at ~20 Hz (cached for `observe`)
5. MCP client calls `act` tool, adapter sends `Action` message to server

## Implementation Notes

- MCP server logs to stderr to avoid polluting stdout (JSON-RPC channel)
- Last snapshot is cached in a Tokio mutex; `observe` returns the cached value
- `act` tool forwards actions directly to the game server WebSocket
- Scripted bot runs a simple chase-and-shoot AI loop for smoke testing

## Hardening (completed)

- [x] Session lifecycle: `Hello` / `Welcome` / clean disconnect
- [x] Observe returns full snapshot with round state, scores, time remaining
- [x] Act validates action schema and forwards to server
- [x] Scripted bot mode for end-to-end testing without MCP client
- [x] Same input pipeline as humans (shared `Action` message type)
- [x] MCP tools documented with schemas and examples

## Future (post-Slice 1)

- Session summaries: periodic `observe` with historical stats (kills, deaths, accuracy)
- Goal setting: agent declares intent (e.g., "flank target", "defend area")
- Low-Hz summaries for LLM context (not full snapshot spam)
- Clawbot integration: screenshot observations via separate tooling (spend-gated)

## Testing

```bash
# Run adapter tests
cargo test

# Manual smoke test (server must be running)
cargo run -- scripted-bot --name TestBot

# MCP client test (requires MCP-compatible client like Claude Desktop)
cargo run -- mcp < test_input.jsonl
```

Example `test_input.jsonl`:
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"observe","arguments":{}}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"act","arguments":{"forward":true,"fire":true}}}
```
