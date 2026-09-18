# Plan: Second Contested Frequency scrap map

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/second-scrap-map`
**Spend:** $0. Loopback. No GCP apply, no ElevenLabs, no look_at/hit reopen.
**Status:** Ready to merge. Tip face: second scrap map (Compliance Yard).

## Goal

Raise sticky fun with a second Contested Frequency scrap map (compact choke yard), selectable via CLI or rotating each round, without breaking Solo Scrap, pads, drone hub, weapon roles, MVP, or killstreak.

Product locks: Contested Frequency; Unreal+CS + pixel grit; port 6767; fail-under 80 unfiltered; look_at/hit HOLD; no GCP apply.

## Tip priorities

MVP / podium Host drama shipped (#79 / `986af60`). **Second scrap map is the NOW tip.**

## Non-goals

- GCP apply / Terraform apply
- ElevenLabs / paid voice
- look_at / hit confirm reopen (HOLD)
- Full campaign / multi-arena lobby UI
- Navmesh rewrite
- Neon props or Doom branding

## Research (tip `986af60`)

| Seam | Tip today | Gap |
|---|---|---|
| Server layout | Single `arena_obstacles()` + fixed pads | No second layout or map select |
| CLI | `--bind`, `--bots` only | No `--map` / rotate |
| Snapshot | mode/playlist/host/pickups | No map id for client switch |
| Godot | One `arena.tscn` | No Compliance Yard face |
| Solo Scrap | `solo_scrap.sh` + boot menu | No map pick / FRAGR_MAP |
| Pads / drone | Fixed coords; hub (0,0) clear | Map 2 must keep claim + hub |

## Architecture impact

| Area | Change |
|---|---|
| `server` sim | `MapKind` (Arena Duel = 1, Compliance Yard = 2); per-map solids, pads, spawn radius; optional round rotate |
| `server` main | `--map` and `--map-rotate` CLI; pass into session |
| `server` protocol | Snapshot `map_id` + `map_name` (serde defaults = map 1) |
| `agent-adapter` | Mirror Snapshot map fields |
| `client` | `arena_compliance_yard.tscn`; switch layout from Snapshot; boot / Solo hint |
| `tools/solo_scrap.sh` | `FRAGR_MAP` / `--map` to server |
| `docs` | This plan; tip priorities; protocol note; README load path |

## Map 2: Compliance Yard

Same 50x50 bounds. Hub (0,0) clear for Compliance Drone. Tighter lanes than Arena Duel.

Solids (XZ AABBs; Godot props mirror):

| kind | centers | half | notes |
|---|---|---|---|
| pillars | (±5, ±5) | 1.0, 1.0 | Inner yard posts |
| N bars | (±7, -8) | 3.5, 0.45 | Gap at x=0 |
| S bars | (±7, 8) | 3.5, 0.45 | Gap at x=0 |
| E bars | (8, ±7) | 0.45, 3.5 | Gap at z=0 |
| W bars | (-8, ±7) | 0.45, 3.5 | Gap at z=0 |
| edge crates | (0, ±16), (±16, 0), (±14, ±14) | 1.0, 1.0 | Outer scrap |

Pads (claim radius 1.75; approach lanes free):

| pad | x | z |
|---|---|---|
| pad_rail | 12 | 12 |
| pad_scatter | -12 | -12 |
| pad_flechette | -12 | 12 |
| pad_health_n | 0 | 11 |
| pad_health_s | 0 | -11 |
| pad_armor | 11 | 0 |

Spawn ring radius: `ARENA_SIZE * 0.25` (tighter than map 1's 0.3).

## Select / rotate

- Default: map 1 (Arena Duel), unchanged feel.
- `--map 1|2|arena|compliance-yard` locks map for the process.
- `--map-rotate` alternates map each `start_round` (keeps current as first round).
- Snapshot carries `map_id` / `map_name` so Godot swaps visuals mid-session.

## Beat

1. Plan + tip priorities (MVP shipped; second map NOW).
2. `MapKind` + solids/pads/spawn; wire into move/hitscan/spawn/pickups.
3. CLI `--map` / `--map-rotate`; session init.
4. Snapshot map fields; adapter mirror; protocol note.
5. Godot Compliance Yard scene + snapshot-driven switch; boot/solo hint; `solo_scrap.sh` FRAGR_MAP.
6. Tests: map 2 choke, pad claim, drone hub clear, rotate, CLI parse; coverage fail-under 80.
7. PR via gh api.

## Protocol

Additive Snapshot fields (defaults preserve old clients):

```json
{
  "type": "snapshot",
  "map_id": 2,
  "map_name": "Compliance Yard",
  "...": "..."
}
```

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
```

Play: `cargo run -p fragr-server -- --bind 127.0.0.1:6767 --bots 4 --map 2` then Solo Scrap / Godot; or `FRAGR_MAP=2 ./tools/solo_scrap.sh`. Confirm pads claim, drone at hub, weapon roles, MVP / killstreak still fire.

## Spend / safety

$0. No secrets. No attribution, emoji, or em/en dashes in commits, PR text, or docs.

## Success criteria

- [x] Plan in tree; tip priorities list second map as NOW (MVP shipped)
- [x] Map 2 playable via `--map 2` (or rotate); default remains map 1
- [x] Godot face matches; Solo Scrap / pads / drone / roles / MVP / killstreak intact
- [x] Tests + coverage fail-under 80; PR open
