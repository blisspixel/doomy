# Plan: Unmissable Warmup Contested Frequency TV bumper

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/warmup-tv-bumper`
**Spend:** $0. Loopback. No GCP apply, no ElevenLabs, no look_at/hit reopen.
**Status:** In flight (NOW tip after named+Warmup Host drama + public-or-local #84 / tip `79180c7`).

## Goal

Raise Warmup so Godot Contested Frequency scrap open is NOT blink-and-miss. Spectator feels Among Us / CS beta / Vampire Survivors / 1990 online-Doom energy:

- Big map title
- Roster callsigns visible as chips
- Giant GOES LIVE IN N countdown
- Full-frame Host flash (+1s linger into Active if needed)

Pixel grit bone / gunmetal / ember. Not neon.

## Tip priorities

Named scrap bots (#82) and Warmup Host drama (#83) shipped. **This is the NOW tip.**

## Non-goals

- Soft protocol redesign / map_id polish
- GCP apply / Terraform apply
- ElevenLabs / paid voice
- look_at / hit confirm reopen (HOLD)
- Neon HUD flood or Doom branding
- Tailscale as a product goal (public OR local only)

## Research (tip after #83)

| Seam | Tip today | Gap |
|---|---|---|
| Warmup Host wire | Snapshot host_line has roster + map + secs; round_time_left during Warmup | Roster case ends with bare `{secs}.` not GOES LIVE IN |
| Godot Warmup | Small RoundMessage flash + panel round label | Easy to miss in ~2s Warmup; no full-frame TV, no roster chips, countdown not giant |
| Tip stills | Mid-join Host flash, FP, weapons | No Warmup TV bumper still |

## Architecture impact

| Area | Change |
|---|---|
| `client` HUD | Full-frame WarmupTv overlay: veil, league chip, huge map, giant countdown, roster chips, Host line; live refresh; ~1s Active linger |
| `client` game_manager | Pass roster callsigns into Warmup TV; refresh each Warmup Snapshot; linger on RoundStart |
| `client` tip_capture | Capture `11_tip_warmup_tv_bumper_16x9.png` (early Warmup and/or forced TV) |
| `server` protocol | Enrich roster Warmup host_line to say GOES LIVE IN N (client chrome still primary) |
| `docs` | This plan; tip priorities; screenshots README + root README tip embed |

## Wire

Prefer client chrome first. Existing Warmup Snapshot already carries map_name, players, round_time_left, host_line.

Optional server string polish (same helpers, no new fields):

```text
HOST: CONTESTED FREQUENCY. ARENA DUEL TUNES IN. DEAD AIR DAN, NIGHTFALL ON THE SCRAP. GOES LIVE IN 2.
```

## Behavior

1. Warmup join / first Warmup Snapshot: raise full-frame WarmupTv (not only RoundMessage).
2. Each Warmup Snapshot: refresh map title, roster chips from player callsigns, giant GOES LIVE IN N, Host line.
3. RoundStart / Active: linger WarmupTv ~1s (FIGHT / LIVE energy), then hide so RoundStart chrome can own the fight bumper.
4. Disconnect resets WarmupTv + Host chrome latch.
5. Tip capture writes Warmup TV still for README proof.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
```

Godot: Warmup overlay readable at tip still capture. No tool attribution, emoji, or em/en dashes in commits/PR/docs.

## How to see unmissable Warmup

1. `cargo run -p fragr-server -- --bots 4` (port 6767).
2. Open Godot client, spectate or Join during Warmup (first ~2s after boot / after Ended linger).
3. Full-frame Contested Frequency TV: huge map title, roster chips, giant GOES LIVE IN N, Host flash.
4. Overlay lingers ~1s into Active / RoundStart so it is not blink-and-miss.
5. Tip still: `docs/screenshots/11_tip_warmup_tv_bumper_16x9.png`.

## Success

- PR open; Warmup TV is unmissable.
- Tip priorities: named+Warmup shipped; this NOW.
- CI-ready (Rust tests + fail-under 80 if Rust touched).
