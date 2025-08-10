# B.1 — Unsafe/FFI Audit

Scope: `shadowcat-cursor-review/` at `eec52c8`

- Summary
  - No `unsafe` blocks, `extern "C"`, or FFI bindings detected across the crate.
  - Primary risks are not memory-unsafe code but async lifecycle, Drop behavior, and locking patterns.

- Findings (none-unsafe, but safety-adjacent)
  - StdioTransport Drop spawns async task
    - Risk: spawning in `Drop` can fail if no Tokio runtime; Drop cannot `await`, so process termination is best-effort and may leak.
    - Cite:
      ```451:459:shadowcat-cursor-review/src/transport/stdio.rs
      impl Drop for StdioTransport {
          fn drop(&mut self) {
              if self.connected {
                  if let Some(mut child) = self.process.take() {
                      tokio::spawn(async move {
                          let _ = child.kill().await;
                      });
                  }
              }
          }
      }
      ```
    - Suggestions:
      - Prefer explicit `close()` that ensures child termination and `await`s the kill; make `Drop` a no-op or minimal logging only.
      - If forced to kill on drop, consider `child.start_kill()` and `std::thread::spawn` with blocking kill, or a background runtime handle explicitly passed in.
  - Additional Drop implementations to review
    - ForwardProxy drops by aborting tasks; abort is non-cooperative and can leak resources in user code that expects graceful shutdown.
      ```682:687:shadowcat-cursor-review/src/proxy/forward.rs
      impl Drop for ForwardProxy {
          fn drop(&mut self) {
              for task in &self.tasks {
                  task.abort();
              }
          }
      }
      ```
      - Suggestion: mirror `shutdown()` semantics in Drop only as a last resort: log and abort if tasks still running after an attempted cooperative shutdown has failed or if Drop is reached without prior shutdown.
    - SSE session stream and connection manager perform async work during Drop via spawned tasks/try locks.
      ```462:491:shadowcat-cursor-review/src/transport/sse/session.rs
      impl Drop for SessionStream { /* spawns to remove connection and notify hooks */ }
      ```
      ```294:311:shadowcat-cursor-review/src/transport/sse/manager.rs
      impl Drop for SseConnectionManager { /* try_read/try_write then close connections */ }
      ```
      ```170:176:shadowcat-cursor-review/src/transport/sse/connection.rs
      impl Drop for SseConnection { /* closes connection if alive */ }
      ```
      ```593:617:shadowcat-cursor-review/src/transport/sse/client.rs
      impl Drop for SseConnectionStream { /* spawns timeout-bound close on runtime */ }
      ```
      - Suggestions:
        - Ensure a dedicated cooperative `shutdown/close` path exists and is exercised by owners; Drop should be best-effort only.
        - For cases spawning in Drop, prefer using `tokio::runtime::Handle::try_current()` with a bounded timeout (as done in `SseConnectionStream`) or fall back to non-async best-effort close to avoid panics when no runtime is present.
  - Reverse proxy metrics uses `std::sync::Mutex` in async context
    - Risk: small, but a sync mutex in async handler paths can block the executor under contention.
    - Cite:
      ```322:337:shadowcat-cursor-review/src/proxy/reverse.rs
      pub struct ReverseProxyMetrics {
          requests_total: AtomicU64,
          requests_failed: AtomicU64,
          request_duration_sum: std::sync::Mutex<std::time::Duration>,
      }
      ```
    - Suggestions:
      - Replace with lock-free accumulation (e.g., `AtomicU64` nanoseconds) and derive average at read time, or use `parking_lot::Mutex` given short critical section.
      - Example lock-free pattern:
        ```rust
        struct ReverseProxyMetrics {
            requests_total: AtomicU64,
            requests_failed: AtomicU64,
            total_nanos: AtomicU64,
        }
        impl ReverseProxyMetrics {
            fn record_request(&self, duration: Duration, success: bool) {
                self.requests_total.fetch_add(1, Ordering::Relaxed);
                if !success { self.requests_failed.fetch_add(1, Ordering::Relaxed); }
                self.total_nanos.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
            }
            fn average_ms(&self) -> f64 {
                let total = self.requests_total.load(Ordering::Relaxed).max(1);
                (self.total_nanos.load(Ordering::Relaxed) as f64 / 1_000_000f64) / total as f64
            }
        }
        ```

- Positive notes
  - No raw pointers, no FFI; third-party crates are idiomatic.

- Action checklist
  - [ ] Replace Drop-based async kill with explicit close semantics; minimize work in Drop.
  - [ ] Remove `std::sync::Mutex` from hot async paths for metrics (or make accumulation lock-free with atomics).
  - [ ] Add/verify cooperative shutdown on owners of SSE/forward proxy tasks; keep Drop best-effort only.
