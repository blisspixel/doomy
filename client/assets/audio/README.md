# Audio Assets

All audio files in this directory are procedurally generated in-repo using `tools/generate_audio.py`.

## License

**CC0 1.0 Universal (Public Domain)**

All audio files are released into the public domain via CC0 1.0 Universal.

To the extent possible under law, the author(s) have dedicated all copyright and related rights to these audio files to the public domain worldwide. These files are distributed without any warranty.

Full license: https://creativecommons.org/publicdomain/zero/1.0/

## Files

- `fire.wav` - Weapon fire sound (square wave + noise + high sine, 0.08s)
- `hit.wav` - Hit confirmation sound (noise impact + sine ping, 0.06s)
- `frag.wav` - Frag/elimination sound (low thump + crunch + sparkle, 0.20s)
- `round_start.wav` - Round start cue (two ascending beeps, 0.18s)
- `round_end.wav` - Round end sound (descending tone sequence, 0.36s)

## Generation

To regenerate these audio files:

```bash
cd tools
python3 generate_audio.py
```

No external dependencies required beyond Python 3 standard library (wave, struct, math, random).

## Technical Details

- Sample rate: 22050 Hz (mono)
- Format: 16-bit PCM WAV
- Total size: ~40 KB
- Synthesis: Pure procedural (sine/square waves, white noise, ADSR envelopes)
