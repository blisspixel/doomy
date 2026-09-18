# Durable self-host recipe (fragr GCE)

**Source:** Researcher HOLD brief for fragr GCP durable host.

This is the durable, multi-hour-playable path for running a fragr server on GCP with predictable costs and restarts. For ephemeral testing, see ZERO-COST.md.

## Recipe

### GCE instance

- **Machine:** Always Free eligible `e2-micro` (non-preemptible)
- **Regions:** `us-west1`, `us-central1`, or `us-east1` only (Always Free tier)
- **Boot disk:** Standard persistent disk, ≤30 GB (within Free Tier envelope)

### Networking

- **Ports:** TCP+UDP 7777 (documented game socket; Slice 1 uses TCP/WS today, UDP later if renet)
- **SSH:** IAP tunnel only (no public 0.0.0.0/0:22)
- **Front door (Nick lock):**
  - **Primary (strangers + agents):** public external IP with tight firewall opening **only** the game port(s) to `0.0.0.0/0` (TCP+UDP 7777). Randos and agents join the same fight with **no VPN**. This is the Minecraft-shaped self-host story.
  - **Private/dev only:** Tailscale Personal ($0) as an optional overlay for Nick smoke tests and operator convenience. **Not** the spectator/agent join path. Do **not** close public 7777 and ship Tailscale-only.
  - Cost of the public path: see cost ceiling honesty below (external IP ESTIMATE).

### Systemd service

Install the fragr-server binary to `/opt/fragr/fragr-server` and run via systemd with `Restart=always`. Sketch unit:

```ini
[Unit]
Description=fragr authoritative game server
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=fragr
WorkingDirectory=/opt/fragr
ExecStart=/opt/fragr/fragr-server
Restart=always
RestartSec=5s
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

Create a dedicated `fragr` user, place binary in `/opt/fragr/`, install unit to `/etc/systemd/system/fragr-server.service`, then `systemctl enable --now fragr-server.service`.

## Near-term scale ladder (Nick)

Rust dedicated authority is the spine. Keep it rock solid and able to grow. Combat tick is an always-on process (sticky WebSocket today; UDP later if renet). It is **not** Cloud Run, Functions, or any scale-to-zero host.

**Near-term reality:** once the arena is actually fun, some agents plus a couple friends try it. Not a launch-day crowd.

**Ladder under the $50 hard cap (plan-only until spend ACK):**

1. **Friends + agents:** public Always Free `e2-micro` with public TCP+UDP 7777 (this recipe).
2. **When load proves it:** one bigger/cheaper VM, or a second arena instance. Do not invent day-zero MIG or LB.
3. **HTTP agent-adapter (optional):** may be Cloud Run with `min_instances=0` only if it stays **off** the combat tick (private path + app auth to the VM). Never put the authoritative tick on scale-to-zero.

Ideal later "serverless / scale great" energy is fine as aspiration. The combat tick still stays on dedicated always-on compute.

## Cost ceiling honesty

### Documented costs (ESTIMATE)

| Item | ESTIMATE | Status |
|------|----------|--------|
| e2-micro VM | $0/mo | **Documented** (Always Free: 1 e2-micro/month in us-west1/us-central1/us-east1) |
| 30 GB pd-standard | $0/mo | **Documented** (Always Free: 30 GB standard persistent disk) |
| External IP (in-use) | ~$3.65/mo | **ESTIMATE** (1 ephemeral IP, ~730h/mo at $0.005/h, not counting 1-hour Free Tier crumb) |
| External IP (reserved, unused) | ~$2.88/mo | **Documented** (not used in this recipe) |
| Egress (sustained play) | Variable | **ESTIMATE** (Free Tier: 1 GB/mo NA egress crumb; 10 sustained players at 50 KB/s avg = 1.8 GB/h, blows crumb quickly) |

**Free Tier IP honesty:** GCP Always Free includes 1 hour/month of external IP at no charge, not full-time free. After that crumb, expect $0.005/h for in-use ephemeral IP. At 730 hours/month, the ESTIMATE is ~$3.65/mo.

### Steady-state ceiling

- **Floor:** ~$4/mo (mostly external IP at $0.005/h for sustained durable host)
- **Ceiling (normal):** ESTIMATE << $20/mo with light egress
- **Project hard cap:** $50 total (Nick approval required to raise)

Public stranger/agent join pays the external IP ESTIMATE (~$3.65/mo) and real egress. That is expected for the Minecraft-shaped story. Tailscale Personal ($0) remains useful for private smoke and operator SSH convenience, but it does not replace the public game-port front door.

## Budget safeguards

- **Budget alert:** Set at Nick $50 hard cap (or lower, e.g. $5 or $10 for early warning)
- **Optional billing-disable:** Billing admins can configure project-level billing disable at threshold to prevent runaway charges

Billable egress from 0.0.0.0/0 public play will blow the 1 GB Free Tier crumb quickly. Monitor in GCP Console.

## Throw-outs (not this recipe)

- **Cloud Run / Functions / GKE as tick host:** Authoritative combat tick is long-lived sticky sockets. Never scale-to-zero for the game process. GCE (or later a bigger dedicated VM) only.
- **Day-zero LB / unused static IP:** No forwarding rules or reserved addresses on day one.
- **WS to renet mid-ship change:** Slice 1 transport is WebSocket. Do not scrap and rewrite to renet UDP mid-implementation unless Nick asks.
- **Any terraform apply without Nick approval:** This is still plan-only until spend gate opens.

## Regions (Always Free)

Always Free `e2-micro` availability (as of Researcher brief):

- `us-west1` (Oregon)
- `us-central1` (Iowa)
- `us-east1` (South Carolina)

Do not deploy `e2-micro` outside these regions or you will be billed full price.

## IP pricing sources

- [Compute Engine Pricing: External IP addresses](https://cloud.google.com/compute/all-pricing#ipaddress)
  - In-use ephemeral: $0.005/hour
  - Reserved unused: $0.012/hour
- [GCP Always Free Tier](https://cloud.google.com/free/docs/free-cloud-features#compute)
  - 1 hour/month of external IP usage at no charge (crumb, not sustained-free)

## Next steps

1. Review this recipe vs the $50 hard cap.
2. Get Nick/Chief approval before any `terraform apply`.
3. If approved, stand up GCE with public TCP+UDP 7777 (IAP SSH only), optional Tailscale for private/dev smoke, validate systemd restart behavior.
4. Monitor actual costs in GCP Billing Console; adjust if ceiling approaches.
