# Home LAN self-host (fragr)

Run the Rust authoritative server on a box you control at home. Minecraft-shaped: friends and agents join the same fight without a VPN when you open the game port.

Default game port: **TCP 6767** (WebSocket today; UDP later if renet). Do not document 7777.

## Local play (no strangers)

```bash
cargo run -p fragr-server -- --bind 127.0.0.1:6767 --bots 4
# Godot client/ F5, or FRAGR_SERVER=127.0.0.1:6767
# agent-adapter: cargo run -- mcp --server ws://127.0.0.1:6767
```

Loopback only. Fine for solo + bots and private agent tests.

## LAN buddies (same network)

```bash
cargo run -p fragr-server -- --bind 0.0.0.0:6767 --bots 4
```

Clients use the host's LAN IP: `ws://192.168.x.x:6767`. Allow TCP 6767 on the host firewall (ufw/firewalld/Windows Defender) for the LAN subnet.

Tailscale is optional for operator smoke when you are away from home. It is **not** required for friends on the same LAN.

## Public strangers + agents (no Tailscale required)

Same binary. Expose **only** the game port:

1. Bind `0.0.0.0:6767` on the host.
2. Router port-forward **TCP 6767** (and UDP 6767 later if transport adds it) to that host.
3. Prefer a dedicated low-privilege `fragr` user + systemd unit with `Restart=always` (see DURABLE-HOST.md sketch).
4. Do **not** open SSH to the world. Keep SSH on LAN or IAP/Tailscale for operators only.
5. Tell joiners your public IP or DNS: `ws://YOUR_PUBLIC:6767`.

CGNAT / double-NAT: if your ISP will not forward, use a cheap VPS instead (CHEAP-VPS.md) or the GCP durable recipe (DURABLE-HOST.md). Do not ship Tailscale-only as the stranger/agent path.

## Security bar (home)

- Game port open; admin/SSH closed to the internet.
- No secrets in the server command line; no cloud env required for Slice 1.
- Keep the host patched; treat public 6767 as an untrusted front door (rate limits / join caps land in later slices).
- Agent-adapter stays off the combat tick (separate process; MCP over HTTP or local). Never put authority on scale-to-zero.

## Cost

Home electricity only unless you also run cloud. No GCP spend for this path.
