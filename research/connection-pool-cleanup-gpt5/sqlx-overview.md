# SQLx Pool Cleanup & Lifecycle: Overview

Source examined: `~/src/sqlx/sqlx-core/src/pool/{mod.rs, inner.rs, connection.rs, options.rs}`

Key Architecture
- Pool handle: `Pool<DB>(Arc<PoolInner<DB>>)` — thin, cloneable wrapper over a single Arc inner.
- Inner state: `PoolInner` holds options, semaphore, lock-free idle queue, counters, close event.
- Connections: `PoolConnection` contains a live connection and a handle to the `PoolInner`.
- Background tasks: spawned via `spawn_maintenance_tasks(&Arc<PoolInner>)` using `Weak` refs.

Lifecycle & Shutdown
- Explicit close: `Pool::close()` (via `PoolInner::close()`) is the primary, recommended shutdown.
  - Marks closed via an `Event` (`on_closed`), then acquires all permits and drains idle queue, `close().await` on each.
  - Resets counters to 0.
- Implicit drop:
  - No async work in `Drop` for `Pool` (it’s just an `Arc`).
  - `PoolInner` implements `Drop` only to `mark_closed()` and, if a child pool, return stolen permits to the parent. No async closing in `Drop`.
  - `mod.rs` documents: dropping the last pool may not notify server-side of closure immediately (no async Drop). Users should call `.close().await` for graceful cleanup.

Return & Reuse
- `PoolConnection::Drop` spawns a task to return to pool (`return_to_pool()`) if not `close_on_drop`.
  - If `close_on_drop` set, spawns `take_and_close()` with a 5s timeout to avoid hangs.
  - After return/close, triggers `min_connections_maintenance()`.
- Idle storage: `ArrayQueue<Idle<DB>>` (lock-free), with separate `num_idle` atomic to avoid len() churn.
- Acquisition fairness and throttling: custom `AsyncSemaphore` and counters; `try_increment_size()` guards against exceeding `max_connections`.

Maintenance / Reaper
- `spawn_maintenance_tasks()` uses `Weak` to avoid keeping the pool alive.
- Periodically checks idle and lifetime; closes expired/unhealthy connections, and backfills to `min_connections`.
- Respects global close: listens to `CloseEvent` (event-listener) to stop cleanly.

Health & Hooks
- `test_before_acquire` option can ping connection before reuse.
- `before_acquire` and `after_connect` hooks allow custom gating and initialization; failures close/hard-close accordingly.

Documentation & Guarantees
- Clear Drop semantics warning: pool drop is not a replacement for `.close()` in client/server DBs.
- Acquire fairness and timeouts documented; slow acquire logging.
- Parent/child pool model with permit stealing and returning on drop.

