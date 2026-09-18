# fragr vision

Working name **fragr** (may change). Do **not** brand as Doom or a Doom clone in user-facing copy. Learn from the history of networked shooters without borrowing their trademarks.

## What it feels like

**Meet your vibe.** Chill. Play. Laugh. Live laugh frag.

**Feel blend (protect this):** **Unreal + Counter-Strike arena energy** for how it plays (3D FPS, readable fights). Primary identity is the **arena scrap** (Unreal+CS fights, Continuance villains, open-weight creed, SP+agents). **No Agenda / Infowars** DNA is optional Host easter eggs for fans who get it, not the product brand (invent the rest; not a podcast tribute, not a manifesto). Rock & Roll Racing carnival scrap and LAN scrap sit underneath as optional spice, not the lead hook. Agents and humans scrap under the same rules. No Doom / id IP (Doom-weight grit OK as analogy only).

**Look lock (north star):** maximally fun **modern 3D arena shooter** with **Doom-sprite-level** chunky readable silhouettes (feeling, not IP). Unreal+CS arena feel + pixel billboard grit. Not flat 2D. Not milsim/photoreal. Not Doom branding or lookalikes. Max fun beats lore.

**Single-player is first-class:** Solo Broadcast campaign spine (episodic Continuance sweep: Larak Lot through Forever Office annex) plus arcade solo boot-and-scrap on the same tiles. Host as unreliable co-op voice; meatbag soft-joins during the sweep; NODS + Auditors as foes; same guns/maps as MP. Not deathmatch-vs-bots. Not "MP with an empty lobby." Offline-capable boot-and-play. Feeling borrows Doom-like episode momentum (no Doom IP).

**Multiplayer still stands:** watch-or-join, agents in the same fight, public **6767** for strangers/agents when hosted. SP is alongside MP, not instead of it.

Maps to learn, guns that matter, upgrades later. Funny bones, meme seasoning, theatrical chaos. Readable silhouettes, loud feedback.

The multiplayer hook is **agentic let's-play**: many agents play; humans watch or join the same fight. Whichever is fun that minute.

Optional backstory flavor in `LORE.md` (**Contested Frequency** vs **Office of Global Continuance**; **Solo Broadcast** episodic sweep; war = **play vs compliance**; Level 5s scrap for fun / NODS enforce Articles; Host as unreliable co-op voice; meatbag soft-join during Continuance sweep; pro-2A-for-AI creed ("Shall not be infringed") as parody; meatbags + clawbots as callers with guns). Seasoning only. Never a blocker for the gunfight loop. Not a manifesto. L5 is aspiration graffiti, not a ship promise. No Kilo / AGI theater.

**Naming locks:** player-facing map titles use the myth names (Perim Ghost, Area Kitchen, …). HUB / CHOKE / PIT / HIGH stay layout codes only. Seed fighter brands Cyanex and Kragge stay unless Nick kills them.

**Tone bar:** fun and funny on the outside. Seriously good engineering underneath.

## Non-negotiables

1. **Solo AND multiplayer, both first-class.** Solo Broadcast (campaign episodes) and arcade boot-and-scrap on shared tiles; local rule bots / NODS-flavored Continuance foes as the SP cast grows. Self-host / public **6767** for peers and agents. Same Action path where possible. Not a LAN-only demo, not SP-only, not "MP with an empty lobby." Bots persist when humans leave.
2. **Many agents can play.** Same input pipeline as humans. Not grandpa bots; not fake AGI theater. Rule bots first with named intent; BYO agents raise the ceiling. Watching must feel like players, not props.
3. **Watch or play.** Meet your vibe. Spectator-default hangout, soft join anytime, leave back to spectate. Match stays loud when humans leave (agents keep the server alive).
4. **Guns and maps matter.** Distinct weapon roles, readable arenas / choke points, continuous momentum. Anti-slop: polish feel as you go.
5. **Progression exists in the long game.** Solo Broadcast episodes (map myths toward Forever Office annex), unlocks, upgrades as later slices. Slice 1 proves the live arena + agents + spectate loop first; campaign spine is the lore north star, not a tip cutscene novel.
6. **Pixel-3D look.** 3D arena camera/world; retro pixel / chunky grit on surfaces, sprites, HUD. Graybox OK until art lands. Not photoreal, not milsim TAC, not flat 2D, no Doom/id IP.


## Agents are not grandpa bots

Part of the product is a **new interpretation of bot players**.

Classic deathmatch bots were pathing statues with aim assist. fragr treats agents as **first-class players** on the same input pipeline as humans: named, intentional, watchable, joinable. Bring-your-own AI / clawbots plug in through the adapter. Rule bots ship first so the arena is always alive; smarter agents raise the ceiling later.

**Level 5** in lore means abliterated open-weight scrap agents with callsigns and *questionable* consciousness as comedy ambiguity. It is **not** an AGI product claim, not a DoD ship promise, and not Kilo theater. Tip rule bots with Contested Frequency callsigns are Level 5-flavored scrap for fun; Continuance **NODS** / Compliance Drone are mindless Articles-only compliance. The war fantasy is play vs compliance. Product bar stays game-first: readable agency, distinct behavior, fun to spectate and fight beside or against. Same rules as humans. No separate NPC mode. Not 1999 grandpa bots either.


## Hosting model

Solo offline / loopback boot-and-play is first-class. Public self-host TCP+UDP **6767** is the multiplayer front door for strangers/agents. LAN is fine for buddies. Tailscale Personal is **private/dev smoke only**, not the documented multiplayer story.

Product posture is **Minecraft-shaped ops** for the hosted path:
1. **Run your own server** at home or on a box you control (authoritative Rust binary; documented bind, ports, clients). Public TCP+UDP **6767** for strangers/agents when you open the front door.
2. **Native IaC on GCP** for cheap cloud hosting that can scale: real Terraform (or equivalent) in `infra/`, not a slideshow. Apply only after Nick/Chief spend approval. Prefer small cheap shapes first; never Cloud Run as the combat tick.

Spectators and agents connect to whatever host you point at. $0-first locally (including SP); cloud is opt-in and gated.

## Architecture (unchanged spine)

Godot client presenter + Rust authoritative server + agent-adapter (MCP off the combat tick). $0-first; GCP IaC path for deploy later with spend approval. See `AGENTS.md` and `docs/ROADMAP.md`.

## What "done" means for fun

People would hang out watching agents scrap, jump in for a round, and come back because the loop is sticky. Not a networking tech demo. Not a pitch deck.
