# Plan: fragr exceptional full game (finish line)

**Repo:** https://github.com/blisspixel/fragr
**Spend:** $0 until Nick/Chief approve any cloud or paid asset. Hard cap $50 if approved.
**Status:** Plan of record for the raised finish line. Tip after `d3cbc9c` (port 6767) + junk-act/HUD PR; releases through **v0.4.0**.

## Done means

Exceptional, max-fun, full-enough game people would play and watch:

1. Agentic-first FPS arena with guns: BYO AI, humans play, solo run, spectator/social watch. Human vs agents is the question.
2. Real multiplayer game loop (not a tech demo).
3. Godot client + Rust authoritative server + agent-adapter.
4. Rust server deployable as IaC on GCP (real path in `infra/`; apply only after spend approval).
5. One lean main CI green (Linux: fmt, clippy -D warnings, test, build, 80% llvm-cov).
6. Docs with screenshots when there is something to show.
7. Do not brand as Doom.

Working name: **fragr** (may change).

## Tip reality (2026-09-17 PT)

- **Port 6767** (not 7777). Documented game socket everywhere.
- **Public path:** Always Free-shaped GCE with public TCP+UDP **6767** for strangers and agents (Minecraft-shaped join). See `infra/docs/DURABLE-HOST.md`.
- **Tailscale:** private/dev and operator smoke only. Not the spectator/agent join path.
- **Scale ladder:** e2-micro first; then bigger VM or second arena when load proves it. **Never** Cloud Run / Functions / scale-to-zero as the combat tick. Infra remains **plan-only** until spend ACK.
- **Casino soft prisons:**
  - Join/leave MCP events: **cleared** on `2f38625` (survive tick for broadcast).
  - Junk-act silent success: **cleared in this PR** (`deny_unknown_fields` + MCP act allowlist / `isError`). Sticky state is not overwritten by junk.
  - Remaining soft prisons: be honest in Testy notes; do not paper over.
- Coverage floor: honest **80%** llvm-cov on tip (`81d3f1d`).

## Chief dedicated-server bar

Dedicated server must be rock-solid, secure, and cheap: home LAN **or** a small cloud VM. Not Tailscale-only. Not Cloud Run as the combat tick.

1. **Hardening**
   - Auth/session bounds (who may act; sticky session limits).
   - Junk-act reject (**this PR**): unknown Action fields fail serde; MCP `act` allowlist returns `isError`.
   - Rate limits and input validation on the wire.
   - No trust of client sim; Rust authority owns truth.

2. **Stability**
   - Rust tick authority stays always-on.
   - Clean join / leave / reconnect paths (join/leave Casino cleared on `2f38625`).
   - Second-peer + public **6767** green for strangers/agents.

3. **Cost**
   - Single-process low-RAM home LAN path first.
   - Cheap VPS / Always Free `e2-micro` ladder under the **$50** hard cap.
   - `infra/` IaC remains **plan-only** until spend ACK.

4. **Docs**
   - Canonical port **6767** everywhere.
   - Scrub leftover **7777** (mention only as "not 7777" historical note if needed).

## Researcher DIY (Clawd / Hermes / L5)

Architecture lock (matches standing throw-outs):

- **One MCP door:** agent-adapter is the only agent ingress. Adapter maps tools to the same `Action` path humans use on public **:6767**. No second control plane.
- **Skill pack:** `SKILL.md` plus Hermes / OpenClaw config snips so BYO agents can find the door.
- **LLM off tick:** model never sits on the combat tick; observe/act (and later speak) stay off-tick / control-plane.
- **L5 aspiration, not DoD:** deeper agent competence is a north star, not a ship gate.

### Ship order after junk-act / HUD PR

1. (done / this PR) Junk-act reject + round HUD face.
2. (done) Join/leave Casino path (`2f38625`).
3. **Adapter `--name` / session join-leave** - tip still hardcodes MCP Agent; agents need named sessions and clean join/leave.
4. **SKILL.md + Hermes / OpenClaw snips** - document the one door.
5. **Off-tick speak / taunt** - only after the door is trusted.

Throw-outs already match standing locks (no matchmaking gate, no esports-AI claims, no LLM overseer on the gunfight).

## Next (after this PR)

1. **Adapter `--name` / session join-leave** - tip hardcodes MCP Agent; named sessions + clean join/leave.
2. **SKILL.md + Hermes / OpenClaw snips** - one MCP door documented.
3. **Off-tick speak / taunt** - after the door is trusted (LLM still off tick).
4. **Spectator face juice / one mode fantasy / amazement bar** - fun to watch and join; not an MCP cell farm.

Art note for later (not this PR): muted accents OK; not neon puke everywhere.

## Foundation (shipped)

Slice 1 polish and follow-ons landed through v0.4.0 playable tip: weapons, assets, audio, MCP Prison observe/act, juice, port 6767, join/leave Casino path.

## Vertical slices (remaining / refresh)

| Slice | Outcome | Spend |
|---|---|---|
| Face | Spectator juice + Host voice + HUD honesty | $0 |
| Fantasy | One mode fantasy deepened | $0 |
| Amazement | Replayable round loop people watch | $0 |
| Net | UDP/renet spike if WS feel is the limiter; keep Godot as presenter | $0 |
| Host | Self-host docs + `infra/` plan-only; apply gated | $0 until approved |

Testy when stranger-playable / published face exists.

## Hosting (self-host + GCP)

- Run your own Rust server (home/VPS you control).
- Public strangers/agents: TCP+UDP **6767** on a durable VM.
- Tailscale optional for private/dev only.
- Native GCP IaC in `infra/` for cheap cloud at scale (apply gated). Combat tick stays always-on GCE (or bigger dedicated), never Cloud Run.

## GCP IaC (real path, fail-closed on spend)

- `infra/` Terraform targeting Nick personal GCP (`blisspixel` / nick@pueo.io lane).
- Default workflows stop at `terraform plan` / validation. Never `apply` without written Nick/Chief OK.
- Prefer Always Free `e2-micro` recipe in `infra/docs/DURABLE-HOST.md`.

## CI

Gitty owns one lean main workflow. Buildy keeps the tree green against it (fmt, clippy, test, build, coverage).

## Anti-patterns (kill)

Boring bots, opaque agency, empty server when humans leave, LLM on the gunfight tick, separate human-only mode, Doom branding, pitch-deck docs without a playable loop, MCP cell farms, junk act keys that silently succeed, neon-everywhere art.

## Verification culture

AGENTS.md commands + evidence. Screenshots when UI exists. Green CI is the floor.

## Gemini DR fold (still holds)

**HOLD (acting on):**
- Rust authority + Godot I/O/spectator; same input path humans/agents
- Spectator-default + anti-slop feel
- renet/UDP as next net spike after WS JSON (not mid-PR)
- MCP off tick; $0-first with public 6767 for strangers when hosted
- gdext hazards if/when we bind Rust into Godot: spike carefully later

**THROW (ignore for now):**
- Esports-indistinguishable AI claims
- Matchmaking / renetcode as a gate
- bitcode-over-bincode without bench
- rapier perfect determinism guarantees
- Concurrent-player/$ claims until measured
- LLM overseer blocking play

## Product feel

See `docs/VISION.md`. Retro pixel networked FPS fun. Agentic let's-play is the new hook. Multiplayer-first: many agents play; humans watch or join. No Doom branding.
