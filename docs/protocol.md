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
- `weapon_swap`: (optional) Switch to weapon type: `"flechette"` | `"rail"` | `"scatter"`

**Notes:**
- Actions are **level-held (sticky)** within each server tick window, not edge-triggered
- Each Action message overwrites the previous pending action state
- All `true` fields are applied together on the next server tick
- Movement keys combine (e.g., forward + left = diagonal)
- Weapon swap is processed immediately on the next tick
- Server enforces rate limits and cooldowns (weapon-specific)
- Spectators that send actions are ignored

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
      "behavior": "Aggressive",
      "score": 3,
      "weapon": "Flechette"
    }
  ],
  "round_state": "Active",
  "round_time_left": 120,
  "frag_limit": 10
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
  - `behavior`: (optional) Bot behavior type if server-side bot
  - `score`: Kills in current round
  - `weapon`: Current weapon name ("Flechette", "Rail", or "Scatter")
- `round_state`: (optional) Current round state ("Warmup", "Active", "Ended")
- `round_time_left`: (optional) Seconds remaining in active round
- `frag_limit`: (optional) Frag limit for current round

**Notes:**
- Dead players (HP ≤ 0) are omitted from the snapshot
- Clients must handle players appearing/disappearing
- No delta compression in v1 (future optimization)
- Round fields present when round system is active

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

**Round Start Event:**
```json
{
  "type": "event",
  "event": "round_start",
  "round_number": 1,
  "frag_limit": 10,
  "time_limit": 180
}
```

**Round End Event:**
```json
{
  "type": "event",
  "event": "round_end",
  "winner": "Bot1",
  "reason": "Frag limit reached"
}
```

**Player Joined Event:**
```json
{
  "type": "event",
  "event": "player_joined",
  "player": "NewPlayer",
  "role": "agent"
}
```

**Player Left Event:**
```json
{
  "type": "event",
  "event": "player_left",
  "player": "OldPlayer"
}
```

**Fields:**
- `event`: Event type (`frag`, `respawn`, `round_start`, `round_end`, `player_joined`, `player_left`)
- `killer` / `victim`: Player names involved in frag
- `player`: Player name for respawn, join, or leave
- `role`: Role of joining player ("spectator", "human", "agent")
- `round_number`: Round counter (starts at 1)
- `frag_limit`: (optional) Frag limit for the round (null if time-only)
- `time_limit`: (optional) Time limit in seconds (null if frag-only)
- `winner`: (optional) Winner name if any (null for draw/time)
- `reason`: Round end reason ("Frag limit reached", "Time limit reached", etc.)

**Notes:**
- Events are sent asynchronously as they occur (off the snapshot tick)
- MCP clients receive events in two ways:
  - Buffered in the `recent_events` field of the `observe` tool response (last 50 events)
  - Via the dedicated `get_events` tool for explicit retrieval
- Events capture match drama (frags, respawns, rounds) without forcing agents onto the combat tick

## Implementation Notes

### Arena Bounds
- Size: 50×50 units (±25 from origin)
- Floor at Y=0
- Players spawn at Y=1.5
- Walls prevent movement outside bounds

### Combat
- **Weapon types**: Three distinct roles
  - **Flechette** (default): Balanced all-purpose
    - Damage: 25 HP
    - Cooldown: 10 ticks (500ms)
    - Spread: 0.1 radians (tight, ~5.7 degrees)
  - **Rail**: High-skill precision weapon
    - Damage: 75 HP
    - Cooldown: 40 ticks (2.0s)
    - Spread: 0.05 radians (very tight, ~2.9 degrees)
  - **Scatter**: Close-range spam weapon
    - Damage: 15 HP
    - Cooldown: 5 ticks (250ms)
    - Spread: 0.3 radians (wide, ~17.2 degrees)
- **Hitscan**: Instant hit detection, no projectile travel
- **Range**: 100 units
- **Respawn delay**: 60 ticks (3 seconds)

### Movement
- **Speed**: 5 units/second
- **Turn speed**: 2 radians/second
- **Collision**: Simple AABB with 0.5 unit radius

### Round System
- **Warmup**: 3 seconds (60 ticks)
- **Frag limit**: Default 10 kills
- **Time limit**: Default 180 seconds (3600 ticks)
- **End delay**: 5 seconds between rounds
- **Scoring**: Per-round kills, reset each round
- **Persistence**: Bots remain active when humans leave

## Example Session

```
// Client connects and identifies
C→S: {"type": "hello", "role": "agent", "name": "MyBot"}

// Server welcomes
S→C: {"type": "welcome", "player_id": "...", "role": "agent"}

// Round starts after warmup
S→C: {"type": "event", "event": "round_start", "round_number": 1, "frag_limit": 10, "time_limit": 180}

// Server sends periodic snapshots
S→C: {"type": "snapshot", "tick": 1, "players": [...], "round_state": "Active", "round_time_left": 180, "frag_limit": 10}
S→C: {"type": "snapshot", "tick": 2, "players": [...], "round_state": "Active", "round_time_left": 180, "frag_limit": 10}

// Client sends actions
C→S: {"type": "action", "forward": true, "fire": false}
C→S: {"type": "action", "turn_right": true, "fire": true}

// Server announces frag
S→C: {"type": "event", "event": "frag", "killer": "MyBot", "victim": "Bot1"}

// More snapshots (with updated scores)
S→C: {"type": "snapshot", "tick": 45, "players": [{"id": "...", "name": "MyBot", "score": 1, ...}], ...}

// Round ends when limit reached
S→C: {"type": "event", "event": "round_end", "winner": "MyBot", "reason": "Frag limit reached"}

// Next round starts after delay
S→C: {"type": "event", "event": "round_start", "round_number": 2, "frag_limit": 10, "time_limit": 180}
```

## Future Considerations (Post-Slice 1)

- **Binary protocol**: Replace JSON with efficient binary (bincode, flatbuffers)
- **Delta compression**: Send only changed fields
- **Interest management**: Filter snapshots by visibility/distance
- **UDP option**: Low-latency channels for actions (alongside WS for reliability)
- **Prediction**: Client-side movement prediction for smoother human play
