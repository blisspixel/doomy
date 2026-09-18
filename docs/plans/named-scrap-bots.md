# Plan: Named scrap bot personalities

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/named-scrap-bots`
**Spend:** $0. Loopback. No GCP apply, no ElevenLabs, no look_at/hit reopen.
**Status:** Shipped (#82 / `f58b2d0`).

## Goal

Raise spectator/join fun with Contested Frequency scrap-league rule-bot names (not generic Bot1 / role labels). Sticky behavior chips stay. Host-aware roster intros on Warmup / mid-join. Tiny soft: MCP `round_state` surfaces `map_id` / `map_name`.

Product locks: Contested Frequency parody; port 6767; fail-under 80 unfiltered; look_at/hit HOLD; no GCP apply; maps 1+2 stay.

## Tip priorities

Shipped. Tip NOW: rule-bot Contested Frequency taunts. See [`rule-bot-taunts.md`](./rule-bot-taunts.md).

## Non-goals

- GCP apply / Terraform apply
- ElevenLabs / paid voice
- look_at / hit confirm reopen (HOLD)
- L5 AGI / LLM bots on tick
- New map / choke layout (maps 1+2 stay)
- Neon HUD flood or Doom branding

## Research (tip `8c4f02c`)

| Seam | Tip today | Gap |
|---|---|---|
| Rule-bot names | Rusher / Sniper / Flanker / Tank / ... | Role labels, not scrap-league callsigns |
| Behavior chips | Snapshot `behavior` + Godot label/scoreboard chips | Keep; do not remove |
| Host chrome | Sticky host_line (league / compliance / MVP) | No roster intro naming dialed-in scrap bots |
| Tip face | Labels show `player_name` + chip | Colors keyed to old role names |
| MCP `round_state` | mode / playlist / mvp / host | Missing `map_id` / `map_name` (Testy soft from Yard) |

## Architecture impact

| Area | Change |
|---|---|
| `server` session | Contested Frequency callsign roster (8) assigned on spawn / min_bots refill |
| `server` protocol | `bot_intro_host_line` / `roster_host_line` helpers |
| `server` sim | Sticky `roster_host_line` for Warmup / mid-join; RoundStart can use it |
| `client` | BOT_COLORS / Cyanex-Kragge lists keyed to new callsigns; labels already show names |
| `agent-adapter` | `round_state` includes `map_id` / `map_name` from last snapshot |
| `docs` | This plan; tip priorities; README / protocol notes |

## Roster (Contested Frequency callsigns)

Behavior chips stay Aggressive / Defensive / Flanker / Balanced:

| Callsign | Behavior | Faction flavor |
|---|---|---|
| Dead Air Dan | Aggressive | Host booth grit |
| Nightfall | Defensive | Night Watch rail |
| Static Kid | Flanker | Static Kids push |
| Aunt Linda | Balanced | Radio callsign |
| Scout Ant | Flanker | Art-bible callsign |
| Crackpot | Defensive | Conspiracy seasoning |
| Buzzkill | Aggressive | Close scrap |
| Tin Foil Tina | Balanced | Hangar Candy lane |

Overflow refill suffixes: `Dead Air Dan-2`, etc. (same pattern as tip).

## Host-aware intros

- After rule-bot spawn / refill, refresh sticky `roster_host_line` from current rule-bot display names.
- Snapshot during Warmup (no pressure / boss / Ended) prefers roster Host line so mid-join reads who dialed in.
- RoundStart `host_line` uses roster line when present.
- Optional: empty arena clears roster sticky back to None (default league Host line).

## Soft: map on round_state

MCP `round_state` result adds `map_id` and `map_name` from the last Snapshot (defaults 1 / Arena Duel when no snap). Observe already carries them on Snapshot.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
```

No `--ignore-filename-regex`. No tool attribution, emoji, or em/en dashes in commits/PR/docs.

## Success

- PR open with scrap-league named rule bots.
- Tip face labels read callsigns + sticky behavior chips.
- Warmup / mid-join Host line names the roster.
- MCP `round_state` carries map_id / map_name.
- CI-ready (fmt, clippy -D, tests, unfiltered cov >= 80).
