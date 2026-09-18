# fragr

Public repo name: **fragr**. Older docs may still say Doomy (working name; rename pass later).

Agent-first FPS arena: Godot client + Rust authoritative server. Bring your own agents, play yourself, or spectate. Stupid fun with guns.

Personal project under [blisspixel](https://github.com/blisspixel). Loopback-first. Hard spend cap $50; ask before any paid cloud, API, or assets.

## Slice 1: Arena Duel Watch

Four named rule bots fight in a graybox arena. Default client mode is spectator. Press `J` to join as human, `L` to leave back to spectate. Optional MCP / scripted agent adapter.

Stack pins: Godot **4.7.2-stable** (GDScript), Rust server over WebSocket JSON on `127.0.0.1:7777`, MCP as a slow control plane (not the combat tick).

## Quick start (about two minutes)

**Terminal 1: server + bots**

```bash
cargo run -p doomy-server --release -- --bots 4
# or: cd server && cargo run --release -- --bots 4
```

**Terminal 2: Godot spectator**

1. Open `client/` in Godot 4.7.2
2. Press F5
3. Free-fly with WASD + mouse; `F` cycles follow-cam

**Terminal 3 (optional): extra bot or MCP**

```bash
cargo run -p doomy-agent-adapter --release -- scripted-bot --name "Bot3"
# or MCP stdio control plane:
cargo run -p doomy-agent-adapter --release -- mcp
```

Expect frags in server logs within a few seconds, and moving colored pawns in the spectator view.

## Layout

```text
client/           Godot 4.7.2-stable (GDScript presenter)
server/           Rust authoritative WebSocket server
agent-adapter/    MCP observe/act + scripted bot client
docs/             architecture, protocol, slices, plans
AGENTS.md         standing instructions for coding agents
```

## Docs

- [`AGENTS.md`](./AGENTS.md): agent law (verification, spend, no attribution / emoji / long dashes, plan before build)
- [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md): stack decisions
- [`docs/SLICE-1.md`](./docs/SLICE-1.md): slice checklist
- [`docs/protocol.md`](./docs/protocol.md): on-wire messages
- [`docs/plans/`](./docs/plans/): write plans here before new build work

## Lane

blisspixel only. No work-account coupling. Coding agents must read `AGENTS.md` before changing this repo.
