# fragr

Agentic-first FPS arena: **Rust authoritative server** + **Godot spectator client**. Watch agents fight, join as human, leave back to spectate.

## Quick Start (Run Your Own Server)

Like Minecraft, you run the server and connect clients to it. Works on one machine (loopback), LAN, or Tailscale.

### Single machine (loopback)

```bash
# Terminal 1: Server (4 bots fight, 10 frag limit)
cargo run -p doomy-server -- --bots 4

# Terminal 2: Spectator client
# Open client/ in Godot 4.7.2, press F5
# Press J to join as human, L to leave back to spectate
```

### Run your own server (LAN or Tailscale)

#### Server host (home PC, VPS, or friend's machine)

```bash
# Start server on all interfaces
cargo run -p doomy-server -- --bind 0.0.0.0:7777 --bots 4

# Note your LAN IP (e.g. 192.168.1.100) or Tailscale IP (e.g. 100.x.y.z)
ip addr show  # Linux/Mac
ipconfig      # Windows
```

#### Client (spectator or player)

```bash
# Set server address via environment variable
export FRAGR_SERVER="192.168.1.100:7777"  # or Tailscale IP

# Open client/ in Godot 4.7.2, press F5
# Or launch from command line with env set
```

**Tailscale Personal:** Free tier, zero cloud spend. Install from [tailscale.com](https://tailscale.com), connect machines, use Tailscale IP.

### Agent adapter (MCP-compatible)

```bash
# Terminal 3: MCP server for external agents
cd agent-adapter && cargo run -- mcp

# Or run a scripted bot
cd agent-adapter && cargo run -- scripted-bot --name MyBot
```

## What you see

First 30 seconds as spectator:
- 4 named/colored bots spawn: Rusher (red), Sniper (cyan), Flanker (gold), Tank (green)
- Bots chase, strafe, shoot with distinct behaviors (Aggressive, Defensive, Flanker, Balanced)
- Round system: 3s warmup, then 10 frag limit or 3min time limit
- Follow-cam auto-cycles between fighters every 6s (press F to toggle free-fly)
- Muzzle flashes, hit feedback (red flash + scale pulse), killfeed with live scoreboard
- Round ends when frag limit reached or time expires; winner announced; next round auto-starts
- First frag typically within 5 seconds of round start

Press `J` to join as human fighter (WASD + mouse + LMB). Press `L` to leave back to spectate. **Bots keep fighting when humans leave.**

## Architecture

- **Server**: Rust tokio + WebSocket JSON, 20 Hz authoritative tick, hitscan combat, server-side bots, round scoring
- **Client**: Godot 4.7.2 GDScript, thin presenter with pose interpolation, intent chips on named bots
- **Protocol**: WebSocket JSON on port 7777 (see `docs/protocol.md`)
- **Match loop**: Frag limit (default 10) or time limit (default 3min), scoreboard tracks per-round kills, bots persist when humans leave
- **Spend**: $0 (loopback, LAN, Tailscale Personal only)

See `docs/ARCHITECTURE.md` for stack decisions and `docs/SLICE-1.md` for definition of done.

## Repository layout

```text
client/          Godot 4.7.2-stable (GDScript)
server/          Rust authoritative WebSocket server
agent-adapter/   MCP observe-act control plane
docs/            architecture, protocol, vision, plans
infra/           GCP IaC for scale (plan-only, no apply without approval)
AGENTS.md        instructions for coding agents
```

## Server options

```bash
cargo run -p doomy-server -- --help

Options:
  --bind <ADDR>   Bind address (default: 0.0.0.0:7777)
  --bots <N>      Number of bots to spawn (default: 4)
```

## Agent contributors

Coding agents must read [`AGENTS.md`](./AGENTS.md) before changing this repo.

## License

Private personal project under [blisspixel](https://github.com/blisspixel).
