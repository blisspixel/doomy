# Plan: the weapon economy, and the discovery that there isn't one

**Status:** spec (2026-09-19)
**Branch:** `feat/economy-*` (one PR per rung)
**Spend:** $0 for the mechanics. New view models and icons add roughly six dollars to the art estimate in `art-pipeline.md`.

## The thing we found first

The weapon pads are decoration.

`Action.weapon_swap` is applied unconditionally in the tick loop. Any player, any agent, any bot can hold any of the three weapons instantly, for free, at any moment. Claiming a weapon pad sets the same field the swap already sets. Nobody has ever needed to race anyone for the rail, because the rail was always in their hands if they wanted it.

That explains a measurement that has been sitting in the plans looking like a balance problem. The flechette is 65 to 75 percent of all shots fired and the rail is 2 to 4 percent, and no amount of tuning the weapon table was ever going to move that, because there was no cost to holding the best general-purpose gun and no reason to walk anywhere.

So this plan is not "add ammo to fragr". It is: the pickup economy does not exist yet, and ammo is the thing that makes it exist. One line gates the swap on a set of owned weapons. Everything below is what that line needs around it to be fun rather than annoying.

## The feel to aim at

A kart racer's item box. You get something often and you lose it often. Holding a good weapon is a temporary state you enjoy rather than an inventory you build, and the moment you pick one up is an upgrade moment the way trading up off a dead opponent is in Halo.

That sets the dials. Pads are generous, reserves are thin, and churn beats hoarding. A full pickup is a fight and a half. Melee outlasts a gun because you find it less often, and still breaks, because nothing is permanent except your fists.

## The three rules

Carry a melee, a sidearm and two found weapons. Reload, per weapon, magazine and reserve. Run out, because ammunition is finite and found.

Picking up a third primary means choosing which one hits the floor, under fire, with a number in your head about how much each has left. Running a weapon dry drops you to the sidearm; running that dry drops you to your fists. That descent is supposed to be frightening rather than merely inconvenient, and it is the reason the fists are in the game at all.

The canonical magazines and reload times are in [`docs/WEAPONS.md`](../WEAPONS.md).

## The argument, briefly

**Shared pools, not per-weapon counters.** Doom ran eight weapons on four pools and the sharing is what makes the decision: plasma now, or cells saved for the BFG. Quake did the same and got the item-timing meta this repo has said it wants. Quake 3 then moved to per-weapon counters capped at 200 and ammo stopped mattering in duel, because no single gun can be fired long enough for its own counter to bite. Halo's per-weapon model works, but only because you carry two guns; the two-slot limit is doing the work and the ammo model is a slot model wearing a hat. With eight carried guns it becomes eight counters and no pressure.

Four pools also happens to be the only version that fits the HUD this repo already specified, which allows sprites and at most one line of text.

**Reloads, kept short.** The research argued against them: the cooldown is already the cadence, a magazine adds a second rhythm, and a reload is a second authoritative timer needing prediction and a cancel rule. Every one of those costs is real and they are accepted, because a reload is the rhythm of *not* firing and the gap between the two is where fights are won. The mitigation is the numbers. Nothing takes two seconds, most are near one, and the two slow ones are the weapons where the moment after the shot was always the interesting part.

**Scarce things get charges, not pools.** The signature weapon, the melee upgrade and the thrown mine each carry their own count with their own clock. That is how they can be individually spectacular without destabilising the economy.

## The pools

| Pool | Feeds | Cap |
|---|---|---|
| Tacks | Tack sidearm, Repeater | 220 |
| Darts | Flechette, Scatter | 120 |
| Cores | Rail, Arc | 100 |
| Cans | Lobber | 25 |

Sizes are denominated in kills at a realistic hit rate rather than in shots, checked against the agents' measured 15 to 17 percent so that a bad shot never starves. The load-bearing number is that **the scatter costs four darts per pull**. Without it a fast wide gun is free to spray, which is exactly what the pre-table measurements showed.

## The ladder

Five guns join the three that exist. Every time to kill sits in or beside the 0.6 to 1.2 second band the gunfeel plan asks for.

| Weapon | Role | Damage | Cooldown | Bare kill |
|---|---|---|---|---|
| Fists | Melee, always carried | 20 | 0.40 s | 5 hits, 1.60 s |
| Tack | Sidearm, found beside spawn | 20 | 0.25 s | 5 hits, 1.00 s |
| Flechette | Mid workhorse | 25 | 0.20 s | 4 hits, 0.60 s |
| Scatter | Close shred | 40 falling to 14 | 0.45 s | 3 hits, 0.90 s |
| Rail | Long precision | 80 | 1.00 s | 2 hits, 1.00 s |
| Repeater | Heavy full auto | 14 | 0.10 s | 8 hits, 0.70 s |
| Lobber | Splash, projectile | 65 direct, 45 splash | 0.80 s | 2 hits plus travel |
| Arc | Energy, ignores armour | 18 | 0.15 s | 6 hits, 0.75 s |
| Denial | Signature, five charges | 250 | 1.25 s | One hit |

The arc earns its slot on a case the current triangle cannot answer. Against a fighter at full armour and full health the flechette needs eight hits and the rail three; the arc still needs six, because it ignores armour. That is a reason to exist beyond glowing.

## What stops it being annoying

Three deliberate valves, because "limits that make sense, not an annoying limit on ammo" is the whole brief.

**You spawn with your fists and nothing else.** Doom starts you with fists and a pistol; this keeps the fists and puts the pistol on the ground.

The research argued the opposite case and argued it well: the respawn delay is three seconds, the arena is fifty metres, a rail covers the whole floor, and a bare spawn will push spawn deaths through the threshold the harness enforces. That risk is real. It is answered by the maps rather than by the loadout.

**There is a sidearm within about two seconds of every spawn point.** That is a map rule, it is checkable, and it is what turns a weaponless spawn from a death sentence into the first decision of a life. The pistol stops being something you have and becomes the first thing you do, every time.

Three supports underneath it. Spawn protection already exists and covers the grab. The sidearm pad runs on the valve clock rather than the contested clock, because denying somebody the ability to fight at all is a different thing from beating them to a rail, and the first should not be possible. And the sidearm's ammunition trickles back, so nobody is ever reduced to their hands by arithmetic, only by circumstance.

What this buys is a window. Every life has a few seconds in it where you are holding nothing, which makes a punch kill possible and makes anyone who catches you in that window feel like they earned it. It is also the only way the melee ladder means anything: if you always had a knife, finding a knife would not be a moment.

If the measurements say it does not work, the fix is a shorter walk to the sidearm, not a weapon in the loadout. The spawn-death threshold now judges the interval rather than the raw ratio, so it will report a real regression here instead of noise, and time-to-first-shot after respawn is the number that says whether the walk is the right length.

**The sidearm refills itself.** One tack every two seconds while below forty and while you have not fired for a second. Filling from empty takes eighty seconds, so it is a safety net across a round rather than a resupply. This is fragr's version of the chainsaw in Doom 2016: it is what lets the other three pools be genuinely tight without the game ever reaching a dead end. You can always fight. You frequently cannot fight the way you want to.

**Ammo cannot be denied, geometrically.** The four small ammo pads sit on a ring that takes about sixteen seconds to circle against a ten second respawn clock. A denier is always behind the respawns. Starving an opponent of ammo is not unlikely, it is impossible for one player.

And one rule about dry triggers: pulling an empty trigger plays a click, costs nothing, starts no cooldown, and emits no shot. It must not auto-swap either, because auto-swapping on a pull costs the player the fight they were in.

## Melee, which you always have

**The Shiv.** 35 damage, two metre reach, 0.55 second swing. Three hits kill, which is deliberately the slowest bare time to kill on the board, slower even than the sidearm. Melee is a last resort and a finisher, never a strategy.

**The backstab is a one-shot execute**, as it is in Counter-Strike and Team Fortress, and it is gated four ways because our simulation has no verticality and the reference agents path straight at their target. It needs the rear 120 degree arc, a shorter 1.6 metre reach, its own three second cooldown, and no gunfire in the previous half second. This is the mechanic most likely to need a nerf after first measurement, and the fourth gate below is what catches it.

**The Article Blade** is the energy sword. One per map, adjacent to centre, sixty second clock, replaces the Shiv while held, lost on death. 70 damage on a 0.45 second swing, with a short authoritative lunge, and twelve swings before it dies and returns to its pad. A one-hit-kill sword in a fifty metre arena with no verticality would be oppressive, which is why ours is a two-hit kill with a lunge instead.

## The proximity tin

Carry three. The memorable part of GoldenEye's mines was never the quantity, it was that each one was a decision.

Arms in 1.5 seconds, triggers at 2.5 metres, quarter second fuse so there is a beep and a chance to run, 90 damage at the centre falling to 25 at four metres. It does not one-shot through armour, which keeps the armour pad meaningful. It does half damage to its owner, which is both the joke and the brake.

**It is visible to everyone, always.** An invisible authoritative entity is unfair on a twenty hertz wire and unreadable for spectators and agents. The skill is placement, behind a pad or around a choke, not concealment.

It resolves in the same tick phase as hitscan and emits ordinary hit and frag events, so the killfeed, the most-valuable-player logic, killstreaks and the playtest harness all work unchanged. Its cover check reuses the ray test that already exists, so a low wall shields you from a blast.

## The map, and the one number that matters

The three weapon pads currently form a loop of about 67 metres, which is 13.4 seconds of running. The respawn clock is twelve seconds. **The loop is slower than the clock**, so one runner can hold all three weapons forever and there is no timing skill at all.

Raising the clock to twenty seconds and extending the loop to six pads makes the circuit about 21.6 seconds against a twenty second clock, so one runner can hold about half the pads and never all of them. That is the tension the design references have been asking for, stated as arithmetic.

The rule that generalises to any map size, and which belongs in `map-scale.md`: **the clock must exceed the loop for contested items, and fall below it for the valve items.**

A sidearm pad sits beside every spawn point on an eight second clock, which is short enough that it is never worth contesting and is the support the whole starting loadout rests on. Beyond that, pads go on four rings: the signature weapon alone at centre where there is no cover, health and armour and the blade at eight metres, the four ammo pads at 12.5 metres tucked behind the low walls so resupply is contested by geometry already built, weapons at seventeen and nineteen metres, and two big crates in the deep corners on a thirty second clock, because the big refill should be a commitment.

## What the protocol has to add

The architectural call worth stating plainly: **inventory is unicast, not broadcast.** Putting ammo and owned weapons on every player state in every snapshot would add about 120 bytes per player per tick to a measured 300 byte baseline, a forty percent growth, taking the snapshot at 64 fighters from about 11.8 kB to 16 kB and the broadcast from 15 MB/s to 21 MB/s. Bandwidth is already the wall this scale hits first. Nobody needs an opponent's ammo count. Quake 3 does exactly this, and the fog of war is a feature.

So: a new `Loadout` message sent to one fighter on change only, a `tins` array in the snapshot, two new action bits, five new weapon variants, and a `weapon` field on shot results because inferring the weapon from the snapshot is already fragile and becomes wrong the moment a player can swap in the tick they fire.

One test inverts rather than extends: the weapon swap test currently swaps to a rail nobody owns and asserts it worked. It has to assert the swap is refused.

## The risk, and how it gets caught

The risk is not that players run out of ammo. It is that **the round becomes a supply run**, fights end because someone ran dry rather than because someone won, and the measured time to kill goes back above two seconds with a worse tail.

This is not hypothetical. The planner tier did nothing but give agents a reason to route to pads and hold range, and it moved time to kill from 0.90 seconds to 2.05 and the ninetieth percentile to 7.65. That was one weak reason not to be fighting. Ammo is a much stronger one.

The second order version bites first: **the sidearm becomes the gun people actually fight with**, because it is the only one always loaded. If the trickle is generous enough to feel safe it is generous enough to be optimal, and the ladder becomes scenery.

Four gates, all numbers the harness already prints, run on the same three seeds so the published baselines are the comparison:

1. Time to kill p50 must not exceed 1.40 seconds, and p90 must not exceed today's worst of 7.65.
2. Frags per minute must stay within fifteen percent of baseline and the longest gap without a frag must not exceed twenty seconds. A drop in frags with unchanged accuracy means time moved from fighting to walking.
3. No single weapon may exceed 45 percent of fire ticks. Today the flechette fails this at 65 to 75 percent, and fixing that is the economy's entire job. If the sidearm lands above 45, the trickle is too generous.
4. Each weapon's kill distance median must sit inside its own band. This also catches an overpowered backstab, which shows up as kill distance collapsing back toward two metres.

Two numbers to add to the harness first, both cheap:

- **Dry trigger ratio.** Above eight percent of fire ticks and the economy is stingy. This turns "annoying limit on ammo" from a feel complaint into a threshold.
- **Time to first shot after respawn.** Above three seconds and the spawn kit is too thin.

## Order

Sequencing decides whether any of this is measurable.

0. **Teach the reference agents ammo and tins first.** The harness is the only feedback loop this design has, and an agent spraying a dry gun at a wall makes every gate above meaningless.
1. **Gate the swap on owned weapons.** One line, plus the tests that invert. Measure immediately: this alone should move the weapon mix.
2. **Pools, the four ammo types, pad grants and the sidearm trickle.** The dry trigger ratio lands with it.
3. **The pad rings and the new clocks**, which is where the timing meta appears.
4. **The Shiv and the backstab.**
5. **The five new guns**, one at a time, each measured against gate three before the next.
6. **The tin, then the Blade, then the Denial.**

## Related

- `gunfeel.md`: the weapon table these numbers extend, and the measured baselines.
- `massive-arenas.md`: why inventory is unicast.
- `map-scale.md`: the clock-versus-loop rule belongs there too.
- `arena-choke-geometry.md`: the low walls the ammo pads hide behind.
- `agent-playtest-loop.md`: the harness that runs the four gates.
