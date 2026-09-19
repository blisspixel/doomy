# Plan: snapshot efficiency and massive arenas

**Status:** planned (2026-09-18)
**Branch:** `feat/wire-*` and `feat/scale-*` (one PR per rung)
**Spend:** $0 until the scale ladder needs a machine bigger than a laptop; that is Phase 3 and needs approval.

## Goal

The roadmap promises small squads today, full servers of thirty-two to sixty-four next, and arenas of hundreds where most fighters are agents. Today every tick serialises the whole world as JSON and sends it to every client, hit tests are all-pairs, and the RNG is unseeded. This plan is the design that gets from here to there without a rewrite, in rungs that are each measured before the next is claimed. It is written to be decision-complete: the numbers, formats, and algorithms are chosen here so implementation is mechanical.

## Non-goals

- A new engine or an ECS. The tick loop stays a plain loop over vectors.
- Cross-arena play. An arena is the unit of simulation; scale beyond one arena is more arenas.
- Compressing the agent door. `observe` stays JSON and human-readable; agents pay in bandwidth, not in clarity.

## Where the cost is today

- Snapshot: about 300 bytes per player per tick as JSON, sent to every client, twenty times a second. Eight fighters and eight spectators is roughly 380 KB/s out of the server; sixty-four fighters and sixty-four watchers would be 25 MB/s, which is the wall.
- Hit tests: every firing player against every other player every tick, each with an angle test and a ray-versus-boxes cover test. Fine at sixteen; at two hundred fighters all firing it is forty thousand cover rays a tick.
- Bots: each rule bot scans every player each tick to pick a target.
- Determinism: `rand::random` for spawn angles and spread, so a run cannot be replayed.

## Design

### 1. Seeded simulation

One `StdRng` per `GameState`, seeded from `--seed` or the clock, logged at boot and printed in the status line. Every random draw in the sim goes through it. Rule bots get a child RNG each from the same seed. This is a prerequisite for the playtest harness's paired rounds, for the benchmark being repeatable, and for replays later. Zero cost.

### 2. Spatial grid

A uniform grid over the arena bounds with 8 unit cells (`HashMap<(i16, i16), SmallVec<[u16; 8]>>` rebuilt each tick from player positions; rebuilding is cheaper than maintaining). Queries: fighters within radius R (target selection, pickups, sound alerts, interest sets), and the cells a ray crosses (hitscan candidates by 2D DDA). Hit tests go from all-pairs to candidates along the ray; bot target scans go from all players to the cells within their sight range.

### 3. Interest management

Each client has a relevance set recomputed every tick from the grid:

- Fighters within 32 units of the client's own fighter (for spectators: within 32 units of the camera target, or everyone when the director camera is wide).
- Anyone who damaged or was damaged by the client's fighter in the last two seconds.
- The top three scorers, as score lines only (name and score), so the scoreboard is always right.
- Everyone, once, on join and on round start (a full snapshot), then relevance only.

A fighter that leaves the set gets one last `gone` marker so the client can hide it. Spectators in the wide director view get the full set at 10 Hz instead of 20 Hz. Agents get their relevance set through `observe` exactly like a human client with the same radius, which also bounds `observe` cost at scale.

### 4. Delta snapshots with acknowledgement

The server keeps the last eight snapshots it sent to each client, keyed by tick. Each client acknowledges the newest tick it has applied in its regular input message (humans) or `observe` call (agents). A snapshot is encoded as a delta against the newest acknowledged baseline: only fields that changed for players in the relevance set, plus adds and removes. If no baseline is acknowledged within eight ticks the server sends a full snapshot again. Order is guaranteed on WebSocket; on UDP (buttery-controls stage 7) the baseline mechanism is exactly what tolerates loss.

### 5. Binary wire format for humans (fragr-wire v1)

JSON stays for `Hello`, `Welcome`, events, speak, the agent door, and spectators who ask for it. Humans and the playtest harness move to binary frames on the same socket once the delta path exists. Little-endian, versioned by a leading byte, fixed layouts, no self-describing overhead:

| Field | Type | Notes |
|---|---|---|
| version | u8 | 1 |
| kind | u8 | 1 snapshot, 2 ack, 3 input |
| tick | u32 | |
| base_tick | u32 | 0 for a full snapshot |
| count | u16 | entries that follow |
| per entry: slot | u16 | slot index assigned at join, mapping sent in `Welcome` and on change |
| per entry: mask | u8 | which of the fields below are present |
| x, z | i16 each | centimetres, range about 327 metres |
| vx, vz | i16 each | centimetres per second |
| yaw | u16 | 65536 per turn |
| hp | u8 | |
| armor | u8 | |
| weapon and flags | u8 | weapon in the low two bits; fired, shielded, dead in the next bits |
| score | u16 | present when changed |

A changed fighter costs 4 to 14 bytes; an unchanged one costs nothing. At sixty-four fighters with everyone moving and a 32 unit relevance radius, a client sees perhaps twenty fighters a tick at about 12 bytes each, under 5 KB/s at 20 Hz. Snapshot age and bytes per client go on the status line so this is measured, not asserted.

### 6. Tick budget model and the scale ladder

Per tick the server does: inputs (O(fighters)), movement with collision (O(fighters times obstacles), constant per map), hit tests (O(shots times candidates)), bots (O(bots times neighbours)), interest sets (O(clients times neighbours)), encoding (O(clients times relevant)). With the grid every term is linear in fighters times a local neighbourhood. The ladder measures tick time p50 and p99 with `--bench N M` from playtest rung 3:

| Rung | Fighters | Clients | Pass at 20 Hz | Pass at 60 Hz movement step |
|---|---|---|---|---|
| Squad | 16 | 16 | tick p99 under 4 ms | under 2 ms |
| Server | 64 | 64 | tick p99 under 10 ms, under 20 KB/s per client | under 5 ms |
| Arena | 128 | 64 | tick p99 under 20 ms, under 30 KB/s per client | under 8 ms |
| Massive | 256 | 64 | tick p99 under 35 ms, under 40 KB/s per client | measured, no claim |

A rung is claimed only with its table in this file and the status line JSON in the release notes.

### 7. Arenas as the unit of scale

One process runs one arena today. The next step is one process running several arenas, each its own tokio task with its own `GameState`, RNG, and clients, joined by arena id in `Hello`. No shared state between arenas; a lobby is a list. Regions in Phase 3 are more processes. Cross-arena spectating is a second socket.

### 8. Agents at scale

- Scripted agents: the adapter's roster mode runs many agents in one process with one socket each; the playtest harness already does sixteen in-process.
- Decision brains: bounded by the provider's rate limit (about twenty requests a second per key), so a hundred brains at three decisions a second is four keys or a lower cadence; the brain backs off on its own when rate limited. Cost math in `decision-brain.md`.
- MCP clients: `observe` returns the relevance set, so an agent's cost per call does not grow with the arena.

## Rungs

1. Seeded RNG and the spatial grid, with hit tests and bot scans on the grid. Evidence: identical results to all-pairs on the sim tests, a benchmark line at 16 and 64.
2. Interest sets and per-client snapshots (still JSON), spectators at 10 Hz in the wide view. Evidence: bytes per client before and after at 64.
3. Delta snapshots with acknowledgement and the full-resend fallback. Evidence: a loss-injection test on the in-process harness.
4. fragr-wire v1 for humans and the harness; JSON kept for everything else. Evidence: the ladder table through the Server rung.
5. Several arenas per process. Evidence: the Arena rung measured with two arenas on one process.

## Success criteria

- [ ] Seed printed on boot and in the status line; a paired-round playtest reproduces frag counts.
- [ ] Hit tests on the grid match all-pairs on every sim test.
- [ ] Server rung passed and tabled.
- [ ] Arena rung passed and tabled.
