## C.2 — Proxy engine and session lifecycle (Phase C)

Scope: `shadowcat-cursor-review@eec52c8`

### Summary
- Forward proxy uses task aborts; add cooperative shutdown and join-with-timeout.
- Initialize request tracking and negotiation flow are solid; document TTL and bounds.
- Interceptor effects need explicit behavior mapping per action.
- Recording should propagate accurate `TransportContext` instead of defaulting to stdio in some paths.
- Reverse proxy metrics currently mix atomics and a Mutex; consider lock-free accumulation for p50/p99 endpoints.

### Key citations
- Forward proxy shutdown and task aborts:
```651:659:shadowcat-cursor-review/src/proxy/forward.rs
pub async fn shutdown(&mut self) { /* send shutdown */ for task in self.tasks.drain(..) { task.abort(); } }
```
```682:687:shadowcat-cursor-review/src/proxy/forward.rs
impl Drop for ForwardProxy { fn drop(&mut self) { for task in &self.tasks { task.abort(); } } }
```

- Transport closure must be awaited exactly once during shutdown:
```474:476:shadowcat-cursor-review/src/transport/http.rs
handle.abort();
```

- Initialize request tracking and negotiation:
```269:349:shadowcat-cursor-review/src/proxy/forward.rs
// Tracks initialize requests with TTL and MAX_TRACKED_REQUESTS, feeds VersionNegotiator on response
```

- Recording and interceptor flow:
```521:623:shadowcat-cursor-review/src/proxy/forward.rs
// Records to SessionManager and TapeRecorder, then applies InterceptorChain (Continue/Modify/Block/Mock/Pause/Delay)
```

- SessionManager recording uses a default stdio context here:
```835:842:shadowcat-cursor-review/src/session/manager.rs
let context = MessageContext::new(
    session_id,
    direction,
    TransportContext::stdio(), // Default context - could be improved
);
```

- Reverse proxy constructs HTTP contexts for SSE/HTTP paths:
```734:742:shadowcat-cursor-review/src/proxy/reverse.rs
let context = MessageContext::new(
    &session.id,
    MessageDirection::ClientToServer,
    TransportContext::http("POST".to_string(), "/mcp/v1/sse".to_string()),
);
```

- Reverse proxy metrics with Mutex accumulation and exposition endpoint:
```318:361:shadowcat-cursor-review/src/proxy/reverse.rs
struct ReverseProxyMetrics { /* AtomicU64s + Mutex<Duration> */ }
```
```1253:1334:shadowcat-cursor-review/src/proxy/reverse.rs
async fn handle_metrics(...) -> impl IntoResponse { /* builds Prometheus text output */ }
```

### Proposals
- Cooperative shutdown for forward proxy
  - Introduce `with_shutdown(token)` or store a `CancellationToken` and have readers/writers exit on signal.
  - In `shutdown()`, send signal, then `join` readers/writers with a small timeout; only `abort()` if unresponsive.
  - Ensure transports’ `close()` is called exactly once and awaited.

- Interceptor effects mapping (make behavior explicit)
  - Continue: forward message; record original frame; annotate metadata `intercepted=false`.
  - Modify: forward modified message; record both original and modified with linkage metadata (IDs and frame_id); propagate modified message downstream.
  - Block: for requests, synthesize an error response and emit on the opposite channel; record block event in session (with reason). For notifications, drop and record a block event.
  - Mock: send provided response on opposite channel; do not forward original; record mock with linkage to request id.
  - Pause: enqueue and await `resume` with bounded timeout; on timeout, synthesize error; document API for resumption hook.
  - Delay: sleep for specified duration before forwarding; document upper bounds and cancellation behavior.

- Accurate `TransportContext` in recordings
  - Where proxy knows the transport type of the source (reader context has `transport_type`), build `TransportContext` accordingly when recording via `SessionManager` to avoid the default `stdio()` placeholder. This maintains correct edge metadata for analytics and replay.

  Example sources:
  - Forward proxy reader paths often know direction and transport (see construction sites using `TransportContext::stdio()` in `forward.rs` around 1010, 1176, 1188, 1451, 1473). Replace with the actual transport context from the reader side.
  - Reverse proxy already uses HTTP contexts in several spots; ensure parity where it currently defaults to stdio (e.g., `1020`, `1091`).

- Metrics ergonomics
  - Replace `Mutex<Duration>` accumulation with an atomic bucketed histogram or lock-free duration accumulator (e.g., `AtomicU64` nanoseconds) to avoid lock contention on `/metrics`.

- Lifecycle states and observability
  - Define states: starting → running → draining → shutdown. Export counters for each state and per-direction message counts (C→S, S→C). Add a “draining” gauge when shutdown is initiated but queues are not empty.

### Action checklist (C.2)
- Add shutdown token plumbing and join-with-timeout guidance to docs.
- Define an explicit mapping for interceptor actions; update docs with expected proxy behavior and recording rules.
- Document how to construct accurate `TransportContext` for recordings from proxy readers.
- Note metrics improvement and state counters for observability in docs.
- Ensure single transport `close()` invocation and await during shutdown.

### Addendum (Delta)
Delta findings against `shadowcat-delta@b793fd1` will be appended here (shutdown sequencing, interceptor effects, recording context accuracy), preserving existing `eec52c8` citations.
