# fragr-brain

An example agent whose intent comes from a decision model and whose reflexes stay local. It is the third rung of the agent ladder: the scripted bot reacts, the playtest reflex agents react, and this one asks a brain what to do a few times a second while a local controller keeps playing every tick.

The brain is Jev, TypeSafe AI's decision model, reached either at TypeSafe's own endpoint or through OpenRouter. Jev does not generate text. It answers typed questions (a choice between named options, a yes-or-no probability, a score on an ordered scale) with calibrated confidence, in a few hundred milliseconds, for about four cents per million input tokens. That shape fits a shooter far better than a chat model: no prose to parse, no invalid actions to guard against, no warm-up.

Traditional agents keep their door. Any MCP client still drives a fighter through `agent-adapter`; this crate is a separate, optional client on the same wire protocol.

## Run it for free

```bash
# Terminal 1
cargo run -p fragr-server -- --bind 127.0.0.1:6767 --bots 3

# Terminal 2: local rules only, no key, no spend
cargo run -p fragr-brain -- play --name Brain-1 --max-seconds 120
```

`--provider local` is the default. The same loop runs, the same telemetry is built, and the same controller plays; only the decision comes from rules instead of a model. CI exercises this path in-process.

## Run it with a brain

Put a key in `.env` at the repository root (gitignored) or in the environment:

```
typesafe=sk_...          # or TYPESAFE_API_KEY
openrouter=sk-or-...     # or OPENROUTER_API_KEY
```

Then pre-approve a cap. Nothing paid happens without one:

```bash
cargo run -p fragr-brain -- --provider typesafe --max-spend-usd 5 play --name Jev-1 --decision-hz 3
cargo run -p fragr-brain -- --provider openrouter --max-spend-usd 5 play --name Jev-2
```

Both providers send the same body: a `state` string, a `model`, and a `questions` map. TypeSafe expects `jev-latest` at `https://api.typesafe.ai/v1/systemone`. OpenRouter expects `typesafe/jev-1.13` at `https://openrouter.ai/api/alpha/decisions` and gets the app attribution headers pointing at this repository. Override the model with `--model`.

## Budget controls

Every paid path goes through one gate, in this order:

1. **Pre-approval.** `--max-spend-usd` defaults to zero. A paid provider with a zero cap refuses to start and says how to approve one.
2. **Estimate before send.** Each request is priced from its byte length (four characters per token, rounded up, plus overhead) at `--price-input-per-million` and `--price-output-per-million`, which default to Jev's list price. If the estimate would cross a cap, the call is not sent.
3. **Settle after return.** The provider's reported usage (and OpenRouter's reported cost) replaces the estimate in the running total.
4. **Ledger on disk.** Every sent call, successful or not, lands in `.agents/spend/brain.json`. The total carries across runs. `--max-total-usd` caps that total; `--max-calls` caps the count regardless of price. `fragr-brain spend` prints it.
5. **Fail open to rules.** A refused, failed, slow, or low-confidence decision hands the fighter to the local rules for that cycle. A cap refusal turns the brain off for the rest of the run, once, with a warning. The fighter never stops playing.
6. **Provider-side backstop.** For OpenRouter, create a key with its own dollar limit in the dashboard; `fragr-brain --provider openrouter key` shows the limit and what remains, and warns when the key has none.

A rough budget: a state is about 220 characters, the three questions about 900, so a call is roughly 300 input tokens. At three decisions per second that is under one tenth of a cent per minute, or about five cents an hour of continuous play. A five dollar cap is over a hundred hours.

## What the brain is asked

The state is four short lines, deterministic, lowest information that still decides the fight:

```
SELF hp=75 armor=0 weapon=flechette under_fire=no recent_damage=0 score=0 top_rival=0
ENEMY name=Probe-2 dist=12.3 hp=mid weapon=rail
PADS health=8.1 armor=none weapon.scatter=2.0
ROUND state=active time_left=90 fighters=2
```

The questions are fixed so estimates stay honest and providers can cache:

- `stance`, a choice: `push_enemy`, `fall_back_heal`, `hold_angle`, `kite_distance`.
- `weapon`, a choice: `scatter`, `flechette`, `rail`, with the ranges spelled out.
- `danger`, a score: `safe`, `watchful`, `pressured`, `critical`, `dying`.

Answers below `--confidence-floor` (default 0.65) do not change the stance. The controller then turns the stance into wire actions every 50 ms: aim at the nearest living fighter, fire inside the held weapon's range, close, hold and strafe, back off, or run to the nearest health pad.

Try one decision by hand, with or without sending:

```bash
cargo run -p fragr-brain -- --provider typesafe ask --dry-run --state "$(printf 'SELF hp=20 armor=0 weapon=flechette under_fire=yes recent_damage=40 score=1 top_rival=3\nENEMY name=Kragge dist=6.0 hp=high weapon=scatter\nPADS health=9.0 armor=none\nROUND state=active time_left=60 fighters=4\n')"
cargo run -p fragr-brain -- --provider typesafe --max-spend-usd 0.01 ask --state "..."
```

## What it reports

`play` prints a JSON summary when it leaves: snapshots seen, actions sent, decisions by source (remote, low confidence, failed, local, budget refusals), frags, deaths, dollars this run, dollars in the ledger, the last plan, and the last state string.

## Known limits

- Jev reads instructions literally, is weak at arithmetic and multi-step reasoning, and can be distracted by irrelevant state. The state is kept tiny and the questions are blunt on purpose.
- TypeSafe's customer agreement forbids publishing benchmarks or performance figures about the service. Keep win rates out of the repository; the playtest harness reports stay under gitignored `.agents/`.
- The OpenRouter alias `~typesafe/jev-latest` is not in its public catalog; the dated id is the default here.
- No A2A surface yet. Team play between brains is a roadmap item.
