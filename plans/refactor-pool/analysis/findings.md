# Findings (2025-08-19)

- Implemented a fresh, transport-agnostic pool in `src/pool` on branch `refactor/pool` (worktree: `shadowcat-connection-pooling`).
- Public API: `PoolOptions`, `Pool<T>`, `PoolConnection<T>`, `PoolStats`, and `traits::PoolableResource`.
- Behavior:
  - Acquire uses semaphore + timeout; reuses healthy idle else creates via factory.
  - Weak-backed maintenance with first-tick absorption; periodic cleanup of idle/expired.
  - `close().await` marks closed, stops maintenance, drains and closes idle.
  - Drop (last ref) provides best‑effort idle cleanup (notify; await maintenance up to 5s; close idle).
  - Fair capacity release: permit is released only after resource is requeued to idle.
- Tests added cover reuse, close semantics, idle timeout cleanup, and fairness.

Open items / future enhancements:
- Close event helper implemented: `Pool::close_event()` and acquire now cancels
  promptly when close starts (sqlx-style behavior). Added unit test to verify
  pending acquires resolve with error after `close()` begins.
- Health hooks implemented (SQLx-style):
  - `after_create` for new resources, fail acquire on error.
  - `before_acquire` for idle resources, return false/Err to close-and-retry.
  - `after_release` on drop, false/Err closes instead of requeue.
  Includes `PoolConnectionMetadata { age, idle_for }` and examples.
- (Optional) Lock-free idle queue + atomic counters if profiling shows contention.

Pilot integration started: added adapter and `process_via_stdio_pooled_v2` for
stdio upstream using the new pool (keeping existing paths intact for now).
