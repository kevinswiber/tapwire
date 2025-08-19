# Phase H Plan ↔ Code Traceability

Date: 2025-08-19

## H.0 Fix Connection Pool Leak

- Plan path reference: `src/proxy/reverse/upstream/pool.rs` (mismatch)
  - Actual file: `shadowcat/src/proxy/pool.rs`
- Current code status:
  - `PooledConnection::Drop` uses `try_send` and spawns cleanup on failure (non-blocking). Good.
  - Semaphore uses `OwnedSemaphorePermit` held by `PooledConnection` (good).
  - Return channel is bounded with configurable multiplier (good).
  - Maintenance loop still locks `Arc<Mutex<mpsc::Receiver<T>>>` and holds guard across awaits (needs change).
  - `cleanup_idle_connections` awaits while holding `idle_connections` lock (could be improved).
- Plan delta required:
  - Explicitly require moving `Receiver<T>` ownership into the maintenance task.
  - Add acceptance checks for return processing and idle reuse.

## H.1 Fix Stdio Subprocess Spawning

- Plan file paths:
  - References `src/transport/subprocess.rs` (mismatch)
  - Actual file: `shadowcat/src/transport/outgoing/subprocess.rs`
- Current code status:
  - Upstream stdio uses pool with factory that calls `Subprocess::connect()` (reuses only if pool returns idle).
  - `Subprocess::is_connected()` returns a bool field; not updated on stdout EOF; no `try_wait()` child check.
  - `receive_response()` does not set `connected=false` on channel close.
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

