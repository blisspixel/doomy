# Plans

Durable plans live here (or as `docs/SLICE-*.md`) **before** implementation starts.

Each plan should cover: goal, non-goals, architecture impact, protocol or API changes, verification, spend/safety gates, and success criteria.

Do not treat chat as the plan of record.

## Tip priorities (after v0.5.0)

Named agents / SKILL.md shipped in **v0.5.0**. Honest coverage lock shipped (PR #57). Agent aim + hit-confirm shipped. Mode fantasy / Contested Frequency shipped. Tip screenshots shipped (#60). Off-tick speak / taunt shipped (incl. speak rate-limit isError). Sticky host_line on Snapshot shipped (#63, Casino-cleared on MCP). Godot Host flash mid-join shipped (#64). Ship order from tip:

1. **MCP session tools** - First-class `join` / `leave` / `round_state` (not lifecycle side effects only). See [`mcp-session-tools.md`](./mcp-session-tools.md).
2. **Pixel-3D look bar** (NEXT after this PR) - Unreal+CS arena play feel; 3D Godot world/camera; retro pixel surfaces/sprites/HUD; SP boot-and-scrap first-class alongside watch-or-join MP; no Doom/id IP. Does not block MCP session tools.

Shipped recently:

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
| [`mcp-session-tools.md`](./mcp-session-tools.md) | **in flight** (this tip) | First-class MCP join / leave / round_state tools + tip gallery stills. |
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
