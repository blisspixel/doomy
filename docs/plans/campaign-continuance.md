# Plan: single-player campaign (Continuance)

**Status:** planned (2026-09-18)
**Branch:** `feat/campaign-*` (one PR per rung below)
**Spend:** $0. Enemy sprites and maps are authored in-repo; Host and enemy voice lines come from the audio pipeline when needed.

## Goal

A single-player campaign at the bar of Doom 1 and Doom 2, in this lore: the Office of Global Continuance has seized the Contested Frequency towers, and you fight through their compliance apparatus to put the station back on the air. Three episodes of eight to nine hand-built maps, an enemy roster where each type is a different problem, keys and secrets, an episode boss, a weapon ladder that grows across the run, four difficulty tiers, and continue-from-last-map. Monsters are server-authoritative entities on the same tick as bots, so agents can play the campaign too, and a spectator can watch a solo run.

## Non-goals

- Cutscenes, lore codices, dialogue trees. The Host frames each map in one line.
- A separate single-player engine. The campaign is a server mode (`--mode campaign --map e1m1`) with the Godot client presenting it.
- Multiplayer campaign co-op in the first episode (it follows once the mode is stable; the architecture allows it).

## Rungs

1. **Arcade ladder (first playable).** `--mode arcade`: rounds versus escalating rule-bot rosters with a boss beat every third round, a results card, local best scores saved under `user://`. Reuses everything that exists. Evidence: a full ladder run in a headless smoke, best scores persisted.
2. **Enemy roster.** Ten Continuance types, each a distinct problem, implemented as server entities with the bot controller seam: Compliance Drone (existing boss, demoted to elite), Clerk (hitscan chip damage, weak), Jammer (slow projectile, blocks radio until killed), Enforcer (rusher, melee), Turret (static, high damage, telegraphed), Auditor (floats, resurrects Clerks), Redactor (invisible until it fires), Walker (armoured, stomp knockback), Drone Swarm (many, fragile), Continuance Walker boss (episode end). Infighting through pain states so enemies can be baited into each other. Evidence: a deterministic sim test per type and an infighting test.
3. **Level format and keys.** Maps authored in TrenchBroom as Quake `.map` files (Valve 220 format). A Rust tool parses them with the `quake-map` crate and emits the server-owned JSON manifest: brush half-spaces for collision (point and capsule tests need no vertex build), point entities for spawns, items, monsters with skill flags, doors with red, gold, and cyan keys, triggers, secrets with a counter, and exit switches. The server loads the manifest and validates it; the client renders the same `.map` through func_godot (MIT, GDScript). func_godot's Godot 4.7 compatibility is unstated and must be tested first; the fallback is the Rust tool emitting meshes. Evidence: a map validator test, one hand-built map with every element type, one source of truth for geometry.
4. **Episode one.** Eight maps that teach in order (movement, Scatter, keys, Rail, secrets, Turrets, Auditors, boss), built to the Romero rules in `docs/DESIGN-REFERENCES.md`, each with a par time. Evidence: a recorded full-episode run, map-by-map notes in `docs/plans/campaign-e1.md`.
5. **Ladder and tiers.** Weapon pickups unlock across the episode; four difficulty tiers scale monster count, damage, and item scarcity; continue-from-last-map with the loadout carried. Evidence: tests on the tier tables and the save format.
6. **Episodes two and three.** Same bar, new families of maps (Compliance Yard interiors, Perim Ghost outskirts, the tower), the Walker boss and a final broadcast. Evidence: recorded runs.

## Research notes (2026-09-18)

- **Doom's monster spec, from the released source.** Each type has spawn, see, pain, melee, missile, death, gib, and raise states; the AI is `A_Look` and `A_Chase`. `A_Look` wakes on a sector sound target unless the monster is flagged ambush, else on sight in a forward arc. `A_Chase` each tic: count down reaction time (eight tics for every type, zeroed on damage), count down the infight threshold, melee if in range, else fire if `P_CheckMissileRange` passes (closer means likelier, being hit provokes, capped at 200 of 256), and after a shot set just-attacked so the next tic moves. Movement picks one of eight directions, diagonals first, and re-picks on a block or every random 0 to 15 steps. Pain chance out of 256 by type: zombie 200, shotgun guy 170, imp 200, demon 180, lost soul 256, cacodemon 128, knight and baron 50, revenant 100, mancubus 80, chaingunner 170, arachnotron 128, pain elemental 128, arch-vile 10, spider 40, cyberdemon 20. Sound floods through two-sided lines, one sound-block line costs a step, closed doors stop it. Infighting: retarget on damage when the threshold is zero (then hold the grudge for 100 tics), same species never hurt each other with projectiles, hitscan is not exempt, the arch-vile is never targeted. Our rung 2 roster implements these as a table per type plus the threshold rule.
- **Orthogonal roster.** Each Doom type sits on stay-back versus charge and projectile versus hitscan, plus one problem: hitscan grunts punish standing still, the imp teaches dodging, the demon is melee pressure, the lost soul ignores floors, the cacodemon is a slow flying sponge, the revenant forces line-of-sight breaks, the mancubus forces gap-finding, the arachnotron suppresses, the chaingunner is a priority target, the pain elemental is a timer, the arch-vile rewrites the fight. Our ten types map onto that grid: Clerk (hitscan grunt), Enforcer (rusher), Jammer (slow projectile with a radio cost), Turret (suppressor, telegraphed), Auditor (resurrects Clerks, arch-vile role), Redactor (invisible until it fires, the ambush tutor), Walker (armoured stomp), Drone Swarm (fragile pressure), Compliance Drone (elite), the Continuance Walker boss.
- **Encounter rules worth keeping.** Romero's eight: floor height changes with texture changes, borders, alignment, light and space contrast, reachable exteriors, several secrets per level, loops that revisit, landmarks. eev.ee's design notes: ambush flags and sound-block lines, closets and teleport traps, visible but unreachable goals, keys gating only some routes, secrets hinted by texture and geometry, an ammo baseline counted in shotgun blasts, surplus health because extra medikits do not help once you are dead. Doom 2016's push-forward: resources come from combat actions, arenas assume full resources, weapons map to enemy roles, never make the player want to stop engaging.
- **Arcade ladder references.** Doom Eternal Horde: round types (arena waves, timed blitz, bonus coin, traversal), score scales with demon size, bounty targets decay in value, per-round tallies with bonuses for unspent lives, three lives plus earned ones. Killing Floor 2: specimens per wave times a difficulty modifier times a player modifier, one special squad per wave, trader time between waves. Devil Daggers: one metric (survival time), instant restart, an in-run new-record notice, replays from the leaderboard. Results cards that stick: itemised score lines, the best shown before, during, and after, an explicit new best, ranked runs not careers.
- **Difficulty as Doom does it.** Three spawn bits per thing (easy, normal, hard) so designers hand-place three rosters per map; the easiest tier halves player damage; the easiest and the hardest double ammo pickups; nightmare and fast mode halve demon and imp attack tics, speed projectiles, drop the pause before missiles, and respawn monsters after twelve seconds. Our four tiers: three placement bits plus a damage multiplier and a fast toggle.
- **Persistence and authority.** The client keeps `campaign.cfg` (episode, map, skill, loadout) and `scores.cfg` (best per ladder tier) as `ConfigFile` under `user://`. Solo play launches `fragr-server` as a child with `OS.create_process` on loopback, as the Solo Scrap launcher already does; the server issues the results card and keeps its own small save so continue-from-last-map is validated server-side. Agents join the same socket.
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
