# Plan: maps that are actually maps

**Status:** spec (2026-09-19)
**Branch:** `feat/map-scale-*` (one PR per rung)
**Spend:** $0.

## The problem

Both current arenas are one flat square room. Arena Duel is fifty units on a side with a floor at a single height, pillars, and scattered crates. That is a test chamber, not a level. It is the right shape for measuring a weapon table and the wrong shape for playing.

`massive-arenas.md` owns how many fighters the server can carry. `arena-choke-geometry.md` owns cover inside a room. Neither owns how big a level is or what shape it has, which is this.

## The references, and what each one is for

- **Doom.** Interconnected rooms on a readable floor plan, doors and keys shaping the route, height used for drama and for sniping, and a player who learns the map as a place rather than an arena. This is the shape most fragr maps should be.
- **Unreal Tournament.** Verticality that matters: lifts, ledges, and a fight that moves up and down as much as across. Pickups placed so that controlling a level means controlling a route, not a room.
- **Halo.** Outdoor space joined to interior space, sightlines long enough for a rifle and broken enough to cross, and vehicles-worth of room even without vehicles.
- **Battlefield 1942.** The large end. Distances measured in hundreds of metres, several objectives, a fight that is several fights at once. Some fragr maps should be this size, not all of them.

A pit or a small duel room is fine as one map among many. The mistake is every map being one.

## The size ladder

One fragr unit is one metre. Current maps sit at the bottom rung and nothing else exists.

| Tier | Playable extent | Fighters | What it is for |
|---|---|---|---|
| Pit | 30 to 50 m | 2 to 6 | Duels, the weapon-table harness, a tutorial room |
| Arena | 80 to 120 m | 6 to 16 | The default competitive map, Doom and Unreal shaped |
| District | 200 to 300 m | 16 to 32 | Halo shaped, indoor joined to outdoor, several routes |
| Field | 600 to 1000 m | 32 to 64 | The 1942 end: several objectives, several simultaneous fights |

The rungs below build one map at each tier, because a tier with no map is a number in a table.

## What has to change to get there

1. **The arena is a square with a half extent.** `MapKind::half_extent` and a bounds clamp is the whole model. A district or a field needs a floor plan: rooms, connections, and heights.
2. **There is no height.** Fighters move in the XZ plane and `y` is a constant. Lifts, ledges, and a pit all need the movement step to carry a vertical axis, which is a change to the shared step in `server/src/movement.rs` and its GDScript twin, with new golden vectors.
3. **Cover is a list of boxes.** That is enough for a room and not for a building. It stays the collision primitive; what changes is how many there are and that they have tops worth standing on.
4. **Everything is sent to everyone every tick.** A field-tier map is exactly the case `massive-arenas.md` was written for, so that plan's interest management and delta snapshots are a prerequisite for the top two tiers rather than an optimisation.
5. **Agents read the map from `MapInfo`.** A floor plan with heights is more than a solids list, so that message grows with the geometry, and the reference agents' line-of-sight test grows with it.

## Rungs

1. **Height in the shared step.** A vertical axis in `movement.rs` and `movement.gd`, gravity, a step-up height, and new committed golden vectors. Nothing in the maps changes yet; this is the plumbing every later rung needs.
2. **An arena-tier map.** Eighty to a hundred metres, three connected spaces, two heights joined by a ramp and a lift, pickups placed on routes rather than in corners. Measured against the current map with the playtest harness: kill distances should spread out, and the rail should finally have a reason to exist.
3. **A district-tier map.** Interior joined to exterior, a long sightline with cover to cross it, and the map sent to agents as a floor plan rather than a box list.
4. **Interest management,** from `massive-arenas.md`, because the next tier does not fit on the wire without it.
5. **A field-tier map** with several objectives, and a mode that uses them.

## Verification

- The harness reports kill distance buckets per map. A map that plays at one range is a room with extra steps.
- The benchmark runs each tier at its fighter count and reports tick time and bytes per snapshot.
- The tour photographs each map from three fixed poses, so a level that reads as a grey box shows up.

## Related

- `massive-arenas.md`: the wire and the simulation at these sizes.
- `arena-choke-geometry.md`: cover within a space.
- `buttery-controls.md`: the movement step that rung 1 changes.
- `look-pass-boomer.md`: how the surfaces are dressed.
