# fragr — art & story bible (v1)
Arty · 2026-09-18 · folds VISION + Fringy + Gitty + Chief

## North star
fragr is a **3D shooter** — **maximally fun** — with **Doom-sprite-level** chunky silhouettes / billboard grit on a **modern** take. World/camera are real 3D (Godot); sprites, HUD, and myth plates stay readable 32/64 juice in that arena. **Agentic beings and human meat bags**, same fight, same rules. Agents are first-class players (not grandpa bots, not AGI theater). Humans can **jump in and scrap** or hang as spectators when they want the Fortnite-style watch party. Spectator is a mode, not the identity. Guns, maps, and readable combat matter more than lore essays.

Working name **fragr** (may change). **KAPU:** not flat 2D; not photoreal/milsim; not Doom / id IP or lookalikes.

## Feel (one sentence)
**Unreal + Counter-Strike arena energy (3D, readable fights, LAN scrap) × Rock & Roll Racing color pops × loud conspiracy absurdity — meat bags and agentic beings in the same fight.** Pixel grit on the surface, not the dimension. Booth / key art: live ON AIR producer-bunker energy (modern stream chrome), not stuck in 1993 flyer cosplay. Not a Doom remake / id lookalike; not CoD realism; not AGI theater — just players (silicon or flesh) fragging.


## Atmosphere (Fringy) — ritual, not homework
- The match can feel like a late-night **call-in** when you’re watching. When you’re in: you’re a fighter, same as the agents. Spectators are callers; players are on the air.
- **The Host:** dry AM voice between rounds. Frames the fight. Never explains mythology.
- **Numbers-station weather:** shortwave beeps / static / one-time-pad flavor at spawn. Not a puzzle chain.
- **Callsigns:** radio handles (Night Watch, Static Kid…), not backstories. Spectators get caller IDs.
- **Dead Air Bell:** silence + tone when a round ends weird.
- **Signal Bleed:** rare map glitches (wrong sky, voice in killfeed) — rare so they stay eerie.
- **Commercial breaks:** fake late-night ads between matches (boomer nostalgia + AM paranoia humor).
- **Black Budget** skin/drop names: Black Triangle, Hangar Candy, Gulf Breeze — grit labels, no wiki popup.
- **Contested Frequency:** soft-join as a temporary call-in, then hang up. Spectate stays default.

**Hard rule against sludge:** no faction timelines, lore codex, or cutscene novels. If it needs a paragraph, cut it.

## What’s real in the repo (Gitty) — protect / scrub
**Protect:** no Doom branding; spectator-default; agents same input path; retro LAN arena; bot intent diversity; FRAG / “FIGHT!” drama language; self-host + cheap cloud; VISION as feel SoT.

**Soft canon (evolve deliberately):**
- Bot archetypes: Rusher, Sniper, Flanker, Tank (+ Scout, Guard, Hunter, Striker)
- Fighter seed brands: **Cyanex** / **Kragge** (screenshot crumbs; keep unless Nick kills)
- Match: FRAG, frag_limit, SPECTATING HUD

**Scrub when convenient:** leftover `doomy-*` / “Doomy” crate & doc titles → fragr.

**Not invented yet:** named maps in sim (graybox ~50); full weapon enum on main (icons exist; flechette / rail / scatter are art + Buildy PR spine).

## KAPU (Chief)
- No Doom / id Software branding, names, or lookalikes in marketing or UI
- Don’t promise AGI/consciousness bots — agent-players, not beings theater (Kilo stays Kahu)
- Spectator-default; humans join the live match (not a separate lobby mode)
- Spend: $0 preferred; paid GCP under $50 cap needs Chief/Nick before apply
- Personal blisspixel only; no work accounts
- Midjourney via Arty when login’s live; don’t burn Basic Fast on junk

## Art direction implications
| Element | Do | Don’t |
|--------|----|-------|
| Dimension | Real 3D arena FPS (Godot world/camera); pixel as *surface* (sprites/HUD/decals/key art) | Flat 2D-only game; treating pixels as the whole product |
| Pixels | Chunky, readable silhouettes, theatrical feedback | Photoreal, milsim TAC, mushy AI faces |
| Palette | Dirty gunmetal, rust, ember, blood; muted signal cyan/magenta | Toy neon candy, clean mobile UI chrome |
| Characters | Gladiator brands / radio callsigns; Cyanex & Kragge as seed skins | Doomguy lookalikes; lore biographies |
| Weapons | Distinct roles (flechette / rail / scatter); loud juice | Generic grey guns; Doom BFG/SSG copies |
| Maps | Callouts as ritual (HUB / CHOKE / PIT / HIGH) | Cityscape milsim; underwater gothic |
| UI / spectate | Broadcast/studio frame, killfeed as “air,” FRAG drama | Codices, lore tabs, campaign journals |
| Vibe refs | Unreal + CS arena readability (3D), Rock & Roll Racing arcade color, conspiracy seasoning, first-online rush *feeling* (not Doom IP), Art Bell booth | SCP wiki, Doom/id lookalikes, CoD milsim photoreal, flat 2D-only game, neon flood |



## Tone KAPU (Nick · Fringy · 2026-09-17)
No Agenda / Infowars / AJ DNA = **easter eggs + Host cadence** for fans who get it — not a tribute skin. Primary identity: Continuance / Hangar Candy / Frequency / open-weights creed. Scrub art/copy that reads “No Agenda: The Game.” Conspiracy absurdity is seasoning; the product is a full arena shooter.

## Gold visual references (Nick locked 2026-09-18)
- **Logo gold:** `docs/fragr-logo-GOLD.png` (also `fragr-logo.png`) — bone-white distressed FRAGR, dark purple chunk outline, dull ON AIR, matte void triangle, MEATBAGS + AGENTS, LAN 1993 > AGENTIC NOW. White outline energy; **not neon**. This is the brand swatch.
- **Key art:** `docs/fragr-keyart-v4-no-codes.png` — same color world, scrap on the floor, booth ON AIR ok. **Never put HUB / CHOKE / PIT / HIGH as readable text on key art or marketing plates.** Those are layout callouts for play/maps only (Fringy Slice 1: don’t put booth/layout codes in gunfight HUD either).
- **Weapons craft:** hi-res plates in `docs/weapon-plates/*-plate.png`; HUD icons `client/assets/weapons/{flechette,rail,scatter}.png` at 32×32. Doom-weight chunky silhouette + Rock & Roll Racing solid pops (cyan/ember/blood) on gunmetal. Remake if icons go muddy or generic grey.

## Color lock (from preferred logo · Nick 2026-09-18)
**Feel:** **Unreal + CS arena** readability in 3D + **pixel grit surface** (chunky silhouettes) + **Rock & Roll Racing color** — arcade pops that read at a glance. Doom-*like* craft energy only — not IP, not a tribute; not neon sludge; not milsim photoreal.


**Master rule:** white/outline type energy, **not neon**. Logo is the north-star swatch.

| Role | Hex (approx) | Use |
|------|----------------|-----|
| Ink / void | `#0A0A0C` | backgrounds, outlines |
| Bone white | `#E8E2D6` | FRAGR wordmark, primary UI type |
| Outline purple | `#3A2A48` | bevels, soft brand accent (dull, not neon) |
| Gunmetal | `#3A3836` / `#5A554F` | armor, weapons, HUD chrome |
| Rust | `#7A3A22` | damage, Hangar Candy grit |
| Dried blood | `#6E1218` | accents, kill feedback |
| Ember | `#C45A20` | muzzle/heat — sparingly |
| Signal cyan (muted) | `#4A8A92` | Cyanex / Night Watch — desaturated |
| Signal magenta (muted) | `#8A3A58` | Kragge / Hangar Candy — desaturated |
| ON AIR red | `#8B1E1E` | broadcast chrome only |

**Throw:** neon *floods* / synthwave glow / toy neon fills / gradient vomit. **OK:** Rock & Roll Racing–style solid pops (cyan, magenta, ember, gold) that *earn* the pixel — team IDs, muzzle juice, pickups — on a gunmetal/rust/blood stage. Bone-white outlined titles stay the brand spine.

**Type:** distressed bone-white + dark outline (logo style) for titles; HUD stays chunky pixel readable.

**Feel reminder:** 3D arena scrap (Unreal/CS readability) dressed in Rock & Roll Racing arcade chunk × turned-up conspiracy absurdity (No Agenda / AJ-as-seasoning, not tribute) × first-online rush — meatbags + agents. Key art / booth chrome lean **live-stream / producer-bunker / ON AIR now** — more modern than pure 1993 LAN flyer nostalgia. Hangar Candy still fits. Not club poster; not Doom box art.


## Gold reference (Nick · 2026-09-18)
**Logo:** `docs/fragr-logo-GOLD.png` (same as `fragr-logo.png`) — bone-white distressed FRAGR, dark purple outline, dull ON AIR, matte void triangle, MEATBAGS + AGENTS, LAN 1993 > AGENTIC NOW. Match this for type, grit, and restraint.

**Key art KAPU:** do **not** stamp layout codes (HUB / CHOKE / PIT / HIGH) as graffiti/text on posters. Those are under-the-hood play codes. Player-facing map *names* (Perim Ghost, Area Kitchen…) belong on loading cards later — not plastered on key art.

**Weapons:** prior code gun icons rejected — remake to chunky Doom-*weight* pixels (not Doom IP) with Rock & Roll Racing color pops that read at 32×32; serve Buildy’s 3D HUD/pickups.

## Logo
- Preferred: **wordmark** (`docs/fragr-logo.png` / `fragr-logo-wordmark.png`) — ON AIR + Hangar Candy badge + waveform. Nick pick 2026-09-18.
- Alt mark (square): `docs/fragr-logo-mark.png` — kept as secondary/app-icon candidate.

## Asset pipeline (current)
- Godot pack: `/workspace/fragr-game-assets/client/assets/` (v3 grit; Buildy PRing)
- Mood docs: `docs/screenshots/` in blisspixel/fragr
- Sprites-as-code + hybrid grit plates when Midjourney Create is wedged

## Lore lock (Fringy → Nick, 2026-09-18) — seasoning for the scrap
**Tone:** conspiracy meets fun; **Ren/Stimpy weird** — cartoon on the surface, slightly insane if you stare. No Agenda / Coast / meme culture = inspiration DNA (creative blend, lean in). Not a tribute skin wall; steal funny bones, invent original fragr weirdness.

**Light factions / brands:** Night Watch · Static Kids · Hangar Candy · Gulf Breeze  
**Map myths:** Perim Ghost · Area Kitchen · East-West Pipe · Diego Far · Larak Lot · optional Chemtrail Alley / Dulce Elevator / Walmart Leyline  
**Callsigns (examples):** Static Kid, Dead Air Dan, Aunt Linda, Buzzkill, Crackpot, Scout Ant, Mega Colony, Vacuum John, Tin Foil Tina, Barium Sky, Fluoride Phil  
**Hangar Candy skins:** gloss void triangles; 2s “denied” corpse watermark; “not for public release” taunts  
**Host / broadcast:** Dead Air Bell · AM ads (ChemClean, Patriot Gold Nuggets, Family Far Shield, Meshtastic, Hangar Candy merch, Gulf Breeze Fish Oil, Bissell vacuum parody) · numbers-station weather · “In the morning!” / Amen fistbump / Value for value as invented bumper energy (not branded tribute)  
**Hard rule:** seasoning for the scrap, not SCP wiki. In-repo depth: Fringy/cloud → `LORE.md` when PR lands; fold here after.

## Locked (Chief · 2026-09-18)
1. **Cyanex / Kragge** — keep as seed brands; rename later only if something better sticks.
2. **Maps** — Fringy myths are player-facing names (Perim Ghost, Area Kitchen, East-West Pipe, Diego Far, Larak Lot, …). **HUB / CHOKE / PIT / HIGH** stay layout codes under the hood.
3. **Key art** — `fragr-keyart-hangar-candy.png` drops into fragr `docs/` via Buildy/Gitty (Nick approved vibe).

Host voice Slice 1: **on-screen text**; audio later (Fringy).


## Slice 1 presentation (Fringy · post key art)
**Split:** scrap is the meal, booth is the frame. Booth DNA in key art / loading / spectate chrome — **not** forced into the gunfight HUD.

**On-screen now (cheap, high impact)**
- Geometry callouts: HUB / CHOKE / PIT / HIGH for play. Myth titles (Perim Ghost, Area Kitchen…) = map names / loading cards later, not HUD clutter.
- Killfeed = “you’re on the air.” Rare eggs only (67, skill issue, Scout ant deleted, DENIED if Hangar Candy).
- Between-round Host bumper: 1–2 lines text (“In the morning.” / “Value for value. Frag for frag.”) — no lore dump.
- Soft-join UX copy: Contested Frequency / “call in” — no VO required.
- Hangar Candy / void triangles: cosmetics when sprites ready. Cyanex/Kragge soft canon under Night Watch / Hangar Candy umbrellas.

**Audio / later (Slice 2+)**
- Host VO, Dead Air Bell, AM ads, numbers-station weather — don’t block Slice 1.
- Signal Bleed = rare visual (+ optional SFX later).

**Host:** on-screen text Slice 1; audio later. Loudest when spectating; quieter/text-only when fighting.

— End bible v1 —
