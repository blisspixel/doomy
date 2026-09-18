# GCP zero-cost path checklist (fragr)

Source: Researcher QUALITY HOLD brief for fragr GCP zero-cost (held by Gitty for review).

For durable multi-hour-playable host with cost ceiling honesty (public game-port front door, optional Tailscale private/dev, systemd restart, external IP pricing), see DURABLE-HOST.md.

## Allowed in $0 draft

- [ ] One non-preemptible `e2-micro` in us-west1, us-central1, or us-east1
- [ ] Boot disk within Free Tier PD envelope (aim <= 30 GB standard)
- [ ] Ephemeral external IP (ESTIMATE ~$3.65/mo for sustained use; Free Tier IP is 1 hour/month crumb, not always-free)
- [ ] Firewall: public game port (6767 TCP/WS for Slice 1; UDP later if renet) open for strangers + agents (tight: game ports only). SSH via IAP only. Tailscale Personal optional for private/dev smoke, do not close public 6767 for the shipped join path.
- [ ] SSH only via IAP (no 0.0.0.0/0:22)
- [ ] Optional Cloud Run HTTP adapter: min_instances=0, invoker IAM, no game socket

- [ ] Separate least-privilege SAs (adapter SA != VM SA)
- [ ] Secrets only in Secret Manager (not image, not committed state/tfvars)
- [ ] Cloud Run invoker IAM (no allUsers unless Nick ACK)
- [ ] Adapter to game: private IP + app auth (HMAC/mTLS/token)

## Forbidden without Nick paid-PoC ACK

- [ ] Load balancer / forwarding rules
- [ ] Reserved static IPv4 left unused
- [ ] e2-micro outside the three Free Tier regions
- [ ] Larger machine types for "just in case"
- [ ] Cloud Run min_instances > 0
- [ ] MIG / second VM on day zero
- [ ] Verbose VPC flow logs, Cloud NAT, VPN "for cleanliness"
- [ ] Authoritative tick on Cloud Run / Functions

- [ ] GKE "free cluster" used as if compute were free
- [ ] Private Service Connect endpoints on day zero
- [ ] Committed terraform.tfstate or secret-bearing tfvars
- [ ] Missing budget alert / billing export at Nick spend gate

## Protocol note

Slice 1 transport is WebSocket on the Rust server. Cloud Run still is not the preferred long-lived authority host. Prefer GCE for the tick process.

## Apply ritual

1. `terraform fmt` / `validate` / `plan`
2. Gitty review vs this checklist
3. Nick/Chief written ACK if any line can bill
4. Only then `apply`

## Nick spend gate extras

- Budget alert and/or billing export before apply.
- Treat 1 GB Free Tier egress as ESTIMATE that will blow up with real players.
