# The weapons

The canonical list. Eleven things you can hold, what each is for, where you get it, and every sound it makes.

Balance numbers live here and nowhere else. `plans/gunfeel.md` explains how they were arrived at, `plans/weapon-economy.md` explains the ammunition and the pickup economy, and `docs/lore/guns.md` is what they get called on the radio.

## What you start with

**Your fists.** That is the entire loadout, on every spawn, for everyone, in every mode.

Everything else in this file is found on the floor, kept until you die, and lost when you do. Doom starts you with fists and a pistol; fragr keeps the fists and puts the pistol on the ground, which is further than Doom goes and is the point.

This only works because of a rule that belongs to the maps rather than to the weapons: **there is a sidearm within about two seconds of every spawn point.** You begin each life with nothing and you end that with a decision, not a death sentence. The pistol stops being something you have and becomes the first thing you do, every time, which is a ritual rather than an inventory.

What it buys is a window. Every life has a few seconds in it where you are holding nothing but your hands, and that means a punch kill is possible and it means anyone who catches you in that window has earned something. It is also the only way the melee ladder means anything: if you always had a knife, finding a knife would not be a moment.

Nobody is a class. There are no loadouts and no roles: if you are sniping it is because you walked to where the rail was. The enemies are the opposite, and deliberately so. A Continuance unit has one shape, one behaviour and one attack, and it never varies, so you learn a silhouette once and know it forever. That roster is [`docs/ENEMIES.md`](./ENEMIES.md).

## The ladder

| # | Weapon | Role | Damage | Cooldown | Bare kill | Ammunition | Where |
|---|---|---|---|---|---|---|---|
| 1 | **Fists** | Melee, always carried | 20 | 0.40 s | 5 hits, 1.60 s | none | Always |
| 2 | **Shiv** | Melee, found | 35 | 0.55 s | 3 hits, 1.10 s | none | Pad, common |
| 3 | **Tack** | Sidearm, found | 20 | 0.25 s | 5 hits, 1.00 s | Tacks | Pad, beside every spawn |
| 4 | **Flechette** | Mid workhorse | 25 | 0.20 s | 4 hits, 0.60 s | Darts | Pad |
| 5 | **Scatter** | Close shred | 40 falling to 14 | 0.45 s | 3 hits, 0.90 s | Darts, 4 per pull | Pad |
| 6 | **Rail** | Long precision | 80 | 1.00 s | 2 hits, 1.00 s | Cores, 2 per shot | Pad |
| 7 | **Repeater** | Heavy full auto | 14 | 0.10 s | 8 hits, 0.70 s | Tacks | Pad |
| 8 | **Lobber** | Splash, projectile | 65 direct, 45 splash | 0.80 s | 2 hits plus travel | Cans | Pad, outer ring |
| 9 | **Arc** | Energy, ignores armour | 18 | 0.15 s | 6 hits, 0.75 s | Cores | Pad, outer ring |
| 10 | **Proximity tin** | Thrown, placed | 90 at centre | 1.5 s to arm | Area denial | 3 carried | Pad |
| 11 | **Article Blade** | Melee upgrade | 70 | 0.45 s | 2 hits, 0.45 s | 12 swings | Plinth, near centre |
| 12 | **Denial** | Signature | 250 | 1.25 s | 1 hit | 5 charges, no refill | Plinth, centre |

Three melee tiers, six guns and a sidearm, a thrown mine and a signature weapon. Four ammunition pools feed the guns. Tacks for the sidearm and the repeater, Darts for the flechette and the scatter, Cores for the rail and the arc, Cans for the lobber. The tin, the blade and the signature weapon carry their own counts and sit outside the pools entirely.

## Reloads, and what happens instead

**No weapon has a magazine reload.** The cooldown already is the cadence, and a reload puts a second rhythm on top of the first where the two fight each other. At a measured time to kill under a second and a half, a two second reload is two entire fights of standing still.

But every weapon is mechanical and has to sound like it. So each one has a **cycle**: the thing it does between shots that you hear whether or not you are looking at it. The scatter pumps. The rail's capacitor winds back up. The arc's coil settles. The repeater's barrel spins down when you let go.

Two weapons genuinely reload, because they are single-chamber and it would be strange if they did not. The lobber breaks, ejects and takes another one. The Denial does not reload at all and never will, which is the whole character of it: five, and then it is a very expensive club.

## The sound set

Every weapon owns its own set. Nothing is shared, because a shared fire sound is the fastest way to make eight guns feel like one gun with different numbers.

| Event | When | Every weapon? |
|---|---|---|
| **fire** | The shot leaves | Yes |
| **cycle** | Between shots: pump, recharge, spin-down, settle | Yes |
| **reload** | Break, eject, load | Lobber only |
| **raise** | You switch to it | Yes |
| **dry** | Trigger pulled on an empty pool | Yes |
| **impact_flesh** | It hits a fighter | Yes |
| **impact_hard** | It hits the world | Yes |
| **pickup** | Claimed from a pad | Yes |

That is eight events across twelve weapons, minus the ones that do not apply, which comes to **eighty-six sounds**. Your fists have no pickup and no dry trigger, because they are never empty and you never find them.

Three more that belong to the economy rather than to any one weapon: an ammunition pickup per pool, a health pickup, and an armour pickup.

### The tin has its own vocabulary

A thrown mine is four sounds and every one of them is doing work. The **throw**, so you know it left. The **arm**, a single tone at a second and a half, which is the sound that tells everyone within earshot that the floor over there is now a problem. The **trigger**, a quarter second before it goes, because instant detonation at twenty ticks a second reads as a bug rather than a mine. And the **detonation**.

### What each one should sound like

The fire sounds that exist already set the register: dry, punchy, no reverb, 1993 arcade. The rest follow from it.

- **Fists** are the sound of somebody who has run out of options. Cloth, breath, and a flat connect with nothing metal in it.
- **Shiv** is cloth and a short scrape. It should sound cheap, because it is.
- **Tack** is flat and unimpressive on purpose. It is the sound of a gun you are trying to replace.
- **Flechette** is the needle chatter that already ships.
- **Scatter** is the boom and the pump that already ships, with the pump promoted to its own cycle so you hear it when you are not firing.
- **Rail** is the electric crack and the cold ring that already ships, and its cycle is the capacitor winding back up, which is the sound that tells an opponent they have one second.
- **Repeater** is a spin-up, a sustained rattle and a spin-down, and the spin-down is the important one because it is the sound of somebody letting go of the trigger near you.
- **Lobber** is a hollow thump, then a break and a clack for the reload.
- **Arc** is a discharge that ends in a settle rather than a tail.
- **Article Blade** is a hum at rest, a swing that changes pitch, and a hit that does not sound like metal on metal.
- **Denial** is the only weapon allowed to sound expensive. It should make people in the room look up.

## Generating them

The spec is `tools/audiogen/specs/sfx-weapons.json` and it runs through the pipeline that already made the three fire sounds in the game.

```
cargo run -p fragr-audiogen -- batch --spec tools/audiogen/specs/sfx-weapons.json --dry-run
```

Dry run reports **77 to generate, about 1612 credits**. The three existing fire sounds are skipped rather than overwritten, so the flechette, scatter and rail keep the voices they already have and the new set is built around them.

Two things the dry run taught, both now baked into the spec. The generator refuses anything under half a second, so a dry click and an impact are both authored at the floor and trimmed afterwards rather than requested short. And the skip-if-exists behaviour means this spec can be run repeatedly as weapons land, generating only what is missing, which is how it should be used: one weapon at a time with `--only`, not eighty-six sounds in one night.

Developer-only. Never in CI, never called by the game.

## Related

- `docs/MODES.md`: where you use them.
- `plans/weapon-economy.md`: the ammunition pools, the pads, and why weapon pads did not matter until now.
- `plans/gunfeel.md`: the measured baselines and the feel work.
- `docs/ART-ASSET-LIST.md`: the view models, world pickups, icons and held sprites each of these needs drawn.
- `tools/audiogen/specs/sfx-weapons.json`: the generation spec for every sound above.
- `docs/lore/guns.md`: the flavour.
