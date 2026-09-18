# Plan: public server hardening

**Status:** planned (2026-09-18)
**Branch:** `feat/hardening-*` (one PR per rung)
**Spend:** $0 for everything here. A VM for the stranger test needs written approval and sits under the cap.

## Goal

Phase 2 of [`../ROADMAP.md`](../ROADMAP.md): a server you can open to the internet on port 6767 (or behind TLS on 443) that strangers and their agents can join without it falling over, plus the status endpoint and benchmark mode the 1.0 bar needs. This plan records the research of 2026-09-18 as concrete settings and crates.

## Non-goals

- Accounts, identity, or matchmaking. Tickets prove a join was issued, not who a player is.
- A web export. Godot Web is out of scope for the project; the ticket travels in the first frame.
- A metrics stack. The status JSON is enough until something scrapes it.

## Design

1. **Frame and buffer caps.** `tokio-tungstenite` takes a `WebSocketConfig` through `accept_async_with_config`. Defaults are 64 MiB per message and 16 MiB per frame with an unlimited write buffer. Set `max_message_size` to 64 KiB, `max_frame_size` to 64 KiB, and `max_write_buffer_size` between 256 KiB and 1 MiB; a full write buffer surfaces as an error, which is the cue to drop a slow reader. The pinned 0.24 exposes plain fields; the current 0.30 makes the struct non-exhaustive with builder setters and adds a read buffer size, so the upgrade is its own PR.
2. **Accept hardening.** Wrap the handshake in a five second `tokio::time::timeout` (slow-loris); cap connections per IP with a `DashMap` and total connections with a `tokio::sync::Semaphore`; rate-limit accepts per IP with `governor` (a keyed limiter with a `DashMap` store, `retain_recent` on a timer to bound memory); ping and drop connections idle beyond a set number of seconds. Tower middleware does not fit a raw accept loop; the hand-rolled token bucket is the fallback.
3. **Join tickets.** An HMAC-SHA256 ticket, `base64url(json).base64url(mac)`, carrying arena, name, role, nonce, and a sixty second expiry, compared in constant time (`hmac`, `sha2`, `subtle`). The ticket travels in the first frame after the upgrade. A separate rejoin token serves reconnect. Origin checked on upgrade.
4. **Names and roles.** Three to sixteen characters, NFKC normalised, allowlist letters, digits, underscore, and hyphen; reserved names such as server and admin; duplicates get a suffix. Admin is gated on a token, never on a name, since without an auth server anyone can take any name.
5. **Speak and action limits.** Speak keeps the shipped 60-tick cooldown from `docs/protocol.md` (mirrored in the adapter); this plan adds kick-on-repeat and an action-rate cap per session; junk dropped silently. A ban list file with id, IP, reason, and expiry, plus an allowlist, in the style of Veloren's `banlist.ron`.
6. **TLS.** Terminate at Caddy: `reverse_proxy localhost:6767` upgrades WebSockets automatically, `stream_close_delay 5m` survives reloads, certificates by DNS-01 (no open port, works behind NAT, DuckDNS or a DNS plugin) for a home box or HTTP-01 on a VM. The client connects with `connect_to_url("wss://play.example", TLSOptions.client())` by name, never by IP. No in-process rustls; issuance and reload would be extra code for no gain.
7. **Status over HTTP.** The status line and `--bench N M` are built by playtest rung 3 in Phase 1. This rung serves the same JSON to a plain `GET /status` on the game port, answered from the WebSocket upgrade callback before any upgrade, so no HTTP crate is added; a `status` WebSocket message mirrors it for Godot. Percentiles come from `hdrhistogram`, the one new crate here. A Prometheus `/metrics` endpoint waits until something scrapes it.
8. **Protocol version.** `protocol_version` in `Hello`, rejection with a clear code and the supported list. The MCP side is covered in [`agent-door-2026.md`](./agent-door-2026.md).

## The cheapest sane public setup

Plaintext on port 6767 stays the LAN path and the documented public path, as the README and hosting guides say. The TLS option is a small VM with the firewall open only on 22, 80, and 443, fragr bound to loopback, Caddy in front, or a home box with DuckDNS, DNS-01 certificates, and only 443 forwarded. Both go in `infra/` once proven.

## Verification

- A test for every reject path (oversize frame, bad ticket, expired ticket, bad name, rate limit, idle, per-IP cap).
- Property tests over the wire parser in `server/src/tests.rs` on stable Rust (cargo-fuzz needs nightly and stays a local extra with a committed corpus if ever used).
- A flood test against a local server with the caps on, watching the status JSON stay sane.
- The status and benchmark JSON documented in `docs/protocol.md`.
- The stranger session from the roadmap, recorded, with the hosting guide updated from what went wrong.

## Rungs

1. Frame and buffer caps, handshake timeout, idle timeout, per-IP and global caps. Tests. New crates: none (`DashMap` only if the per-IP map outgrows a mutexed `HashMap`).
2. Accept rate limiting and kick-on-repeat with `governor`; ban and allow lists. New crate: `governor`.
3. Join tickets and rejoin tokens; protocol version in `Hello`. New crates: `hmac`, `sha2`, `subtle`.
4. Status over HTTP on the game port and the Godot status message. New crate: `hdrhistogram`.
5. Caddy guide and the wss client path; the stranger session.

## Success criteria

- [ ] Rung 1 shipped with reject-path tests.
- [ ] Rate limits and lists shipped.
- [ ] Tickets and protocol version shipped.
- [ ] Status and benchmark mode shipped and documented.
- [ ] A recorded stranger session behind TLS.
