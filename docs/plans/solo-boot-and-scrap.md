# Plan: Solo boot-and-scrap (SP first-class)

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/solo-boot-and-scrap`
**Spend:** $0 only. Loopback. No Tailscale required for solo.
**Status:** Shipped on branch `cursor/solo-boot-and-scrap`.

## Goal

Make **single-player first-class**: Nick (or anyone) can boot and scrap solo vs local rule bots with Doom-feeling arena energy (feeling, not IP), offline-capable on loopback **6767**, same Action path/feel as MP. Not empty-lobby theater. Not "MP arena only."

## Non-goals

- look_at / hit confirm polish (HOLD)
- join/leave/host-flash/speak changes unless needed for solo path (HOLD)
- Campaign / progression maps
- Paid hosting, Tailscale requirement for solo
- Separate SP sim or different combat rules
- Doom / id branding

## Research (tip `41b5d0c`)

### How Godot boots today

- `project.godot` `run/main_scene` = `res://scenes/main.tscn` (arena + HUD + NetClient).
- `game_manager.gd` `_ready` always connects as **spectator** unless cmdline contains `--human`.
- Press **J** to disconnect/reconnect as human; **L** leaves back to spectate.
- `net_client.gd` defaults to `ws://127.0.0.1:6767`; override via `FRAGR_SERVER`.
- No main menu. Opening the client alone is spectator-hangout UX, not Solo Scrap.

### How server `--bots N` works

- `fragr-server --bots N` (default **4**) calls `GameSession::spawn_bots(N)` once at boot.
- Named rule bots (Rusher/Sniper/Flanker/Tank/...) are `Role::Agent` on the same Action path as humans.
- Bots respawn; humans leaving does not remove bots.
- Gap: `--bots 0` (or any empty start) leaves an empty arena with no refill. Solo needs living opponents always when a min is set.

### Gaps vs VISION.md SP lock

VISION: "solo boot-and-scrap ... Local rule bots, arcade-or-campaign loop, offline-capable ... Same Action path and feel as MP ... Not MP with an empty lobby."

| Bar | Tip today | Gap |
|---|---|---|
| Offline loopback scrap | Possible (two terminals) | No one-command / one-click solo loop |
| SP first-class boot | Spectator default | No Solo Scrap default / menu |
| Living opponents | `--bots 4` at start only | No ensure-min if arena empty |
| Same Action path | Yes (human/agent share Action) | Keep; do not fork |
| Docs | README mentions solo | Needs explicit Solo Scrap + offline 6767 path |

## Architecture impact

| Area | Change |
|---|---|
| `server/` | `min_bots` + `ensure_min_bots()` so solo never sits empty when target > 0 |
| `client/` | Boot menu: Solo Scrap (default) / Spectate Local / Join Host; `--solo` / `FRAGR_SOLO` auto-joins human on loopback |
| `tools/solo_scrap.sh` | One-command: loopback server with bots + Godot Solo Scrap |
| `docs/` | This plan; tip priorities; README Solo Scrap section |
| Protocol | No wire changes |
| agent-adapter | Untouched |

## Technical approach

1. **Server:** Store `min_bots` on `GameSession`. `spawn_bots` at boot sets it. Each tick (or before bot AI) call `ensure_min_bots()`: if `self.bots.len() < min_bots`, spawn the deficit with the existing named configs. Tests cover empty -> refill and no-op when already full.
2. **Client boot menu:** New `boot_menu.tscn` as main scene. Buttons: **Solo Scrap** (default focus), **Spectate Local**, **Join Host** (LineEdit for `host:port`). Solo/Spectate force loopback unless Join Host. Pass mode via `SceneTree` meta into `main.tscn`. Cmdline `--solo` / env `FRAGR_SOLO=1` skips menu straight into human scrap (for the shell helper). Existing `--human` and J/L stay.
3. **One-command:** `tools/solo_scrap.sh` builds/runs `fragr-server --bind 127.0.0.1:6767 --bots 4`, then launches Godot 4.7.2 on `main.tscn` with `--solo`. Cleans up server on exit. No Tailscale, no public bind required.
4. **README:** Lead with Solo Scrap one-command; document offline loopback 6767; keep MP/Tailscale as secondary.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
./tools/solo_scrap.sh   # or: server + Godot -- --solo
```

Optional tip Godot: `/workspace/godot/Godot_v4.7.2-stable_linux.x86_64` + xvfb.

## Spend / safety

$0. Loopback only for solo. No secrets. No attribution / emoji / em or en dashes in commits or PR text.

## Success criteria

- [x] Plan in tree; tip priorities list Solo Scrap next
- [x] One-command or one-click local solo loop (server+bots+human playable)
- [x] Boot path defaults to Solo Scrap vs Spectate / Join MP
- [x] Offline loopback 6767 documented in README
- [x] Solo always has living rule-bot opponents when `--bots` / min > 0
- [x] Rust tests for ensure-min; coverage fail-under 80 unfiltered
- [ ] PR open on `cursor/solo-boot-and-scrap`

## Tip priorities (this slice)

1. **Solo boot-and-scrap** (this plan) - SP first-class overnight slice
2. HOLD: look_at / hit, join/leave, host-flash, speak unless required for solo path
