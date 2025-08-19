# Patterns Observed in SQLx Pool

1) Single Arc Inner for All Shared State
- Pattern: `Pool` is a thin handle over `Arc<PoolInner>`; clones are cheap; lifetime of inner is tied to all handles.
- Benefit: Accurate last-reference detection; simple sharing; avoids per-field Arcs.

2) No Async in Drop; Explicit Close API
- Drop of `PoolInner` marks closed and releases parent permits; no awaits.
- Users are encouraged to call `Pool::close().await` for graceful shutdown; docs clearly set expectations.

3) Weak-Backed Maintenance Tasks
- Background reaper uses `Weak<PoolInner>` and a `CloseEvent` to exit; tasks never keep the pool alive.
- Periodic housekeeping: reap by idle timeout / max lifetime; refill to `min_connections`.

4) Lock-Free Idle Queue + Atomics
- Uses `ArrayQueue<Idle>` plus `num_idle` atomic; avoids lock contention and queue-length instability under churn.
- Size tracked via `AtomicU32` with `DecrementSizeGuard` RAII to avoid leaks.

5) Return/Close via Spawned Tasks with Timeouts
- `PoolConnection::Drop` spawns async return; `close_on_drop` spawns async `close()` with timeout to avoid hangs.
- After returning/closing, triggers `min_connections_maintenance` asynchronously.

6) Health Gating and Hooks
- `test_before_acquire` ping option; `before_acquire` and `after_connect` hooks to validate connection fitness.
- On hook failure: close/hard-close with appropriate logging.

7) Fair Acquire + Parent/Child Pools
- Fair semaphore; parent-child pools with permit stealing; `Drop` returns stolen permits to parent.

8) Clear Documentation of Drop Semantics
- Docs explicitly warn that dropping pool is not a clean server-side close; `.close()` is recommended.

