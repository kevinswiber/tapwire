# MCP Integration Requirements

## Overview

Based on analysis of shadowcat's proxy implementations, this document defines the concrete requirements for integrating session management and interceptors into the MCP crate.

## Architectural Requirements

### 1. Unified State Management
All MCP server and client implementations must share a common state pattern:

```rust
struct SharedState {
    session_manager: Arc<SessionManager>,
    interceptor_chain: Arc<InterceptorChain>,
    pause_controller: Arc<PauseController>,
    tape_recorder: Option<Arc<TapeRecorder>>,
    event_tracker: Arc<EventTracker>, // For SSE
}
```

### 2. Message Processing Pipeline
Every message must flow through:
1. Session validation/creation
2. Interceptor chain (request)
3. Core processing
4. Interceptor chain (response)
5. Session update

### 3. Connection Lifecycle
```
Connect → Create Session → Process Messages → Cleanup Session → Disconnect
             ↓                    ↓                ↓
        Interceptors         Interceptors    Interceptors
```

## Component Integration

### Server Integration

#### Current Server Structure
```rust
pub struct Server<C: Connection, H: ServerHandler> {
    sessions: Arc<RwLock<HashMap<String, ClientSession<C>>>>,
    handler: Arc<H>,
    // Missing: session_manager, interceptor_chain
}
```

#### Required Server Structure
```rust
pub struct Server<C: Connection, H: ServerHandler> {
    // Session management
    session_manager: Arc<SessionManager>,
    
    // Message processing
    interceptor_chain: Arc<InterceptorChain>,
    pause_controller: Arc<PauseController>,
    
    // Handler and connection management
    handler: Arc<H>,
    connections: Arc<RwLock<HashMap<SessionId, C>>>,
    
    // Optional features
    tape_recorder: Option<Arc<TapeRecorder>>,
    event_tracker: Arc<EventTracker>,
}
```

#### serve_connection Integration
```rust
async fn serve_connection(connection: C, state: SharedState) {
    // 1. Create session
    let session = state.session_manager.create_session(...).await;
    
    // 2. Setup service with interceptors
    let service = service_fn(move |req| {
        let state = state.clone();
        let session = session.clone();
        
        async move {
            // Apply interceptors
            let msg = parse_message(req)?;
            let context = InterceptContext::new(msg, ...);
            
            let action = state.interceptor_chain.intercept(&context).await?;
            match action {
                Continue => process_normally(msg, session).await,
                Block { reason } => return_error(reason),
                Modify(new_msg) => process_normally(new_msg, session).await,
                // ...
            }
        }
    });
    
    // 3. Serve with hyper
    http1::Builder::new()
        .serve_connection(connection, service)
        .await;
    
    // 4. Cleanup
    state.session_manager.cleanup_session(session.id).await;
}
```

### Client Integration

#### Current Client Structure
```rust
pub struct Client<C: Connection> {
    pool: Pool<ConnectionKey, C>,
    next_id: Arc<AtomicU64>,
    // Missing: session tracking, interceptors
}
```

#### Required Client Structure
```rust
pub struct Client<C: Connection> {
    // Connection pooling
    pool: Pool<ConnectionKey, C>,
    
    // Session management
    session_manager: Arc<SessionManager>,
    current_session: Arc<RwLock<Option<SessionId>>>,
    
    // Message processing
    interceptor_chain: Arc<InterceptorChain>,
    
    // Request tracking
    next_id: Arc<AtomicU64>,
    pending_requests: Arc<RwLock<HashMap<JsonRpcId, PendingRequest>>>,
}
```

## Data Flow Requirements

### Request Flow (Server)
```
HTTP Request → Extract Session ID → Get/Create Session
    ↓
Parse Message → Create InterceptContext → Apply Request Interceptors
    ↓
Process (Handler or Upstream) → Apply Response Interceptors
    ↓
Update Session → Return Response
```

### Request Flow (Client)
```
Method Call → Create Session (if needed) → Create Message
    ↓
Apply Request Interceptors → Send via Connection
    ↓
Receive Response → Apply Response Interceptors → Update Session
    ↓
Return Result
```

## Session Management Requirements

### Session Operations
- `create_session(transport: TransportType) -> Session`
- `get_session(id: SessionId) -> Option<Session>`
- `update_session(id: SessionId, updater: impl FnOnce(&mut Session))`
- `cleanup_session(id: SessionId)`
- `list_sessions() -> Vec<SessionId>`

### Session Features
- Automatic cleanup of idle sessions
- Rate limiting per session
- Event tracking for SSE reconnection
- Metadata storage for custom data
- Protocol version tracking

### Storage Backends
- In-memory (default)
- SQLite (persistent)
- Redis (distributed)

## Interceptor Requirements

### Interceptor Operations
- `register_interceptor(interceptor: Arc<dyn Interceptor>)`
- `unregister_interceptor(name: &str)`
- `intercept(context: &InterceptContext) -> InterceptAction`

### Built-in Interceptors
- **McpInterceptor**: Protocol validation
- **RulesInterceptor**: Rule-based filtering
- **HttpPolicyInterceptor**: HTTP-specific policies
- **RecordingInterceptor**: Tape recording
- **RateLimitInterceptor**: Request throttling

### Interceptor Actions
- `Continue`: Pass through unchanged
- `Modify(msg)`: Transform message
- `Block { reason }`: Reject with error
- `Mock { response }`: Return synthetic response
- `Pause { timeout }`: Manual intervention
- `Delay { duration, then }`: Delayed action

## SSE/WebSocket Requirements

### SSE Session Tracking
- Track last-event-id per session
- Support reconnection with event replay
- Maintain event buffer per session
- Clean up on disconnect

### WebSocket Session Management
- Bidirectional session tracking
- Ping/pong for keepalive
- Automatic reconnection support

## Performance Requirements

### Concurrency
- SessionManager must support concurrent access
- InterceptorChain must not block
- Lock contention must be minimized

### Memory Management
- Session cleanup must be automatic
- Event buffers must be bounded
- No memory leaks on disconnection

### Latency
- Interceptor overhead < 1ms p99
- Session lookup < 100μs p99
- No blocking operations in hot path

## Migration Requirements

### Backward Compatibility
- Existing Connection trait must work
- Current pool implementation unchanged
- Client/Server APIs preserved where possible

### Incremental Adoption
- Session management optional initially
- Interceptors can be empty chain
- Features can be enabled gradually

### Testing
- All existing tests must pass
- New integration tests for sessions
- Performance benchmarks required

## Implementation Priority

### Phase 1: Core Infrastructure
1. Port Session types and traits
2. Port InterceptContext and actions
3. Create basic SessionManager
4. Create basic InterceptorChain

### Phase 2: Server Integration
1. Add session_manager to Server
2. Integrate into serve_connection
3. Add interceptor pipeline
4. Update handlers

### Phase 3: Client Integration
1. Add session tracking
2. Add interceptor support
3. Update request methods

### Phase 4: Advanced Features
1. Persistence workers
2. SQLite/Redis stores
3. SSE event tracking
4. Rules engine

## Success Criteria

### Functional
- [ ] Sessions persist across requests
- [ ] Interceptors process all messages
- [ ] SSE reconnection works
- [ ] Rate limiting enforced
- [ ] Graceful shutdown preserves state

### Performance
- [ ] 10,000+ concurrent sessions
- [ ] < 5% latency overhead
- [ ] < 100KB memory per session
- [ ] No memory leaks

### Quality
- [ ] 90% test coverage
- [ ] Zero clippy warnings
- [ ] Full documentation
- [ ] Integration examples