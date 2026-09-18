# Plan: Solo Broadcast campaign (Episode 0 face)

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `feat/solo-broadcast-ep0`
**Spend:** $0. Loopback. No GCP apply, no ElevenLabs, no look_at reopen, no Doom IP.
**Status:** In flight. Tip base `6881009` (after brain third-tier #103).

## Goal

Ship **Contested Frequency Solo Broadcast** as a first-class lore campaign path: the meatbag soft-joins the Host during a Continuance compliance sweep. Not deathmatch-vs-bots. Same Action path / gunfeel as MP. Offline-capable.

First playable slice: **Episode 0 face** (title card, Host cold open, objective chip, win/fail beat) that feels like a solo game.

## Product lock (Solo Broadcast)

| Lock | Meaning |
|---|---|
| Solo Broadcast | Episodic Continuance sweep campaign. Host is unreliable co-op booth voice. |
| Foes | NODS (Null-Objective Drones), Auditors / Continuance elites, occasional Level 5 rival or ally (ambiguity; no Kilo; no AGI theater). |
| Same scrap | Same guns, maps, and Action path as MP. |
| Win condition (season) | Unmetered one more night. Continuance regrows. Rematch is the point. |
| Not | Bots-in-DM, full 13-episode campaign, cutscenes, GCP apply, look_at reopen, Doom IP. |

## Campaign spine (plan of record; Ep0 ships only)

1. **Ep0 Calibration** (this PR) cold-open on **Larak Lot**
2. Area Kitchen
3. East-West Pipe
4. Perim Ghost
5. Diego Far
6. Weird mid (Hangar Candy / Chemtrail / weather dial)
7. Forever Office annex finale (stop L5 to NOD conversion)

Article curses stay jokes. Level 5s are aspiration graffiti / optional ally-rival, not a ship promise.

## Insanely-fun bar (this PR)

Stranger smiles in 60s playing or watching, or not done.

**Ship order in this PR (juice spike only):**
1. Episode 0 face on one map (Larak Lot) with maxed Host / objective / frag feedback chrome
2. NODS vs Auditor silhouette/behavior split if cheap (NODS stiff labels; Auditor elite)
3. Time-to-first-frag under ~30s (short Warmup on Solo Broadcast)
4. Fail/win Host lines from Fringy

**HOLD this PR:** killcam, more maps, Continuance Sweep events, agent tickets, cutscenes.

## Episode 0 face (Fringy spec; ship this)

**Title:** Solo Broadcast: Calibration (first Contested Frequency campaign episode)

**Map face:** Larak Lot (floodlight parking). Geometry reuses map 1 or 2; wire/HUD `map_name` / episode chrome sell **Larak Lot**.

**Objective:** Survive Continuance probe vans; clear NODS; soft-touch a jammer dish so Host stays on air.

**Foes:** NODS only + one Auditor elite at end (clipboard shield chrome).

**Host lines (exact text):**
1. In the morning. Calibration night. Continuance brought NODS. You're on the air.
2. Null-Objective Drones don't trash-talk. That's how you know they're approved.
3. Jammer's up. Seize the dish or I go text-only. Value for value.
4. Amen, fistbump. Frequency still unmetered. Don't touch that dial.

**Fail:** Continuance compliance splash: "Citizen Handle assigned." Reload.

**Win:** Unlock callsign stub + teaser for Area Kitchen.

## Architecture impact

| Area | Change |
|---|---|
| `server` CLI | `--solo-broadcast` enables Episode 0 path (MP unchanged when off). |
| `server` protocol | Optional Snapshot episode fields + `episode_start` / `episode_complete` / `episode_fail` events. |
| `server` sim | Solo Broadcast: NODS roster labels, jammer dish pad claim, Auditor elite (Compliance boss retitled), objective progress, win/fail Host lines. |
| `agent-adapter` | Mirror optional episode fields / events. |
| `client` | Unmissable Solo Broadcast chrome: title card, Host cold-open bumper, objective chip; ghost rival sticky name OK. |
| `tools/solo_scrap.sh` | Pass `--solo-broadcast` by default for Solo Broadcast boot. |
| `docs` | This plan; tip priorities NOW; VISION/LORE already seasoned in #87. |

## Wire (summary)

Snapshot (omitted when not Solo Broadcast):

- `episode_id`: `"ep0"`
- `episode_title`: `"Solo Broadcast: Calibration"`
- `episode_objective`: short objective chip text
- `episode_progress`: e.g. `NODS 2/5 | JAMMER | AUDITOR`
- `episode_phase`: `nods` / `jammer` / `auditor` / `won` / `failed`

Events:

- `episode_start` { id, title, objective, host_line, map_name }
- `episode_complete` { id, reason, host_line, unlock_teaser }
- `episode_fail` { id, reason, host_line }

## Beat (shippable)

1. Warmup / Solo boot: title card + Host line 1. Map face **Larak Lot**.
2. Active: rule bots labeled NODS. Objective chip tracks clear NODS (human frags toward goal, default 5).
3. After NODS goal (or mid-beat): jammer dish pad becomes the seize target; Host line 3.
4. Soft-touch jammer (walk claim) + NODS cleared: spawn **Auditor** (Compliance Drone retitled, clipboard shield Host line).
5. Frag Auditor: `episode_complete`, Host line 4, Area Kitchen teaser, end round.
6. Time-out without complete: `episode_fail`, "Citizen Handle assigned."

MP path: flag off; no episode fields; existing Contested Frequency DM unchanged.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
./tools/solo_scrap.sh
```

## Spend / safety

$0. No secrets. No attribution, emoji, or em/en dashes in commits, PR text, or docs. Nick Seal / blisspixel only.

## Success criteria

- [x] Plan in tree; tip priorities NOW = Solo Broadcast Episode 0 face
- [x] Solo boot shows episode title card + Host cold open + objective chip
- [x] Server `--solo-broadcast` supports NODS clear + jammer + Auditor without breaking MP
- [x] Win/fail Host lines match Fringy spec; Area Kitchen teaser on win
- [ ] Tests + coverage fail-under 80 if Rust touched
- [ ] PR open on `feat/solo-broadcast-ep0`

## Tip priorities (this slice)

1. **Solo Broadcast Episode 0** (this plan) - Calibration face on Larak Lot
2. HOLD: full campaign maps, cutscenes, AGI theater, GCP apply, look_at reopen
