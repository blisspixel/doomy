# Doomy Network Protocol

WebSocket JSON protocol between clients and the authoritative server.

**Transport:** WebSocket on `ws://127.0.0.1:7777` (configurable)
**Format:** JSON text messages
**Tick rate:** ~20 Hz (50ms per tick)

## Connection Flow

1. Client connects to WebSocket
2. Client sends `Hello` message with role and name
3. Server responds with `Welcome` message
4. Server begins broadcasting `Snapshot` messages at tick rate
5. Clients (human/agent roles) can send `Action` messages
6. Server may send `Event` messages for notable occurrences

## Message Types

### Client → Server

#### Hello

Initial handshake message. Must be sent immediately after connection.

```json
{
  "type": "hello",
  "role": "spectator" | "human" | "agent",
  "name": "PlayerName"
}
```

**Fields:**
- `role`: Connection role
  - `spectator`: Read-only, receives snapshots, cannot control a player
  - `human`: Keyboard/mouse player, receives snapshots, sends actions
  - `agent`: Bot/MCP agent, receives snapshots, sends actions
- `name`: Display name (shown in game and logs)

#### Action

Sent by `human` or `agent` roles to control their player. All fields are optional booleans defaulting to `false`.

```json
{
  "type": "action",
  "forward": true,
  "back": false,
  "left": false,
  "right": false,
  "turn_left": false,
  "turn_right": false,
  "fire": false,
  "weapon_swap": "rail"
}
```

**Fields:**
- `forward` / `back`: Move forward/backward
- `left` / `right`: Strafe left/right
- `turn_left` / `turn_right`: Rotate view left/right
- `fire`: Fire weapon
- `weapon_swap` (optional): Change to specified weapon (`"flechette"`, `"rail"`, or `"scatter"`)

**Notes:**
- Actions are discrete intents applied on the next server tick
- Movement keys combine (e.g., forward + left = diagonal)
- Server enforces rate limits and cooldowns per weapon type
- Spectators that send actions are ignored
- Weapon swap takes effect immediately on the next tick

### Server → Client

#### Welcome

Server response to `Hello`. Confirms connection and provides player ID.

```json
{
  "type": "welcome",
  "player_id": "550e8400-e29b-41d4-a716-446655440000" | null,
  "role": "spectator" | "human" | "agent"
}
```

**Fields:**
- `player_id`: UUID of the player entity (null for spectators)
- `role`: Echoed role from Hello

#### Snapshot

Periodic state broadcast containing all visible game entities. Sent at ~20 Hz.

```json
{
  "type": "snapshot",
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
      "weapon": "flechette"
    }
  ]
}
```

**Fields:**
- `tick`: Server tick counter
- `players`: Array of visible player states
  - `id`: Player UUID
  - `name`: Display name
  - `x`, `y`, `z`: Position in world space (arena is ±25 units)
  - `yaw`: Rotation in radians (0 = +X axis, counter-clockwise)
  - `hp`: Health points (0-100)
  - `just_fired`: True on the tick a weapon was fired (for muzzle flash)
  - `weapon`: Current weapon (`"flechette"`, `"rail"`, or `"scatter"`)

**Notes:**
- Dead players (HP ≤ 0) are omitted from the snapshot
- Clients must handle players appearing/disappearing
- No delta compression in v1 (future optimization)

#### Event

Notable game occurrences sent immediately (not tied to snapshot cadence).

**Frag Event:**
```json
{
  "type": "event",
  "event": "frag",
  "killer": "Bot1",
  "victim": "Bot2"
}
```

**Respawn Event:**
```json
{
  "type": "event",
  "event": "respawn",
  "player": "Bot2"
}
```

**Fields:**
- `event`: Event type (`frag` or `respawn`)
- `killer` / `victim`: Player names involved in frag
- `player`: Player name for respawn

## Implementation Notes

### Arena Bounds
- Size: 50×50 units (±25 from origin)
- Floor at Y=0
- Players spawn at Y=1.5
- Walls prevent movement outside bounds

### Combat

Three distinct weapon roles with different TTK profiles:

**Flechette** (default, all-rounder):
- **Damage**: 15 HP per hit
- **Fire rate**: 6 tick cooldown (~300ms, 3.3 shots/sec)
- **Range**: 100 units
- **TTK**: 7 hits to kill (2.1s optimal)

**Rail** (precision hard hitter):
- **Damage**: 50 HP per hit
- **Fire rate**: 25 tick cooldown (~1250ms, 0.8 shots/sec)
- **Range**: 120 units
- **TTK**: 2 hits to kill (1.25s optimal)

**Scatter** (close range):
- **Damage**: 8 HP per pellet × 5 pellets = 40 HP max at point-blank
- **Fire rate**: 15 tick cooldown (~750ms, 1.3 shots/sec)
- **Range**: 30 units
- **Spread**: 0.3 radian cone (5 raycasts)
- **TTK**: 3 hits point-blank (1.5s optimal), 5+ hits at range

**General**:
- All weapons use instant hitscan (no projectile travel)
- **Respawn delay**: 60 ticks (3 seconds)

### Movement
- **Speed**: 5 units/second
- **Turn speed**: 2 radians/second
- **Collision**: Simple AABB with 0.5 unit radius

## Example Session

```
// Client connects and identifies
C→S: {"type": "hello", "role": "agent", "name": "MyBot"}

// Server welcomes
S→C: {"type": "welcome", "player_id": "...", "role": "agent"}

// Server sends periodic snapshots
S→C: {"type": "snapshot", "tick": 1, "players": [...]}
S→C: {"type": "snapshot", "tick": 2, "players": [...]}

// Client sends actions
C→S: {"type": "action", "forward": true, "fire": false}
C→S: {"type": "action", "turn_right": true, "fire": true}

// Server announces frag
S→C: {"type": "event", "event": "frag", "killer": "MyBot", "victim": "Bot1"}

// More snapshots
S→C: {"type": "snapshot", "tick": 45, "players": [...]}
```

## Future Considerations (Post-Slice 1)

- **Binary protocol**: Replace JSON with efficient binary (bincode, flatbuffers)
- **Delta compression**: Send only changed fields
- **Interest management**: Filter snapshots by visibility/distance
- **UDP option**: Low-latency channels for actions (alongside WS for reliability)
- **Prediction**: Client-side movement prediction for smoother human play
