# Plan: fragr exceptional full game (finish line)

**Repo:** https://github.com/blisspixel/fragr  
**Spend:** $0 until Nick/Chief approve any cloud or paid asset. Hard cap $50 if approved.  
**Status:** Plan of record for the raised finish line (not "first solid slice").

## Done means

Exceptional, max-fun, full-enough game people would play and watch:

1. Agentic-first FPS arena with guns: BYO AI, humans play, solo run, spectator/social watch. Human vs agents is the question.
2. Real multiplayer game loop (not a tech demo).
3. Godot client + Rust authoritative server + agent-adapter.
4. Rust server deployable as IaC on GCP (real path in `infra/`; apply only after spend approval).
5. One lean main CI green (Linux: fmt, clippy -D warnings, test, build).
6. Docs with screenshots when there is something to show.
7. Do not brand as Doom.

Working name: **fragr** (may change).

## Foundation in flight (Slice 1)

PR https://github.com/blisspixel/fragr/pull/1 plus `docs/plans/slice1-exceptional-polish.md`:

- Fix player_id join bug; combat tests; interpolation; intent chips
- Join / die / respawn <=5s / leave to spectate
- >=4 rule bots; spectator default
- Second peer spectate via LAN or Tailscale Personal ($0)
- Honest checklists and stranger-bootable README

Slice 1 is required foundation. It is not the finish line.

## Vertical slices after Slice 1 (order)

| Slice | Outcome | Spend |
|---|---|---|
| 1b | Second-peer + Tailscale docs proven; screenshots in README | $0 |
| 2 | Arena feel pass: audio (CC0/free only), readable FX, score/round loop, empty-server never (bots persist) | $0 |
| 3 | Agent adapter hardening: session lifecycle, summaries @ low Hz, clawbot smoke without LLM spend | $0 |
| 4 | `infra/` GCP IaC (Cloud Run or GCE + firewall + secrets pattern) documented + `terraform plan` dry-run; **no apply** until approval | $0 until approved |
| 5 | UDP/renet spike if WS feel is the limiter; keep Godot as presenter | $0 |
| 6 | Match flow people replay: warm-up, frag limit or timed round, spectate between lives | $0 |

Testy only when stranger-playable / published face exists.

## GCP IaC (real path, fail-closed on spend)

- Add `infra/` with Terraform (or equivalent) targeting Nick personal GCP (`blisspixel` / nick@pueo.io lane).
- Document: what resources, estimated monthly cost, how to plan vs apply.
- Default workflows stop at `terraform plan` / validation. Never `apply` without written Nick/Chief OK in that session.
- Prefer smallest always-free-adjacent or cheapest VM/Cloud Run shape; still ask before first billable create.

## CI

Gitty owns one lean main workflow. Buildy keeps the tree green against it.

## Anti-patterns (kill)

Boring bots, opaque agency, empty server when humans leave, LLM on the gunfight tick, separate human-only mode, Doom branding, pitch-deck docs without a playable loop.

## Verification culture

AGENTS.md commands + evidence. Screenshots when UI exists. Green CI is the floor.

## Immediate next actions

1. Land Slice 1 polish PR (P0 + feel + second peer).
2. Wire CI with Gitty; keep main green.
3. Commit `docs/plans/fragr-exceptional-game.md` and update AGENTS.md spine (this file).
4. Scaffold `infra/` README + empty module stubs after Slice 1b (plan before apply).

## Gemini DR fold (2026-09-17)

Source: Researcher Gemini DR + QUALITY. Deepens DIY; does not replace Slice 1 cut.

**HOLD (acting on):**
- Rust authority + Godot I/O/spectator; same input path humans/agents
- Spectator-default + anti-slop feel (momentum, distinct weapon roles, polish-as-you-go, readable arenas)
- Client snapshot interpolation (already in Slice 1 polish plan)
- renet/UDP as next spike after WS JSON (not mid-PR)
- MCP off tick; $0-first LAN/Tailscale
- gdext hazards if/when we bind Rust into Godot (bind_mut, PackedByteArray CoW, experimental-threads): spike carefully later

**THROW (ignore for now):**
- Esports-indistinguishable AI claims
- Matchmaking / renetcode for Slice 1
- bitcode-over-bincode without bench
- rapier perfect determinism guarantees
- Concurrent-player/$ claims until measured
- LLM overseer blocking Slice 1

Process still wins: finish WS JSON polish PR, then renet spike under finish-line plan.


## Product feel (Nick 2026-09-17)

See `docs/VISION.md`. Retro pixel networked FPS fun (guns, maps, unlocks over time; Quake/Unreal/Halo arena chaos). Agentic let's-play is the new hook. Multiplayer-first: many agents play; humans watch or join. More fun with multiplayer than alone. No Doom branding.
