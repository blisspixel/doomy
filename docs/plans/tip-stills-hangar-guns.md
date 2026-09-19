# Tip stills: v0.13.0 hangar / guns face

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `docs/tip-face-hangar-guns`
**Spend:** $0. Loopback. No Cloud Agent. No soft louder stance. No TTK proof tag.
**Status:** in flight
**Tip base:** main `cc4eff9` (#125 live tip_capture hangar dish + Hangar Candy dual chip gone; includes #124 guns that kill).

## Goal

Refresh README tip embeds so strangers see the **v0.13.0 hangar / guns** face (dish in hangar, guns that kill), not stale Episode 0 Calibration stills that still read Hangar Candy dual chip. Recapture via live `tip_capture` / `docs/screenshots` pipeline on tip. Keep README a product front door (no lore sludge).

## Tip priorities

#125 shipped live hangar dish latch + chrome strip hide. #124 shipped guns that kill. **README tip face refresh is the NOW tip.** Soft louder #119 stance chips and TTK Casino proof are not this PR (Testy Casinos `cc4eff9` in parallel).

## Non-goals

- Soft louder spectator stance chips
- TTK / dish-claim release tag or Casino proof writeup
- New gameplay, protocol, or HUD features
- Relabeling mood art as tip proof
- Lore sludge on the README front door

## Locks

- Contested Frequency pixel grit
- Port 6767
- Coverage fail-under 80
- Attribution lock, no emoji, no em/en dashes
- Nick Seal / blisspixel noreply authorship only

## Stills to refresh

| File | Why |
|---|---|
| `20_jammer_dish_follow_16x9.png` | README lead: dish at follow in live hangar |
| `22_jammer_dish_overview_16x9.png` | README: dish at overview |
| `23_jammer_dish_seize_label_16x9.png` | SEIZE JAMMER label under HUD |
| `09_tip_weapons_frags_16x9.png` | Guns that kill combat still |
| `10_tip_human_join_fp_16x9.png` | Join FP weapon face |
| `01_arena_overview_16x9.png` | Arena + Contested Frequency after #125 |
| `02_spectator_hud_16x9.png` | Host / scoreboard without Hangar Candy strip |
| `07_tip_compliance_pressure_16x9.png` | Mid-scrap killfeed |
| `08_tip_host_flash_midjoin_16x9.png` | Mid-join Host flash |
| `11_tip_warmup_tv_bumper_16x9.png` | Warmup TV (table; not README lead) |
| `12_tip_calibration_larak_16x9.png` | Calibration chrome (secondary; demoted from README lead) |

## Capture path

1. `cargo run -p fragr-server -- --bind 127.0.0.1:6767 --bots 4 --solo-broadcast` (or reuse Testy loopback on `cc4eff9`)
2. Godot 4.7.2-stable on PATH as `godot`
3. `tools/capture_tip_screenshots.sh` (Xvfb + opengl3)
4. Driver: `client/scripts/tip_capture.gd` (Calibration + Warmup TV + timed scrap + jammer dish proof + mid-join + Join FP)

## Docs updates

- `README.md`: Screenshots section leads with dish + weapons; drops Calibration / Warmup as front-door lead
- `docs/screenshots/README.md`: tip table + status text for hangar / guns face; Calibration marked secondary
- `docs/plans/README.md`: this plan indexed; `live-tip-dish-map-chip.md` marked shipped (#125)

## Verification

```bash
# tip_capture against --solo-broadcast
tools/capture_tip_screenshots.sh
# Inspect: 20/22/23 dish in hangar; 09 guns; no Hangar Candy strip; no pink frames
```

## Success criteria

- [ ] Plan in tree; index row present; #125 plan shipped
- [ ] README tip gallery leads hangar dish + guns
- [ ] Tip stills recaptured on `cc4eff9` pipeline
- [ ] One lean PR; squash-merge when CI green
- [ ] No soft stance / TTK work in this PR

## Spend and safety

- $0 local only
- No cloud apply, no paid APIs
- No tool attribution, emoji, or em/en dashes
