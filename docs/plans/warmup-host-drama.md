# Plan: Warmup / pre-round Host drama

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/warmup-host-drama`
**Spend:** $0. Loopback. No GCP apply, no ElevenLabs, no look_at/hit reopen.
**Status:** In flight (NOW tip after named scrap bots).

## Goal

Raise scrap-league feel: Warmup / pre-round countdown Host drama (named roster mention, map name, Contested Frequency bumper) so rounds open with energy, not a silent start.

Product locks: Contested Frequency Host parody; port 6767; fail-under 80; look_at/hit HOLD; no GCP apply; named bots + maps stay.

## Tip priorities

Named scrap bots (`f58b2d0` / #82) shipped. **This is the NOW tip.**

## Non-goals

- GCP apply / Terraform apply
- ElevenLabs / paid voice
- look_at / hit confirm reopen (HOLD)
- New map / choke layout (maps 1+2 stay)
- Neon HUD flood or Doom branding

## Research (tip `f58b2d0`)

| Seam | Tip today | Gap |
|---|---|---|
| Warmup Host | Sticky roster line names dialed-in scrap bots | No map name, no countdown, weak Contested Frequency bumper |
| round_time_left | Active only | Warmup countdown invisible on Snapshot / MCP |
| Godot Warmup | "WARMUP - Contested Frequency tuning in"; no flash | Countdown / map chrome not readable; silent until RoundStart |
| RoundStart | Exists with roster host_line | Keep; enrich Warmup so pre-round sells first |

## Architecture impact

| Area | Change |
|---|---|
| `server` protocol | `warmup_host_line` / `round_open_host_line` Contested Frequency helpers |
| `server` sim | Warmup Snapshot sticky host_line with roster + map + secs; `round_time_left` during Warmup; RoundStart uses round-open Host line |
| `server` session | Roster names feed Warmup drama (existing refresh path) |
| `client` HUD | Warmup round chrome shows map + countdown; readable Host chip |
| `client` game_manager | Flash Host once on Warmup Snapshot join (scrap energy) |
| `agent-adapter` | observe / round_state already surface Snapshot host_line + round_time_left |
| `docs` | This plan; tip priorities; protocol Warmup note |

## Wire

Warmup Snapshot example:

```json
{
  "type": "snapshot",
  "round_state": "Warmup",
  "round_time_left": 2,
  "map_name": "Arena Duel",
  "host_line": "HOST: CONTESTED FREQUENCY. ARENA DUEL TUNES IN. DEAD AIR DAN, NIGHTFALL ON THE SCRAP. 2."
}
```

RoundStart keeps fight bumper (map + roster, no countdown):

```json
{
  "event": "round_start",
  "host_line": "HOST: CONTESTED FREQUENCY. ARENA DUEL. DEAD AIR DAN, NIGHTFALL ON THE SCRAP. FIGHT!"
}
```

## Behavior

1. Session refresh stores rule-bot callsigns on sim.
2. Each Warmup Snapshot: compute Contested Frequency Host line from map + roster + remaining seconds; expose remaining seconds as `round_time_left`.
3. RoundStart clears Ended sticky and emits round-open Host line (map + roster).
4. Godot: Warmup round label shows map + countdown; sticky Host chip updates; mid-Warmup join flashes Host once.
5. MCP `observe` / `round_state` read Snapshot `host_line` and `round_time_left` during Warmup.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
```

No `--ignore-filename-regex`. No tool attribution, emoji, or em/en dashes in commits/PR/docs.

## How to see Warmup drama

1. `cargo run -p fragr-server -- --bots 4` (port 6767).
2. Open Godot client, spectate or Join during Warmup (first ~2s after boot / after Ended linger).
3. Round label shows map + countdown; Host chip names roster and Contested Frequency bumper.
4. MCP `round_state` / `observe` while Warmup: `host_line` carries drama; `round_time_left` counts down.
5. RoundStart still fires the fight bumper when Active begins.

## Success

- PR open; Warmup sells the scrap.
- Tip priorities: named bots shipped; this NOW.
- Protocol + adapter see Warmup host_line / countdown.
- Rust tests green; coverage fail-under 80.
