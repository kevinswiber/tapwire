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
- (Optional) Close event helper to cancel waiters predictably (sqlx-style).
- (Optional) Health hooks (before/after acquire) if needed by consumers.
- (Optional) Lock-free idle queue + atomic counters if profiling shows contention.

Integration is intentionally deferred; existing proxy still uses old pool. Migration will follow once API is locked.
