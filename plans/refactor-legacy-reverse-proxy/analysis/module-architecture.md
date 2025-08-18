# Module Architecture Design

## Module Tree Structure

```
src/proxy/reverse/
├── mod.rs                    (50 lines - public exports and docs)
├── error.rs                  (50 lines - error types)
├── config/
│   ├── mod.rs               (30 lines - exports)
│   ├── upstream.rs          (150 lines - upstream configs)
│   ├── load_balancing.rs   (50 lines - load balancing strategies)
│   └── server.rs            (70 lines - server configs)
├── metrics.rs               (50 lines - metrics collection)
├── server/
│   ├── mod.rs              (100 lines - server struct)
│   ├── builder.rs          (100 lines - builder pattern)
│   ├── state.rs            (150 lines - AppState and setup)
│   └── router.rs           (200 lines - route configuration)
├── mcp_handler.rs          (500 lines - main /mcp endpoint handler)
├── sse_streaming.rs        (450 lines - SSE response streaming)
├── upstream/
│   ├── mod.rs              (50 lines - upstream traits)
│   ├── stdio_processor.rs  (200 lines - stdio upstream processing)
│   ├── http_processor.rs   (250 lines - HTTP upstream processing)
│   └── selector.rs         (150 lines - upstream selection/routing)
├── session_ops.rs          (200 lines - session operations for reverse proxy)
├── middleware/
│   ├── mod.rs              (50 lines - middleware traits)
│   ├── auth.rs             (100 lines - auth middleware)
│   ├── rate_limit.rs       (100 lines - rate limiting)
│   └── interceptor.rs      (150 lines - message interception)
├── response/
│   ├── mod.rs              (50 lines - response utilities)
│   └── builder.rs          (100 lines - response construction)
└── tests/
    ├── mod.rs              (50 lines - test utilities)
    ├── unit/               (400 lines - unit tests)
    └── integration/        (400 lines - integration tests)
```

**Total Lines:** ~2,750 (current: 3,682 minus ~900 for admin UI)
**Largest Module:** 500 lines (mcp_handler.rs)
**Smallest Module:** 30 lines (config/mod.rs)
**Note:** Admin UI (~900 lines) will be removed entirely

## Module Responsibilities

### Core Modules

#### `error.rs` (50 lines)
**Responsibility:** Error type definitions and conversions
**Public API:**
```rust
pub enum ReverseProxyError { ... }
pub type Result<T> = std::result::Result<T, ReverseProxyError>;
impl From<X> for ReverseProxyError
impl IntoResponse for ReverseProxyError
```

#### `metrics.rs` (50 lines)
**Responsibility:** Performance and usage metrics collection
**Public API:**
```rust
pub struct ReverseProxyMetrics { ... }
impl ReverseProxyMetrics {
    pub fn new() -> Self
    pub fn record_request(&self, duration: Duration, success: bool)
    pub fn get_stats(&self) -> MetricsSnapshot
}
```

### Config Module (300 lines total)

#### `config/upstream.rs` (150 lines)
**Responsibility:** Upstream server configuration
**Public API:**
```rust
pub struct UpstreamConfig { ... }
pub struct HealthCheckConfig { ... }
pub struct PoolConfig { ... }
impl UpstreamConfig {
    pub fn validate(&self) -> Result<()>
    pub fn stdio_command(&self) -> Option<&[String]>
    pub fn http_url(&self) -> Option<&str>
}
```

#### `config/load_balancing.rs` (50 lines)
**Responsibility:** Load balancing strategies
**Public API:**
```rust
pub enum LoadBalancingStrategy { 
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    Random,
    HealthyFirst,
}
```

#### `config/server.rs` (70 lines)
**Responsibility:** Server-wide configuration
**Public API:**
```rust
pub struct ServerConfig { ... }
impl ServerConfig {
    pub fn bind_address(&self) -> SocketAddr
    pub fn max_body_size(&self) -> usize
}
```

### Server Module (550 lines total)

#### `server/mod.rs` (100 lines)
**Responsibility:** Main server struct and lifecycle
**Public API:**
```rust
pub struct ReverseProxyServer { ... }
impl ReverseProxyServer {
    pub async fn start(self) -> Result<()>
    pub async fn start_with_shutdown(self, shutdown: ShutdownToken) -> Result<()>
}
```

#### `server/builder.rs` (100 lines)
**Responsibility:** Server construction with builder pattern
**Public API:**
```rust
pub struct ReverseProxyServerBuilder { ... }
impl ReverseProxyServerBuilder {
    pub fn new() -> Self
    pub fn config(mut self, config: ServerConfig) -> Self
    pub fn upstream(mut self, upstream: UpstreamConfig) -> Self
    pub async fn build(self) -> Result<ReverseProxyServer>
}
```

#### `server/state.rs` (150 lines)
**Responsibility:** Application state management
**Internal API:**
```rust
pub(crate) struct AppState { ... }
impl AppState {
    pub fn new(config: Arc<ServerConfig>) -> Result<Self>
    pub fn session_manager(&self) -> &Arc<SessionManager>
    pub fn metrics(&self) -> &Arc<ReverseProxyMetrics>
}
```

#### `server/router.rs` (200 lines)
**Responsibility:** HTTP route configuration
**Internal API:**
```rust
pub(crate) fn create_router(state: AppState) -> Router
pub(crate) fn configure_middleware(router: Router) -> Router
```

### Main Handler Module (500 lines)

#### `mcp_handler.rs` (500 lines)
**Responsibility:** Main /mcp endpoint request handling
**Public API:**
```rust
pub async fn handle_mcp_request(
    State(state): State<AppState>, 
    headers: HeaderMap,
    req: Request
) -> Response
```
**Note:** This will be refactored from the current 550-line function into smaller helper functions

### SSE Streaming Module (450 lines)

#### `sse_streaming.rs` (450 lines)
**Responsibility:** Server-Sent Events response streaming
**Public API:**
```rust
pub async fn handle_mcp_sse_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: Request
) -> Result<Sse<impl Stream>>

pub async fn proxy_sse_from_upstream(
    transport: Box<dyn OutgoingTransport>,
    receiver: UnboundedReceiver<SseEvent>,
    session: Arc<Session>
) -> Result<()>
```

### Upstream Module (650 lines total)

#### `upstream/mod.rs` (50 lines)
**Responsibility:** Upstream processing traits
**Public API:**
```rust
pub trait UpstreamProcessor: Send + Sync {
    async fn process(&self, msg: Message) -> Result<Response>;
}
```

#### `upstream/stdio_processor.rs` (200 lines)
**Responsibility:** Stdio upstream processing
**Internal API:**
```rust
pub(crate) async fn process_via_stdio_pooled(...) -> Result<Value>
```

#### `upstream/http_processor.rs` (250 lines)
**Responsibility:** HTTP upstream processing
**Internal API:**
```rust
pub(crate) async fn process_via_http(...) -> Result<Value>
pub(crate) async fn process_via_http_hyper(...) -> Result<Response>
```

#### `upstream/selector.rs` (150 lines)
**Responsibility:** Upstream selection and routing
**Internal API:**
```rust
pub(crate) struct UpstreamSelector { ... }
impl UpstreamSelector {
    pub async fn select(&self, session: &Session) -> Result<&UpstreamConfig>
    pub async fn route(&self, upstream: &UpstreamConfig, msg: Message) -> Result<Response>
}
```

### Session Operations Module (200 lines)

#### `session_ops.rs` (200 lines)
**Responsibility:** Session operations specific to reverse proxy
**Internal API:**
```rust
pub(crate) async fn get_or_create_session(
    session_manager: &Arc<SessionManager>,
    session_id: SessionId,
    transport_type: TransportType,
) -> Result<Arc<Session>>

pub(crate) async fn update_session_state(
    session: Arc<Session>,
    new_state: SessionState,
) -> Result<()>
```

### Middleware Module (400 lines total)

#### `middleware/mod.rs` (50 lines)
**Responsibility:** Middleware trait definitions
**Public API:**
```rust
pub trait Middleware: Send + Sync {
    async fn process(&self, req: Request, next: Next) -> Result<Response>;
}
```

#### `middleware/auth.rs` (100 lines)
**Responsibility:** Authentication middleware
**Public API:**
```rust
pub fn auth_middleware(State(state): State<AppState>) -> impl Middleware
```

#### `middleware/rate_limit.rs` (100 lines)
**Responsibility:** Rate limiting middleware
**Public API:**
```rust
pub fn rate_limit_middleware(State(state): State<AppState>) -> impl Middleware
```

#### `middleware/interceptor.rs` (150 lines)
**Responsibility:** Message interception middleware
**Public API:**
```rust
pub fn interceptor_middleware(State(state): State<AppState>) -> impl Middleware
```

### Response Module (150 lines total)

#### `response/mod.rs` (50 lines)
**Responsibility:** Response utilities
**Public API:**
```rust
pub fn create_mcp_response(msg: Value, headers: HeaderMap) -> Response
pub fn create_error_response(error: ReverseProxyError) -> Response
```

#### `response/builder.rs` (100 lines)
**Responsibility:** Response construction helpers
**Public API:**
```rust
pub struct ResponseBuilder { ... }
impl ResponseBuilder {
    pub fn new() -> Self
    pub fn status(mut self, status: StatusCode) -> Self
    pub fn json(mut self, value: Value) -> Self
    pub fn build(self) -> Response
}
```

## Internal Structure Per Module

### Layered Architecture
```
┌─────────────────────────────────────┐
│         Handler Layer               │ <- User requests
├─────────────────────────────────────┤
│        Middleware Layer             │ <- Cross-cutting concerns
├─────────────────────────────────────┤
│        Transport Layer              │ <- Protocol handling
├─────────────────────────────────────┤
│         Session Layer               │ <- State management
├─────────────────────────────────────┤
│         Storage Layer               │ <- Persistence
└─────────────────────────────────────┘
```

### Module Communication
- **Downward only:** Higher layers call lower layers
- **Via traits:** All cross-module calls through traits
- **No cycles:** Strict acyclic dependency graph

## Extension Points

### Handler Extensions
```rust
// Custom handlers can be added
impl RequestHandler for CustomHandler {
    async fn handle(&self, ctx: RequestContext) -> Result<Response> {
        // Custom logic
    }
}
```

### Middleware Extensions
```rust
// Custom middleware can be inserted
impl Middleware for CustomMiddleware {
    async fn process(&self, req: Request, next: Next) -> Result<Response> {
        // Pre-processing
        let response = next.call(req).await?;
        // Post-processing
        Ok(response)
    }
}
```

### Transport Extensions
```rust
// New transports can be added
impl TransportProcessor for CustomTransport {
    async fn process(&self, msg: Message) -> Result<Response> {
        // Custom transport logic
    }
}
```

## Module Size Validation

| Module | Target | Actual | Status |
|--------|--------|--------|--------|
| mcp_handler.rs | 500 | - | ✅ Under 500 |
| sse_streaming.rs | 450 | - | ✅ Under 500 |
| upstream/http_processor.rs | 250 | - | ✅ Under 500 |
| upstream/stdio_processor.rs | 200 | - | ✅ Under 500 |
| server/router.rs | 200 | - | ✅ Under 500 |
| session_ops.rs | 200 | - | ✅ Under 500 |
| All others | <200 | - | ✅ Under 500 |

**All modules meet the 500-line requirement!**

## Key Architecture Changes

1. **Removed Admin UI**: ~900 lines eliminated (will be handled separately)
2. **Renamed to avoid conflicts**:
   - `transport/` → `upstream/` (for upstream server processing)
   - `session/handler.rs` → `session_ops.rs` (reverse proxy specific operations)
3. **Simplified handlers**: Only one main handler for `/mcp` endpoint
4. **SSE as streaming module**: Not a handler but a response streaming utility
5. **Clear separation**: Upstream processing vs main transport layer