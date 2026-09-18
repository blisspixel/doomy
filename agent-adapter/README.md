# Doomy Agent Adapter

Agent adapter for Doomy - provides both **MCP slow control plane** and fast scripted bot modes.

## Design Philosophy

The agent adapter is a **slow strategic control plane**, not a 20-60 Hz combat interface. Suitable for:
- LLM agents with 1-10 second think times
- High-level decision making and goal setting
- Deliberative AI that observes and acts at human-like cadences

Fast reactive bots run server-side (or via scripted-bot mode) and own the tick loop.

## Modes

### 1. MCP Server Mode

Exposes MCP stdio tools for external agents and clawbots to control a player.

```bash
cargo run -- mcp --server ws://127.0.0.1:7777
```

#### Available Tools

- **observe**: Get current game state observation including recent events
  - Returns: JSON with:
    - `status`: "connecting" when waiting for first snapshot, omitted otherwise
    - `message`: Explanation when status is "connecting"
    - `tick`: Current game tick number (once connected)
    - `players`: Array of player states (id, name, x, y, z, yaw, hp, just_fired)
    - `self_player_id`: UUID of this agent's player (null for spectators)
    - `recent_events`: Array of recent game events (frags, respawns) - last 50 events

- **act**: Send action to game server
  - Parameters (all optional booleans, default false):
    - `forward`, `back`, `left`, `right`: Movement directions
    - `turn_left`, `turn_right`: Rotation directions
    - `fire`: Shoot weapon
  - **Behavior**: Actions are **level-held/sticky** within each server tick window. Set true to activate for the current tick, false to deactivate. The server applies all true actions on the next tick. Multiple act calls between ticks will overwrite previous values.

- **get_events**: Get recent game events (frags, respawns)
  - Parameters:
    - `clear` (optional boolean): Clear event buffer after retrieving (default: false)
  - Returns: Last 50 game events in chronological order

### 2. Scripted Bot Mode

Runs a simple chase-and-shoot AI bot.

```bash
cargo run -- scripted-bot --name "MyBot" --server ws://127.0.0.1:7777
```

Bot behavior:
- Finds nearest enemy
- Turns to face them
- Moves forward if far away
- Fires when aligned and in range

## Protocol

Uses the same WebSocket JSON protocol as the main server:

```json
// Client → Server
{"type": "hello", "role": "agent", "name": "BotName"}
{"type": "action", "forward": true, "fire": false, ...}

// Server → Client
{"type": "welcome", "player_id": "uuid", "role": "agent"}
{"type": "snapshot", "tick": 123, "players": [...]}
{"type": "event", "event": "frag", "killer": "Bot1", "victim": "Bot2"}
```

## MCP Integration

The MCP mode implements the MCP protocol over stdio:

```json
// Initialize
{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {...}}

// List tools
{"jsonrpc": "2.0", "id": 2, "method": "tools/list"}

// Call observe (before first snapshot)
{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "observe"}}
// Response: {"status": "connecting", "message": "Waiting for first snapshot from server", "self_player_id": "...", "recent_events": []}

// Call observe (after first snapshot)
{"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {"name": "observe"}}
// Response: {"tick": 123, "players": [...], "self_player_id": "...", "recent_events": [...]}

// Call act
{"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {
  "name": "act",
  "arguments": {"forward": true, "fire": true}
}}
```

## Building

```bash
cargo build --release
```

## Testing

Start the game server first:
```bash
cd ../server && cargo run -- --bots 0
```

Then run a bot:
```bash
cargo run -- scripted-bot
```

Or test MCP mode with a simple client:
```bash
cargo run -- mcp | head -100
```
