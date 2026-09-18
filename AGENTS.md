# AGENTS.md — Doomy

Guidance for coding agents working in this repository. Humans: start with `README.md` and `docs/`.

## What this is

**Doomy** (working name) is a modern Doom-like built as a monorepo:

- `client/` — Godot **4.7.2-stable**, GDScript only. Thin presenter (render, audio, spectator UI, input). Not sim authority.
- `server/` — Rust authoritative game server (tokio). Owns tick, combat, spawns, scoring, server-side rule bots.
- `agent-adapter/` — slow control plane for clawbots / MCP-style tooling (`observe` / `act` / join / goals). Not the combat tick.
- `docs/` — architecture, protocol, slice checklists.

**Product spine:** default human mode is **spectator** (watch agents fight). Humans can **join the same match** and leave back to spectate. Feel like a community server with continuous drama, not a pitch deck.

**Lane:** personal `blisspixel` / Nick only. No work accounts. No Spark/DGX.

## Truth ranking

1. Source, tests, manifests, lockfiles, `git` history  
2. `docs/protocol.md` for on-wire shapes once implemented  
3. `docs/ARCHITECTURE.md` and `docs/SLICE-1.md` for intent and slice bars  
4. This file for agent operating rules  

If prose and code disagree, **code wins**. Update the prose in the same change when behavior materially moves.

Distinguish: vision / planned / implemented / tested / shipped / proven. Do not treat a checklist item as done without evidence.

## Hard constraints (project law)

- **Spend:** hard cap **$50** total for cloud/API/hosting/assets. Slice 1 and normal iteration are **$0** (loopback / LAN). Any money needs Nick or Chief approval **before** purchase. Prefer home-host, then Tailscale Personal ($0), then Oracle Always Free if needed. Paid VPS is an approval gate.
- **Authority:** Rust server is source of truth for positions, damage, HP, frags. Godot never decides combat outcomes.
- **MCP / LLM off the hot path:** agents and humans share the same discrete action channel into the server. Scripted/utility AI runs at tick rate on the server (or via adapter-injected intents). MCP/JSON-RPC is for slow ops (join, summaries, goals), never aim/fire at 20–60 Hz. No paid model APIs without approval.
- **Transport (Slice 1):** WebSocket JSON on `127.0.0.1:7777`. UDP/`renet` is a later spike, not a silent mid-slice rewrite unless Nick asks.
- **Client pin:** Godot **4.7.2-stable**, GDScript only (no .NET export template for Slice 1).
- **Dependencies:** minimal and intentional. Prefer std / existing crates. No Bevy client, no lightyear (Bevy-centric), no second HTTP client / logger / serializer without consolidating.
- **Secrets:** none required for local play. Never commit credentials. Temporary agent scratch goes in gitignored `.agents/` only.

## Canonical seams

| Concern | Home |
|---|---|
| Sim tick, hit detection, bots | `server/` |
| Wire protocol messages | shared types in `server` (and mirrored docs in `docs/protocol.md`); client/adapter speak that schema |
| Presentation / cameras / HUD | `client/` |
| External agent tooling | `agent-adapter/` |
| Product / stack decisions | `docs/ARCHITECTURE.md` |
| Current vertical slice DoD | `docs/SLICE-1.md` |

Before adding a second way to log, configure, serialize, or talk to the server, search the tree and reuse the existing seam.

## Verification (run before claiming done)

Commands must match the repo as it exists. If a directory is missing, scaffold it first or skip that row.

**Rust (`server/`, `agent-adapter/` when present):**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

Do not “fix” Clippy by broad `#[allow]`, silencing warnings workspace-wide, or deleting checks. Narrow, justified allows only.

**Godot (`client/` when present):**

```bash
# Prefer a 4.7.2-stable editor binary named `godot` or pass the full path.
godot --headless --path client --import
godot --headless --path client --check-only --script res://path/to/script.gd
```

Godot may exit `0` even when the log contains `SCRIPT ERROR` / `Parse Error`. Treat log contents as the verifier. A stranger must be able to open `client/` in 4.7.2 without missing `project.godot` fields.

**Playable smoke (Slice 1 bar):**

1. `cargo run -p doomy-server` (or `cd server && cargo run`) listens on `127.0.0.1:7777` with no cloud env.  
2. Bots fight; damage/frag within ~30s in server logs.  
3. Godot spectator shows the match (presentation only).  
4. Optional: human join + leave-to-spectate; adapter `observe`/`act` for at least one pawn.

Evidence beats assertion. Prefer a real smoke run over “should work.”

## Working style

- Prefer a smaller **exceptional** playable over a larger half-built scaffold.
- Fix root causes; do not weaken the checker that caught the failure.
- After meaningful work, update the durable artifacts the change makes true: tests, `docs/protocol.md`, README run steps, slice checklist boxes, this file if constraints moved.
- **No tool or model attribution anywhere:** no "Generated by", "Written with", "Co-Authored-By" for Cursor, Claude, Codex, ChatGPT, Gemini, Grok, or any other assistant/model; no AI coauthor trailers; no Cursor/agent PR footers or HTML badges; no "made with AI" notes in README, docs, comments, commits, or PR text. Name products only when documenting a runtime integration (e.g. an MCP client), never as authors.
- No emoji. No em dashes or en dashes in repo prose, commits, or PR text.
- Comments explain intent, invariants, and tradeoffs, not obvious narration.
- Research current docs for Godot/Rust crates when versions or APIs may have changed; do not trust memory alone for release pins.

## Out of scope unless Nick asks

Public hosting, paid assets, browser client, prediction/rollback polish, multiple maps, CI ownership (coordinate with Gitty when CI is wanted), renaming off “Doomy,” Bevy rewrite.

## Nested guidance

If a package later needs tighter rules, add a nested `AGENTS.md` there. Closest file wins for local detail; root constraints above still apply.
