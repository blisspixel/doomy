# Plan: ElevenLabs Audio Drama (Deferred)

**Status:** Deferred until Nick approves spend.  
**Gate:** Requires Nick approval before calling ElevenLabs API or spending money.  
**Current:** CC0 procedural audio stays default.

---

## Current Audio (Shipped)

CC0 1.0 Universal procedural audio (Python-generated, no attribution required):
- `fire.wav` - weapon fire
- `hit.wav` - damage impact
- `frag.wav` - kill confirmation
- `round_start.wav` - round begin
- `round_end.wav` - round end

Generated via `tools/generate_audio.py` using NumPy/SciPy. Free, open, no spend.

---

## Future Audio Drama (When Approved)

**ElevenLabs API** could add:
- **Frag stingers:** Bot-specific callouts on kills ("Rusher eliminated Sniper")
- **Bot personality VO:** Taunts, reactions, tactical calls
- **Round announcer:** Professional sports-style commentary
- **Ambient stingers:** Tension music, victory fanfares
- **Host voice:** Conspiracy-weird Ren/Stimpy energy, meme integration (67 ritual, skill issue)
- **Dead Air Bell:** Fake AM radio ads, station breaks
- **Fashion faction callouts:** Team-specific branding audio

**Workflow (when approved):**
1. Write script text for each audio clip
2. Call ElevenLabs API with approved voice models
3. Download generated audio
4. Commit to repo as `.wav` or `.ogg`
5. Wire into appropriate game events

**Cost estimate:** ~$0.01-0.05 per clip, $5-10 for full drama pack (100-200 clips).

**Approval gate:** Nick must explicitly approve before making API calls or spending.

---

## Implementation Notes (For Later)

- Keep CC0 procedural as fallback
- Add audio config: `audio.drama_mode` (off, procedural, elevenlabs)
- Layer VO over base sound effects (fire/hit still play)
- Volume mix: VO at 80%, base SFX at 100%
- Random variation: 3-5 variants per event for freshness

---

## Non-Goals

- No live TTS during gameplay (pre-generated only)
- No music licensing (procedural or CC0 only)
- No voice actor hiring (API-generated only when approved)

---

**Summary:** Audio drama is a polish pass for later. Current CC0 procedural audio is sufficient for gameplay. Nick gates any spend.
