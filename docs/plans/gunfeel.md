# Plan: gunfeel and aim

**Status:** in flight (aim defaults shipped 2026-09-19)
**Branch:** `feat/gunfeel-*`
**Spend:** $0.

## Goal

Make shooting and aiming feel like the games that got it right, with numbers taken from their sources rather than from taste, and with the parameters in one place so the playtest harness and the QA tour can sweep them. This plan carries the research of 2026-09-18 and the starting parameter set it produced. Movement plumbing lives in `buttery-controls.md`; this plan owns what the weapons and the aim do once the plumbing is honest.

Unit note: one fragr unit is one metre. Classic values are converted at one Quake or Hammer unit equals one inch, which is the common convention and not a claim from id or Valve.

## Two defects the research found in the current code

1. **The default mouse sensitivity is about six times Counter-Strike's.** The client turns 0.003 radians per mouse count, which is 0.172 degrees per count, or 6.6 centimetres per 360 degrees at 800 counts per inch. Counter-Strike 2 ships 0.022 degrees per count at sensitivity 1.25, which is 41.6 centimetres per 360. Competitive players usually sit between 30 and 50. A player cannot aim at six times their muscle memory, and no amount of netcode fixes it. **Shipped:** the client now uses the Source convention, 0.022 degrees per count with a default sensitivity of 1.5, giving 34.7 centimetres per 360 at 800 counts per inch, and the setting is stored in those units so a player can paste a number from another game.

2. **What the code calls weapon "spread" is deterministic aim forgiveness, not dispersion.** `check_hitscan` accepts a target when the angle to it is inside the spread cone and the perpendicular distance is inside two player radii. Every shot inside the cone hits. Rail's 0.04 radians is 2.3 degrees of free aim error, which is console-grade bullet magnetism handed to a mouse; Quake 3's rail has none, and Halo 4's assault rifle bullet magnetism is 3 degrees. The fix is to separate the two ideas: a random dispersion inside a cone (which a player learns to manage) from an aim-assist cone (which only a gamepad gets, and smaller).

## The parameter set

Every number has a reason and a source or an explicit "ours to tune". Damage is per shot, intervals in seconds, cones in radians, ranges in units.

| Parameter | Today | Target | Why |
|---|---|---|---|
| Top speed | 5.0 | 6.5 | Between Counter-Strike's rifle speed (5.46) and Quake 3's (8.13). Crossing the 50 unit arena drops from 10 s to 7.7 s. |
| Acceleration time constant | 0.06 | 0.06 | Ninety-five percent of top speed in 0.18 s, near Quake 3's 0.157 s. Keep. |
| Deceleration time constant | 0.04 | 0.09 | Stop distance 0.20 to 0.59 units; Counter-Strike's is 0.85. Stopping becomes a commitment, which is what gives a movement penalty teeth. |
| Jump or dodge | neither | ground dodge: 2.0x top speed for 0.20 s, 1.2 s cooldown, double tap within 0.25 s, then 0.35x speed clamp for 0.3 s | Unreal Tournament's dodge is 1.5x with a hard landing penalty. A dodge stays in two dimensions: no gravity, no ground state, no three-dimensional hit test, one input bit and one cooldown field on the wire. A jump costs all of that and desynchronises prediction on landing. |
| Flechette | 25 / 0.50 s / 0.10 / 42 | 25 / 0.20 s / 0.045 standing, 0.018 moving / 40 / no falloff / 0.25 s switch | Four shots in 0.60 s bare, 1.4 s through armour. Halving the cone moves "aim matters" from beyond 10 units to beyond 22. |
| Rail | 75 / 2.00 s / 0.04 / 100 | 80 / 1.00 s / 0.012 standing, 0.004 moving / 60 / no falloff / 0.35 s switch | Two shots in 1.0 s mirrors Quake 3's rail (100 damage, 1.5 s). 0.012 radians is 0.7 degrees, near Quake 3's zero-spread rail. Range 60 fits a 50 unit arena. |
| Scatter | 15 / 0.25 s / 0.38 / 14 | 40 / 0.45 s / 0.20 standing, 0.12 moving / 12 / full damage to 4 units then linear to 0.35x at 12 / 0.20 s switch | Three shots point blank in 0.90 s, eight at the edge. Banded falloff follows Titanfall 2 and Counter-Strike. |
| Movement inaccuracy | none | cone multiplied by a lerp from 1.0 to 0.40 with speed over top speed, recovering with a 0.10 s time constant, no penalty below 30 percent of top speed | Counter-Strike's penalty is a pure function of current speed and is zero below 34 percent of the weapon's own maximum. New mechanic. |
| Time to kill | 1.5 to 3.5 s | 0.6 to 1.2 s bare, under 1.8 s through full armour | Counter-Strike's rifle is 0.20 to 0.30 s, Valorant's 0.21, and a modern shooter benchmark is 0.2 to 0.3 s close quarters. fragr is Quake-slow in a Counter-Strike-sized arena with no vertical escape, which is the worst of both. |
| Hit zones | none | none | Quake 3 and Doom both apply a scalar with no hit location; Unreal Tournament gives headshots only to its sniper. Uniform damage also keeps the two-dimensional hit test honest. |
| Hit marker | none | 120 ms marker with a 40 ms audio tick; 250 ms on a kill | Sized against Quake 3's 200 ms pain twitch. Ours to tune. |
| Muzzle flash | none | 20 to 30 ms | Quake 3 uses 20 ms. A pop, not a glow. |
| View kick | none | 0.8 degrees flechette, 2.5 degrees rail, in over 100 ms and out over 400 ms | Quake 3's damage kick uses exactly those two times. |
| Screen shake | none | rotation only, peak 1.0 degree, squared decay over 200 ms | Quake 3's run roll peaks near 1.6 degrees; staying under 2 avoids nausea. Ours to tune. |
| View bob | none | 0.04 units vertical, 0.6 degrees pitch and roll, about 1.8 Hz at top speed | Quake 3's bob is 1.6 units vertical and 0.002 radians of pitch. Bob is much smaller than it feels. |
| Crosshair | fixed | static inner pip plus a detached outer ring showing the current cone | Valve labels its legacy dynamic crosshair "fake recoil, inaccurate feedback"; the split style separates the aim reference from the spread readout. |
| Mouse sensitivity | 0.003 rad per count | **shipped**: Source convention, 0.022 degrees per count, default sensitivity 1.5 (34.7 cm per 360 at 800 counts per inch) | A player can paste a number from another game. The single largest aim fix available. |
| Raw input | accumulated | raw relative motion, accumulation off, no acceleration, no smoothing | Matches the convention every competitive shooter uses. |
| Field of view | implicit 75 vertical (about 107 horizontal at 16:9) | explicit, 90 to 120 horizontal, height-keeping so wider monitors see more, hipfire sensitivity not scaled by it | Quake 3 ships 90 horizontal at 4:3; Counter-Strike 2 ships 75 with a zoom ratio of 1. |
| Gamepad look | flat 2.2 radians per second | radial deadzone 10 percent, outer 95, exponent 2.0, 180 degrees per second cap, friction to 0.6x inside a 3 degree cone, no added magnetism | The hit cone is already magnetism, so the pad gets friction only. Deadzone and exponent are ours to tune; the platform defaults of 24 percent are unusable. |

## Measuring it without human testers

The playtest harness already sees every shot: the server publishes a shot result per fire with hit or miss and the damage. Three additions make the weapon triangle and the time to kill measurable from agents alone, and they need no new wire data:

- Time to kill and shots to kill per victim, as a distribution with the interquartile range, by weapon.
- Accuracy by distance bucket with an interval on each rate, so a fighter with nine shots is not compared naively with one with nine hundred.
- Engagement distance histogram per weapon. The triangle works when each weapon's kills peak in its own band and the overlaps are small.

The rest lives in `benchmark-and-stats.md`. Restricted play is the cheapest balance test: the win rate of an agent forbidden one weapon against one that is not.

Input-to-photon latency needs a camera or a light sensor and stays a manual measurement; the research points at an open harness for it.

## Rungs

1. **Shipped.** Aim defaults: Source-convention sensitivity with a sane default, raw motion, and the setting documented in the units other games use.
2. Weapon table: the damage, interval, cone, range, falloff, and switch values above, behind one constants block the harness can sweep. Time to kill measured before and after.
3. Dispersion separated from aim assist: a random cone per shot, the assist cone gamepad-only and smaller, both on the server.
4. Movement inaccuracy and the recovery constant; the split crosshair that shows it.
5. Feedback: hit marker, muzzle flash, view kick, shake, bob, kill marker.
6. Ground dodge with its cooldown and recovery penalty, in the shared movement step with golden vectors.
7. Field of view and gamepad curves in settings, swept by the feel probes.

## Success criteria

- [x] Default sensitivity within the 30 to 50 centimetres per 360 band at 800 counts per inch, stored in portable units.
- [ ] Time to kill inside the target band, measured by the harness before and after.
- [ ] Each weapon's kill distances peak in its own band, shown by the histogram.
- [ ] Dispersion and aim assist are separate, and the assist is gamepad-only.
- [ ] Every feedback timing implemented and visible in a tour still.
- [ ] The dodge exists, is in the golden vectors, and the playtests prefer it.

## Sources

Quake 3 and its client from the ioquake3 tree (movement constants, kick and bob timings, muzzle flash time); the released Doom source (friction, damage scalar); the Unreal Tournament script mirror (dodge); Counter-Strike item and weapon sources (speeds, accuracy model, falloff); Riot's published netcode and balance posts; a modern shooter's published time-to-kill benchmark; latency studies from NVIDIA and academic work on aiming under latency; Valve's lag compensation source; Godot's camera documentation.
