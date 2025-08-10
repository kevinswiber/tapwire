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

- Positive notes
  - No raw pointers, no FFI; third-party crates are idiomatic.

- Action checklist
  - [ ] Replace Drop-based async kill with explicit close semantics; minimize work in Drop.
  - [ ] Remove `std::sync::Mutex` from hot async paths for metrics.
