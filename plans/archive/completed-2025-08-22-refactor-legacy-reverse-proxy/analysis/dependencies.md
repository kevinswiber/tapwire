# Dependency Analysis for legacy.rs

## External Crate Dependencies

### Web Framework
- **axum** (6 imports)
  - `extract`: Request, State, Json, DefaultBodyLimit
  - `http`: HeaderMap, StatusCode
  - `response`: Response, IntoResponse, Sse, sse::Event
  - `routing`: get, post
  - Router
- **tower** (1 import)
  - ServiceBuilder
- **tower_http** (2 imports)
  - `cors::CorsLayer`
  - `trace::TraceLayer`

### Async Runtime
- **tokio** (2 imports)
  - `net::TcpListener`
  - Implicit: spawn, select, time
- **tokio_stream** (1 import)
  - `wrappers::UnboundedReceiverStream`
- **futures** (1 import)
  - StreamExt, TryStreamExt

### Serialization
- **serde** (1 import)
  - Serialize, Deserialize
- **serde_json** (1 import)
  - Value

### Utilities
- **uuid** (1 import)
  - Uuid
- **tracing** (1 import)
  - debug, error, info, instrument, warn

### Standard Library
- **std** (4 imports)
  - `net::SocketAddr`
  - `path::PathBuf`
  - `sync::atomic::{AtomicU64, Ordering}`
  - `sync::Arc`
  - `time::Instant`

## Internal Module Dependencies

### High-Level Dependency Count
1. **transport** (4 uses) - Most heavily used
2. **mcp** (2 uses) - Protocol handling
3. **auth** (2 uses) - Authentication/authorization
4. **error** (1 use) - Error types
5. **interceptor** (1 use) - Message interception
6. **proxy** (1 use) - Connection pooling
7. **rate_limiting** (1 use) - Rate limiting
8. **recorder** (1 use) - Session recording
9. **session** (1 use) - Session management
10. **shutdown** (1 use) - Graceful shutdown

### Detailed Internal Dependencies

#### Transport Module
```rust
use crate::transport::{
    constants::DEFAULT_MAX_BODY_SIZE,
    create_mcp_response_headers,
    extract_mcp_headers_optional,
    parse_json_rpc,
    transport_to_json_rpc,
    McpHeaders,
    ResponseMode,
    OutgoingTransport,
    SubprocessOutgoing,
    pause_controller::PauseController,
    sse::event::SseEvent,
};
```

#### MCP Module
```rust
use crate::mcp::{
    event_id::EventIdGenerator,
    Delivery,
    Direction,
    MessageContext,
    MessageEnvelope,
    ProtocolMessage,
    SessionId,
    TransportType,
};
```

#### Auth Module
```rust
use crate::auth::{
    gateway::AuthGateway,
    middleware::jwt_auth_middleware,
    oauth::AuthContext,  // Used in extensions
};
```

#### Error Module
```rust
use crate::error::{
    Result,
    ReverseProxyError,
    ReverseProxyResult,
};
```

#### Other Modules
```rust
use crate::interceptor::{InterceptAction, InterceptContext, InterceptorChain};
use crate::proxy::pool::{create_outgoing_pool, ConnectionPool, PoolConfig, PoolableOutgoingTransport};
use crate::rate_limiting::{middleware::rate_limiting_middleware, MultiTierRateLimiter};
use crate::recorder::TapeRecorder;
use crate::session::{Session, SessionManager};
use crate::shutdown::ShutdownToken;
```

## Dependency Relationships

### Core Dependencies Flow
```
legacy.rs
    ├── transport (I/O operations)
    │   ├── OutgoingTransport
    │   ├── SubprocessOutgoing
    │   └── SSE handling
    ├── mcp (Protocol)
    │   ├── Messages
    │   ├── Sessions
    │   └── Event IDs
    ├── session (State)
    │   └── SessionManager
    ├── auth (Security)
    │   ├── AuthGateway
    │   └── JWT middleware
    └── proxy::pool (Connections)
        └── ConnectionPool
```

### Circular Dependency Risks
- **Low Risk**: No direct circular dependencies detected
- **Potential Issues**:
  - Heavy coupling with transport module
  - Tight integration with SessionManager
  - Direct access to multiple internal modules

### Database/Storage Interactions
- **SessionManager**: SQLite-backed session storage
  - Session creation and retrieval
  - State persistence
- **TapeRecorder**: Session recording to disk
  - Message capture
  - Replay functionality

### Network/Transport Usage
- **HTTP Transport**:
  - Axum for HTTP server
  - Hyper for HTTP client operations
- **SSE Transport**:
  - Server-Sent Events for streaming
  - UnboundedReceiverStream for async streaming
- **Stdio Transport**:
  - SubprocessOutgoing for process communication
  - Connection pooling for subprocess reuse

## Refactoring Implications

### High-Priority Decoupling
1. **Transport Layer**: Extract transport-specific logic to dedicated modules
2. **Protocol Handling**: Separate MCP protocol logic from HTTP handling
3. **Session Management**: Create cleaner interface for session operations

### Module Boundaries
Based on dependencies, natural module boundaries:

1. **config/** - All configuration types (minimal deps)
2. **server/** - Server lifecycle (axum, tokio)
3. **handlers/** - Request handlers (axum, mcp, transport)
4. **sse/** - SSE-specific code (axum::sse, tokio_stream)
5. **processing/** - Message processing (mcp, transport)
6. **admin/** - Admin endpoints (minimal deps)

### Dependency Injection Opportunities
- Pass SessionManager as trait instead of concrete type
- Abstract transport operations behind traits
- Use dependency injection for:
  - AuthGateway
  - InterceptorChain
  - ConnectionPool
  - TapeRecorder

### Testing Improvements
Current tight coupling makes testing difficult. Refactoring should:
- Enable mocking of transport layer
- Allow testing handlers without full server setup
- Separate business logic from I/O operations