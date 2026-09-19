# Plan: decision brain agent

**Status:** shipped (#102, 2026-09-18)
**Branch:** `feat/decision-brain`
**Spend:** $0 by default. Paid providers need an explicit per-run cap on the command line; the pre-approved ceiling for developer sessions is 5 dollars per run. Never in CI.

## Treating the model as a policy network (2026-09-19)

A walkthrough of wiring a decision model into a real-time game made three points that this crate had wrong, and one it already had right.

**The model is stateless, so it cannot see its own oscillation.** Each call is a fresh situation with no trace of what the fighter just did, and two situations a tick apart look identical, so a fighter picks push, then hold, then push forever. The state now carries the last few decisions, with a run of the same decision collapsed to one entry so holding a stance for a while does not push the oscillation out of the window.

**The distribution is the policy, and taking its largest entry throws that away.** A calibrated model returns a probability for every option; using only the argmax makes the fighter deterministic, so the same situation always produces the same move and a fighter that walked into a corner walks into it again. The stance is now drawn from the distribution, which keeps the model's own ordering (its favourite is still what it usually does) while letting the rest of the distribution break a loop. The draw comes from a per-fighter xorshift stream seeded off the fighter's name, kept in the repo rather than taken from a crate for the same reason the simulation's stream is, so a run reproduces and two fighters do not move in lockstep.

This is the same failure the reference agents hit from the other direction. They stood still because a policy with no memory and no randomness has nothing to do when the situation repeats. The harness fixed it with a patrol and a wedge counter; the brain fixes it with memory and a draw.

**Several typed questions ride in one call.** Already true here: the stance, the weapon and the danger score go together. The point worth adding is using a `noul` question as a local override rather than as advice: ask whether the fighter is stuck, and when the answer is confident enough, ignore the movement choice and run a local escape for a moment before handing control back. That is the next rung.

**A richer observation.** The state is words rather than numbers, which is right, but it describes only the nearest enemy. A vision cone, with each visible entity's bearing bucket and what it appears to be doing, is what a fighter would actually use to choose between backing off and flanking. Also on the next rung.

## Goal

Give fragr a reference agent whose macro intent comes from a decision model at two to five decisions per second while a local controller plays every tick. It is another way to engage as an agent, not a new kind of participant: one agent may combine a language model, other ML, and a decision model, and the server sees one fighter. The first brain is Jev, TypeSafe AI's decision model, reached natively or through OpenRouter. Any paid call sits behind one budget gate with a pre-approved cap, a pre-send estimate, a post-return settlement, and a ledger on disk.

## Why a decision model and not a chat model

The research (2026-09-18) is consistent: no published game agent runs a remote language model above about two decisions per second, and the ones that try inside the tick loop lose to a one-megabyte local policy. A Haiku-class model with a JSON schema needs about 1.4 seconds end to end for a sixty-token answer and costs about 13 dollars an hour at five decisions per second. TypeSafe states an end-to-end response time of 70 to 500 ms; Jev returns a distribution instead of prose, and lists at 0.042 dollars per million input tokens with free output. Measured through OpenRouter on 2026-09-18: roughly 700 input tokens and three hundredths of a cent per call, which is about thirty-three cents an hour at three decisions per second. It cannot produce an invalid action because the options are ours.

The two-tier split is the same one SIMA 2 and every working design uses: a slow brain sets intent, a fast local loop executes. Our fast loop is the existing reflex policy, extended with stances.

## Non-goals

- Replacing the MCP adapter. Traditional agents keep that door; this is a second client on the same wire.
- Any model on the combat tick. The controller never waits on the network.
- A general LLM provider abstraction. One decision shape, two endpoints. Chat models can come later behind the same budget gate if a use appears.
- Publishing Jev performance numbers. TypeSafe's customer agreement forbids it. Playtest reports stay local.

## Shape

- Crate: `agents/brain` (`fragr-brain`), Rust, workspace member, wire types from `fragr-server`.
- Modules: `budget` (caps, ledger, pricing, estimate), `telemetry` (snapshot to a four-line state string), `plan` (stances, local rules, the 20 Hz controller), `decision` (question set, answer parsing, confidence gating), `provider` (endpoints, headers, transport, the budgeted call), `bot` (the play loop), `dotenv` (key discovery).
- Command: `fragr-brain [--provider local|typesafe|openrouter] [--max-spend-usd N] play|ask|spend|key`.
- Cadence: controller every 50 ms from the latest snapshot; one decision in flight at a time at `--decision-hz` (0.1 to 5, default 3); a late answer is simply late.
- Failure policy: refused, failed, slow, or low-confidence decisions fall back to local rules for that cycle. A budget refusal turns the brain off for the rest of the run, once.

## Providers (verified against both OpenAPI documents on 2026-09-18)

| | TypeSafe native | OpenRouter |
|---|---|---|
| Endpoint | `POST https://api.typesafe.ai/v1/systemone` | `POST https://openrouter.ai/api/alpha/decisions` |
| Model | `jev-1.13.0` pinned (`jev-latest` and `jev-preview` alias it) | `typesafe/jev-1.13` (served as `jev-1.13-20260917`); `~typesafe/jev-latest` also accepted; plain `typesafe/jev-latest` returns 400 |
| Auth | `Authorization: Bearer` | `Authorization: Bearer` plus `HTTP-Referer` and `X-OpenRouter-Title` for app attribution |
| Body | `state`, `model`, `questions` map | same |
| Answers | `answers.<name>.{choice,noul,score}` plus `confidence`, `probabilities` | same, plus `id`, `provider` |
| Usage | `input_tokens`, `output_tokens` | same, plus `cost` in dollars |
| Price | 0.042 per million input, output free | same |
| Key limits | none exposed | `GET /api/v1/key` reports `limit`, `limit_remaining`, `usage`; keys can be created with a hard dollar limit |
| Key names | `TYPESAFE_API_KEY`, `typesafe` | `OPENROUTER_API_KEY`, `openrouter` |

Question types are `choice` (named options with descriptions), `noul` (probability of yes; the name is TypeSafe's, short for Bernoulli), and `score` (ordered levels; the answer is the expected zero-based index into the list, with probabilities keyed "0" upward). Limits: 64k tokens per request, questions evaluated in parallel.

## Budget gate

1. `--max-spend-usd` defaults to zero; a paid provider with a zero cap refuses to start.
2. Estimate before send: request bytes divided by two (the measured billing ratio), rounded up, plus overhead, at the configured price. A call that would cross the run cap, the ledger cap (`--max-total-usd`), or the call cap (`--max-calls`) is not sent.
3. Settle after return: reported tokens (and OpenRouter's reported cost) replace the estimate.
4. Ledger at `.agents/spend/brain.jsonl` (gitignored, one JSON line per call, appended under a file lock) records every sent call, successful or not, and carries totals across runs and processes. `fragr-brain spend` prints it. The per-run cap is refused above five dollars by a constant in the code.
5. Provider backstop: an OpenRouter key with its own dollar limit; `fragr-brain key` shows it.

## Verification

- Unit tests: caps and ledger, dotenv discovery, telemetry rendering (exact string), local rules per stance, controller per stance, question serialization, answer parsing for both provider shapes, confidence gating, provider request shapes with the token masked, error extraction, the budgeted call (refused calls are never sent; failed calls are still charged).
- In-process tests boot the real server and run the bot: local provider plays for free and never touches the transport; scripted remote answers drive the plan and the ledger; a zero cap refuses once and hands over to rules; a failing endpoint falls back and still counts.
- CLI tests cover every subcommand with a scripted transport, including the dry run masking the key.
- Coverage stays above the unfiltered 80 percent floor.
- A real call against each provider is a developer smoke with a key and a cap, recorded in the ledger, not in CI.

## Research notes (2026-09-18) and what they changed

Sources: TypeSafe's state, primitives, confidence, retries, and model-jaggedness pages; OpenRouter's decisions schema and limits pages; chess engine testing statistics for the experiment design. Full citations live in the session research; the repository keeps the conclusions.

- **State as an object of words.** TypeSafe: use an object with descriptive names, include only what the questions need, do comparisons in code, and never rely on the model to compare numbers. Applied: `Telemetry::state_object` sends health tiers, range buckets (close, mid, far), pad nearness, a score edge, and a clock bucket. Names and raw distances are gone.
- **Criteria that say what belongs and what belongs to a neighbour.** Each stance option now reads "For: ... Not for: ...". Score levels describe situations, not degrees, and the bot takes the most likely level rather than the expectation, which TypeSafe documents as weakly calibrated. Note: OpenRouter's schema types choice criteria as strings only, so structured criteria would need the native endpoint.
- **Gate on margin, confidence as backup.** `confidence` is a statistic derived from the probabilities and its formula is unpublished; TypeSafe's worked example treats 0.60 versus 0.38 (confidence 0.39) as clear enough to act on. Default gate: margin at least 0.2, or confidence at least 0.65. Live sampling before this change showed the 0.65 confidence floor alone rejecting most stance answers.
- **Backoff, never in-cycle retry.** 429, 529, other server errors, and timeouts double the decision interval up to sixteen times the base; a success resets it. TypeSafe's default limit is 1,200 requests per minute per key, so four to six brain fighters at three to five decisions per second would saturate it.
- **Pin the version.** Thresholds are tuned against Jev 1.13; defaults are `jev-1.13.0` natively and `typesafe/jev-1.13` through OpenRouter. OpenRouter also accepts `~typesafe/jev-latest`; plain `typesafe/jev-latest` returns 400.
- **One state per request.** Several fighters per request are possible through field paths but conflict with the distractor guidance; each fighter asks alone.
- **Privacy.** TypeSafe does not train on inputs, and OpenRouter's provider feed marks the Jev endpoint as zero data retention. OpenRouter's own logging is off by default.
- **What stays out of the public repo.** TypeSafe's customer agreement forbids publishing benchmarks or performance information about the service. Cost math from the public price page, the architecture, the state and question text, and the harness are fine to publish; latency figures, win rates, and accuracy plots attributed to Jev stay in gitignored ledgers and reports.

## Validation design (rung 2)

The question is whether a brain fighter beats a reflex fighter, and by how much, at a cost we can state up front.

- **Unit of measurement:** a round pair on the same map seed and item timers with the brain and reflex fighters swapping spawn sides, scored as win, draw, or loss for the brain by frags. Pairs, not single rounds, so spawn luck cancels (the chess engine testing literature models paired games with a pentanomial distribution for this reason).
- **Sample size:** to detect a ten point win-rate difference from even at 80 percent power with a two-sided test at the 5 percent level takes 194 rounds (one-sample test of a proportion against 0.5); 259 rounds at 90 percent power; 85 rounds for a fifteen point effect; 783 for five points. At 200 rounds the 95 percent interval half-width is about seven points. Draws count as half a win.
- **More than two policies:** TrueSkill or Elo over the pool of reflex, planner, and brain fighters.
- **Cost:** at measured token counts and three decisions per second, one brain fighter costs about 33 cents an hour, so 200 three-minute rounds cost about 3.30 dollars per brain fighter, or about 5.50 at five decisions per second. A mirror pair with two brain fighters doubles that. Worst case under 12 dollars.
- **Harness:** a `brain` tier in `tools/playtest` fielding brain agents next to reflex agents under a cap, with decision-source counts and backoffs in the report, and a paired-round mode with fixed seeds.
- **Reporting:** results go to gitignored `.agents/playtest/` and a private note, never to the repository, per the terms above.

## Rungs

1. This plan: crate, budget gate, local and remote tiers, docs. Shipped.
2. Playtest tier `brain` with the paired-round validation design above.
3. Richer questions once the campaign lands: target choice among several enemies, objective intent, a `noul` for whether to speak a taunt.
4. A team surface (A2A-style or a shared blackboard through the adapter) so brains can coordinate without touching the tick.

## Success criteria

- [x] Local provider plays in-process on every PR at zero cost (the three second `local_provider_plays_for_free` test); the thirty second smoke in AGENTS.md covers a round.
- [x] A paid provider refuses to start without a cap and stops at the cap, proven by the budget and bot tests.
- [x] A developer smoke against OpenRouter recorded in the local ledger (2026-09-18: three `ask` calls and a live `play` session under a 25 cent cap). TypeSafe native still needs a key (waitlist).
- [ ] Playtest tier `brain` with paired rounds and a private results note.
