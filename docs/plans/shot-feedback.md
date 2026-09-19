# Plan: make a shot land somewhere

**Status:** spec (2026-09-19)
**Branch:** `feat/shot-fx-*` (one PR per rung)
**Spend:** $0. Placeholder textures are built in code or recoloured from the muzzle flash that already ships.

A hit and a miss look identical in the world. There is no tracer, nothing marks the wall, and nothing sparks off a fighter. The only feedback a player gets is a hit marker on the HUD, which tells them a shot connected but never where a shot went.

That is the same problem the agents had before they were told where the walls are, in reverse. A player cannot see where a miss went, so there is nothing to correct against.

## The wire does not say where a shot ended

`ShotResult` carries shooter, target, a hit flag, damage, and health after. No point, no direction, not even the weapon that fired.

The client cannot work it out either, for three independent reasons:

1. **The dispersion jitter is drawn from the server's private stream.** The client knows the shooter's yaw but not the jittered aim. For the scatter that is a cone eleven degrees wide, so a guessed ray is simply wrong. For the rail at 0.7 degrees it still misses the wall by the better part of a metre at range.
2. **On a miss, nothing computes a stopping point at all.** The cover test is a yes-or-no veto inside the per-target loop. A shot that hits nobody returns nothing, and no code on either side ever asks where it ended.
3. **The client has no collidable world.** The arena scene is mesh instances and lights, with not one collision shape in it. The obstacle truth lives in the server as Rust literals, and duplicating it into GDScript would create a second copy of map geometry to keep in step.

So the server has to say. Seven new fields on the shot result, all defaulted so nothing that reads the wire today breaks: the weapon, the jittered aim, the impact point in world coordinates, the surface normal, and what kind of surface it was.

The impact point is absolute rather than a distance because fires resolve in order within a tick, so a fighter can shoot and then be killed on the same tick, and a dead fighter is absent from the snapshot. An absolute point always renders. The normal comes along because the server's ray-versus-box test already knows which face won and at what sign; it is two floats the server has for free against a duplicated geometry table on the client.

Cost is roughly 110 bytes per shot result. At 64 fighters all firing the fastest weapon that is about 1.8 kB per snapshot, a fifth of a payload already flagged for delta compression.

## The bug this uncovered

The server's yaw is `atan2(dz, dx)`, so a fighter facing yaw shoots along positive X at yaw zero. The client sets rotation directly from that yaw and treats local positive Z as forward. **Those are ninety degrees apart.** They agree at exactly two angles and disagree everywhere else.

Two things corroborate it. The first-person eye nudge pushes the camera along positive Z, which is behind where it should be. And the muzzle sprite is parented at positive X on the pawn, which is where you would put it if you believed positive X were forward.

Nothing renders this today, which is why it has sat there. A tracer renders it immediately.

The order below is deliberate about this: the tracer is drawn first purely from server-space endpoints, which needs no convention at all and is therefore right regardless, and the convention is fixed only once the beam visibly fails to leave the barrel. That proves the mismatch instead of arguing about it, and it is settled by a headless harness that builds a real node and a real camera and asserts their basis vectors against the server's forward vector across a sweep of angles.

While in there, one related thing to check rather than assume: the fix that put fighters back on the floor moved the visuals down by 1.5 and left the pawn root at the server's height. The first-person eye height is still added on top of the root, which would put the eye well above the body it belongs to.

## How the effects are built

**Tracers are one `MultiMeshInstance3D`, not a node per shot.** All live tracers render in a single draw call, with live entries packed at the front of the instance array and swap-removed on expiry. The mesh is a crossed pair of quads built once, so it reads from any camera angle without fighting the billboard system. Per-instance colour gives tint and fade with no shader and no material duplication. Lifetimes are a deadline sweep in `_process`, never a timer per shot: the existing muzzle flash awaits a scene-tree timer per shot, which at 64 fighters firing the fastest weapon is hundreds of timer allocations a second.

**Width has to be computed in screen space.** A three centimetre beam is a sixth of a pixel at spectator distance, and the look pass is about to render the 3D layer at a quarter resolution, which makes it worse. The width comes from a pure function of distance, viewport height and field of view, targeting two pixels, with a floor. It is a static function beside the far-camera scale function that already has a headless test, for the same reason.

**Sparks are pooled billboarded sprites**, configured like the muzzle sprite that already works, with alpha cut so transparency sorting never enters the picture, offset slightly along the surface normal so they do not fight the wall.

**Scorch marks are quads, not decals.** Godot's decal node would project correctly onto arbitrary geometry and is not rendered by the Compatibility renderer, which the look pass commits to switching to. Writing effects that vanish when the look pass lands would be self-inflicted. Every surface in the map is an axis-aligned face, so a quad oriented from the wire normal is correct without projection.

**The simulation is two-dimensional**, so the wire carries no height. Fighter impacts go at the sprite's centre, surface impacts at the shooter's plane. Every scorch at exactly one height would read as a stripe, so they get a small vertical jitter derived from the shot's own identity, computed identically on every client so two spectators see the same wall.

## Everyone sees them, which takes one line

Shot results are currently dropped unless they belong to the local player or the fighter being followed. That filter is right for the HUD, which belongs to one viewpoint, and fatal for world effects, which belong to everyone. Splitting the handler into a world pass with no filter and a HUD pass with the existing one makes tracers and impacts appear for the local player from everyone's shots, and for spectators in every camera mode, because the effects are world nodes rather than HUD overlays.

Seeing where an enemy's misses land is how a player learns where that enemy is. That is the point.

Agents need nothing built. The adapter clones the whole snapshot into `observe`, so the new fields arrive the moment the server emits them. And it matters to them more than it looks: a surface of "cover" on a miss is the same lesson `MapInfo` taught, arriving per shot. An agent that fires into a pillar now learns that it fired into a pillar.

## Budget

At 64 fighters all firing the fastest weapon that is 320 shots a second, about 29 live tracers and 42 live sparks at any instant. Pools are 48 tracers, 64 sparks, 48 scorches, all pre-allocated, stealing the oldest rather than allocating mid-match. Two draw calls for tracers and scorches together; the sparks are the only per-node cost and therefore the smallest pool with the shortest life.

One case breaks the budget: every fighter on the scatter, whose cosmetic fan multiplies streaks sixfold. Two rules live in the allocator rather than in a comment. A cosmetic streak may never evict an authoritative one, and the fan is skipped entirely when free slots run low or the shooter is far from the camera. An authoritative tracer is always drawn; a cosmetic one is always optional.

The scatter fan needs stating plainly in the code: the server rolls one ray, not pellets. The extra streaks are decoration, seeded from the shot's own identity so every viewer sees the same fan, ending in air, never spawning an impact, so nothing about them can be mistaken for a hit.

## Rungs

1. **The server says where the shot ended.** Server only. New fields, the ray test extended to report the face, the boundary planes solved, the protocol document updated. Accepted when a shot down a clear lane ends on the arena boundary, a shot at a pillar ends on its near face with the right normal, and the playtest numbers are identical on three seeds, proving the trace changed nothing about who got hit.
2. **A line appears where the shot went.** The tracer pool, the width function, and the handler split so world effects stop being gated by viewpoint. One look for all three weapons, drawn from snapshot position to impact point, no convention involved. Accepted by a headless harness checking the width function against hand-computed values and the pool against a synthetic burst.
3. **The line leaves the barrel.** The yaw convention, settled by a harness that asserts basis vectors across a sweep of angles rather than by a screenshot. The tracer origin moves to the muzzle.
4. **Something happens where it lands.** Sparks, fighter versus surface, and the hit flash driven from the authoritative shot with the shooter's weapon rather than from a health drop, which currently misattributes the weapon and double-counts when two shots land on one tick.
5. **Each weapon reads as itself.** The rail as a beam that persists, the flechette as a travelling streak, the scatter as a fan. Accepted when the tour's strip shows the rail visible for more frames than the scatter, which is the strip's way of saying it reads as a beam.
6. **The wall remembers.** Scorch ring with hold, fade, spacing suppression and a camera cull. Its tour probe is the only one that must be visible for every frame of its strip: a mark that is gone by the next state is not a mark.
7. **Publish the evidence.** The strip reaches the README, and the line in the gunfeel plan that says nothing marks where a shot landed gets closed with the stamp of the run that shows it.

Every rung names a node for the visual QA tour to watch across a firing strip, so each one is accepted by a number in a manifest rather than an opinion. The rung order moves the tracer ahead of the impact, which revises what the gunfeel plan said, because the tracer is the instrument that proves the new wire field is right. A wrong impact point shows up instantly as a beam pointing at nothing; a spark in the wrong place just looks like a spark.

## Related

- `gunfeel.md`: where the gap was recorded, and the feel this serves.
- `look-pass-boomer.md`: the renderer switch that rules decals out and makes the width function necessary.
- `massive-arenas.md`: the wire budget this spends into.
- `ART-ASSET-LIST.md`: the tracer and impact plates, currently placeholders built in code.
