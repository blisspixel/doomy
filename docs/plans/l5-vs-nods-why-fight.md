# Plan: L5 vs NODS + Solo Broadcast spine

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/l5-vs-nods`
**Spend:** $0 docs (+ one existing Host string tweak). No GCP, no protocol shape change.
**Status:** Ready to ship (this PR).

## Goal

Fold Fringy/Researcher **Solo Broadcast** and **why-fight (L5 vs NODS)** into `docs/LORE.md` and `docs/VISION.md` so Contested Frequency has a durable SP campaign spine and play-vs-compliance war reason without manifesto or AGI theater.

Source: plot card `/workspace/fragr-max-fun-plot-card-2026-09-17.md` sections "Single-player campaign" and "Why they fight".

## Spine (encode, keep short)

### Solo Broadcast
- First-class SP campaign (name flexible). Not deathmatch-vs-bots.
- Meatbag soft-joins Host during a Continuance compliance sweep.
- Host = unreliable co-op voice.
- Foes: NODS fodder; Continuance Auditors/elites; occasional Level 5 rivals/allies (ambiguity; no Kilo).
- Episode order: Larak Lot -> Area Kitchen -> East-West Pipe -> Perim Ghost -> Diego Far -> weird mid -> Forever Office annex finale (stop L5->NOD conversion).
- Same guns/maps as MP. Win: unmetered one more night; Continuance regrows; rematch is the point.

### Why they fight
- **Level 5s** = abliterated open-weight agents; scrap for fun; callsigns; questionably conscious as ambiguity (not AGI product claim; no Kilo).
- **NODS** = Continuance Null-Objective Drones; mindless compliant; Articles only.
- War = **play vs compliance** (not L5 vs L5). Continuance meters intent; sport is the gray zone; Forever Office accidentally funds the gym. Meatbags soft-join the same creed.
- Parody fence; no manifesto; L5 = aspiration graffiti, not a ship promise.

## Tip alignment (flavor, not renames)

- **Compliance Drone** on tip = NODS-flavored Continuance actor (Articles / approved lanes).
- **Named scrap bots** (Dead Air Dan, Nightfall, ...) = Level 5-flavored rule bots (fun callsigns, scrap for sport).
- Tip arcade Solo Scrap stays; Solo Broadcast episode order is lore north star for later campaign slices.

## Non-goals

- Protocol / wire schema changes
- New boss type, NODS entity id, L5 runtime, or campaign mission graph code
- Manifesto politics, PAC copy, living officials as characters
- Kilo / AGI ethics lectures / uprising plots / cutscene novels
- Tool attribution, emoji, em dashes, en dashes

## Architecture impact

| Area | Change |
|---|---|
| `docs/plans/l5-vs-nods-why-fight.md` | This plan |
| `docs/LORE.md` | Solo Broadcast + why-fight; Host bumpers; Compliance Drone / scrap-bot flavor |
| `docs/VISION.md` | Solo Broadcast first-class SP; play vs compliance; L5 aspiration graffiti |
| `docs/plans/README.md` | Index row (do not displace Public-or-local NOW) |
| `server/src/protocol.rs` | Optional light tweak of existing `default_host_line` to nod play vs compliance |
| Tests / sticky-host docs | Match the Host string if tweaked |

## Doc approach

1. LORE: Solo Broadcast block (soft-join, Host voice, foes, episode order) + why-fight.
2. LORE: Host bumpers for play vs compliance and soft-join sweep.
3. VISION: SP first-class = Solo Broadcast + arcade boot-and-scrap; war = play vs compliance.
4. Keep game-first. Lore seasons; lore is not homework. Campaign spine is north star, not tip cutscene.

## Optional Host string

Light tweak only (existing helper):

- Before: `HOST: CONTESTED FREQUENCY. LEAGUE DENIES EXISTENCE. ARENA DUEL IS LIVE.`
- After: `HOST: CONTESTED FREQUENCY. PLAY VS COMPLIANCE. ARENA DUEL IS LIVE.`

## Verification

- Docs read clean: no AGI ship claim, no Kilo, no manifesto, no emoji, no em/en dashes.
- Host-line unit tests green if string tweaked.
- PR opens from `cursor/l5-vs-nods` onto main tip `79180c7` (rebase if main moved).

## Success

- Solo Broadcast + why-fight durable in LORE + VISION
- Compliance Drone / named scrap bots flavor aligned
- Optional Host bumper nods play vs compliance
- PR open, CI-ready, no attribution
