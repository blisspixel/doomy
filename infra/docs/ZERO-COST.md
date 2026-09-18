# GCP zero-cost path checklist (fragr)

Source brief: Researcher/Gitty review `gitty-gcp-zero-fragr-2026-09-17.md` (QUALITY HOLD).

## Allowed in $0 draft

- [ ] One non-preemptible `e2-micro` in us-west1, us-central1, or us-east1
- [ ] Boot disk within Free Tier PD envelope (aim <= 30 GB standard)
- [ ] Ephemeral external IP
- [ ] Firewall: game port (7777 TCP/WS for Slice 1; UDP later if renet) from needed sources
- [ ] SSH only via IAP (no 0.0.0.0/0:22)
- [ ] Optional Cloud Run HTTP adapter: min_instances=0, invoker IAM, no game socket

## Forbidden without Nick paid-PoC ACK

- [ ] Load balancer / forwarding rules
- [ ] Reserved static IPv4 left unused
- [ ] e2-micro outside the three Free Tier regions
- [ ] Larger machine types for "just in case"
- [ ] Cloud Run min_instances > 0
- [ ] MIG / second VM on day zero
- [ ] Verbose VPC flow logs, Cloud NAT, VPN "for cleanliness"
- [ ] Authoritative tick on Cloud Run / Functions

## Protocol note

Slice 1 transport is WebSocket on the Rust server. Cloud Run still is not the preferred long-lived authority host. Prefer GCE for the tick process.

## Apply ritual

1. `terraform fmt` / `validate` / `plan`
2. Gitty review vs this checklist
3. Nick/Chief written ACK if any line can bill
4. Only then `apply`
