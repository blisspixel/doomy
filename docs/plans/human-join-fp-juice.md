# Human join first-person scrap juice

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/human-join-fp-juice`
**Spend:** $0. No ElevenLabs, no GCP apply, no renet, no look_at/hit reopen.
**Status:** **shipped** (#74 / `094f941`). Tip face: Join FP scrap juice.

## Goal

Raise human join fun: when you Join (or Solo Scrap), the local pawn gets first-person scrap juice, not only the spectator third-person face.

Ship readable FP/local feedback:

- Center crosshair (bone / gunmetal, not neon)
- Held-weapon face (sprite viewmodel with light walk bob)
- Spawn flash and damage flash overlays (ember / blood grit)

Keep Contested Frequency pixel grit (bone / gunmetal / ember). Unreal+CS readability in a real 3D world, Doom-sprite energy without Doom branding.

## Tip priorities

Reconnect + Godot null shipped (#73). Human join FP juice shipped (#74). Next tip: killstreak Host juice. look_at / hit HOLD. Port 6767. Coverage fail-under 80.

## Non-goals

- GCP apply
- ElevenLabs
- look_at / hit reopen (HOLD; mouse turn stays discrete turn_left / turn_right)
- New map / choke layout
- Server sim or protocol shape changes beyond soft docs
- Neon HUD flood or photoreal viewmodels

## Soft docs (Testy)

Ensure `pickup` kind (`weapon` / `health` / `armor`) is listed in the protocol Event `event` enum summary if still missing. Pickup event shapes and Snapshot pad `kind` already document the three kinds; the enum bullet is the soft gap.

## Architecture impact

| Area | Change |
|---|---|
| `client` spectator_cam | FP mode: eye-height follow on local pawn; pitch client-only; yaw tracks server pawn |
| `client` game_manager | Enable FP on human join / Solo; mouse to turn_left/turn_right; hit/respawn local flashes; own weapon HUD |
| `client` hud | Crosshair, FP weapon sprite + bob, damage/spawn ColorRect flashes |
| `client` player_pawn | Hide local body/label while FP so the viewmodel owns the face |
| `client` main.tscn | HUD nodes for crosshair / flashes / FP weapon |
| `docs/protocol.md` | Add `pickup` to Event `event` enum bullet if missing |
| `docs/plans/README.md` | Tip priorities: reconnect shipped; this NOW |
| Tip stills | Recapture join FP still if Xvfb path works; else note follow-up |

## Behavior

1. Spectate: unchanged third-person follow / free-fly. No crosshair. No FP weapon.
2. Join / Solo (human): camera locks FP on local pawn (`net_client.player_id`). Crosshair on. Held weapon sprite shows current weapon with light bob while moving. Local pawn billboard hidden.
3. Mouse X while playing: drives `turn_left` / `turn_right` on Action (no look_at). Mouse Y: client pitch only.
4. Local `hit` event: brief blood/ember damage vignette. Local `respawn` (or first spawn after join): brief ember spawn flash.
5. Leave (L): restore spectator cam and clear FP juice.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
# Godot parse / import (optional local)
/workspace/godot/godot --path client --headless --import --quit-after 120
# Playable: server --bots 4, Godot Join or Solo, feel FP scrap juice
```

## Success criteria

- Plan in tree; tip priorities list this as NOW (reconnect shipped)
- Join mode feels scrapier: crosshair + weapon face + spawn/damage flash
- Protocol Event enum documents `pickup` (+ kind already on event / Snapshot)
- Rust tests green; coverage fail-under 80
- PR open; tip stills updated or follow-up noted

## Spend and safety

- $0 local only
- No cloud apply, no paid APIs

## Tip stills

Capture path: `tools/capture_tip_screenshots.sh` (Xvfb + opengl3). FP join still: `10_tip_human_join_fp_16x9.png`. Spectator tip stills remain the Contested Frequency gallery baseline.
