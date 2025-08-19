# Comparison: SQLx vs. Shadowcat Pool

## Architecture
- SQLx: `Pool(Arc<PoolInner>)`; Shadowcat: now also uses an inner-Arc pattern. Good alignment.
- SQLx: idle queue = lock-free `ArrayQueue` and atomics; Shadowcat: `VecDeque` under `Mutex` with periodic cleanup. Acceptable but higher contention; may be fine at current scale.

## Maintenance / Reaper
- SQLx: task owns state via `Weak`, listens to a `CloseEvent`, handles idle timeout and max lifetime, refills `min_connections`.
- Shadowcat: maintenance loop now owns the mpsc receiver and interval; good. It handles returned connections and a periodic cleanup that re-validates health and ages.
  - Suggestion: consider `Weak`-based cancellation and a `CloseEvent`-like mechanism; current `Notify` works but does not propagate to future waits.

## Shutdown Semantics
- SQLx: No async work in Drop; explicit `.close().await` drains and closes idle; docs warn users.
- Shadowcat: last-reference Drop currently `notify_one()` and aborts maintenance task; does not close idle connections (no await in Drop).
  - Options: (a) keep as-is but document that explicit `shutdown()` is required for graceful cleanup; (b) spawn backstop async cleanup on last Drop.

## Return Path on Drop
- SQLx: `PoolConnection::Drop` always spawns an async return/close; `close_on_drop` uses timeout.
- Shadowcat: `PooledConnection::Drop` uses `try_send` to return; on error it decrements active but does not close the connection.
  - Risk: leaked OS resources under channel backpressure or closure.
  - Recommendation: extract the connection from `TrySendError` and `close().await` it in a spawned task, with a timeout.

## Health Signaling
- SQLx: has `test_before_acquire` and hooks; reliably marks broken connections; reaps unhealthy.
- Shadowcat: checks `is_healthy()` before returning to idle and during cleanup; but subprocess transport previously didn’t set `connected=false` on EOF.
  - Recommendation: ensure subprocess flips `connected=false` on stdout close and optionally check child `try_wait()` in `is_connected()`.

## Concurrency Controls
- SQLx: RAII `DecrementSizeGuard` updates size atomically and releases semaphore consistently; parent/child pool semantics.
- Shadowcat: owned `OwnedSemaphorePermit` per pooled connection; explicit active/total counters behind `Mutex` (fine, slightly heavier).

## Documentation
- SQLx: clear and extensive docs on Drop vs Close.
- Shadowcat: add a “Drop semantics” section to docs mirroring SQLx; advise calling `shutdown()` in server shutdown.

