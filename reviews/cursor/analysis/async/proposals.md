# Phase B — Concrete Proposals for Cancellation & Concurrency

Scope: `shadowcat-cursor-review/` at `eec52c8`

## Standard shutdown pattern
- Prefer `tokio_util::sync::CancellationToken` (if adding dep is acceptable). Otherwise use `tokio::sync::broadcast` or `watch` + `select!`.
- Components adopt a common API:
  - `fn with_shutdown(self, token: CancellationToken) -> Self`
  - `async fn start(&mut self) -> Result<()>` (spawns tasks that `select!` on `token.cancelled()`).
  - `async fn stop(&mut self)` cancels token and `JoinHandle::await` with timeout; last resort `abort()`.

## Forward proxy (cooperative shutdown)
- Problem: relies on `abort()`; some tasks don’t select on shutdown.
- Proposal:
  - Inject `CancellationToken` into read/process/write loops and background tasks.
  - Each loop uses:
    ```
    tokio::select! {
      biased;
      _ = token.cancelled() => break,
      maybe_msg = transport.receive() => { /* ... */ }
    }
    ```
  - Keep `abort()` as fallback if join times out.

## Health checker (graceful stop)
- Problem: endless spawned loops; no stop.
- Proposal:
  - Store `JoinHandle` per upstream and a shared `CancellationToken`.
  - Add `stop_health_checks()` that cancels token and awaits handles with timeout.
  - Replace raw `sleep`/`interval` with `select!` against `token.cancelled()`.

## StdioTransport lifecycle
- Problem: async work in `Drop`.
- Proposal:
  - Make `close()` responsible for terminating and awaiting the child; return `Result<()>`.
  - `Drop` only logs if `connected`; optionally call `child.start_kill()` (non-async) as last resort.
  - Add shutdown token to stdin/stdout tasks and break loops on cancellation; close channels on `close()`.

## ReplayTransport receive (avoid await-in-lock)
- Problem: holds mutex guard across `.await` on `recv()`.
- Proposal:
  - Single-consumer assumption allows taking the receiver out of the mutex:
    - `let mut rx = { self.outbound_rx.lock().await.take() }.ok_or(Closed)?;`
    - `let msg = rx.recv().await;`
    - `*self.outbound_rx.lock().await = Some(rx);`
  - Add shutdown token to `select!` between `recv()` and cancellation.

## Reverse proxy metrics (lock-free)
- Problem: `std::sync::Mutex<Duration>` in async context.
- Proposal:
  - Track `total_nanos: AtomicU64`, `requests_total: AtomicU64`, `requests_failed: AtomicU64`.
  - On record: `total_nanos.fetch_add(duration.as_nanos() as u64, Relaxed)`.
  - On read: compute average from atomics; expose both sum and count.

## Testing additions
- Add integration tests that verify cooperative shutdown:
  - Start components, trigger cancellation, assert tasks exit within bounds and resources freed.
- Add replay receive close test: ensure `close()` completes even if `receive()` is pending.
