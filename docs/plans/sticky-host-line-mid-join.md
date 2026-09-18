# Sticky host_line for mid-join

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/sticky-host-line-mid-join`
**Spend:** $0. No ElevenLabs, no GCP apply, no renet.
**Status:** Plan of record for Contested Frequency Host chrome on mid-join / mid-round observe.

## Goal

Make `host_line` sticky so late joiners and mid-round observers always see Contested Frequency Host chrome without waiting for the next RoundStart.

## Hypothesis (verified)

`host_line` lived only on `round_start`. Snapshot carried `mode_name` / `playlist` / `pressure` but not `host_line`. Godot HUD showed the Host bumper only from `round_start` (and compliance banner only from the live `compliance_ping` event). Mid-join after Warmup missed RoundStart; mid-join after the compliance ping missed the bumper event.

## Non-goals

- Reopen look_at / hit
- ElevenLabs / GCP apply
- Welcome-only chrome (Welcome stays mode/playlist; sticky current Host text is Snapshot)

## Architecture impact

| Area | Change |
|---|---|
| `server` protocol | Snapshot always carries `host_line` (serde default = league Host line) |
| `server` sim | `snapshot()` sets league Host line, or compliance Host line while pressure is live |
| `agent-adapter` | Mirror Snapshot `host_line` so observe stays in sync |
| `client` HUD | Sticky Host chrome from Snapshot on join; flash once on first Snapshot Host line |
| `docs/protocol.md` | Snapshot field + mid-join note |

## Wire

Snapshot adds:

```json
"host_line": "HOST: CONTESTED FREQUENCY. LEAGUE DENIES EXISTENCE. ARENA DUEL IS LIVE."
```

While `pressure` is `"compliance"`:

```json
"host_line": "HOST: CONTINUANCE COMPLIANCE PING. APPROVED LANES ONLY."
```

`round_start.host_line` unchanged. Old clients ignore the new Snapshot field; old Snapshot without `host_line` deserializes via default.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
```

No tool attribution, emoji, or em/en dashes in commits/PR/docs.

## Success

- PR open; mid-join Snapshot carries current `host_line`
- Client shows sticky Host chrome on join without waiting for RoundStart
- Mid-join during compliance sees the compliance Host line
- CI-ready (fmt, clippy -D, tests, unfiltered cov >= 80)
