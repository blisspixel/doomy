# Plan: buttery controls

**Status:** in flight (stage 1 shipped 2026-09-18)
**Branch:** `feat/controls-*` (one PR per stage below)
**Spend:** $0.

## Goal

The 1.0 bar in [`../ROADMAP.md`](../ROADMAP.md) asks for first-person movement and aim that feel buttery: client-side prediction with server reconciliation, interpolation on every other fighter, no rubber-banding on a LAN or a good connection, input latency under fifty milliseconds on a LAN, and all of it measured. This plan turns the research of 2026-09-18 into a design with numbers and a staged order.

## Non-goals

- Predicting other fighters, projectiles, or pickups. Only the local pawn is predicted.
- A new transport before the spike passes its thresholds. Stages 1 to 6 stay on WebSocket.
- Rewriting the playtest, brain, or adapter cadence; they consume the wire, they do not define it.

## Protocol changes

- Stage 1: `Action` gains an absolute `yaw` (f32) and an input `seq` (u32); the server acknowledges `last_seq` in the snapshot it sends that client.
- Stage 3: `PlayerState` gains velocity (three f32).
- Stage 4: every tick-count field on the wire (`respawn_in`, cooldowns, `duration_ticks`, the 160-tick linger) becomes seconds or milliseconds, with the tick rate stated in `Hello`.
- Each of these lands in `docs/protocol.md` in the same PR. The adapter reads the wire types from `fragr-server` (as the playtest harness and the brain do), so a wire change is edited once and the compiler finds every reader.

## Dependents of the tick change (stage 4)

Every `*_TICKS` constant in `server/src/sim.rs` and `server/src/protocol.rs`, the adapter's speak cooldown mirror, the playtest harness's spawn-death window, the brain's 50 ms controller interval, and the campaign's monster tables to come. Stage 4 lands before campaign rung 2 so those tables are authored in seconds from the start.

## Where we stand (from the code, not the docs)

- The server ticks at 20 Hz and every snapshot is the full JSON world (`server/src/run.rs`).
- The client sends an Action every rendered frame (`client/scripts/game_manager.gd`), so at 144 frames per second it sends 144 messages a second, nearly all redundant.
- Mouse yaw is quantised into `turn_left` and `turn_right` bits (`client/scripts/spectator_cam.gd`), the server turns at a fixed rate, and the camera copies the pawn's yaw, which `player_pawn.gd` smooths with an exponential lerp of roughly a 100 ms time constant. The look axis round-trips the network and then passes two smoothing stages. That is the mush. Prediction alone does not fix it unless yaw becomes client-owned.

## Design

1. **Client-owned yaw.** Every input carries an absolute yaw (f32). The server clamps only for sanity. Mouse motion is applied to the camera in the same frame it arrives and never waits on the network.
2. **Prediction and reconciliation for the local pawn only.** Inputs are numbered. The client applies each input locally at once and keeps the unacknowledged ones. The server returns the authoritative state plus the last processed sequence number; the client rewinds to that state and replays the unacknowledged inputs. Other fighters are never predicted.
3. **One move function, written twice.** The move integrator (acceleration, friction, maximum speed, dt equal to one tick) and static collision (capsule against a shared list of boxes) live in Rust and in GDScript with the same golden test vectors. The predicted step never uses the Godot physics server, because a Rust server cannot bit-match Jolt. Floats: f32 in both languages with a tolerance, corrections blended per frame by 0.95 for errors under 25 cm and 0.85 above 1 m, snapped beyond 1 m. Switch to fixed-point millimetres only if the correction metric stays noisy.
4. **Timeline interpolation for others.** A ring of tick-stamped states, rendered at server time minus 100 ms at 20 Hz snapshots (150 ms while on TCP, because one loss stalls the stream), 33 to 50 ms once snapshots run at 60 Hz. Extrapolate at most 100 ms, then freeze. The exponential lerp goes away.
5. **Lag compensation for hitscan.** The server rewinds targets by `clamp(RTT / 2 + interpolation delay, 0, 200 ms)` and keeps 250 ms of history. This matters even on a LAN: at 7 m/s a target moves 0.7 m during a 100 ms interpolation delay, more than a capsule radius.
6. **Sim at 60 Hz, snapshots at 20 Hz.** Input granularity drops from 50 ms to 16.7 ms and the rewind history gets three times finer for trivial CPU. Every `*_TICKS` constant becomes seconds. Snapshots carry the tick, the last acknowledged input sequence, and velocity so 20 Hz interpolation does not lose the finer sim.
7. **One input per client physics frame.** Sixty a second, each packet bundling every unacknowledged input so loss costs nothing. Per-frame sending stops.
8. **Mouse feel in Godot 4.7.** `Input.use_accumulated_input` stays on; sum `relative` in `_unhandled_input`, apply the same frame, never multiply by delta; transform events by the viewport's final transform when the window stretches. Sensitivity exposed on the Source convention of 0.022 degrees per count at sensitivity 1 (the current 0.003 radians per count equals about 7.8 on that scale). `physics_jitter_fix` set to 0. The camera's physics interpolation off, rotation in `_process`, position from the body's interpolated transform. Godot 4.7.2 already fixes high polling rate mice on Windows.
9. **Gamepad look.** Radial deadzone 0.12 with rescale, response exponent 1.5 to 2, 250 degrees per second maximum yaw with a 0.25 s ramp in the outer five percent of the stick, aim friction at half speed inside a three degree cone around a fighter. Magnetism and snap later, if at all. These starting values are ours to tune, not sourced.
10. **Transport spike.** WebSocket JSON stays the control plane and the path for spectators and agents. Candidate A is a 12-byte header (sequence, ack, ack bits) over UDP with `PacketPeerUDP` on the client and `tokio::net::UdpSocket` on the server; candidate B is ENet through `ENetMultiplayerPeer` with a Rust binding still to be verified. WebTransport is not in Godot 4.7. Decide with the measurements below.

## Design detail (decision-complete)

Everything below is chosen so a later implementer does not have to choose. Numbers are starting values; the QA tour and the feel probes tune them, and a change lands with its golden vectors regenerated.

### The movement model, written once in words

State per fighter: position `(x, z)` in units, velocity `(vx, vz)` in units per second, yaw in radians in `[0, 2 pi)`. No vertical motion. Radius 0.5. The step is:

1. Wish direction: forward, back, left, right bits combined into a unit vector in the yaw frame (the same trigonometry as today, normalised when non-zero).
2. Target velocity: wish direction times top speed. Top speed 5.0, halved under the compliance slow (today's values, unchanged so balance holds).
3. Velocity approach: `v = v + (target - v) * min(1, dt / tau)` with `tau_accel = 0.06 s` when the target is faster than the current speed along the wish and `tau_decel = 0.04 s` otherwise. This replaces instantaneous velocity so stops and starts read as weight without feeling slow. If the QA tour says it feels mushy, halve both taus; the wire does not change.
4. Move: `x += vx * dt`, `z += vz * dt`, then collide.
5. Collide: clamp to the arena bounds, then the axis-separated slide against the obstacle boxes exactly as `resolve_move` does today (try full move, then x only, then z only, else stay). Boxes are the map manifest's obstacles inflated by the radius. Sliding zeroes the blocked velocity component so the next step does not push into the wall again.
6. Yaw: taken from the input, normalised into `[0, 2 pi)`. The server does not turn fighters any more; `turn_left` and `turn_right` remain on the wire for agents and are applied at the old rate only when the input carries no yaw.

Written twice: `server/src/movement.rs` (`pub fn step(state, input, dt, obstacles) -> state`) and `client/scripts/movement.gd` (`static func step(state, input, dt, obstacles)`), both pure functions over plain data, no engine calls. Both compile against the same golden vectors.

### Golden vectors

`docs/golden/move_vectors.json`: a list of cases, each with an obstacle list, an initial state, a `dt`, a list of inputs, and the expected state after every input, generated by the Rust step and committed. A Rust test asserts the Rust step reproduces the file to 1e-6; a headless Godot test (`client/scripts/test_move_golden.gd`, run by `tools/godot_check.sh`) asserts the GDScript step matches every step to 1e-4 and the final state of a 1000-step case to 1e-2. Regeneration is `cargo test -p fragr-server golden -- --ignored` writing the file; the diff is reviewed like code. Cases: straight run, diagonal run (normalisation), start and stop (the taus), slide along a wall in x, slide in z, corner stop, arena edge clamp, yaw wrap at 0 and 2 pi, compliance slow.

### Wire changes, exact

`Action` (client to server) gains three optional fields, all `serde(default)`, so old clients and every agent keep working:

- `seq: u32` input sequence, increasing by one per input, wrapping. Absent means an unnumbered agent action.
- `yaw: f32` absolute yaw in radians. Present means client-owned yaw; the server normalises and stores it.
- `view_tick: u32` the server tick the client was rendering other fighters at when this input was sampled (for lag compensation). Absent means no rewind.

A new unicast message, humans only, sent every tick after the snapshot:

```json
{"type": "ack", "seq": 4123, "tick": 88210, "x": 12.25, "z": -3.5, "vx": 4.9, "vz": 0.7}
```

`seq` is the last input applied to that fighter. Snapshots gain `vx` and `vz` per fighter (stage 3) and a `tick_hz` field in `Welcome` (stage 2). Every tick-count field on the wire becomes seconds as a float (`respawn_in_s`, `cooldown_s`, `time_left_s`), with the old fields kept for one release and the adapter mirror updated in the same PR.

### Input cadence and bundling

The client samples inputs in `_physics_process` at the server's movement rate (60 Hz after stage 2; 20 Hz before), assigns `seq`, applies the step locally, and sends the unacknowledged inputs (at most eight, oldest first) in one message. The server applies each input once, in order, skipping any `seq` it has already applied; it never applies more than four inputs from one client in one step (a client that falls behind is clamped, not trusted). Per-frame sending stops.

### Reconciliation

The client keeps a ring of the last 64 `(seq, input, state_after)`. On `ack`:

1. Find the ring entry for `ack.seq`. If missing (too old), snap to the ack state and clear the ring.
2. Error `e = ack.position - entry.state_after.position`.
3. If `|e| > 1.0` snap: set the state to the ack state and replay every input after `seq` with the shared step. Otherwise replay from the ack state the same way, and add `e` to a `visual_offset` that the renderer subtracts, decaying per frame by `0.95` when `|offset| < 0.25` and `0.85` above, so the camera never pops.
4. Record `|e|` into the correction histogram reported on the status line and by the feel probes (p99 under 10 cm is the pass).

### Interpolation of other fighters

A ring of the last eight snapshots with their ticks. Server time is estimated as `snapshot.tick * dt` plus an offset filtered with an exponential moving average over arrival times (alpha 0.1), never jumping more than 50 ms per second. Render time is server time minus 100 ms at 20 Hz snapshots on TCP (150 ms while packet loss is observed), 50 ms once snapshots come at 60 Hz or over UDP. Each other fighter is positioned by linear interpolation between the two snapshots bracketing render time; if render time is beyond the newest snapshot, extrapolate with the snapshot velocity for at most 100 ms, then hold. The exponential lerp in `player_pawn.gd` is deleted. Yaw interpolates by shortest arc.

### Lag compensation

The server keeps, per fighter, a ring of the last 15 movement steps (250 ms at 60 Hz) of `(tick, x, z)`. A fire input carrying `view_tick` rewinds every other fighter to `clamp(view_tick, now - 12 steps, now)` for the hit test, then restores. Without `view_tick` (agents, old clients) no rewind. Rewind is capped so a client cannot claim a shot from further back than 200 ms; the status line counts clamped rewinds.

### Tick migration (stage 2)

The sim gets a movement step at 60 Hz and keeps combat, bots, pickups, and snapshots on every third step (20 Hz). All `*_TICKS` constants become `Duration` or seconds, converted once at the tick boundary: cooldowns, spawn shield, respawn, compliance slow, round timers, end delay, the speak cooldown mirror in the adapter, the playtest spawn-death window, and the brain's controller interval, which becomes "one per snapshot" rather than 50 ms. `Welcome` states `tick_hz` and `snapshot_hz`.

### Mouse and gamepad pipeline in the client

Mouse: sum `event.relative` in `_unhandled_input` (accumulated input stays on), apply yaw and pitch to the camera in the same `_process`, never multiplied by delta, transformed by the viewport's final transform when stretched. Sensitivity is stored as a Source-convention number (0.022 degrees per count at 1.0) and exposed in settings; the current 0.003 radians per count corresponds to about 7.8. `physics_jitter_fix` is 0. The camera's `physics_interpolation_mode` is off; its position comes from the predicted pawn state plus the visual offset.

Gamepad: radial deadzone 0.12 rescaled to a full range, exponent 1.8, yaw rate 250 degrees per second at full deflection with a 0.25 s ramp when the stick is beyond 0.95, aim friction at half rate while a fighter is inside a 3 degree cone. All five values live in one dictionary in `settings.gd` so the feel probes can sweep them.

### Order of stages, revised

1. **Shipped.** Client-owned yaw and numbered inputs on the 20 Hz sim; the `ack` message; turn bits kept for agents. The look axis no longer round-trips: the camera and the fighter's facing move on the frame the mouse moves, and the server takes the absolute value. Bundling several unacknowledged inputs per message comes with stage 3, when there is something to replay.
2. Tick migration: 60 Hz movement step, constants to seconds, `tick_hz` in `Welcome`.
3. The shared step in both languages with golden vectors, prediction and reconciliation, correction metrics on the status line.
4. Timeline interpolation for others with snapshot velocity; the lerp removed.
5. Lag compensation with `view_tick` and the bounded rewind.
6. Gamepad curves and aim friction from the settings dictionary.
7. The transport spike against the pass thresholds above; fragr-wire v1 from `massive-arenas.md` is what goes over it.

## Measurements

The benchmark mode prints these; the 1.0 release notes quote them.

- Transport, LAN: RTT p99 under 5 ms, jitter (p99 arrival delta) under 3 ms, zero loss. Public same-region: RTT p50 under 60 ms, p99 under 100 ms, loss under 1 percent. Under 20 KB/s down per client at eight players. Correction magnitude p99 under 10 cm.
- Feel: motion-to-photon under 50 ms at 60 Hz and under 35 ms at 144 Hz, measured with a phone at 240 frames per second filming mouse and screen or an Open-Source-LDAT build. Frame budget 6.9 ms at 144 Hz; p99 frame time under 1.5 times the median; no frame over twice the median during a sixty second bot match. Aiming measurably degrades from about 41 ms of local latency, and 4 to 12 ms of frame-time variance reads as less smooth, so those are the lines.
- Repeatable tests: a headless Godot harness injects mouse motion and asserts yaw changes in the same frame and the predicted body moves on the next tick; a Rust test asserts an input is acknowledged within one tick; a sixty second LAN run logs frame time, correction error, snapshot age, and RTT, and CI gates on the p99s.

## Staged PRs

The order is the revised one at the end of the design detail: yaw and numbered inputs, tick migration, the shared step with prediction, interpolation, lag compensation, gamepad, transport. Stage 3 needs the status line from playtest rung 3 for its correction metrics.

## Sources

Gabriel Gambetta's client-server series; Valve's Source multiplayer networking article; the Overwatch gameplay architecture and netcode talk (GDC 2017); Glenn Fiedler on snapshot interpolation, state synchronisation, UDP versus TCP, and floating point determinism; Riot on Valorant's 128 tick servers and peeker's advantage; the ioquake3 `sv_fps` default; Godot 4.7 docs on physics interpolation, Input, and the high polling rate mouse fix; Insomniac's aim assist talk (GDC 2013); the CHI 2015 latency and aiming study.

## Success criteria

- [x] Stage 1 shipped: yaw is client-owned and inputs are numbered, proven by the server tests (client yaw wins over the turn bits and steers the same tick, acks report the state the input produced, agents unchanged). The same-frame yaw harness lands with the QA tour's feel probes.
- [ ] Prediction with golden vectors passing in both languages.
- [ ] Others interpolate on a timeline; no lerp.
- [ ] 60 Hz sim; constants in seconds.
- [ ] Lag compensation bounded and tested.
- [ ] Gamepad response curve asserted by a headless harness at five stick magnitudes, and the aim friction cone by two.
- [ ] Transport decided with a benchmark table in this file.
