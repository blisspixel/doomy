# Defensive Programming + 80% Line Coverage

**Status:** Active  
**Target:** Slice 1 quality bar raise

## Goal

Harden fragr Rust workspace (`server`, `agent-adapter`) with defensive validation at trust boundaries and prove behavior with ~80% line coverage, measured and enforced in CI.

## Non-goals

- Godot client coverage (GDScript not in scope)
- 100% coverage perfection (80% is the bar)
- Rewriting existing working code without test-driven reason

## Architecture impact

None. Pure quality lift: add tests, add validation, wire coverage measurement. No protocol changes, no new features, no behavior changes except rejecting malformed input instead of panicking.

## Validation boundaries (defensive programming)

**Trust boundaries** where external/untrusted data enters:

1. **WebSocket messages** (`server/src/net.rs`)
   - Hello: validate role, name length (e.g. 1-32 chars)
   - Action: validate booleans only (serde handles this), but add explicit Action validation for future extensibility (aim angles, weapon indices)

2. **Protocol deserialize** (`server/src/protocol.rs`, `agent-adapter/src/protocol.rs`)
   - Reject invalid JSON gracefully
   - Reject unknown message types gracefully
   - Add tests for malformed ClientMessage/ServerMessage

3. **Agent adapter inputs** (`agent-adapter/src/main.rs`)
   - MCP tool call arguments: validate action fields before sending to server
   - observe when no snapshot yet (return empty, not crash)
   - act when not connected (fail gracefully)

**Validation strategy:**

- Fail closed: reject bad input, log, return error or default
- Never panic on client-controlled data
- Prefer Result/Option and handle None cases explicitly

## Test coverage targets

**Target: ~80% line coverage**, measured with `cargo llvm-cov` on Linux.
**CI enforces:** Ratchet floor (currently 50%) that only moves up as tests are added.

Focus areas:

- **Protocol** (`protocol.rs` in both crates)
  - Valid Hello/Welcome/Action/Snapshot round-trip
  - Malformed JSON deserialize
  - Unknown message types
  - Edge cases (empty name, missing fields with defaults)

- **Sim** (`server/src/sim.rs`)
  - Already has good tests; extend to cover:
    - Action edge cases (all false, all true)
    - Boundary clamping (arena edges)
    - Respawn timer edge (0, 1, 60)
    - Round state transitions (warmup -> active -> ended -> active)
    - Empty game state (no players)
    - Single player (no targets for hitscan)

- **Net session** (`server/src/net.rs`)
  - Hello with valid role
  - Hello with missing/invalid fields
  - Action only routed when not Spectator
  - Disconnect cleanup (client removed from list)
  - Single player_id consistency (same ID for entire connection)

- **Agent adapter** (`agent-adapter/src/main.rs`, `agent-adapter/src/protocol.rs`)
  - MCP initialize
  - observe when connected (returns snapshot)
  - observe when no snapshot yet (returns empty)
  - act with partial arguments (defaults apply)
  - act with all arguments
  - Protocol types match server

## CI changes

**File:** `.github/workflows/ci.yml`

Add step after "Run tests":

```yaml
- name: Install llvm-cov
  run: cargo install cargo-llvm-cov

- name: Check coverage
  run: |
    cargo llvm-cov --workspace --lcov --output-path lcov.info
    cargo llvm-cov report --fail-under-lines 50
```

Keep existing: fmt, clippy -D warnings, test, build.

Coverage runs on Linux only (ubuntu-latest). Ratchet floor starts at 50%, increases as tests are added toward ~80% target.

## Verification steps

Local (before push):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace

# Install cargo-llvm-cov if not present
cargo install cargo-llvm-cov

# Measure coverage
cargo llvm-cov --workspace

# Check 80% bar
cargo llvm-cov report --fail-under-lines 80
```

CI will enforce same checks on PR.

## Success criteria

- [ ] `docs/plans/defensive-coverage.md` committed
- [ ] `AGENTS.md` Verification section updated with defensive + coverage requirements
- [ ] Protocol validation tests added (both server and agent-adapter)
- [ ] Action validation tests added
- [ ] Net session tests added
- [ ] Adapter observe/act edge tests added
- [ ] Sim coverage gaps filled (if any)
- [ ] Workspace line coverage measured with llvm-cov
- [ ] CI step added: coverage measured with ratchet floor (50%, moves up only)
- [ ] Local verification passes: fmt, clippy -D warnings, test, build
- [ ] PR opened, ready for merge

Target: ~80% line coverage (standing goal, enforced via ratchet floor that increases with new tests)

## Commands

```bash
# Local test + coverage check
cargo test --workspace
cargo llvm-cov --workspace
cargo llvm-cov report --fail-under-lines 80

# Local full verification
cargo fmt --all -- --check && \
cargo clippy --workspace --all-targets -- -D warnings && \
cargo test --workspace && \
cargo build --workspace && \
cargo llvm-cov report --fail-under-lines 80
```

## Notes

- Coverage target is line coverage, not branch coverage (simpler, still meaningful)
- If first measurement shows e.g. 70%, acceptable to set floor to 70% in CI with TODO comment for 80%, but prefer hitting 80% in this PR
- Tests must prove behavior, not just call functions with no asserts
- No tool/model attribution anywhere
- No emoji, no em/en dashes
