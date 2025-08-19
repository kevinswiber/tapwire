# Recommendations for Shadowcat Pool Cleanup

## 1) Clarify Drop vs. Shutdown Semantics (Docs + Code)
- Add documentation like SQLx: dropping the last pool does not perform graceful server-side closes; call `shutdown().await` for a clean stop.
- Option A (simple): Keep last-reference Drop to `notify_one()` and stop maintenance; rely on explicit shutdown for graceful cleanup.
- Option B (backstop): On last Drop, `tokio::spawn` an async cleanup that (a) notifies, (b) awaits maintenance handle, (c) drains and `close().await` idle connections. This emulates explicit shutdown if the owner forgets.

## 2) Fix Return-Channel Error Cleanup
- On `try_send` error in `PooledConnection::Drop`, extract the connection from `TrySendError` and close it in a spawned task, with a short timeout. Then decrement active.
- Rationale: Prevent leaked subprocesses/file handles if the return channel is full or closed.

## 3) Background Task Cancellation & Ownership
- Continue to have maintenance own the receiver and interval (good).
- Consider `Weak` refs + a `CloseEvent`-like construct so tasks don’t keep the pool alive and cancellation is uniform.

## 4) Health Semantics for Subprocess
- Ensure `Subprocess::receive_response()` flips `connected=false` on EOF; optionally check `child.try_wait()` in `is_connected()`.
- Gate return-to-idle on `is_healthy()`.

## 5) Optional: Reduce Contention at Scale
- If contention appears, consider replacing `Mutex<VecDeque>` with a lock-free queue (e.g., `ArrayQueue`) and atomics for counts.
- Keep `get_idle_connection()` health checks off-lock (already improved).

## 6) Tests to Cement Behavior
- Drop-vs-shutdown: prove maintenance continues across clone drops; prove explicit `shutdown()` drains and closes.
- Backpressure: simulate full return channel; ensure connections are closed on drop.
- Reuse: persistent stdio server with `max_connections=1` → only one subprocess spawned across N requests.
- Dead stdio process: single-shot CLI → returned connection is discarded; no leak.

## 7) Operational Guidance
- Provide examples of calling `shutdown()` during server shutdown and how to interpret pool stats.
- Document stdio limitation for single-shot tools; recommend HTTP or persistent stdio servers for throughput.

