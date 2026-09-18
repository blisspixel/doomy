# Plan: Audio and Match Drama Pass

**Repo:** https://github.com/blisspixel/fragr  
**Branch:** `cursor/audio-drama-pass-a3c8`  
**Spend:** $0. CC0 or procedural audio only. No paid assets.  
**Status:** Plan of record for exceptional audio feedback.

## Goal

Make fights feel better with free/local audio only. Give spectators and players instant auditory feedback for key combat events: weapon fire, hits, frags, and round transitions. Ship with only CC0-licensed or procedurally generated sounds (no unclear licenses, no paid assets).

## Non-goals

- Music or ambient soundscapes
- Advanced 3D spatial audio (stereo panning only if trivial)
- Per-weapon audio variety beyond basic fire sound
- Voice lines or character-specific sounds
- Paid asset packs or unclear-license samples

## Audio events (priority order)

1. **Weapon fire** (high priority): short, punchy sound when any fighter shoots
2. **Hit confirmation** (high priority): distinct impact sound when shots connect
3. **Frag** (high priority): satisfying elimination sound when a fighter is fragged
4. **Round start** (medium priority): brief audio cue when a new round begins
5. **Round end** (medium priority): completion sound when frag limit is reached

Optional if time/simple: camera shake on frag (may already exist from polish pass).

## Audio sourcing strategy

**Prefer procedural generation first, CC0 library second:**

1. **Procedural (in-repo WAV generation):** Use GDScript `AudioStreamGenerator` or a tiny tool script to synthesize simple waveforms (sine/square/noise bursts) and export to WAV. This gives us complete control, zero licensing ambiguity, and tiny file sizes.

2. **CC0 library fallback:** If procedural is too time-expensive, use sources like:
   - [freesound.org](https://freesound.org) (filter by CC0 1.0 Universal license)
   - [opengameart.org](https://opengameart.org) (CC0 section)
   - [sonniss.com GDC bundles](https://sonniss.com/gameaudiogdc) (check license per-file)

**License documentation:** All audio files must have clear CC0 attribution or "procedurally generated in-repo" notes in `client/assets/audio/README.md` and brief mention in root `README.md`.

## Implementation plan

### Audio files (client/assets/audio/)

Create directory `client/assets/audio/` with:

- `fire.wav` - weapon fire sound (target: <50 KB, 0.1-0.2s duration)
- `hit.wav` - hit confirmation (target: <30 KB, 0.05-0.1s duration)
- `frag.wav` - frag/elimination (target: <50 KB, 0.2-0.3s duration)
- `round_start.wav` - round begin cue (target: <30 KB, 0.1-0.2s duration)
- `round_end.wav` - round complete (target: <50 KB, 0.2-0.3s duration)
- `README.md` - license attribution for each file

Total target: <250 KB for all audio assets.

### Godot integration (client/scripts/)

**Use AudioStreamPlayer nodes (2D or non-spatial):**

- Add AudioStreamPlayer nodes to appropriate scenes (likely `arena.tscn` or `main.tscn`)
- Load AudioStream resources for each event type
- Trigger `.play()` on network events from server snapshots or local actions
- Set default volume to reasonable levels (suggest -10 to -15 dB to avoid clipping)
- Handle missing streams gracefully (check `if audio_stream != null` before play)

**Event wiring:**

1. **Fire:** Play when local player shoots (immediate) OR when snapshot shows `just_fired` flag on remote fighters
2. **Hit:** Play when snapshot includes hit events or local player receives damage
3. **Frag:** Play when killfeed updates with new frag (game_manager.gd likely handles this)
4. **Round start/end:** Play on round lifecycle events (likely in game_manager.gd round state machine)

### Camera shake (optional)

If `spectator_cam.gd` or `player_pawn.gd` does not already have camera shake on frag, add a simple trauma-based shake (0.2-0.5s duration, 2-5 pixel random offset) triggered by frag events. Only ship if trivial to implement (<30 minutes).

## Verification

Before done:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

Manual smoke test:

1. `cargo run -p doomy-server -- --bots 4`
2. Open `client/` in Godot 4.7.2, press F5
3. Confirm audible feedback for:
   - Weapon fire (immediate and frequent)
   - Hit confirmation (when shots connect)
   - Frag sound (when killfeed updates)
   - Round start (at match begin or after round ends)
   - Round end (when frag limit reached)
4. Test with volume muted and missing audio files (should not crash or spam errors)

## Architecture impact

- New directory: `client/assets/audio/` with 5 WAV files + README
- Modified scripts: `game_manager.gd`, `player_pawn.gd`, and/or `spectator_cam.gd` to wire audio events
- Optional modified: `hud.gd` if round state events are cleanest there
- No server changes required (audio is client-side presentation only)
- No protocol changes required (existing snapshot events sufficient)

## Success criteria

1. Five audio events working: fire, hit, frag, round start, round end
2. All audio files CC0-licensed or procedurally generated, documented in `client/assets/audio/README.md`
3. Brief license mention added to root `README.md` (one line in architecture or quick start section)
4. Volume defaults are sane (not clipping, not inaudible)
5. Client handles missing audio streams without crashing
6. Total audio assets <250 KB
7. Spend remains $0
8. No emoji, no tool attribution, no em/en dashes in any new prose or commits

## Build order

1. Create `client/assets/audio/` directory
2. Generate or source 5 audio files (CC0 only)
3. Document licenses in `client/assets/audio/README.md`
4. Add AudioStreamPlayer nodes to Godot scenes
5. Wire fire, hit, frag, round start, round end events in scripts
6. Test audio playback in spectator and player modes
7. Update root `README.md` with audio license note
8. Full verification suite above
9. Optional: add camera shake on frag if trivial

## Spend / safety

$0 only. Fail closed on any paid or unclear-license audio. CC0 1.0 Universal or in-repo procedural generation only.
