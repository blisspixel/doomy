# Audio assets

Two pipelines feed this directory. Every file is an ordinary asset the client loads by name; nothing here calls a network at runtime.

## Generated with `fragr-audiogen` (ElevenLabs, developer-only)

Sound effects and music produced by `tools/audiogen` (see `tools/audiogen/README.md`). Provenance for each generated file lives in `audiogen-manifest.json` next to it: prompt, model, format, duration, channel count, byte size, generation time. Regenerate with the batch specs under `tools/audiogen/specs/`.

Generated files are owned by the project under the ElevenLabs terms for the account that produced them and are distributed with the repository under its Apache 2.0 license. Sound effects are stereo 24 kHz 16-bit WAV by default (the API returns stereo PCM). Music and the spoken news station are 44.1 kHz MP3.

## Earlier procedural set (retired)

The first effect set was synthesised procedurally and dedicated to the public domain under CC0 1.0 Universal. It has been replaced by generated effects with manifest entries; the old files remain in git history. Any file not listed in `audiogen-manifest.json` came from that earlier set.

## Files the client loads

| File | Used for |
|---|---|
| `fire.wav`, `hit.wav` | Fallback weapon fire and hit confirm |
| `fire_flechette.wav`, `fire_rail.wav`, `fire_scatter.wav` | Per-weapon fire |
| `hit_flechette.wav`, `hit_rail.wav`, `hit_scatter.wav` | Per-weapon hit |
| `frag.wav` | Elimination stinger |
| `round_start.wav`, `round_end.wav` | Round cues |
| `radio/<station>/` | Contested Frequency radio tracks, grouped by station id; `radio/stations.json` names the stations |

Loading paths: `client/scripts/player_pawn.gd` (per-weapon fire and hit), `client/scripts/game_manager.gd` (frag and round cues), `client/scripts/radio.gd` (radio tracks, discovered through the manifest, never by directory listing). Import presets: keep WAV as samples, MP3 as streams, loop flags off unless the manifest marks a file as looping.

Radio controls in the match: R next station, N next track, M radio on or off (D-pad up, down, left on a gamepad). Every switch shows a station card (badge, name, tagline) above the track toast. The radio ducks under Host lines and sits lower while you play; LOCK IN never ducks for combat.
