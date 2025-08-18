# Integration Points Analysis for legacy.rs

## SessionManager Integration

### Usage Patterns
- **Location**: Stored in `AppState`, accessed via `State` extractor
- **Primary Operations**:
  - `get_or_create()` - Session creation/retrieval
  - `get()` - Session lookup
  - `update()` - State updates
  - Session lifecycle management

### Key Integration Points

#### 1. Session Creation (Lines 2202-2282)
```rust
async fn get_or_create_session(
    session_manager: &Arc<SessionManager>,
    session_id: SessionId,
    ...
) -> Result<Arc<Session>>
```
- Creates new sessions with transport info
- Handles initialization state
- Manages session versioning

#### 2. Session Lookup
Multiple locations:
- `handle_mcp_request` (Line ~1150)
- `handle_mcp_sse_request` (Line ~1700)
- `proxy_sse_from_upstream` (Line ~1900)

#### 3. Session State Updates
- Protocol version tracking
- Connection state changes
- Error state management

### Dependencies
- Requires `Arc<SessionManager>` in AppState
- Thread-safe access via Arc
- Async operations throughout

## Interceptor Chain Integration

### Usage Patterns
- **Location**: Optional in `AppState`
- **Injection Points**:
  - Before upstream processing
  - After upstream response
  - On errors

### Key Integration Points

#### 1. Request Interception (Line ~1350)
```rust
if let Some(interceptor) = &app_state.interceptor_chain {
    let action = interceptor.intercept(context, Direction::ToServer).await?;
    match action {
        InterceptAction::Continue(msg) => { /* process */ }
        InterceptAction::Block(response) => { /* return response */ }
        InterceptAction::Pause => { /* handle pause */ }
    }
}
```

#### 2. Response Interception (Line ~1450)
- Intercepts responses before sending to client
- Can modify or block responses
- Supports async operations

### Interceptor Context
- Session information
- Message direction
- Transport type
- Timing information

## Transport Layer Usage

### Transport Types Supported
1. **Stdio Transport** (`SubprocessOutgoing`)
   - Process spawning
   - Pipe communication
   - Connection pooling

2. **HTTP Transport**
   - Hyper client
   - Header forwarding
   - Response streaming

3. **SSE Transport**
   - Event streaming
   - Keep-alive handling
   - Reconnection support

### Key Integration Points

#### 1. Transport Selection (Line ~1400)
```rust
match upstream.transport_type {
    TransportType::Stdio => process_via_stdio_pooled(...),
    TransportType::Http => process_via_http(...),
    _ => Err(...)
}
```

#### 2. Connection Pooling (Lines 490-520)
- Creates pools for stdio transports
- Manages connection lifecycle
- Handles pool exhaustion

#### 3. Transport Utilities
- `parse_json_rpc()` - Message parsing
- `transport_to_json_rpc()` - Message conversion
- `create_mcp_response_headers()` - Header generation

## Authentication/Authorization Hooks

### Auth Gateway Integration
- **Location**: Optional in `AppState`
- **Usage**: Token validation, user context

### Key Integration Points

#### 1. JWT Middleware (Line ~970)
```rust
.layer(middleware::from_fn_with_state(
    app_state.clone(),
    jwt_auth_middleware,
))
```

#### 2. Auth Context Extraction
- From request extensions
- Contains user_id, roles, permissions
- Used in admin endpoints

#### 3. Authorization Checks
- Policy engine integration
- Role-based access control
- Request filtering

## Recording/Replay Integration

### TapeRecorder Integration
- **Location**: Optional in `AppState`
- **Usage**: Message capture for replay

### Key Integration Points

#### 1. Request Recording (Line ~1380)
```rust
if let Some(recorder) = &app_state.tape_recorder {
    recorder.record_request(envelope).await?;
}
```

#### 2. Response Recording (Line ~1480)
```rust
if let Some(recorder) = &app_state.tape_recorder {
    recorder.record_response(envelope).await?;
}
```

### Recording Context
- Session ID
- Timestamp
- Transport type
- Message direction

## Rate Limiting Integration

### MultiTierRateLimiter
- **Location**: In `AppState`
- **Applied**: Via middleware layer

### Key Integration Points

#### 1. Middleware Layer (Line ~980)
```rust
.layer(middleware::from_fn_with_state(
    app_state.clone(),
    rate_limiting_middleware,
))
```

#### 2. Tier Selection
- Based on auth context
- User roles determine tier
- Configurable limits

## Metrics Collection

### ReverseProxyMetrics
- **Location**: In `AppState`
- **Updates**: Throughout request lifecycle

### Key Integration Points

#### 1. Request Metrics (Line ~1100)
```rust
app_state.metrics.increment_requests();
```

#### 2. Response Metrics (Line ~1500)
```rust
app_state.metrics.record_response_time(duration);
```

#### 3. Error Metrics
```rust
app_state.metrics.increment_errors();
```

### Metrics Tracked
- Request count
- Response times
- Error rates
- Active sessions
- Transport usage

## Shutdown Handling

### ShutdownToken Integration
- **Usage**: Graceful shutdown coordination
- **Location**: Passed to server methods

### Key Integration Points

#### 1. Server Shutdown (Line ~929)
```rust
pub async fn run_with_shutdown(self, mut shutdown: ShutdownToken) -> Result<()>
```

#### 2. Cleanup Operations
- Close active connections
- Flush recordings
- Save session state
- Release resources

## Database Interactions

### Via SessionManager
- SQLite for session persistence
- Async SQLx operations
- Connection pooling

### Operations
- Session CRUD
- State persistence
- Query optimization
- Transaction management

## AppState Structure

```rust
struct AppState {
    session_manager: Arc<SessionManager>,
    config: Arc<ReverseProxyConfig>,
    pools: Arc<HashMap<String, Arc<ConnectionPool<...>>>>,
    interceptor_chain: Option<Arc<InterceptorChain>>,
    tape_recorder: Option<Arc<TapeRecorder>>,
    auth_gateway: Option<Arc<AuthGateway>>,
    rate_limiter: Arc<MultiTierRateLimiter>,
    metrics: Arc<ReverseProxyMetrics>,
    pause_controller: Arc<PauseController>,
}
```

## Refactoring Implications

### Clean Interfaces Needed
1. **SessionManager**: Define trait for session operations
2. **Transport**: Abstract transport operations
3. **Interceptor**: Standardize interception points
4. **Recorder**: Create recording trait
5. **Metrics**: Define metrics interface

### Dependency Injection Opportunities
- Pass dependencies as traits, not concrete types
- Use constructor injection
- Avoid static/global state
- Enable easy mocking for tests

### Module Boundaries
Based on integration points:

1. **session/** - Session management module
2. **transport/** - Transport abstraction layer
3. **interceptor/** - Interception framework
4. **auth/** - Authentication/authorization
5. **recording/** - Recording/replay functionality
6. **metrics/** - Metrics collection
7. **middleware/** - HTTP middleware components