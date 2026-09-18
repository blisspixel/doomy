# Plans

Durable plans live here (or as `docs/SLICE-*.md`) **before** implementation starts.

Each plan should cover: goal, non-goals, architecture impact, protocol or API changes, verification, spend/safety gates, and success criteria.

Do not treat chat as the plan of record.

## Tip priorities (after v0.5.0)

Named agents / SKILL.md shipped in **v0.5.0**. Honest coverage lock shipped (PR #57). Agent aim + hit-confirm shipped (look_at + shot_results/hit). Ship order from tip:

1. **Mode fantasy / Contested Frequency** - named scrap-league identity on wire + HUD; one Continuance pressure beat; not an MCP cell farm. See [`mode-fantasy-contested-frequency.md`](./mode-fantasy-contested-frequency.md).
2. **Off-tick speak / taunt** - only after the adapter door stays trusted; LLM stays off the combat tick.
3. **Tip screenshots** - replace mood README embeds with live Godot captures; clear MOOD-ART-ONLY banner when stills are real. See [`tip-screenshots.md`](./tip-screenshots.md).
4. **Pixel-3D look bar** - Unreal+CS arena play feel; 3D Godot world/camera; retro pixel surfaces/sprites/HUD; SP boot-and-scrap first-class alongside watch-or-join MP; no Doom/id IP.

Shipped recently:

- **Agent aim + hit-confirm** - `look_at` on Action + structured shot/hit feedback on observe/events. See [`agent-aim-hit-confirm.md`](./agent-aim-hit-confirm.md).

Port is **6767**. Public TCP+UDP 6767 for strangers/agents. Tailscale is private/dev only. Scale ladder: Always Free e2-micro then bigger/second arena. Never Cloud Run as combat tick. Infra stays plan-only until spend ACK.

**Chief dedicated-server bar:** rock-solid / secure / cheap (home LAN or small VM). Hardening (auth/session, junk-act reject, rate limits, input validation, no trust client sim). Stability (Rust tick, clean join/leave/reconnect, second-peer + public 6767). Cost ladder under $50. Canonical port 6767; scrub leftover 7777.


## Index

| Plan | Status | One-liner |
|---|---|---|
| [`mode-fantasy-contested-frequency.md`](./mode-fantasy-contested-frequency.md) | **in flight** (this tip) | Contested Frequency named mode + Continuance compliance ping. |
| [`agent-aim-hit-confirm.md`](./agent-aim-hit-confirm.md) | **shipped** | look_at + shot_results/hit observe feedback. |
| [`fragr-exceptional-game.md`](./fragr-exceptional-game.md) | **in flight** (finish-line roadmap) | Raised bar: exceptional agentic FPS people play and watch. |
| [`testy-playtest-fixes.md`](./testy-playtest-fixes.md) | **shipped** (join/leave) + **in flight** (junk-act this PR) | Casino soft prisons: join/leave on 2f38625; junk-act reject this PR. |
| [`slice1-exceptional-polish.md`](./slice1-exceptional-polish.md) | **shipped** | Slice 1 polish foundation (PR #1 era); superseded as active cut by finish-line plan. |
| [`fun-playable-pass.md`](./fun-playable-pass.md) | **shipped** / largely landed | Fun face, sprites, match drama toward v0.2-v0.4. |
| [`weapons-system.md`](./weapons-system.md) | **shipped** (v0.3.0) | Flechette / rail / scatter roles. |
| [`client-assets-wiring.md`](./client-assets-wiring.md) | **shipped** | Pixel assets wired into Godot client. |
| [`audio-drama.md`](./audio-drama.md) | **shipped** / CC0 path | Match audio drama with free/procedural audio. |
| [`terraform-zero-cost-gcp.md`](./terraform-zero-cost-gcp.md) | **shipped** (plan-only, PR #5) | Zero-cost GCP IaC; no apply without approval. |
| [`tip-screenshots.md`](./tip-screenshots.md) | **in flight** | Tip screenshot capture (Xvfb, Viewport API). |
| [`honest-coverage-lock.md`](./honest-coverage-lock.md) | **shipped** | Unfiltered llvm-cov fail-under 80; no carve-outs. |
| [`SPRINT-24H.md`](./SPRINT-24H.md) | **superseded** | 24h sprint framing; use finish-line roadmap instead. |
| [`elevenlabs-later.md`](./elevenlabs-later.md) | **HOLD** | Paid ElevenLabs voice; gated on Nick spend approval. |
