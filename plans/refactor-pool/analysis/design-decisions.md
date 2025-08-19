# Design Decisions (2025-08-19)

## Patterns Adopted
- Single `Arc<Inner>`: centralizes shared state and enables last-reference detection.
- Weak-backed maintenance: background task does not keep the pool alive; uses `Notify` to stop.
- Explicit `close().await`: deterministic shutdown path that marks closed, awaits maintenance end, and drains idle.
- Best‑effort Drop backstop: on last reference, notify + await maintenance (bounded) + close idle.
- Fair capacity release: semaphore permit is released only after resource is requeued to idle (spawned return task).

## Alternatives Considered
- mpsc return channel: replaced by direct return via spawned task to reduce moving parts and align with fairness (permit released after requeue).
- Lock-free queue: deferred; Mutex<VecDeque> is sufficient for current scale; consider ArrayQueue + atomic counters if contention appears.
- Hooks (before/after acquire): deferred until a consumer needs them; keep API minimal.

## API Surface
- `PoolOptions { max_connections, acquire_timeout, idle_timeout, max_lifetime, health_check_interval }`
- `Pool<T>` where `T: PoolableResource`
- `PoolConnection<T>` — checked-out resource; returns to idle on Drop
- `PoolStats { idle, max, closed }`
- `traits::PoolableResource` — `is_healthy()`, `close()`, `resource_id()`

## Error Semantics
- Acquire timeout → `ShadowcatError::Timeout`
- Exhausted → `ShadowcatError::PoolExhausted`
- Closed → `ShadowcatError::Protocol("Pool closed")` (consider a dedicated error in follow-up)

## Open Questions / Follow-ups
- DONE: Close event helper to cancel/short-circuit waiters predictably. We
  added a shared `Notify`-based close event; `acquire()` races the semaphore
  against `shutdown.notified()`, so close cancels waiters deterministically.
- Introduce health hooks if consumers need gating.
- Optional: lock-free idle queue + atomic `num_idle` for higher concurrency.
- Optional: min-connections maintenance if we need proactive warmup.
