# infra (GCP)

Native IaC to run the fragr Rust game server on GCP **cheaply**, with room to scale.

## Product intent

- Players can **run their own server** (home, LAN, Tailscale, any box) without this folder.
- This folder is the **cloud path**: Terraform (preferred) to stand up a small, billable-aware deployment when Nick/Chief approve spend.

## Status

Draft / plan-only. **Do not `terraform apply` without written spend approval.** Default workflow stops at `fmt` / `validate` / `plan`.

## Target shape (v0 draft)

Prefer the smallest workable shape first:

1. One GCE VM **or** Cloud Run + UDP/TCP realities checked (game traffic may prefer VM + open game port).
2. Firewall allowing the game port (default 7777) and SSH/IAP as needed.
3. Secrets via Secret Manager only if required (Slice 1 needs none).
4. Clear capacity knobs documented (machine size, max peers estimate as estimate until measured).

Exact module layout lands in a follow-up PR. This README is the contract.

## Spend gate

Hard project cap $50 unless Nick raises it. Flag Chief/Nick before any create that can bill.
