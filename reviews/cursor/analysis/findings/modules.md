# Phase A.1 — Repo Inventory and Module Mapping

Short template per CURSOR_RUST_CODE_REVIEWER.md.

- Summary:
  - Mapped core subsystems in `shadowcat-cursor-review/src`: transport, proxy, session, recorder, interceptor, auth, metrics, rate_limiting.
  - Identified key entry points, responsibilities, and immediate hot paths to examine later.

- Critical Issues:
  - None blocking at inventory stage.

- Observations:
  - Transport
    - Trait and core types in:
      ```22:76:shadowcat-cursor-review/src/transport/mod.rs
      pub trait Transport: Send + Sync { /* connect, send, receive, close, session_id */ }
      pub enum TransportType { Stdio, Http, Sse }
      pub struct SessionId(pub Uuid)
      ```
    - Stdio implementation with JSON-RPC parsing and batch error handling:
      ```1:32:shadowcat-cursor-review/src/transport/stdio.rs
      use super::{ MessageContext, MessageDirection, MessageEnvelope, ProtocolMessage, SessionId, Transport, ... }
      ```
      ```339:396:shadowcat-cursor-review/src/transport/stdio.rs
      async fn receive(&mut self) -> TransportResult<MessageEnvelope> { /* batch array -> JSON-RPC error */ }
      ```
    - HTTP and HTTP-MCP helpers re-exported; SSE transport includes session management and reconnect:
      ```26:36:shadowcat-cursor-review/src/transport/mod.rs
      pub use http::{HttpServer, HttpTransport};
      pub use http_mcp::{ HttpMcpTransport, McpHeaders, ... };
      pub use replay::{ReplayTransport, ...};
      ```
      ```12:20:shadowcat-cursor-review/src/transport/sse/session.rs
      use crate::transport::ProtocolMessage; use crate::transport::SessionId;
      ```
  - Proxy
    - Forward proxy orchestrates transports, session manager, tape recorder, interceptor chain:
      ```67:99:shadowcat-cursor-review/src/proxy/forward.rs
      pub fn new() -> Self { ... }
      #[instrument] pub async fn start<C,S>(&mut self, mut client_transport: C, mut server_transport: S) -> Result<()>
      ```
      ```512:623:shadowcat-cursor-review/src/proxy/forward.rs
      async fn process_message<T>(...) { record -> session/tape; run interceptor chain; forward or mock/block }
      ```
    - HTTP forward proxy server for HTTP<->stdio/http bridging:
      ```708:742:shadowcat-cursor-review/src/proxy/forward.rs
      pub struct HttpForwardConfig {...}; pub struct HttpForwardProxy {...}; pub async fn start(self) -> Result<()>
      ```
    - Reverse proxy server (Axum) with load balancing config:
      ```35:47:shadowcat-cursor-review/src/proxy/reverse.rs
      pub struct ReverseProxyServer { bind_address, session_manager, config }
      ```
      ```138:154:shadowcat-cursor-review/src/proxy/reverse.rs
      pub struct ReverseProxyConfig { bind_address, session_config, upstream_configs, load_balancing_strategy }
      ```
  - Session
    - Session model and in-memory store:
      ```53:71:shadowcat-cursor-review/src/session/store.rs
      pub struct Session { pub id: SessionId, pub transport_type: TransportType, ... }
      impl Session { pub fn new(id: SessionId, transport_type: TransportType) -> Self }
      ```
    - Manager with metrics and cleanup:
      ```3:11:shadowcat-cursor-review/src/session/manager.rs
      use crate::transport::{ MessageContext, MessageEnvelope, ProtocolMessage, SessionId, ... }
      ```
    - SSE integration and lifecycle hooks:
      ```309:379:shadowcat-cursor-review/src/session/sse_integration.rs
      #[derive(thiserror::Error)] pub enum SessionError { ... } // includes InvalidSessionId, TooManyConnections
      pub trait SseSessionLifecycle { on_session_created, on_session_initialized, ... }
      ```
  - Recorder
    - Tape recorder API:
      ```150:175:shadowcat-cursor-review/src/recorder/tape.rs
      pub struct TapeRecorder; pub async fn start_recording(&self, session: &Session, name: String) -> RecorderResult<TapeId>
      ```
      ```207:236:shadowcat-cursor-review/src/recorder/tape.rs
      pub async fn record_frame(&self, envelope: MessageEnvelope) -> RecorderResult<()>; pub async fn stop_recording(&self, ...) -> RecorderResult<Tape>
      ```
    - Tape player API:
      ```91:109:shadowcat-cursor-review/src/recorder/replay.rs
      pub struct TapePlayer; impl TapePlayer { pub fn new() -> Self; /* playback controls */ }
      ```
  - Interceptor
    - Chain and actions exported in module:
      ```19:21:shadowcat-cursor-review/src/interceptor/mod.rs
      pub use engine::{ InterceptAction, InterceptContext, Interceptor, InterceptorChain, ... }
      ```
    - Rule-based interceptor with reloadable rule files, metrics:
      ```134:151:shadowcat-cursor-review/src/interceptor/rules_interceptor.rs
      pub fn new(); pub fn with_config(config: RuleInterceptorConfig) -> Self
      ```
      ```546:603:shadowcat-cursor-review/src/interceptor/rules_interceptor.rs
      async fn intercept(&self, ctx: &InterceptContext) -> InterceptResult<InterceptAction> { ... }
      ```
  - Auth
    - Token validation, JWKS, audience/issuer checks:
      ```95:123:shadowcat-cursor-review/src/auth/token.rs
      pub struct TokenValidationConfig { issuer, audience, algorithms }
      impl From<&crate::auth::oauth::OAuth2Config> for TokenValidationConfig
      ```
      ```246:274:shadowcat-cursor-review/src/auth/token.rs
      pub async fn validate_token(&self, token: &str) -> Result<TokenClaims> { decode_header -> fetch key -> decode -> validate_claims }
      ```
  - Metrics and rate limiting
    - Session metrics counters and summary:
      ```557:569:shadowcat-cursor-review/src/session/manager.rs
      pub fn get_metrics(&self) -> SessionStats { ... }
      ```
    - Multi-tier rate limiter integrated in tests and proxy configs; see `rate_limiting/`.

- Suggestions:
  - For Phase B/C, prioritize transport trait boundaries, forward/reverse proxy flows, and session lifecycle coupling with recorder and interceptors.

- Action Checklist:
  - Document hot paths to deep‑dive: stdio receive/serialize, forward proxy read/process/write loops, version negotiation, SSE reconnect.
  - Confirm header/token scrubbing in reverse/forward HTTP paths.
  - Inventory metrics exposure points and logging levels on hot paths.
