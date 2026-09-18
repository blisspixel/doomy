# Honest coverage lock

## Cheat (what Gitty called out)

CI previously ran:

```bash
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80 \
  --ignore-filename-regex '(agent-adapter|server/src/(main|net)\.rs)'
```

That carved `agent-adapter` and `server` `main.rs` / `net.rs` out of the denominator, so a ~91% score only proved protocol + sim. Nick's lock: **>=80% by testing**, never by carving files out.

## Fix

1. Drop `--ignore-filename-regex` from CI and AGENTS.md. Keep `--fail-under-lines 80`.
2. Extract `server` join/leave/action/tick broadcast into `session.rs` and cover it with unit tests.
3. Add real WebSocket behavioral tests for Hello / Welcome / Action / disconnect / snapshot+event broadcast in `net.rs` paths.
4. Extract adapter MCP observe / act / get_events (plus initialize / tools/list / ingest) into `mcp.rs` with behavioral unit tests; Hello / `--name` resolution stays covered.
5. Encode: coverage ignores for production crates are **KAPU**.

## Non-goals

- Lowering the 80% floor
- Re-adding filename ignores for production crates
- Claiming mood art is tip gameplay

## Verification

```bash
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
```

Unfiltered report must pass fail-under 80 locally and in CI.
