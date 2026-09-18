# Audio Assets

All audio files in this directory are procedurally generated in-repo using `tools/generate_audio.py`.

## License

**CC0 1.0 Universal (Public Domain)**

All audio files are released into the public domain via CC0 1.0 Universal.

To the extent possible under law, the author(s) have dedicated all copyright and related rights to these audio files to the public domain worldwide. These files are distributed without any warranty.

Full license: https://creativecommons.org/publicdomain/zero/1.0/

## Files

- `fire.wav` - Weapon fire sound (kick + snap + crack, snappy arcade punch, 0.06s)
- `hit.wav` - Hit confirmation sound (thwack + ding + thump, satisfying feedback, 0.08s)
- `frag.wav` - Frag/elimination sound (massive bass + explosion + rising sweep + sparkle cascade, sells the moment, 0.30s)
- `round_start.wav` - Round start cue (charge-up + impact beep + punch, arcade excitement, 0.20s)
- `round_end.wav` - Round end sound (victorious chord + bass thump, dramatic fanfare, 0.35s)

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
- Total size: ~45 KB
- Synthesis: Pure procedural (sine/square waves, white noise, frequency sweeps, chord synthesis, ADSR envelopes)
- Design: Snappy arcade feel, not subtle corporate beeps
