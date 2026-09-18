# Mode fantasy: Contested Frequency

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/mode-fantasy-contested-frequency`
**Spend:** $0. No ElevenLabs, no GCP apply, no renet.
**Status:** Plan of record for shipping one readable named mode fantasy.

## Goal

Ship **Contested Frequency** as a named scrap-league mode humans, agents, and spectators feel in one round. Not only frag radio or an MCP cell farm. Game first; parody fiction as seasoning.

Product locks:

- Contested Frequency = sanctioned scrap league that denies it exists. Host never hung up.
- Playlist under the lie: **Arena Duel** (same Action path, SP + MP).
- Antagonist: Office of Global Continuance (parody). Creed wall lines OK as Host easter eggs.
- Unreal+CS feel + Doom-sprite grit (feeling, not IP). Port 6767. Live laugh frag.
- LLM off tick.

## Non-goals

- ElevenLabs / paid voice
- GCP apply
- renet / UDP rewrite
- L5
- Reopening round_* Casino items
- Off-tick speak / taunt (next tip after this)

## Architecture impact

| Area | Change |
|---|---|
| `server` protocol | Snapshot + Welcome carry `mode_name` / `playlist`; optional `pressure`; new `compliance_ping` event; RoundStart carries mode + Host line |
| `server` sim | Mid-round Continuance compliance ping once per Active round; brief move slow while pressure is live |
| `agent-adapter` | Mirror protocol fields so observe/events stay in sync |
| `client` HUD | Named league identity, scrap-league scoreboard/killfeed, Host bumpers + compliance banner |
| `docs/protocol.md` | Wire shapes for mode + pressure beat |

## Pressure beat (ONE)

**Mid-round Host compliance ping.**

- Fires once per Active round at a configured tick (default ~15s into Active).
- Emits `compliance_ping` with Host message and duration.
- While active, all fighters move at half speed (approved lanes only). Snapshot `pressure` = `"compliance"`.
- HUD shows a Host bumper for the duration. Parody Continuance fiction, not a manifesto.

## Protocol (summary)

- `Welcome`: `mode_name`, `playlist` (defaults for old clients).
- `Snapshot`: `mode_name`, `playlist`, optional `pressure`, sticky `host_line` (see sticky-host-line-mid-join).
- `round_start`: `mode_name`, `playlist`, `host_line`.
- `compliance_ping`: `message`, `duration_ticks`.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
```

No `--ignore-filename-regex`. No tool attribution, emoji, or em/en dashes in commits/PR/docs.

## Success

- PR open with Contested Frequency named on wire + HUD.
- One compliance pressure beat changes round feel.
- One round of play reads as scrap league, not bare frag radio.
- CI-ready quality bar (fmt, clippy -D, tests, unfiltered cov >= 80).
