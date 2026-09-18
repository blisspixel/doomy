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
3. **Level format and keys.** Maps as data (`server/maps/*.json` plus the matching Godot scene): brushes, spawns, item and monster placements, doors with red, gold, and cyan keys, secrets with a counter, exit switches. Server validates a map on load. Evidence: a map validator test, one hand-built map with all element types.
4. **Episode one.** Eight maps that teach in order (movement, Scatter, keys, Rail, secrets, Turrets, Auditors, boss), built to the Romero rules in `docs/DESIGN-REFERENCES.md`, each with a par time. Evidence: a recorded full-episode run, map-by-map notes in `docs/plans/campaign-e1.md`.
5. **Ladder and tiers.** Weapon pickups unlock across the episode; four difficulty tiers scale monster count, damage, and item scarcity; continue-from-last-map with the loadout carried. Evidence: tests on the tier tables and the save format.
6. **Episodes two and three.** Same bar, new families of maps (Compliance Yard interiors, Perim Ghost outskirts, the tower), the Walker boss and a final broadcast. Evidence: recorded runs.

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
