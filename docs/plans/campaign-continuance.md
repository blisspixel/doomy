# Plan: single-player campaign (Continuance)

**Status:** planned (2026-09-18)
**Branch:** `feat/campaign-*` (one PR per rung below)
**Spend:** $0. Enemy sprites and maps are authored in-repo; Host and enemy voice lines come from the audio pipeline when needed.

## Goal

A single-player campaign at the bar of Doom 1 and Doom 2, in this lore: the Office of Global Continuance has seized the Contested Frequency towers, and you fight through their compliance apparatus to put the station back on the air. Three episodes of eight to nine hand-built maps, an enemy roster where each type is a different problem, keys and secrets, an episode boss, a weapon ladder that grows across the run, four difficulty tiers, and continue-from-last-map. Monsters are server-authoritative entities on the same tick as bots, so agents can play the campaign too, and a spectator can watch a solo run.

## Non-goals

- Cutscenes, lore codices, dialogue trees. The Host frames each map in one line.
- A separate single-player engine. The campaign is a server mode (`--mode campaign --map e1m1`) with the Godot client presenting it.
- A separate co-op engine. Co-op is the same server mode with more seats; it is a rung below, not a non-goal.

## Rungs

1. **Arcade ladder and Survival sweep (first playable).** `--mode arcade`: rounds versus escalating rule-bot rosters with a boss beat every third round, a results card, local best scores saved under `user://`. `--mode survival`: the same engine with no end, the way zombies modes work: the round you fell on is the score, rounds escalate in count and mix, frags earn points that open the next section of the map and buy off the pads, a downed partner can be revived for a few seconds, and the Host counts rounds like a countdown. Both are co-op from the start because the server already seats several fighters. Reuses everything that exists. Evidence: a full ladder run and a survival run to round ten in headless smokes, best round persisted.
2. **Enemy roster.** The canonical list, with each type's tell and its answer, is [`docs/ENEMIES.md`](../ENEMIES.md); this rung implements it. Lands after stage 4 of `plans/buttery-controls.md` so every timing in the tables is authored in seconds, never ticks. Ten Continuance types, each a distinct problem, implemented as server entities with the bot controller seam: Compliance Drone (existing boss, demoted to elite), Clerk (hitscan chip damage, weak), Jammer (slow projectile, blocks radio until killed), Enforcer (rusher, melee), Turret (static, high damage, telegraphed), Auditor (floats, resurrects Clerks), Redactor (invisible until it fires), Walker (armoured, stomp knockback), Drone Swarm (many, fragile), Continuance Walker boss (episode end). Infighting through pain states so enemies can be baited into each other. Evidence: a deterministic sim test per type and an infighting test.
3. **Level format and keys.** Maps authored in TrenchBroom as Quake `.map` files (Valve 220 format). A Rust tool parses them with the `quake-map` crate and emits the server-owned JSON manifest: brush half-spaces for collision (point and capsule tests need no vertex build), point entities for spawns, items, monsters with skill flags, doors with red, gold, and cyan keys, triggers, secrets with a counter, and exit switches. The server loads the manifest and validates it; the client renders the same `.map` through func_godot (MIT, GDScript) using the atlas and the texel constant from stage 2 of `plans/look-pass-boomer.md`. func_godot's Godot 4.7 compatibility is unstated and must be tested first; the fallback is the Rust tool emitting meshes. Evidence: a map validator test, one hand-built map with every element type, one source of truth for geometry.
4. **Episode one.** Eight maps that teach in order (movement, Scatter, keys, Rail, secrets, Turrets, Auditors, boss), built to the Romero rules in `docs/DESIGN-REFERENCES.md`, each with a par time. Evidence: a recorded full-episode run, map-by-map notes in `docs/plans/campaign-e1.md`.
5. **Ladder and tiers.** Weapon pickups unlock across the episode; four difficulty tiers scale monster count, damage, and item scarcity; continue-from-last-map with the loadout carried. Evidence: tests on the tier tables and the save format.
6. **Episodes two and three.** Same bar, new families of maps (Compliance Yard interiors, Perim Ghost outskirts, the tower), the Walker boss and a final broadcast. Evidence: recorded runs.
7. **Co-op through the episodes.** Friends and agents join a party on the same server; keys, doors, and secrets are shared; the results card is per party; the save is per party. Counter-op: one spectator seat possesses monsters in turn, with the fair-play lanes keeping it honest. Evidence: a recorded two-human-plus-two-agent run of episode one.
8. **World map screen.** The Perimeter as a map the party moves across between episodes, remembering what was cleared and what Continuance regrew. Evidence: tour stills and the save format.

## Framework detail (decision-complete)

The campaign is content on top of two frameworks: maps as data and monsters as tables. Both are specified here so the content work (24 maps, ten types) is authoring, not engineering.

### Map manifest (`server/maps/<name>.json`, version 1)

```json
{
  "version": 1,
  "name": "larak-lot",
  "bounds": [-40.0, -40.0, 40.0, 40.0],
  "solids": [
    {"box": [-3.0, -1.0, 3.0, 1.0]},
    {"poly": [[0.0, 0.0], [4.0, 0.0], [4.0, 2.5], [1.0, 3.0]]}
  ],
  "spawns": [{"x": 0.0, "z": -30.0, "yaw": 1.57, "team": "player", "skill": 7}],
  "monsters": [{"type": "clerk", "x": 10.0, "z": 5.0, "yaw": 3.14, "skill": 6, "ambush": true, "group": "lot-east"}],
  "items": [{"kind": "health", "amount": 25, "x": 2.0, "z": 2.0, "skill": 7}, {"kind": "weapon", "weapon": "rail", "x": 8.0, "z": -8.0, "skill": 7}, {"kind": "key", "color": "red", "x": 20.0, "z": 20.0, "skill": 7}],
  "doors": [{"id": "gate-a", "poly": [[10.0, 0.0], [12.0, 0.0], [12.0, 6.0], [10.0, 6.0]], "key": "red", "open_s": 6.0, "stay_open": false}],
  "triggers": [{"id": "t1", "poly": [[...]], "once": true, "on_enter": [{"spawn_group": "lot-east"}, {"open_door": "gate-a"}, {"host": "closet-open"}]}],
  "secrets": [{"id": "s1", "poly": [[...]]}],
  "exit": {"poly": [[...]], "next": "area-kitchen"}
}
```

- `solids` are convex polygons in the XZ plane (boxes are the fast path). Collision: a fighter or monster of radius r is blocked when its centre is inside a solid inflated by r; for boxes that is today's `Aabb2::expand`; for polygons it is the point-in-convex test against each edge offset outward by r, with rounded corners approximated by the edge offsets (a 0.5 unit corner error is invisible at this scale). Hitscan and sight use the same solids with the existing ray-versus-box slab test and a ray-versus-edge test for polygons.
- `skill` is a three-bit mask, easy 1, normal 2, hard 4, as Doom does it, so designers hand-place rosters per tier; the fourth tier (nightmare) uses the hard rosters with the fast flag.
- Everything is validated on load: bounds contain everything, at least one player spawn, exit present unless the map is an arena, door keys exist as items or are reachable from a previous map (checked by the episode manifest), trigger targets resolve, no two solids overlap a spawn.

### The map tool (`tools/mapc`, Rust)

`fragr-mapc build maps/larak-lot.map --out server/maps/larak-lot.json` parses a TrenchBroom Valve 220 `.map` with the `quake-map` crate, keeps brushes whose entity is worldspawn or a `func_*` solid, intersects each brush's planes to a convex hull, projects the hull onto XZ at the player's height band (the slab between floor 0 and eye height 1.6; brushes entirely above or below are decoration), emits a box when the projection is axis-aligned and a polygon otherwise, and maps point entities by classname: `info_player_start` to spawns, `monster_<type>` to monsters (spawnflags carry skill and ambush bits, TrenchBroom's `angle` to yaw), `item_health`, `item_armor`, `weapon_<name>`, `item_key_<color>`, `func_door` (targetname and key), `trigger_once` and `trigger_multiple` (target list), `trigger_secret`, `trigger_exit` (next map). A `.fgd` file in `tools/mapc/fragr.fgd` gives TrenchBroom the entity definitions. `fragr-mapc check` runs the validator. The client renders the same `.map` through func_godot with the look-pass atlas; geometry has one source.

### Monster tables

Monsters are server entities on the same tick as bots. A type is a row:

| Field | Meaning |
|---|---|
| `name` | `clerk`, `enforcer`, `jammer`, `turret`, `auditor`, `redactor`, `walker`, `swarm`, `drone`, `walker_boss` |
| `hp`, `radius`, `speed` | units per second; turrets 0 |
| `pain_chance` | 0 to 1, as Doom's value over 256 |
| `reaction_s` | delay before the first attack after waking (0.25 s baseline, 0 on damage) |
| `sight_arc_deg`, `sight_range` | forward cone for waking on sight |
| `sound_range` | units a fired weapon or a hit alerts through, not through closed doors |
| `melee` | optional `{range, damage, cooldown_s}` |
| `attack` | optional `{kind: hitscan or projectile, damage, cooldown_s, spread_deg, projectile: {speed, radius, homing: bool}}` |
| `attack_bias` | how attack probability rises as distance falls: chance per tick `= clamp((bias - dist) / bias, 0.05, 1.0) * base` (Doom's distance check in one number) |
| `infight_hold_s` | how long a grudge against another monster lasts (Doom's 100 tics is about 3 s) |
| `special` | `resurrect` (auditor), `invisible_until_fire` (redactor), `stomp_knockback` (walker), `jam_radio` (jammer) |
| `fast` | speed and cooldown multipliers under the nightmare tier |

States per monster: `Idle`, `Alert(target, since)`, `Chase(target)`, `Attack(target, wind_up_until)`, `Pain(until)`, `Dead(since)`. Rules, all in seconds:

1. Idle: wake on sight (target inside the arc and range with a clear ray) or on sound (an alert event within `sound_range` whose path crosses no closed door), unless `ambush`, which wakes on sight only.
2. Chase: each step pick one of eight compass directions toward the target, diagonals first, re-pick on a block or every random 0.4 to 1.2 s (the seeded RNG); melee if in range; else attack when the cooldown is ready and the distance roll passes; after an attack, one step of movement before the next attack.
3. Pain: on damage roll `pain_chance`; success interrupts the current state for the pain duration (0.3 s) and resets reaction time to 0 ("awake now").
4. Infighting: damage from a monster of another type retargets the victim to the attacker for `infight_hold_s`; same type never retargets; auditors are never targeted and always retarget. Hitscan grunts can hit each other; projectiles skip their own type.
5. Projectiles are server entities with position, velocity, radius, owner, and damage; they hit the first fighter or monster on their path (grid candidates) or a solid, and vanish.
6. Dead monsters leave a corpse marker in the snapshot for the client's death animation and are removed after 10 s; auditors resurrect corpses within 8 units at full hp.

Each type's row is a test: a deterministic sim test spawns the type against a stationary player and asserts wake, chase, attack timing, and pain behaviour to the tick.

### Encounters, keys, secrets, exit

Triggers are polygons with once or repeat semantics and an action list: spawn a group (closets and teleport traps), open or close a door, play a Host line, alert monsters in a radius. Doors are solids that open on a key (removed from the solids while open) and close after `open_s` unless `stay_open`. Secrets count when first entered and print on the results card. The exit polygon ends the map when a player enters it with no boss alive: results card (time, frags, secrets found, deaths, par time), then the next map from the episode manifest.

### Episode manifest and saves

`server/maps/episodes.json`: episodes as ordered map lists with par times and a boss flag per map. The server keeps `campaign.json` in its data directory: episode, map index, skill, carried weapons, hp and armor at exit, best time per map. `--campaign-continue` loads it; a fresh `--campaign e1` resets. The client mirrors best scores in `user://scores.cfg` for the boot menu.

### Agents in the campaign

Telemetry adds visible monsters by type, range bucket, and whether awake; keys held; a locked door seen; an objective hint (`find_key`, `open_door`, `reach_exit`, `clear_boss`). The brain gets one more question, an objective choice; the reflex layer keeps shooting.

## Framework detail (decision-complete)

The campaign is content on top of two frameworks: maps as data and monsters as tables. Both are specified here so the content work (24 maps, ten types) is authoring, not engineering.

### Map manifest (`server/maps/<name>.json`, version 1)

```json
{
  "version": 1,
  "name": "larak-lot",
  "bounds": [-40.0, -40.0, 40.0, 40.0],
  "solids": [
    {"box": [-3.0, -1.0, 3.0, 1.0]},
    {"poly": [[0.0, 0.0], [4.0, 0.0], [4.0, 2.5], [1.0, 3.0]]}
  ],
  "spawns": [{"x": 0.0, "z": -30.0, "yaw": 1.57, "team": "player", "skill": 7}],
  "monsters": [{"type": "clerk", "x": 10.0, "z": 5.0, "yaw": 3.14, "skill": 6, "ambush": true, "group": "lot-east"}],
  "items": [{"kind": "health", "amount": 25, "x": 2.0, "z": 2.0, "skill": 7}, {"kind": "weapon", "weapon": "rail", "x": 8.0, "z": -8.0, "skill": 7}, {"kind": "key", "color": "red", "x": 20.0, "z": 20.0, "skill": 7}],
  "doors": [{"id": "gate-a", "poly": [[10.0, 0.0], [12.0, 0.0], [12.0, 6.0], [10.0, 6.0]], "key": "red", "open_s": 6.0, "stay_open": false}],
  "triggers": [{"id": "t1", "poly": [[...]], "once": true, "on_enter": [{"spawn_group": "lot-east"}, {"open_door": "gate-a"}, {"host": "closet-open"}]}],
  "secrets": [{"id": "s1", "poly": [[...]]}],
  "exit": {"poly": [[...]], "next": "area-kitchen"}
}
```

- `solids` are convex polygons in the XZ plane (boxes are the fast path). Collision: a fighter or monster of radius r is blocked when its centre is inside a solid inflated by r; for boxes that is today's `Aabb2::expand`; for polygons it is the point-in-convex test against each edge offset outward by r, with rounded corners approximated by the edge offsets (a 0.5 unit corner error is invisible at this scale). Hitscan and sight use the same solids with the existing ray-versus-box slab test and a ray-versus-edge test for polygons.
- `skill` is a three-bit mask, easy 1, normal 2, hard 4, as Doom does it, so designers hand-place rosters per tier; the fourth tier (nightmare) uses the hard rosters with the fast flag.
- Everything is validated on load: bounds contain everything, at least one player spawn, exit present unless the map is an arena, door keys exist as items or are reachable from a previous map (checked by the episode manifest), trigger targets resolve, no two solids overlap a spawn.

### The map tool (`tools/mapc`, Rust)

`fragr-mapc build maps/larak-lot.map --out server/maps/larak-lot.json` parses a TrenchBroom Valve 220 `.map` with the `quake-map` crate, keeps brushes whose entity is worldspawn or a `func_*` solid, intersects each brush's planes to a convex hull, projects the hull onto XZ at the player's height band (the slab between floor 0 and eye height 1.6; brushes entirely above or below are decoration), emits a box when the projection is axis-aligned and a polygon otherwise, and maps point entities by classname: `info_player_start` to spawns, `monster_<type>` to monsters (spawnflags carry skill and ambush bits, TrenchBroom's `angle` to yaw), `item_health`, `item_armor`, `weapon_<name>`, `item_key_<color>`, `func_door` (targetname and key), `trigger_once` and `trigger_multiple` (target list), `trigger_secret`, `trigger_exit` (next map). A `.fgd` file in `tools/mapc/fragr.fgd` gives TrenchBroom the entity definitions. `fragr-mapc check` runs the validator. The client renders the same `.map` through func_godot with the look-pass atlas; geometry has one source.

### Monster tables

Monsters are server entities on the same tick as bots. A type is a row:

| Field | Meaning |
|---|---|
| `name` | `clerk`, `enforcer`, `jammer`, `turret`, `auditor`, `redactor`, `walker`, `swarm`, `drone`, `walker_boss` |
| `hp`, `radius`, `speed` | units per second; turrets 0 |
| `pain_chance` | 0 to 1, as Doom's value over 256 |
| `reaction_s` | delay before the first attack after waking (0.25 s baseline, 0 on damage) |
| `sight_arc_deg`, `sight_range` | forward cone for waking on sight |
| `sound_range` | units a fired weapon or a hit alerts through, not through closed doors |
| `melee` | optional `{range, damage, cooldown_s}` |
| `attack` | optional `{kind: hitscan or projectile, damage, cooldown_s, spread_deg, projectile: {speed, radius, homing: bool}}` |
| `attack_bias` | how attack probability rises as distance falls: chance per tick `= clamp((bias - dist) / bias, 0.05, 1.0) * base` (Doom's distance check in one number) |
| `infight_hold_s` | how long a grudge against another monster lasts (Doom's 100 tics is about 3 s) |
| `special` | `resurrect` (auditor), `invisible_until_fire` (redactor), `stomp_knockback` (walker), `jam_radio` (jammer) |
| `fast` | speed and cooldown multipliers under the nightmare tier |

States per monster: `Idle`, `Alert(target, since)`, `Chase(target)`, `Attack(target, wind_up_until)`, `Pain(until)`, `Dead(since)`. Rules, all in seconds:

1. Idle: wake on sight (target inside the arc and range with a clear ray) or on sound (an alert event within `sound_range` whose path crosses no closed door), unless `ambush`, which wakes on sight only.
2. Chase: each step pick one of eight compass directions toward the target, diagonals first, re-pick on a block or every random 0.4 to 1.2 s (the seeded RNG); melee if in range; else attack when the cooldown is ready and the distance roll passes; after an attack, one step of movement before the next attack.
3. Pain: on damage roll `pain_chance`; success interrupts the current state for the pain duration (0.3 s) and resets reaction time to 0 ("awake now").
4. Infighting: damage from a monster of another type retargets the victim to the attacker for `infight_hold_s`; same type never retargets; auditors are never targeted and always retarget. Hitscan grunts can hit each other; projectiles skip their own type.
5. Projectiles are server entities with position, velocity, radius, owner, and damage; they hit the first fighter or monster on their path (grid candidates) or a solid, and vanish.
6. Dead monsters leave a corpse marker in the snapshot for the client's death animation and are removed after 10 s; auditors resurrect corpses within 8 units at full hp.

Each type's row is a test: a deterministic sim test spawns the type against a stationary player and asserts wake, chase, attack timing, and pain behaviour to the tick.

### Encounters, keys, secrets, exit

Triggers are polygons with once or repeat semantics and an action list: spawn a group (closets and teleport traps), open or close a door, play a Host line, alert monsters in a radius. Doors are solids that open on a key (removed from the solids while open) and close after `open_s` unless `stay_open`. Secrets count when first entered and print on the results card. The exit polygon ends the map when a player enters it with no boss alive: results card (time, frags, secrets found, deaths, par time), then the next map from the episode manifest.

### Episode manifest and saves

`server/maps/episodes.json`: episodes as ordered map lists with par times and a boss flag per map. The server keeps `campaign.json` in its data directory: episode, map index, skill, carried weapons, hp and armor at exit, best time per map. `--campaign-continue` loads it; a fresh `--campaign e1` resets. The client mirrors best scores in `user://scores.cfg` for the boot menu.

### Agents in the campaign

Telemetry adds visible monsters by type, range bucket, and whether awake; keys held; a locked door seen; an objective hint (`find_key`, `open_door`, `reach_exit`, `clear_boss`). The brain gets one more question, an objective choice; the reflex layer keeps shooting.

## Research notes (2026-09-18)

- **Doom's monster spec, from the released source.** Each type has spawn, see, pain, melee, missile, death, gib, and raise states; the AI is `A_Look` and `A_Chase`. `A_Look` wakes on a sector sound target unless the monster is flagged ambush, else on sight in a forward arc. `A_Chase` each tic: count down reaction time (eight tics for every type, zeroed on damage), count down the infight threshold, melee if in range, else fire if `P_CheckMissileRange` passes (closer means likelier, being hit provokes, capped at 200 of 256), and after a shot set just-attacked so the next tic moves. Movement picks one of eight directions, diagonals first, and re-picks on a block or every random 0 to 15 steps. Pain chance out of 256 by type: zombie 200, shotgun guy 170, imp 200, demon 180, lost soul 256, cacodemon 128, knight and baron 50, revenant 100, mancubus 80, chaingunner 170, arachnotron 128, pain elemental 128, arch-vile 10, spider 40, cyberdemon 20. Sound floods through two-sided lines, one sound-block line costs a step, closed doors stop it. Infighting: retarget on damage when the threshold is zero (then hold the grudge for 100 tics), same species never hurt each other with projectiles, hitscan is not exempt, the arch-vile is never targeted. Our rung 2 roster implements these as a table per type plus the threshold rule.
- **Orthogonal roster.** Each Doom type sits on stay-back versus charge and projectile versus hitscan, plus one problem: hitscan grunts punish standing still, the imp teaches dodging, the demon is melee pressure, the lost soul ignores floors, the cacodemon is a slow flying sponge, the revenant forces line-of-sight breaks, the mancubus forces gap-finding, the arachnotron suppresses, the chaingunner is a priority target, the pain elemental is a timer, the arch-vile rewrites the fight. Our ten types map onto that grid: Clerk (hitscan grunt), Enforcer (rusher), Jammer (slow projectile with a radio cost), Turret (suppressor, telegraphed), Auditor (resurrects Clerks, arch-vile role), Redactor (invisible until it fires, the ambush tutor), Walker (armoured stomp), Drone Swarm (fragile pressure), Compliance Drone (elite), the Continuance Walker boss.
- **Encounter rules worth keeping.** Romero's eight: floor height changes with texture changes, borders, alignment, light and space contrast, reachable exteriors, several secrets per level, loops that revisit, landmarks. eev.ee's design notes: ambush flags and sound-block lines, closets and teleport traps, visible but unreachable goals, keys gating only some routes, secrets hinted by texture and geometry, an ammo baseline counted in shotgun blasts, surplus health because extra medikits do not help once you are dead. Doom 2016's push-forward: resources come from combat actions, arenas assume full resources, weapons map to enemy roles, never make the player want to stop engaging.
- **Arcade ladder references.** Doom Eternal Horde: round types (arena waves, timed blitz, bonus coin, traversal), score scales with demon size, bounty targets decay in value, per-round tallies with bonuses for unspent lives, three lives plus earned ones. Killing Floor 2: specimens per wave times a difficulty modifier times a player modifier, one special squad per wave, trader time between waves. Devil Daggers: one metric (survival time), instant restart, an in-run new-record notice, replays from the leaderboard. Results cards that stick: itemised score lines, the best shown before, during, and after, an explicit new best, ranked runs not careers.
- **Difficulty as Doom does it.** Three spawn bits per thing (easy, normal, hard) so designers hand-place three rosters per map; the easiest tier halves player damage; the easiest and the hardest double ammo pickups; nightmare and fast mode halve demon and imp attack tics, speed projectiles, drop the pause before missiles, and respawn monsters after twelve seconds. Our four tiers: three placement bits plus a damage multiplier and a fast toggle.
- **Persistence and authority.** The client keeps `campaign.cfg` (episode, map, skill, loadout) and `scores.cfg` (best per ladder tier) as `ConfigFile` under `user://`. Solo play launches `fragr-server` as a child with `OS.create_process` on loopback (today `tools/solo_scrap.sh` does this from the shell; in-client spawning is new work in the boot menu); the server issues the results card and keeps its own small save so continue-from-last-map is validated server-side. Agents join the same socket.
- **Agents in the campaign.** Every published attempt at Doom by language models from pixels fails at tick rate (well under two percent of Doom 2's first level even with the game paused; zero frags where a 1.3 million parameter model scores 178). The ones that work hand the model symbolic state and let a local controller fight. That is fragr's shape already: extend the telemetry with visible enemies by type and range, keys held, locked doors seen, and an objective hint, and let the brain pick objectives (keys, doors, secrets, retreat) while the reflex layer shoots.

## Verification

- Server tests per rung; `cargo llvm-cov` floor holds.
- Headless smoke: server in campaign mode plus the scripted adapter bot completing e1m1 in a bounded tick count.
- Godot CI job; regenerated tip screenshots when the client shows new elements.

## Success criteria

- [ ] Arcade ladder shipped and played to a results card.
- [ ] Ten enemy types with tests; infighting works.
- [ ] Map format with keys, secrets, and exits; validator in CI.
- [ ] Episode one playable end to end with par times.
- [ ] Difficulty tiers and continue.
- [ ] Episodes two and three.
- [ ] Co-op through episode one with agents in the party.
- [ ] World map screen.
