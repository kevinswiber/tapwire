# Code Audit: Pool + Subprocess (Second Pass)

Date: 2025-08-19

## Connection Pool (`shadowcat/src/proxy/pool.rs`)

- Drop path:
  - Uses `try_send` to `return_tx`; on error spawns async cleanup and decrements active. Good.
  - Holds `OwnedSemaphorePermit` in `PooledConnection`, so permit returns on drop. Good.

- Acquire:
  - Attempts reuse via `get_idle_connection()`; increments `active_connections` for both reused and new connections. Good.

- Maintenance loop:
  - Pattern:
    - `let mut return_rx = self.return_rx.lock().await;` (receiver behind `Arc<Mutex<_>>`).
    - Drains via `try_recv()`; then `tokio::select!` with `return_rx.recv()`, `interval.tick()`, `shutdown.notified()`.
  - Risk:
    - Holding a `MutexGuard<Receiver<T>>` across awaits in a select loop is atypical and error-prone; can lead to starvation or mis-polling.
    - Interval’s first tick is immediate; not absorbed, increasing risk of bias against recv branch.
  - Recommendation:
    - Move ownership of `Receiver<T>` into the spawned maintenance task; remove `Arc<Mutex<_>>`.
    - Call `interval.tick().await` once before the select loop to avoid immediate tick bias.

- Cleanup of idle:
  - Awaits `connection.is_healthy().await` while holding the `idle_connections` lock.
  - Recommendation: collect candidates first, release lock for any await, then reacquire to remove/close.

## Subprocess Transport (`shadowcat/src/transport/outgoing/subprocess.rs`)

- `connect()`:
  - Spawns child process and sets `connected = true`.
  - Spawns stdout/stderr reader tasks; stdout reader logs EOF but doesn’t inform main transport.

- `receive_response()`:
  - When `rx.recv().await` returns `None` (stdout closed), returns `TransportError::ConnectionClosed` but does not set `self.connected = false`.

- `is_connected()`:
  - Returns the `connected` flag only; does not check if child has exited (via `try_wait`).

- Recommendations:
  - In `receive_response()`, set `self.connected = false` on `None` from stdout channel.
  - Optionally check `child.try_wait()` in `is_connected()` and return `false` if the process exited.
  - (Optional) Wire a notification from stdout reader to flip connection state on EOF.

## Upstream Stdio (`shadowcat/src/proxy/reverse/upstream/stdio.rs`)

- Uses pool with a factory that connects a `Subprocess` (good). Reuse will occur only if idle connections are returned and marked healthy.
- With current subprocess health semantics, returned connections may be incorrectly considered healthy; fix above is needed to enable reuse.

## Bottom Line

The remaining blockers to pooling are (1) receiver ownership in the maintenance loop and (2) subprocess health flagging on EOF. Addressing both should enable actual reuse and remove the per-request spawn overhead for persistent stdio servers.

