# AGENTS.md - fragr

Guidance for coding agents working in this repository. Humans: start with `README.md` and `docs/`.

## What this is

**fragr** (working name) is an agentic-first FPS arena. Do not brand as Doom. Monorepo:

- `client/` - Godot **4.7.2-stable**, GDScript only. Thin presenter (render, audio, spectator UI, input). Not sim authority.
- `server/` - Rust authoritative game server (tokio). Owns tick, combat, spawns, scoring, server-side rule bots.
- `agent-adapter/` - slow control plane for clawbots / MCP-style tooling (`observe` / `act` / join / goals). Not the combat tick.
- `docs/` - architecture, protocol, slice checklists, plans, vision.
- `infra/` - native GCP IaC for cheap scale (plan-only until spend approval). Self-host / run-your-own-server is first-class; LAN/Tailscale optional.

**Product spine:** default human mode is **spectator** (watch agents fight). Humans can **join the same match** and leave back to spectate. Feel like a community server with continuous drama, not a pitch deck.

**Finish line:** exceptional full multiplayer game; run-your-own-server (Minecraft-shaped) + GCP IaC cheap scale; CI green; no Doom branding. Slice 1 is floor not finish.

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
- **MCP / LLM off the hot path:** agents and humans share the same discrete action channel into the server. Scripted/utility AI runs at tick rate on the server (or via adapter-injected intents). MCP/JSON-RPC is for slow ops (join, summaries, goals), never aim/fire at 20-60 Hz. No paid model APIs without approval.
- **Transport (Slice 1):** WebSocket JSON (default bind `0.0.0.0:7777`; clients use loopback or `FRAGR_SERVER`). UDP/`renet` is a later spike, not a silent mid-slice rewrite unless Nick asks.
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

1. `cargo run -p fragr-server` (or `cd server && cargo run`) listens on `127.0.0.1:7777` with no cloud env. 
2. Bots fight; damage/frag within ~30s in server logs. 
3. Godot spectator shows the match (presentation only). 
4. Optional: human join + leave-to-spectate; adapter `observe`/`act` for at least one pawn.

Evidence beats assertion. Prefer a real smoke run over “should work.”


## Research and plan before build

Do not start implementation until durable plan docs exist for the change.

1. **Orient** in the real repo (`README.md`, `docs/`, source, tests, git history). Code and manifests outrank stale prose.
2. **Research** current primary sources for the stack actually in use (Godot 4.7.2 docs/releases, Rust crates, MCP spec). Do not trust model memory for pins, flags, or APIs that can change.
3. **Write the plan** into tracked docs before coding: goal, non-goals, architecture impact, protocol or API changes, verification steps, spend/safety gates, and success criteria. Prefer updating `docs/SLICE-*.md`, `docs/ARCHITECTURE.md`, `docs/protocol.md`, or a short `docs/plans/<slug>.md` over chat-only plans.
4. **Build** only after the plan is in the tree (or an explicit Nick exception). Keep the first playable slice exceptional; do not skip docs to move faster.
5. After shipping behavior, update the same docs so the next session inherits truth.

## Working style

- Prefer a smaller **exceptional** playable over a larger half-built scaffold.
- Fix root causes; do not weaken the checker that caught the failure.
- After meaningful work, update the durable artifacts the change makes true: tests, `docs/protocol.md`, README run steps, slice checklist boxes, this file if constraints moved.
- **README screenshots must match the playable tip.** When UI elements, weapons, sprites, HUD features, or arena zones land in the client, regenerate screenshots to show what actually runs now. Prefer `docs/screenshots/` linked from README. Mood art is acceptable when clearly labeled in `docs/screenshots/README.md`, but stale mood stubs that misrepresent the current build are forbidden. Replace mood plates with live captures as soon as the feature is playable.
- **HARD LOCK - zero tool/model attribution:** no "by Claude/Codex/Cursor/Copilot/Grok", no tool Co-authored-by, no Made-with/Generated-by badges, no platform PR footers. Commits/PRs/assets look like Nick/blisspixel only. Also:
- **No tool or model attribution anywhere:** no "Generated by", "Written with", or assistant/model coauthor trailers for Cursor, Claude, Codex, ChatGPT, Gemini, Grok, or any other tool; no Cursor/agent PR footers or HTML badges; no "made with AI" notes in README, docs, comments, commits, or PR text. Name products only when documenting a runtime integration, never as authors.
- **No emoji** in repo prose, comments, commits, PR titles, or PR bodies.
- **No em dashes or en dashes.** Use commas, periods, parentheses, colons, or hyphens in compound adjectives only when needed. Rewrite sentences instead of using long dashes.

- Comments explain intent, invariants, and tradeoffs, not obvious narration.
- Research current docs for Godot/Rust crates when versions or APIs may have changed; do not trust memory alone for release pins.

## Out of scope unless Nick asks

Paid cloud apply without approval, paid assets, browser client, Bevy rewrite. CI ownership stays with Gitty when asked. Product rename may change "fragr" later; do not brand as Doom.

## Nested guidance

If a package later needs tighter rules, add a nested `AGENTS.md` there. Closest file wins for local detail; root constraints above still apply.
