# fragr roadmap

The order of operations to take fragr from a playable vertical slice to an exceptional game that people play, watch, and host. This file is the plan of record for sequencing. Feel and non-negotiables live in [`VISION.md`](./VISION.md). Bounded work items live in [`plans/`](./plans/README.md). Implementation truth is source, tests, and CI.

Every item below is in exactly one state: **planned**, **in progress**, **shipped** (merged to `main`), or **proven** (shipped and demonstrated with evidence: a test, a recorded smoke, a screenshot from the tip, or a real session). Do not promote an item without the evidence.

## The shape of the plan

1. **Prove it locally.** Solo play against bots, agent play through the adapter, and small multiplayer on one machine or a LAN. Zero spend. Everything here is testable in CI or a recorded smoke.
2. **Expose it.** A hardened server you can run on a home box or a small VM with the port open, where strangers and their agents join and it does not fall over.
3. **Make it cloud native.** GCP Terraform that applies cleanly, scales along a ladder, and stays under the spend cap. Not before the exposed server is proven.
4. **Deepen it.** More maps, modes, vehicles, progression, and the let's-play tooling that makes watching as good as playing.

The engineering ladder for scale runs through every phase: small squads first (four to twelve fighters, the current bar), then full servers (thirty-two to sixty-four), then large agent-heavy arenas (hundreds of fighters where most are agents). Each rung has its own measurements and is not claimed until measured.

## Where we are (2026-09-18)

**Shipped and proven on the tip:**

- Rust authoritative server at 20 Hz with hitscan combat, respawn, round scoring, two maps, weapon and health pads, a mid-round boss, and eight named rule bots with four behaviors.
- Godot 4.7.2 client as a thin presenter: boot menu, Solo Scrap, spectator director camera, human join and leave, first-person weapon face, HUD with killfeed and Host bumpers.
- MCP adapter with `join`, `leave`, `observe`, `act`, `speak`, `get_events`, `round_state`, plus a scripted bot. Unknown action fields are rejected. Speak is rate limited.
- CI on Linux: fmt, clippy with warnings denied, tests, an unfiltered 80 percent line coverage floor, release build.
- Live tip screenshots, a one-command Solo Scrap launcher, self-host guides, and plan-only GCP Terraform.

**Not built yet (honest list):** low-latency transport (WebSocket JSON only), client prediction, authentication or join tokens, per-connection rate limits and size caps, reconnect resume, desktop export presets, a Godot job in CI, protocol versioning, a status endpoint, persistent stats, progression, music and the radio stations, voiced Host, a single-player campaign (only one boss beat exists), a real art pass on sprites, guns, and levels, load tests, any cloud apply, vehicles, objective modes, larger maps.

## Phase 0: Foundations that make everything else cheaper

Status: **in progress**. Small, high-leverage, mostly tooling.

- **Standards.** `AGENTS.md` refreshed; workspace lints in `Cargo.toml`; CI actions on current majors; coverage tool installed as a prebuilt binary; dependency advisories checked in CI. Shipped with this roadmap.
- **Godot in CI.** Shipped: the `godot` CI job runs `tools/godot_check.sh` (import, parse every script, the radio and far-cam harnesses).
- **One protocol crate.** The adapter currently mirrors the server's wire types in its own `protocol.rs`. Extract `fragr-protocol` shared by both so the wire cannot drift. Planned.
- **Dev audio pipeline.** `tools/audiogen` generates sound effects and music through the ElevenLabs API for developers only, writes assets plus a manifest, and never runs in CI or at player runtime. Shipped with this roadmap; spoken bulletins (text to speech) and credit estimates are next.
- **Rust and GDScript only.** Port `tools/generate_audio.py` to a Rust subcommand of audiogen and delete the Python. Planned.
- **Sprite and texture pipeline.** Palette lock from `palette.json`, fixed base sizes, import presets, and a documented path from source plates to atlases. Planned.
- **Headless integration smoke.** A Rust test that boots the server, connects N scripted clients, and asserts frags, join, leave, and round transitions in a bounded time. Runs in CI. Planned.

## Phase 1: Local excellence (offline, zero spend)

Status: **planned**. This phase decides whether the game is fun. Everything here is validated on one machine against bots, in single player, and through the agent adapter.

1. **Movement and gunfeel.** Acceleration and friction that reward strafing, air control, a jump, weapon switch timing, recoil kick, hit reactions on the target, and screen feedback on the shooter. Evidence: a playtest checklist in `plans/` with numbers, tip screenshots, and a short recorded clip.
2. **Boomer shooter look pass.** Render the world at a low internal resolution and upscale with nearest filtering, limit surfaces to the locked palette with dithering, rebuild fighter sprites with eight facing directions and walk, fire, pain, and death frames, rebuild weapon view models with idle, fire, and bob frames, add muzzle flash and impact frames, and lay the HUD out on a grid so nothing overlaps. Level surfaces get a coherent tile atlas with baked lighting and trim. Plan: `plans/look-pass-boomer.md`. Evidence: regenerated tip screenshots and an updated art bible.
3. **Sound and music.** A full effect set (per weapon fire, impact by surface, footsteps, pickup, pain, death, respawn, round stingers) produced with the dev audio pipeline, then the **Contested Frequency radio**: in-game stations (rock, EDM, chill, hip hop, country, world, a "lock in" station of pure frag music, and a spoken news station that is lore bulletins), at least twenty tracks per station, two to six minutes each, mostly with lyrics that live in the lore (conspiracy radio, self-deprecating boomer-shooter humor, agents taking over, nods to Hermes, Pi, and the clawdbots). Station switching in the HUD, ducking under Host callouts. Plan first in `plans/radio-stations.md`, then generate in waves within the monthly credit budget. Host voice lines come after text Host lines are final. Evidence: assets committed with a manifest, wired in the client, heard in a smoke run.
4. **Bots that read as players.** Cover use, pickup seeking, target selection with memory, difficulty tiers, and behavior chips that stay truthful. Evidence: deterministic sim tests per behavior plus a recorded spectate.
5. **Agent competence ladder.** Example agents at three levels: scripted reflex, planner using `observe` and `act` every few ticks, and an off-tick LLM-driven example that shows the door works without touching the combat tick. Rule bots stay good, but agentic play is the more interesting product, so the door gets the attention: keep the adapter on the current MCP specification revision (it still advertises the 2024-11-05 handshake; the current revision is 2026-07-28), evaluate the official Rust MCP SDK, and add an Agent2Agent (A2A) style surface if it lets agent teams coordinate without touching the tick. Evidence: adapter transcripts committed under `docs/skills/`, an integration test for the scripted levels, a protocol-revision compatibility test.
6. **Agents that field agents.** One adapter process runs a roster of scripted bots, and an agent can request a rule bot teammate through an MCP tool. Server enforces the roster cap. Evidence: adapter tests and a recorded session.
6b. **Agent playtest loop.** Local agents play the game and file structured feedback so most iteration does not need human testers: a `playtest` harness boots a server, runs N agents through the adapter for a fixed number of rounds, and emits a report (time to first frag, deaths per minute, weapon usage spread, idle time, stuck detection, pickup contention, frustration signals such as repeated spawn deaths, and free-text notes from an LLM-driven observer reading the event stream). Reports land in `.agents/` locally and a summary table in the plan doc for the change under test. Humans still judge fun; agents catch the rest. Plan: `plans/agent-playtest-loop.md`. Evidence: the harness runs in CI on a small configuration and the report format is documented. Rung 1 shipped: `tools/playtest` runs four reflex agents through one round on every PR with `--assert`.
7. **Single-player campaign.** The bar is Doom 1 and Doom 2, rebuilt in this lore: three episodes of eight to nine hand-built maps each, a Continuance enemy roster of at least ten distinct types where each one is a different problem (drones, enforcers, turrets, jammers, a compliance walker), keys and secrets, an episode boss, a weapon ladder that grows across the run, difficulty tiers, and continue-from-last-map. Server-authoritative monsters run on the same tick as bots so agents can play the campaign too. Starts with an arcade ladder (rounds with escalating rosters and boss beats, a results card, local best scores) as the first playable rung. Plan: `plans/campaign-continuance.md`. Evidence: a full episode run in a recorded smoke, results persisted locally, map-by-map plan docs.
8. **Small multiplayer on a LAN.** Two to twelve humans and agents on one server, join and leave without ghosts, spectators in the same match. Evidence: a recorded two-machine session and reconnect tests.
9. **Controller support.** Full gamepad play (move, look with sensitivity curves, fire, weapon switch, join, leave, camera cycle, menu navigation) with prompts that switch between keyboard and pad glyphs. Works on the boot menu and in the match. Evidence: a headless input-map check plus a recorded pad session.
10. **Benchmark mode.** A boot menu entry and a `--benchmark` server flag that run a fixed scripted scenario (N bots on a fixed map for a fixed number of ticks) and print tick time percentiles, snapshot bytes per tick, and client frame time. Same numbers in CI on every PR so regressions show up as a diff, and the scale ladder has a ruler. Evidence: a benchmark table in `docs/` updated with each release.

Exit bar: the fun bar below passes on a LAN session with mixed humans and agents, and a stranger can be handed the repo and reach a fight in under two minutes.

## Phase 2: Exposed server (public, still cheap)

Status: **planned**. The server becomes something you would open to the internet.

1. **Hardening.** Join tokens or a server password, per-connection message rate and size caps, idle timeouts, player and spectator caps, name validation, structured audit logs for join, leave, and rejects. Evidence: tests for every reject path and a fuzz run over the wire parser.
2. **Protocol versioning.** `protocol_version` in `Hello`, clear rejection on mismatch, and the MCP adapter moved to the current MCP specification revision. Evidence: compatibility tests.
3. **Transport spike.** Measure WebSocket latency under load, then prototype the UDP path described in [`TRANSPORT.md`](./TRANSPORT.md). Keep WebSocket for spectators and agents. Decide with numbers. Evidence: a benchmark table in a plan doc.
4. **Snapshot efficiency.** Delta snapshots, interest management by distance, and a binary encoding option once the JSON path is measured. Evidence: bytes per tick per client before and after.
5. **Reconnect and resume.** A session token that reattaches a dropped human or agent to its pawn within a grace window. Evidence: tests plus a recorded kill-and-reconnect.
6. **Status endpoint and server list.** A tiny read-only status response (map, players, round) so a server browser or a Discord bot can show what is live. Evidence: documented and tested.
7. **Desktop exports.** Export presets for Windows, macOS, and Linux checked in, built on tags in CI, attached to releases. Evidence: a release with binaries that boot to Solo Scrap.
8. **Observability.** Tick time histogram, per-client bandwidth, crash-free uptime, and a health check. Evidence: metrics visible in logs during a load test.
9. **Prove it with strangers.** Home box or cheap VPS with port 6767 open, at least one session with people and agents who are not the maintainer. Evidence: a recorded session and a hosting guide updated from what actually went wrong.

Exit bar: a public server runs for a week without intervention, and the hosting guide gets someone else from zero to hosting in an evening.

## Phase 3: Cloud native on GCP (gated by spend)

Status: **planned**. Nothing applies until spend is approved in writing. Hard cap 50 dollars total.

1. **Container and service unit.** A reproducible server image and a systemd unit for the VM path. Evidence: image boots locally and passes the smoke.
2. **Terraform validated in CI.** `terraform fmt` and `validate` run without credentials on every PR. `plan` runs only with an explicit workflow input. Evidence: the CI job.
3. **First apply.** One small always-free-shaped instance, public 6767, IAP-only SSH, budget alert, billing export. Evidence: an outside smoke and the bill.
4. **Scale ladder.** Bigger instance, then a second arena, then regional instances. Never a scale-to-zero platform as the combat tick. Evidence: measured fighters per instance per tick budget at each rung.
5. **Operations.** Auto-restart, log shipping, stats backup if anything persists, a runbook for the three most likely failures. Evidence: the runbook and a rehearsed recovery.

## Phase 4: Depth and longevity

Status: **planned**. Only after Phase 2 is proven, so that new content lands on a stable base.

- **Maps that teach.** Verticality, flow loops, item control, and named callouts. Learn from the best Unreal Tournament arenas: every corridor has a reason and every fight has a second option.
- **Bigger modes.** Team deathmatch with COD-sized squads first, then objective control on larger maps with vehicles in the spirit of Battlefield 1942 conquest, without borrowing its art. Vehicles are server-authoritative entities on the same action path.
- **Massive agent arenas.** Hundreds of fighters where most are agents. Depends on the scale ladder: interest management, sharded arenas, and a measured tick budget. Not a marketing claim until measured.
- **Progression and cosmetics.** Unlocks and skins (Hangar Candy) that never change combat. Local first, server-authoritative when accounts exist.
- **Let's-play tooling.** Director camera that follows the story of a round, highlight reels, a stream overlay, and match replays from recorded snapshots.
- **Community servers.** A server list, mod hooks for maps and rosters, and a documented content pipeline.
- **Steam release, later.** fragr is an open-source passion project first. A Steam build only makes sense after the exposed server is proven and the campaign exists; it would add store presence and friends-list joining, not change the game. No store spend before then.

## The fun bar

Concrete, checkable, and required before any phase is called done. Evidence is a screenshot, a test, or a recorded session.

- **Ten seconds.** Boot to a live fight in under ten seconds. Something happens on screen in the first five.
- **Readable.** Any fighter is identifiable at thirty meters. Any weapon is identifiable by silhouette and by sound alone.
- **Feedback.** Every hit has three signals: visual, audio, and HUD. Every frag has a callout.
- **Drama.** At least one Host or killfeed beat every thirty seconds of play. Round end has a podium. Warmup has a countdown.
- **Agency.** Bots visibly change behavior and react to the player. Agent joins are announced. Behavior chips are never lies.
- **Watchable.** The spectator camera never stares at nothing. The killfeed is legible from a couch.
- **Sticky.** The next round starts without a menu. Leaving to spectate never ends the match.
- **No slop.** No overlapping HUD text, no placeholder sprites in a shipped screenshot, no mood art labeled as gameplay.

## How work moves

- Every item gets a plan in `docs/plans/` before code (goal, non-goals, architecture impact, protocol changes, verification, spend gate, success criteria).
- A change ships through a branch and a PR that passes CI, then a squash merge to `main` and a tag when it changes what a player sees.
- Shipping an item means updating this file, the plan index, and any doc the change made stale, in the same PR.
- Anything that bills money stops for written approval first. Anything that touches the wire updates [`protocol.md`](./protocol.md).

## The 1.0 bar

Version 1.0 is a promise, not a milestone count. Until every line below is proven, releases stay at 0.x no matter how much has shipped.

- **Controls feel buttery.** First-person movement and aim with client-side prediction and server reconciliation, interpolation on every other fighter, no rubber-banding on a LAN or a good connection, input latency under fifty milliseconds on a LAN, sixty frames per second at 1080p on a modest machine with a full server. Mouse and gamepad both tuned. All of it measured by the benchmark mode and printed in the release notes.
- **Validated everywhere it claims to run.** A two-machine LAN session, a public server that stays up for a week with strangers on it, agents playing through the adapter at all three tiers, the single-player campaign complete through episode one, and desktop exports for Windows, macOS, and Linux that boot to Solo Scrap on a clean machine.
- **Extremely polished.** No placeholder art anywhere: every weapon, fighter, map surface, and HUD element final; the radio, effects, and Host voice complete; onboarding to a fight in under a minute with no docs; a twenty-four hour soak with no crash; the fun bar passing on a recorded session; docs and hosting guides current.
- **Hardened and honest.** The exposed-server phase complete (join tokens, rate and size caps, protocol versioning, reconnect resume, status endpoint), the playtest harness thresholds tightened to the shipped feel, no known bugs that lose a round, and a changelog that matches the releases.
