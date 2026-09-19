# Plans

Bounded work lives here as one file per item, written **before** implementation starts. Sequencing across items lives in [`../ROADMAP.md`](../ROADMAP.md). Chat is never the plan of record.

Each plan covers: goal, non-goals, architecture impact, protocol or API changes, verification, spend and safety gates, and success criteria. When the work ships, update the plan's status here and in the roadmap in the same PR.

Status words: **planned**, **in flight**, **shipped** (merged to `main`, PR number noted), **proven** (shipped plus evidence), **superseded** (kept for history, no longer directs work).

## Standing facts

- Game port is **6767** (TCP today; UDP reserved for the planned low-latency transport).
- Local play is $0. Public hosting sits under the $50 cap and needs written approval before anything bills. GCP Terraform stays plan-only until then.
- Tailscale is private smoke only, never the documented join path.
- The dedicated server bar is rock solid, secure, and cheap: input validation, rate limits, clean join and leave and reconnect, and a home box or small VM first.

## Index

| Plan | Status | One-liner |
|---|---|---|
| [`solo-story-episodes.md`](./solo-story-episodes.md) | **shipped** (#106) | Solo Broadcast Episode 0 Calibration / Larak Lot face + juice bar. |
| [`tip-stills-ep0.md`](./tip-stills-ep0.md) | **shipped** (#110) | Recapture tip stills + README Solo Broadcast face after Episode 0. |
| [`ep0-nods-progress-fix.md`](./ep0-nods-progress-fix.md) | **in flight** | Calibration NODS credit for meatbags, jammer dish silhouette, map_name honesty. |
| [`massive-arenas.md`](./massive-arenas.md) | **planned** | Seeded sim, spatial grid, interest sets, delta snapshots, binary wire, tick budget, and the measured scale ladder to hundreds of fighters. |
| [`visual-qa-tour.md`](./visual-qa-tour.md) | **planned** | Manifest-driven tour of every player-facing state with stills, a contact sheet, and feel probes for the agent developer to critique. |
| [`buttery-controls.md`](./buttery-controls.md) | **planned** | Client-owned yaw, prediction and reconciliation, timeline interpolation, 60 Hz sim, lag compensation, gamepad curves, transport spike, all with pass numbers. |
| [`public-server-hardening.md`](./public-server-hardening.md) | **planned** | Frame caps, accept limits, HMAC join tickets, name rules, Caddy TLS, status endpoint and benchmark mode. |
| [`agent-door-2026.md`](./agent-door-2026.md) | **planned** | MCP 2026-07-28 compliance with legacy clients kept, the rmcp decision, a team blackboard before A2A. |
| [`decision-brain.md`](./decision-brain.md) | **shipped** (#102) | Decision-brain agent: Jev (TypeSafe or OpenRouter) sets intent a few times a second, local controller plays every tick, hard spend cap with a ledger. |
| [`brain-third-tier-surface.md`](./brain-third-tier-surface.md) | **shipped** (#103) | Surface fragr-brain beside rule bots and MCP agents in the skill card and README, plus the observe-only stance chip. |
| [`radio-stations.md`](./radio-stations.md) | **shipped** (library) | Contested Frequency radio: eight stations, generated library, client player with ducking. |
| [`look-pass-boomer.md`](./look-pass-boomer.md) | **planned** | Boomer shooter look pass: low-res render target, dither, atlas, eight-direction sprites, weapon view models, HUD grid. |
| [`campaign-continuance.md`](./campaign-continuance.md) | **planned** | Single-player campaign at the Doom 1 and 2 bar: arcade ladder, ten-type enemy roster, map format, three episodes. |
| [`agent-playtest-loop.md`](./agent-playtest-loop.md) | **in flight** (rung 1 shipped, #95) | Playtest harness: scripted agents play rounds and file a metrics report; thresholds run in CI. |
| [`warmup-tv-bumper.md`](./warmup-tv-bumper.md) | **shipped** (#89) | Full-frame Warmup TV bumper: map title, roster chips, countdown, Host flash lingering into Active. |
| [`controller-and-desktop-platforms.md`](./controller-and-desktop-platforms.md) | **shipped** (#88) | Gamepad join, solo, and match input on the same action path; Windows, macOS, and Linux export presets. |
| [`bug-hunt-polish-pass.md`](./bug-hunt-polish-pass.md) | **shipped** (#90) | Tip feel polish: boss down round-end wipe, layout null guard, spectator cam validity, net send hardening. |
| [`l5-vs-nods-why-fight.md`](./l5-vs-nods-why-fight.md) | **shipped** (#87) | Solo Broadcast and L5 versus NODS: the why-fight spine folded into LORE and VISION. |
| [`arty-gold-face-pack.md`](./arty-gold-face-pack.md) | **shipped** (#86) | Gold face pack: Cyanex and Kragge gold idle billboards plus broadcast HUD chrome. |
| [`rule-bot-taunts.md`](./rule-bot-taunts.md) | **shipped** (#85) | Named rule bots speak Contested Frequency scrap-radio taunts. |
| [`public-or-local-join.md`](./public-or-local-join.md) | **shipped** (#84) | Docs scrub: public OR local join; Tailscale private only; spend honesty. |
| [`warmup-host-drama.md`](./warmup-host-drama.md) | **shipped** (#83) | Warmup and pre-round Host countdown drama (roster, map, bumper). |
| [`named-scrap-bots.md`](./named-scrap-bots.md) | **shipped** (#82) | Contested Frequency named rule-bot roster and Host intros. |
| [`ended-linger-mvp-rehydrate.md`](./ended-linger-mvp-rehydrate.md) | **shipped** (#81) | Ended linger plus mid-join structured MVP rehydrate. |
| [`second-scrap-map.md`](./second-scrap-map.md) | **shipped** (#80) | Second Contested Frequency scrap map (Compliance Yard). |
| [`round-end-mvp-drama.md`](./round-end-mvp-drama.md) | **shipped** (#79) | Round-end MVP and podium Host drama. |
| [`coverage-climb-main.md`](./coverage-climb-main.md) | **shipped** (#78) | Climb unfiltered coverage by testing server and adapter main shells. |
| [`tip-stills-recapture.md`](./tip-stills-recapture.md) | **shipped** (#77) | Recapture tip stills after weapon roles. |
| [`weapon-roles-excellence.md`](./weapon-roles-excellence.md) | **shipped** (#76) | Flechette, Rail, and Scatter roles that read in the hand. |
| [`killstreak-host-juice.md`](./killstreak-host-juice.md) | **shipped** (#75) | Multi-kill Host callouts and HUD flash at streak 2, 3, 5. |
| [`human-join-fp-juice.md`](./human-join-fp-juice.md) | **shipped** (#74) | Join first-person juice: crosshair, weapon face and bob, spawn and damage flash. |
| [`reconnect-and-godot-null.md`](./reconnect-and-godot-null.md) | **shipped** (#73) | Clean reconnect (no ghost) plus Godot add_child null guard on pads. |
| [`arena-choke-geometry.md`](./arena-choke-geometry.md) | **shipped** (#72) | Arena choke geometry: low walls, crates, pillar cover, server collision. |
| [`health-pads.md`](./health-pads.md) | **shipped** (#71) | Mid-arena health and light armor pads. |
| [`weapon-pickups.md`](./weapon-pickups.md) | **shipped** (#70) | Mid-map weapon pickups for chase energy. |
| [`continuance-sp-boss.md`](./continuance-sp-boss.md) | **shipped** (#69) | Mid-round Compliance Drone boss beat for Solo Scrap. |
| [`solo-boot-and-scrap.md`](./solo-boot-and-scrap.md) | **shipped** (#68) | Single-player first-class solo boot-and-scrap versus local rule bots. |
| [`far-cam-fighter-scale.md`](./far-cam-fighter-scale.md) | **shipped** (#67) | Distance-aware fighter billboard scale for far spectators. |
| [`pixel-3d-look-bar.md`](./pixel-3d-look-bar.md) | **shipped** (#66) | Pixel-3D look bar: materials, billboards, lighting, tip stills. |
| [`mcp-session-tools.md`](./mcp-session-tools.md) | **shipped** (#65) | First-class MCP join, leave, and round_state tools. |
| [`godot-host-flash-mid-join.md`](./godot-host-flash-mid-join.md) | **shipped** (#64) | Godot Host bumper flash on first mid-round Snapshot. |
| [`sticky-host-line-mid-join.md`](./sticky-host-line-mid-join.md) | **shipped** (#63) | Sticky Snapshot host_line for mid-join Host chrome. |
| [`offtick-speak-taunt.md`](./offtick-speak-taunt.md) | **shipped** | Off-tick speak and taunt for agents; spectators and MCP events. |
| [`mode-fantasy-contested-frequency.md`](./mode-fantasy-contested-frequency.md) | **shipped** | Contested Frequency named mode plus Continuance compliance ping. |
| [`agent-aim-hit-confirm.md`](./agent-aim-hit-confirm.md) | **shipped** | look_at plus shot_results and hit observe feedback. |
| [`testy-playtest-fixes.md`](./testy-playtest-fixes.md) | **shipped** | Join and leave events plus junk-act reject. |
| [`fragr-exceptional-game.md`](./fragr-exceptional-game.md) | **superseded** by `../ROADMAP.md` | Earlier finish-line framing; kept for the decisions it records. |
| [`slice1-exceptional-polish.md`](./slice1-exceptional-polish.md) | **shipped** | Slice 1 polish foundation (PR #1 era). |
| [`fun-playable-pass.md`](./fun-playable-pass.md) | **shipped** | Fun face, sprites, match drama toward v0.2 to v0.4. |
| [`weapons-system.md`](./weapons-system.md) | **shipped** (v0.3.0) | Flechette, rail, and scatter roles. |
| [`client-assets-wiring.md`](./client-assets-wiring.md) | **shipped** | Pixel assets wired into the Godot client. |
| [`audio-drama.md`](./audio-drama.md) | **shipped** | Match audio drama with procedural CC0 audio. |
| [`terraform-zero-cost-gcp.md`](./terraform-zero-cost-gcp.md) | **shipped** (plan-only, PR #5) | Zero-cost GCP IaC; no apply without approval. |
| [`tip-screenshots.md`](./tip-screenshots.md) | **shipped** (#60) | Tip screenshot capture (Xvfb, Viewport API). |
| [`honest-coverage-lock.md`](./honest-coverage-lock.md) | **shipped** | Unfiltered llvm-cov fail-under 80; no carve-outs. |
| [`SPRINT-24H.md`](./SPRINT-24H.md) | **superseded** | 24 hour sprint framing. |
| [`elevenlabs-later.md`](./elevenlabs-later.md) | **superseded** by `tools/audiogen` | Spend approved 2026-09-18 for developer-side generation only; see `tools/audiogen/README.md`. |
