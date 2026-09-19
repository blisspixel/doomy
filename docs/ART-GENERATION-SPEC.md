# Art generation spec

The machine-facing half of the art pipeline. [`ART-ASSET-LIST.md`](./ART-ASSET-LIST.md) says what the game needs. [`plans/art-pipeline.md`](./plans/art-pipeline.md) says why the pipeline is shaped this way. This file is what a generation run executes: prompts, parameters, naming, and the checks a frame has to pass before it is committed.

It is written to be run by a program with an API key, not typed into a web interface.

## The contract every generated frame meets

Repeated here because it is the whole reason the pipeline exists, and because a run that ignores it produces assets that look wrong in a way no amount of retrying fixes.

1. **Albedo only.** No baked shadow, specular, ambient occlusion, rim light, or cast shadow. The engine lights the scene. A frame with a shadow painted into it is lit twice and reads as fake the moment a muzzle flash goes off beside it.
2. **Flat, even illumination** in the prompt, and every lighting word in the negative prompt.
3. **Transparent background** for anything that is not a tile.
4. **One palette.** Generated at 24-bit, then quantised to `docs/palette.json` in CIE L\*a\*b\* by `tools/pixelforge`. Prompting a palette helps and does not substitute for the quantisation step.
5. **Authored at four times target, downscaled by an exact integer.** A 64-pixel sprite is generated at 256. Generators cannot reliably emit small canvases, and a non-integer downscale destroys the pixel grid.
6. **A normal map beside every albedo**, derived from a depth pass. See "Normals" below.

## Prompt construction

Every prompt is assembled from four parts, in this order, so that a change to the house style is one edit rather than two thousand.

```
<STYLE> <SUBJECT> <VIEW> <TECHNICAL>
```

**STYLE** is fixed for the whole project and never varies:

> retro pixel art sprite, 1990s PC shooter, chunky readable silhouette, limited palette of desaturated rust orange, bone white, gunmetal grey and deep shadow, flat even lighting, no gradients

**SUBJECT** is the entry from the asset list, written as a noun phrase with the one or two details that make it readable at 64 pixels. Silhouette first, detail second. At 64 pixels a face is four pixels; a shoulder line is twenty.

**VIEW** is the camera, and for characters it is the direction:

> front view, orthographic, centred, full body, feet at the bottom edge

**TECHNICAL** is fixed:

> transparent background, no shadow, no ambient occlusion, no specular highlights, no rim light, no ground plane, no text, no watermark, no border

**Negative prompt**, fixed for everything:

> photorealistic, 3d render, smooth shading, gradient, blur, anti-aliasing, drop shadow, cast shadow, ambient occlusion, specular, glossy, reflection, rim light, lens flare, bloom, depth of field, text, signature, watermark, frame, border, background scenery

## Parameters

| Parameter | Value | Why |
|---|---|---|
| Size | 4x the target, square | Exact integer downscale; see the contract |
| Seed | Recorded per frame | A seed that produced a keeper is how the next direction stays on model |
| Batch | 4 per prompt | Pick one, keep the seed, discard the rest. Cost per usable frame is what matters, not cost per generation |
| Guidance | Mid range | High guidance flattens silhouettes into symmetry, which is the single most common failure at this size |
| Reference image | The chosen frame, where the provider supports it | This is what holds a character together across eight directions |

Providers differ in what these are called and whether they exist. Confirm per provider before a full run:

- Does it accept a **negative prompt**? If not, the contract is enforced only by `pixelforge` and by rejection, which raises the cost per usable frame.
- Does it accept a **reference image** for consistency? Without it, eight-direction character work is luck and the roster should be reordered to do tiles and effects first.
- Does it support **seamless tiling** on both axes? Tiles need it; without it they need manual edge work and are not worth generating.
- What is the **rate limit and batch ceiling**? The run is thousands of frames and needs to be resumable.

### Answered for Higgsfield

Measured against the live API. Full numbers and costs are in [`plans/higgsfield-pipeline.md`](./plans/higgsfield-pipeline.md).

- **No negative prompt**, on any image model reachable through the endpoint. The list above is appended to the prompt as avoidance language, which is weaker, so the contract leans harder on quantisation and on throwing frames away.
- **Reference images yes**, up to sixteen, through `image_urls` on `marketing-studio/image`. This is the mechanism that keeps a roster on-model and it is not wired up yet.
- **Seamless tiling: no field for it.** Tiles need edge work or a different approach.
- **Concurrency about four**, and exceeding it returns HTTP 400 rather than 429. Runs are resumable through the ledger.

One rule here was decided by experiment rather than by reasoning, and it overrides the instinct to copy Boltgun: **ask the generator for the stylised sprite, not for a photoreal render to be shrunk later.** A photoreal prop is lit photographically, holds a narrow band of values, and turns to mud at sprite scale. The same subject asked for as limited-palette pixel art survives the downscale intact. Boltgun renders detailed models and reduces them, but its artists control the contrast of that render and a prompt cannot.

## Per-group specification

### Fighters and enemies

- Target 64 px, generate 256.
- Five directions per state, named `front`, `front_quarter`, `side`, `back_quarter`, `back`. Directions six to eight are mirrored in engine, not generated.
- Direction one is generated first and becomes the reference image for two to five.
- VIEW per direction: `front view`, `three-quarter front view turned 45 degrees right`, `side profile view facing right`, `three-quarter rear view turned 135 degrees right`, `rear view`.
- States from the asset list: idle 4, walk 6, fire 2, pain 1, death 5, gib death 5.
- Walk frames are generated as a sheet where the provider supports it, otherwise per frame with the previous frame as reference, accepting that per-frame animation is where generated art is weakest and hand touch-up is most likely.

### Weapon view models

- Target 256 px, generate 1024. These are the largest thing on screen and the only assets where detail survives.
- VIEW: `first person view from the shooter's perspective, weapon held at the lower right of frame, angled up and to the left, muzzle pointing away into the distance`.
- Frames: idle 1, fire 3, reload 4, lower and raise 2.
- The fire frames are generated with the idle frame as reference so the weapon does not change shape as it shoots.

### Effects

Generated as sprite sheets on a transparent background, one row per effect, and this group goes first because it is where the game has nothing.

- Target 32 to 64 px per frame, generate 4x.
- STYLE gains: `bright additive effect sprite, clean hard edges, no soft glow`.
- The TECHNICAL clause gains `single effect centred on transparent background, no character, no weapon, no environment`.
- Per effect, the frame counts are in the asset list. Each needs an obvious arc: appear, peak, dissipate. A four-frame effect whose frames differ only in opacity is a fade, not an effect, and is rejected.

### Tiles

- Target 64 px per metre of texture, generate 256, seamless both axes.
- VIEW: `flat top-down texture, orthographic, no perspective`.
- TECHNICAL gains `seamless tileable texture, edges match on all four sides`.
- Rejected if `pixelforge` finds a visible seam, which it checks by wrapping the tile and measuring the gradient across the join against the gradient within the body.

### Interface

- Target 32 px icons, generate 128.
- STYLE gains: `simple bold icon, single subject, high contrast, readable at 32 pixels`.
- These are the smallest and the most sensitive to symmetry collapse; expect the highest rejection rate and budget for it.

## Normals

A normal map beside every albedo, because a sprite that cannot react to a muzzle flash is what makes generated art look pasted on.

1. Run the chosen albedo through a depth-estimation model to get a height map.
2. `pixelforge` converts height to a tangent-space normal by a Sobel gradient, with a strength constant per group: characters read best around 1.5, weapon view models around 2.5, tiles around 1.0.
3. The normal is quantised only in the sense of being downscaled; it is not palette-locked, because it is not a colour.

Depth estimation is an external model, like the generators. Nothing about it enters the repository as tooling.

## Naming and provenance

```
<group>/<subject>/<state>_<direction>_<frame>.png
<group>/<subject>/<state>_<direction>_<frame>_n.png
```

Every committed frame has a manifest row recording tool, model, prompt, negative prompt, seed, parameters, palette hash, the source generation id, and whether a human edited it. The manifest is the disclosure if the game is ever listed somewhere that requires one, and it is how a frame that came out wrong is regenerated a month later.

Provenance chunks are stripped from the PNGs; the manifest is the record. Nothing here pretends a generated frame was drawn by hand.

## Acceptance

A frame is committed only if it passes `pixelforge`:

- Downscales by an exact integer with no resampling artefacts.
- Quantises to the palette with no colour outside it.
- Has an alpha channel with a clean edge, no halo, no semi-transparent fringe wider than one pixel.
- Is not near-identical to the previous frame in the same animation, which catches a generator that produced a fade instead of motion.
- For tiles, wraps without a visible seam.
- Has a normal map of the same dimensions.

And if it passes a human look at the contact sheet the visual QA tour produces, which is where silhouette, readability and whether it belongs in the same game as everything else are judged.

## Order

From the asset list, and it is the order the game gets better rather than the order the list is written:

1. Effects, about 90 frames.
2. Interface, about 40 frames.
3. One fighter, all states, all directions, 115 frames, which proves the pipeline before committing to eleven hundred.
4. The three existing weapons as full view models.
5. One tile set and its props, enough to dress one arena-tier map.
6. The rest of the fighters and the enemy roster.
7. The remaining tile sets as the map tiers land.

Steps one and two are about 130 frames and are where the attention and the money should go first.
