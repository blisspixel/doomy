# Plan: Mid-arena health (and armor) pads

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/health-pads`
**Spend:** $0. Loopback. No GCP apply, no ElevenLabs, no look_at reopen.
**Status:** In flight. Weapon pickups shipped (#70); this is the NOW tip.

## Goal

Raise Quake scrap chase with **authoritative mid-arena health pads** (and optional light armor scrap) alongside existing weapon pickups. Race the floor for medkits mid-fight, claim on touch, wait the respawn timer.

Product locks: Contested Frequency; Unreal+CS + pixel grit; port 6767; fail-under 80 unfiltered; look_at/hit HOLD; weapon pickups already on tip.

## Non-goals

- GCP apply / Terraform apply
- ElevenLabs / paid voice
- look_at / hit confirm reopen (HOLD)
- Full unlock tree / progression inventory
- New map geometry
- Breaking existing weapon pads

## Research (tip `404381a`)

| Seam | Tip today | Gap |
|---|---|---|
| Pickups | 3 weapon pads (Rail / Scatter / Flechette) | No health / armor chase |
| Snapshot | pickups with weapon id | Need kind + amount; player armor |
| Events | pickup with weapon | Need kind distinguish health vs weapon |
| Player | hp only | Simple armor field (absorb damage) |
| Godot | scrap weapon crate billboards | Distinct medkit / armor scrap look |
| MCP observe | mirrors pickups | Health pads visible; event kind |
| Solo Scrap + MP | same GameState tick | Pads on shared sim |

## Architecture impact

| Area | Change |
|---|---|
| `server` protocol | PickupState `kind` + optional `amount`; Pickup event kind; PlayerState `armor` |
| `server` sim | 2 health pads (+40 HP, cap max) + 1 armor scrap (+25); claim; respawn ~15s; armor absorb |
| `agent-adapter` | Mirror kind/amount/armor; observe description |
| `client` | Distinct scrap medkit billboards (blood/ember, not neon) |
| `docs` | This plan; tip priorities; protocol.md |

## Beat

1. Two mid-arena health pads: N `(0, 0.4, 8)` and S `(0, 0.4, -8)`, +40 HP capped at max.
2. One armor scrap at E `(8, 0.4, 0)`, +25 armor (cap 100), simple absorb-before-HP.
3. Snapshot `pickups` carries `kind` (`weapon` / `health` / `armor`) and optional `amount`.
4. Living player within claim radius of an available pad claims it when useful (health only if damaged; armor only if under cap). Pad unavailable; respawn ~15s; `pickup` event with `kind`.
5. Weapon pads unchanged (still claim + swap; respawn ~12s).
6. Works Solo Scrap + MP. Godot renders distinct medkit/armor scrap from snapshot.

## Protocol (summary)

```json
"pickups": [
  {"id":"pad_rail","kind":"weapon","weapon":"Rail","x":12.0,"y":0.4,"z":12.0,"available":true},
  {"id":"pad_health_n","kind":"health","amount":40,"x":0.0,"y":0.4,"z":8.0,"available":true},
  {"id":"pad_armor","kind":"armor","amount":25,"x":8.0,"y":0.4,"z":0.0,"available":false,"respawn_in":100}
]
```

```json
{"type":"event","event":"pickup","player":"Rusher","player_id":"...","kind":"health","amount":40,"pickup_id":"pad_health_n"}
```

PlayerState gains `armor` (default 0).

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
```

Play: Solo Scrap, take damage, walk onto mid health pad, watch HP climb and pad vanish until respawn. Weapon pads still swap.

## Spend / safety

$0. No secrets. No attribution, emoji, or em/en dashes in commits, PR text, or docs.

## Success criteria

- [x] Plan in tree; tip priorities list health pads as NOW (weapon pickups shipped)
- [x] Authoritative health/armor pads claim on touch; Snapshot + pickup event with kind
- [x] Godot scrap medkits readable (blood/ember accents, not neon)
- [x] MCP observe sees health pads; event distinguishes weapon vs health
- [x] Tests + unfiltered fail-under 80
- [ ] PR open on `cursor/health-pads`

## Pad positions

| id | kind | amount | x | y | z |
|---|---|---|---|---|---|
| pad_rail | weapon | - | 12 | 0.4 | 12 |
| pad_scatter | weapon | - | -12 | 0.4 | -12 |
| pad_flechette | weapon | - | -12 | 0.4 | 12 |
| pad_health_n | health | 40 | 0 | 0.4 | 8 |
| pad_health_s | health | 40 | 0 | 0.4 | -8 |
| pad_armor | armor | 25 | 8 | 0.4 | 0 |
