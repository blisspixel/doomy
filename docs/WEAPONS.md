# The weapons

The canonical list. Eleven things you can hold, what each is for, where you get it, and every sound it makes.

Balance numbers live here and nowhere else. `plans/gunfeel.md` explains how they were arrived at, `plans/weapon-economy.md` explains the ammunition and the pickup economy, and `docs/lore/guns.md` is what they get called on the radio.

## What you start with

**A knife and a pistol.** That is the whole loadout, on every spawn, for everyone.

Everything else in this file is found on the floor, kept until you die, and lost when you do. That is Doom's rule rather than Quake's, and it is the rule the pickup economy is built on: a starter weapon you always have is a starter weapon nobody ever leaves, and walking to a gun has to be a decision or the map is just scenery.

Nobody is a class. There are no loadouts and no roles: if you are sniping it is because you walked to where the rail was. The enemies are the opposite, and deliberately so. A Continuance unit has one shape, one behaviour and one attack, and it never varies, so you learn a silhouette once and know it forever. That roster is [`docs/ENEMIES.md`](./ENEMIES.md).

## The ladder

| # | Weapon | Role | Damage | Cooldown | Bare kill | Ammunition | Where |
|---|---|---|---|---|---|---|---|
| 1 | **Shiv** | Melee, always carried | 35 | 0.55 s | 3 hits, 1.10 s | none | Spawn |
| 2 | **Tack** | Sidearm, always carried | 20 | 0.25 s | 5 hits, 1.00 s | Tacks | Spawn |
| 3 | **Flechette** | Mid workhorse | 25 | 0.20 s | 4 hits, 0.60 s | Darts | Pad |
| 4 | **Scatter** | Close shred | 40 falling to 14 | 0.45 s | 3 hits, 0.90 s | Darts, 4 per pull | Pad |
| 5 | **Rail** | Long precision | 80 | 1.00 s | 2 hits, 1.00 s | Cores, 2 per shot | Pad |
| 6 | **Repeater** | Heavy full auto | 14 | 0.10 s | 8 hits, 0.70 s | Tacks | Pad |
| 7 | **Lobber** | Splash, projectile | 65 direct, 45 splash | 0.80 s | 2 hits plus travel | Cans | Pad, outer ring |
| 8 | **Arc** | Energy, ignores armour | 18 | 0.15 s | 6 hits, 0.75 s | Cores | Pad, outer ring |
| 9 | **Proximity tin** | Thrown, placed | 90 at centre | 1.5 s to arm | Area denial | 3 carried | Pad |
| 10 | **Article Blade** | Melee upgrade | 70 | 0.45 s | 2 hits, 0.45 s | 12 swings | Plinth, near centre |
| 11 | **Denial** | Signature | 250 | 1.25 s | 1 hit | 5 charges, no refill | Plinth, centre |

Four ammunition pools feed the eight guns. Tacks for the sidearm and the repeater, Darts for the flechette and the scatter, Cores for the rail and the arc, Cans for the lobber. The tin, the blade and the signature weapon carry their own counts and sit outside the pools entirely.

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

That is eight events across eleven weapons, minus the ones that do not apply, which comes to **eighty-two sounds**.

Three more that belong to the economy rather than to any one weapon: an ammunition pickup per pool, a health pickup, and an armour pickup.

### The tin has its own vocabulary

A thrown mine is four sounds and every one of them is doing work. The **throw**, so you know it left. The **arm**, a single tone at a second and a half, which is the sound that tells everyone within earshot that the floor over there is now a problem. The **trigger**, a quarter second before it goes, because instant detonation at twenty ticks a second reads as a bug rather than a mine. And the **detonation**.

### What each one should sound like

The fire sounds that exist already set the register: dry, punchy, no reverb, 1993 arcade. The rest follow from it.

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

Dry run reports **72 to generate, about 1512 credits**. The three existing fire sounds are skipped rather than overwritten, so the flechette, scatter and rail keep the voices they already have and the new set is built around them.

Two things the dry run taught, both now baked into the spec. The generator refuses anything under half a second, so a dry click and an impact are both authored at the floor and trimmed afterwards rather than requested short. And the skip-if-exists behaviour means this spec can be run repeatedly as weapons land, generating only what is missing, which is how it should be used: one weapon at a time with `--only`, not eighty-two sounds in one night.

Developer-only. Never in CI, never called by the game.

## Related

- `plans/weapon-economy.md`: the ammunition pools, the pads, and why weapon pads did not matter until now.
- `plans/gunfeel.md`: the measured baselines and the feel work.
- `docs/ART-ASSET-LIST.md`: the view models, world pickups, icons and held sprites each of these needs drawn.
- `tools/audiogen/specs/sfx-weapons.json`: the generation spec for every sound above.
- `docs/lore/guns.md`: the flavour.
