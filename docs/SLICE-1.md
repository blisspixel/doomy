# Slice 1 — Arena Duel Watch (build checklist)

**Goal:** Smallest exceptional playable: one arena, agent/bots fight, spectator watches, optional human join stub. **$0 spend. Loopback only.**

**Pin:** Godot `4.7.2-stable` · Rust stable · `ws://127.0.0.1:7777`

**Done when all success criteria below are green.**

---

## Success criteria

- [ ] `cargo run -p doomy-server` (or `cd server && cargo run`) listens on `127.0.0.1:7777` with no env secrets / cloud config.
- [ ] At least four automated fighters (rule bots) engage; within ~30s damage or a frag is visible in **server logs and** Godot spectator.
- [ ] Spectator is presentation-only (no local authority); killing the server drops/freezes the match cleanly.
- [ ] Agent-adapter (or documented equivalent) drives ≥1 fighter via `observe` / `act`; MCP tool surface listed in README even if partially stubbed.
- [ ] Optional human: same Godot client can join with keyboard move + shoot in the same match.
- [ ] Incremental spend: **$0**.

---

## Checklist

### 0. Scaffold (~1–2 h)
- [ ] Create monorepo folders: `server/`, `client/`, `agent-adapter/`, `docs/`
- [ ] Godot project: create with **4.7.2-stable**, GDScript, forward+/mobile OK; save under `client/`
- [ ] `server/Cargo.toml`: tokio, tokio-tungstenite, serde, serde_json, futures-util, clap, tracing
- [ ] Root `README.md`: three-terminal run instructions (placeholders OK until green)
- [ ] `docs/protocol.md`: draft JSON for `Hello`, `Action`, `Snapshot`

### 1. Wire echo (~2–3 h)
- [ ] Server accepts WebSocket; on connect expect `Hello { role, name }`
- [ ] Reply `Welcome { player_id?, role }`
- [ ] Godot `WebSocketPeer` connects; print connected + welcome in console
- [ ] Reject unknown roles; accept `spectator` | `human` | `agent`

### 2. Snapshot render (~3–4 h)
- [ ] Server tick @ **20 Hz**; maintain list of `Player { id, x, y, z, yaw, hp }`
- [ ] Spawn 2 dummy players that strafe or circle in code (no combat yet)
- [ ] Broadcast `Snapshot { tick, players[] }` each tick (or every 2nd tick)
- [ ] Godot: graybox arena mesh; spawn/update capsule (or MeshInstance) per player id
- [ ] Spectator free-fly camera (WASD + mouse look **local only**)

### 3. Combat (~3–4 h)
- [ ] Action schema: `{ forward, back, left, right, turn_left, turn_right, fire }` booleans (or small enums)
- [ ] Apply actions next tick; clamp to arena AABB
- [ ] Hitscan **or** slow projectile; on hit reduce HP; at 0 → respawn after N ticks
- [ ] Log `frag killer→victim` to stdout
- [ ] Snapshot includes hp (and optionally `just_fired` for muzzle flash stub)

### 4. Bots that fight (~2–3 h)
- [ ] Server-side bot controller **or** headless scripted WS client: chase nearest enemy + fire when roughly facing
- [ ] Start match with 2 bots automatically on server boot (CLI flag `--bots 4`)
- [ ] Verify frags happen without any LLM / MCP yet

### 5. Spectator UX (~2 h)
- [ ] Default scene = spectator (no join as human unless flag/button)
- [ ] Follow-cam option: cycle focus among living players (key `F` or similar)
- [ ] Simple HUD: “SPECTATING · tick N · players”

### 6. Agent-adapter (~3–4 h)
- [ ] Process connects as `role=agent`, owns one pawn
- [ ] Tools/API: `session_join`, `observe`, `act`, `session_leave`
- [ ] Prefer MCP stdio if quick; else HTTP/JSON with same shapes + note “MCP wrapper next”
- [ ] Scripted loop using adapter proves external drive path (even without an LLM)
- [ ] Document tool schemas in `agent-adapter/README.md`

### 7. Human join stub (~1–2 h)
- [ ] Godot “Join as human” button or `-- --human` / scene switch
- [ ] Map keys → same `Action` messages as agents
- [ ] Confirm human can take damage / frag a bot

### 8. Slice freeze (~2 h)
- [ ] README: exact commands for server / spectator / adapter (and human)
- [ ] `docs/protocol.md` matches implemented messages
- [ ] No paid deps, no cloud endpoints
- [ ] Manual 30s watch test: bots fight, spectator follows, looks “game-like” not tech-demo-only

---

## Explicitly defer (not Slice 1)

- Prediction / rollback / interpolation polish beyond simple lerp
- UDP / renet / lightyear
- LLM-backed agents or screenshot observations **[SPEND GATE]**
- Multiple maps, persistence, auth, matchmaking
- Public hosting / Tailscale packaging (nice docs ok; no paid VPS)
- Creating the GitHub remote (wait for Nick/Buildy)

---

## Smoke test script (manual)

```bash
# T1 — server (with 2 bots)
cd server && cargo run -- --bind 127.0.0.1:7777 --bots 2

# T2 — Godot spectator
# Open client/ in Godot 4.7.2 → F5 (main scene = spectator)

# T3 — optional adapter / human
cd agent-adapter && cargo run -- # or join as human in Godot UI
```

Expect: named/colored pawns fighting, shots, HP changes, killfeed, at least one frag in ~30s; join as human and leave back to spectate.

---

## Spend gate reminder

Do **not** introduce: cloud hosts, paid APIs, paid assets. If something seems to require money, stop and flag for Chief/Nick — do not buy.
