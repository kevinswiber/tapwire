# Nanobot (Go) vs Shadowcat (Rust) — MCP Gateway/Proxy Comparison

## Overview
- Purpose: Compare Nanobot’s Go-based MCP gateway/reverse proxy with Shadowcat’s Rust MCP proxy and in-progress `crates/mcp` implementation.
- Focus: Transports, sessions, auth, resilience, WebSocket alignment, and practical patterns to borrow.

## Transport Architecture
- Shadowcat
  - Message-level abstraction: `Sink<Value>/Stream<Result<Value>>` across transports.
  - Line transports: `tokio_util::Framed + JsonLineCodec` for stdio/subprocess; planned worker-backed HTTP JSON/SSE.
  - WebSocket: separate, feature-gated transport (GET+Upgrade) per SEP-1287; sessions in data per SEP-1364 alt 2.
  - Proxy API: Incoming/Outgoing/Bidirectional traits; dedicated session/SSE components.
- Nanobot
  - HTTP JSON + SSE in a single `HTTPClient`:
    - POST JSON-RPC to server; on streaming/initialize failures, starts SSE (`Accept: text/event-stream`).
    - First SSE data payload is treated as a URL to use for subsequent POSTs (vendor convention).
    - Maintains `Mcp-Session-Id` header; reinitializes on session-not-found.
    - Integrates OAuth: on 401, performs auth flow and retries.
  - Stdio transport available.
  - No generalized sink/stream abstraction; logic embedded in the HTTP client.

## Session Model
- Shadowcat: In-memory store + persistence worker; LRU/expiry; SSE integration; plan to enforce WS session rules (required; single connection); protocol version negotiation centralized.
- Nanobot: DB-backed session store (GORM); overlays in-memory cache; syncs attributes; clears `sessionId` and re-initializes on 404/session-not-found.

## Auth & Security
- Shadowcat: Auth utilities and retry header parsing; plans for OAuth and authorized SSE/WS paths.
- Nanobot: Built-in OAuth (callbacks, token storage), robust handling of oauth2 client errors (refresh), and retry.

## Resilience & Backpressure
- Shadowcat: Planned workerization for HTTP with bounded queues; SSE reconnect (Last-Event-ID) and backoff with jitter; codec hardening; rate-limit header parsing.
- Nanobot: “Ensure SSE” loop guarded by locks; resets initialize and resends; simple SSE parsing (bufio.Scanner); no explicit backpressure channels.

## WebSocket
- Shadowcat: WS as a distinct transport per SEP-1287; sessions required; `sessionId` in data; subprotocol auth; single-connection-per-session.
- Nanobot: No WS transport found; relies on HTTP JSON + SSE.

## Versioning
- Shadowcat: Explicit version module; supports 2025-03-26 and 2025-06-18; draft planned.
- Nanobot: Initialize handling correct but version negotiation not clearly centralized.

## Proxy Features
- Shadowcat: Recording/replay, rate limiting, retry, session management, transport factories, typed errors, compliance framework.
- Nanobot: Reverse TLS client (local TCP → remote TLS) for sandboxing, runtime/agents system, session-backed UI, broader integrations (LLM/chat).

## What Shadowcat Can Borrow From Nanobot
- OAuth lifecycle integration:
  - 401 with `WWW-Authenticate` → run OAuth flow → swap in authenticated client → retry original request once.
  - Detect oauth2 client errors (e.g., refresh failures) → reset to unauthenticated client → resend to trigger 401 path.
- Session reinitialization:
  - On session-not-found (404/semantic), clear `sessionId`, re-run initialize, then emit `notifications/initialized`.
- Optional endpoint discovery compatibility:
  - Support an optional mode that reads the first SSE data payload as a POST `message_url` (with strict validation; off by default).
- Practical `Accept` header strategies:
  - POST: `Accept: application/json, text/event-stream`; SSE GET: `Accept: text/event-stream`.

## What Nanobot Could Borrow From Shadowcat
- Workerization + backpressure: background task owning HTTP client; bounded queues for requests/responses; clear async boundaries.
- WebSocket per SEP-1287/SEP-1364: better support across intermediaries/cloud providers; enforce sessions + single connection.
- Message-level abstraction for transports and testing (sink/stream façade).
- Centralized version negotiation and hardened line codecs (max length, discard mode, CRLF handling, ignore policy for non-standard notifications).

## OAuth & Recovery Checklist (Extracted from Nanobot)
- 401 + `WWW-Authenticate` present → run OAuth flow → swap in authenticated client/headers → retry original request exactly once.
- OAuth client-layer errors (e.g., refresh failures) → detect in error chain (prefix `oauth2:`) → reset to unauthenticated client → resend to trigger 401 → OAuth.
- Session not found (e.g., 404 + semantic) → clear cached `sessionId` → re-run initialize → send `notifications/initialized`.
- Initialize fallback: if POST initialize fails with streaming-required semantics, attempt SSE-first path before retrying initialize.
- State guards: protect `sessionId` and cached initialize message with a mutex; atomic updates on success.
- SSE resilience: maintain reconnect-needed flag; on SSE EOF/error, reconnect with Last-Event-ID; background reader dispatches messages; backoff with jitter.

## “SSE Endpoint Discovery” (Vendor Convention, Not MCP Spec)
- Definition: Some servers emit a URL via SSE (often the first data payload) that the client uses as the POST endpoint for JSON-RPC (Nanobot pattern). This is not part of the MCP spec nor rmcp.
- Rationale: Enables dynamic, session-scoped POST paths (sticky/sharded) that fare better through intermediaries/CDNs than a static base path.
- Safe, optional integration for Shadowcat (off by default):
  - On SSE connect, read the first SSE data payload. If it’s a valid URL on the same origin (or allowlisted), set it as `message_url` for subsequent POSTs.
  - Don’t rely on `event: endpoint`; validate payload as URL; timebox waiting; fallback to configured POST URL if absent/invalid.
  - Protect `message_url` with a mutex; optionally accept later rotations.

## Shadowcat HTTP Worker Hooks (Proposed)
- OAuth handler trait: on 401, mint authenticated client/headers and retry once; detect oauth2 client errors and reset to unauthenticated client.
- Session lifecycle: on session-not-found, clear `sessionId`, reinitialize, send `notifications/initialized`.
- SSE connect/reconnect: GET with `Accept: text/event-stream`, manage Last-Event-ID, apply backoff with jitter; background reader loop.
- Optional endpoint discovery compatibility (off by default): apply strict URL validation; same-origin restriction unless allowlisted.
- Backpressure: bounded request/response queues; clear behavior under saturation (block or error); observability hooks.
- Telemetry: metrics for auth retries, session reinitializations, SSE reconnects, endpoint discoveries, queue saturation; trace spans on initialize/SSE/auth.

## Recommendations
- Shadowcat
  - Land workerized HTTP JSON/SSE with bounded queues and background client receiver; integrate OAuth + recovery (401, oauth2 client errors).
  - Keep WS as a separate transport per SEP-1287 (sessions required; single connection; subprotocol auth; `sessionId` in data) to handle intermediary constraints.
  - Consider optional endpoint discovery compatibility for interoperability; keep disabled by default and validate strictly.
- Nanobot
  - Add WS transport per SEP-1287/SEP-1364 (sessions in data; single connection per session).
  - Introduce worker/backpressure patterns for HTTP/SSE; centralize version negotiation; harden the SSE/codec layer.

## References
- Nanobot code inspected: `pkg/mcp/httpclient.go`, `pkg/mcp/stdio.go`, `pkg/mcp/serversession.go`, `pkg/session/*`, `pkg/reverseproxy/*`.
- Shadowcat plan: `plans/mcp-compliance-check/*` and `shadowcat-mcp-compliance/*` modules (transport, session, mcp, retry).
- Specs: SEP-1287 (WebSocket transport), SEP-1364 (session in data, alt 2), MCP protocol versions 2025-03-26 / 2025-06-18.

— End —
