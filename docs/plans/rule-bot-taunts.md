# Plan: Rule-bot Contested Frequency taunts

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/rule-bot-taunts`
**Spend:** $0. Loopback. No GCP apply, no ElevenLabs, no look_at/hit reopen.
**Status:** In flight (NOW tip after Warmup Host drama).

## Goal

Raise scrap radio feel: named rule bots occasionally speak Contested Frequency taunts/callouts (off-tick speak path already exists for agents) so spectator killfeed/Host has personality beyond silence.

Product locks: Contested Frequency parody; speak rate-limit honest; port 6767; fail-under 80; look_at/hit HOLD; no GCP apply; no ElevenLabs.

## Tip priorities

Warmup Host drama (`ea9cdcc` / #83) shipped. **This is the NOW tip.**

## Non-goals

- GCP apply / Terraform apply
- ElevenLabs / paid voice
- look_at / hit confirm reopen (HOLD)
- L5 AGI / LLM bots on tick
- New map / choke layout
- Changing SPEAK_COOLDOWN honesty for humans/MCP
- Replacing Host killstreak bumpers (bots speak *in addition*, not instead)

## Research (tip `ea9cdcc`)

| Seam | Tip today | Gap |
|---|---|---|
| Off-tick speak | `try_speak` + SPEAK_COOLDOWN; Godot `show_speak`; MCP events | Humans/MCP/adapters can speak; server rule bots stay silent |
| Adapter scripted-bot | Periodic canned lines via WS Speak | Does not cover `--bots` server rule bots |
| Named roster | Dead Air Dan / Nightfall / ... | Callsigns on labels/Host, no in-fight radio chatter |
| Frag / death / killstreak / Warmup | Events + Host lines | No bot speak hooks on those beats |

## Architecture impact

| Area | Change |
|---|---|
| `server` protocol | Contested Frequency callsign taunt pools + pick helpers |
| `server` sim | After frag/death/killstreak and during Warmup, named rule bots may `try_speak`; respect SPEAK_COOLDOWN; skip Compliance boss |
| `server` session | Tick path unchanged (events already drain Speak) |
| Godot / MCP | Already show speak events; verify only |
| `docs` | This plan; tip priorities; brief protocol note |

## Behavior

1. Named rule bots = `GameState.bots` entries with behavior other than Compliance.
2. On frag: killer may speak a callsign-flavored boast (probabilistic; not every frag).
3. On death: victim may speak a short death callout (lower chance).
4. On killstreak tier 2/3/5: killer prefers a streak taunt over a plain frag taunt (same tick).
5. During Warmup: occasionally one dialed-in rule bot speaks a tuning-in line.
6. All paths go through `try_speak` so SPEAK_COOLDOWN (60 ticks) is honest; RateLimited/Rejected are silent drops for bots (no Error unicast spam).
7. Lines stay under SPEAK_MAX_CHARS; Contested Frequency parody tone; callsign-flavored pools with generic fallback.

## Sample lines (flavor)

- Dead Air Dan (frag): `dead air cleared`
- Nightfall (death): `night watch resets`
- Static Kid (warmup): `tuning the glitch`
- Buzzkill (streak): `approved lanes? nah`
- Tin Foil Tina (frag): `not for public release`

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
```

Behavioral tests:

1. Callsign picker returns callsign-flavored lines for known roster names.
2. Frag/death/killstreak hooks can emit `GameEvent::Speak` for a rule bot.
3. Second speak inside SPEAK_COOLDOWN does not emit.
4. Warmup path can emit a Speak from a dialed-in rule bot.
5. Compliance boss does not get scrap-radio taunts.

No `--ignore-filename-regex`. No tool attribution, emoji, or em/en dashes in commits/PR/docs.

## Success

- PR open from `cursor/rule-bot-taunts` onto main tip post #83.
- Named rule bots talk sometimes on frag/death/Warmup/killstreak.
- Godot/MCP speak path unchanged and still works.
- CI-ready (fmt, clippy -D, tests, unfiltered cov >= 80).
