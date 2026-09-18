# Plan: Bug-hunt polish pass

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/bug-hunt-polish`
**Base tip:** `79180c7`
**Spend:** $0. Loopback. No GCP apply. look_at / hit HOLD.
**Author:** Nick Seal <32712898+blisspixel@users.noreply.github.com>

## Goal

Hunt tip for feel bugs / soft prisons / leftover wrongness after Warmup Host drama (#83) and public-or-local docs (#84). Ship a concrete polish PR (not docs-only).

## Research (tip `79180c7`)

| Finding | Severity | Action |
|---|---|---|
| Live Compliance Drone survives into Ended; `clear_boss` on `start_round` wipes with no `boss_down`. MCP / pressure / mid-join see drone during podium or silent recycle. Parked soft: boss_down on round_end wipe. | Soft prison + tip | **Fix now** |
| `_apply_map_from_snapshot` `packed.instantiate()` then `add_child` with no null guard (player/pickup already guarded in #73). | Stability | **Fix now** |
| Spectator follow cam can keep freed pawn refs; `get_followed_target` returns invalid instances. | Spectator feel | **Fix now** |
| `net.rs` outbound `serde_json::to_string(...).unwrap()` can panic a client send task. | Stability | **Fix now** |
| `boss_down` with JSON `killer: null` (round wipe) should not stringify into a fake killer chip. | Tip face | **Fix now** |
| Host bumper `await` hide races (Warmup blanks RoundStart). | Tip-face | **Parked**: open PR #89 `cursor/warmup-tv-bumper` owns Warmup tip chrome; do not duplicate. |
| Leftover `7777` in tip code | Scrub | None in code; docs only mention as historical "not 7777". **Parked** (docs OK). |
| look_at / hit confirm reopen | HOLD | **Parked** |
| Public OR local / Tailscale test-only | Docs | Shipped #84. No change. |
| Linger Ended / map_id on round_state | Shipped | #81 / #82. No change. |

## Architecture impact

| Area | Change |
|---|---|
| `server` protocol | `boss_round_wipe_host_line()` for recycle / round-end dismiss. |
| `server` sim | On `end_round`, wipe live boss and emit `BossDown` (killer null) before `RoundEnd`. |
| `server` net | Outbound serialize failure logs and drops the send task instead of unwrap panic. |
| `server` tests | Round-end with live boss emits wipe `boss_down` then `round_end`; boss absent from Ended snapshot. |
| `client` game_manager | Null-guard map layout instantiate; harden `boss_down` killer null. |
| `client` spectator_cam | Filter invalid targets; `get_followed_target` validity check. |
| `docs/plans` | This plan; tip priorities index entry. |

## Non-goals

- Warmup TV bumper / Host bumper await race (owned by #89)
- look_at / hit HOLD reopen
- GCP apply / Terraform apply / ElevenLabs
- Scrubbing historical "not 7777" doc notes
- Attribution, emoji, em/en dashes

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
```

Manual: spectate `--bots 4` with early boss spawn; wait for frag/time limit; confirm `boss_down` (no killer) then podium, no drone on Ended snapshot. Map rotate / Compliance Yard swap must not crash on null layout.

## Spend / safety

$0. No secrets. Ship via `gh` API (Cloud Agent quota exhausted). No attribution, emoji, or em/en dashes.

## Success criteria

- [x] Plan in tree listing findings + fix-now vs parked
- [x] Live boss wiped on round_end with `boss_down` (killer null)
- [x] Map layout instantiate null-guarded
- [x] Spectator cam drops freed follow targets
- [x] Net outbound serialize does not unwrap-panic
- [x] Rust tests for boss wipe; coverage fail-under 80
- [ ] PR open on `cursor/bug-hunt-polish` with real fixes (not docs-only)
