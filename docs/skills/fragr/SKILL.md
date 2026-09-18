# fragr agent skill (OpenClaw / Hermes)

Control a fighter on the local arena without touching the combat tick.

## Prerequisites

1. Server listening on port 6767:
   `cargo run -p fragr-server -- --bots 4`
2. Adapter as your agent name:
   `cargo run -p fragr-agent-adapter -- mcp --server ws://127.0.0.1:6767 --name YourAgent`
   Env alternate: `FRAGR_AGENT_NAME=YourAgent`

Boot still Hellos on adapter start with `--name`. First-class session tools: `join`, `leave`, `round_state`.

## Tools

- `observe` - current snapshot (poses, hp, weapon, score, behavior, round fields, shot_results) plus recent_events
- `act` - discrete intents: forward/back/left/right, turn_left/turn_right, fire, weapon_swap, look_at
- `speak` - short off-tick taunt/callout (max 80 chars, rate-limited); spectators see it; lands in recent_events; rate-limit / reject returns `isError` (never a fake success)
- `get_events` - last ~50 game events (frag, hit, respawn, round_start, round_end, join/leave, compliance_ping, boss_spawn, boss_down, speak)
- `join` - ensure Hello/Welcome (optional `name`, else `--name`); idempotent if already joined
- `leave` - clean disconnect; `isError` if not connected
- `round_state` - round summary (state, number, time, frag limit, mode_name, host_line, pressure, mvp, mvp_frags) from last snapshot + recent round_start/round_end; Snapshot mvp wins while Ended

## Loop

1. `join` if needed (boot Hello already joined)
2. `observe` / `round_state`
3. Choose action from structured state (not pixels). Aim with `look_at.player_id` (or x/z). Read `shot_results` / `hit` events for damage feedback.
4. `act` at ~1-10 Hz
5. Repeat until round_end; `leave` to disconnect cleanly (or process exit)

Same Action path as humans and scripted bots. Keep LLM off the 20 Hz tick.

## Speak / taunt

Use `speak` for short Contested Frequency callouts. Keep LLM off the 20 Hz tick. Same spectators + events ring as frags. If rate-limited, the tool returns `isError` with a clear message; wait ~3s and retry.
