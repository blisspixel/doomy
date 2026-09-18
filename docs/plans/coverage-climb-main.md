# Coverage climb: main.rs shells

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/coverage-climb-main`
**Spend:** $0. No ElevenLabs, no GCP apply, no renet, no look_at/hit reopen.
**Status:** Ready to ship.

## Goal

Gitty leftover after honest coverage lock: climb unfiltered llvm-cov by testing the remaining empty shells, especially `server/src/main.rs` (~0%) and thin / untested seams in `agent-adapter/src/main.rs`. Prefer real behavioral and integration proofs (CLI args, startup bind, WS Hello/Welcome smoke, MCP stdio line apply, scripted-bot handshake) over fake coverage. Never lower fail-under 80. No carve-outs / ignore-filename-regex.

## Tip priorities

Tip stills recapture shipped (#77 / `4aeb160`). **Coverage climb main is the NOW tip.** look_at / hit HOLD. Port 6767. Honest llvm-cov fail-under 80 unfiltered.

## Non-goals

- Feature gameplay / protocol changes
- look_at / hit reopen (HOLD)
- GCP apply
- Lowering fail-under 80
- Re-adding filename ignores for production crates
- Weakening Clippy `-D warnings`

## Locks

- Contested Frequency pixel grit (bone / gunmetal / blood / ember)
- Port 6767
- Coverage fail-under 80 (unfiltered, no ignore-filename-regex)
- look_at / hit HOLD
- No GCP apply

## Approach

1. **Server `main.rs`:** Keep product main thin. Extract `run_server(bind, bots, shutdown)` so the bind / accept / session tick / command apply path is testable. Add clap `Args::try_parse_from` coverage. Add a WS Hello -> Welcome -> tick broadcast smoke that shuts down cleanly.
2. **Adapter `main.rs`:** Keep MCP / scripted-bot behavior intact. Add clap subcommand parse tests. Extract MCP stdio line apply so oversized / malformed / leave / join / act / speak paths are proven without hollowing. Add WS handshake smoke for `mcp_connect_and_hello` + `mcp_leave_session` and a short scripted-bot connect against a local welcome server.
3. Keep CI: `cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80` with no ignore-filename-regex.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
cargo build --workspace
```

Record before/after line % (workspace TOTAL and both `main.rs` files).

## Success criteria

- Plan in tree; tip priorities list coverage climb as NOW (tip stills shipped)
- PR open on `cursor/coverage-climb-main`
- `server/src/main.rs` no longer ~0% empty shell
- Adapter main seams exercised with real CLI / WS / stdio-line behavior
- CI green at fail-under 80; Clippy still `-D warnings`

## Coverage delta (local llvm-cov)

| Scope | Before | After |
|---|---|---|
| Workspace TOTAL lines | 87.91% | 92.25% |
| `server/src/main.rs` | 0.00% | 83.61% |
| `agent-adapter/src/main.rs` | 73.14% | 90.15% |

Fail-under 80 unfiltered still enforced. No ignore-filename-regex.

