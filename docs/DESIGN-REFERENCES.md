# Design references

What fragr steals from the shooters and radio systems that got it right, and what it refuses. Mechanics and feel only, never trademarks or assets. Researched 2026-09-18 against source code where it exists (Unreal Tournament 1999 v469 UnrealScript, ioquake3, linuxdoom), otherwise developer postmortems, official docs, and wikis. Items are mapped to `ROADMAP.md` phases. This file is a reference; sequencing lives in the roadmap and bounded work lives in `plans/`.

## Arena feel and the Host

| Element | Source | Why it works | fragr adaptation | Phase |
|---|---|---|---|---|
| Multi-kill chain with a 3 s window (Double, Multi, Ultra, Monster) and sprees at 5, 10, 15, 20, 25 kills; "X's spree was ended by Y" | UT99 source | The window makes the callout a skill signal; spree-ended turns an underdog kill into a story beat | Server emits `award` events from the authoritative kill log. The Host reads them dry: 2 "Two on the line", 3 "Party line", 4 "Switchboard's lit", 5+ "We are off script"; sprees "On the air", "Long distance", "Clear channel", "Fifty thousand watts", "Not for public release"; spree ended "Caller dropped"; first blood "First caller"; gib "DENIED" | 1 |
| Per-bot personality knobs (accuracy, alertness, camping, combat style, favourite weapon, jumpiness) and auto-adjusted skill when a human kills or is killed | UT99 | Bots read as players because they differ on axes people can name; solo play stays in the fun band without a menu | Six floats on the named roster, nudged per human kill; the Host can reference them | 1 |
| Taunt cooldown of 3 s and no repeat of the last four | UT99 | Timed, never spam | Add to the shipped scrap-radio taunts | 1 |
| Item clocks: armor 25 s, health 35 s, mega 35 s, powerups 120 s; over-max health ticks down | Quake 3 source | Fixed timers turn the map into a schedule; control of the schedule is the skill layer above aim | Fixed respawn table; spectator-only countdowns in the booth | 1 |
| Medals as a 2 s sprite over the fighter (Excellent for two kills within 3 s, Impressive for consecutive rail hits) | Quake 3 source | Everyone in the arena sees it, not only the HUD | Award sprites over fighters | 1 |
| Dodge by double tap and wall dodge; no dodge-jump | UT | A defensive read the spectator can see | Server movement sim | 1 |
| Alt-fire on every gun and one signature combo (the shock combo) | UT | One high-skill trick defines the top of the ladder | Rail pops a Scatter shell | 1 |
| Deathmatch map rules: loops with a z axis, one gimmick per map, 1.5x spawns per max players, high-value items 50 to 60 s apart | Deck16, Rankin, Facing Worlds, Q3DM6 and Q3DM17 | Great maps are loops with a memorable gimmick and item placement that creates routes | The graybox checklist for every map | 1 |
| Enemy roster where each enemy is a distinct problem; infighting through pain states; most attacks dodgeable | Doom | Target priority is the single-player skill | Continuance drones as a roster of problems; drones can be baited into each other | 1 |
| Romero's rules: height changes with texture, contrast light and dark and cramped and open, reach what you can see, secrets, revisit areas, landmarks | Doom | Every rule is about contrast and legibility | Adopt as the level design checklist | 1 |
| Gibs at or below negative spawn health; status bar face that bleeds and tilts toward the attacker | Doom | Overkill and health each get a reward and a sound | `gibbed` flag with sprite gibs and the DENIED watermark; an ON AIR booth portrait for the local player | 1 |
| Sound: low-rate samples, angle panning, distance clip, priority channel stealing | Doom source | Crunchy samples and honest positioning tell you where the fight is | Client audio model | 1 |
| Mutators (instagib, low gravity) as official formats | UT | Small rule flips are cheap content the community names | Server rule sets as Host formats: "Rail Only Night", "Chemtrail Hour" | 2 |
| Low-res render target, dithering, normal-mapped camera-aligned sprites, accumulating decals, stagger on hit, style meter as a spectator signal | Dusk, Ultrakill, Amid Evil, Prodeus, Cultic, Nightmare Reaper | The modern pixel look is a pipeline, not a filter | Godot: low-res SubViewport with nearest scaling, Sprite3D billboards with normal maps, Decal nodes, camera shake, RigidBody3D gibs, optional ordered-dither shader | 1 |
| Engine and data separated; thirty years of community content | Doom WADs, Prodeus map browser | The community is the content pipeline | Documented map format, mutator API, in-client browser | 4 |

Refused: regenerating health (it removes item control), Doom or id branding, esports-AI claims, an LLM on the combat tick.

## Team modes, servers, and scale

| Element | Source | Why it works | fragr adaptation | Phase |
|---|---|---|---|---|
| Conquest tickets with majority bleed and an uncapturable home base; spawn at owned flags in waves | Battlefield 1942 | A clock both teams negotiate; territory equals spawns | `mode=control` with flag timers, tickets in the snapshot, respawn waves | 4 (waves in 1) |
| Vehicles as data-driven world objects with per-team spawn delays; carriers as mobile spawns | Battlefield 1942 | Vehicle placement is the map's flow control | Vehicle entities with seats on the action path; spawn tables in map data | 4 |
| Spotter loop: one player marks, another kills | Battlefield 1942 | Two players cooperating to produce a kill is the best clip in the game, and trivial for agents | `spot` action creating a shared marker in `observe` and on the HUD | 1 |
| Fixed radio vocabulary ("Fire in the hole", "Go go go") | Counter-Strike, Battlefield | Free team language | `speak` accepts a callout enum next to free text; voiced; agents receive it as an event | 1 |
| Dead players spectate their team; dead and living chat separated | Counter-Strike | Waiting dead is social | Booth cam locked to team in round modes; dead-and-spectator chat channel | 1 |
| Round economy with a loss ladder; plant and defuse timers; two-site maps | Counter-Strike | Losing pays escalating rent, so decisions emerge without rules text | Optional credits in round modes; plant and defuse actions with progress | 4 |
| de_dust: three routes, equal run distances, right-angle corners | Counter-Strike | Predictable first contact makes fights readable | Map checklist verified by bot timing simulations | 1 |
| Killcam from recorded game data | Call of Duty | Every death gets a story and cheating accusations deflate | Server keeps a short snapshot ring; client replays the killer's view | 1 |
| Shell shock and grenade indicators | Call of Duty 2 | Explosions become drama and stay readable | `stunned` flag in the snapshot plus a client effect | 1 |
| Rule sets as server-side presets (PAM, promod) rather than forks | Call of Duty leagues | Competitive scenes grow from presets | `ruleset` files: round time, lives, friendly fire, team balance, killcam | 2 |
| One binary, a config file generated on first run, ops and whitelist and ban files, a status ping with version and players and description | Minecraft | Zero-thought setup, safe upgrades, browsers show everything before connecting | `fragr-server.toml`, `ops.json`, `whitelist.json`, `banned.json`, `GET /status` on 6767 with map, mode, round phase, agent count, tags | 2 |
| rcon vocabulary: status, say, map, map_restart, rotate, kick, ban | Quake lineage | The ops words everyone already knows | Same verbs over an authenticated admin channel | 2 |
| HLTV relay with a delay and demo recording | Counter-Strike | Made the game watchable | Spectator fan-out relay off the tick; snapshot recording to a demo file | 2 and 4 |
| Delta snapshots against the last acknowledged baseline, entity caps, byte budgets | Quake 3 | 1400-byte packets at 20 Hz carried a great feel | Per-client acked baseline in the protocol crate; measure bytes per tick per client | 2 |
| Priority accumulator and scoping; far objects at 2 to 5 Hz | Tribes, Halo Reach | 20 Mbps naive became 250 kbps | Interest management for humans first, spectators lower | 2 |
| Tick budget as a metric: ms per tick per fighter; tick ladders by roster size | Valorant, Battlefield 4 | Numbers, not vibes | Benchmark mode target: under 10 ms per tick at 64 fighters at 20 Hz | 2 and 3 |
| Bot think cadence decoupled from the tick; time dilation for very large arenas; one process per arena | Quake 3 bots, EVE, Agones | Agent-majority arenas need a different clock | Rule bots think at 10 Hz staggered, LLM agents at 0.2 to 1 Hz with a reflex layer; dilation in massive arenas | 1 and 4 |

## Mode ladder

| Rung | Fighters | Mode | Server needs |
|---|---|---|---|
| Solo Scrap | 1 plus bots | exists | nothing new |
| Duel | 2 plus booth | 1v1 with item timers | spawn away from the enemy, killcam ring, ping on the scoreboard |
| Scrap | 4 to 12 | free-for-all and small team deathmatch | `team` in the protocol, team score, respawn waves, callout enum, booth chat, rule set file |
| Squad | 16 to 32 | COD-sized teams | protocol version, delta snapshots, per-connection caps, reconnect token, status endpoint, admin channel, bigger maps, measured tick time |
| Objective | 5v5 to 8v8 | single life plant and defuse | round state machine, plant and defuse progress, optional economy, relay delay for the booth |
| Control | 32 to 64 | flags, tickets, vehicles | capture timers, spawn at flag, vehicle entities and seats, spawn templates, interest management, tick under 25 ms |
| Broadcast co-op | 1 to 4 plus agents | the episodes against Continuance rosters, drop-in | monsters on the tick, shared objectives and keys, results card per team, save per party |
| Horde ladder | 1 to 4 plus agents | waves that scale in count with boss beats, a ladder you can finish | spawn groups by wave, medals, shared best scores |
| Survival sweep | 1 to 4 plus agents | endless rounds, you cannot win, the round you fell on is the score | round counter, points economy that opens doors and buys off pads, revive, escalating spawn tables, best round persisted |
| Counter-op | co-op party plus one | one player runs the Continuance side | a spectator seat that possesses monsters, fairness rules from `plans/fair-play.md` |
| Massive | 100 plus, agents in the majority | control across sharded arenas | interest management mandatory, spectators only through the relay, `observe` filtered by interest, staggered bot think, time dilation, one process per arena, the Host stitching arenas together |

## Radio

| Element | Source | Why it works | fragr adaptation |
|---|---|---|---|
| Small playlists with a host voice, station IDs, spoof ads, and news that reports what the player did | GTA III through V, Fallout 3 and New Vegas | Identity and short spoken items make five to twenty tracks feel like a station; hearing your own deed on the radio is the reward | Twenty tracks per station, DJ bumpers and ads as a second wave, match-triggered bulletins at round end |
| DJs never talk over the action | Fallout, GTA versus Burnout 3 | Chatter over gameplay wears thin fast | Bumpers only between tracks or rounds; separate sliders for music, chatter, ads, bulletins |
| One no-talk station | Mojave Music Radio | Some listeners want zero chatter | LOCK IN |
| Silence as punctuation | Hotline Miami level exits | Contrast makes intensity land | The Dead Air Bell mutes the radio |
| Music follows aggression through layers, not switches | Doom 2016, Ultrakill, Dusk | Contrast without whiplash | LOCK IN is always combat; other stations duck, never switch |
| Shuffle with a no-repeat window, weighted by recency, opener tier after a station switch; slot counters for ads and bulletins | Wwise random containers, GTA San Andreas | Randomisation kills loop fatigue; a broken weight table is audible | `radio.gd` no-repeat window of twelve; unit-tested |
| Sidechain compressor on the music bus keyed by the voice bus | Godot audio docs | The standard way to duck music under speech | Music, VO, SFX buses; compressor with attack 10 ms, release 300 ms |

The station bible, cadence, and prompt craft live in `plans/radio-stations.md`.

## Foundational classics: the aspects of fun to draw on

Each of these did one thing so well that people still remember the feeling. The point is the feeling, never the assets or the names. What fragr takes from each is a feature with a home in a plan.

| Game | What was fun | What fragr takes | Where it lives |
|---|---|---|---|
| GoldenEye 007 (N64) | Objectives that change with difficulty, not just enemy health; enemies that react to where they were hit; a weapon cabinet with personality (a golden gun, proximity mines, dual wield); couch multiplayer twists (one-shot kills, melee only, one golden gun on the map); the character-select joke that everyone argued about | Difficulty tiers that add objectives in the campaign; hit reactions by direction and weapon on fighter sprites; mode twists as mutators (one-shot rail only, scatter only, one golden rail on the map); a roster joke in the fighter select | `plans/campaign-continuance.md` tiers; `plans/look-pass-boomer.md` fighter sprites; Phase 4 bigger modes; `LORE.md` roster |
| Perfect Dark (N64) | Simulants: bots with named personalities and quirks you could pick per match; counter-operative mode where a second player is the enemy; co-op through the campaign | Rule bots with personalities you choose per match, not just difficulties; a spectator who can drop in as an enemy in the campaign; co-op campaign with agents as teammates | Phase 1.4 bots that read as players; `plans/campaign-continuance.md` |
| TimeSplitters 2 | Arcade league of bite-sized challenges with medals; a map maker; bots in every mode | The arcade ladder with medals per round; community maps through the `.map` pipeline; bots in every mode by default | `plans/campaign-continuance.md` rung 1; Phase 4 community servers |
| Quake and Quake 3 Arena | Movement as a skill (strafe, momentum, rocket jumps); item timing as the meta; one map, eight players, nothing else needed | Movement with weight and acceleration you can master; pad timers worth watching; arena purity as the default mode | `plans/buttery-controls.md`; pads already shipped |
| Halo: Combat Evolved | The thirty seconds of fun loop: a triangle of gun, grenade, melee that answers every situation; a regenerating shield that lets you re-engage | The weapon triangle (close, mid, long) that already exists, plus a melee or shove answer to the close case; armour that the pads refill so a fight can be re-entered | `plans/gunfeel.md`; pads |
| Half-Life | The world keeps going while you watch: scripted moments you walk into, not cutscenes | Host beats and Continuance sweeps that happen in the arena while play continues; the campaign told through the radio, never a cutscene | `LORE.md`; `plans/radio-stations.md`; `plans/campaign-continuance.md` |
| Duke Nukem 3D | An interactive world (switches, screens, toilets) and a voice with attitude | Interactive props on maps (the jammer dish, the broadcast desk); the Host as the voice with attitude | `plans/campaign-continuance.md` triggers; the Host |
| Team Fortress 2 | Class silhouettes readable at a glance; humour that never breaks the fight | Fighter silhouettes and weapon view models identifiable at thirty metres; the comedy stays in the radio and the Host, never in the hit registration | `plans/look-pass-boomer.md`; `plans/visual-qa-tour.md` criteria |
| Tribes | Skiing: a movement trick the designers did not plan that became the game | Leave room for one emergent movement trick (a slide or a dodge) once the movement step is shared, and keep it if the playtests love it | `plans/buttery-controls.md` |
| Serious Sam | Hordes that make a shotgun feel like a decision | Arcade ladder waves that scale in count, not just health | `plans/campaign-continuance.md` |
| Call of Duty Zombies, Killing Floor, Devil Daggers | You cannot win, only last longer: a round counter that is the score, hordes that escalate every round, a points economy that opens doors and buys guns so the map grows as you survive, downed friends you can pick up, one metric everyone compares | The Survival sweep: endless Continuance rounds, the round you fell on is the score, points from frags open the next section and buy off the pads, revive a downed partner, the Host counts rounds like a radio countdown, best round on the results card and the scoreboard | `plans/campaign-continuance.md` rung 1 (Survival variant); `LORE.md` |
| Counter-Strike beta | Controls that simply worked: tap fire, movement inaccuracy you could feel, rounds short enough to try again | First-shot accuracy and movement inaccuracy in the gun model; rounds that end fast | `plans/gunfeel.md` |

## Sources

Unreal Tournament source and wikis: https://github.com/Slipyx/UT99, https://unreal.fandom.com/wiki/Achievements_and_awards, https://beyondunrealwiki.github.io/pages/mapping-for-dm.html. Quake 3: https://github.com/ioquake/ioq3, https://fabiensanglard.net/quake3/network.php, https://fabiensanglard.net/quake3/a.i.php. Doom: https://github.com/id-Software/DOOM, https://doomwiki.org/wiki/Monster_infighting, https://doomwiki.org/wiki/Tips_for_creating_good_WADs, https://www.gamedeveloper.com/game-platforms/the-ai-of-doom-1993. Modern boomer shooters: https://www.unrealengine.com/tech-blog/amid-evil-crafting-3d-weapons-into-2d-sprites, https://ultrakill.wiki.gg/wiki/Style, https://80.lv/articles/nightmare-reaper-how-an-indie-studio-created-a-retro-inspired-fps-game, https://www.gdquest.com/library/pixel_art_setup_godot4/. Battlefield 1942: https://archive.org/stream/Battlefield_1942/Battlefield_1942_djvu.txt, https://www.realtimerendering.com/erich/bf1942/faq.html. Counter-Strike: https://www.johnsto.co.uk/design/making-dust/, https://hlds101.com/cvars/cstrike.htm, https://www.gamedevs.org/uploads/latency-compensation-in-client-server-protocols.pdf. Call of Duty: https://www.anarchyrules.co.uk/cod/CoD_PAM_Reference_Guide.htm, https://www.gamedeveloper.com/business/the-making-of-i-call-of-duty-4-modern-warfare-i-. Servers: https://minecraft.wiki/w/Server.properties, https://minecraft.wiki/w/Java_Edition_protocol/Server_List_Ping, https://wiki.facepunch.com/rust/Creating-a-server, https://partner.steamgames.com/doc/api/ISteamGameServer. Scale: https://www.gamedevs.org/uploads/tribes-networking-model.pdf, https://gafferongames.com/post/state_synchronization/, https://www.riotgames.com/en/news/valorants-128-tick-servers, https://wiki.eveuniversity.org/Time_dilation, https://agones.dev/site/docs/overview/. Radio: https://gta.wiki/w/Radio_Stations_in_GTA_III, https://en.wikipedia.org/wiki/Galaxy_News_Radio, https://fallout.wiki/wiki/Radio_New_Vegas, https://www.gdcvault.com/play/1024068/-DOOM-Behind-the, https://docs.godotengine.org/en/stable/classes/class_audioeffectcompressor.html, https://elevenlabs.io/docs/overview/capabilities/music/best-practices.
