# Doomy Agent Adapter

Agent adapter for Doomy - provides both MCP server and scripted bot modes.

## Modes

### 1. MCP Server Mode

Exposes MCP stdio tools for external agents and clawbots to control a player.

```bash
cargo run -- mcp --server ws://127.0.0.1:7777
```

#### Available Tools

- **observe**: Get current game state observation
  - Returns: JSON snapshot with tick, players (id, name, x, y, z, yaw, hp, just_fired)

- **act**: Send action to game server
  - Parameters (all optional booleans):
    - `forward`, `back`, `left`, `right`: Movement
    - `turn_left`, `turn_right`: Rotation
    - `fire`: Shoot weapon

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

// Call observe
{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "observe"}}

// Call act
{"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {
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
