# Recommendations to Improve Phase H

## Task-Level Additions/Edits

- H.0 Fix Connection Pool Leak
  - Add: Refactor maintenance loop to own `mpsc::Receiver<T>` (remove `Arc<Mutex<Receiver<_>>>`).
  - Add: Remove awaits while holding `idle_connections` lock; collect and then act.
  - Acceptance: Return channel drain verified via test; `idle_connections` increases after first request; active/total counters stable; no deadlocks in stress test.

- H.1 Fix Stdio Subprocess Spawning
  - Add: Update `Subprocess` to set `connected = false` on stdout EOF / `recv()==None`; optionally check `child.try_wait()` in `is_connected()`.
  - Add: Document stdio pooling limitation for single-shot CLIs and recommend persistent servers/HTTP for throughput.
  - Acceptance: With a persistent test server and `max_connections=1`, N requests spawn exactly one process and reuse; with a single-shot CLI, pool does not reuse and does not leak.

- H.2 Add Server Drop Implementation
  - Clarify: Ensure server shutdown signals pool shutdown, drains return channel briefly, and cleans up sessions/SSE if applicable.
  - Acceptance: Shutdown test confirms maintenance task stops, channels closed, idle cleared, and process handles cleaned up.

- H.4 Implement SSE Reconnection
  - Add: Prefer integrating `transport::sse::reconnect` module; avoid re-implementing reconnection logic in reverse stream layer.
  - Add: Policy — exponential backoff (base 100–500ms), full jitter, max backoff 30s, max attempts infinite with cap per session; reset on successful message.
  - Add: Tests — deterministic timer control; verify no thundering herd (staggered starts), idempotent re-subscription, and recovery from transient 5xx.
  - Acceptance: Reconnects within policy bounds; no duplicate subscriptions; steady CPU under churn.

- H.5 Add Request Timeouts
  - Add: Separate connect, request, and response timeouts with sensible defaults and per-upstream overrides.
  - Acceptance: Hanging upstreams time out and surface typed errors; retries (if enabled) respect overall deadline.

- H.9 Performance Benchmarks
  - Add: Lightweight script (wrk/k6 or Rust microbench) measuring p50/p95 latency and throughput for stdio and HTTP paths with/without pool reuse.
  - Acceptance: Targets in Phase H success criteria are validated on CI Linux runner; artifacts stored for diffing.

## Validation Improvements

- Replace `valgrind` command in docs with:
  - CI (Linux): `heaptrack` or `valgrind --tool=massif` during steady-state load.
  - Local (macOS): `cargo-instruments` or `leaks` for spot checks.

## Documentation

- Add a short “Transport Compatibility” note: stdio pooling requires persistent servers; single-shot tools will not be reused.
- Add “Operational Runbook” entries: how to read pool stats, common failure modes, and tuning knobs (`max_connections`, `return_channel_multiplier`).
