# Agent Adapter

MCP-compatible control plane for external agents to observe and act in the fragr arena. Runs separate from the hot-path combat tick.

## What this is

Third agent tier (decision brain, not MCP): see [`agents/brain/README.md`](../agents/brain/README.md). MCP remains the BYO chat-agent door.

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

Connect via MCP client (stdio) and use the tools below.
Boot path still sends Hello with `--name` / `FRAGR_AGENT_NAME` (default `MCP Agent`). First-class `join` / `leave` / `round_state` tools are also available (idempotent join; leave disconnects cleanly).

### Scripted Bot (standalone test)

```bash
cd agent-adapter
cargo run -- scripted-bot --server ws://127.0.0.1:6767 --name MyBot
```

Connects as an agent role, observes snapshots, aims with `look_at.player_id`, shoots at ~20 Hz, and occasionally speaks a Contested Frequency taunt.

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
- Snapshot may include `shot_results` (per-tick hit-confirm: `hit`, `damage`, `target_hp_after`)
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
  "weapon_swap": null,
  "look_at": { "player_id": "550e8400-e29b-41d4-a716-446655440000" }
}
```

All fields are optional. Movement and fire are booleans (default `false`). `weapon_swap` is an optional string: `"flechette"`, `"rail"`, or `"scatter"`. `look_at` is an optional object: prefer `player_id` (UUID), or both `x` and `z` for a world point. The server snaps yaw toward the target.

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
- `look_at` is applied by the server (yaw snap) after movement/turn on the next tick
- Server enforces weapon-specific cooldowns (Flechette: 500ms, Rail: 2.0s, Scatter: 250ms)
- Call rate: 1-10 Hz typical for MCP agents; faster allowed but limited by server tick rate

### `speak`

Send a short off-tick taunt/callout (not sticky Action). Rate-limited on the server (~3s). Spectators see it; it appears in `recent_events` / `get_events`.

**Input schema:**
```json
{
  "text": "nice scrap"
}
```

**Rules:**
- `text` required string; trimmed; max 80 chars; no control characters
- Unknown fields -> schema error (`isError: true`)
- Empty / overlong -> schema error (`isError: true`)
- Rate-limited speak (mirrored ~3s / 60 ticks) -> `isError: true` (never a success toast on a no-op)
- Spectators / no player_id -> `isError: true`

### `get_events`

Get recent game events (player joins/leaves, frags, hits, respawns, round start/end) explicitly.

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

### `join`

Ensure the agent is in the arena (Hello / Welcome). Optional `name`; when omitted, reuses `--name` / `FRAGR_AGENT_NAME`.

**Input schema:**
```json
{
  "name": "ArenaFox"
}
```

All fields optional. Unknown fields -> schema error (`isError: true`). Empty name -> schema error.

**Behavior:**
- Already joined -> success, idempotent (`Already joined as '...'`)
- Not connected -> adapter reconnects, sends Hello, awaits Welcome
- Boot Hello-on-start remains valid; `join` is first-class for re-join after `leave`

### `leave`

Clean WebSocket disconnect from the arena.

**Input schema:**
```json
{}
```

No fields. Unknown fields -> schema error (`isError: true`).

**Behavior:**
- Connected -> success; clears local session; closes WebSocket
- Not connected -> `isError: true` (`leave rejected: not connected`)

### `round_state`

Current round summary without scraping full `observe`.

**Input schema:**
```json
{}
```

No fields. Unknown fields -> schema error (`isError: true`).

**Output (example):**
```json
{
  "connected": true,
  "self_player_id": "550e8400-e29b-41d4-a716-446655440000",
  "session_name": "ArenaFox",
  "round_state": "Active",
  "round_number": 3,
  "round_time_left": 90,
  "frag_limit": 10,
  "mode_name": "Contested Frequency",
  "playlist": "Arena Duel",
  "host_line": "HOST: CONTESTED FREQUENCY. LEAGUE DENIES EXISTENCE. ARENA DUEL IS LIVE.",
  "pressure": null,
  "mvp": null,
  "mvp_frags": null,
  "map_id": 1,
  "map_name": "Arena Duel",
  "last_round_start": {"event": "round_start", "round_number": 3},
  "last_round_end": null
}
```

Fields come from the last snapshot plus the most recent `round_start` / `round_end` in the events buffer. While Ended, Snapshot `mvp` / `mvp_frags` / sticky `host_line` rehydrate mid-join even if `round_end` was missed. `map_id` / `map_name` always present (defaults to Arena Duel when the snapshot omitted them).

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
1. Adapter connects to game server WebSocket (boot Hello with `--name`, or later via `join`)
2. Sends `Hello` with `role=agent` and name
3. Receives `Welcome` with assigned `player_id`
4. Begins receiving `Snapshot` messages at ~20 Hz (cached for `observe` / `round_state`)
5. MCP client calls `act` / `speak`; adapter forwards on the open socket
6. `leave` closes the socket cleanly; `join` reconnects and Hellos again

## Implementation Notes

- MCP server logs to stderr to avoid polluting stdout (JSON-RPC channel)
- Last snapshot is cached in a Tokio mutex; `observe` returns the cached value
- `act` tool forwards actions directly to the game server WebSocket
- Scripted bot runs a simple chase-and-shoot AI loop for smoke testing

## Hardening (completed)

- [x] Session lifecycle: boot Hello / Welcome; first-class `join` / `leave` tools; clean disconnect
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
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"round_state","arguments":{}}}
{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"leave","arguments":{}}}
{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"join","arguments":{"name":"ArenaFox"}}}
```
