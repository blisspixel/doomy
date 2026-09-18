# Plan: Mid-map weapon pickups (Quake chase energy)

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/weapon-pickups`
**Spend:** $0. Loopback. No GCP apply, no ElevenLabs, no look_at reopen.
**Status:** In flight. Continuance drone shipped (#69); this is the NOW tip.

## Goal

Raise arena fun with **authoritative mid-map weapon pickups** so Contested Frequency scrap has Quake/Unreal chase energy: race the floor for Rail / Scatter / Flechette pads, claim on touch, wait the respawn timer. Not spawn-with-all-guns forever only.

Product locks: Contested Frequency / Continuance parody; Unreal+CS + pixel grit; port 6767; LLM off tick; fail-under 80 unfiltered; look_at/hit HOLD; Continuance drone already shipped.

## Non-goals

- GCP apply / Terraform apply
- ElevenLabs / paid voice
- look_at / hit confirm reopen (HOLD)
- Full unlock tree / progression inventory
- New map geometry
- Removing Action `weapon_swap` (agents and tests keep it; pads are the chase layer)

## Research (tip `cf7c463`)

| Seam | Tip today | Gap |
|---|---|---|
| Weapons | Flechette / Rail / Scatter via `weapon_swap` always | No floor pads; no mid-fight race |
| Snapshot | players, shot_results, pressure, host_line | No pickup state |
| Events | frag / hit / boss_* / speak / compliance | No pickup claim event |
| Godot | fighter billboards, zone grit | No scrap crate / pad chrome |
| MCP observe | mirrors last Snapshot JSON | Needs pickups array (+ optional event) |
| Solo Scrap + MP | same `GameState` tick | Pads must live on shared sim |

## Architecture impact

| Area | Change |
|---|---|
| `server` protocol | `PickupState` on Snapshot; `pickup` GameEvent |
| `server` sim | 3 fixed pads; touch claim; respawn timer; reset on round start |
| `agent-adapter` | Mirror PickupState + Pickup event; observe sees pickups |
| `client` | Scrap crate / pad billboards (bone-white / gunmetal / ember) |
| `docs` | This plan; tip priorities; protocol.md |

## Beat

1. Three floor pads at fixed arena offsets: Rail (NE), Scatter (SW), Flechette (NW).
2. Snapshot always carries `pickups` (id, weapon, x/y/z, available, optional respawn_in ticks).
3. Living player within claim radius of an available pad claims it: weapon swaps to pad weapon, pad goes unavailable, respawn timer starts (~12s), `pickup` event fires.
4. Works in Solo Scrap and multiplayer (same authoritative tick).
5. Bots lightly steer toward a better available pad when holding Flechette (chase energy without a new map).
6. Godot renders readable pads from snapshot (not neon).

## Protocol (summary)

Snapshot field:

```json
"pickups": [
  {"id":"pad_rail","weapon":"Rail","x":12.0,"y":0.4,"z":12.0,"available":true},
  {"id":"pad_scatter","weapon":"Scatter","x":-12.0,"y":0.4,"z":-12.0,"available":false,"respawn_in":80}
]
```

Event:

```json
{"type":"event","event":"pickup","player":"Rusher","player_id":"...","weapon":"Rail","pickup_id":"pad_rail"}
```

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
```

Play: Solo Scrap or `cargo run -p fragr-server -- --bots 4`, walk onto a pad, watch weapon change and pad vanish until respawn.

## Spend / safety

$0. No secrets. No attribution, emoji, or em/en dashes in commits, PR text, or docs.

## Success criteria

- [x] Plan in tree; tip priorities list weapon pickups as NOW (drone shipped)
- [x] Authoritative pads claim on touch; Snapshot + optional pickup event
- [x] Godot scrap pads readable (bone-white / gunmetal / ember)
- [x] MCP observe sees pickups
- [x] Tests + unfiltered fail-under 80
- [x] PR open on `cursor/weapon-pickups` (#70)
