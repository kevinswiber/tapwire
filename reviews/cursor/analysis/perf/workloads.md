# Phase A.3 — Hot Paths & Workloads Inventory

Short template per CURSOR_RUST_CODE_REVIEWER.md.

- Summary:
  - Identified expected workloads and hot paths across transports, proxying, recording, and interception.

- Critical Issues:
  - None at this stage.

- Observations:
  - Expected workloads
    - **Forward proxy (stdio↔HTTP)**: bursty request/response pairs; small JSON payloads (0.5–50KB); p95 fan‑out minimal.
    - **Reverse proxy (HTTP)**: concurrent client requests; auth validation; potential SSE streaming.
    - **Recorder**: append‑only writes per frame; occasional flush; replay reads sequentially.
    - **Interceptor chain**: per‑frame evaluation; likely small rulesets initially, growth over time.
  - Hot paths
    - Transport parsing/serialization and IO:
      ```339:396:shadowcat-cursor-review/src/transport/stdio.rs
      async fn receive(&mut self) -> TransportResult<MessageEnvelope> { /* parse, batch reject */ }
      ```
      ```238:266:shadowcat-cursor-review/src/transport/stdio.rs
      fn serialize_message(&self, msg: &ProtocolMessage) -> TransportResult<String>
      ```
    - Forward proxy read/process/write pipeline:
      ```269:409:shadowcat-cursor-review/src/proxy/forward.rs
      async fn read_messages_with_tracking(...)
      ```
      ```416:510:shadowcat-cursor-review/src/proxy/forward.rs
      async fn read_messages_with_version_negotiation(...)
      ```
      ```512:623:shadowcat-cursor-review/src/proxy/forward.rs
      async fn process_message(...)
      ```
      ```625:649:shadowcat-cursor-review/src/proxy/forward.rs
      async fn write_messages(...)
      ```
    - Session manager metrics and cleanup cadence:
      ```350:359:shadowcat-cursor-review/src/session/manager.rs
      self.metrics.cleanup_runs.fetch_add(1, ...);
      ```
    - SSE reconnect/backoff handling:
      ```213:338:shadowcat-cursor-review/src/transport/sse/reconnect.rs
      use tracing::{debug, error, info, instrument, trace, warn};
      ```
  - Benchmarks to consider
    - Microbench: JSON encode/decode for `ProtocolMessage`; batched throughput under 1MB max size.
    - End‑to‑end: forward proxy stdio target with interceptor pass‑through; measure p50/p95 latency and CPU.
    - SSE: reconnect latency and message loss under intermittent upstream failures.
    - Recorder: sustained write throughput and memory under backpressure.

- Suggestions and Fixes:
  - Add criterion benches for transport serialization and interceptor decision function.
  - Gate verbose tracing on hot paths behind debug to avoid prod overhead.

- Positive Notes:
  - Clear separation of parsing and envelope construction; structured logging instrumentation present.

- Action Checklist:
  - Define representative payload sets (small/medium/large) for microbenches.
  - Add black‑box E2E perf test using HTTP forward proxy with disabled auth.
  - Include SSE resilience scenario with controlled failure injection.
