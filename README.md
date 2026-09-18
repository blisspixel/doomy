# fragr

Agentic-first FPS arena: **Rust authoritative server** + **Godot spectator client**. Watch agents fight, join as human, leave back to spectate. Loopback + LAN + Tailscale ($0).

## Quick Start

### Single machine (loopback)

```bash
# Terminal 1: Server (4 bots fight)
cargo run -p doomy-server -- --bots 4

# Terminal 2: Spectator client
# Open client/ in Godot 4.7.2, press F5
# Press J to join as human, L to leave back to spectate
```

### Two machines (LAN or Tailscale)

#### Machine 1 (server host)

```bash
# Start server on all interfaces
cargo run -p doomy-server -- --bind 0.0.0.0:7777 --bots 4

# Note your LAN IP (e.g. 192.168.1.100) or Tailscale IP (e.g. 100.x.y.z)
ip addr show  # Linux/Mac
ipconfig      # Windows
```

#### Machine 2 (spectator peer)

```bash
# Set server address via environment variable
export FRAGR_SERVER="192.168.1.100:7777"  # or Tailscale IP

# Open client/ in Godot 4.7.2, press F5
# Or launch from command line with env set
```

**Tailscale Personal:** Free tier, no cloud spend. Install from [tailscale.com](https://tailscale.com), connect both machines, use Tailscale IP.

### Agent adapter

```bash
# Terminal 3: MCP-compatible agent
cd agent-adapter && cargo run --
```

## What you see

First 30 seconds as spectator:
- 4 named/colored bots spawn: Rusher (red), Sniper (cyan), Flanker (gold), Tank (green)
- Bots chase, strafe, shoot with distinct behaviors (Aggressive, Defensive, Flanker, Balanced)
- Follow-cam auto-cycles between fighters every 6s (press F to toggle free-fly)
- Muzzle flashes, hit feedback (red flash), killfeed with scores
- First frag typically within 5 seconds

Press `J` to join as human fighter (WASD + mouse + LMB). Press `L` to leave back to spectate.

## Architecture

- **Server**: Rust tokio + WebSocket JSON, 20 Hz authoritative tick, hitscan combat, server-side bots
- **Client**: Godot 4.7.2 GDScript, thin presenter with pose interpolation
- **Protocol**: WebSocket JSON on port 7777 (see `docs/protocol.md`)
- **Spend**: $0 (loopback, LAN, Tailscale Personal only)

See `docs/ARCHITECTURE.md` for stack decisions and `docs/SLICE-1.md` for Slice 1 definition of done.

## Repository layout

```text
client/          Godot 4.7.2-stable (GDScript)
server/          Rust authoritative WebSocket server
agent-adapter/   MCP observe-act control plane
docs/            architecture, protocol, vision, plans
AGENTS.md        instructions for coding agents
```

## Agent contributors

Coding agents must read [`AGENTS.md`](./AGENTS.md) before changing this repo.

## License

Private personal project under [blisspixel](https://github.com/blisspixel).
