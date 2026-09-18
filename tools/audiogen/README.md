# fragr-audiogen

Developer-only tool that produces sound effects and music beds for fragr through the ElevenLabs HTTP API and writes them as ordinary assets under `client/assets/audio/`. Players never run it. CI never runs it. The game never calls the API.

Crate: `tools/audiogen` (`fragr-audiogen`). Rust only, one HTTP dependency, tested without the network.

## Put in the key

Create an API key in the ElevenLabs dashboard, then pick one of these. Never commit the key.

PowerShell (current session only):

```powershell
$env:ELEVENLABS_API_KEY = "your-key"
```

Bash:

```bash
export ELEVENLABS_API_KEY="your-key"
```

Or put it in a `.env` file at the repository root. The tool reads `.env` automatically when the environment variable is unset and accepts `ELEVENLABS_API_KEY=...` or the shorter `elevenlabs=...`:

```text
elevenlabs=your-key
```

Or keep it in a file under the gitignored agent scratch directory and point the tool at it:

```bash
mkdir -p .agents && printf '%s' "your-key" > .agents/elevenlabs.key
cargo run -p fragr-audiogen -- --api-key-file .agents/elevenlabs.key quota
```

`.agents/`, `.env`, and `*.key` are gitignored. The tool never prints the key.

## Commands

Check remaining credits:

```bash
cargo run -p fragr-audiogen -- quota
```

One sound effect (24 kHz mono WAV by default, which Godot imports as a sample):

```bash
cargo run -p fragr-audiogen -- sfx --name fire_rail \
  --prompt "heavy railgun shot, electric charge crack then cold metallic ring, retro arcade, dry" \
  --seconds 0.9 --influence 0.6
```

One music bed (MP3 at 44.1 kHz and 128 kbps, which Godot streams):

```bash
cargo run -p fragr-audiogen -- music --name music/match_01 \
  --prompt "driving industrial synth-metal loop for a 90s arena shooter, 140 bpm, no vocals" \
  --length-ms 60000 --instrumental
```

Spoken lines for the news station and the Host (v3 model, delivery tags such as `[sighs]` work):

```bash
cargo run -p fragr-audiogen -- voices
cargo run -p fragr-audiogen -- tts --name radio/news/generic-01-count --voice <voice_id> \
  --text "[sighs] Good evening. The Continuance lost count again." --stability 0.5
```

`voices` prints the account's default voices with ids for casting. Default voices retire at the end of 2026, so cast from that list rather than from ids copied out of old docs. Spec items of kind `tts` take `text`, `voice`, optional `model`, `stability`, `format`, `title`. The scripts for the news station live in `specs/radio-news-scripts.json` and become `tts` items once voices are cast.

Everything in a spec file (see `specs/sfx-core.json`):

```bash
cargo run -p fragr-audiogen -- batch --spec tools/audiogen/specs/sfx-core.json
cargo run -p fragr-audiogen -- batch --spec tools/audiogen/specs/sfx-core.json --only frag
```

Useful flags on every command: `--dry-run` prints the exact request and the credit estimate and writes nothing, `--overwrite` replaces existing files (the default is to skip them), `--out-dir` changes the destination, `--max-credits N` refuses to start a run whose estimate exceeds N.

Batch runs work in waves: `--prefix radio/rock/` selects a folder, `--limit 5` generates at most five new files, and files that already exist are skipped, so re-running the same spec continues where it stopped. Every batch prints `batch: K to generate, ~N credits estimated` before it calls the API.

Spec items may carry a `title` (recorded in the manifest and shown by the in-game radio) and, for music, `length_range_ms: [lo, hi]` instead of `length_ms`; the tool then picks a fixed length per track name inside the range, so a spec of 2 to 6 minute tracks stays varied and reproducible.

## What it writes

- The audio file at `<out-dir>/<name>.<ext>`. `pcm_*` formats become `.wav`, `mp3_*` become `.mp3`.
- `<out-dir>/audiogen-manifest.json`, one entry per generated name with the prompt, model, format, duration, channel count, byte size, and generation time. That is how anyone regenerates or audits an asset later.

Names are lowercase with `_`, `-`, and `/` for folders, no extension. They should match what the client loads, for example `fire_rail`, `hit_scatter`, `frag`, `round_start`, or `music/match_01`.

## Formats and tiers

- Sound effects default to `pcm_24000`, available on every paid tier. `pcm_44100` needs a higher tier. MP3 is available everywhere.
- Music defaults to `mp3_44100_128`. Higher bitrates exist on higher tiers.
- The API does not document the channel count of PCM output. Measured on 2026-09-18 it is stereo, and the MP3 variant carries a stereo frame header. When you pass `--seconds`, the tool infers mono or stereo from the byte count and records it in the manifest. Without a duration, mono is assumed, so pass `--seconds` for effects.
- Sound effects run 0.5 to 30 seconds. Music runs 3 seconds to 10 minutes. `--loop` asks for a seamless loop on sound effects.

## Cost and limits

Sound effects and music cost credits per second of audio. Music is only enabled on paid plans. The API rate limits concurrent requests per tier; the tool retries on 429 and 5xx with backoff and stops on any other error. Run `quota` before a big batch.

## Licensing of the output

Generated files are your assets under the ElevenLabs terms for your plan. Paid plans allow commercial use for a project like this; a studio-scale commercial release has extra terms. Keep the manifest so the provenance of every file is clear. The procedural CC0 files produced by the older generator are unaffected.
