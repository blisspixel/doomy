# Plan: Slice 1 exceptional polish (fragr PR #1)

**Repo:** https://github.com/blisspixel/fragr  
**Branch:** `cursor/slice1-arena-duel-8557`  
**Spend:** $0. No paid assets, APIs, or hosting.  
**Status:** Plan of record before further implementation.

## Goal

Take Slice 1 from "scaffold that mostly demos bots" to an **exceptional** loopback playable: spectator delight, correct human join/leave, smooth enough presentation, real verification evidence, docs that match the tree.

## Non-goals

- UDP / renet migration
- Prediction / rollback
- LLM agents or paid APIs
- Public hosting / Tailscale packaging
- Full rename of crate/binary names off `doomy-*` to `fragr-*` (completed in separate rename PR)
- CI (Gitty when asked)

## Findings from code review (evidence)

### P0: Human / agent player_id mismatch (bug)

In `server/src/net.rs`, Hello handling creates a Welcome `player_id` (UUID A), then later creates a **second** UUID B used for `GameCommand::Action`. In `server/src/main.rs`, `GameCommand::Connected` creates a **third** UUID C for `GameState::add_player` and `client_to_player`.

Actions therefore target an ID that is not in the sim. Human join can look connected while shots/movement never apply to the pawn the client thinks it owns.

**Fix:** One player UUID per non-spectator connection. Generate once on Hello, send that ID in Welcome, pass it through `Connected`, use it for Action mapping. Add a focused regression test.

### P1: No client interpolation

Pawns appear to snap to snapshot poses. For a 20 Hz WS snapshot stream, add simple render interpolation (lerp/slerp toward latest authoritative pose) without giving the client sim authority.

### P1: Missing real automated tests

No meaningful `#[cfg(test)]` / integration coverage found for hitscan, frag, bot tick, or ID wiring. `cargo test` green is not enough without assertions.

### P2: Feel gaps vs research fun bar

- Bot **intent chips** not shown on HUD (named behaviors exist server-side: Rusher/Sniper/Flanker/Tank).
- Slice checklist in `docs/SLICE-1.md` still unchecked; smoke script still mentions `--bots 2` in one place.
- Optional: short CC0 or engine-default hit/frag audio only if free and already available (no asset-store spend).

### P2: Docs drift

README on the branch was updated with quick start; keep protocol + SLICE-1 checkboxes honest after fixes.

## Architecture impact

- Server net + main command plumbing only for P0.
- Client pawn / game_manager for interpolation and optional intent labels (protocol may need a small optional `intent` or `behavior` string on `PlayerState` if not already present; prefer optional field, backward compatible).
- Tests under `server/src/` (unit) and optionally a tiny WS smoke binary or `tokio::test` if practical without flaking.

## Verification

Before done:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
cargo run -p fragr-server -- --bots 4   # frags in logs within ~30s
```

Manual: Godot 4.7.2 open `client/`, F5 spectator, press J, confirm own pawn moves/shoots and can frag; press L back to spectate. Kill server: client handles disconnect cleanly.

No emoji, no em/en dashes, no tool attribution in any new prose/commits/PR text.

## Success criteria

1. Single consistent player_id path: Welcome == sim ID == Action ID (proven by test).
2. Human join can move, shoot, take damage, frag a bot in the same match.
3. Spectator pawns interpolate (no harsh teleport at 20 Hz under normal conditions).
4. At least unit tests covering hitscan frag + player_id wiring.
5. HUD shows fighter names and a simple intent/behavior chip when available.
6. `docs/SLICE-1.md` checkboxes updated to match proven reality; smoke commands use `--bots 4`.
7. Spend remains $0.

## Build order

1. Fix player_id wiring + regression test.
2. Add sim combat unit tests (hitscan/frag/respawn).
3. Client interpolation for remote pawns.
4. Intent/behavior on snapshot + HUD chip (minimal).
5. Doc/checklist sync + PR body note of what changed.
6. Full verification suite above.

## Spend / safety

$0 only. Fail closed on any paid dependency. Loopback only.

## Added DoD (Chief / Nick 2026-09-17)

Second peer must spectate the same match on LAN or Tailscale Personal ($0):

- Server `--bind` supports non-loopback (document `0.0.0.0:7777` or LAN IP).
- Client accepts server host/URL for a second machine.
- README documents two-machine spectator via LAN and Tailscale Personal.
- No paid VPS.

