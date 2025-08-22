# Acceptance Criteria Updates (Actionable)

Date: 2025-08-19

## H.0/H.1 Pool + Stdio

- Pool return processing
  - Given: A pool with `max_connections = 1` and a persistent stdio server
  - When: Two requests are sent sequentially
  - Then: `idle_connections` increases to 1 after first request; second request reuses the same connection (no new spawn); active returns to 0 after completion.

- Maintenance loop architecture
  - Assert: `ConnectionPool` maintenance loop owns the `mpsc::Receiver<T>` passed at spawn time; no `Arc<Mutex<Receiver<_>>>` remains.
  - Assert: The interval’s first tick is absorbed before entering the select loop.

- Subprocess health
  - Given: A subprocess whose stdout closes after response
  - When: `receive_response()` observes channel close (`None`)
  - Then: `is_connected()` becomes false thereafter; returning such a connection results in it being discarded, not added to idle.

## H.4 SSE Reconnection

- Backoff policy
  - Given: Config with base=200ms, factor=2.0, cap=30s
  - Then: Backoff sequence matches [200ms, 400ms, 800ms, …, 30s], jitter within ±10% full jitter.

- Resilience
  - Given: Flaky upstream dropping every N events
  - Then: Stream resumes and reaches 100 events without duplicate delivery; metrics show reconnection attempts with increasing backoff.

## H.9 Benchmarks

- StdIO path
  - Baseline: Per-request spawn time ~10ms → throughput ~100 rps.
  - Target (persistent server): >1,000 rps with `max_connections=1` and reuse; p95 latency < 20ms on local runner.

