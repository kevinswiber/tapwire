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
  - Representative payload classes
    - Small (~1 KB): initialize/ping-style messages with minimal params and short IDs. Target 0.5–2 KB.
    - Medium (~32 KB): moderate tool responses (arrays of 10–50 small objects). Target 16–64 KB.
    - Large (~256 KB): large tool responses or logs (arrays/maps with nested fields). Target 128–512 KB.
    - Notes: Keep within configured per-message limits; include both request, notification, and response shapes.
  - E2E forward-proxy scenario outlines
    - FP‑S1 (Baseline latency): 1 client; initialize → simple request/response → shutdown. Record p50/p95 latency and CPU.
    - FP‑S2 (Concurrency): 32 clients; steady 200 RPS mixed small/medium payloads for 60s. Record throughput, p95/p99, errors.
    - FP‑S3 (Batch near limit): batches totaling ~0.9 MB; verify batch rejection paths and backpressure behavior.
  - Benchmarks to consider
    - Microbench: JSON encode/decode for `ProtocolMessage`; batched throughput under 1 MB cap across Small/Medium/Large payloads.
    - Microbench: Interceptor decision function with 0/5/20 rules; match vs no‑match cases.
    - End‑to‑end: forward proxy stdio target with interceptor pass‑through; measure p50/p95 latency and CPU.
    - SSE: reconnect latency and event loss under intermittent upstream failures.
    - Recorder: sustained write throughput and memory under backpressure.

- Suggestions and Fixes:
  - Add criterion benches for transport serialization and interceptor decision function.
  - Gate verbose tracing on hot paths behind debug to avoid prod overhead.

- Positive Notes:
  - Clear separation of parsing and envelope construction; structured logging instrumentation present.

- Action Checklist:
  - Representative payload sets (small/medium/large): defined above.
  - Add black‑box E2E perf test using HTTP forward proxy with disabled auth: outline above (FP‑S1..S3).
  - Include SSE resilience scenario with controlled failure injection: included in benchmarks list.

## Candidate criterion benches (skeletons to implement later)

- transport_encode_decode.rs
  - Bench cases: Small/Medium/Large `ProtocolMessage` serialize → parse round‑trip.
  - Metrics: ns/op, bytes/op, allocations/op.
- interceptor_decision.rs
  - Bench cases: 0/5/20 rules; message matches 0/1/N rules.
  - Metrics: ns/op, allocations/op; sensitivity to rule count.
- envelope_build.rs
  - Bench cases: build `MessageEnvelope` from raw JSON and context.
  - Metrics: ns/op; ensure buffer reuse path is covered.
