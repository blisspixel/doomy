# fragr vision

Working name **fragr** (may change). Do **not** brand as Doom or a Doom clone in user-facing copy. Learn from the history of networked shooters without borrowing their trademarks.

## What it feels like

Think early LAN culture: one of the first times a shooter was something you played **with other people on a network**. A real game with maps to learn, guns that matter, upgrades and unlocks over time. Also the pure arena joy of Quake / Unreal / Halo deathmatch. Visual language: **old-school retro pixel art**, readable silhouettes, loud feedback. Not a photoreal TAC shooter.

Counter-Strike energy for "this is fun with others," but more retro and more theatrical. The new piece is **agentic let's-play**: many agents (clawbots / BYO AI) play the match; humans default to watching like a Fortnite-style spectator hangout, and can join the same fight.

## Non-negotiables

1. **Multiplayer-first.** Solo can exist later as "you + bots," but the product is not a campaign you finish alone. It should be more fun with multiple agents and/or humans than by yourself.
2. **Many agents can play.** Same input pipeline as humans. Rule bots first; smarter agents later. Named agency and intent so watching is readable.
3. **Watch or play.** Spectator-default. Soft join into the live match. Leave back to spectate. Match does not go empty when humans leave (agents keep the server alive).
4. **Guns and maps matter.** Distinct weapon roles, readable arenas / choke points, continuous momentum. Anti-slop: polish feel as you go.
5. **Progression exists in the long game.** Levels / maps to beat, unlocks, upgrades (Halo / Doom-campaign DNA) as later slices. Slice 1 proves the live arena + agents + spectate loop first.
6. **Retro pixel art direction.** Graybox is OK until art lands; target aesthetic is retro pixels, not modern milsim.

## Architecture (unchanged spine)

Godot client presenter + Rust authoritative server + agent-adapter (MCP off the combat tick). $0-first; GCP IaC path for deploy later with spend approval. See `AGENTS.md` and `docs/plans/fragr-exceptional-game.md`.

## What "done" means for fun

People would hang out watching agents scrap, jump in for a round, and come back because the loop is sticky. Not a networking tech demo. Not a pitch deck.
