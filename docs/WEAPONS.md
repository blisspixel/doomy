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

Magazine is what is in the weapon; reserve is what you are carrying for it. Reload swaps one for the other and costs the time in the table.

| # | Weapon | Role | Damage | Cooldown | Magazine | Reload | Ammunition | Where |
|---|---|---|---|---|---|---|---|---|
| 1 | **Fists** | Melee, always carried | 20 | 0.40 s | none | none | none | Always |
| 2 | **Shiv** | Melee, found | 35 | 0.55 s | 25 hits | none | breaks | Pad, common |
| 3 | **Tack** | Sidearm, found | 20 | 0.25 s | 12 | 0.9 s | Tacks | Pad, beside every spawn |
| 4 | **Flechette** | Mid workhorse | 25 | 0.20 s | 30 | 1.1 s | Darts | Pad |
| 5 | **Scatter** | Close shred | 40 falling to 14 | 0.45 s | 6 | 1.3 s | Darts | Pad |
| 6 | **Rail** | Long precision | 80 | 1.00 s | 4 | 1.4 s | Cores | Pad |
| 7 | **Repeater** | Heavy full auto | 14 | 0.10 s | 60 | 1.8 s | Tacks | Pad |
| 8 | **Lobber** | Splash, projectile | 65 direct, 45 splash | 0.80 s | 1 | 1.0 s | Cans | Pad, outer ring |
| 9 | **Arc** | Energy, ignores armour | 18 | 0.15 s | 24 | 1.2 s | Cores | Pad, outer ring |
| 10 | **Proximity tin** | Thrown, placed | 90 at centre | 1.5 s to arm | 3 carried | none | none | Pad |
| 11 | **Article Blade** | Melee upgrade | 70 | 0.45 s | 12 swings | none | none | Plinth, near centre |
| 12 | **Denial** | Signature | 250 | 1.25 s | 5, no refill | never | none | Plinth, centre |

Three melee tiers, six guns and a sidearm, a thrown mine and a signature weapon. Four ammunition pools feed the guns.

Reserve carried, in magazines: sidearm three, flechette three, scatter three, rail two, repeater two, lobber four charges, arc two. A full pickup is a fight and a half.

Reload times are all under two seconds and most are close to one, because the measured time to kill is under a second and a half and a reload has to be a decision rather than a nap. The rail and the lobber are slow on purpose: they are the weapons where the moment after the shot is the interesting part. Tacks for the sidearm and the repeater, Darts for the flechette and the scatter, Cores for the rail and the arc, Cans for the lobber. The tin, the blade and the signature weapon carry their own counts and sit outside the pools entirely.

## Weapons are consumable

The closest thing to how this should feel is a kart racer's item box. You are getting something often. You are also losing it often. Holding a good weapon is a temporary state you enjoy and then lose, not an inventory you build.

So the pads are generous and the reserves are thin. You will find a rail several times in a round and you will fire it maybe nine times each time you do. The moment you pick something up is an upgrade moment, the way it is in Halo when you trade up off a dead opponent, and the moment it runs dry is a real event that changes what you are doing.

**Melee lasts longer and still ends.** The shiv takes twenty-five hits before it breaks. The Article Blade has twelve swings and returns to its plinth. Melee outlasts a gun because you find it less often, and it still runs out, because nothing here is permanent except your fists.

**Reserves are two or three magazines, not ten.** The exact numbers are in the table. The feel to aim at is that a full pickup is a fight and a half, not an afternoon.

**The campaign is the other axis.** In an arena everything is on the floor from the first second and the churn is the whole game. In an episode it works the way Doom and Duke Nukem did: you start with almost nothing, the ladder opens up as the episode goes, and the strong weapons arrive late and are hard to find. The first time you round a corner onto a rail should be a moment, and the episode that hands it over should have made you wait for it.

Same weapons, same numbers, different availability. A map decides which rungs exist in it, and an episode decides the order you meet them in. The best things are late and hidden, and a secret worth finding is usually a weapon you were not supposed to have yet.

## You carry four things, you reload, and you run out

Three rules, and they are the point of the whole design.

**You carry a melee, a sidearm, and two found weapons.** Not twelve. Picking up a third primary means choosing which one hits the floor, and you make that choice under fire with a number in your head about how much ammunition each one has left. A loadout you never have to edit is not a loadout, it is a menu you looked at once.

**Every weapon reloads.** A magazine and a reserve, per weapon, and the reload takes time you do not have. The cooldown is the rhythm of firing and the reload is the rhythm of not firing, and the gap between them is where fights are actually won. Reload timings are short enough that a competent player reloads in cover rather than never: nothing here takes two seconds.

**You run out.** Ammunition is found, it is finite, and a weapon whose reserve is empty is dead weight you are carrying instead of something better. Running dry drops you to the sidearm; running the sidearm dry drops you to your fists. That descent is a real thing that happens in a long fight, and it is supposed to be frightening rather than merely inconvenient.

The Denial does not reload and never will. Five charges, and then it is a very expensive club.

## The sound set

Every weapon owns its own set. Nothing is shared, because a shared fire sound is the fastest way to make eight guns feel like one gun with different numbers.

| Event | When | Every weapon? |
|---|---|---|
| **fire** | The shot leaves | Yes |
| **cycle** | Between shots: pump, recharge, spin-down, settle | Yes |
| **reload** | Magazine out, magazine in | Yes, except the fists and the Denial |
| **raise** | You switch to it | Yes |
| **dry** | Trigger pulled on an empty magazine | Yes |
| **empty** | The reserve is gone too, and this weapon is finished | Yes |
| **impact_flesh** | It hits a fighter | Yes |
| **impact_hard** | It hits the world | Yes |
| **pickup** | Claimed from a pad | Yes |

That is nine events across twelve weapons, minus the ones that do not apply. Your fists have no pickup, no reload and no dry trigger, because they are never empty and you never find them.

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

Dry run reports **92 to generate, about 2152 credits**. The three existing fire sounds are skipped rather than overwritten, so the flechette, scatter and rail keep the voices they already have and the new set is built around them.

Two things the dry run taught, both now baked into the spec. The generator refuses anything under half a second, so a dry click and an impact are both authored at the floor and trimmed afterwards rather than requested short. And the skip-if-exists behaviour means this spec can be run repeatedly as weapons land, generating only what is missing, which is how it should be used: one weapon at a time with `--only`, not ninety-five sounds in one night.

Developer-only. Never in CI, never called by the game.

## Related

- `docs/MODES.md`: where you use them.
- `plans/weapon-economy.md`: the ammunition pools, the pads, and why weapon pads did not matter until now.
- `plans/gunfeel.md`: the measured baselines and the feel work.
- `docs/ART-ASSET-LIST.md`: the view models, world pickups, icons and held sprites each of these needs drawn.
- `tools/audiogen/specs/sfx-weapons.json`: the generation spec for every sound above.
- `docs/lore/guns.md`: the flavour.
