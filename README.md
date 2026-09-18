# Doomy

Working name for a modern Doom-like: **Godot client** + **Rust authoritative server**, agent-playable, **spectator-default**, optional human join.

Personal project under [blisspixel](https://github.com/blisspixel). Loopback-first. Hard spend cap $50; ask before any paid cloud/API/assets.

## Status

Slice 1 (**Arena Duel Watch**) is in progress. See `docs/SLICE-1.md` for the definition of done and `docs/ARCHITECTURE.md` for stack decisions.

Intended layout:

```text
client/          Godot 4.7.2-stable (GDScript)
server/          Rust authoritative WebSocket server
agent-adapter/   MCP / control-plane observe-act
docs/            architecture, protocol, slices
AGENTS.md        instructions for coding agents
```

## Agent contributors

Coding agents must read [`AGENTS.md`](./AGENTS.md) before changing this repo.

## License / lane

Private personal project. Stay on blisspixel. No work-account coupling.
