# Cheap VPS self-host (fragr)

Generic small VM (any provider) for public stranger/agent join without Tailscale. Prefer this when home CGNAT blocks port-forward. For GCP Always Free shape + Terraform plan-only, see DURABLE-HOST.md and ZERO-COST.md.

Default game port: **TCP 6767**. Combat tick is always-on dedicated process. Not Cloud Run / Functions / scale-to-zero.

## Shape (under $50 hard cap)

- 1 vCPU / 1 GB RAM class is enough for early friends + agents (Slice 1).
- Ubuntu LTS or similar; persistent disk small.
- Public IPv4 with firewall allowing **only** TCP 6767 (UDP 6767 later if renet).
- SSH: key-only, optionally allowlisted IPs or provider console/serial. Never 0.0.0.0/0 password SSH.
- Monthly ESTIMATE should stay well under the $50 project hard cap. Flag Chief/Nick before paid create if unclear.

## Install sketch

```bash
# as root once
useradd --system --home /opt/fragr --shell /usr/sbin/nologin fragr
mkdir -p /opt/fragr
# copy release binary built elsewhere:
install -o fragr -g fragr -m 0755 fragr-server /opt/fragr/fragr-server
```

Systemd unit (same as DURABLE-HOST.md):

```ini
[Unit]
Description=fragr authoritative game server
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=fragr
WorkingDirectory=/opt/fragr
ExecStart=/opt/fragr/fragr-server --bind 0.0.0.0:6767 --bots 4
Restart=always
RestartSec=5s
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

```bash
systemctl enable --now fragr-server.service
```

Firewall example (ufw):

```bash
ufw default deny incoming
ufw allow 6767/tcp
# SSH only from your admin IP if possible
ufw enable
```

## Join path

- Strangers / agents: `ws://VPS_PUBLIC_IP:6767` (or your DNS). **No VPN.**
- Optional Tailscale: operator/dev overlay only. Do not close public 6767 for the shipped path.
- Godot clients set `FRAGR_SERVER` or the client URL to that host.
- Agent-adapter: `cargo run -- mcp --server ws://VPS_PUBLIC_IP:6767` (or run adapter beside the server on private localhost and expose only what you intend).

## Security / scale

- Rock-solid restart via systemd; watch journald.
- Keep authority on this VM; optional HTTP adapter may sit elsewhere later with private path + app auth.
- When load proves it: bigger VM or second arena instance. No day-zero load balancer.
- IaC for a named cloud (GCP Terraform in this repo) stays **plan-only** until Nick/Chief spend ACK. Manual VPS create still counts against the $50 hard cap if it bills.

## Throw-outs

- Tailscale-only public join
- Authoritative tick on serverless / scale-to-zero
- Opening SSH to the world
- Documenting port 7777
