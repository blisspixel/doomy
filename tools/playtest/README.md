# fragr-playtest

Scripted agents play fragr rounds and file a metrics report, so most iteration on feel does not need human testers. The harness boots the authoritative server in-process on a free loopback port, connects reflex agents over the real WebSocket wire, watches the match as a spectator, and folds what it saw into numbers: time to first frag, frags per minute, longest gap without a frag, Host beats per minute, spawn deaths, per-agent idle and stuck time, weapon usage, and snapshot bytes per tick.

```bash
cargo run -p fragr-playtest -- --agents 4 --rounds 1 --frag-limit 3 --time-limit-seconds 45 --assert --report .agents/playtest/run.json
```

`--assert` exits non-zero when a frustration threshold is crossed: no round completed, an agent stuck (no movement and no fire during an active round) for more than five seconds, spawn deaths above ten percent of frags, or fewer than one frag per minute with four or more agents. It also fails when the sticky flechette, rail, or scatter clean-hit time to kill leaves the 0.5 to 1.2 second band (the #124 table). CI runs exactly that command on every PR. Reports land under the gitignored `.agents/` directory; put the summary table for a change under test into that change's plan doc.

Agent tiers: `reflex` (face the nearest fighter, close in, fire in range) ships now. The planner tier, server status metrics, and the optional off-tick observer are described in `docs/plans/agent-playtest-loop.md`.
