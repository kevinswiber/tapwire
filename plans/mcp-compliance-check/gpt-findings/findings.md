MCP Compliance Architecture Findings (Aug 2025)

Scope
- Reviewed plan docs (CURRENT-ARCHITECTURE, transport v2, HTTP unified, decision log).
- Compared to rmcp (~/src/modelcontextprotocol/rust-sdk) transport patterns and lifecycle.
- Reviewed code in:
  - `shadowcat-mcp-compliance/crates/mcp`
  - `shadowcat-mcp-compliance/src/{transport, session, mcp, retry}`
- Updated WebSocket alignment to SEP-1287 + SEP-1364 alt 2.

High-Level Summary
- **Use message-level transports**: `Sink<Value>/Stream<Result<Value>>` is the right abstraction.
- **Line transports**: `Framed + JsonLineCodec` are appropriate for stdio/subprocess/TCP/Unix.
- **HTTP**: Adaptive JSON+SSE is correct, but needs a worker owning HTTP I/O, queues, SSE reconnect, and response multiplexing.
- **WebSocket**: Must be a separate, feature-gated transport (GET+Upgrade), not an HTTP response mode.

Comparison With rmcp
- rmcp: custom `Transport<Role>` + `IntoTransport` adapters (AsyncRead/Write, Sink/Stream, worker, child-process, SSE, streamable HTTP) with role-typed messages and workerized lifecycles.
- Our plan: direct `Sink/Stream` keeps simplicity; incorporate rmcp lessons: workerization, `Arc<Mutex>` for concurrent sends, hardened codecs, session lifecycle.

WebSocket Alignment (SEP-1287 + SEP-1364 alt 2)
- Handshake: HTTP GET + `Connection: Upgrade`, `Upgrade: websocket`, `Sec-WebSocket-Key`, `Sec-WebSocket-Version`.
- Sessions: REQUIRED. `sessionId` moves into the data layer (every MCP message includes it over WS).
- Single connection per session: server MUST close older connections; client should reconnect and resume.
- Auth: via WebSocket subprotocol (Sec-WebSocket-Protocol), not headers/cookies.
- Transport shape: separate `WsTransport` as `Sink/Stream<Value>` with ping/pong, idle timeouts, message size limits, and backpressure handling.

Code Review: crates/mcp
- transport/http/mod.rs
  - Conflates WS with HTTP POST response mode (Status 101). Action: split WS into its own module; correct GET+Upgrade handshake.
  - Async in `poll_flush`: placeholder that enqueues into `single_responses`. Action: add worker task; `Sink` enqueues requests to worker; `Stream` drains worker responses/events.
  - SSE not wired: Action: integrate `streaming::sse` for common/request-scoped streams; reconnect with Last-Event-ID and backoff.
  - Sessions: Not modeled. Action: optional for HTTP; explicit in WS.
- transport/http/streaming/sse.rs
  - Good base: parser, backoff with jitter, last-event-id tracking. Action: integrate into HTTP worker and surface errors/metrics.
- transport/codec.rs
  - Basics OK. Action: CRLF handling, overlong-line discard mode, optional skip for non-standard notifications; expand tests.
- transport/stdio.rs, transport/subprocess.rs
  - Framed usage OK. Action: concurrency wrapping for sinks at service layer; fix subprocess test (don’t use `echo` for JSON).
- client.rs
  - Concurrency: `request()` can block forever unless `.run(self)` drains the stream (and it consumes `self`). Action: spawn background receiver in constructor or `connect()`, provide shutdown, ensure `request/notify` work alone.
  - Versioning: wire initialize + negotiation to `version` module.
- version.rs
  - Versions present: 2025-03-26, 2025-06-18. If ‘draft’ required by plan, add and encode differences.

Code Review: shadowcat-mcp-compliance/src
- transport/traits.rs, transport/mod.rs
  - Custom traits are fine for proxy; plan adapters to crates/mcp `Sink/Stream` to avoid duplication.
- transport/http.rs (utils)
  - Header-based session handling works for HTTP, diverges from WS rules. Action: add comment about SEP-1364 alt 2 and do not apply header expectations to WS.
- session/*
  - Solid: LRU, expiry, persistence worker pattern. Action: enforce WS single-connection-per-session; add reconnection continuity checks and metrics.
- mcp/mod.rs
  - Protocol hub is good. Action: centralize version negotiation here and have transports/clients consult; align error types.
- retry/http.rs
  - Useful Retry-After and rate-limit parsing. Action: apply in HTTP worker for backoff pacing.

Workerization Plan (HTTP)
- Worker owns HTTP client, session state, retry policy, and SSE streams.
- Public `Sink` enqueues requests to a bounded mpsc; public `Stream` reads from a bounded response channel.
- Reconnection: Exponential backoff + jitter; Last-Event-ID for SSE; resume general stream; log and surface retry events.
- Backpressure: bounded queues; explicit behavior (block/drop/return error) under pressure; observability hooks.

Client Concurrency Plan
- Spawn a background task on `Client::new/::connect` that continuously reads from `transport.next()` and routes messages to pending request channels or handler callbacks.
- Keep a shutdown handle; ensure graceful close; make `request/notify` ergonomic.

Prioritized TODOs
- High
  - Create `ws` transport per SEP-1287/SEP-1364; gate by feature; tests for session enforcement and reconnection.
  - Implement HTTP worker; remove async from `poll_*`; integrate SSE module; support optional sessions and delete-session on drop where applicable.
  - Fix client background receiver and shutdown.
- Medium
  - Harden `JsonLineCodec`; add tests for CRLF, overlong discard, malformed lines, and ignore policy.
  - Introduce minimal role tagging; wire version negotiation across client/server.
  - Session manager: enforce single WS connection per session and reconnection continuity; add tests.
- Low
  - Replace subprocess JSON test; add channel transport tests for backpressure/fairness; centralize retry/backoff utilities (HTTP, SSE, WS).

Recommendation
- Proceed with message-level `Sink/Stream` transports and Framed for line protocols.
- Keep HTTP adaptive for JSON+SSE, but implement WS as a separate, feature-gated transport (GET/Upgrade, subprotocol auth, sessions required, single connection per session, `sessionId` in every message).
- Add an HTTP worker and a client background receiver to resolve correctness and concurrency risks.
- Harden codecs and versioning; consider minimal role typing for safety.
- Optionally add a compliance-crate adapter to interop with rmcp transports for matrix testing without coupling core.

