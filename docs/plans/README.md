# Plans

Durable plans live here (or as `docs/SLICE-*.md`) **before** implementation starts.

Each plan should cover: goal, non-goals, architecture impact, protocol or API changes, verification, spend/safety gates, and success criteria.

Do not treat chat as the plan of record.

## Tip priorities (after d3cbc9c + junk-act/HUD PR; releases through v0.4.0)

1. **Spectator face juice** - playable watch experience, Host bumpers, readable HUD (round_label restored this PR).
2. **One mode fantasy** - lean into a single arena fantasy; muted accents OK for later art swap (not neon everywhere).
3. **Amazement bar** - fun to watch and join; not an MCP cell farm.

Port is **6767**. Public TCP+UDP 6767 for strangers/agents. Tailscale is private/dev only. Scale ladder: Always Free e2-micro then bigger/second arena. Never Cloud Run as combat tick. Infra stays plan-only until spend ACK.

**Chief dedicated-server bar:** rock-solid / secure / cheap (home LAN or small VM). Hardening (auth/session, junk-act reject this PR, rate limits, input validation, no trust client sim). Stability (Rust tick, clean join/leave/reconnect, second-peer + public 6767). Cost ladder under $50. Canonical port 6767; scrub leftover 7777.

## Index

| Plan | Status | One-liner |
|---|---|---|
| [`fragr-exceptional-game.md`](./fragr-exceptional-game.md) | **in flight** (finish-line roadmap) | Raised bar: exceptional agentic FPS people play and watch. |
| [`testy-playtest-fixes.md`](./testy-playtest-fixes.md) | **shipped** (join/leave) + **in flight** (junk-act this PR) | Casino soft prisons: join/leave on 2f38625; junk-act reject this PR. |
| [`slice1-exceptional-polish.md`](./slice1-exceptional-polish.md) | **shipped** | Slice 1 polish foundation (PR #1 era); superseded as active cut by finish-line plan. |
| [`fun-playable-pass.md`](./fun-playable-pass.md) | **shipped** / largely landed | Fun face, sprites, match drama toward v0.2-v0.4. |
| [`weapons-system.md`](./weapons-system.md) | **shipped** (v0.3.0) | Flechette / rail / scatter roles. |
| [`client-assets-wiring.md`](./client-assets-wiring.md) | **shipped** | Pixel assets wired into Godot client. |
| [`audio-drama.md`](./audio-drama.md) | **shipped** / CC0 path | Match audio drama with free/procedural audio. |
| [`terraform-zero-cost-gcp.md`](./terraform-zero-cost-gcp.md) | **shipped** (plan-only, PR #5) | Zero-cost GCP IaC; no apply without approval. |
| [`tip-screenshots.md`](./tip-screenshots.md) | **in flight** | Tip screenshot capture (Xvfb, Viewport API). |
| [`SPRINT-24H.md`](./SPRINT-24H.md) | **superseded** | 24h sprint framing; use finish-line roadmap instead. |
| [`elevenlabs-later.md`](./elevenlabs-later.md) | **HOLD** | Paid ElevenLabs voice; gated on Nick spend approval. |

