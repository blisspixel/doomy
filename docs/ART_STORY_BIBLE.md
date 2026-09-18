# fragr — art & story bible (v1)
Arty · 2026-09-18 · folds VISION + Fringy + Gitty + Chief

## North star
fragr is a **networked retro-pixel arena** that’s **fun as fuck to play** — humans and/or agents, same fight, same rules. Agents are first-class players (not grandpa bots, not AGI theater). Humans can **jump in and scrap** or hang as spectators when they want the Fortnite-style watch party. Spectator is a mode, not the identity. Guns, maps, and readable combat matter more than lore essays.

Working name **fragr** (may change). Never brand as Doom / a Doom clone.

## Feel (one sentence)
**LAN church that’s fun as fuck** — Quake/Unreal arena joy, optional Art Bell booth when you’re spectating, theatrical pixels, loud feedback.

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
| Pixels | Chunky, readable silhouettes, theatrical feedback | Photoreal, milsim TAC, mushy AI faces |
| Palette | Dirty gunmetal, rust, ember, blood; muted signal cyan/magenta | Toy neon candy, clean mobile UI chrome |
| Characters | Gladiator brands / radio callsigns; Cyanex & Kragge as seed skins | Doomguy lookalikes; lore biographies |
| Weapons | Distinct roles (flechette / rail / scatter); loud juice | Generic grey guns; Doom BFG/SSG copies |
| Maps | Callouts as ritual (HUB / CHOKE / PIT / HIGH) | Cityscape milsim; underwater gothic |
| UI / spectate | Broadcast/studio frame, killfeed as “air,” FRAG drama | Codices, lore tabs, campaign journals |
| Vibe refs | LAN church, Quake/Unreal/Halo arena, Art Bell booth | SCP wiki, Doom marketing, CoD realism |

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

Host voice Slice 1 still open (on-screen text vs audio later).

— End bible v1 —
