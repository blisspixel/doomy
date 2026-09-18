# Ended linger + mid-join MVP rehydrate

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/ended-linger-mvp`
**Spend:** $0. No ElevenLabs, no GCP apply, no renet, no look_at/hit reopen.
**Status:** Shipped (#81 / `8c4f02c`).

## Goal

1. Linger the Ended phase a bit longer so the podium / Host bumper can be read before the next scrap loads.
2. Mid-join structured MVP rehydrate: while Ended, Snapshot and MCP `round_state` always carry `mvp` / `mvp_frags` / `host_line` so late joiners and observers get podium chrome without catching the live `round_end` event.

## Tip priorities

Shipped (#81). Tip NOW: named scrap bots. See [`named-scrap-bots.md`](./named-scrap-bots.md). look_at / hit HOLD. Port 6767. Coverage fail-under 80. Maps 1+2 stay.

## Non-goals

- GCP apply
- look_at / hit reopen (HOLD)
- New map / choke layout (maps 1+2 stay)
- Neon HUD flood or Doom branding
- Changing MVP selection rules (still top score / frags)

## Locks

- Contested Frequency / Continuance parody Host voice
- Port 6767
- Coverage fail-under 80 (unfiltered)
- look_at / hit HOLD
- No GCP apply
- Maps 1 (Arena Duel) and 2 (Compliance Yard) stay

## Architecture impact

| Area | Change |
|---|---|
| `server` sim | Raise default `end_delay_ticks` (5s -> 8s at 20 Hz). Persist `ended_mvp` / `ended_mvp_frags` with sticky `ended_host_line`; clear on `start_round`. |
| `server` protocol | Snapshot gains optional `mvp` / `mvp_frags` (present while Ended; omitted otherwise). |
| `agent-adapter` | Mirror Snapshot MVP fields. `round_state` prefers Snapshot `mvp` / `mvp_frags` / `host_line`, falls back to `last_round_end`. |
| `client` | Tolerate longer Ended; mid-join Snapshot during Ended rehydrates podium / MVP once via structured fields. |
| `docs/protocol.md` | Document Snapshot MVP fields while Ended. |
| `docs/plans/README.md` | Tip priorities: Yard shipped; this NOW. |

## Wire (Snapshot while Ended)

```json
{
  "type": "snapshot",
  "tick": 9001,
  "round_state": "Ended",
  "mvp": "Rusher",
  "mvp_frags": 10,
  "host_line": "HOST: ROUND MVP. Rusher WITH 10 FRAGS. CONTINUANCE DENIES THE PODIUM.",
  "players": []
}
```

Active / Warmup omit `mvp` / `mvp_frags` (or null). Old clients ignore the new fields. Old Snapshots without them deserialize via default.

## Behavior

1. Default Ended linger: `end_delay_ticks = 20 * 8` (8 seconds). Tests may override shorter.
2. On `end_round`: store MVP name, frag count, and Host line; emit `RoundEnd` as today.
3. While `RoundState::Ended`, every Snapshot carries `mvp`, `mvp_frags`, and sticky MVP `host_line`.
4. `start_round` clears ended MVP sticky state.
5. Godot: if Snapshot says Ended and structured MVP is present, show podium / Host bumper once for mid-join (do not re-flash every tick). Display timers tolerate the longer linger.
6. MCP `round_state`: surface `mvp` / `mvp_frags` from Snapshot when present, else from buffered `last_round_end`.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
# Optional Godot import
godot --headless --path client --import --quit-after 120
```

## Success criteria

- Plan in tree; tip priorities list this as NOW (Yard shipped)
- Ended linger long enough to read podium / Host bumper
- Mid-join Snapshot + `round_state` carry structured `mvp` / `mvp_frags` / `host_line` while Ended
- Godot mid-join during Ended shows podium / MVP
- Rust tests green; coverage fail-under 80
- PR open on `cursor/ended-linger-mvp`; no attribution, emoji, or em/en dashes
