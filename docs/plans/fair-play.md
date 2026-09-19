# Plan: fair play (anti-cheat that keeps the game fun)

**Status:** planned (2026-09-18), Phase 2 with hooks in Phase 1
**Branch:** `feat/fair-*`
**Spend:** $0. No kernel driver, no third-party anti-cheat service, ever: this is an open-source game that respects the machine it runs on.

## Goal

Cheating kills fun faster than anything else in a shooter, and fragr has an unusual shape: agents are first-class and a perfectly aimed agent is not a cheat, it is the design. Fairness here means three things: the server decides everything, the server can tell a human from a machine-assisted human, and everyone plays in a lane where the rules are the same for all.

## Non-goals

- Client integrity checks, kernel drivers, obfuscation, or anything that treats the player's computer as hostile. The client is open source; assume it is modified.
- Stopping agents from aiming well. Agents play in agent lanes.
- Perfect detection. The aim is to make cheating pointless and visible, not impossible.

## What the architecture already gives us

The Rust server owns positions, damage, health, frags, spawns, pickups, and rounds. A client sends intents, never state. Speed hacks, teleports, health edits, and ammo edits are impossible by construction because the client has nothing to edit. The remaining client-side levers are exactly the ones every FPS has: aim (client-owned yaw after buttery stage 1), fire timing, and information (seeing through walls in a modified client).

## Design

1. **Validated inputs.** Every input field is checked: finite yaw, at most four inputs applied per client per step, sequence numbers monotonic, `view_tick` within the rewind window, fire honoured only when the weapon cooldown has elapsed on the server clock. Out-of-range input is dropped and counted; a burst of it disconnects with a clear reason.
2. **Information control.** Interest management (`massive-arenas.md`) only sends what a client could see: fighters within radius and, once line-of-sight culling lands, not behind solids. A wallhack shows nothing the server did not send. Spectators get more; a spectator cannot fire.
3. **Lanes.** Every session is one of: humans only, mixed, agents only. The role in `Hello` is a claim; the lane is enforced by behaviour (below). Leaderboards and the results card are per lane. A mixed lane says so on the scoreboard, so nobody is surprised by the fighter with a 250 ms reaction time and 100 percent accuracy.
4. **Behaviour profiling on the server.** Per fighter, rolling statistics that the playtest harness already partly computes: reaction time from target visible to first hit, aim snap angle per input, accuracy per weapon by range bucket, time on target before firing, and input timing regularity. Rule bots and the reference agents give the machine baselines; recorded human sessions give the human baselines. A human-lane fighter whose profile sits in the machine band for a sustained window is flagged, moved to the mixed lane for the rest of the session, and told so politely by the Host ("Caller, you are playing like a clawbot; welcome to the mixed lane"). No bans from statistics alone.
5. **Replays as evidence.** With the seeded sim and input logs, a round can be replayed exactly. Every flag stores the seed and the input log under the server's data directory; a host reviews before any ban. The playtest harness reads the same logs.
6. **Rate and abuse limits.** Speak cooldown, join tickets, per-IP caps, and ban lists from `public-server-hardening.md`. Bans are by ticket issuer and IP with an expiry, reviewed, and never automatic from the profiler.
7. **Fair by design in the rules.** Spawn shields and far-slot respawns already stop spawn camping. Lag compensation is bounded so high ping cannot shoot into the past. Fire is server-clocked so macro fire rates cannot exceed the weapon. The Host's "sus meter" is a parody bit on the HUD; the real profile is on the status line and in the report.
8. **Agents are welcome, labelled.** Agent fighters carry their chip on the scoreboard (already shipped for the brain's stance), the mixed lane is the default for public servers, and a public server can be humans-only with a flag.

## Verification

- Tests for every input rejection and count.
- A profiler test that classifies the four rule bots and the reflex agents as machine and a recorded human input log as human.
- A replay test: same seed and input log reproduce the frag list.
- Tour stills of the lane label and the Host line.

## Rungs

1. Input validation and counters, fire on the server clock (lands with buttery stage 1).
2. Lanes in `Hello` and on the scoreboard; per-lane results.
3. Profiler statistics on the status line and in the playtest report; machine and human baselines recorded.
4. Flag to mixed lane with the Host line; replay storage; review tooling.
5. Line-of-sight culling in interest management.

## Success criteria

- [ ] Every input rejection path tested.
- [ ] Rule bots and reflex agents classified machine; a human log classified human.
- [ ] A flagged session replays to the same frag list.
- [ ] Lanes visible on the scoreboard and in results.
