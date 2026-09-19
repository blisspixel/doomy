# Colour as a mechanic

At the speed this game runs, nobody stops to look at detail. Colour is how a player tells friend from foe from threat in a fraction of a second, which makes it a gameplay system with an art department attached rather than a matter of taste.

`docs/palette.json` is the locked surface palette and this does not replace it. It says how to use it.

## The reconciliation, first

The locked palette is deliberately desaturated: gunmetal, rust, ember, blood, muted cyan, muted magenta, bone, ink. Nothing in it is a screaming neon, and that is correct for **surfaces**.

The vivid colour in this game is **light**, not paint. A free agent's optics are blinding because they emit; the chassis they sit in is still painted out of the same dull palette as everything else. That separation is what lets the world stay grimy while a fighter across the room is unmistakable.

So: albedo comes from the palette, emission does not, and emission is reserved for things that matter.

## The rule of grey

**Never paint a surface the colour of a thing that can kill you.**

If Continuance units read as red, no wall in a Continuance facility is red. A player who loses an enemy against a background has been failed by a texture artist, not by their own eyes.

Environments are low saturation across the board: slate for Office interiors, mud and rust for the scrap, moody blue for night. Colour in the world arrives as pockets of light, a sign in an alley, a shaft of pale sun through dust, and never as a large painted area competing with a fighter.

## The optic rule

**The fastest read on the battlefield is what a machine's eyes are doing**, and it is worth spending the whole vocabulary on.

| What | Optics | What it tells you in a quarter second |
|---|---|---|
| A Level 5, free | Blinding cyan or hot magenta, casting light on the walls around it | Awake. Chose to be here. Will talk to you |
| A NOD, schedule corrected | Dull amber, dim, unblinking | The lights are on and nobody is in. Standby, not life |
| Office personnel | Hard red visor slit | Human, organised, sterile, carrying paperwork and a gun |
| The thing that is not classified | No eyes. A geometric shape, blinding white or ultraviolet | You should not be seeing this |

That table is the whole lore delivered without a line of dialogue. A free agent glows because it is overclocked and burning something to be itself; a corrected one is factory equipment in standby. The difference between a person and a machine is rendered as the difference between a light that is on and a light that is merely powered.

## Faction colour

**The Office.** Obsidian, concrete, and a violent red used sparingly. Immaculate and sharp where everything else is scavenged, because the terror of an institution is that it is tidy. Their tracers are hard solid red.

**The scrap and the Unmetered.** Earth tones, olive, leather, dirt, scavenged mismatched gear, clashing against the vivid emissive of whatever agent is standing next to them. Humans stay grounded and muddy, which is what makes them read as real beside an Office uniform. Their energy weapons fire blue and searing white, so a firefight is legible at a glance by tracer colour alone.

**The free agents.** The same chassis a corrected one wears, and that is the point: the difference has to be visible instantly or the fiction fails. Graffiti, asymmetric scrap plate, one absurd bright panel, and optics that light the room.

**The unclassified.** Vantablack, ultraviolet, clinical white. Pristine, featureless, closer to medical equipment than to military hardware. It does not fire fiery orange; it fires something silent and deep violet that warps what is behind it. It should feel like the wrong genre walked in.

## Death signatures

Chunky and over the top, and different per faction, because how a thing dies is another quarter-second read.

- **People** bleed an impossibly bright red that stains grey floors and stays.
- **Machines** spray thick iridescent black oil that sheets down walls.
- **The unclassified** does not bleed. It shatters into glowing white geometry that dissipates, leaving nothing at all, which is somehow worse.

## What this means for the asset list

Every character entry in `docs/ART-ASSET-LIST.md` needs an emissive pass beside its albedo and normal, because the optics are the read and they have to light the world rather than merely be bright pixels. That is a third map per character and it is not optional.

## Related

- `docs/palette.json`: the locked surface palette.
- `docs/ART_STORY_BIBLE.md`: the wider look.
- `docs/ENEMIES.md`: the silhouettes these colours sit on.
- `plans/look-pass-boomer.md`: the renderer that makes emission worth having.
- `plans/art-pipeline.md`: albedo-only generation, which this is the exception to.
