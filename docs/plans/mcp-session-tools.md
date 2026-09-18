# MCP session tools (join / leave / round_state)

## Goal

Expose first-class MCP tools for session lifecycle and round summary so agents do not treat Hello-on-start and process exit as the only join/leave path, and do not scrape `observe` for round fields.

## Non-goals

- look_at / hit-confirm changes (HOLD)
- Speak rate-limit reopen (already fixed)
- Pixel-3D look bar (shipped separately; was next tip priority after this)
- Protocol Leave message (leave remains clean WebSocket disconnect)

## Tools

| Tool | Behavior |
|---|---|
| `join` | Optional `name` (else `--name` / `FRAGR_AGENT_NAME`). Ensures connected Hello/Welcome. Idempotent success if already joined. Unknown fields -> schema error. |
| `leave` | Clean disconnect. `isError` if not connected. No fields. |
| `round_state` | Current round fields from last snapshot plus recent `round_start` / `round_end` (state, number, time, frag limit, mode_name, host_line, pressure). No scrape of full observe required. |

Hello-on-start remains a valid boot path. Tools are first-class and documented.

## Architecture impact

| Area | Change |
|---|---|
| `agent-adapter` `mcp.rs` | ToolState connected/name; `pending_join` / `pending_leave`; validators; tools/list; tests |
| `agent-adapter` `main.rs` | Boot Hello kept; leave closes WS; join after leave reconnects + Hello/Welcome |
| Docs | adapter README, SKILL.md, protocol.md, ARCHITECTURE MCP note, plans tip priorities |
| README tip gallery | Embed 08 host-flash midjoin + weapons/frags tip still; mood stays under mood/ |

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
```

Unfiltered fail-under 80. No coverage carve-outs.

## Success criteria

- PR open with join / leave / round_state tools real, documented, tested
- README shows 08 + combat tip stills
- CI-ready honest coverage lock
