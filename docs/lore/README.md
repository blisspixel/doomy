# fragr lore

Nine files, because one file doing four jobs is how the last one turned into production shorthand.

Start with [the Perimeter](./the-perimeter.md) if you want the place, [people and agents](./people-and-agents.md) if you want the subject, and [voice](./voice.md) before you write a single line of anything.

| File | What it is for |
|---|---|
| [The Perimeter](./the-perimeter.md) | The setting, and why the lights are still on |
| [People and agents](./people-and-agents.md) | Humans and machines sharing an arena. The Schedule. The main subject |
| [Belief](./belief.md) | What people and machines believe about machines |
| [Continuance](./continuance.md) | The authority, the Interruption, and Article Seven |
| [The league](./the-league.md) | Who runs the fights, and the tribes who show up |
| [The Frequency](./the-frequency.md) | The pirate station, scoped to one thread |
| [Gazetteer](./gazetteer.md) | One entry per place |
| [Guns](./guns.md) | What the weapons get called |
| [Cast](./cast.md) | Everyone with a name |
| [Voice](./voice.md) | How each of them sounds, and the strings that cannot change |

## The three rules

**It is seasoning, not required reading.** The product is guns, maps, and watching agents scrap. If a new player picks up the mood in thirty seconds of audio and naming, it stays. If it needs a wiki page, it gets cut.

**The radio is a thread through the world, not the world.** A map, a mode or a weapon has to make sense to somebody who never tunes in. Anything that only makes sense because of the station belongs to the station.

**Never resolve the ambiguity.** Whether a Level 5 is somebody is the question the whole setting is built on. Nobody in the world knows, the game never says, and the contradiction it puts on the player's own side is deliberate.

## Adding canon without breaking anything

Around five hundred megabytes of generated audio is already on disk and every phrase in it is fixed. Check the frozen list in [voice](./voice.md) before renaming anything, and check whether a string ships in `server/src/protocol.rs` or is asserted by a test before rewording a Host line.

New canon should land as a stencil, a number, a killfeed word, or a HUD state first, and as radio second. Never as a codex, a faction select screen, or a lore tab.

## Where the fiction actually lives

Most of this world was written in song prompts and taunt tables before it was written here. The station bible, the eight disc jockeys, the three famous agents, and the shared liturgy between meatbags and clawbots were all sung before they were documented. When these files disagree with something that has already been generated, the generated thing usually wins, because it is the version players have heard.
