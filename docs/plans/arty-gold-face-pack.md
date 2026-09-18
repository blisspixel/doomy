# Plan: Arty gold face pack (Cyanex / Kragge + broadcast chrome)

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/arty-gold-face`
**Base tip:** `79180c7` (public OR local join #84)
**Spend:** $0. Loopback. No GCP apply. Godot-only preferred.
**Status:** In flight (visual NOW tip).

## Goal

Wire Arty gold face pack into tip so fighter billboards and HUD chrome read Contested Frequency / Hangar Candy scrap grit: muted cyan Cyanex, muted magenta Kragge, ON AIR / Contested Frequency / Hangar Candy chrome. Readable agents. Not neon flood. Not Doom IP.

## Tip priorities

Public OR local join shipped (#84). **This is the visual NOW tip.**

## Non-goals

- GCP apply / Terraform apply
- ElevenLabs / paid voice
- look_at / hit confirm reopen (HOLD)
- Rust / server protocol changes (coverage fail-under 80 stays; prefer Godot-only)
- Neon HUD flood or Doom branding
- Tool attribution in commits / PR body

## Locks

- Doom-sprite billboard grit craft only (no Doom IP)
- Pixel grit surface on 3D arena
- Port 6767
- Coverage fail-under 80 if Rust touched
- look_at / hit HOLD
- No GCP apply
- Public OR local join (Tailscale test-only)
- Zero tool attribution

## Asset source

Box pack at `/workspace/fragr-game-assets/` (Godot-ready):

- `client/assets/characters/64/` Cyanex / Kragge idle + strips + sheet (+ `@4x`)
- `client/assets/ui/` `on_air`, `contested_frequency`, `hangar_candy`, `chrome-strip`
- Docs plates: `docs/agent-plates/`, `docs/ui-chrome/`

HUD-sized badge crops derived into `client/assets/ui/*_badge.png` and `chrome_strip_hud.png` for tip face (nearest, no neon).

## Architecture impact

| Area | Change |
|---|---|
| `client/assets/characters/64` | Overwrite idle billboards with gold pack |
| `client/assets/ui` | Add broadcast chrome + compact HUD badges |
| `client/scenes/main.tscn` | TextureRect chrome nodes on HUD |
| `client/scripts/hud.gd` | Show / hide chrome by Warmup / Active / Host face |
| `client/scripts/player_pawn.gd` | Keep strip paths; gold PNGs load via existing Cyanex / Kragge wiring |
| `docs/agent-plates` `docs/ui-chrome` | Plates for readable agent / chrome reference |
| `docs/plans` | This plan; tip priorities (visual NOW) |

## Wire

1. Copy gold character PNGs over tip idle billboards (Cyanex muted cyan; Kragge muted magenta Hangar Candy).
2. Land UI chrome plates + HUD badge crops under `client/assets/ui/` with nearest `.import`.
3. HUD: top chrome strip always on while connected; Contested Frequency badge warm on Warmup / Host flash; ON AIR lit on Active; Hangar Candy grit badge stays quiet in the corner.
4. player_pawn continues `cyanex_idle_strip` / `kragge_idle_strip` (4-frame bob); brand color stays on labels, body near-white multiply.
5. Tip stills refresh if Xvfb capture path works; otherwise leave prior stills and note in PR.

## Behavior

- Warmup: Contested Frequency badge + chrome strip sell tuning-in scrap.
- Active: ON AIR badge lit (broadcast chrome only; dull red, not neon).
- Ended / disconnect: chrome dims; ON AIR off.
- Host bumpers (Warmup / mid-join / RoundStart) keep text drama; badges reinforce without cluttering killfeed.

## Verification

```bash
# assets present
test -f client/assets/characters/64/cyanex_idle_strip.png
test -f client/assets/characters/64/kragge_idle_strip.png
test -f client/assets/ui/on_air_badge.png
test -f client/assets/ui/chrome_strip_hud.png

# Godot-only: no Rust churn
git diff --stat origin/main...HEAD | grep -E '^(server|agent-adapter)/' && exit 1 || true
```

Optional tip stills: `tools/capture_tip_screenshots.sh` under Xvfb when server + Godot available.

## Success

- Plan in tree; tip priorities list this as visual NOW (public OR local shipped)
- Cyanex / Kragge gold idle live on tip face via existing billboard paths
- Contested Frequency / ON AIR / Hangar Candy chrome live on HUD / Warmup / Host face
- PR open on `cursor/arty-gold-face`; CI-ready; Godot-only (no coverage reopen)
