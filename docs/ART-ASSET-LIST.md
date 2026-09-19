# The asset list

Every sprite, texture, and effect fragr needs, itemised, so a generation run has a target instead of a vibe. `plans/art-pipeline.md` owns how assets are made and the rendering contract they must meet. `plans/look-pass-boomer.md` owns how the world is dressed. This file owns what exists.

## How to read it

- **Sizes are source sizes.** Everything is authored at four times the target and nearest-downscaled by `tools/pixelforge`, because a one-metre tile at sixteen pixels is too small to prompt.
- **Every entry is albedo only**, with a normal map beside it. No baked shadow, specular, or ambient occlusion. Light comes from the engine.
- **Every entry is palette-locked** to `docs/palette.json`, quantised in perceptual colour space, ordered dithering only.
- **Characters are eight-direction billboards.** Directions one to five are authored, six to eight are mirrored in engine, exactly as Doom did. The flipped weapon hand is accepted.
- **Counts below are frames**, not files. A four-frame walk in five directions is twenty frames.
- Status: **have** means it exists in the repository today, **need** means it does not.

---

## 1. Fighters

The people in the arena. 64 px frames, five authored directions.

### States every fighter needs

| State | Frames | Note |
|---|---|---|
| Idle | 4 | Breathing loop |
| Walk | 6 | Doubles as run, played faster |
| Fire | 2 | One per weapon class where the pose differs |
| Pain | 1 | Flash frame, reused for all damage |
| Death | 5 | Ends on a floor pose that stays |
| Gib death | 5 | For the rail and explosive kills |

That is 23 frames per direction, 115 per fighter across five directions.

### The roster

| Fighter | Status | What it is |
|---|---|---|
| Scrap League default | have (idle only) | The body everyone starts from |
| Scrap League heavy | need | Slower silhouette, wider shoulders |
| Scrap League light | need | Thin, fast, reads as quick at distance |
| Agent chassis | need | Visibly machine-driven, so a player knows what they are fighting |
| Continuance officer | need | The authority faction, clean where everything else is scrap |
| Congregation adherent | need | The belief-system faction from the lore |
| Pirate crew | need | The station's own people |

Seven bodies, 805 frames.

### Customization

The point is that two players in the same match are not the same picture, without authoring seven hundred bodies.

| Layer | Options | How |
|---|---|---|
| Palette swap | 12 | Index remap over the locked palette, free at runtime, no new art |
| Head or helmet | 8 | Separate sprite layer composited over the body, per direction |
| Torso rig | 6 | Backpack, plating, harness, as a layer |
| Marking or decal | 10 | Number, stripe, faction mark, as a layer |
| Trail or aura | 4 | Effect, not sprite; earned rather than chosen |

Head 8 × 5 directions × 4 key poses = 160 frames. Torso the same shape, 120 frames. Markings are static, 50 frames. Layers are composited by `pixelforge` into per-combination sheets, or at runtime if the sheet count gets silly.

---

## 2. Enemies

Not players. Survival and campaign. The canonical roster, with what each type does and how it is answered, is [`docs/ENEMIES.md`](./ENEMIES.md); this section is only the frames.

| Enemy | Status | Frames | Note |
|---|---|---|---|
| Compliance Drone | have (placeholder) | 115 | Existing boss, demoted to elite when the roster lands |
| Clerk | need | 92 | Small, upright, clipboard. The tutorial enemy |
| Sweeper | need | 115 | The basic body, cheap and numerous |
| Sweeper, ranged | need | 115 | Same silhouette, different colour and fire pose |
| Sweeper, heavy | need | 115 | Same silhouette again, wider and slower |
| Crawler | need | 92 | Low to the ground, no fire state |
| Jammer | need | 60 | Squat and stationary. Idle, fire, damaged, destroyed |
| Enforcer | need | 115 | Heavy, armoured, a wind-up before the charge |
| Turret | need | 24 | Static: idle, fire, damaged, destroyed |
| Redactor | need | 115 | Invisible until it fires, so the fire frames carry it |
| Auditor | need | 115 | Floats, clipboard shield, resurrects Clerks |
| Continuance Walker | need | 160 | Episode boss, larger canvas at 128 px |

Twelve entries, about 1130 frames. The Sweeper family shares one body across three variants, distinguished by colour and by how they move, because a family you can read at a glance is worth more than three unrelated shapes.

---

## 3. Weapons

Three exist and a boomer shooter wants a full ladder. Each weapon is four separate assets.

### Per weapon

| Asset | Size | Frames |
|---|---|---|
| First-person view model | 256 px | idle 1, fire 3, reload 4, lower and raise 2 |
| World pickup sprite | 64 px | 1, plus a 4-frame bob or glint |
| HUD icon | 32 px | 1 |
| Held sprite for the fighter billboard | 32 px | 1 per direction |

The canonical list of what each weapon is, what it costs to fire, and every sound it makes is [`docs/WEAPONS.md`](./WEAPONS.md). This section is only the pixels.

### The ladder

| Weapon | Status | Role |
|---|---|---|
| Sidearm | need | The gun you always have, weak, never useless |
| Flechette | have (icon and held only) | Fast mid-range needle gun |
| Scatter | have (icon and held only) | Close-range burst |
| Rail | have (icon and held only) | Long-range single shot |
| Launcher | need | Splash, the crowd answer, the Survival staple |
| Arc | need | Chains between targets, the energy slot |
| Heavy repeater | need | Sustained fire, ammo hungry |
| Signature weapon | need | The one per act the campaign hands out |

Eight guns, about 190 view-model frames.

### Melee

Melee does not exist yet and a boomer shooter is not complete without it.

| Asset | Status | Frames |
|---|---|---|
| Fists view model | need | idle 1, swing 5, hit 2. The one melee everybody has, so it is seen more than any other view model in the game |
| Wrench view model | need | idle 1, swing 5, hit 2 |
| Blade view model | need | idle 1, swing 5, hit 2 |
| Held sprites | need | 2 per direction |
| Impact effect | need | 4 |

---

## 4. Effects

The largest gap and the most visible. `plans/gunfeel.md` records that a hit and a miss currently look identical in the world.

| Effect | Status | Frames | Note |
|---|---|---|---|
| Muzzle flash | have (one) | 4 per weapon class | Three classes, first person and world |
| Tracer | need | 3 | Rail gets a beam, the rest get a streak |
| Impact, hard surface | need | 5 | Sparks and a puff |
| Impact, fighter | need | 5 | The hit confirm in the world, not just on the HUD |
| Impact decal | need | 4 variants | Stays on the wall, fades |
| Explosion | need | 8 | Launcher, barrels, the drone's death |
| Arc discharge | need | 6 | The energy weapon's chain |
| Gib chunks | need | 6 | Rail and explosive kills |
| Pickup flare | need | 4 | What a pad does when taken |
| Spawn effect | need | 6 | Arrival, and the spawn shield while it holds |
| Respawn pad idle | need | 4 | The pad breathing while it waits |
| Shield hit | need | 4 | Armour absorbing rather than flesh |
| Footstep puff | need | 3 | Ground contact, which also sells weight |
| Teleport or lift | need | 6 | Needed by the map tiers with height |

About 90 frames, and the highest value per frame in the whole list.

---

## 5. World

### Tiles

Sixteen texels per metre, authored at 64, seamless on both axes.

| Set | Status | Tiles | Where |
|---|---|---|---|
| Scrap yard | have (5) | 16 | The current arenas |
| Hangar interior | need | 16 | Arena tier, indoor |
| Station corridor | need | 16 | District tier, the connective tissue |
| Continuance facility | need | 16 | Clean, bright, the authority spaces |
| Outdoor ground | need | 12 | District and field tiers |
| Rock and cliff | need | 10 | Field tier boundaries |
| Industrial floor | need | 12 | Grating, plate, hazard stripe |
| Water and hazard | need | 8 | Animated, four frames each |

Eight sets, 106 tiles.

### Props

Every prop is a collision box in the server, so each needs an agreed footprint.

| Prop | Status | Count | Note |
|---|---|---|---|
| Crate cluster | have (3) | 8 | Vary the silhouette; the current three repeat visibly |
| Barrel | need | 4 | Including one that explodes |
| Pillar and column | need | 6 | The cover the choke plan wants |
| Railing and catwalk | need | 8 | Needed the moment maps get height |
| Door and shutter | need | 6 | Doom's floor plan needs doors |
| Lift platform | need | 3 | Map tier rung 1 |
| Console and terminal | need | 6 | Objectives, and set dressing that explains the place |
| Antenna and dish | have (1) | 4 | The station's own hardware |
| Wreck and debris | need | 10 | Large silhouettes that break sightlines |
| Vegetation | need | 6 | Field tier |
| Light fixture | need | 6 | Where the light in the room comes from |

Eleven families, about 67 props.

### Skies and backdrops

| Asset | Status | Note |
|---|---|---|
| Night scrap sky | need | The default |
| Dust storm | need | Field tier, limits sightlines honestly |
| Interior void | need | What a windowless map shows |
| Distant city silhouette | need | Parallax band, sells scale at the field tier |

---

## 6. Items

| Item | Status | Frames | Note |
|---|---|---|---|
| Health pad | have (placeholder) | 4 | Idle bob |
| Armour scrap | have (placeholder) | 4 | |
| Weapon pad | have (placeholder) | 4 | One per weapon, tinted |
| Ammo pickup | need | 4 | Only if the weapons get ammo; see `gunfeel.md` |
| Powerup, damage | need | 6 | |
| Powerup, speed | need | 6 | |
| Powerup, shield | need | 6 | |
| Objective marker | need | 6 | Field tier modes |
| Key or token | need | 4 | Doom-shaped progression in the campaign |

---

## 7. Interface

`plans/hud-rebuild.md` says what the HUD becomes. This is what it needs drawn.

| Asset | Status | Note |
|---|---|---|
| Display font | need | One face, two weights, hard outline, legible at 480 by 270 |
| Health icon | need | Cross or heart, 32 px |
| Armour icon | need | Shield, 32 px |
| Ammo icon | need | Per weapon class, 32 px |
| Crosshair set | need | 4, one per weapon class, plus hit and kill states |
| Killfeed weapon icons | need | 8, matching the weapon ladder |
| Stance chips | have | The agent stance marks |
| Faction marks | need | 6, one per faction in the lore |
| Menu background | need | One plate, boot and pause |
| Scoreboard frame | need | One |

---

## Counts and order

| Group | Frames, roughly |
|---|---|
| Fighters and customization | 1135 |
| Enemies | 1130 |
| Weapons and melee | 220 |
| Effects | 90 |
| Tiles | 106 |
| Props | 67 |
| Items | 44 |
| Interface | 40 |
| **Total** | **about 2550** |

Generate in this order, because it is the order the game gets better:

1. **Effects.** Ninety frames that change how every shot feels, and the only group where the game currently has nothing rather than something rough.
2. **Interface.** The font and the icons, because `hud-rebuild.md` stage 1 is blocked on them.
3. **One fighter, all states, all directions.** Proves the pipeline end to end at 115 frames before committing to 1135.
4. **The three existing weapons** as full view models.
5. **One tile set and its props,** enough to dress one arena-tier map properly.
6. **The rest of the fighters and the enemy roster.**
7. **The remaining tile sets** as the map tiers land.

Steps one and two together are about 130 frames, which is where the money and the attention should go first.
