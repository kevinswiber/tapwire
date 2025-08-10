# B.3 — Send/Sync and Locking Analysis

Scope: `shadowcat-cursor-review/` at `eec52c8`

- Summary
  - Lock usage is primarily `tokio::sync::{Mutex,RwLock}` with short critical sections; a few places use `std::sync::Mutex`.
  - Notable await-in-lock patterns in replay transport and recorder playback where it is acceptable but could be optimized.

- Findings

1) ReplayTransport receive holds lock across await
   - Cite:
     ```368:381:shadowcat-cursor-review/src/transport/replay.rs
     let mut outbound_rx = self.outbound_rx.lock().await; let outbound_rx = outbound_rx.as_mut().ok_or(TransportError::Closed)?; match outbound_rx.recv().await { ... }
     ```
   - Risk: lock held for potentially long `recv()`; competes with close/stop which also locks the same fields.
   - Suggestion: take ownership of the receiver out of the mutex (swap) before awaiting; or re-architect with a channel that doesn’t require holding the lock while awaiting.

2) Recorder playback uses multiple locks in command handlers
   - Cite:
     ```319:345,351:361,365:375,397:403:shadowcat-cursor-review/src/recorder/replay.rs
     state.write().await; start_time.lock().await; current_frame.write().await; config.write().await
     ```
   - Risk: minimal as operations are short; order is consistent inside each branch.
   - Suggestion: none required; consider documenting lock order if expanded.

3) ReverseProxyMetrics with sync mutex
   - Cite:
     ```319:337:shadowcat-cursor-review/src/proxy/reverse.rs
     request_duration_sum: std::sync::Mutex<std::time::Duration>
     ```
   - Risk: blocking executor threads under contention.
   - Suggestion: replace with lock-free atomic accumulation or `parking_lot::Mutex` if a mutex is kept.

4) SessionManager pending request maps and rate limiting
   - Observation: uses `RwLock` with small critical sections; rate limiting updates occur inside a single write guard then released before further awaits.
   - Suggestion: OK; consider sharding if contention observed in profiling.

- Action checklist
  - [ ] Refactor replay receive to avoid await-in-lock.
  - [ ] Replace sync mutex for metrics accumulation.
  - [ ] Add doc comments on lock ordering in recorder playback if complexity grows.
