# Plan: a HUD that shuts up

**Status:** in progress (2026-09-19). Rung 1 of the visual QA tour landed and took the first measurements.
**Branch:** `feat/hud-quiet-*` (one PR per rung)
**Spend:** $0. Layout, sprites from the locked palette, no paid assets.

## Goal

A player should be able to tell health, armour, weapon, ammo, and who is winning at a glance, without reading a sentence. Today the tip does the opposite: the screen carries six blocks of prose, two of them clipped off the edge of the window, and a nameplate large enough to hide an enemy behind it.

The reference is a boomer shooter HUD, not a simulation readout: sprites, bars, and numbers with an icon beside them. `plans/look-pass-boomer.md` owns how the world is drawn. This plan owns everything drawn on top of it.

## What the first tour measured (2026-09-19)

`tools/qa_tour.sh`, eight states, 1280 by 720, Godot 4.7.2, with the game paused for each measurement so world motion is not counted as chrome.

| State | HUD coverage |
|---|---|
| boot menu | 0.0% |
| warmup spectator | 23.3% |
| arena overview | 18.9% |
| combat follow | 22.1% |
| first person | 19.9% |
| first person with a pickup in view | 20.1% |
| compliance pressure | 22.4% |
| killfeed and scoreboard | 20.5% |

About a fifth of the screen, in every state, before the world-space nameplates and the centred host lines are counted, because those are drawn in the scene rather than the HUD layer and so do not show up in this number at all. The true figure a player sees is worse than the table.

## Findings, in the order they hurt

1. **The status panel is clipped off the left edge.** Every line in the top-left block starts mid-word: "CE DRONE ON DECK", "ove/look, RT/A: Fire", "LIANCE DRONE". The panel is anchored so that part of it sits outside the window. A player cannot read the one part of the HUD that is pure text.
2. **The control legend is on screen during play.** "Move/look, RT/A: Fire, LB/RB: Weapon, Y/T: Speak, R/M or D-pad: Radio, ESC: Mouse" belongs in a pause or settings screen, not over a firefight.
3. **World-space nameplates are enormous and collide.** "Static Kid [FLK]" rendered across the top of the screen at a size that hides what is behind it, and in the spectator states three nameplates overlapped each other and the killfeed at once. Name, stance, score, and health are all spelled out in words: `Aunt Linda [BAL] +1 [100 HP]`.
4. **Host lines are centred prose over the middle of the screen.** "HOST: CONTINUANCE COMPLIANCE PING. APPROVED LANES ONLY." sat directly across the crosshair, overlapping a second line, "CONTESTED FREQUENCY PRESSURE", in the same place.
5. **Badges are drawn twice.** The chrome strip carries ON AIR, Contested Frequency, and Hangar Candy along the top, and a second, smaller ON AIR and Contested Frequency sit immediately below it. One of the two is redundant.
6. **An unstyled black panel sits behind the weapon in the bottom-right corner** and clips the view model.
7. **The crosshair is a bare plus with no feedback.** Nothing changes on a hit, a kill, or a reload.

## Rung 1 landed (2026-09-19)

Nothing is clipped off the window and nothing is drawn twice. The first-person panel went from twenty lines to ten.

| | Before | After |
|---|---|---|
| Lines of text in the playing panel | 20 | 10 |
| HUD coverage, first person | 19.9% | 18.8% |
| HUD coverage, arena overview | 18.9% | 15.5% |
| HUD coverage, warmup spectator | 23.3% | 22.1% |

What was cut, and why each was a duplicate rather than a judgement call:

- The status panel grew in both directions when its text outran its width, so half of every long line sat outside the window. It grows to the right now, every label wraps, and the panel clips as a safety net.
- The ON AIR and Contested Frequency badges were drawn under a chrome strip that already bakes both, the same way Hangar Candy was already being suppressed. They are the fallback for when the strip is down.
- The map name was on screen three times: in the playlist line, in a MAP line under it, and in the corner chip. The corner chip keeps it.
- The host line and the pressure line said the same sentence one above the other. The pressure line now only appears when there is no host line.
- The leader and rival line named the top two fighters directly above a scoreboard whose first two rows are the top two fighters.
- The scoreboard carried two header lines repeating the league and playlist already on the panel's first line, and listed eight names, which ran the panel off the bottom of the screen. Four names, no headers.
- Connection status, wall clock, and head count are for whoever is debugging the client. They are hidden while playing, where the round line already carries the clock.
- The control legend now shows for eight seconds after joining and then gets out of the way. It still belongs in a settings screen, which is rung 2.
- The panel behind the weapon icon sat flush in the corner with nothing in it. It is inset and only appears when the icon it backs does.

The number that did not move much is the interesting one. Coverage fell only a point in first person while the text halved, because most of the coverage is the dark panel behind the words rather than the words. Rung 2 should make the panel fit its content instead of reserving a block.

## Rungs

1. ~~**Nothing clipped, nothing duplicated.**~~ Landed 2026-09-19, table above. Anchor every HUD element inside the safe area, delete the duplicate badge pair, remove the black panel behind the view model. Gate: the tour's stills show no element crossing a window edge, checked by sampling the outer eight pixel border of the HUD layer for opaque content. Evidence: before and after stills in the same state.
2. **The legend leaves the game.** Controls move to the settings screen and a first-run overlay that the player dismisses. Gate: HUD coverage in the first-person state drops below 12%.
3. **Nameplates become a bar and a chip.** Health as a short bar, stance as the existing stance chip sprite, score as a number with an icon. The name stays, at a size that does not exceed a fixed fraction of screen height at any distance, with a hard cap on how many draw at once and a fade by distance. Gate: three fighters in frame at eight units produce no overlapping nameplate rectangles, asserted in the tour.
4. **Host lines move out of the centre.** A single line, one at a time, in a fixed band that never crosses the crosshair, with a queue rather than two lines stacked on the same pixels. Gate: no HUD text within the central tenth of the screen in any tour state.
5. **A crosshair that answers.** Hit tick, kill confirm, weapon-shaped spread. This is the one place to add rather than remove, and it belongs with `plans/gunfeel.md` since it is the same feedback loop as the guns.
6. **Sprites for everything countable.** Health, armour, and ammo as icon plus number. No word on screen outside the killfeed, the scoreboard, and menus.

## Verification

- The tour runs on every rung and the manifest records HUD coverage per state, so a regression is a number that went up, not an argument.
- A ceiling in the tour: any state above 12% HUD coverage fails once rung 2 lands, in the same spirit as the playtest thresholds.
- Readability at 480 by 270, since `plans/look-pass-boomer.md` renders the world at a quarter resolution.

## Related

- `plans/look-pass-boomer.md`: how the world underneath is drawn.
- `plans/visual-qa-tour.md`: the tour that produced these numbers.
- `plans/gunfeel.md`: crosshair feedback is part of how a gun feels.
- `plans/art-pipeline.md`: where the icons and chips come from.
