# Testy Stranger Playtest Feedback Fixes

**Target:** v0.3.0 (f6ecdea with weapons)  
**Spend:** $0  
**Author:** Nick Seal <32712898+blisspixel@users.noreply.github.com>

## Issues

1. **MCP events buffer incomplete:** Agent-adapter drops join/leave and round events (round_start, round_end) even though server emits them
2. **Observe vs README mismatch:** Live observe output thinner than agent-adapter README promises
3. **Window title fixed:** Godot project now shows "fragr Client"

## Root Causes

### Issue 1: Missing event types in adapter protocol
- Server emits `RoundStart` and `RoundEnd` events (sim.rs lines 92-96, 117-119)
- Server protocol.rs defines these in `GameEvent` enum (protocol.rs lines 128-137)
- Agent-adapter protocol.rs only defines `Frag` and `Respawn` (agent-adapter/src/protocol.rs lines 64-68)
- Adapter cannot parse round events, logs warnings and drops them

Join/leave events: server does not currently emit these as explicit events. Server logs them but does not send Event messages to clients. Need to add PlayerJoined and PlayerLeft to GameEvent enum in both server and adapter, then emit them from net.rs when players connect/disconnect.

### Issue 2: Observe returns raw snapshot
- Server Snapshot includes round_state, round_time_left, frag_limit as optional fields (server/src/protocol.rs lines 95-99)
- Adapter observe just returns cached snapshot as-is (agent-adapter/src/main.rs lines 287-301)
- README promises these fields in observe output (agent-adapter/README.md lines 65-67)
- Fields already present in snapshot, so observe should be passing them through correctly
- Need to verify actual server behavior and ensure fields are consistently populated

### Issue 3: Godot project name

Client application name has been updated to "fragr Client" in `client/project.godot`.

## Implementation Plan

### 1. Add round and join/leave events to adapter protocol (Issue 1)

**File:** `agent-adapter/src/protocol.rs`

Add RoundStart, RoundEnd, PlayerJoined, PlayerLeft to GameEvent enum:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum GameEvent {
    Frag { killer: String, victim: String },
    Respawn { player: String },
    RoundStart {
        round_number: u32,
        frag_limit: Option<u32>,
        time_limit: Option<u32>,
    },
    RoundEnd {
        winner: Option<String>,
        reason: String,
    },
    PlayerJoined {
        player: String,
        role: String,
    },
    PlayerLeft {
        player: String,
    },
}
```

### 2. Add join/leave events to server protocol and emit them (Issue 1)

**File:** `server/src/protocol.rs`

Add PlayerJoined and PlayerLeft to GameEvent enum (same structure as above).

**File:** `server/src/net.rs`

When accepting a new connection after Welcome, push PlayerJoined event to game state.
When connection drops, push PlayerLeft event to game state.

### 3. Update adapter README to match implementation (Issue 2)

**File:** `agent-adapter/README.md`

Review current observe example (lines 44-71). The server Snapshot already includes round_state, round_time_left, frag_limit as optional fields. Verify these are present in observe output by:
- Checking that server populates them in snapshot (sim.rs line 405-444)
- Confirming adapter passes them through (main.rs line 289 creates observation from snapshot_value)

If fields are missing in output, fix server snapshot generation. If present, ensure README example matches current behavior exactly.

### 4. Rename Godot window title (Issue 3)

**File:** `client/project.godot`

Line 13: Application name updated to `config/name="fragr Client"`

## Verification

1. **Cargo checks:**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo build --workspace
   ```

2. **Manual smoke test:**
   - Start server with 4 bots
   - Connect scripted bot via adapter
   - Verify get_events returns round_start, round_end events
   - Verify observe returns round_state, round_time_left, frag_limit
   - Check that join/leave events appear when bots connect/disconnect
   - Open Godot client, verify window title shows "fragr Client"

3. **Test coverage:**
   - Add test cases for round event parsing in agent-adapter/src/main.rs tests
   - Add test for join/leave event parsing
   - Verify existing tests still pass

## Success Criteria

- Testy can see join/leave events in get_events/recent_events
- Testy can see round_start/round_end events in get_events/recent_events
- observe output matches agent-adapter README examples (round_state, round_time_left, frag_limit present)
- Godot window title shows "fragr Client"
- All cargo checks green
- Zero Cursor/Claude/Codex attribution
- No emoji, no em/en dashes
- $0 spend
