# Plan: Host-per-NODS-tick juice

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `feat/host-per-nods-tick`
**Spend:** $0. Loopback. No Cloud Agent. No release tag.
**Status:** shipped (#120).
**Tip base:** main after #116 jammer dish (`5c94706` or newer).

## Goal

When Solo Broadcast Calibration NODS cleared count increments, pay a short Host / HUD beat on every NODS frag, not only the first Host line. Keep unserious Contested Frequency booth energy. No AGI theater. Do not spam speak.

## Tip context

- #111 credited meatbag NODS clears and shipped jammer Snapshot chrome.
- #116 made the jammer dish unmissable in-camera.
- Soft optional deferred from `jammer-dish-silhouette.md`: Host-per-NODS-tick.

## Root cause (verified)

`GameState::note_nods_frag` only sets `solo_broadcast.host_line` when `nods_cleared == 1`. Snapshot `episode_progress` already ticks `NODS n/5` every clear, but Godot never flashes on that change, so clears 2..4 feel mute after the first Host bumper.

## Ship in this PR

1. Plan + plans README index row.
2. Server: rate-sane Host bump on every credited NODS clear while phase stays Nods (`episode0_host_line_nods_tick`). Goal clear still owns the jammer Host line (no double Host). No new speak events. No new GameEvent type.
3. Client: on `episode_progress` change while `episode_phase == nods` (after first paint), short Host / HUD flash (`show_nods_tick`): ember streak flash, brief RoundMessage, badge lift. Skip empty-to-first paint and non-nods phases so join and jammer handoff stay quiet.
4. Tests: host_line advances on clears 1..4; goal clear still jammer Host; progress chip still ticks.

## Non-goals

- New wire event or speak spam
- Tip still recapture / release tag
- Tab-less stance chips
- Neon flood or AGI Host monologue
- Jammer / Auditor / win juice beyond existing lines

## Architecture impact

| Area | Change |
|---|---|
| `docs/plans/host-per-nods-tick.md` | This plan |
| `docs/plans/README.md` | Index row |
| `server/src/protocol.rs` | `episode0_host_line_nods_tick` |
| `server/src/sim.rs` | Host bump per NODS clear; jammer still wins at goal |
| `server/src/tests.rs` | Host tick assertions |
| `client/scripts/hud.gd` | Progress-change flash latch + `show_nods_tick` |
| `client/scripts/game_manager.gd` | Unchanged Snapshot path (chrome setter owns flash) |

## Behavior

1. NODS clear 1..4: sticky Snapshot `host_line` updates to a short booth tick with `NODS n/goal`.
2. Same clear: Godot sees `episode_progress` change under phase `nods` and fires a brief Host flash (not a long killstreak bumper).
3. Clear that hits goal: phase Jammer + jammer Host only. No NODS-tick flash on that Snapshot (phase left nods).
4. Mid-join / reconnect: first progress paint does not flash; existing Host join latch still applies.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
bash tools/godot_check.sh
```

Focused: `note_nods_frag_*`, new host tick assertions.

## Success

- Plan landed before code
- One lean PR `feat/host-per-nods-tick`, squash-merged when CI green
- Each NODS frag (1..4) pays Host sticky + short HUD flash
- Goal clear still reads as jammer Host
- No speak spam, no tag
