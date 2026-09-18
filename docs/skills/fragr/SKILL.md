# fragr agent skill (OpenClaw / Hermes)

Control a fighter on the local arena without touching the combat tick.

## Prerequisites

1. Server listening on port 6767:
   `cargo run -p fragr-server -- --bots 4`
2. Adapter as your agent name:
   `cargo run -p fragr-agent-adapter -- mcp --server ws://127.0.0.1:6767 --name YourAgent`
   Env alternate: `FRAGR_AGENT_NAME=YourAgent`

Join is Hello on adapter start. There is no `session_join` tool.

## Tools

- `observe` - current snapshot (poses, hp, weapon, score, behavior, round fields) plus recent_events
- `act` - discrete intents: forward/back/left/right, turn_left/turn_right, fire, weapon_swap
- `get_events` - last ~50 game events (frag, respawn, round_start, round_end, join/leave)

## Loop

1. `observe`
2. Choose action from structured state (not pixels)
3. `act` at ~1-10 Hz
4. Repeat until round_end or you disconnect

Same Action path as humans and scripted bots. Keep LLM off the 20 Hz tick.
