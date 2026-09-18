# Agent aim + hit-confirm

## Goal

Make MCP and scripted agents able to aim and get readable combat feedback so they stop being farm fodder. Agents share the same Action path as humans. LLM stays off the combat tick.

## Non-goals

- Reopen round_* event redesign
- ElevenLabs, GCP apply, renet, L5
- Mode fantasy / Contested Frequency HUD (optional later, only after aim works)
- Godot hit markers (nice-to-have only if cheap)

## Problem

On v0.5.0 tip, agents only have sticky `turn_left` / `turn_right`. Slow MCP rates cannot track moving targets the way server bots do. Observe exposes poses and HP but not structured "did my shot hit / who took damage / HP delta" feedback, so agents cannot close the fire loop without vision.

## Wire changes

### Action: `look_at`

Add optional nested object on `Action` (still `deny_unknown_fields` on Action and LookAt):

```json
{
  "type": "action",
  "look_at": { "player_id": "<uuid>" }
}
```

or world point:

```json
{
  "type": "action",
  "look_at": { "x": 10.0, "z": -5.0 }
}
```

Rules:

- Authoritative server applies yaw toward the target on the Action tick (same path as humans).
- Prefer `player_id` when present and the target is alive in sim; else use `x`/`z` when both present.
- Invalid / missing target is a no-op for yaw (does not reject the whole Action).
- Sticky like other Action fields: each act overwrites pending Action.
- MCP `act` validates `look_at` shape; unknown keys still schema-error.

### Observe / snapshot: `shot_results`

Each Snapshot carries `shot_results` (omit when empty) for fires resolved on that tick:

```json
{
  "shooter_id": "<uuid>",
  "shooter": "ArenaFox",
  "hit": true,
  "target_id": "<uuid>",
  "target": "Bot1",
  "damage": 25,
  "target_hp_after": 75
}
```

Misses set `hit: false` with null/omitted target fields and `damage: 0`.

### Events: `hit` (damage only)

When damage is applied, also push `GameEvent::Hit` into the existing event ring so `observe.recent_events` / `get_events` expose structured hit feedback without prose:

```json
{
  "event": "hit",
  "shooter": "ArenaFox",
  "shooter_id": "<uuid>",
  "target": "Bot1",
  "target_id": "<uuid>",
  "damage": 25,
  "target_hp_after": 75
}
```

Do not reopen round_* shapes.

## Architecture impact

| Area | Change |
|---|---|
| `server` protocol + sim | LookAt on Action; apply yaw in tick; ShotResult on Snapshot; Hit event on damage |
| `agent-adapter` protocol + mcp | Mirror types; validate `look_at`; tools/list schema; scripted bot uses `look_at.player_id` |
| Docs | `docs/protocol.md`, adapter README, `docs/skills/fragr/SKILL.md`, this plan, plans tip priorities |

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
```

Behavioral tests:

1. Action with `look_at` player_id / x+z sets yaw toward target.
2. Unknown Action / look_at fields still fail deserialize / MCP act.
3. Fire hit populates `shot_results` and Hit event with damage + target_hp_after; miss has hit=false.
4. Scripted bot path builds look_at actions (unit on compute helper).

Unfiltered llvm-cov fail-under 80 (no ignore-filename-regex). Ship via gh api.

## Success criteria

- PR open from `cursor/agent-aim-hit-confirm` onto main tip (post honest-coverage).
- Agents can turn toward a target via `act.look_at`.
- Observe / events give structured hit and damage feedback.
- CI-ready quality bar (fmt, clippy -D warnings, tests, coverage >= 80).
