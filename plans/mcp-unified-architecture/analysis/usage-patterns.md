# Session & Interceptor Usage Patterns Analysis

## Executive Summary

After analyzing the actual usage in shadowcat's proxy implementations, we've identified critical integration patterns that must be preserved when porting to the MCP crate.

## Session Manager Usage Patterns

### 1. Core Responsibilities
The SessionManager is used throughout the proxy for:
- **Session lifecycle**: Create, get, update, cleanup
- **State persistence**: Store session data with configurable backends
- **Event tracking**: Record session events for SSE reconnection
- **Rate limiting**: Per-session request throttling
- **Metrics**: Session counts, cleanup stats

### 2. Integration Points

#### Builder Pattern
```rust
// From proxy/builders.rs
pub struct ForwardProxyBuilder {
    session_manager: Option<Arc<SessionManager>>,
    // ...
}

// Default creation if not provided
let session_manager = self.session_manager.unwrap_or_else(|| {
    let session_config = self.config.to_session_config();
    Arc::new(SessionManager::with_config(session_config))
});
```

#### State Management
```rust
// From proxy/reverse/state.rs
pub struct AppState {
    pub session_manager: Arc<SessionManager>,
    pub interceptor_chain: Arc<InterceptorChain>,
    // These work together
}
```

#### Request Processing
```rust
// From handlers/mcp.rs
// 1. Get or create session
let (session_id, session) = get_or_create_session(
    &app.session_manager,
    session_id,
    &mcp_headers,
).await?;

// 2. Track protocol versions
track_initialize_request_version(
    &app.session_manager, 
    &session_id, 
    &msg
).await?;

// 3. Pass session to interceptors (via context)
```

### 3. Session Structure
```rust
// From store.rs
pub struct Session {
    pub id: SessionId,
    pub transport_type: TransportType,
    pub upstream_session_id: Option<SessionId>, // For dual-session tracking
    pub status: SessionStatus,
    pub metadata: HashMap<String, String>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub protocol_version: Option<String>,
    pub last_event_id: Option<String>, // SSE reconnection
    pub pending_requests: VecDeque<PendingRequest>,
}
```

## Interceptor Chain Usage Patterns

### 1. Core Responsibilities
The InterceptorChain manages:
- **Message transformation**: Modify requests/responses
- **Access control**: Block/allow based on rules
- **Pause/resume**: Manual intervention support
- **Mocking**: Return synthetic responses
- **Recording**: Integration with tape recorder

### 2. Integration Flow

#### Pipeline Processing
```rust
// From pipeline.rs
pub async fn apply_request_interceptors(
    interceptor_chain: &Arc<InterceptorChain>,
    pause_controller: &Arc<PauseController>,
    message: ProtocolMessage,
    session_id: SessionId,
    transport_type: TransportType,
) -> Result<InterceptResult> {
    let context = InterceptContext::new(
        message,
        Direction::ClientToServer,
        session_id,
        transport_type,
        frame_id,
    );
    
    match interceptor_chain.intercept(&context).await {
        Ok(InterceptAction::Continue) => Continue(context.message),
        Ok(InterceptAction::Modify(msg)) => Continue(msg),
        Ok(InterceptAction::Block { reason }) => EarlyReturn(error_response(reason)),
        // ...
    }
}
```

#### Upstream Integration
```rust
// From upstream/stdio.rs
pub async fn send_request(
    &self,
    message: ProtocolMessage,
    session: &Session,
    interceptor_chain: Option<Arc<InterceptorChain>>,
) -> Result<Response> {
    // Interceptors are optional at upstream level
    if let Some(chain) = interceptor_chain {
        // Apply transformations
    }
    // Send to upstream
}
```

### 3. Context Propagation
```rust
pub struct InterceptContext {
    pub message: ProtocolMessage,
    pub direction: Direction,
    pub session_id: SessionId,      // Links to session
    pub transport_type: TransportType,
    pub timestamp: Instant,
    pub frame_id: u64,
    pub metadata: BTreeMap<String, String>,
}
```

## Critical Integration Requirements

### 1. Shared State Pattern
Sessions and interceptors always work together through shared AppState:
```rust
pub struct AppState {
    pub session_manager: Arc<SessionManager>,
    pub interceptor_chain: Arc<InterceptorChain>,
    pub pause_controller: Arc<PauseController>,
    pub tape_recorder: Option<Arc<TapeRecorder>>,
    // All these components interact
}
```

### 2. Lifecycle Management
- SessionManager must be created before server starts
- InterceptorChain registered during initialization
- Both must gracefully shutdown together

### 3. SSE Integration
```rust
// SSE needs special session tracking
pub struct SseSessionParams {
    pub session_id: SessionId,
    pub event_tracker: Arc<EventTracker>, // Works with SessionManager
    pub interceptor_chain: Arc<InterceptorChain>,
    // ...
}
```

### 4. Performance Considerations
- SessionManager uses RwLock for concurrent access
- InterceptorChain must not block under high load
- Both need efficient cleanup mechanisms

## Integration Strategy for MCP Crate

### Phase 1: Core Types
1. Port Session struct and traits
2. Port InterceptContext and InterceptAction
3. Define storage traits

### Phase 2: Manager Implementation
1. Port SessionManager with in-memory store
2. Port InterceptorChain engine
3. Add PauseController

### Phase 3: Integration Points
1. Update Server to use SessionManager
2. Add interceptor support to serve_connection
3. Update Client for session tracking

### Phase 4: Advanced Features
1. Add persistence worker
2. Implement SQLite/Redis stores
3. Add SSE session tracking
4. Implement rules engine

## Key Differences for MCP Crate

### Current MCP Implementation
- Basic Connection trait
- No session persistence
- No interceptor support
- Simple spawn-per-connection

### Required Changes
1. **Server struct** needs:
   - session_manager: Arc<SessionManager>
   - interceptor_chain: Arc<InterceptorChain>
   - pause_controller: Arc<PauseController>

2. **Connection processing** needs:
   - Session creation on connect
   - Interceptor pipeline for messages
   - Session cleanup on disconnect

3. **Client struct** needs:
   - Session tracking for reconnection
   - Interceptor support for testing

## Usage Examples to Preserve

### Creating a Server
```rust
let session_manager = Arc::new(SessionManager::with_config(config));
let interceptor_chain = Arc::new(InterceptorChain::new());

let server = Server::builder()
    .session_manager(session_manager)
    .interceptor_chain(interceptor_chain)
    .build();
```

### Processing Messages
```rust
// In serve_connection
let session = session_manager.get_or_create(session_id).await?;

let context = InterceptContext::new(
    message,
    direction,
    session_id,
    transport_type,
    frame_id,
);

let action = interceptor_chain.intercept(&context).await?;
match action {
    InterceptAction::Continue => forward_message(message),
    InterceptAction::Block { reason } => return_error(reason),
    // ...
}
```

## Testing Requirements

### Unit Tests
- Mock SessionStore for testing
- Test interceptor chain ordering
- Verify session cleanup

### Integration Tests
- Full request flow with sessions
- Interceptor modification testing
- SSE reconnection scenarios

### Performance Tests
- Session creation under load
- Interceptor chain throughput
- Memory usage with many sessions