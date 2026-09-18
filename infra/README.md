# infra (GCP)

Native IaC to run the fragr Rust **authoritative game server** on GCP cheaply, with room to scale.

## Product intent

- **Run your own server** (home, LAN, Tailscale, any box) without this folder. Minecraft-shaped ops.
- This folder is the **cloud path**: Terraform to stand up a billable-aware deployment when Nick/Chief approve spend.
- LAN/Tailscale are optional buddy paths, not the only story.

## Status

Plan / draft only. **Do not `terraform apply` without written Nick/Chief spend ACK.** Default stops at `fmt` / `validate` / `plan`.

Review gate: Gitty checks drafts against `/workspace/gitty-gcp-zero-fragr-2026-09-17.md` (QUALITY HOLD). Zero tool attribution in commits/PRs.

## $0 default shape (HOLD)

Until Nick explicitly accepts a paid PoC:

1. **Core tick server:** Always Free eligible **`e2-micro` GCE** in **`us-west1` / `us-central1` / `us-east1` only**.
2. **Ephemeral external IP** (no orphan reserved static IPv4).
3. **Tight firewall:** game port(s) + **SSH via IAP only** (no world SSH).
4. **HTTP agent-adapter (optional):** Cloud Run with **`min_instances=0`** + invoker IAM; private path to VM admin API. **Not** the raw game socket.
5. **Never** put authoritative tick on Cloud Run/Functions as a free UDP front door (no inbound UDP). Slice 1 is WebSocket today; still prefer **GCE for long-lived authority**.

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
    ZERO-COST.md      # checklist + sources
  terraform/          # modules land in follow-up PRs; plan-only
```

## Spend gate

Hard project cap $50 unless Nick raises it. Flag Chief/Nick before any create that can bill.
