# Plan: Contested Frequency radio

**Status:** in flight (2026-09-18)
**Branch:** `feat/radio-stations`
**Spend:** ElevenLabs credits from Nick's monthly plan, developer-side only. No runtime API calls. No other spend.

## Goal

An in-game radio with eight stations: seven music stations of at least twenty tracks each (two to six minutes, mostly with lyrics that live in the lore) and one spoken news station of lore bulletins. Station switching in the HUD, ducking under Host callouts and combat, and a shuffle that does not repeat. Assets are generated with `tools/audiogen`, committed with provenance, and loaded by the Godot client from the manifest.

## Non-goals

- Live text to speech or music generation at runtime.
- Adaptive stems or beat-synced transitions (later; `AudioStreamInteractive` exists for it).
- Real artist, band, brand, or politician names anywhere in prompts or lyrics.
- Voice for the Host inside the gunfight HUD (text stays primary there).

## Why this shape (research fold)

Track count is not the lever. The stations people remember shipped five to twenty tracks and won on identity, short spoken items between songs, and randomisation. News that reflects what the player just did is the emotional payoff. DJs never talk over the fight. One no-talk station is standard. Silence is a tool. Sources and the full reference table live in `docs/DESIGN-REFERENCES.md`.

## Station bible

Cadence notation: M music, DJ 5 to 12 s bumper, AD 20 to 30 s fake ad, NB 15 to 40 s bulletin, ID 3 to 6 s station ID. Music stations run about 88 percent music by time. Bumpers, ads, and bulletins are a second wave after the music lands.

| Id | Station | DJ persona | Lyric themes | Tempo and instrumentation |
|---|---|---|---|---|
| `lockin` | LOCK IN | none, a sampled count-in only | Chants only: push, frag for frag, ten in the box. No narrative lyrics. | 160 to 200 BPM; breakcore, industrial metal, speed thrash, hard techno; no calm sections, no fades |
| `rock` | Larak Lot Rock | Buzzkill, burnt-out roadie, deadpan, signs off "don't touch that dial" | LAN 1993, rage quit and respawn, skill issue confessions, rail discipline as romance, open-weights protest slogans | 110 to 170 BPM; fuzz and baritone guitars, live drums, garage, alt, thrash, stoner |
| `edm` | Signal Bleed | Barium Sky, clawbot DJ, speaks in BPM and tick rates, polite about the uprising it is not planning | Spawn weather digits as mantras, tick rate as heartbeat, "shall not be infringed", callers who cannot stop pressing join | 128 to 174 BPM; techno, breaks, drum and bass, acid, industrial electro |
| `hiphop` | The Lot | Mega Colony, calm braggadocio, ant-war metaphors | Hangar Candy stamps, scout ant tactics, the ticket economy, "not for public release" punchlines, the Host selling gold | 88 to 140 BPM; boom bap, G-funk, trap, drill hats, scratch hooks |
| `chill` | Dead Air | Dead Air Dan, whisper-quiet, long pauses, reads weather digits like poetry | Waiting for the bell, the booth at night, the pipe that stayed open, counting frags like sheep | 60 to 90 BPM; Rhodes, dub bass, pedal steel, trip-hop kits, tape hiss |
| `country` | Gulf Breeze Country | Aunt Linda, sweet, gossipy, sells fish oil, blesses meatbags and clawbots equally | Diego Far, a base that does not exist, sniped in his own house, coordinates instead of her name, tent revival for the LAN faithful | 70 to 130 BPM; Telecaster, pedal steel, banjo, fiddle, upright bass |
| `world` | Leyline World Service | Vacuum John, globetrotting, reads numbers in four languages | "Peace without interruption, please hold", the Walmart Leyline triangle, spawn weather as a wedding toast | 96 to 126 BPM; cumbia, afrobeat, dub, Balkan brass, electro-swing |
| `news` | Contested Frequency News | The Host (straight man), Tin Foil Tina (field), Fluoride Phil (science desk) | Spoken only: generic lore bulletins, PSAs, ads, callers, numbers weather, match-triggered bulletins | Beds and stings only: 12 s sting, 90 s loopable underscore, 10 s sung ID |

Running gags available to every station: the Continuance motto "peace without interruption", the Dead Air Bell, spawn weather digits, Hangar Candy stamps, the 67 ritual, skill issue, scout ant deleted, Perim Ghost and East-West Pipe and Diego Far and Larak Lot, clawdbots and meatbags in the same fight, Hermes and Pi and L5 as agent names people gossip about, and the Host's gold and water filters. Humor is self-deprecating boomer-shooter nostalgia and AI-takeover satire where the machines are very polite about it.

## Prompt craft (what the API rewards)

Every prompt states genre, mood, instrumentation, tempo and key, and production era; names the vocal delivery; narrates the arrangement in order; carries one hook line in quotes and one paragraph of lyric direction naming a character, a map, one running gag, and three concrete objects. Sixty to one hundred twenty words. Instrumentals say so explicitly. No real artist or brand names. Keep chorus lines to six to nine syllables with open vowels so the vocal stays intelligible. Generate takes, keep the best.

## Asset layout and provenance

- Files: `client/assets/audio/radio/<station>/<NN>-<slug>.mp3` at `mp3_44100_128`. Sung IDs and stings as `.mp3`, spoken bulletins as `.mp3`.
- Provenance: `client/assets/audio/audiogen-manifest.json` (prompt, model, format, length, title, generation time) written by the tool. That is the sidecar metadata.
- Specs: `tools/audiogen/specs/radio-<station>.json`, one per station, twenty items each with `title`, `length_range_ms` of `[120000, 360000]`, and `instrumental` on about a quarter of the tracks (Lock In may run half).
- Licensing: assets ship under the repository's Apache 2.0 license with the manifest as the record of how they were made. Prompts never name real artists.

## Budget and waves

Music costs about 900 credits per minute. One hundred forty tracks averaging four minutes is about 504,000 credits, which is a full month of a 500,000-credit plan, so the library lands in waves and the tool refuses runs that exceed `--max-credits`:

1. Wave 1: Lock In, Rock, and EDM, ten tracks each (about 108,000 credits). Verify the six-minute end of the range works in prompt mode; fall back to five minutes if the API caps prompt-mode length.
2. Wave 2: the remaining four music stations, ten each.
3. Wave 3: fill every station to twenty.
4. Wave 4: news station beds, stings, and spoken bulletins; DJ bumpers and ads for the music stations.

`fragr-audiogen batch --spec tools/audiogen/specs/radio-rock.json --limit 10 --max-credits 40000` is one wave step. Re-running continues where it stopped because existing files are skipped.

## Client behaviour

- `client/scripts/radio.gd` builds the station list from `client/assets/audio/radio/stations.json` and the manifest entries whose name starts with `radio/<station>/`. No directory scanning, so exports work.
- Shuffle: no repeat within the last twelve plays (or the pool size minus one when smaller), a fresh random seed per session, play history kept per station for the session.
- Keys: `radio_next_station` (R), `radio_next_track` (N), `radio_toggle` (M). Gamepad: D-pad up, down, and left.
- Volume states: spectating 0 dB relative; playing as human minus 6 dB; Host callouts duck a further 9 dB for three seconds with a short recovery; LOCK IN never ducks for combat. The Dead Air Bell mutes the radio for its duration (round-end wiring in a follow-up).
- HUD: a small label bottom right shows the station name on switch and the track title on each new track, then fades. Never in the killfeed corner.
- Missing assets are fine: with no tracks the radio stays silent and the HUD says the station is off the air.

## Verification

- `cargo test -p fragr-audiogen` covers spec parsing, length ranges, credit estimates, prefix and limit waves, and the manifest.
- `godot --headless --path client --check-only --script res://scripts/radio.gd` and the harness `res://scripts/test_radio.gd` (station grouping, shuffle no-repeat, station cycling, volume state table).
- A recorded Solo Scrap session with the radio audible, station switches, and a Host callout ducking the music.
- Every generated file has a manifest entry; the client loads by manifest, never by directory listing.

## Success criteria

- [ ] Seven music stations with at least twenty tracks each, two to six minutes, mostly with lyrics on theme, committed with provenance.
- [ ] News station with a sting, an underscore bed, a sung ID, and at least twenty spoken bulletins in the five classes.
- [ ] Radio plays in Solo Scrap and spectate, switches stations and tracks with the keys, ducks under Host lines, and never repeats within twelve plays.
- [ ] Docs updated: `client/assets/audio/README.md`, `README.md` controls, this plan marked shipped, roadmap item ticked.
