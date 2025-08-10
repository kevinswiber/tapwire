# Phase B — Concrete Proposals for Cancellation & Concurrency

Scope: `shadowcat-cursor-review/` at `eec52c8`

## Standard shutdown pattern
- Prefer `tokio_util::sync::CancellationToken` (if adding dep is acceptable). Otherwise use `tokio::sync::broadcast` or `watch` + `select!`.
- Components adopt a common API:
  - `fn with_shutdown(self, token: CancellationToken) -> Self`
  - `async fn start(&mut self) -> Result<()>` (spawns tasks that `select!` on `token.cancelled()`).
  - `async fn stop(&mut self)` cancels token and `JoinHandle::await` with timeout; last resort `abort()`.

### Concrete patterns

- Forward proxy adoption of CancellationToken
  ```rust
  pub struct ForwardProxyRuntime { cancel: CancellationToken, handles: Vec<JoinHandle<()>> }
  impl ForwardProxy {
      pub fn with_runtime(self, rt: ForwardProxyRuntime) -> Self { /* store runtime */ }
      async fn run_reader(ctx: MessageReadContext<T>, token: CancellationToken) { loop {
          tokio::select! { _ = token.cancelled() => break,
              msg = { let mut t = ctx.transport.write().await; t.receive() } => { /* process */ }
          }
      }}
      async fn run_writer(transport: Arc<RwLock<T>>, mut rx: mpsc::Receiver<MessageEnvelope>, token: CancellationToken) {
          loop { tokio::select! { _ = token.cancelled() => break, Some(env) = rx.recv() => {
              if let Err(e) = { let mut t = transport.write().await; t.send(env).await } { break; }
          }}}
      }
      pub async fn shutdown(&mut self) { self.rt.cancel.cancel(); for h in self.rt.handles.drain(..) {
          let _ = tokio::time::timeout(Duration::from_secs(2), h).await; }
      }
  }
  ```

- Stdio transport cooperative shutdown
  ```rust
  struct StdioTransport { cancel: CancellationToken, /* ... */ }
  fn setup_io_channels(&mut self, stdin: ChildStdin, stdout: ChildStdout) {
      let c1 = self.cancel.child_token(); let c2 = self.cancel.child_token();
      tokio::spawn(async move { let mut stdin = stdin; while let Some(msg) = tokio::select!{
          _ = c1.cancelled() => None, m = stdin_rx.recv() => m } { /* write & flush */ } });
      tokio::spawn(async move { let mut reader = BufReader::new(stdout); loop { let mut line=String::new();
          tokio::select!{ _ = c2.cancelled() => break, res = reader.read_line(&mut line) => { /* handle */ } } } });
  }
  async fn close(&mut self){ self.cancel.cancel(); /* drop channels */ if let Some(mut child)=self.process.take(){ let _=child.start_kill(); let _=child.wait().await; } }
  ```

- Health checker runtime management
  See pattern sketch in `analysis/async/cancellation.md`.

- ReplayTransport receive without await-in-lock
  ```rust
  async fn receive(&mut self) -> TransportResult<MessageEnvelope> {
      let mut rx = { self.outbound_rx.lock().await.take() }.ok_or(TransportError::Closed)?;
      let msg = rx.recv().await.ok_or(TransportError::Closed)?;
      *self.outbound_rx.lock().await = Some(rx);
      Ok(msg)
  }
  ```

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
