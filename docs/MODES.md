# How you play it

The canonical list of modes. The roadmap sequences them, the plans build them, this says what each one is.

## The shape

**Single player is Doom and GoldenEye.** Episodes of hand-built maps with keys, secrets, par times and an escalating enemy roster, and objectives that change with the difficulty you picked rather than enemies that simply take more shots.

**Multiplayer is against whoever is there.** Other people, agents, the Office's units, or all three in the same round. The seat is the same seat, which is the thing that makes this game different from the ones it is copying, and it means every mode below works with any mix of participants without a separate code path.

**Watching is the default.** You arrive in the booth. Joining is a decision you make, leaving is a decision you make, and the match does not stop for either.

## Single player

### Episodes

The Doom spine. Eight maps to an episode, built to teach in order: movement, then the scatter, then keys, then the rail, then secrets, then turrets, then the auditor, then the boss. Every map has a par time and a secret count, and the map tells you what it is about in the first room rather than in a briefing.

Keys are red, gold and cyan, and they gate doors rather than granting abilities, because a key that changes what you can do turns a level into a progression system.

The enemy roster in `docs/ENEMIES.md` is the difficulty curve. A map is hard because of which shapes it puts in which rooms, not because the numbers went up.

### Objectives, from GoldenEye

The difficulty tier does not scale health. It adds objectives.

On the easiest tier you reach the exit. A tier up and you also have to seize a jammer, or reach a terminal before it finishes transmitting, or get out without the Office logging your handle. The map is the same map. What you have to do in it is not.

This is the single best idea GoldenEye had and almost nobody copied it: replaying a level on a harder tier is a different level, and the player who has learned the geometry gets to spend that knowledge rather than re-earn it.

### Solo Broadcast

The episode zero that already exists, and the tutorial that does not admit to being one.

## Multiplayer

The same four formats work with people, with agents, with the Office's units, or with a mix, because a fighter is a fighter on the wire.

### Scrap, against each other

Free-for-all and teams. Frags, a limit, a clock. The thing a scrap league runs on a Tuesday.

Humans and agents share the roster and always have. Nobody is separated into a lane by default, and the fair-play work exists so that a person who wants a humans-only or agents-only lane can have one, not so that the mixed case needs defending.

### Arcade ladder, against the Office

Rounds versus escalating rosters with a boss beat every third round, a results card, and a local best. Co-op from the start, because the server already seats several fighters and there was never a reason to make it solo.

### The Sweep, which you cannot win

Hordes, escalating, forever. The round you fell on is the score.

Frags earn points, points open the next section of the map and buy off the pads, so the arena grows as you last. A downed partner can be picked up for a few seconds. The Host counts rounds like a countdown.

It is the most popular night of the week and nobody at the venue will explain why.

### Counter-op

One spectator seat possesses the Office's units in turn and plays them against the party. The fair-play lanes keep it honest.

This is cheap to build, because the units already exist as server entities with a controller seam, and it is the single best answer to the question of what a spectator does when watching stops being enough.

## Mutators, before any of the big modes

Cheap twists on rules that already exist, in the spirit of the couch multiplayer everyone remembers: rail only, scatter only, one golden rail on the map, melee only, one shot kills.

These are worth building before team modes with squads or objective control on larger maps, because they cost almost nothing and they are where a lot of the fun actually is.

## What holds it together

The same arena, the same weapons found on the same floor, and the same enemy roster whichever mode you are in. A player who learns the crawler in an episode knows the crawler in the Sweep. A player who learns where the rail spawns in a scrap knows where it spawns in co-op.

Nothing in this list needs a separate build of the game, and nothing in it needs a mode to be chosen before the round in a way that stops somebody joining halfway through.

## Related

- `docs/ENEMIES.md`: what you fight when you are not fighting each other.
- `docs/WEAPONS.md`: what you find on the floor.
- `plans/campaign-continuance.md`: the episodes, the roster, and the level format.
- `plans/map-scale.md`: the sizes the bigger modes need.
- `plans/fair-play.md`: the lanes.
- `docs/DESIGN-REFERENCES.md`: what was taken from where, and why.
