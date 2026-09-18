# Weapon roles excellence (Flechette / Rail / Scatter)

**Repo:** https://github.com/blisspixel/fragr
**Branch:** `cursor/weapon-roles`
**Spend:** $0. No ElevenLabs, no GCP apply, no renet, no look_at/hit reopen.
**Status:** Shipped (#76 / `9734faf`). Tip stills recapture is NOW; see [`tip-stills-recapture.md`](./tip-stills-recapture.md).

## Goal

Flechette / Rail / Scatter must feel distinct in Join FP and on bots: not samey hitscan with a skin swap. Sharpen authoritative roles (rate, damage, spread, range), bot tactics that prefer weapons by engagement range, and Godot FP juice (viewmodel / crosshair / fire / light hit marker) plus distinct procedural fire/hit SFX per weapon.

## Tip priorities

Killstreak Host juice shipped (#75 / `979f263`). **Weapon roles excellence shipped. Tip stills recapture is the NOW tip.** Hit markers land as part of FP feel when the local (or followed) player scores a hit from Snapshot `shot_results` / `hit` observe. look_at / hit HOLD. Port 6767. Coverage fail-under 80.

## Non-goals

- GCP apply
- ElevenLabs
- look_at / hit reopen (HOLD)
- New map / choke layout
- Neon HUD flood or Doom branding
- Protocol shape break (wire already has `shot_results` / `hit`)

## Locks

- Contested Frequency pixel grit (bone / gunmetal / blood / ember)
- Port 6767
- Coverage fail-under 80 (unfiltered)
- look_at / hit HOLD

## Role table (authoritative)

| Weapon | Role | Damage | Cooldown | Spread (rad) | Range |
|---|---|---|---|---|---|
| Flechette | Mid workhorse | 25 | 10 (~0.5s) | 0.10 | 42 |
| Rail | Long precision | 75 | 40 (~2.0s) | 0.04 | 100 |
| Scatter | Close shred | 15 | 5 (~0.25s) | 0.38 | 14 |

Range caps hitscan so Scatter dies at distance and Rail owns long lanes. Spread / cooldown / damage already differ; range makes the roles real in the arena.

## Architecture impact

| Area | Change |
|---|---|
| `server` protocol | `WeaponType::range_units()`; slight Rail/Scatter spread sharpen |
| `server` sim | Hitscan uses per-weapon range; bots fight / seek pads by role |
| `client` HUD | Per-weapon FP crosshair + viewmodel pose; light hit marker + floating damage |
| `client` game_manager | Route `shot_results` / `hit` to local (and optional followed) hit juice |
| `client` player_pawn | Distinct fire/hit streams + muzzle juice per weapon |
| `tools/generate_audio.py` | Procedural per-weapon fire/hit WAVs (CC0 path) |
| `docs/plans/README.md` | Tip priorities: killstreak shipped; this NOW |
| Tip stills | Ensure `10_tip_human_join_fp` in screenshots table; note recapture of 01/07/08/09 when Xvfb path works |

## Behavior

1. Server: each weapon has distinct range; miss beyond range. Existing damage / cooldown / spread stay role-defining; Rail tighter, Scatter wider.
2. Bots: engage at preferred distance for held weapon (Rail long, Scatter close, Flechette mid). Flechette bots still chase Rail/Scatter pads; once armed, hold the role lane.
3. Join FP: crosshair shape and viewmodel scale/offset/modulate follow weapon. Successful local hit from `shot_results` flashes a grit hit marker and a brief floating damage number (ember/bone, not neon). Optional: same when spectator follows the shooter.
4. Audio: distinct fire and hit SFX per weapon via procedural generator; fallback to legacy `fire.wav` / `hit.wav` if missing.
5. look_at / hit protocol HOLD. No new Event kinds required.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
# Optional Godot import
godot --headless --path client --import --quit-after 120
# Playable: server --bots 4, Join; swap / pick Rail vs Scatter vs Flechette and feel roles
```

## How each weapon should feel

1. `cargo run -p fragr-server -- --bots 4` (port 6767).
2. Open Godot client, Join or Solo Scrap.
3. **Flechette:** mid crosshair, snappy mid-range chatter, balanced viewmodel.
4. **Rail:** tight center dot, long reach, heavy fire kick, cold gunmetal flash, sparse cadence.
5. **Scatter:** wide ring crosshair, short reach, chunky close spray, ember burst, fast cadence.
6. Land a hit: grit marker + floating damage; SFX matches the weapon.

## Success criteria

- Plan in tree; tip priorities list this as NOW (killstreak shipped)
- Three roles read in hand and in bot tactics
- Rust tests green; coverage fail-under 80
- PR open on `cursor/weapon-roles`
- Tip stills table includes `10_tip_human_join_fp`; recapture note for 01/07/08/09 if needed

## Spend and safety

- $0 local only
- No cloud apply, no paid APIs
- No tool attribution, emoji, or em/en dashes
