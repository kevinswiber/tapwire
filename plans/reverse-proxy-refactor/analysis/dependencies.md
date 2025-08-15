# Dependency Analysis

## External Dependencies

### Web Framework
- **axum** - Core web framework
  - Router, handlers, middleware
  - SSE support via `response::sse`
  - Request/response extraction
- **tower** - Service builder for middleware
- **tower_http** - HTTP-specific middleware
  - CORS support
  - Tracing/logging layer

### Async Runtime
- **tokio** - Async runtime
  - TcpListener for server binding
  - Task spawning for concurrent operations
  - Channels (mpsc) for SSE streaming
  - Time utilities (intervals, sleep)
- **tokio_stream** - Stream utilities
  - UnboundedReceiverStream for SSE

### HTTP Client
- **reqwest** - HTTP client for upstream connections
  - Connection pooling
  - Timeout support
  - Streaming response bodies

### Serialization
- **serde** - Serialization framework
- **serde_json** - JSON support
  - Value type for dynamic JSON
  - JSON-RPC message handling

### Utilities
- **futures** - Stream extensions (StreamExt, TryStreamExt)
- **uuid** - UUID generation for correlation IDs
- **tracing** - Structured logging
- **chrono** - Timestamp generation (indirect via serde_json)

## Internal Dependencies

### Core Systems
- **session** (`crate::session`)
  - `Session` - Session state management
  - `SessionManager` - Thread-safe session storage

### Transport Layer (`crate::transport`)
- **Core Types**
  - `ProtocolMessage` - MCP message types
  - `SessionId` - Session identification
  - `TransportType` - Transport enumeration
  - `McpHeaders` - MCP protocol headers
  - `MessageEnvelope` - Message with context
  - `MessageDirection` - Client/Server direction
  
- **Functions**
  - `parse_json_rpc()` - Parse JSON-RPC messages
  - `transport_to_json_rpc()` - Convert to JSON-RPC
  - `extract_mcp_headers_optional()` - Header extraction
  - `create_mcp_response_headers()` - Header creation

- **Directional Transport** (`crate::transport::directional`)
  - `SubprocessOutgoing` - Stdio subprocess transport
  - `OutgoingTransport` - Transport trait

### Proxy Infrastructure (`crate::proxy`)
- **Connection Pooling** (`crate::proxy::pool`)
  - `ConnectionPool` - Generic connection pool
  - `PoolableOutgoingTransport` - Poolable transport wrapper
  - `create_outgoing_pool()` - Pool factory
  - `PoolConfig` - Pool configuration

### Security & Middleware
- **Authentication** (`crate::auth`)
  - `AuthGateway` - OAuth/JWT gateway
  - `jwt_auth_middleware()` - JWT validation middleware
  - `AuthGatewayConfig` - Auth configuration

- **Rate Limiting** (`crate::rate_limiting`)
  - `MultiTierRateLimiter` - Tiered rate limiting
  - `rate_limiting_middleware()` - Rate limit middleware
  - `RateLimitConfig` - Rate limit configuration

### Message Processing
- **Interceptors** (`crate::interceptor`)
  - `InterceptorChain` - Chain of interceptors
  - `InterceptAction` - Action enumeration
  - `InterceptContext` - Message context
  - `McpInterceptorConfig` - Interceptor configuration

- **Pause Controller** (`crate::transport::pause_controller`)
  - `PauseController` - Manual pause/resume control

### Observability
- **Recording** (`crate::recorder`)
  - `TapeRecorder` - Session recording to disk

- **Audit** (`crate::audit`)
  - `AuditConfig` - Audit logging configuration

- **Event IDs** (`crate::mcp::event_id`)
  - `EventIdGenerator` - Correlation ID generation

### Infrastructure
- **Error Handling** (`crate::error`)
  - `Result` - Standard result type
  - `ReverseProxyError` - Proxy-specific errors
  - `ReverseProxyResult` - Proxy result type

- **Shutdown** (`crate::shutdown`)
  - `ShutdownToken` - Graceful shutdown signaling

- **Protocol** (`crate::protocol`)
  - `DEFAULT_PROTOCOL_VERSION` - Default MCP version
  - `SUPPORTED_VERSIONS` - Supported version list

- **Constants** (`crate::transport::constants`)
  - `DEFAULT_MAX_BODY_SIZE` - Request size limit

## Coupling Points

### Tight Coupling Areas
1. **Session Manager** - Used throughout for state management
   - Lines: 1080-1086, 1262-1269, 1346-1353, 1497-1503
   - Hard to replace or mock

2. **Interceptor Chain** - Deeply integrated in request/response flow
   - Request interception: lines 1103-1251
   - Response interception: lines 1360-1484
   - Difficult to bypass or simplify

3. **AppState** - Shared state struct passed everywhere
   - Contains 11+ Arc-wrapped components
   - Required by all handlers

4. **Transport Processing** - Mixed with business logic
   - `process_via_http()` knows about SSE detection
   - `process_via_stdio_pooled()` manages pooling directly

### Loose Coupling Opportunities

1. **Upstream Selection**
   - Already isolated in `select_upstream()`
   - Could be extracted to strategy pattern

2. **Metrics Collection**
   - Mostly isolated to specific points
   - Could be made more pluggable

3. **Admin Interface**
   - Large self-contained function
   - Could be moved to separate module/crate

4. **Health Checks**
   - Simple endpoints with minimal dependencies
   - Easy to extract

## Dependency Graph

```
reverse.rs
├── Web Layer (axum, tower)
│   └── Handlers & Middleware
├── Transport Layer
│   ├── HTTP (reqwest)
│   ├── Stdio (SubprocessOutgoing)
│   └── SSE (axum::response::sse)
├── Session Management
│   └── SessionManager (Arc<SessionManager>)
├── Message Processing
│   ├── InterceptorChain
│   ├── PauseController
│   └── JSON-RPC parsing
├── Security
│   ├── AuthGateway
│   └── RateLimiter
└── Observability
    ├── Metrics
    ├── TapeRecorder
    └── EventIdGenerator
```

## Circular Dependencies
None identified - dependencies flow downward from reverse.rs to internal modules.

## External API Surface
The module exposes:
- HTTP endpoints via Axum router
- MCP protocol via HTTP/SSE
- Admin interface
- Health/metrics endpoints

## Recommendations for Decoupling

1. **Create Transport Abstraction**
   - Define trait for upstream communication
   - Implement for HTTP, Stdio, SSE
   - Remove transport details from handlers

2. **Extract Message Pipeline**
   - Separate interceptor processing
   - Create reusable pipeline for both directions
   - Reduce code duplication

3. **Isolate Session Operations**
   - Create session facade/service
   - Reduce direct SessionManager access
   - Easier testing and mocking

4. **Separate Concerns**
   - Move admin UI to separate module
   - Extract metrics to dedicated handler
   - Isolate health checks

5. **Dependency Injection**
   - Reduce AppState god object
   - Use traits for dependencies
   - Enable better testing