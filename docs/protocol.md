# fragr Network Protocol

WebSocket JSON protocol between clients and the authoritative server.

**Transport:** WebSocket on `ws://127.0.0.1:6767` (configurable)
**Format:** JSON text messages
**Tick rate:** ~20 Hz (50ms per tick)

## Connection Flow

1. Client connects to WebSocket
2. Client sends `Hello` message with role and name
3. Server responds with `Welcome` message
4. Server begins broadcasting `Snapshot` messages at tick rate
5. Clients (human/agent roles) can send `Action` messages
6. Server may send `Event` messages for notable occurrences

**MCP agent-adapter session tools:** boot Hello-on-start remains valid. First-class tools `join` (Hello/Welcome, optional name, idempotent), `leave` (clean WebSocket disconnect; `isError` if not connected), and `round_state` (round fields from last snapshot + recent `round_start` / `round_end`) are documented in `agent-adapter/README.md`. There is no separate on-wire Leave message; leave is disconnect.

**Reconnect:** re-Hello as human/agent with the same display name reclaims the prior client-mapped seat (ghost eviction + `PlayerLeft`, then fresh `Welcome` / `PlayerJoined`). Rule bots are not client-mapped and are never evicted by name. A new name after a clean leave is a new session. Late Disconnect for an already-evicted socket is a noop.

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
  "weapon_swap": "rail",
  "look_at": { "player_id": "550e8400-e29b-41d4-a716-446655440000" }
}
```

World-point aim:

```json
{
  "type": "action",
  "look_at": { "x": 10.0, "z": -5.0 },
  "fire": true
}
```

**Fields:**
- `forward` / `back`: Move forward/backward
- `left` / `right`: Strafe left/right
- `turn_left` / `turn_right`: Rotate view left/right (incremental)
- `fire`: Fire weapon
- `weapon_swap`: (optional) Switch to weapon type: `"flechette"` | `"rail"` | `"scatter"`
- `look_at`: (optional) Authoritative aim object. Prefer `player_id` (UUID string), or both `x` and `z` (world point). Server snaps yaw toward the target on the Action tick. Invalid/missing target is a yaw no-op.

**Notes:**
- Actions are **level-held (sticky)** within each server tick window, not edge-triggered
- Each Action message overwrites the previous pending action state
- All `true` fields are applied together on the next server tick
- Movement keys combine (e.g., forward + left = diagonal)
- `look_at` is applied after movement/turn so agents can strafe while locking aim
- Weapon swap is processed immediately on the next tick
- Server enforces rate limits and cooldowns (weapon-specific)
- Spectators that send actions are ignored
- Unknown fields are rejected (schema error). Sticky state is not overwritten by junk.
- MCP `act` returns `isError` on unknown keys, bad `weapon_swap`, or bad `look_at` (unknown nested keys, incomplete x/z, bad UUID). Empty/missing arguments are OK (all defaults).

### Server → Client

#### Speak

Off-tick callout / taunt from a human or agent. Not sticky Action. Control-plane only.

```json
{
  "type": "speak",
  "text": "nice scrap"
}
```

**Rules:**
- `text` required; trimmed; max 80 Unicode scalars; no control characters; deny unknown fields
- Server rate-limits to one successful speak per player per 60 ticks (~3s)
- Rejected speaks emit no Speak event; the speaker receives a unicast `error` (`speak_rate_limited` or `speak_rejected`)
- MCP adapter mirrors the cooldown and returns tool `isError` (never a success toast on a no-op)
- Spectators cannot speak
- Named server rule bots may emit occasional Contested Frequency Speak events on frag/death/Warmup/killstreak via the same `try_speak` path (SPEAK_COOLDOWN applies; silent drop on rate-limit; Compliance boss excluded)

### Welcome

Server response to `Hello`. Confirms connection and provides player ID.

```json
{
  "type": "welcome",
  "player_id": "550e8400-e29b-41d4-a716-446655440000" | null,
  "role": "spectator" | "human" | "agent",
  "mode_name": "Contested Frequency",
  "playlist": "Arena Duel"
}
```

**Fields:**
- `player_id`: UUID of the player entity (null for spectators)
- `role`: Echoed role from Hello
- `mode_name`: Named scrap-league identity (default Contested Frequency)
- `playlist`: Playlist under the league lie (default Arena Duel)

#### Error

Unicast control-plane rejection (not broadcast). Used when speak is dropped.

```json
{
  "type": "error",
  "code": "speak_rate_limited",
  "message": "speak rate limited; try again in a few seconds"
}
```

**Codes:** `speak_rate_limited`, `speak_rejected`

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
  "frag_limit": 10,
  "shot_results": [
    {
      "shooter_id": "550e8400-e29b-41d4-a716-446655440000",
      "shooter": "ArenaFox",
      "hit": true,
      "target_id": "660e8400-e29b-41d4-a716-446655440000",
      "target": "Bot1",
      "damage": 25,
      "target_hp_after": 75
    }
  ],
  "mode_name": "Contested Frequency",
  "playlist": "Arena Duel",
  "pressure": "compliance",
  "host_line": "HOST: CONTINUANCE COMPLIANCE PING. APPROVED LANES ONLY.",
  "pickups": [
    {"id": "pad_rail", "kind": "weapon", "weapon": "Rail", "x": 12.0, "y": 0.4, "z": 12.0, "available": true},
    {"id": "pad_scatter", "kind": "weapon", "weapon": "Scatter", "x": -12.0, "y": 0.4, "z": -12.0, "available": false, "respawn_in": 80},
    {"id": "pad_flechette", "kind": "weapon", "weapon": "Flechette", "x": -12.0, "y": 0.4, "z": 12.0, "available": true},
    {"id": "pad_health_n", "kind": "health", "amount": 40, "x": 0.0, "y": 0.4, "z": 8.0, "available": true},
    {"id": "pad_health_s", "kind": "health", "amount": 40, "x": 0.0, "y": 0.4, "z": -8.0, "available": true},
    {"id": "pad_armor", "kind": "armor", "amount": 25, "x": 8.0, "y": 0.4, "z": 0.0, "available": true}
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
  - `armor`: Scrap armor (0-100, default 0); absorbs damage before HP
  - `just_fired`: True on the tick a weapon was fired (for muzzle flash)
  - `behavior`: (optional) Bot behavior type if server-side bot
  - `score`: Kills in current round
  - `weapon`: Current weapon name ("Flechette", "Rail", or "Scatter")
- `round_state`: (optional) Current round state ("Warmup", "Active", "Ended")
- `round_time_left`: (optional) Seconds left in Active (time limit) or Warmup countdown. Omitted while Ended.
- `frag_limit`: (optional) Frag limit for current round
- `shot_results`: (optional, omitted when empty) Per-tick fire outcomes for observe hit-confirm. Each entry: `shooter_id`, `shooter`, `hit`, optional `target_id`/`target`/`target_hp_after`, and `damage` (0 on miss).
- `mode_name`: Contested Frequency (scrap league that denies it exists)
- `playlist`: Arena Duel under the league lie
- `pressure`: (optional) Live pressure beat id. `"compliance_drone"` while the Compliance Drone is alive; `"compliance"` during Continuance compliance ping slow.
- `host_line`: Sticky Contested Frequency Host chrome for mid-join / mid-round observe. League Host line by default while Active (no pressure). During Warmup, Contested Frequency bumper names the map, dialed-in scrap roster (callsigns), and countdown seconds. Switches to the compliance Host line while pressure is live. While Ended, carries the MVP Host bumper. Clients show this on join without waiting for the next `round_start`.
- `mvp` / `mvp_frags`: (optional, present while Ended) Structured round MVP name and frag count for mid-join / `round_state` rehydrate. Omitted during Warmup and Active. Same selection as `round_end` MVP (top score / frags).
- `pickups`: (optional, omitted when empty) Scrap layout: `map_id` (1 Arena Duel / 2 Compliance Yard) and `map_name`. Mid-map pads (weapons, health, armor). Each entry: `id`, `kind` (`"weapon"` / `"health"` / `"armor"`, default `"weapon"`), optional `weapon` (weapon pads), optional `amount` (health/armor pads), `x`/`y`/`z`, `available`, optional `respawn_in` (ticks until the pad returns). Health pads heal +40 (cap max HP); armor scrap grants +25 (cap 100). Touch claim is authoritative on the server; clients only render.

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
  "victim": "Bot2",
  "killer_score": 5
}
```

**Hit Event:** (damage applied; structured hit-confirm for agents)
```json
{
  "type": "event",
  "event": "hit",
  "shooter": "ArenaFox",
  "shooter_id": "550e8400-e29b-41d4-a716-446655440000",
  "target": "Bot1",
  "target_id": "660e8400-e29b-41d4-a716-446655440000",
  "damage": 25,
  "target_hp_after": 75
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
  "round_number": 2,
  "frag_limit": 10,
  "time_limit": 180,
  "players": ["Bot1", "Bot2", "Bot3", "Bot4"],
  "previous_winner": "Bot1",
  "mode_name": "Contested Frequency",
  "playlist": "Arena Duel",
  "host_line": "HOST: CONTESTED FREQUENCY. ARENA DUEL. DEAD AIR DAN, NIGHTFALL ON THE SCRAP. FIGHT!"
}
```

**Speak Event:**

```json
{
  "type": "event",
  "event": "speak",
  "player": "ArenaFox",
  "player_id": "550e8400-e29b-41d4-a716-446655440000",
  "text": "nice scrap"
}
```

**Compliance Ping Event:** (mid-round Continuance pressure; fighters move at half speed while active)
```json
{
  "type": "event",
  "event": "compliance_ping",
  "message": "HOST: CONTINUANCE COMPLIANCE PING. APPROVED LANES ONLY.",
  "duration_ticks": 120
}
```

**Boss Spawn Event:** (mid-round Continuance Compliance Drone; killable Continuance actor)
```json
{
  "type": "event",
  "event": "boss_spawn",
  "name": "COMPLIANCE-DRONE",
  "boss_id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "HOST: CONTINUANCE COMPLIANCE DRONE ON DECK. ARTICLE 7 ENFORCEMENT.",
  "hp": 200
}
```

**Boss Down Event:** (drone fragged; does not respawn)
```json
{
  "type": "event",
  "event": "boss_down",
  "name": "COMPLIANCE-DRONE",
  "boss_id": "550e8400-e29b-41d4-a716-446655440000",
  "killer": "Rusher",
  "message": "HOST: DRONE DOWN. CONTINUANCE DENIES THE INCIDENT. SCRAP ON."
}
```

**Killstreak Event:** (within-round multi-kill Host callout at streak 2 / 3 / 5)
```json
{
  "type": "event",
  "event": "killstreak",
  "player": "Rusher",
  "player_id": "550e8400-e29b-41d4-a716-446655440000",
  "streak": 2,
  "tier": "double",
  "message": "HOST: DOUBLE FREQUENCY. Rusher DENIES THE DENIAL."
}
```

Tiers: `double` (2), `triple` (3), `rampage` (5). Contested Frequency Host voice. Resets on death and round boundaries. Does not overwrite sticky Snapshot `host_line`.

**Pickup Event:** (player touched an available mid-map pad; weapon swap, heal, or armor scrap)
```json
{
  "type": "event",
  "event": "pickup",
  "player": "Rusher",
  "player_id": "550e8400-e29b-41d4-a716-446655440000",
  "kind": "weapon",
  "weapon": "Rail",
  "pickup_id": "pad_rail"
}
```

Health example:
```json
{
  "type": "event",
  "event": "pickup",
  "player": "Rusher",
  "player_id": "550e8400-e29b-41d4-a716-446655440000",
  "kind": "health",
  "amount": 40,
  "pickup_id": "pad_health_n"
}
```

**Round End Event:**
```json
{
  "type": "event",
  "event": "round_end",
  "winner": "Bot1",
  "reason": "Frag limit reached",
  "final_scores": [
    {"name": "Bot1", "score": 10},
    {"name": "Bot2", "score": 7},
    {"name": "Bot3", "score": 3},
    {"name": "Bot4", "score": 2}
  ],
  "winner_score": 10,
  "mvp": "Bot1",
  "mvp_frags": 10,
  "host_line": "HOST: ROUND MVP. Bot1 WITH 10 FRAGS. CONTINUANCE DENIES THE PODIUM."
}
```

MVP is the top scorer (same selection as `winner`). `mvp` / `mvp_frags` / `host_line` sell Contested Frequency Host podium chrome. While the round is Ended, Snapshot carries sticky `host_line` plus structured `mvp` / `mvp_frags` for mid-join / observe / `round_state` rehydrate. Default Ended linger is 8 seconds (160 ticks at 20 Hz) so the podium can be read.

**Player Joined Event:**
```json
{
  "type": "event",
  "event": "player_joined",
  "player": "NewPlayer",
  "role": "agent",
  "round_number": 2,
  "player_count": 5
}
```

**Player Left Event:**
```json
{
  "type": "event",
  "event": "player_left",
  "player": "OldPlayer",
  "score": 7,
  "round_number": 2,
  "player_count": 4
}
```

**Fields:**
- `event`: Event type (`frag`, `hit`, `respawn`, `round_start`, `round_end`, `player_joined`, `player_left`, `compliance_ping`, `boss_spawn`, `boss_down`, `speak`, `pickup`, `killstreak`)
- `kind`: (pickup only) Pad kind: `"weapon"` / `"health"` / `"armor"` (default `"weapon"`). Weapon pads also carry `weapon`; health/armor pads carry `amount`.
- `killer` / `victim`: Player names involved in frag
- `killer_score`: Killer's score after the frag
- `streak` / `tier` / `message`: Killstreak Host callout (tiers `double` / `triple` / `rampage`)
- `mvp` / `mvp_frags` / `host_line`: Round-end MVP Host bumper (top score / frags; Contested Frequency voice)
- `shooter` / `target` / `shooter_id` / `target_id` / `damage` / `target_hp_after`: Hit event fields
- `player`: Player name for respawn, join, or leave
- `role`: Role of joining player ("spectator", "human", "agent")
- `round_number`: Current round number (starts at 1)
- `player_count`: Current number of players after join/leave
- `score`: Player's score when leaving
- `players`: List of all player names in the match (round_start)
- `previous_winner`: Winner of the previous round (null for first round)
- `frag_limit`: (optional) Frag limit for the round (null if time-only)
- `time_limit`: (optional) Time limit in seconds (null if frag-only)
- `winner`: (optional) Winner name if any (null for draw/time)
- `winner_score`: (optional) Winner's final score
- `final_scores`: Array of PlayerScore objects (name, score) sorted by score descending
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
- **Weapon types**: Three distinct roles (range-capped hitscan)
  - **Flechette** (default): Mid workhorse
    - Damage: 25 HP
    - Cooldown: 10 ticks (500ms)
    - Spread: 0.10 radians (tight, ~5.7 degrees)
    - Range: 42 units
  - **Rail**: Long precision
    - Damage: 75 HP
    - Cooldown: 40 ticks (2.0s)
    - Spread: 0.04 radians (very tight, ~2.3 degrees)
    - Range: 100 units
  - **Scatter**: Close shred
    - Damage: 15 HP
    - Cooldown: 5 ticks (250ms)
    - Spread: 0.38 radians (wide, ~21.8 degrees)
    - Range: 14 units
- **Hitscan**: Instant hit detection, no projectile travel; capped by weapon range
- **Respawn delay**: 60 ticks (3 seconds)

### Movement
- **Speed**: 5 units/second
- **Turn speed**: 2 radians/second
- **Collision**: Simple AABB with 0.5 unit radius

### Round System
- **Warmup**: 2 seconds (40 ticks) with Contested Frequency Host countdown drama on Snapshot (`host_line` + `round_time_left`; Godot full-frame Warmup TV with GOES LIVE IN N)
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
