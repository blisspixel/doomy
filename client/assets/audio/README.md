# Audio assets

Two pipelines feed this directory. Every file is an ordinary asset the client loads by name; nothing here calls a network at runtime.

## Generated with `fragr-audiogen` (ElevenLabs, developer-only)

Sound effects and music produced by `tools/audiogen` (see `tools/audiogen/README.md`). Provenance for each generated file lives in `audiogen-manifest.json` next to it: prompt, model, format, duration, channel count, byte size, generation time. Regenerate with the batch specs under `tools/audiogen/specs/`.

Generated files are owned by the project under the ElevenLabs terms for the account that produced them and are distributed with the repository under its Apache 2.0 license. Sound effects are stereo 24 kHz 16-bit WAV by default (the API returns stereo PCM). Music is 44.1 kHz MP3.

## Procedural fallback (CC0)

The original effect set was synthesised with `tools/generate_audio.py` (sine, square, noise, envelopes) and is dedicated to the public domain under CC0 1.0 Universal: https://creativecommons.org/publicdomain/zero/1.0/. Any file not listed in `audiogen-manifest.json` came from that generator. A Rust port of the generator is planned so the tree stays Rust and GDScript only.

## Files the client loads

| File | Used for |
|---|---|
| `fire.wav`, `hit.wav` | Fallback weapon fire and hit confirm |
| `fire_flechette.wav`, `fire_rail.wav`, `fire_scatter.wav` | Per-weapon fire |
| `hit_flechette.wav`, `hit_rail.wav`, `hit_scatter.wav` | Per-weapon hit |
| `frag.wav` | Elimination stinger |
| `round_start.wav`, `round_end.wav` | Round cues |
| `music/` | Music beds and radio stations (planned wiring, see `docs/ROADMAP.md`) |

Loading paths: `client/scripts/player_pawn.gd` (per-weapon fire and hit), `client/scripts/game_manager.gd` (frag and round cues). Import presets: keep WAV as samples, MP3 as streams, loop flags off unless the manifest marks a file as looping.
