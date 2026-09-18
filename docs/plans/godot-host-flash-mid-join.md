# Godot Host flash on mid-join Snapshot

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/godot-host-flash-mid-join`
**Spend:** $0. No ElevenLabs, no GCP apply, no renet.
**Status:** Plan of record for Godot Host bumper flash on first mid-round Snapshot.

## Goal

When a spectator or human joins mid-round, the first Snapshot that carries sticky `host_line` flashes Host chrome once in Godot (same energy as the RoundStart Host bumper), not only a quiet mode-label chip.

## Context

Sticky `host_line` on Snapshot shipped in #63 and Casino-cleared on MCP observe. Wire is proven. Godot Host flash still needed an honest mid-round path: flash once on first Active/Ended Snapshot after connect, never spam later Snapshots, and avoid double-flashing Warmup joins that still get RoundStart.

## Non-goals

- Reopen look_at / hit
- Change speak rate-limit (stays closed / isError)
- Rust protocol changes unless a wire bug is found
- ElevenLabs / GCP apply

## Architecture impact

| Area | Change |
|---|---|
| `client` HUD | `set_host_line(..., flash_on_first)` one-shot; `reset_host_chrome` on disconnect; `show_host_join` bumper |
| `client` game_manager | Flash only when Snapshot `round_state` is Active or Ended; play RoundStart sound on that flash |
| `client` tip_capture | Optional mid-join reconnect still `08_tip_host_flash_midjoin_16x9.png` |
| Plans tip priorities | Sticky host_line shipped; this plan is the tip Host-flash cut |

## Behavior

1. Connect mid-round (Active or Ended): first non-empty Snapshot `host_line` calls `show_host_join` once and latches `host_line_seen`.
2. Later Snapshots only refresh the sticky Host chip on the mode label.
3. Warmup connect: sticky Host chip updates without Snapshot flash; RoundStart still owns the Host bumper.
4. Disconnect clears Host chrome latch so a later reconnect can flash again.

## Verification

```bash
# Godot script parse / import (4.7.2-stable)
/workspace/godot/Godot_v4.7.2-stable_linux.x86_64 --path client --headless --import --quit-after 60

# Optional tip proof (server with bots + Xvfb capture)
cargo run -p fragr-server -- --bots 4
tools/capture_tip_screenshots.sh
# Inspect docs/screenshots/08_tip_host_flash_midjoin_16x9.png for HOST: RoundMessage chrome
```

No Rust required for the happy path. If Rust is touched, keep unfiltered `cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80`.

No tool attribution, emoji, or em/en dashes in commits/PR/docs.

## Success

- PR open on `cursor/godot-host-flash-mid-join`
- Mid-join first Active/Ended Snapshot flashes Host line once
- Warmup does not double-flash before RoundStart
- Tip priorities mark sticky host_line shipped and this cut in flight or shipped
- CI green path (Godot-only diff preferred)
