# Round-end MVP / podium Host drama

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/round-end-mvp`
**Spend:** $0. No ElevenLabs, no GCP apply, no renet, no look_at/hit reopen.
**Status:** Shipped (#79 / `986af60`).

## Goal

Raise sticky spectator/join fun: round-end MVP / podium Host drama (leader name, frag count, Contested Frequency Host bumper) so `round_end` sells like a scrap league, not a silent reset.

## Tip priorities

Shipped. Tip NOW: Ended linger + MVP rehydrate. See [`ended-linger-mvp-rehydrate.md`](./ended-linger-mvp-rehydrate.md). look_at / hit HOLD. Port 6767. Coverage fail-under 80.

## Non-goals

- GCP apply
- ElevenLabs
- look_at / hit reopen (HOLD)
- New map / choke layout
- Neon HUD flood or Doom branding

## Locks

- Contested Frequency / Continuance parody Host voice
- Unreal+CS readability + pixel grit
- Port 6767
- Coverage fail-under 80 (unfiltered)
- look_at / hit HOLD
- No GCP apply

## Architecture impact

| Area | Change |
|---|---|
| `server` protocol | `mvp_host_line` helper; `GameEvent::RoundEnd` gains `mvp`, `mvp_frags`, `host_line` |
| `server` sim | On `end_round`, MVP = top score / frags; emit Host bumper; sticky `ended_host_line` on Snapshot while Ended |
| `agent-adapter` | Mirror RoundEnd MVP fields; `round_state` / observe surface mvp via last_round_end + sticky host_line |
| `client` HUD | Round-end bumper + brief podium flash with MVP name and top frags |
| `client` game_manager | Pass mvp / mvp_frags / host_line / final_scores into HUD |
| `docs/protocol.md` | Document round_end MVP fields |
| `docs/plans/README.md` | Tip priorities: coverage climb shipped; this NOW |

## Wire

```json
{
  "type": "event",
  "event": "round_end",
  "winner": "Rusher",
  "reason": "Frag limit reached",
  "final_scores": [
    {"name": "Rusher", "score": 10},
    {"name": "Anchor", "score": 7},
    {"name": "Ghost", "score": 3}
  ],
  "winner_score": 10,
  "mvp": "Rusher",
  "mvp_frags": 10,
  "host_line": "HOST: ROUND MVP. Rusher WITH 10 FRAGS. CONTINUANCE DENIES THE PODIUM."
}
```

MVP is the top scorer (same selection as `winner`). `mvp` / `mvp_frags` are explicit for agents and podium chrome. Empty lobby: `mvp` null, Host line notes no MVP.

## Behavior

1. On `end_round`: sort scores, pick MVP (max frags), build Contested Frequency Host line, push `RoundEnd` with mvp fields + host_line.
2. While `RoundState::Ended`, Snapshot sticky `host_line` is the MVP bumper (mid-join / observe see it).
3. Round start clears ended Host sticky and restores league default Host line.
4. Godot: `round_end` shows Host bumper (message), MVP name + frag count, brief podium lines from top of `final_scores`, ember flash (same grit as killstreak).
5. MCP: `observe` / `get_events` buffer the event; `round_state` exposes `last_round_end` (mvp fields) and sticky Snapshot `host_line`.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
# Optional Godot import
godot --headless --path client --import --quit-after 120
# Playable: server --bots 4, spectate; wait for frag/time limit; Host bumper sells MVP
```

## How to see MVP

1. `cargo run -p fragr-server -- --bots 4` (port 6767).
2. Open Godot client, spectate or Join.
3. Wait for frag limit or time limit. Round-end Host bumper names the MVP and frag count; brief podium lists top scrap scores.
4. Mid-join during Ended: Snapshot `host_line` still carries the MVP bumper.
5. MCP `get_events` / observe `recent_events` show `event: round_end` with `mvp`, `mvp_frags`, `host_line`. `round_state.last_round_end` mirrors the same.

## Success criteria

- Plan in tree; tip priorities list this as NOW (coverage climb shipped)
- Round_end sells MVP on tip face (Host bumper + podium flash)
- Protocol + adapter see MVP on the wire / observe
- Rust tests green; coverage fail-under 80
- PR open on `cursor/round-end-mvp`; no attribution, emoji, or em/en dashes
