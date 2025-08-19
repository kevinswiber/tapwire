# Detailed Review: Stdio Connection Pool Reuse Failure

Date: 2025-08-19

## Summary

- Symptom: Every stdio request spawns a new subprocess; the pool never reuses.
- Confirmed: `PooledConnection` Drop calls `try_send` to return, but the maintenance loop never processes returns.
- Primary suspect: Maintenance loop holds a `MutexGuard<mpsc::Receiver<T>>` across awaits and polls `recv()` inside `tokio::select!`. This is brittle and can starve/mis-poll. The receiver should be owned by the background task, not gated through `Arc<Mutex<_>>`.
- Additional issue: `Subprocess::is_connected()` stays `true` even after stdout closes; returned connections can be “healthy” but dead.
- Minor: `cleanup_idle_connections` awaits while holding the `idle_connections` lock; better to avoid.

## What’s Likely Going Wrong

1. Receiver under a Mutex
   - In `pool.rs`, the return channel is `Arc<Mutex<mpsc::Receiver<T>>>`. The maintenance task does `let mut return_rx = self.return_rx.lock().await;` and then uses `return_rx.recv()` inside `tokio::select!`, holding the guard across awaits.
   - This is an atypical pattern for `mpsc::Receiver` and can lead to starvation or unexpected polling behavior. The single consumer of an `mpsc::Receiver` should own it directly in the consuming task.

2. Interval’s immediate tick
   - `tokio::time::interval` fires an immediate first tick. Combined with the guarded receiver, this can bias the loop toward the interval branch and starve the `recv()` branch, especially at startup. Draining with `try_recv` helps but doesn’t fix the structural issue.

3. Health signaling for subprocess
   - `Subprocess` sets `connected = true` in `connect()` and never flips it to false unless `close()` is called. When stdout closes after a response (typical for many CLI tools), `receive_response()` gets `None` from the channel, but `is_connected()` still returns `true`. This misleads the pool and can keep dead connections around.

## Code References

- `shadowcat/src/proxy/pool.rs`
  - Drop path (return via channel): lines ~54–87
  - Acquire with owned semaphore permit: lines ~189–246
  - Maintenance loop: lines ~284–328
  - Returned connection processing: lines ~332–357
- `shadowcat/src/proxy/reverse/upstream/stdio.rs`
  - Factory creates new subprocess per acquire; logs pool stats around acquire: lines ~78–107, ~160–170
- `shadowcat/src/transport/outgoing/subprocess.rs`
  - `connect()` marks `connected = true`; reader tasks signal EOF only via logs
  - `receive_response()` returns error on closed stdout channel but doesn’t clear `connected`
  - `is_connected()` only checks the flag; does not inspect child status

## Recommendations

### Fix 1 — Move receiver into the maintenance task (own the rx)

- Replace `Arc<Mutex<mpsc::Receiver<T>>>` with moving the `Receiver<T>` into the maintenance task when spawning it in `ConnectionPool::new()`.
- Keep only `return_tx` on the pool struct.
- In `new()`:
  - `let (return_tx, return_rx) = mpsc::channel(channel_size);`
  - Spawn task with `maintenance_loop(return_rx, shutdown.clone())`.
  - Remove `return_rx` from `ConnectionPool<T>` entirely.
- Change signature and loop:

```rust
async fn maintenance_loop(
    &self,
    mut return_rx: tokio::sync::mpsc::Receiver<T>,
    shutdown: Arc<tokio::sync::Notify>,
) {
    let mut interval = tokio::time::interval(self.config.health_check_interval);
    interval.tick().await; // absorb immediate first tick
    loop {
        tokio::select! {
            Some(conn) = return_rx.recv() => {
                self.process_returned_connection(conn).await;
            }
            _ = interval.tick() => {
                self.cleanup_idle_connections().await;
            }
            _ = shutdown.notified() => {
                // Optional small drain
                while let Ok(Some(conn)) = tokio::time::timeout(
                    std::time::Duration::from_millis(10),
                    return_rx.recv(),
                ).await {
                    self.process_returned_connection(conn).await;
                }
                break;
            }
        }
    }
}
```

Why: Removes the guard-across-await antipattern and lets `select!` poll `recv()` fairly.

### Fix 2 — Adjust interval behavior to avoid first-tick bias

- Either call `interval.tick().await` before the loop (as above), or construct the interval with `interval_at(now + period, period)` so the first tick isn’t immediate.

### Fix 3 — Mark `Subprocess` disconnected when the process ends

- In `receive_response()`, if `rx.recv().await` returns `None`, set `self.connected = false` before returning `TransportError::ConnectionClosed`.
- Optionally, in `is_connected()`, check `child.try_wait()` and return `false` if the child has exited.
- Optionally, have `spawn_stdout_reader` notify on EOF via a channel; the transport sets `connected = false` upon notification.

Why: The pool must see dead subprocesses as unhealthy and avoid returning them to idle.

### Fix 4 — Avoid awaits while holding locks

- `cleanup_idle_connections()` currently awaits on `connection.is_healthy().await` with the `idle_connections` lock held. Refactor to collect candidates while holding the lock, release the lock for any async checks, then reacquire to remove/close.

### Fix 5 — Channel sizing and return path robustness

- Keep a bounded return channel sized to `max_connections * return_channel_multiplier` (2–4× is fine). With Fix 1, the receiver will drain promptly.
- Keep `try_send` in `Drop`. On `Full`/`Closed`, spawn cleanup and `decrement_active()` (already implemented) to avoid leaks.

### Fix 6 — Testing and metrics

- Add an integration test using a persistent stdio MCP server (Python or Rust) that handles multiple requests without exiting:
  - Configure pool with `max_connections = 1`.
  - Send N requests; assert only one subprocess PID is observed and `idle_connections` stabilizes at 1.
- Document the behavior for CLI tools that exit after a single request (like `echo`): pooling cannot reuse such processes; recommend HTTP transport or persistent stdio servers for throughput.

## Expected Outcome

- With Fix 1 (own the receiver) and Fix 2 (interval adjustment), the maintenance loop should consistently process returned connections, populating `idle_connections` and enabling reuse.
- With Fix 3 (accurate `is_connected()`), the pool will discard dead subprocesses rather than attempting to reuse them.
- Semaphore permits remain scoped to `PooledConnection` and correctly enforce `max_connections`.

## Notes on Alternatives

- Unbounded return channel: A short-term workaround that can reduce backpressure issues but doesn’t correct the receiver/guard architecture. Use only as a temporary measure.
- Direct return in `Drop` without channel: Feasible with proper locking, but increases coupling and risk of async work in drop paths. The channel + background task pattern is cleaner.
- Process pool (pre-spawned): Viable future path if many CLI tools are single-shot. Different design: pool processes rather than connections, assigning one request per process.

## Closing Thoughts

The pool’s semaphore/permit handling and `try_send` drop return are good. The main blocker to reuse is the receiver ownership pattern in the maintenance loop combined with the interval’s immediate tick and the subprocess health flag not being cleared on EOF. Fixing those should eliminate the repeated respawns and recover throughput.

