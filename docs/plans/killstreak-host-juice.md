# Killstreak / multi-kill Host juice

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/killstreak-host-juice`
**Spend:** $0. No ElevenLabs, no GCP apply, no renet, no look_at/hit reopen.
**Status:** Plan of record for killstreak Host callouts + HUD flash.

## Goal

Raise scrap drama: per-player killstreak / multi-kill Host callouts plus a brief HUD flash so spectator and join feel Quake/Unreal arena energy under Contested Frequency voice (Continuance parody, not Doom IP).

## Tip priorities

Human join FP juice shipped (#74 / `094f941`). **Killstreak Host juice is the NOW tip.** look_at / hit HOLD. Port 6767. Coverage fail-under 80.

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

## Architecture impact

| Area | Change |
|---|---|
| `server` protocol | `GameEvent::Killstreak` + Host line helpers for tiers 2 / 3 / 5 |
| `server` sim | Per-player `killstreak` within round; emit at 2/3/5; reset on death and round start/end |
| `agent-adapter` | Mirror Killstreak event so observe / get_events see streak on the wire |
| `client` HUD | Brief streak flash + Host bumper when `killstreak` fires |
| `client` game_manager | Route `killstreak` events to HUD |
| `docs/protocol.md` | Document `killstreak` event shape + enum bullet |
| `docs/plans/README.md` | Tip priorities: FP juice shipped; this NOW |

## Wire

```json
{
  "type": "event",
  "event": "killstreak",
  "player": "Rusher",
  "player_id": "550e8400-e29b-41d4-a716-446655440000",
  "streak": 2,
  "tier": "double",
  "message": "HOST: DOUBLE FREQUENCY. Rusher DENIES THE DENIAL."
}
```

Tiers (Contested Frequency wording, Quake/Unreal energy without Doom IP):

| Streak | Tier | Host line pattern |
|---|---|---|
| 2 | `double` | `HOST: DOUBLE FREQUENCY. {name} DENIES THE DENIAL.` |
| 3 | `triple` | `HOST: TRIPLE SCRAP. CONTINUANCE LOSES COUNT.` |
| 5 | `rampage` | `HOST: FREQUENCY RAMPAGE. {name} BREAKS EVERY APPROVED LANE.` |

Only those thresholds emit. Streak 4 is silent (count keeps climbing toward 5). Streaks above 5 do not re-fire unless reset then rebuilt.

## Behavior

1. On frag: killer `killstreak += 1`; victim streak resets to 0. If killer streak is 2, 3, or 5, emit `killstreak` after the `frag` event.
2. Boss frags count toward streak (same as scrap frags).
3. Round start and round end clear every player streak.
4. Godot: `killstreak` shows Host bumper via RoundMessage plus a brief ember streak flash (same grit as spawn/damage flashes). Sticky Snapshot `host_line` is not overwritten by streak callouts.
5. MCP observe / get_events buffer the event like other GameEvents.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
# Optional Godot import
godot --headless --path client --import --quit-after 120
# Playable: server --bots 4, spectate or Join; watch Host bumper on 2/3/5 streaks
```

## How to trigger streaks

1. `cargo run -p fragr-server -- --bots 4` (port 6767).
2. Open Godot client, spectate or Join.
3. Wait for a fighter (or you) to land 2, then 3, then 5 frags without dying in the same round. Host bumper + streak flash fire at those counts.
4. Dying or round end resets the counter; MCP `get_events` / observe `recent_events` show `event: killstreak`.

## Success criteria

- Plan in tree; tip priorities list this as NOW (FP juice shipped)
- Multi-kills sell on tip face (Host bumper + flash)
- Protocol + adapter see streak on the wire
- Rust tests green; coverage fail-under 80
- PR open on `cursor/killstreak-host-juice`

## Spend and safety

- $0 local only
- No cloud apply, no paid APIs
- No tool attribution, emoji, or em/en dashes
