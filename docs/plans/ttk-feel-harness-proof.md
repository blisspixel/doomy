# Sticky TTK / feel harness proof

**Status:** in flight
**Branch:** `feat/ttk-feel-harness-proof`
**Spend:** $0
**Related:** #124 (guns that kill in a second), [`gunfeel.md`](./gunfeel.md), [`agent-playtest-loop.md`](./agent-playtest-loop.md)

## Goal

Make the #124 weapon-table claim CI-assertable from `tools/playtest`, not only restated in plan prose. Flechette, rail, and scatter clean-hit time to kill must stay sticky inside the gunfeel band, and `--assert` must fail if the table drifts.

## Non-goals

- Soft louder stance chips (separate work)
- Dish chrome / tip_capture / jammer silhouette
- Asserting measured playtest median TTK on the short CI sample (sample noise; report only)
- Feedback juice, dodge, movement inaccuracy (later gunfeel rungs)

## Plan table

| Piece | Owner | Assert |
|---|---|---|
| Flechette 4 hits / 0.6 s | `sticky_weapon_ttk_table` | hits and seconds exact; band 0.5..1.2 s |
| Rail 2 hits / 1.0 s | same | hits and seconds exact; band 0.5..1.2 s |
| Scatter 3 hits / 0.9 s point-blank | same | hits and seconds exact; band 0.5..1.2 s |
| `--assert` path | `check_thresholds` | always runs `check_sticky_ttk_table` |
| Per-weapon measured TTK | `WeaponReport.time_to_kill_s` | unit-tested attribution; CI reports, does not threshold |

## Architecture impact

| Area | Change |
|---|---|
| `tools/playtest` | Sticky table helpers, `--assert` wiring, per-weapon TTK quantiles |
| `docs/plans` | This plan plus index row |
| Server / client / dish | None |

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p fragr-playtest
cargo test --workspace
cargo llvm-cov --workspace --locked --summary-only --fail-under-lines 80
cargo run -p fragr-playtest -- --agents 4 --rounds 1 --frag-limit 3 --time-limit-seconds 45 --assert --report .agents/playtest/ttk-proof.json
```

## Success criteria

- [x] Sticky flechette / rail / scatter TTK asserted in playtest tests
- [x] `fragr-playtest --assert` fails if the table leaves the band
- [x] Per-weapon `time_to_kill_s` present on combat reports
- [x] Unfiltered coverage floor remains 80
- [ ] Lean PR squash-merged when green
