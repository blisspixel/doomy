# Plan: Continuance SP boss beat (Compliance Drone)

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/continuance-sp-boss`
**Spend:** $0. Loopback. No GCP apply, no ElevenLabs, no look_at reopen.
**Status:** In flight. Solo Scrap shipped; this is the NOW tip.

## Goal

Raise solo scrap amazement with **one** Continuance boss beat in Contested Frequency: a mid-round **Compliance Drone** NPC that is observe-visible, killable, Host-called, and distinct from deathmatch rule bots and the existing compliance slow ping.

Product locks: Office of Global Continuance parody fiction; Host easter eggs OK; game first; Unreal+CS + pixel grit; port 6767; LLM off tick; fail-under 80 unfiltered; look_at/hit HOLD.

## Non-goals

- GCP apply / Terraform apply
- ElevenLabs / paid voice
- look_at / hit confirm reopen (HOLD)
- Full campaign / progression maps
- Article curse as a second simultaneous beat (drone is the shippable cut; curse remains lore seasoning)
- Separate SP sim or different combat rules

## Research (tip `685fdcd`)

| Seam | Tip today | Gap |
|---|---|---|
| Continuance pressure | Mid-round `compliance_ping` + half-speed lanes | No killable Continuance actor |
| Solo Scrap | `min_bots` / `ensure_min_bots` + boot menu | Needs a boss that does not break min_bots refill |
| Rule bots | Rusher/Sniper/Flanker/Tank/... Aggressive/Defensive/... | No Compliance silhouette or AI |
| MCP | observe / get_events / round_state / speak | No `boss_spawn` / `boss_down` |
| Godot | BOT_COLORS per named bot; pressure chip for compliance | No drone chrome |

## Architecture impact

| Area | Change |
|---|---|
| `server` protocol | `boss_spawn` / `boss_down` events; `boss_host_line()`; Snapshot `pressure` may be `compliance_drone` while drone is alive |
| `server` sim | Mid-Active spawn of COMPLIANCE-DRONE (200 HP, Rail, taller y, `BotBehavior::Compliance`); no respawn; Host callouts; owned on `GameState` only so `min_bots` stays clean |
| `server` session | Drive AI for state-only bots (boss) in `tick_messages` without counting them toward `min_bots` |
| `agent-adapter` | Mirror `BossSpawn` / `BossDown` on GameEvent |
| `client` | Drone color/scale/behavior chip; HUD Host bumper on spawn/down; pressure chip for `compliance_drone` |
| `docs` | This plan; tip priorities; protocol.md event shapes |

## Beat (ONE)

**Mid-round Continuance Compliance Drone.**

1. Fires once per Active round at configured tick (default ~20s into Active, after the ~15s compliance ping).
2. Spawns `COMPLIANCE-DRONE` at arena center (taller y), Rail, 200 HP, `behavior: Compliance`.
3. Emits `boss_spawn` with Host message. Snapshot `pressure` = `"compliance_drone"` while alive; sticky Host line switches to the drone line.
4. Distinct AI: orbit / hold mid range, precise Rail poke (not a Rusher clone).
5. On kill: frag + `boss_down` Host callout; drone does not respawn; pressure clears (or falls back to compliance slow if still ticking).
6. Solo Scrap: rule bots and `min_bots` unchanged; drone is extra Continuance teeth.

## Protocol (summary)

- `boss_spawn`: `name`, `boss_id`, `message`, `hp`
- `boss_down`: `name`, `boss_id`, `killer` (optional), `message`
- Snapshot `pressure`: `"compliance_drone"` while drone alive (takes priority over `"compliance"`)

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
```

Trigger: Solo Scrap or `cargo run -p fragr-server -- --bots 4`, start Active round, wait ~20s (or set `boss_spawn_ticks` in tests). Observe `boss_spawn` / drone in snapshot; frag it for `boss_down`.

## Spend / safety

$0. No secrets. No attribution, emoji, or em/en dashes in commits, PR text, or docs.

## Success criteria

- [x] Plan in tree; tip priorities list Continuance SP boss as NOW (solo shipped)
- [x] One killable Continuance drone beat in Solo Scrap path
- [x] `boss_spawn` / `boss_down` on wire + MCP-readable
- [x] Godot shows distinct silhouette / Host callout
- [x] Tests + coverage fail-under 80 unfiltered
- [ ] PR open on `cursor/continuance-sp-boss`

## Tip priorities (this slice)

1. **Continuance SP boss** (this plan) - Compliance Drone beat for Solo Scrap amazement
2. HOLD: look_at / hit, ElevenLabs, GCP apply, full campaign
