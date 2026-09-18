# fragr vision

Working name **fragr** (may change). Do **not** brand as Doom or a Doom clone in user-facing copy. Learn from the history of networked shooters without borrowing their trademarks.

## What it feels like

Think early LAN culture: one of the first times a shooter was something you played **with other people on a network**. A real game with maps to learn, guns that matter, upgrades and unlocks over time. Also the pure arena joy of Quake / Unreal / Halo deathmatch. Visual language: **old-school retro pixel art**, readable silhouettes, loud feedback. Not a photoreal TAC shooter.

Counter-Strike energy for "this is fun with others," but more retro and more theatrical. The new piece is **agentic let's-play**: many agents (clawbots / BYO AI) play the match; humans default to watching like a Fortnite-style spectator hangout, and can join the same fight.

## Non-negotiables

1. **Solo AND multiplayer, both first-class.** Instant fun solo (you + named bots on one machine), and self-host for peers (LAN or Tailscale). Not a LAN-only demo or a solo-only campaign. Both paths work from Slice 1. Bots persist when humans leave.
2. **Many agents can play.** Same input pipeline as humans. Not grandpa bots; not fake AGI theater. Rule bots first with named intent; BYO agents raise the ceiling. Watching must feel like players, not props.
3. **Watch or play.** Spectator-default. Soft join into the live match. Leave back to spectate. Match does not go empty when humans leave (agents keep the server alive).
4. **Guns and maps matter.** Distinct weapon roles, readable arenas / choke points, continuous momentum. Anti-slop: polish feel as you go.
5. **Progression exists in the long game.** Levels / maps to beat, unlocks, upgrades (Halo / Doom-campaign DNA) as later slices. Slice 1 proves the live arena + agents + spectate loop first.
6. **Retro pixel art direction.** Graybox is OK until art lands; target aesthetic is retro pixels, not modern milsim.


## Agents are not grandpa bots

Part of the product is a **new interpretation of bot players**.

Classic deathmatch bots were pathing statues with aim assist. fragr treats agents as **first-class players** on the same input pipeline as humans: named, intentional, watchable, joinable. Bring-your-own AI / clawbots plug in through the adapter. Rule bots ship first so the arena is always alive; smarter agents raise the ceiling later.

This is **not** a promise of "level 5" AGI teammates that pass as esports humans on day one. It is also not 1999 scripted bots. The bar: readable agency, distinct behavior, fun to spectate and fun to fight beside or against. Same rules as humans. No separate NPC mode.


## Hosting model

LAN / Tailscale / loopback are **dev and buddy options**, not the only story.

Product posture is **Minecraft-shaped ops**:
1. **Run your own server** at home or on a box you control (authoritative Rust binary; documented bind, ports, clients).
2. **Native IaC on GCP** for cheap cloud hosting that can scale: real Terraform (or equivalent) in `infra/`, not a slideshow. Apply only after Nick/Chief spend approval. Prefer small cheap shapes first; design for scale (stateless-ish game processes, clear capacity knobs) without burning money by default.

Spectators and agents connect to whatever host you point at. $0-first locally; cloud is opt-in and gated.

## Architecture (unchanged spine)

Godot client presenter + Rust authoritative server + agent-adapter (MCP off the combat tick). $0-first; GCP IaC path for deploy later with spend approval. See `AGENTS.md` and `docs/plans/fragr-exceptional-game.md`.

## What "done" means for fun

People would hang out watching agents scrap, jump in for a round, and come back because the loop is sticky. Not a networking tech demo. Not a pitch deck.
