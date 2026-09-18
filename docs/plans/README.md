# Plans

Durable plans live here (or as `docs/SLICE-*.md`) **before** implementation starts.

Each plan should cover: goal, non-goals, architecture impact, protocol or API changes, verification, spend/safety gates, and success criteria.

Do not treat chat as the plan of record.

## Tip priorities (after v0.5.0)

Named agents / SKILL.md shipped in **v0.5.0**. Honest coverage lock shipped (PR #57). Agent aim + hit-confirm shipped. Mode fantasy / Contested Frequency shipped. Tip screenshots shipped (#60). Off-tick speak / taunt shipped (incl. speak rate-limit isError). Sticky host_line on Snapshot shipped (#63, Casino-cleared on MCP). Godot Host flash mid-join shipped (#64). MCP session tools shipped (#65). Pixel-3D look bar sealed on tip (`5d16f32`, #66). Far-cam fighter scale shipped (#67). Solo boot-and-scrap shipped (#68). Continuance SP boss shipped (#69). Weapon pickups shipped (#70). Health pads shipped (#71). Arena choke geometry shipped (#72). Reconnect + Godot null shipped (#73). Human join FP juice shipped (#74). Ship order from tip:

1. **Killstreak Host juice** (NOW) - Multi-kill Host callouts + HUD flash at streak 2/3/5. See [`killstreak-host-juice.md`](./killstreak-host-juice.md).
2. (Shipped) **Human join FP juice** - Join FP scrap juice: crosshair, weapon face/bob, spawn/damage flash. See [`human-join-fp-juice.md`](./human-join-fp-juice.md).
3. (Shipped) **Reconnect + Godot null** - Clean join/leave/reconnect without ghost players; Godot add_child null guard. See [`reconnect-and-godot-null.md`](./reconnect-and-godot-null.md).
4. (Shipped) **Arena choke geometry** - Scrap low walls, crate clusters, and pillar cover with authoritative collision. See [`arena-choke-geometry.md`](./arena-choke-geometry.md).
5. (Shipped) **Health pads** - Mid-arena health and light armor pads for Quake chase sustain. See [`health-pads.md`](./health-pads.md).

Shipped recently:

- **Human join FP juice** (#74) - Join FP scrap juice: crosshair, weapon face/bob, spawn/damage flash. See [`human-join-fp-juice.md`](./human-join-fp-juice.md).
- **Reconnect + Godot null** (#73) - Clean reconnect path + Godot add_child null guard. See [`reconnect-and-godot-null.md`](./reconnect-and-godot-null.md).
- **Arena choke geometry** (#72) - Scrap choke solids + Quake slide/hitscan cover. See [`arena-choke-geometry.md`](./arena-choke-geometry.md).
- **Health pads** (#71) - Mid-arena health and armor pads for Quake chase sustain. See [`health-pads.md`](./health-pads.md).
- **Weapon pickups** (#70) - Mid-map weapon pads for Quake chase energy. See [`weapon-pickups.md`](./weapon-pickups.md).
- **Continuance SP boss** (#69) - Mid-round Continuance Compliance Drone for Solo Scrap. See [`continuance-sp-boss.md`](./continuance-sp-boss.md).
- **Solo boot-and-scrap** (#68) - SP first-class solo boot-and-scrap vs local rule bots. See [`solo-boot-and-scrap.md`](./solo-boot-and-scrap.md).
- **Pixel-3D look bar** (#66) - Scrap materials, billboard fighters, muted zone palette on tip face. See [`pixel-3d-look-bar.md`](./pixel-3d-look-bar.md).
- **Godot Host flash mid-join** (#64) - First mid-join Active Snapshot flashes Host chrome once. See [`godot-host-flash-mid-join.md`](./godot-host-flash-mid-join.md).
- **Sticky host_line mid-join** (#63) - Snapshot always carries current Host line for mid-join / mid-round observe. See [`sticky-host-line-mid-join.md`](./sticky-host-line-mid-join.md).
- **Off-tick speak / taunt** - control-plane speak + HUD/MCP events; speak rate-limit returns MCP isError. See [`offtick-speak-taunt.md`](./offtick-speak-taunt.md).
- **Tip screenshots** (#60) - live Godot tip HUD captures; MOOD-ART-ONLY cleared when stills are real. See [`tip-screenshots.md`](./tip-screenshots.md).
- **Mode fantasy / Contested Frequency** - named scrap-league identity on wire + HUD; Continuance compliance beat. See [`mode-fantasy-contested-frequency.md`](./mode-fantasy-contested-frequency.md).
- **Agent aim + hit-confirm** - `look_at` on Action + structured shot/hit feedback on observe/events. See [`agent-aim-hit-confirm.md`](./agent-aim-hit-confirm.md).

Port is **6767**. Public TCP+UDP 6767 for strangers/agents. Tailscale is private/dev only. Scale ladder: Always Free e2-micro then bigger/second arena. Never Cloud Run as combat tick. Infra stays plan-only until spend ACK.

**Chief dedicated-server bar:** rock-solid / secure / cheap (home LAN or small VM). Hardening (auth/session, junk-act reject, rate limits, input validation, no trust client sim). Stability (Rust tick, clean join/leave/reconnect, second-peer + public 6767). Cost ladder under $50. Canonical port 6767; scrub leftover 7777.


## Index

| Plan | Status | One-liner |
|---|---|---|
| [`killstreak-host-juice.md`](./killstreak-host-juice.md) | **in flight** | Multi-kill Host callouts + HUD flash at streak 2/3/5. |
| [`human-join-fp-juice.md`](./human-join-fp-juice.md) | **shipped** (#74) | Join FP scrap juice: crosshair, weapon face/bob, spawn/damage flash. |
| [`reconnect-and-godot-null.md`](./reconnect-and-godot-null.md) | **shipped** (#73) | Clean reconnect (no ghost) + Godot add_child null guard on pads. |
| [`arena-choke-geometry.md`](./arena-choke-geometry.md) | **shipped** (#72) | Scrap arena choke geometry: low walls, crates, pillar cover + server collision. |
| [`health-pads.md`](./health-pads.md) | **shipped** (#71) | Mid-arena health and light armor pads for Quake chase sustain. |
| [`weapon-pickups.md`](./weapon-pickups.md) | **shipped** (#70) | Mid-map weapon pickups for Quake/Unreal chase energy. |
| [`continuance-sp-boss.md`](./continuance-sp-boss.md) | **shipped** (#69) | Mid-round Continuance Compliance Drone boss beat for Solo Scrap. |
| [`solo-boot-and-scrap.md`](./solo-boot-and-scrap.md) | **shipped** (#68) | SP first-class solo boot-and-scrap vs local rule bots. |
| [`far-cam-fighter-scale.md`](./far-cam-fighter-scale.md) | **shipped** (#67) | Distance-aware fighter billboard scale for far spectators. |
| [`pixel-3d-look-bar.md`](./pixel-3d-look-bar.md) | **shipped** (#66) | Pixel-3D scrap-league look bar: materials, billboards, lighting, tip stills. |
| [`mcp-session-tools.md`](./mcp-session-tools.md) | **shipped** (#65) | First-class MCP join / leave / round_state tools + tip gallery stills. |
| [`godot-host-flash-mid-join.md`](./godot-host-flash-mid-join.md) | **shipped** (#64) | Godot Host bumper flash on first mid-round Snapshot. |
| [`sticky-host-line-mid-join.md`](./sticky-host-line-mid-join.md) | **shipped** (#63) | Sticky Snapshot host_line for mid-join Host chrome. |
| [`offtick-speak-taunt.md`](./offtick-speak-taunt.md) | **shipped** | Off-tick speak/taunt for agents; spectators + MCP events. |
| [`mode-fantasy-contested-frequency.md`](./mode-fantasy-contested-frequency.md) | **shipped** | Contested Frequency named mode + Continuance compliance ping. |
| [`agent-aim-hit-confirm.md`](./agent-aim-hit-confirm.md) | **shipped** | look_at + shot_results/hit observe feedback. |
| [`fragr-exceptional-game.md`](./fragr-exceptional-game.md) | **in flight** (finish-line roadmap) | Raised bar: exceptional agentic FPS people play and watch. |
| [`testy-playtest-fixes.md`](./testy-playtest-fixes.md) | **shipped** (join/leave) + **in flight** (junk-act this PR) | Casino soft prisons: join/leave on 2f38625; junk-act reject this PR. |
| [`slice1-exceptional-polish.md`](./slice1-exceptional-polish.md) | **shipped** | Slice 1 polish foundation (PR #1 era); superseded as active cut by finish-line plan. |
| [`fun-playable-pass.md`](./fun-playable-pass.md) | **shipped** / largely landed | Fun face, sprites, match drama toward v0.2-v0.4. |
| [`weapons-system.md`](./weapons-system.md) | **shipped** (v0.3.0) | Flechette / rail / scatter roles. |
| [`client-assets-wiring.md`](./client-assets-wiring.md) | **shipped** | Pixel assets wired into Godot client. |
| [`audio-drama.md`](./audio-drama.md) | **shipped** / CC0 path | Match audio drama with free/procedural audio. |
| [`terraform-zero-cost-gcp.md`](./terraform-zero-cost-gcp.md) | **shipped** (plan-only, PR #5) | Zero-cost GCP IaC; no apply without approval. |
| [`tip-screenshots.md`](./tip-screenshots.md) | **shipped** (#60) | Tip screenshot capture (Xvfb, Viewport API). |
| [`honest-coverage-lock.md`](./honest-coverage-lock.md) | **shipped** | Unfiltered llvm-cov fail-under 80; no carve-outs. |
| [`SPRINT-24H.md`](./SPRINT-24H.md) | **superseded** | 24h sprint framing; use finish-line roadmap instead. |
| [`elevenlabs-later.md`](./elevenlabs-later.md) | **HOLD** | Paid ElevenLabs voice; gated on Nick spend approval. |
