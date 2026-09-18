# Plan: decision brain agent

**Status:** in flight (2026-09-18)
**Branch:** `feat/decision-brain`
**Spend:** $0 by default. Paid providers need an explicit per-run cap on the command line; the pre-approved ceiling for developer sessions is 5 dollars per run. Never in CI.

## Goal

Give fragr a third agent tier next to the scripted bot and the playtest reflex agents: a fighter whose macro intent comes from a decision model at two to five decisions per second while a local controller plays every tick. The first brain is Jev, TypeSafe AI's decision model, reached natively or through OpenRouter. Any paid call sits behind one budget gate with a pre-approved cap, a pre-send estimate, a post-return settlement, and a ledger on disk.

## Why a decision model and not a chat model

The research (2026-09-18) is consistent: no published game agent runs a remote language model above about two decisions per second, and the ones that try inside the tick loop lose to a one-megabyte local policy. A Haiku-class model with a JSON schema needs about 1.4 seconds end to end for a sixty-token answer and costs about 13 dollars an hour at five decisions per second. Jev answers typed questions in 70 to 500 ms, returns a distribution instead of prose, and lists at 0.042 dollars per million input tokens with free output, which is about five cents an hour at three decisions per second on our state size. It cannot produce an invalid action because the options are ours.

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
| Model | `jev-latest` (alias of `jev-1.13.0`) | `typesafe/jev-1.13` (the `~typesafe/jev-latest` alias is not in the catalog) |
| Auth | `Authorization: Bearer` | `Authorization: Bearer` plus `HTTP-Referer` and `X-OpenRouter-Title` for app attribution |
| Body | `state`, `model`, `questions` map | same |
| Answers | `answers.<name>.{choice,noul,score}` plus `confidence`, `probabilities` | same, plus `id`, `provider` |
| Usage | `input_tokens`, `output_tokens` | same, plus `cost` in dollars |
| Price | 0.042 per million input, output free | same |
| Key limits | none exposed | `GET /api/v1/key` reports `limit`, `limit_remaining`, `usage`; keys can be created with a hard dollar limit |
| Key names | `TYPESAFE_API_KEY`, `typesafe` | `OPENROUTER_API_KEY`, `openrouter` |

Question types are `choice` (named options with descriptions), `noul` (probability of yes; the name is TypeSafe's, short for Bernoulli), and `score` (ordered levels). Limits: 64k tokens per request, questions evaluated in parallel.

## Budget gate

1. `--max-spend-usd` defaults to zero; a paid provider with a zero cap refuses to start.
2. Estimate before send: request bytes divided by four, rounded up, plus overhead, at the configured price. A call that would cross the run cap, the ledger cap (`--max-total-usd`), or the call cap (`--max-calls`) is not sent.
3. Settle after return: reported tokens (and OpenRouter's reported cost) replace the estimate.
4. Ledger at `.agents/spend/brain.json` (gitignored) records every sent call, successful or not, and carries totals across runs. `fragr-brain spend` prints it.
5. Provider backstop: an OpenRouter key with its own dollar limit; `fragr-brain key` shows it.

## Verification

- Unit tests: caps and ledger, dotenv discovery, telemetry rendering (exact string), local rules per stance, controller per stance, question serialization, answer parsing for both provider shapes, confidence gating, provider request shapes with the token masked, error extraction, the budgeted call (refused calls are never sent; failed calls are still charged).
- In-process tests boot the real server and run the bot: local provider plays for free and never touches the transport; scripted remote answers drive the plan and the ledger; a zero cap refuses once and hands over to rules; a failing endpoint falls back and still counts.
- CLI tests cover every subcommand with a scripted transport, including the dry run masking the key.
- Coverage stays above the unfiltered 80 percent floor.
- A real call against each provider is a developer smoke with a key and a cap, recorded in the ledger, not in CI.

## Rungs

1. This plan: crate, budget gate, local and remote tiers, docs. Ships with the plan.
2. Playtest tier `brain` so the harness can field brain agents next to reflex agents under a cap, with decision-source counts in the report.
3. Richer questions once the campaign lands: target choice among several enemies, objective intent, a `noul` for whether to speak a taunt.
4. A team surface (A2A-style or a shared blackboard through the adapter) so brains can coordinate without touching the tick.

## Success criteria

- [ ] Local provider plays a round on every PR at zero cost.
- [ ] A paid provider refuses to start without a cap and stops at the cap, proven by tests.
- [ ] A developer smoke against TypeSafe and OpenRouter recorded in the local ledger.
- [ ] Playtest tier `brain`.
