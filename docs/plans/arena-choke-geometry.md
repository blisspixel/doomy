# Plan: Arena choke geometry

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/arena-chokes`
**Spend:** $0. Loopback. No GCP apply, no ElevenLabs, no look_at reopen.
**Status:** In flight (PR #72). Health pads shipped (#71); this is the NOW tip.

## Goal

Raise Quake/Unreal fight quality with **authoritative scrap choke geometry**: low walls, crate clusters, and pillar cover that break long sightlines and create mid-lane fights. Server collision matches Godot props. Pickups, Solo Scrap, and Compliance Drone stay valid.

Product locks: Contested Frequency; Unreal+CS + pixel grit (bone/gunmetal/rust/blood/ember, not neon); port 6767; fail-under 80 unfiltered; look_at/hit HOLD.

## Non-goals

- New second map file (unless a tiny shared prop scene)
- GCP apply / Terraform apply
- ElevenLabs / paid voice
- look_at / hit confirm reopen (HOLD)
- Protocol or wire shape changes
- Navmesh / pathfinding rewrite
- Photoreal scrap piles or neon props

## Research (tip `823eb72`)

| Seam | Tip today | Gap |
|---|---|---|
| Godot arena | Outer walls + zones + 4 pillars + 3 crates + 2 low walls (N/E only) | Incomplete chokes; S/W missing; sparse cover |
| Server move | Arena bounds clamp only | Players walk through scrap props |
| Hitscan | Angle/spread only | Shots ignore cover |
| Spawns | Circle radius 15 | Can land inside solid props |
| Pads | Weapon/health/armor coords fixed | Must stay claimable with free approach |
| Drone | Spawns at (0, 0) center | Must remain clear |
| Protocol | Unchanged pickups/events | No wire change needed |

## Architecture impact

| Area | Change |
|---|---|
| `server` sim | Static AABB2 scrap solids; slide move resolve; hitscan blocked by cover; spawn/respawn push-out of solids |
| `client` arena.tscn | Symmetric low walls (N/S/E/W); denser crate clusters; same scrap materials |
| `docs` | This plan; tip priorities (health shipped, chokes NOW); optional tip still note |
| Protocol / MCP | None |

## Layout (authoritative solids)

Heights are presentation only. Sim is XZ AABBs inflated by `PLAYER_RADIUS` for move; raw AABBs for hitscan.

| id | kind | center (x,z) | half (x,z) | notes |
|---|---|---|---|---|
| pillar_ne/nw/se/sw | pillar | (±7, ±7) | 1.25, 1.25 | Existing visual |
| low_n | low wall | (0, -10) | 4.0, 0.4 | Existing visual |
| low_s | low wall | (0, 10) | 4.0, 0.4 | New (symmetry) |
| low_e | low wall | (10, 0) | 0.4, 4.0 | Existing visual |
| low_w | low wall | (-10, 0) | 0.4, 4.0 | New (symmetry) |
| crates | crate | see table | 1.0, 1.0 | Clusters + existing |

Crate centers (must not overlap pad keepouts):

| x | z | role |
|---|---|---|
| 4 | 16 | existing |
| -15 | 3 | existing |
| 16 | -4 | existing |
| -3.5 | -14 | north flank |
| 3.5 | -14 | north flank |
| -3.5 | 14 | south flank |
| 3.5 | 14 | south flank |
| -14 | -5 | west flank |
| -14 | 5 | west flank |
| 14 | 5 | east flank |
| 14 | -5 | east flank |

Pad keepouts (claim radius 1.75; leave approach lanes):

| pad | x | z |
|---|---|---|
| pad_rail | 12 | 12 |
| pad_scatter | -12 | -12 |
| pad_flechette | -12 | 12 |
| pad_health_n | 0 | 8 |
| pad_health_s | 0 | -8 |
| pad_armor | 8 | 0 |

Center (0,0) stays open for Compliance Drone. Low walls sit outside health pads (±10 vs ±8) so pads remain claimable from the hub; flank around wall ends.

## Beat

1. Document plan + tip priorities (health shipped; chokes NOW).
2. Add `arena_obstacles()` AABB list matching Godot.
3. Slide-resolve move against inflated AABBs; keep outer bounds clamp.
4. Hitscan: if obstacle intersects ray before target, miss (cover works).
5. Spawn/respawn: if point in solid, sample alternate angles on the spawn ring.
6. Godot: add LowWallSouth/West + crate clusters; scrap materials unchanged.
7. Tests: block move into pillar; pad claim still works; hitscan blocked by wall; drone spawn at center; coverage fail-under 80.
8. Tip stills if capture script works; else note follow-up in plan.

## Protocol

No change. Obstacles are sim-local; clients already render props.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
```

Play: Solo Scrap, strafe into a low wall (stop), hide behind pillar from a bot shot, walk onto health pad from hub, wait for Compliance Drone at center.

## Spend / safety

$0. No secrets. No attribution, emoji, or em/en dashes in commits, PR text, or docs.

## Success criteria

- [x] Plan in tree; tip priorities list chokes as NOW (health pads shipped)
- [x] Server solids match Godot chokes; move + hitscan respect cover
- [x] Pads reachable; drone spawn OK; Solo Scrap unchanged on wire
- [x] Godot arena has clear N/S/E/W scrap chokes + crate clusters
- [x] Tests + unfiltered fail-under 80
- [x] PR open on `cursor/arena-chokes` (#72)

## Tip stills

Regenerated live tip captures under `docs/screenshots/` (01/02/07/08/09) after choke geometry landed.
