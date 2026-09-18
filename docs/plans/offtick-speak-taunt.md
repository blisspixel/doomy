# Off-tick speak / taunt

## Goal

Agents can emit a short speak / taunt / callout after the MCP door is trusted. Speak is control-plane / off-tick (LLM never on the 20 Hz combat tick). Spectators see it (HUD Host line / killfeed-adjacent). The line also lands in MCP `recent_events` / `get_events`.

## Non-goals

- ElevenLabs, paid TTS, GCP apply, renet, L5
- Reopening look_at / round_* shapes
- Sticky Action path for chat (speak is a dedicated non-tick channel)

## Tone

Contested Frequency: chill play laugh. Host easter eggs OK. Game first. Parody fiction, not manifesto.

## Wire shape

### Client -> server (dedicated channel)

```json
{"type": "speak", "text": "nice scrap"}
```

- `deny_unknown_fields` on the speak payload.
- Not part of sticky Action. Same human/agent WebSocket; spectators cannot speak.

### Server -> clients / events

```json
{
  "type": "event",
  "event": "speak",
  "player": "ArenaFox",
  "player_id": "<uuid>",
  "text": "nice scrap"
}
```

Broadcast with other events. Buffered in adapter `recent_events` (cap 50) and returned by `get_events`.

### Validation (authoritative server)

| Rule | Value |
|---|---|
| Max length | 80 Unicode scalars after trim |
| Empty / whitespace-only | Reject (no event) |
| Control characters | Reject (no event) |
| Rate limit | 1 successful speak per player per 60 ticks (3s at 20 Hz) |
| Unknown player | No-op |

### MCP tool `speak`

```json
{"name": "speak", "arguments": {"text": "nice scrap"}}
```

- Schema: object with required `text` (string). `additionalProperties` false / unknown keys -> clear schema error.
- Length / empty validated in adapter before send; server re-validates.
- Documented in `docs/skills/fragr/SKILL.md` and `agent-adapter/README.md`.

## Architecture impact

| Area | Change |
|---|---|
| `server` protocol | `ClientMessage::Speak`, `GameEvent::Speak`, `Speak` payload |
| `server` net / session | `GameCommand::Speak`; apply off-tick; push event |
| `server` sim | `try_speak` (length, controls, cooldown); `last_speak_tick` on player |
| `agent-adapter` | Mirror types; MCP `speak`; `pending_speak` on handle outcome |
| Scripted bot | Occasional canned Contested Frequency taunt (no LLM) |
| Godot | Show speak on killfeed-adjacent / Host line without breaking string literals |
| Docs | protocol.md, SKILL.md, adapter README, this plan, plans tip priorities |

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
```

Behavioral tests:

1. Valid speak broadcasts `GameEvent::Speak` and appears in take_events / tick messages.
2. Over-length, empty, control chars rejected (no event).
3. Rate limit: second speak inside 60 ticks emits no Speak event and queues Error unicast; after cooldown it emits again.
4. MCP `speak` validates schema; unknown fields / bad text / rate-limit / spectator -> `isError`; valid sets pending_speak + success text.
5. Scripted bot helper selects a taunt on its cadence.

Unfiltered llvm-cov fail-under 80 (no ignore-filename-regex for main/net/adapter). Ship via gh api. No tool attribution, no emoji, no em/en dashes.

## Success criteria

- PR open from `cursor/offtick-speak-taunt` onto main tip (post #60 screenshots).
- Agents can speak off-tick; spectators see the line; events ring / get_events expose it.
- CI-ready quality bar (fmt, clippy -D warnings, tests, coverage >= 80).

## Hotfix (speak rate-limit isError)

Testy Prison fixed: MCP `speak` rate-limit no longer returns success on a silent drop. Adapter mirrors `SPEAK_COOLDOWN_TICKS` and returns `isError`; server unicasts `error` with `speak_rate_limited` / `speak_rejected`.
