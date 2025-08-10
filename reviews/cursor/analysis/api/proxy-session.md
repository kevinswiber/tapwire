# C.2 — Proxy Engine and Session Lifecycle Review (Prep)

Scope: `shadowcat-cursor-review/` at `eec52c8`

- Components
  - Forward proxy (`src/proxy/forward.rs`): task fan-out, version negotiation, interceptor/recorder integration.
  - Reverse proxy (`src/proxy/reverse.rs`): axum HTTP server, stdio pool, auth/rate limiting middleware, metrics.
  - Session manager (`src/session/manager.rs`): ID extraction, pending request tracking, cleanup, shutdown.

- Initial observations
  - Forward proxy uses `abort()` on tasks; prefer cooperative shutdown with join.
  - Reverse proxy metrics uses sync mutex; consider lock-free.
  - Session ID extraction has solid fallbacks; document invariants for matching responses.

- Early proposals
  - Introduce a shared `Shutdown` token used across proxy, transports, health checker.
  - Define clear lifecycle: start -> running -> draining -> shutdown; add metrics for each.

## Forward proxy API notes
- Readers/writers are spawned and managed via `tasks: Vec<JoinHandle<()>>`; shutdown uses `abort()`.
  ```651:659,682:687:shadowcat-cursor-review/src/proxy/forward.rs
  pub async fn shutdown(&mut self) { /* send shutdown */ for task in self.tasks.drain(..) { task.abort(); } }
  impl Drop for ForwardProxy { fn drop(&mut self) { for task in &self.tasks { task.abort(); } } }
  ```
- Proposal: expose `with_shutdown(token)` and make `shutdown()` await task handles with timeout before abort.

## Reverse proxy API notes
- Metrics exposed via `/metrics` with Prometheus formatting; uses `ReverseProxyMetrics` with a sync mutex.
  ```318:361,1253:1334:shadowcat-cursor-review/src/proxy/reverse.rs
  struct ReverseProxyMetrics { /* AtomicU64s + Mutex<Duration> */ } // metrics endpoint assembles exposition
  ```
- Proposal: switch to lock-free sum using atomics to avoid blocking in request handler; keep endpoint contract.

## Session lifecycle API
- `SessionManager` provides `create_session`, `record_frame`, `complete_session`, `shutdown`, and extraction/helpers.
  ```136:173,243:246,536:556:shadowcat-cursor-review/src/session/manager.rs
  pub async fn create_session(...); pub async fn update_session(...); pub async fn shutdown(&self) -> SessionResult<()>;
  ```
- Observations:
  - Good TTL and LRU cleanup; cleanup task spawned with interval.
  - `process_message_for_session` defaults transport context to stdio; consider passing actual context for accuracy.
- Proposals:
  - Add explicit lifecycle states and counters (running/draining) surfaced via `get_session_stats`.
  - Accept a `TransportContext` when recording from proxies to preserve accurate edge metadata.
