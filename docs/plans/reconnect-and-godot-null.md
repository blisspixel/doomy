# Plan: Clean reconnect + Godot add_child null guard

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/reconnect-godot-null`
**Spend:** $0. No ElevenLabs, no GCP apply, no renet, no look_at reopen.
**Status:** Plan of record for tip-face reconnect polish and soft Godot null on combined pads.

## Goal

Two small tip-face polish items that raise stability and fun:

1. **Clean reconnect path** (Chief bar: join / leave / reconnect). Human or adapter can drop and re-Hello (same name or new session) without soft prison, stuck session, or ghost player.
2. **Soft Godot `add_child` null** that Testy noted on combined pads Casino. Guard null child add on pickup/HUD spawn paths so tip face stays clean.

## Context

- Arena choke geometry shipped (#72 / `13f31c6`). Pads, drone, and chokes stay.
- Join/leave MCP events Casino-cleared earlier; Chief still wants reconnect without ghosts when a prior socket is slow to die.
- Godot J/L currently closes the WebSocketPeer then reuses it. Godot 4 peers are not reliably reusable after close; that sticks join/leave.
- Snapshot sync instantiates pawns and pads then `arena.add_child(...)` with no null guard. Combined weapon+health+armor pads amplify any bad instantiate.

## Non-goals

- GCP apply
- look_at / hit reopen (HOLD)
- ElevenLabs
- New map / choke layout changes
- Protocol Leave message (leave remains clean disconnect)

## Tip priorities

Chokes shipped (#72). This cut is **NOW**.

## Architecture impact

| Area | Change |
|---|---|
| `server` session | On Connected (non-spectator), evict any existing client-mapped player with the same display name (ghost reclaim). Rule bots stay (not in `client_to_player`). Push PlayerLeft for the ghost. |
| `server` tests | Prove leave+rejoin same name; overlapping reconnect same name leaves one player and no stuck mapping; new name is a new session. |
| `client` net_client | Fresh `WebSocketPeer` on each connect; clear `player_id` on disconnect; safe reconnect after close. |
| `client` game_manager | Clear pawns/pads on disconnect; null-guard instantiate + `arena` before `add_child` (players and pickups). |
| Docs | This plan; tip priorities in `docs/plans/README.md`. |

## Behavior

### Server reconnect

1. Hello as human/agent with name N: if a client-mapped player already has name N, remove that ghost (PlayerLeft, drop `client_to_player` entries, remove from sim) before adding the new player.
2. Rule bots (no client mapping) are never evicted by name.
3. Clean leave then join same name: no ghost to evict; one player after join.
4. Overlapping reconnect (new Hello before old Disconnect): eviction clears old mapping so a late Disconnect is a noop and does not remove the new player.
5. New session / new name: no eviction; prior leave must already have removed the old player.

### Godot

1. `connect_to_server` always allocates a new `WebSocketPeer` (and closes any prior open peer).
2. `disconnect_from_server` closes, clears `player_id`, emits once.
3. On disconnect, GameManager frees pawns and pickup nodes and clears maps so rejoin starts clean.
4. `_on_snapshot_received` / `_sync_pickups`: skip `add_child` when instantiate returns null or arena is invalid.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
```

Manual / smoke:

1. `cargo run -p fragr-server -- --bots 4` on 6767.
2. Godot spectator: press J (join human), L (leave spectate), J again. No stuck "Connecting", no duplicate Human Player on the scoreboard.
3. Adapter leave then join same `--name`: one agent in observe; no ghost.
4. Tip face: weapon + health + armor pads render without SCRIPT ERROR / Parameter "p_child" is null.

Locks: port 6767; fail-under 80 unfiltered; look_at/hit HOLD; pads/drone/chokes stay.

No tool attribution, emoji, or em/en dashes in commits, PR text, or docs.

## Success

- PR open on `cursor/reconnect-godot-null`
- Reconnect same name / new session without ghost player (tests prove it)
- Godot null `add_child` guarded; tip face clean on combined pads
- CI-ready honest coverage lock
