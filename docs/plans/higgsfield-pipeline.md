# The Higgsfield art pipeline

How fragr makes pictures. `tools/spritegen` is the program, `docs/ART-GENERATION-SPEC.md` is the contract it executes, and this file is what was measured against the live API so the next run does not have to rediscover it.

The division of labour is fixed: **Higgsfield makes everything you look at** (map textures, weapons, effects, characters), **ElevenLabs makes everything you hear** (`tools/audiogen`). Neither tool ever runs in CI and neither is called by the game.

## What the API actually is

Verified against the account, not read off a marketing page.

- Base is `https://api.higgsfield.ai`. Authentication is a single header, `Authorization: Key <id>:<secret>`, and `.env` holds the whole pair on one `higgsfield=` line because that is exactly the string the header wants.
- Submission returns a `request_id` and a `status_url`. Poll it until `completed`, `failed`, `nsfw` or `canceled`, starting at two seconds and backing off to ten.
- `POST /estimate/<model-id>` prices any request for nothing. This is the budget gate and the reason the tool can refuse a run as a whole instead of discovering the bill halfway through.
- Requests that end `failed` or `nsfw` are **not charged**, and a queued request cancelled before it starts is refunded.
- Output URLs live for at least seven days and then may vanish, so the tool downloads every frame rather than storing links.
- Concurrency is about four in flight and exceeding it returns **HTTP 400**, not 429. There are no rate-limit headers and no `Retry-After`.
- `image_urls` accepts up to sixteen reference images. This is the single most valuable field in the API for a game: once one weapon is approved it becomes the style board for the rest of the armoury, which is how thousands of frames stay on-model.

## What things cost

Measured, per image, through the estimate endpoint on 2026-09-19.

| Model | Parameters | Cost | What it is for |
|---|---|---|---|
| `marketing-studio/image` | `low` / 2k | $0.019 | Exploration. The workhorse |
| `marketing-studio/image` | `medium` / 2k | $0.100 | Promoting a keeper |
| `marketing-studio/image` | `high` / 2k | $0.373 | Rarely worth it |
| `higgsfield-ai/soul/v2/standard` | 720p | $0.004 | Photoreal people. Cheapest thing here |
| `higgsfield-ai/soul/v2/standard` | 1080p | $0.006 | Photoreal people |
| `recraft/v4.1/pro/text-to-image` | 2k png | $0.210 | Has real palette and background-colour fields |
| `higgsfield-ai/soul/standard` | v1 | $0.094 | Superseded. Fifteen times the price of v2 for no gain |

The shape of the spend follows from that table: **explore wide at two cents and promote only keepers at ten**. Generating everything at `high` costs twenty times as much and, as the next section shows, does not buy a better sprite.

`marketing-studio/image` is the model for props and weapons. It honours a transparent background, which nothing else here reliably does, and it generates isolated objects rather than scenes.

## The finding that decides the house style

The obvious plan was the Boltgun plan: render something detailed and shrink it, which is how Boltgun, Prodeus and Doom itself made their sprites. It is wrong for us, and it was cheap to prove.

Eight frames were generated, the same four subjects asked for twice: once as `retro pixel art sprite, 1990s PC shooter, limited palette`, and once as `high detail 3D rendered game weapon prop`. Both sets were then reduced to the same 180-pixel sprite height by the same code.

The rendered set collapsed into mud. The stylised set survived and came out crisp.

The reason is value contrast, not resolution. A model asked for a photoreal prop lights it photographically: olive on olive, a narrow band of middle values, detail carried by subtle shading that does not exist any more once the sprite is 180 pixels tall. A model asked for a limited palette returns high value separation, and value separation is the only thing that survives a large downscale.

Boltgun's pipeline works because human artists control the contrast and palette of the render. A prompt cannot control that tightly enough. So fragr asks the generator for the end state and enforces the rest locally.

**Ask for the stylised sprite. Quantise it against `docs/palette.json`. Never ask for a photoreal render and shrink it.**

The corollary matters too: reduction without palette quantisation produces mush from either arm. The palette step is not a finishing touch, it is what makes the sprite a sprite.

## The other half: reduction

`tools/spritegen reduce` is local, free, and does four things in an order that is not arbitrary.

1. **Trim to the alpha bounding box.** A generator centres its subject in whatever canvas it likes; trimming first means the downscale ratio is set by the sprite rather than by the empty space around it.
2. **Downscale by area averaging**, so every source pixel contributes to exactly one output pixel. Nearest-neighbour here would throw away most of what was paid for.
3. **Quantise to the palette in CIE L\*a\*b\***, not RGB. RGB distance picks visibly wrong swatches in dark and saturated regions; Lab picks the one an eye would call nearest.
4. **Harden the alpha.** A sprite has no half-transparent edge. Everything goes fully on or fully off, which is what gives a silhouette its bite.

## How the money is kept under control

Three mechanisms, because an art run is the only part of this project that can spend real money by accident.

- **The whole run is priced before anything is generated.** If the total exceeds the cap, nothing is generated at all.
- **`--max-spend-usd` is required for a live run and is itself capped** at `HARD_CAP_USD`, five dollars. A run that needs more than that should be split and approved a piece at a time.
- **An append-only ledger** sits beside the output. Every frame is recorded the moment it lands, and a repeated run generates only what is missing. The expensive failure for this tool is paying twice for the same picture.

## Using it

```
# Free. Assembles and prints the prompts, sends nothing.
cargo run -p fragr-spritegen -- prompts --spec tools/spritegen/specs/weapons-viewmodel-bakeoff.json

# Free. Prices the run against the live estimate endpoint.
cargo run -p fragr-spritegen -- price --spec tools/spritegen/specs/weapons-viewmodel-bakeoff.json

# Spends money. Refuses without a cap, and refuses if the priced run exceeds it.
cargo run -p fragr-spritegen -- gen --spec tools/spritegen/specs/weapons-viewmodel-bakeoff.json --max-spend-usd 0.30

# Free, local, no network.
cargo run -p fragr-spritegen -- reduce --input art/raw/weapons-bakeoff --out art/sprites/weapons \
  --height 180 --palette docs/palette.json --preview-scale 4
```

`art/raw/` is gitignored because provider output is large and regenerable from the spec. Reduced sprites are small and are committed.

## What is not done yet

- **Reference images are not wired up.** `image_urls` is the mechanism for keeping the armoury on-model and the tool does not use it yet. This is the highest-value next change, and it needs the presigned upload flow (`POST /files/generate-upload-url`, then PUT to the returned URL).
- **Normal maps.** The contract in `ART-GENERATION-SPEC.md` asks for a normal beside every albedo, derived from a depth pass. Nothing generates one.
- **Animation.** Per-frame animation is where generated art is weakest. The image-to-video models on this same key are a real route to a sprite sheet: generate a still, animate it, extract frames. Untested.
- **Concurrency.** The tool submits one frame at a time. The account allows about four, so a large run is currently four times slower than it needs to be.

## Related

- [`docs/ART-GENERATION-SPEC.md`](../ART-GENERATION-SPEC.md): the contract every frame meets
- [`docs/ART-ASSET-LIST.md`](../ART-ASSET-LIST.md): what the game needs
- [`docs/ART-COLOR.md`](../ART-COLOR.md): how colour carries faction
- [`docs/palette.json`](../palette.json): the palette every sprite is quantised to
- [`plans/art-pipeline.md`](./art-pipeline.md): why the pipeline is shaped this way
