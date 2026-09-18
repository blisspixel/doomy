# Brain as third agent tier surface

Status: **in flight** (this PR)

## Goal

Make the three-rung agent ladder unmistakable in the skill card and front-door README, and wire an observe-only stance chip so `fragr-brain` shows up beside rule bots on the HUD.

1. Server rule bots (Aggressive / Defensive / Flanker / Balanced)
2. MCP BYO via agent-adapter
3. fragr-brain (Jev / local + 20 Hz controller; paid only with `--max-spend-usd`)

## Non-goals

- No new MCP tool for stance (brain speaks the wire directly)
- No combat trust of client behavior strings
- No lore expansion on the README front door

## Protocol

New client message `set_display_behavior` with deny-unknown-fields body `{ "behavior": string }`. Server stores it on the Agent player as `display_behavior` and echoes it in Snapshot `PlayerState.behavior` when the player is not a rule bot. Max 32 scalars, trim, reject controls. Humans and rule bots ignored.

## Verification

- Unit: protocol deserialize + deny unknown; session snapshot echoes Agent label; human/rule-bot reject
- Brain: wire helper only republishes on stance change; integration spectator sees chip after join
- HUD: `_short_behavior` maps the four stance names
- `cargo test` / clippy `-D warnings` / honest coverage >= 80%

## Success

Skill + README name MCP and brain; stance chip visible for brain Agents; tests green.
