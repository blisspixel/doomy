# Plan: art pipeline (sprites, weapons, icons, textures)

**Status:** planned (2026-09-18)
**Branch:** `feat/art-*`
**Spend:** about 15 to 40 dollars for the full first asset set through two pixel-art services, under a per-run cap like the brain agent's. **Needs written approval before the first paid call**; nothing here is approved yet. Local open-weight generation is free and covered below.

## Goal

Dramatically better fighters, enemies, weapon view models, icons, and map textures without a hand that can draw: a repeatable pipeline where a model produces palette-locked pixel art in the eight directions the engine mirrors, a Rust tool makes every frame conform to the palette, grid, and sheet layout, and a person touches only what matters (silhouettes, faces, muzzle alignment, ground contact). Everything shipped is Apache 2.0 with provenance recorded, and nothing pretends to be hand-drawn.

## Non-goals

- Painting textures or characters by hand at scale. Touch-up only.
- General image models for final frames. They cannot emit small canvases and their consistency across directions is luck.
- Training or fine-tuning models. Off the table, and several providers forbid using outputs for it anyway.
- Photoreal or 3D-rendered sprites. The look is chunky pixel art at 64 pixels; see `look-pass-boomer.md`.

## What the research found (2026-09-18)

Two services do the hard parts natively: exact small canvases (16 to 128 px), palette locking to a supplied palette image, eight rotational views of one character, skeleton animation per direction, and seamless tiling. Their terms assign or disclaim output ownership with no attribution requirement. General models need at least half a megapixel, so every result would need a 16x nearest downscale plus quantisation, and none can hold one character across eight directions on purpose. Two providers are excluded on terms: one trains on outputs, one forbids stripping metadata (this repository strips provenance chunks) and claims free-tier assets.

| Tool | Native small canvas and palette lock | Eight directions and animation | Output terms | Price band |
|---|---|---|---|---|
| PixelLab | Yes, 16 to 128 px, target palette | Native eight views in one call, skeleton animation per direction | Free to use, modify, distribute for any purpose except training models | Under a cent for a 64 px image; about ten cents for an eight-view character |
| Retro Diffusion | Yes, 12 to 384 px, `input_palette`, `tile_x` and `tile_y` | An eight-direction rotation style, 4 to 16 frame animation, an "FPS Weapon" style | No ownership claimed; full rights retained | 3 to 18 cents per image; rotation up to 25 cents; a tileset 10 cents |
| Google image model (concept sheets only) | No (512 px minimum) | Up to four character references | No ownership claim; pixel watermark | About 7 cents per image |
| Local Apache 2.0 stack (FLUX.2 klein 4B, Qwen-Image and Qwen-Image-Edit, Z-Image-Turbo, HiDream) with Apache pixel LoRAs | No, 512 to 1024 px then downscale | Qwen-Image-Edit can rotate; otherwise luck | Free; weights and outputs clean | Hardware: 8 to 16 GB of VRAM |
| Excluded | | | Trains on outputs, or forbids metadata removal and claims free-tier assets, or keeps a licence over assets, or no API | |

Copyright reality: prompt-only frames are not protectable under current guidance, hand-edited frames and the assembled sheets and game are. Apache 2.0 over prompt-only frames is harmless; the manifest says which frames were touched.

## The pipeline

1. **Concept sheets.** A general model produces reference sheets per fighter, enemy, and weapon from the art bible and the palette, about forty images, for humans to pick from. These never ship.
2. **Fighters and enemies.** PixelLab creates each character at 64 px in all eight views from the chosen concept and the palette, then skeleton animation per direction: idle 4, walk 4, fire 2, death 4; pain by inpaint. Keep views one to five, drop six to eight, mirror in-engine exactly as Doom did (rotation zero faces the viewer, clockwise, mirrored lumps), and accept the flipped weapon hand.
3. **Weapon view models.** Retro Diffusion's FPS weapon style with animation: idle, fire 2, reload beat, muzzle flash frames, for each of the three weapons and the ones the campaign adds.
4. **Icons and HUD sprites.** PixelLab at 64 and 32 px: pickups, weapon icons, health and armour glyphs, radio station marks, the status face.
5. **Textures.** Retro Diffusion with seamless tiling on both axes and the palette image from `docs/palette.json`, generated at four times the 16 texels per metre target and nearest-downscaled, since a one metre tile at 16 px is too small to prompt.
6. **`tools/pixelforge` (Rust).** Every generated file passes through it before it is committed: exact-integer nearest downscale, palette snap to `palette.json` with optional 4 by 4 Bayer, alpha threshold and stray-pixel cleanup, a one pixel outline in the palette's outline colour, sheet packing with the frame counts the client's `Sprite3D` needs, provenance chunk and text chunk stripping, and a manifest entry with tool, prompt, seed, palette hash, and a human-edit flag. Crates: `image` and `exoquant` (both MIT); not `imagequant` (GPL). Python is not used anywhere in this pipeline.
7. **Touch-up in Pixelorama** (MIT, built in Godot): silhouettes and faces, muzzle flash alignment, death-frame ground contact, hand continuity across weapon frames, HUD numerals, and a final palette review. The tour's contact sheet is where the review happens.

## Provenance and honesty

- Strip provenance chunks (the repository already does) and never claim a frame is hand-drawn; the manifest records the tool and prompt, which is a developer-integration record and allowed by the attribution lock.
- One provider embeds a pixel-level watermark that may or may not survive pixelation; that is fine, since nothing is being hidden.
- If the game is ever listed on a store that requires disclosure of generated art, the manifest is the disclosure.

## Spend gate

Estimate for the first full set, with two retries: concept sheets 3 dollars, twelve characters with eight views and four states about 24 dollars, three weapons 2 dollars, forty icons 1 dollar, thirty tiles about 6 dollars; about 39 dollars worst case, about 15 if first attempts land, about 28 with icons and tiles done locally on a GPU. Each tool run takes a `--max-spend-usd` cap and appends to a ledger under `.agents/spend/`, the same gate the brain uses. Nothing runs until the approval line is written into this file.

Approval: not yet given.

## Rungs

1. `tools/pixelforge` with tests on synthetic images (downscale, snap, outline, pack, strip, manifest); palette validation against the art bible; the tour renders the contact sheet.
2. Local free path proven: one fighter through an Apache 2.0 model plus LoRA and the tool, so the pipeline works at zero cost even if it looks rough.
3. Concept sheets and the two fighters through PixelLab after approval; the look pass stage 3 consumes them.
4. Weapons and icons; look pass stages 4 and 5.
5. Tiles; look pass stage 2.
6. The campaign roster (ten types) as the campaign's rung 2 lands.

## Success criteria

- [ ] `pixelforge` shipped with tests; every committed sprite passes it.
- [ ] One fighter produced at zero cost through the local path.
- [ ] Two fighters, eight directions, four states, touched up and in the client.
- [ ] Three weapon view models with fire frames.
- [ ] Both arenas dressed with generated, palette-locked tiles.
- [ ] Manifest covers every shipped generated asset.
