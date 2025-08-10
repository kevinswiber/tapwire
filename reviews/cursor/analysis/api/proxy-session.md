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
