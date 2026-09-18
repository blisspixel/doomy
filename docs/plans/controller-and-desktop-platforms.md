# Controller support and desktop platforms (Win / Mac / Linux)

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/controller-platforms`
**Spend:** $0. No ElevenLabs, no GCP apply, no renet, no look_at/hit reopen.
**Status:** in flight (NOW). Tip face: gamepad join + first-class desktop exports.

## Goal

Human Join / Solo Scrap playable with a gamepad on the same Action path as keyboard/mouse. Document and ship Godot desktop export presets for Windows, macOS, and Linux as first-class. Prefer Godot 4.x built-in InputMap joypad actions. Keep Contested Frequency feel.

## Tip priorities

Public or local join shipped (#84). Controller + Win/Mac/Linux NOW. look_at / hit HOLD. Port 6767. Coverage fail-under 80. Public OR local. Zero attribution.

## Non-goals

- GCP apply
- ElevenLabs
- look_at / hit reopen (HOLD; mouse and stick turn stay discrete turn_left / turn_right)
- Second control plane or agent-only input path
- Mobile / Web / console export SKUs
- Server sim or protocol shape changes (weapon_swap and speak already on wire)
- Neon HUD flood or Co-authored-by / Made-with badges

## Architecture impact

| Area | Change |
|---|---|
| `client/project.godot` | Joypad bindings beside existing keys (sticks, triggers, bumpers, Start, face buttons) |
| `client` game_manager | Join/leave/speak/weapon cycle via InputMap; same Action dict |
| `client` spectator_cam | Right-stick look/turn; F-cycle targets; gamepad works when mouse free |
| `client` net_client | Forward optional `weapon_swap`; `speak` helper |
| `client` hud / boot_menu | Controller works hint; deadzone note in docs |
| `client/export_presets.cfg` | Windows Desktop, macOS, Linux/X11 runnable presets |
| `README.md` | Controls map + run from editor + export builds; Rust server note |
| `docs/plans/README.md` | Tip priorities: this NOW; public-or-local shipped |

## Control map (ship face)

| Action | Keyboard / mouse | Gamepad |
|---|---|---|
| Move | WASD | Left stick |
| Look / turn | Mouse | Right stick (yaw -> turn_left/turn_right; pitch client-only) |
| Fire | LMB | RT / R2 or A |
| Weapon cycle | `[` / `]` | LB / RB |
| Speak / taunt | T | Y |
| Join | J | A (while spectating) |
| Leave / menu | L / Esc (mouse) | Start |
| Spectator cycle | F | D-pad right / Select cycle |
| Free-fly toggle | V | Back |

Stick deadzone: **0.25** on InputMap axis actions (documented). Same Action path; no second plane.

## Behavior

1. Spectate: F / gamepad cycles follow targets. Left stick free-fly when free-fly on. Right stick looks.
2. Join / Solo: left stick move, right stick turn (Action bits) + local pitch, RT/A fire, bumpers weapon_swap, Y/T speak, Start/L leave.
3. weapon_swap cycles flechette -> rail -> scatter on existing Action field.
4. speak sends off-tick Speak control message (rate-limited server-side).

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
/workspace/godot/godot --path client --headless --import --quit-after 120
# Playable: server --bots 4, Godot Join or Solo with gamepad plugged in
```

Godot-only change is OK if Rust untouched; keep CI green.

## Success criteria

- Plan in tree; tip priorities list controller + desktop platforms as NOW
- Gamepad playable Join / Solo Scrap on same Action path
- export_presets.cfg for Windows / macOS / Linux; README run + export docs
- Deadzone documented; UI hint that controllers work
- PR open; CI-ready; no attribution / emoji / em or en dashes

## Spend and safety

- $0 local only
- Port 6767; public OR local; fail-under 80; look_at/hit HOLD
