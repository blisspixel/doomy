# Plan: the agent door in 2026

**Status:** planned (2026-09-18)
**Branch:** `feat/mcp-2026`
**Spend:** $0.

## Goal

Keep the MCP adapter on the current specification without breaking the clients people actually use, decide whether to adopt the official Rust SDK, and give agent teams a way to coordinate off the combat tick. The decision-brain agent ([`decision-brain.md`](./decision-brain.md)) is a separate client on the wire and is not affected.

## Non-goals

- Serving the old handshake forever. Legacy negotiation stays until the major clients drop it, then it goes, and this plan names the date when that happens.
- Any model on the combat tick.
- A2A before there is a cross-host recruitment case.

## What changed in MCP 2026-07-28

- `initialize` and `notifications/initialized` are gone for modern clients. Every request carries `_meta` with `io.modelcontextprotocol/protocolVersion` and `io.modelcontextprotocol/clientCapabilities`; a missing `_meta` is rejected with `-32602`, an unsupported version with `-32022` and a `data.supported` list.
- `server/discover` is mandatory and returns `resultType`, `supportedVersions`, `capabilities`, `instructions`, `ttlMs`, `cacheScope`, and `serverInfo` in `_meta`.
- Every result carries `resultType` (`complete` or `input_required`). List results (`tools/list`, `prompts/list`, `resources/list`, `resources/read`, `resources/templates/list`) carry `ttlMs` and `cacheScope`, and tools are returned in a deterministic order.
- Removed: `ping`, `logging/setLevel`, roots list-changed notifications, `resources/subscribe` (replaced by `subscriptions/listen`), server-initiated requests, SSE resumability, `Mcp-Session-Id`. Deprecated for twelve months: roots, sampling, logging, the HTTP plus SSE transport.
- stdio remains a standard transport (newline-delimited JSON-RPC, nothing else on stdout). Streamable HTTP is POST-only with `Mcp-Method`, `Mcp-Name`, and `MCP-Protocol-Version` headers mirrored from the body.
- Dual era is allowed: answer `initialize` with legacy semantics for old clients and serve `_meta`-carrying requests statelessly in the same process.

## Who still speaks the old way

The TypeScript SDK v2 probes `server/discover` and falls back to one legacy codec covering 2024-10-07 through 2025-11-25. Claude Code defaults to legacy negotiation for stdio servers with no deprecation notice. The Inspector negotiates both eras. Cursor's status is unknown. One trap from the field: a server that hardcodes `2024-11-05` against a client asking for `2025-11-25` gets its tools refused by Claude Desktop, so the adapter must echo the client's version whenever it supports it.

## The official Rust SDK

`rmcp` 3.4.0 (2026-09-15), MSRV 1.88, Apache-2.0, five releases between August and September 2026. It provides stdio and streamable HTTP transports, tool and prompt macros, a `negotiate_initialize` handler for legacy clients, and claims full server conformance for 2026-07-28 with compatibility back to 2025-11-25 and earlier. Known issues at the time of writing: an unbounded stdio line buffer, two P0 header bugs on the HTTP transport, an empty `cacheScope` dropping tools, and naming churn across minors. Verdict: lower risk than hand-rolling the dual-era plumbing while the spec keeps moving, provided fragr's tool logic stays behind its own trait so the transport layer can swap either way.

## Teams: A2A or a blackboard

A2A v1.0 is stable, sits under the Linux Foundation's Agentic AI Foundation as of 2026-08-27, and has official Rust crates (`a2a-lf` 0.3.1 and companions), all still 0.x. Its shape for "agents that field agents" is a Task sent to a roster agent that replies working, then completed with an artifact holding a join token, or input required to negotiate. Tasks are seconds-scale and async, so they belong on the planning loop, never the combat tick.

For one game server, a blackboard through MCP is enough: a `team/board` resource, `post_intent` and `read_board` tools, short `ttlMs`, and `subscriptions/listen` for changes, with the roster cap enforced by the server. Adopt A2A only if agents from other hosts must recruit each other; A2A itself frames its scope as agents partnering rather than agents using tools.

Game servers already exposed as MCP servers (a text adventure with ten tools, a board-game arena with six, a Minecraft bridge with sixty-one) all expose observe and act as tools with polled state and rarely use resources. fragr's `observe` returning a compact snapshot with a tick number matches that pattern; an opt-in event stream through `subscriptions/listen` can come later.

## Rungs

1. **Current revision first.** The adapter targets 2026-07-28 as its native era: `server/discover`, `_meta`-carrying stateless requests, `resultType`, `ttlMs`, `cacheScope`, and `serverInfo`; it keeps `initialize` only as a compatibility path for clients that still negotiate the old way, echoes the client's version when supported, and stops depending on `ping` and logging notifications. Evidence: a compatibility test that drives the adapter as a 2024-11-05 client, a 2025-11-25 client, and a 2026-07-28 client.
2. **SDK spike.** The seven tools on `rmcp` stdio behind the existing tool trait, the conformance suite run, and a decision recorded here by test count and diff size.
3. **Team blackboard.** `team/board`, `post_intent`, `read_board`, a scripted teammate on request, roster cap enforced server-side. Evidence: adapter tests and a recorded session of two agents coordinating a push.
4. **A2A** only when cross-host recruitment becomes real.

## Success criteria

- [ ] Compatibility test across the three client eras passes.
- [ ] SDK decision recorded with numbers.
- [ ] Blackboard shipped with a recorded two-agent session.
