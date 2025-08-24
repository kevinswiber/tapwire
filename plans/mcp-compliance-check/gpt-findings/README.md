**Executive Summary**
- **Keep message-level transports**: Use `Sink<Value>/Stream<Result<Value>>` for all transports; line-delimited transports use `tokio_util::Framed + JsonLineCodec`.
- **HTTP JSON/SSE**: Model as one adaptive transport but back it with a worker task that owns the HTTP client, request queue, SSE reconnect, and response multiplexing.
- **WebSocket (SEP-1287)**: Implement as a separate, feature-gated transport with GET+Upgrade, subprotocol-based auth, required sessions, `sessionId` in every message (SEP-1364 alt 2), and single-connection-per-session enforcement.
- **Client fix**: Add an internal background receiver so `request()` works without calling `run()`; provide clean shutdown.
- **Harden codecs + versioning**: Improve `JsonLineCodec` robustness; centralize version negotiation; consider minimal role typing to avoid direction mixups.

**Why This Direction**
- **Simplicity**: Direct `Sink/Stream` is idiomatic and easy to test; keeps ergonomics while integrating with futures ecosystem.
- **Correctness**: Workerized HTTP avoids doing async in `poll_*`, handles SSE reconnection and backpressure, and enables session lifecycle.
- **Spec Alignment**: WS as a distinct transport matches SEP-1287; sessions in the data layer match SEP-1364 alt 2; single-connection-per-session keeps state simple and reliable.
- **Interoperability**: Clear separation enables immediate compatibility matrix work and reuse of hardened patterns validated by rmcp.

**Key Issues Found**
- **Conflated WS in HTTP**: Current `HttpTransport` treats 101 Upgrade as POST response mode; must be split into a dedicated WS transport.
- **Async in `poll_flush`**: `HttpTransport` enqueues and fakes responses; needs a worker that performs actual HTTP I/O and emits responses/events.
- **SSE not integrated**: SSE module exists but is not wired; add common/request-scoped streams, reconnection (Last-Event-ID), and backoff.
- **Client deadlock risk**: `request()` awaits a oneshot but nothing drives `transport.next()` unless `.run(self)` is spawned; add internal receiver task.
- **Codec robustness**: Improve CRLF handling, overlong line discard, and tolerance for non-standard notifications.

**Prioritized Actions**
- **High**
  - Split out `WsTransport` (feature-gated): GET+Upgrade, text frames, subprotocol auth, sessions required, `sessionId` in every message, single-connection-per-session.
  - Add HTTP worker: bounded request/response queues, SSE integration, retry/backoff with jitter, session lifecycle (optional for HTTP).
  - Fix client concurrency: background receiver and shutdown handle; ensure no request/receive deadlocks.
- **Medium**
  - Harden `JsonLineCodec`; expand tests; optional ignore policy for non-standard notifications.
  - Wire version negotiation through client/server; add minimal role tagging for safety.
  - Enforce “single WS connection per session” in session manager; add reconnection continuity checks.
- **Low**
  - Replace subprocess test target; add channel transport tests for backpressure and fairness; centralize retry/backoff utilities.

**Decision Points**
- **WS timeline**: Implement now behind a feature flag (recommended) vs. defer until SEP merges; tests gated by config.
- **Role typing scope**: Minimal tags now for safety vs. defer until compliance runner stabilizes.
- **Interop**: Add an optional adapter in the compliance crate to consume rmcp transports for matrix testing without coupling core crates (recommended).

**Risks And Mitigations**
- **Scope creep**: Workerization and WS add surface area; mitigate by feature gating WS and keeping worker APIs internal.
- **Spec drift**: SEP-1287/1364 details may change; mitigate by concentrating WS specifics in a module with a thin `Sink/Stream` boundary and tests that mirror the SEP behaviors.
- **Concurrency bugs**: Introduce bounded channels, clear ownership, and cancellation tokens; add targeted tests for shutdown and backpressure.

**References**
- Detailed analysis: `plans/mcp-compliance-check/gpt-findings/findings.md`
- SEP-1287 WebSocket: sessions required, subprotocol auth, single-connection-per-session
- SEP-1364 alt 2: `sessionId` in data layer

If you want, I can next draft skeletons for the HTTP worker and the WS transport to make the integration path concrete.

