# Plan: Fun Playable Pass (finish line push)

**Repo:** https://github.com/blisspixel/fragr  
**Branch:** `cursor/fun-playable-pass-932d`  
**Spend:** $0 only. No paid assets, APIs, or hosting.  
**Status:** Plan of record for exceptional fun pass on current main.

## Goal

Take fragr from solid foundation (match loop + MCP events landed) to exceptionally fun and immediately playable. Stranger runs cargo + opens Godot and feels a compelling fight within 60 seconds.

## Non-goals

- Paid assets, APIs, or hosting
- Waiting on separate assets PR
- Terraform apply (infra plan-only stays frozen)
- New infrastructure beyond fun polish
- Doom branding

## Current state (evidence from main)

Main already has:
- Server: 20 Hz authoritative tick, round system (warmup, frag limit, time limit), 4 named bots with distinct behaviors (Aggressive, Defensive, Flanker, Balanced)
- Client: Godot 4.7.2, pose interpolation, hit feedback (red flash + scale pulse), muzzle flash, behavior chips on HUD, killfeed, scoreboard, round messages
- Protocol: WebSocket JSON, MCP event support
- Join/leave: J to join as human, L to leave back to spectate
- Tests: server/src/tests.rs exists

## Gaps vs exceptional fun bar

After reviewing current implementation:

### P0: Arena readability
- Current arena is basic gray box (from arena.tscn)
- Spawns are not visually marked (players spawn in circle pattern but no ground markers)
- No visual callouts or contrast zones to help orient spectators

**Fix:** Add simple colored rectangles/zones in arena for visual interest and orientation without blocking on art pack.

### P1: Combat juice timing
- Hit feedback exists (red flash + scale pulse 0.2s)
- Muzzle flash exists (bright yellow emission 0.1s)
- Killfeed exists and shows in HUD
- Death cam missing (brief follow-on-frag moment)

**Fix:** Tighten muzzle flash timing (0.08s), add brief camera focus on frag (1s follow killer after scoring), enhance killfeed with color coding.

### P1: Bot drama visibility
- Bots have behaviors: Aggressive, Defensive, Flanker, Balanced
- Behavior chips show in label: "Rusher [100] [Aggressive]"
- Bots already have distinct colors and names

**Already good!** Minor enhancement: make behavior chip more prominent with icon or color coding.

### P2: Audio stubs
- No audio currently
- Godot has built-in AudioStreamPlayer
- Can use procedural beeps via AudioStreamGenerator (free, local, no licensing)

**Fix:** Add minimal procedural audio for shot and frag (optional, skip if adds complexity).

### P0: README 60-second path
- Current README has quick start but could be tighter
- "First 30 seconds as spectator" section exists but buried

**Fix:** Front-load the fun experience in README intro.

## Architecture impact

- Client: arena.tscn gets visual zones (ColorRect or simple mesh planes), spectator_cam.gd adds brief frag-follow
- Client: hud.gd enhances killfeed with color, player_pawn.gd tweaks muzzle timing
- Server: no changes needed (already exceptional)
- README: reorder sections to lead with fun
- Optional: minimal procedural audio if trivial

## Technical approach

### Arena readability (arena.tscn + new ColorRect nodes)
- Add 3-5 colored floor zones (red, blue, green, yellow, cyan rectangles or MeshInstance3D planes)
- Position at strategic spots: center, corners, chokepoints
- Use distinct colors for callout potential ("fight at red" "blue corner")
- Keep it simple: flat colored rects, no textures

### Combat juice timing
- Reduce muzzle flash duration from 0.1s to 0.08s (snappier)
- Add frag-follow camera: on frag event, spectator cam locks to killer for 1s then resumes normal follow cycle
- Enhance killfeed: color-code killer/victim names based on their player_color

### Bot drama visibility
- Behavior chip already shows "[Aggressive]" etc
- Add color coding: Aggressive=red, Defensive=cyan, Flanker=gold, Balanced=white
- Make chip slightly larger or use icon prefix

### Audio stubs (optional, time-boxed)
- Use AudioStreamGenerator for procedural beeps (pure code, no assets)
- Shot sound: 0.05s sine wave at 220 Hz
- Frag sound: 0.1s chord at 440+554 Hz
- If takes more than 30 minutes, skip and note as future work

### README rewrite
- Move "What you see" section to top (after quick start)
- Add "Why it is fun" callout: bot drama, spectate-then-join, persistent fight
- Keep technical details below the fold

## Verification

Before done:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace

# Smoke test
cargo run -p doomy-server -- --bots 4 &
# Open client/ in Godot 4.7.2, press F5
# Verify: spawns visible, bots fight, first frag within 10s, killfeed clear, arena readable
# Press J to join, verify human can frag bot
# Press L to leave, verify spectate resumes
pkill -f doomy-server
```

Manual Godot check:
- Arena has colored zones
- Muzzle flash is snappy (less than 0.1s)
- Killfeed has color-coded names
- Behavior chips are readable
- Camera follows fragger briefly after kill
- Audio plays (if implemented)

No emoji, no em/en dashes, no tool attribution in any new prose/commits/PR text.

## Success criteria

1. Stranger can clone repo, run server, open Godot, and see compelling fight within 60 seconds.
2. Arena has visual interest (colored zones for orientation).
3. Combat feedback is tight and satisfying (muzzle flash < 0.1s, frag-follow camera).
4. Killfeed is clear with color-coded names.
5. README front-loads the fun experience.
6. Tests pass (existing tests remain green, add minimal new tests if needed).
7. All AGENTS.md verification commands pass.
8. $0 spend (no assets, no APIs).

## Build order

1. Enhance arena readability (add colored floor zones to arena.tscn).
2. Tighten combat juice (muzzle flash timing, frag-follow camera, killfeed colors).
3. Enhance behavior chip visibility (color coding).
4. Rewrite README to front-load fun.
5. Optional: add procedural audio stubs (time-boxed to 30 min).
6. Run full verification suite.
7. Commit, push, open PR vs main.

## Spend / safety

$0 only. No assets, no APIs. Use Godot built-ins and code-only solutions.

## Evidence threshold

Manual playthrough video or detailed written confirmation that stranger fun bar is met. Do not claim "should work" without running it.
