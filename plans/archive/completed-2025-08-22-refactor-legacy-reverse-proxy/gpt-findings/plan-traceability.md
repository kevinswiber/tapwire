# Phase H Plan ↔ Code Traceability

Date: 2025-08-19

## H.0 Fix Connection Pool Leak (implemented)

- Plan path reference: `src/proxy/reverse/upstream/pool.rs` (mismatch)
  - Actual file: `shadowcat/src/proxy/pool.rs`
- Current code status:
  - Maintenance loop owns `mpsc::Receiver<T>` and consumes first interval tick; no `Arc<Mutex<Receiver>>` guards.
  - `PooledConnection::Drop` handles backpressure: on `try_send` error extracts connection, closes with timeout, then decrements active.
  - Idle cleanup avoids awaits while holding locks; last-reference Drop spawns async cleanup backstop.
  - Semaphore uses `OwnedSemaphorePermit` held by `PooledConnection`.
- Plan delta required:
  - Mark H.0 as done; keep regression tests for reuse and return-path safety.

## H.1 Fix Stdio Subprocess Spawning (pending)

- Plan file paths:
  - References `src/transport/subprocess.rs` (mismatch)
  - Actual file: `shadowcat/src/transport/outgoing/subprocess.rs`
- Current code status:
  - Upstream stdio uses pool with factory that calls `Subprocess::connect()` (reuses if pool returns idle).
  - `Subprocess::is_connected()` toggling on stdout EOF not yet verified; optional `try_wait()` child check recommended.
  - Ensure `receive_response()` sets `connected=false` on channel close.
- Plan delta required:
  - Implement “mark disconnected on EOF” and optional child status check.
  - Add a persistent stdio test server to validate actual reuse through the pool.
  - Document non-persistent CLI limitation.

## H.2 Add Server Drop Implementation

- Current code status:
  - `ReverseProxyServer` has no `Drop` implementation; shutdown is handled by serving future end.
  - Connection pool has its own shutdown/Drop handling; server should trigger pool shutdown if owned.
- Plan delta required:
  - Track any background tasks spawned by server; ensure they are cancelled on drop.
  - Integrate pool shutdown in server shutdown path (if not already ensured by app lifecycle).

## H.4 Implement SSE Reconnection

- Plan content: detailed steps, backoff, jitter, tests. Good coverage.
- Code status: SSE files not reviewed in-depth here; plan should specify deterministic-timer tests and jitter bounds explicitly.

## H.5 Request Timeouts

- Plan describes config and per-transport application. Ensure timeouts integrate with retries/deadlines to avoid retry storms.

## File Path and Naming Corrections

- Pool: use `shadowcat/src/proxy/pool.rs`.
- Subprocess: use `shadowcat/src/transport/outgoing/subprocess.rs`.
- Traits: `shadowcat/src/transport/traits.rs`.
