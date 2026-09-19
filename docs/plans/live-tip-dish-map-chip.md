# Plan: Live tip_capture dish + Hangar Candy dual chip

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `feat/live-tip-dish-map-chip`
**Spend:** $0. Loopback. No Cloud Agent. No second dish-claim tag.
**Status:** shipped (#125). Soft Prison live re-run gap: see tip-capture-dish-gate.md.
**Tip base:** main `cee91d6` / `v0.13.0` (#123 dish + #124 guns). Casino on #123: studio dish_proof CLEARS; live tip_capture / FP hangar still miss.

## Goal

Stranger tip face must show the jammer dish in **live** tip_capture / hangar FP stills (chunky SEIZE JAMMER that survives dark racks + killfeed), and must not read Hangar Candy as a second map name beside Larak Lot. Prefer both.

## Root cause (verified)

1. **tip_capture force-spawn dies to Snapshot:** `_capture_jammer_dish_proof` calls `_sync_jammer_dish(live)`, then nods-phase Snapshots arrive with `jammer_dish: null` and `_sync_jammer_dish(null)` hides the node. Live hangar stills (20 to 24) show racks / orange pickup cube / killfeed with no bowl.
2. **Studio void != live hangar:** `capture_jammer_dish_proof.gd` black-void stills CLEARED Casino dish_proof; committed docs PNGs are that path. Tip face is tip_capture against `--solo-broadcast`.
3. **Hangar Candy dual chip:** `#123` hid `HangarCandyBadge` and added MapChipLabel (LARAK LOT), but `chrome_strip_hud.png` still bakes ON AIR + Contested Frequency + **Hangar Candy** on the top strip. Strangers read Hangar Candy as the map beside bottom LARAK LOT.

## Ship

1. Plan + plans README index.
2. `game_manager.tip_force_jammer_dish`: while latched, null Snapshot dish keeps the forced live spawn (Warmup-TV pattern). tip_capture sets latch before jammer proof + Join FP dish look.
3. tip_capture: force live dish + aimed follow / overview / label / Join FP look-at-origin so hangar stills include the bowl + SEIZE JAMMER under HUD.
4. HUD: when Snapshot map is Larak Lot (Solo Broadcast), hide `ChromeStrip` (Hangar Candy baked in). Keep OnAir + ContestedFrequency badges + MapChipLabel. HangarCandyBadge stays off.
5. Recapture live tip_capture jammer stills into `docs/screenshots/` (hangar + HUD, not studio void). Update screenshots README.
6. Mark `jammer-dish-unmissable.md` shipped (#123); this plan owns the live tip face gap.

## Non-goals

- Second dish-claim release tag
- Soft louder #119 stance chips
- Whole-main Casino
- New chrome art atlas (hide strip is enough)
- Server seize radius / phase timing changes

## Architecture impact

| Area | Change |
|---|---|
| `docs/plans/live-tip-dish-map-chip.md` | This plan |
| `docs/plans/README.md` | Index row; #123 plan shipped |
| `client/scripts/game_manager.gd` | tip_force_jammer_dish latch |
| `client/scripts/tip_capture.gd` | Latch + force + aim live hangar stills |
| `client/scripts/hud.gd` | Hide ChromeStrip on Larak Lot |
| `docs/screenshots/*jammer*` | Live hangar proof PNGs |
| `docs/screenshots/README.md` | Note live hangar vs studio |

## Verification

```bash
bash tools/godot_check.sh
# tip_capture against --solo-broadcast: 20/22/23 show dish in hangar; no Hangar Candy top chip beside LARAK LOT
```

## Success

- One lean PR on `cee91d6`, squash-merge when CI green
- Live tip stills include dish; Hangar Candy strip off during Solo Broadcast
- No dish-claim tag
