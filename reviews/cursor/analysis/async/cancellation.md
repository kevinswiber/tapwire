# B.2 — Cancellation Safety & Concurrency Review

Scope: `shadowcat-cursor-review/` at `eec52c8`

- Summary
  - Core loops use `tokio::select!` appropriately in replay and interceptor watchers.
  - Some long-lived tasks lack cooperative shutdown; a few await-in-lock patterns exist but mostly short critical sections.

- Findings and suggestions

1) ForwardProxy shutdown coordination
   - Observed: shutdown channel exists and tasks are `abort()`-ed in some code paths; reliance on abort can leak resources.
   - Cite:
     ```55:59,143:147,654:655,659:659:shadowcat-cursor-review/src/proxy/forward.rs
     shutdown_tx: Option<mpsc::Sender<()>>; ... let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
     ... shutdown_rx.recv().await; ... if let Some(shutdown_tx) = self.shutdown_tx.take() { let _ = shutdown_tx.send(()).await; }
     ... task.abort();
     ```
   - Suggestions:
     - Prefer cooperative shutdown: tasks should `select!` on a `shutdown_rx` and exit cleanly, joining handles (`JoinHandle::await`) where feasible.
     - Keep `abort()` only as last-resort on timeout.

2) File watcher event loop
   - Observed: `tokio::spawn` with `loop { select! { event_rx.recv(), shutdown_rx.recv() } }` — good pattern, but no join on shutdown.
   - Cite:
     ```388:415:shadowcat-cursor-review/src/interceptor/rules_interceptor.rs
     tokio::spawn(async move { loop { tokio::select! { Some(event) = event_rx.recv() => { ... } _ = shutdown_rx.recv() => break; } } });
     ```
   - Suggestions:
     - Store handle and await it during `stop_file_watching` to ensure watcher task exits before returning.

3) Health checker background tasks
   - Observed: `start_health_checks` spawns per-upstream loops with retries and sleeps; no shutdown path.
   - Cite:
     ```75:86,90:133,196:201:shadowcat-cursor-review/src/proxy/health_checker.rs
     pub async fn start_health_checks(&self, upstreams: Vec<UpstreamConfig>) { for upstream in upstreams { tokio::spawn(async move { Self::health_check_loop(...).await; }); } }
     ```
   - Suggestions:
     - Add shutdown signal or cancellation token, and a `stop_health_checks` method. Ensure tasks `select!` on shutdown.

4) Replay transport receive path holds lock across `.await`
   - Observed: acquires `outbound_rx` lock and then awaits on `recv()`, holding the mutex.
   - Cite:
     ```368:375:shadowcat-cursor-review/src/transport/replay.rs
     let mut outbound_rx = self.outbound_rx.lock().await; let outbound_rx = outbound_rx.as_mut().ok_or(...)?; match outbound_rx.recv().await { ... }
     ```
   - Risk: serializes receivers, can delay other state changes that need same lock (close/stop).
   - Suggestions:
     - Clone the receiver or take it out of the mutex (e.g., swap with local `Option`) before await; or switch to `tokio::sync::Mutex` + `notify` pattern where lock is released before awaiting.

5) Stdio transport background tasks — backpressure and shutdown
   - Observed: spawned stdin/stdout tasks run indefinitely; shutdown relies on process exit or channel closure. Drop triggers kill via spawn.
   - Cite:
     ```60:101,103:139:shadowcat-cursor-review/src/transport/stdio.rs
     tokio::spawn stdin writer/reader loops; log and break on errors/EOF; no explicit shutdown signal.
     ```
   - Suggestions:
     - Plumb a shutdown broadcast to break both loops; on `close()`, send shutdown and drain channels. Ensure child is terminated and awaited.

6) Metrics mutex in reverse proxy
   - Observed: `std::sync::Mutex<Duration>` used in async context.
   - Cite:
     ```319:337:shadowcat-cursor-review/src/proxy/reverse.rs
     request_duration_sum: std::sync::Mutex<std::time::Duration>,
     ```
   - Suggestions:
     - Use atomics for accumulators or an async-friendly mutex. Prefer lock-free.

- Positive notes
  - Many loops already use `tokio::select!` and timers; tests suggest good coverage of lifecycle edges.

- Action checklist
  - [ ] Introduce structured shutdown (broadcast channel) for forward proxy, health checker, and stdio transport background tasks.
  - [ ] Refactor locking in replay transport receive path to avoid holding locks across await.
  - [ ] Replace sync mutex in async metrics or make accumulation lock-free.
