# Plan: buttery controls

**Status:** planned (2026-09-18)
**Branch:** `feat/controls-*` (one PR per stage below)
**Spend:** $0.

## Goal

The 1.0 bar in [`../ROADMAP.md`](../ROADMAP.md) asks for first-person movement and aim that feel buttery: client-side prediction with server reconciliation, interpolation on every other fighter, no rubber-banding on a LAN or a good connection, input latency under fifty milliseconds on a LAN, and all of it measured. This plan turns the research of 2026-09-18 into a design with numbers and a staged order.

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

## Measurements

The benchmark mode prints these; the 1.0 release notes quote them.

- Transport, LAN: RTT p99 under 5 ms, jitter (p99 arrival delta) under 3 ms, zero loss. Public same-region: RTT p50 under 60 ms, p99 under 100 ms, loss under 1 percent. Under 20 KB/s down per client at eight players. Correction magnitude p99 under 10 cm.
- Feel: motion-to-photon under 50 ms at 60 Hz and under 35 ms at 144 Hz, measured with a phone at 240 frames per second filming mouse and screen or an Open-Source-LDAT build. Frame budget 6.9 ms at 144 Hz; p99 frame time under 1.5 times the median; no frame over twice the median during a sixty second bot match. Aiming measurably degrades from about 41 ms of local latency, and 4 to 12 ms of frame-time variance reads as less smooth, so those are the lines.
- Repeatable tests: a headless Godot harness injects mouse motion and asserts yaw changes in the same frame and the predicted body moves on the next tick; a Rust test asserts an input is acknowledged within one tick; a sixty second LAN run logs frame time, correction error, snapshot age, and RTT, and CI gates on the p99s.

## Staged PRs

1. Client-owned yaw and one numbered input per physics frame; server accepts absolute yaw. The immediate feel win.
2. The shared move function with golden vectors, prediction and reconciliation for the local pawn, correction metrics on the status line.
3. Timeline interpolation for others with snapshot velocity; the lerp removed.
4. 60 Hz sim with 20 Hz snapshots; tick constants to seconds.
5. Lag compensation with bounded rewind.
6. Gamepad curves and aim friction.
7. The transport spike against the pass thresholds above.

## Sources

Gabriel Gambetta's client-server series; Valve's Source multiplayer networking article; the Overwatch gameplay architecture and netcode talk (GDC 2017); Glenn Fiedler on snapshot interpolation, state synchronisation, UDP versus TCP, and floating point determinism; Riot on Valorant's 128 tick servers and peeker's advantage; the ioquake3 `sv_fps` default; Godot 4.7 docs on physics interpolation, Input, and the high polling rate mouse fix; Insomniac's aim assist talk (GDC 2013); the CHI 2015 latency and aiming study.

## Success criteria

- [ ] Stage 1 shipped: yaw is client-owned and inputs are numbered.
- [ ] Prediction with golden vectors passing in both languages.
- [ ] Others interpolate on a timeline; no lerp.
- [ ] 60 Hz sim; constants in seconds.
- [ ] Lag compensation bounded and tested.
- [ ] Gamepad tuned against the values above.
- [ ] Transport decided with a benchmark table in this file.
