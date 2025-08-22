# Gaps and Missing Items

## Critical (Address in Phase H)

1. Connection Pool Maintenance Loop Ownership (resolved)
   - Status: Implemented. Maintenance loop owns `mpsc::Receiver<T>`; no `Arc<Mutex<Receiver<_>>>` guards remain. First interval tick is consumed.
   - Follow-up: Keep acceptance tests to guard against regressions.

2. Subprocess Health Semantics (still outstanding)
   - Gap: No explicit item to correct `Subprocess::is_connected()`/health when stdout closes or process exits.
   - Impact: Dead connections may be considered healthy; pool may retain unusable entries; reuse fails.
   - Action: Add to H.1: set `connected = false` on stdout EOF/`recv() == None`, optionally check `child.try_wait()` in `is_connected()`.

3. Await-in-Lock Patterns (resolved)
   - Status: Idle cleanup drains under lock, performs async checks/close off-lock, then repopulates.
   - Follow-up: None beyond tests.

4. Documentation of Stdio Pooling Limitations
   - Gap: No explicit doc task calling out that single-shot CLIs (e.g., `echo`) cannot be reused.
   - Impact: Surprising behavior and misaligned expectations; support load may increase.
   - Action: Add to H.10 (or H.1) a note in docs: recommend HTTP or persistent stdio servers for throughput.

5. Practical Memory/Leak Testing Method
   - Gap: Success criteria prescribe `valgrind` which isn’t practical for async Rust in CI/macOS.
   - Impact: False sense of coverage; developers skip the step.
   - Action: Propose Linux CI job using `heaptrack` or `valgrind` Massif + steady-state benchmarks; or leverage `cargo-instruments` (macOS) locally.

## High Priority (Plan Now, Implement Next)

6. Minimal Upstream Health Checks
   - Gap: Health checking is missing; not listed in Phase H even as basic liveness.
   - Impact: Load balancing and circuit-breaking can’t make informed choices.
   - Action: Add a minimal liveness probe task (cheap `is_connected()`/HTTP ping) after H.5.

7. Reconnection Policy Details for SSE / Integration
   - Note: An SSE reconnection module exists under `shadowcat/src/transport/sse/reconnect.rs`. Plan should reuse/integrate rather than re-implement to avoid divergence.
   - Gap: H.4 doesn’t specify backoff strategy, jitter, maximum retries, or test matrix.
   - Impact: Fragile reconnection causing thundering herd or poor UX.
   - Action: Define policy: exponential backoff with jitter, cap, and reset on success; add tests.

8. Benchmark Harness & Targets
   - Gap: H.9 mentions benchmarks but no harness or acceptance thresholds tied to tasks.
   - Impact: Hard to validate perf regressions per change.
   - Action: Add a simple wrk/k6-based script or Rust bench and wire it to compare against baselines, recording p95 latency/throughput.

9. Circuit Breaker Placeholder
   - Gap: Review flags missing circuit breaker; not in Phase H.
   - Impact: Transient upstream failures can cascade under load.
   - Action: Add a placeholder subtask to design/enable a basic breaker (even if parked to Phase I).
