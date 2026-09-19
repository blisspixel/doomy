# Plan: rebuild the HUD, because trimming it is not enough

**Status:** spec (2026-09-19)
**Branch:** `feat/hud-rebuild-*` (one PR per stage)
**Spend:** $0. Layout, a display font, and sprites from the locked palette.

`plans/hud-quiet.md` cut what the old HUD said twice. This plan replaces what is left. The cut was necessary and not sufficient: the thing being trimmed was a debug overlay, and no amount of trimming turns a debug overlay into a heads-up display.

## The brutal version

Held against DUSK, Amid Evil, Prodeus, Cultic, Ultrakill and Selaco, here is what a current fragr frame gets wrong.

**1. The three numbers that matter are not on screen.** Health, armour, ammo. A player has no idea how close they are to dying. "Status: Connected to server" was on screen and health was not. Nothing else in this list matters as much as this one.

**2. The top-left block is a debug overlay wearing a HUD's clothes.** A translucent black rectangle, roughly 340 by 320, holding fourteen lines of small prose in the stock Godot font. DUSK's entire HUD is four numbers and a face. Doom's is one bar. No shipped boomer shooter has a paragraph on screen during play.

**3. The ON AIR strip is the worst single element.** It is a glossy, vector-styled broadcast bar pasted across the top centre of a pixel-art game. It does not share the world's palette, its resolution, or its edge treatment, so it reads as a web banner someone dropped on the render. It is also parked exactly where the killfeed belongs, and its saturated red is the highest-contrast thing on screen, which means the eye goes there instead of the crosshair, permanently. Cut it from gameplay. If the broadcast conceit needs a mark, it is a small monochrome chip in one corner, in the world's palette, at the world's pixel size.

**4. Nothing is styled.** Stock font, thin strokes, no outline, no drop shadow, three sizes chosen ad hoc, five competing colours. The red killfeed line over the crate wall is unreadable. Every boomer shooter HUD uses a heavy display face with a hard outline precisely so it survives any background.

**5. The view model is too small and boxed in.** It sits at roughly 240 by 200 in the corner with an unexplained dark panel behind it. In DUSK, Prodeus and Cultic the weapon occupies a third of the screen height and anchors the bottom-right. A small gun in a box reads as an inventory icon, not as the thing in your hands.

**6. The crosshair is a one pixel plus.** It disappears against the floor. It does not change on hit, kill, or spread.

**7. World-space text is competing with the HUD.** Nameplates spell out `Aunt Linda [BAL] +1 [100 HP]` in a font large enough to hide the fighter wearing it, and host lines land across the middle of the screen, sometimes two at once in the same place.

**8. No layout grid.** Elements sit at hand-picked offsets with no shared margin, so nothing lines up with anything and the composition reads as accidental, which it is.

## Who the HUD is for

Spectator and free-fly are worth having and worth making fun to watch, and the broadcast conceit belongs to them. But a person playing fragr is playing a first-person boomer shooter, in the Doom sense: they are behind the gun, the world fills the screen, and the chrome lives at the edges. Every decision below is made for that player. If an element exists to serve the spectator view, it belongs in the spectator view and not over a firefight.

## How loud the broadcast gets

The station is a thread through the world, not the world. It had been given the top of the screen, two badges, the playlist line, a map chip and a round bumper, so a player who had never heard of Contested Frequency was told about it four times a frame. The signal belongs where watching a broadcast is the point, which is the spectator view and the round bumper. A person behind a gun gets the world, not the network ident. Written up in `docs/LORE.md`.

## The target frame

Everything a player needs, nothing else, in the corners:

- **Bottom left:** health as a big number with a cross icon, armour beneath it as a smaller number with a shield icon. Chunky display font, hard outline, palette red and palette blue.
- **Bottom right:** the weapon, large, its silhouette readable, with ammo as a big number beside it. No panel behind it.
- **Top right:** killfeed, three lines maximum, small, fading out, icons for the weapon that did it.
- **Top left:** score and the clock, two short values, nothing else.
- **Centre:** the crosshair, and never anything else. Host lines go to a band above the bottom edge, one at a time, queued.
- **Everywhere:** one display font, two weights, hard outline, a five colour palette tied to meaning.

## What has landed (2026-09-19)

| State | Before the HUD work | Now |
|---|---|---|
| First person | 19.9% | **4.8%** |
| Combat follow | 22.1% | **9.4%** |
| Warmup spectator | 23.3% | **7.4%** |

The ceiling this plan set was twelve percent. The first-person frame is at a third of it.

- **Health and armour are on screen.** A big number with a bar under it bottom-left, armour beside it, red and blue. Health turns pale under thirty-five. Armour at zero fades rather than shouting a nought. There is no ammo to show, because the weapons are cooldown-gated and the protocol has no ammo field; that is a question for `gunfeel.md`.
- **The station is out of gameplay.** The broadcast strip and its badges are spectator furniture now, along with the followed-fighter line, the weapon icon in its box, the map chip, and the league and playlist line. A spectator keeps all of it, because watching a broadcast is the point of that view.
- **There is no box behind the HUD.** The panel paints nothing and every label carries a hard outline, which is what the box was for. That single change is most of the drop in the table, because the coverage was never the words.
- **Nameplates are capped.** A `Label3D` has a fixed size in the world, so a fighter two metres away wore a name that hid the room. Below nine metres it shrinks with the distance, with a floor so it does not vanish at contact range, and beyond that it follows the far camera's curve as before.
- **The crosshair can be seen.** Every part of it has a dark edge two pixels larger on each side, following whatever shape the weapon chose, including the sizes the code changes at runtime.
- **The player can see their own gun fire.** Recorded in `gunfeel.md`: there was no first-person muzzle flash at all, and the view-model kick only played when the shot missed.

What is left on a playing screen is the host line and, for eight seconds after joining, the control legend.

## Stages

1. ~~**Put the numbers on screen.**~~ Landed. Health and armour; there is no ammo to show yet. Health, armour, ammo with icons in the corners, in a display font with an outline. The debug block goes behind a key, off by default. Gate: a still shows health, armour, and ammo, and HUD coverage in first person falls below 12 percent.
2. ~~**Kill the ON AIR strip in gameplay.**~~ Landed, along with the rest of the spectator furniture. It survives as a small palette-correct chip in warmup and on the round bumper, where the broadcast conceit is the point. Gate: the strip does not appear in any playing state's still.
3. **The weapon gets its corner.** View model to roughly a third of screen height, no backing panel, ammo beside it.
4. **One font, one grid.** A display face with outline, a shared margin, and every element snapped to it.
5. **The crosshair answers.** Half landed: it can be seen now. Hit tick and kill confirm still to do. Hit tick, kill confirm, spread that matches the weapon.
6. **Playing means first person.** Joining puts a person behind the gun with the playing HUD, and the spectator chrome, the broadcast strip, and the follow-camera furniture stay with the spectator view where they belong.
7. **Nameplates become a bar and a chip,** size now capped; from `plans/hud-quiet.md` rung 3, and host lines leave the centre.

## Verification

The tour measures it. HUD coverage per state with a 12 percent ceiling, no HUD text in the central tenth of the screen, and a still per stage showing the three numbers present and legible at 480 by 270.

## Related

- `plans/hud-quiet.md`: the cuts that came first, and the findings that started this.
- `plans/look-pass-boomer.md`: the world underneath, including the render target and palette.
- `plans/gunfeel.md`: the crosshair and the firing feedback.
