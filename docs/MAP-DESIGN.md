# Map design

How a fragr map is built. `plans/map-scale.md` says how big; this says what goes in it.

These are not aesthetic preferences. Each rule exists because a specific great map did it and a specific bad feeling happens when you do not.

## The current map fails most of these

Arena Duel is one flat square with concentric rings of boxes in it. It has no rooms, no height, no lanes, one kind of space, and nowhere that is worth more than anywhere else. It is a fair test chamber and it is not a level, and nothing below describes it yet. That is the gap.

## 1. No dead ends

**Every room, corridor and space has at least two exits. Three is better.**

This is Facility's rule, from GoldenEye. Almost no room in that map had one way out, so being chased was the start of a plan rather than the end of one: through a door, left, down a vent, and back around behind the person chasing you.

At the speeds this game runs, a dead end kills momentum and feels terrible, and it turns a fight into a corner someone dies in. A map is a racetrack with branching lanes that feed back into each other, not a tree.

**Check:** trace every enclosed space. Two exits minimum, and the exits go somewhere different.

## 2. The best thing sits in the worst place

**The strongest item is in the most exposed position on the map, visible from several vantage points, with no cover on it.**

This is the Longest Yard's rule. The railgun and the heavy armour sat on precarious, open platforms, so taking them meant announcing yourself to the entire server while helpless in the air.

It makes a weapon spawn into bait. Nobody has to be told to fight over the middle; they fight over it because that is where the thing they want is, and wanting it is a decision with a price.

fragr already does this once and should keep doing it: the signature weapon sits at the centre of the arena where there is no cover at all.

**Check:** the map's best item is visible from at least three places you could be shot from.

## 3. Both sides arrive at the same time

**Measure the geometry so that two fighters holding forward from opposite spawns meet at a chokepoint after four or five seconds.**

Counter-Strike's maps are tuned this way, and it is why a player can pre-fire a corner on timing alone. Predictable rollouts are not a limitation, they are the rhythm that makes a map feel competitive rather than random.

At five metres a second, a five second rollout is twenty-five metres of travel each, so the spawns want to be about fifty metres apart along the fastest route, with the chokepoint in the middle of it.

**Check:** walk it. Both directions. Time it.

## 4. Three kinds of space, one map

**Every map has tight, open and vertical ground, and each one makes a different weapon the right answer.**

Blood Gulch is the clearest version: long shots across the open middle, mid-range fights along the rocky flanks, and shotgun range inside the bases. Every weapon had somewhere it was king.

For fragr that means:

- **Tight.** Corridors, corners, doorways. The scatter's ground, and the only place melee is a plan rather than a mistake.
- **Open.** A courtyard or a long hall with sightlines that justify the rail existing.
- **Vertical.** Catwalks, shafts, multiple levels. Where the lobber's splash and a jump matter.

If the whole map is corridors the rail is decoration. If it is all open ground the scatter is. A player should migrate toward the ground that suits what they are carrying, and that migration is most of the interest in a map.

**Check:** name the three zones. If you cannot, it is one zone.

## 5. Floors are permeable

**Elevation is not just stairs. Put holes in it.**

Grates you can shoot through from below, balconies to drop off, shafts to fall down, ledges reachable with a jump. If someone is holding a doorway, the answer should be going over or under it rather than walking into it.

Verticality used this way is a flanking system rather than decoration, and it is the single largest thing Arena Duel is missing now that a jump exists.

**Check:** can you get from the lower floor to the upper one somewhere other than the stairs?

## 6. Big maps need a focus

**The larger the map, the more deliberately it has to push people together.**

Wake Island is a horseshoe, so whatever route you take, the fastest way anywhere crosses the middle, and everyone converges without being told to.

This matters most at the district and field tiers in `plans/map-scale.md`, where the failure mode is not unfairness, it is emptiness. A hundred metre map with nothing pulling toward a centre is a walking simulator with guns.

**Check:** where does the fastest route between any two points go? If the answer is not "through somewhere interesting", move something.

## What this means for the next map

The one to build is arena tier, eighty to a hundred and twenty metres, and it needs, in this order: height, so rule five and the jump exist at all; three named zones, so rule four is satisfied; a loop structure with no dead ends; a measured five second rollout; and the signature weapon somewhere that costs you to stand.

That is a real level rather than a fair box, and it is the difference between a game that measures well and a game somebody wants to play again after work.

## Related

- `plans/map-scale.md`: the size ladder and the vertical axis in the movement step.
- `docs/WEAPONS.md`: what each zone is for.
- `docs/MODES.md`: the bar, which is that none of this is allowed to become a puzzle.
- `docs/DESIGN-REFERENCES.md`: the wider table of what was taken from where.
