# Transport and Networking

## Current: WebSocket JSON (Slice 1)

**Status:** Implemented  
**Use case:** Loopback, agent play, spectator streams, human play (acceptable latency)

### Details
- Protocol: WebSocket over TCP
- Format: JSON text messages
- Address: Server binds `0.0.0.0:7777` by default (LAN + Tailscale capable)
- Client: Loopback default, accepts `FRAGR_SERVER` env var for remote connections
- Tick rate: ~20 Hz server broadcast
- Multi-peer: Multiple spectators/players can connect to same match
- Pros: Simple, universal, easy to debug, works for all roles, multi-peer ready
- Cons: Higher latency than UDP, more bandwidth than binary

**Good enough for Slice 1.** Spectators do not need low latency, agents operate on slow control plane, and human play is acceptable with minor input lag. Second peer can spectate via LAN or Tailscale Personal ($0).

## Planned: UDP/renet (Post-Slice 1)

**Status:** Next networking spike  
**Use case:** Low-latency human FPS play, competitive matches

### Why UDP/renet?

For smooth human FPS, sub-50ms input latency is ideal. WebSocket over TCP has:
- TCP head-of-line blocking (one dropped packet stalls the stream)
- JSON serialization overhead
- Larger packet sizes

UDP with custom protocol (e.g., `renet`, `laminar`, or hand-rolled) provides:
- Unreliable + reliable channels (input on unreliable, events on reliable)
- Binary serialization (smaller packets)
- No head-of-line blocking
- Client-side prediction + server reconciliation

### Planned Architecture

```
┌──────────────────────────────────────────┐
│  Godot Client (Human)                    │
│  • UDP for actions (unreliable)          │
│  • UDP for snapshots (unreliable)        │
│  • Prediction + reconciliation           │
└──────────────────────────────────────────┘
           ↕ UDP (renet or custom)
┌──────────────────────────────────────────┐
│  Rust Server                             │
│  • UDP socket for fast clients           │
│  • WebSocket for spectators/agents       │
│  • Dual transport support                │
└──────────────────────────────────────────┘
           ↕ WebSocket JSON
┌──────────────────────────────────────────┐
│  Spectators / Agent Adapter              │
│  • Keep WebSocket (good enough)          │
│  • No need for UDP complexity            │
└──────────────────────────────────────────┘
```

### Godot ↔ Rust UDP Options

| Option | Pros | Cons |
|--------|------|------|
| `renet` | Battle-tested, channels, reliable + unreliable | Rust-first; Godot needs custom GDScript wrapper |
| `laminar` | Rust + clean API | Less mature, GDScript integration unclear |
| Custom UDP | Full control, tailored to Doomy | More work, reinvent reliability layer |
| GDExtension | Native Rust in Godot | Build complexity, cross-platform pain |

**Recommendation (TBD after spike):** Try `renet` with GDScript `PacketPeerUDP` wrapper first. If painful, fall back to simple custom UDP with manual ack/sequencing.

## Timeline

- **Slice 1 (now):** WebSocket JSON only
- **Slice 2 (UDP spike):** Prototype UDP, measure latency improvement, decide on library
- **Slice 3+:** Dual transport (WebSocket for spectators/agents, UDP for humans)

## References

- `renet`: https://github.com/lucaspoffo/renet
- `laminar`: https://github.com/TimonPost/laminar
- Godot PacketPeerUDP: https://docs.godotengine.org/en/stable/classes/class_packetpeerudp.html
- Fast-Paced Multiplayer: https://www.gabrielgambetta.com/client-server-game-architecture.html
