# Plan: Public or local join (docs scrub)

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/public-or-local`
**Spend:** $0 docs only. Terraform stays plan-only. No apply.
**Status:** In flight (docs scrub PR).

## Goal

Lock the product path as **public servers OR local play**. README and agent-facing docs lead with Solo Scrap on loopback **6767**, or Minecraft-shaped public self-host (TCP+UDP **6767**). Tailscale Personal is optional private/dev smoke only, never the documented multiplayer front door. Spend honesty: local is **$0**; public host sits under the **$50** hard cap and needs Nick/Chief spend ACK (plan-only until then). Do not claim the product is Tailscale-only $0.

## Non-goals

- Terraform apply or any billable create
- Protocol / transport / bind code changes
- Closing public 6767 or requiring a VPN for strangers/agents
- Elevating Tailscale as the primary multiplayer story
- Tool attribution (Co-authored-by Codex/Cursor/Claude, Made-with badges)
- Emoji, em dashes, or en dashes in prose

## Nick lock (2026-09-18)

1. Product path: **public servers OR local play**. Tailscale is fine for some testing only, not a goal, not the documented multiplayer story.
2. One lean CI-green main only.
3. Zero tool attribution KAPU (no Co-authored-by Codex/Cursor).

## Research (tip `ea9cdcc`)

| Doc | Tip today | Gap |
|---|---|---|
| README Play Modes / Multi-machine | MP framed as LAN or Tailscale; Tailscale Personal called free zero spend front door | Demote Tailscale; lead with public self-host + Solo Scrap |
| README Spend | `$0 (loopback, LAN, Tailscale Personal only)` | Implies Tailscale-only $0 product; omit public-host under-$50 ACK |
| AGENTS.md Spend | Prefer home-host, then Tailscale Personal ($0), then Oracle | Tailscale ranked as preferred MP path |
| VISION Hosting model | Already public 6767 primary; LAN/Tailscale buddy options | Keep; light align if any Prefer Tailscale wording remains |
| infra README / DURABLE-HOST / HOME-LAN / CHEAP-VPS | Public 6767 primary; Tailscale private/dev | Mostly aligned; scrub only if Prefer Tailscale front-door language remains |
| ARCHITECTURE / SLICE-1 | LAN or Tailscale as self-host peers | Soften to public OR local; Tailscale test-only |
| Tip priorities | Warmup Host drama listed as NOW; shipped at `ea9cdcc` (#83) | Mark shipped; add this plan as NOW |

## Architecture impact

| Area | Change |
|---|---|
| `docs/plans/public-or-local-join.md` | This plan |
| `docs/plans/README.md` | Tip priorities + index row |
| `README.md` | Quick Start / Play Modes / Multi-machine / Spend framing |
| `AGENTS.md` | Spend preference order: local $0, public under $50 with ACK; Tailscale optional smoke |
| `docs/VISION.md` | Align only if Prefer Tailscale front-door remains |
| `docs/ARCHITECTURE.md` / `docs/SLICE-1.md` | Demote Tailscale-as-peers front-door wording |
| `infra/` | Align Prefer Tailscale front-door language if any; terraform stays plan-only |
| Code | None unless a string documents Tailscale as required |

## Doc approach

1. **Primary paths:** Solo Scrap local loopback 6767, OR public self-host TCP+UDP 6767 (Minecraft-shaped). Same Action path.
2. **Tailscale Personal:** Optional private/dev smoke only. Demote every line that presents Tailscale as the multiplayer front door.
3. **Spend:** Local $0. Public host under $50 with spend ACK; infra remains plan-only until then. Do not claim product is Tailscale-only $0.
4. **Infra:** Leave terraform plan-only (no apply). Keep public game-port story in DURABLE-HOST / HOME-LAN / CHEAP-VPS.

## Verification

- Docs-only PR: no `cargo` / Godot churn required for this scrub.
- Grep: README / AGENTS / VISION / ARCHITECTURE / SLICE-1 / infra do not present Tailscale as required or as the primary MP join path.
- Spend lines state local $0 and public under $50 with ACK (plan-only).
- Commit author Nick Seal / blisspixel noreply only. No Co-authored-by. No emoji. No em/en dashes.
- Branch `cursor/public-or-local`; PR open via `gh`.

## Spend / safety

Docs and plan only. No cloud apply. Hard cap $50 unchanged. Zero secrets.

## Success criteria

- [ ] Plan tracked; tip priorities updated
- [ ] README says public OR local; Tailscale test-only
- [ ] AGENTS / VISION / ARCHITECTURE / SLICE-1 / infra Prefer Tailscale front-door language scrubbed
- [ ] Terraform plan-only (no apply)
- [ ] PR open; zero tool attribution; CI-ready (docs-only)
