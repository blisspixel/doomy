# infra (GCP)

Native IaC to run the fragr Rust **authoritative game server** on GCP cheaply, with room to scale.

## Product intent

- **Run your own server** (home LAN, cheap VPS, or GCP) without requiring Tailscale for strangers/agents. Minecraft-shaped ops. See `docs/HOME-LAN.md` and `docs/CHEAP-VPS.md`.
- This folder is the **cloud path**: Terraform to stand up a billable-aware deployment when Nick/Chief approve spend.
- LAN/Tailscale are optional buddy paths, not the only story.

## Status

Plan / draft only. **Do not `terraform apply` without written Nick/Chief spend ACK.** Default stops at `fmt` / `validate` / `plan`.

Review gate: Gitty reviews drafts against the Researcher QUALITY HOLD brief for fragr GCP zero-cost (Gitty holds that brief). Zero tool attribution in commits/PRs.

## $0 default shape (HOLD)

Until Nick explicitly accepts a paid PoC:

1. **Core tick server:** Always Free eligible **`e2-micro` GCE** in **`us-west1` / `us-central1` / `us-east1` only**.
2. **Ephemeral external IP** (no orphan reserved static IPv4). **Cost honesty:** ESTIMATE ~$3.65/mo for sustained use (Free Tier IP is 1 hour/month crumb, not always-free at $0.005/h). **Public TCP+UDP 6767** is the stranger/agent join path (no VPN). Tailscale Personal is optional private/dev only , do not close public 6767 for the shipped Minecraft-shaped story.
3. **Tight firewall:** game port(s) + **SSH via IAP only** (no world SSH).
4. **HTTP agent-adapter (optional):** Cloud Run with **`min_instances=0`** + invoker IAM; private path to VM admin API. **Not** the raw game socket.
5. **Never** put authoritative tick on Cloud Run/Functions as a free UDP front door (no inbound UDP). Slice 1 is WebSocket today; still prefer **GCE for long-lived authority**.

## Identity and secrets (required before modules)

- Separate least-privilege service accounts: adapter SA is not the VM SA.
- Secrets in Secret Manager only. Never bake into images or commit `terraform.tfstate` / secret-bearing `.tfvars`.
- Cloud Run: no `allUsers` invoker unless Nick explicitly ACKs. Adapter to game via private IP plus app auth (HMAC / mTLS / token). Open RFC1918 alone is not enough.
- At Nick spend gate: budget alert / billing export. Free Tier NA egress (~1 GB) is an ESTIMATE crumb that blows up with real players.
- No GKE "free cluster" as fake free compute. No Private Service Connect endpoints on day zero.

## Block before apply (unless Nick accepts paid PoC)

- Any load balancer / `forwarding_rule` (~$0.025/h class)
- Unused reserved static IPv4
- Wrong region/machine or Cloud Run `min_instances > 0`
- MIG / second VM eating Free Tier hours
- Verbose flow logs / NAT/VPN "for cleanliness"

## Layout (draft)

```text
infra/
  README.md           # this contract
  docs/
    HOME-LAN.md       # home box: LAN + optional public port-forward (no Tailscale-required)
    CHEAP-VPS.md      # generic cheap VM public join (no Tailscale-required)
    ZERO-COST.md      # GCP checklist + sources
    DURABLE-HOST.md   # GCP durable GCE recipe (systemd + public 6767 primary, Tailscale private/dev, cost ceiling)
  terraform/          # modules land in follow-up PRs; plan-only
```

## Spend gate

Hard project cap $50 unless Nick raises it. Flag Chief/Nick before any create that can bill.
