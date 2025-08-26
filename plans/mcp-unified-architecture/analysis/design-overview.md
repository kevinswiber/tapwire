# MCP Unified Architecture Design Overview

## Current State Analysis

### What We Have Now

#### Server (`src/server.rs`)
```rust
pub struct Server<C: Connection, H: ServerHandler> {
    sessions: Arc<RwLock<HashMap<String, ClientSession<C>>>>, // Basic session map
    handler: Arc<H>,                                          // Message handler
    version: ProtocolVersion,
    max_clients: usize,                                       // Manual counting
    connection_receiver: Arc<RwLock<mpsc::UnboundedReceiver<C>>>, // Unbounded!
    connection_sender: mpsc::UnboundedSender<C>,
    running: Arc<RwLock<bool>>,                              // Should be CancellationToken
}

// Problems:
// 1. Spawns task per client (line 218)
// 2. Basic ClientSession with no persistence
// 3. No interceptor support
// 4. Manual client counting (race conditions)
// 5. Unbounded channels (memory issues)
// 6. Arc<RwLock<bool>> for shutdown (antipattern)
```

#### Client (`src/client.rs`)
```rust
pub struct Client<C: Connection> {
    pool: Arc<Pool<PoolableConnection<C>>>,  // Good - already uses pool
    connection_factory: ConnectionFactory<C>,
    version: ProtocolVersion,
    next_id: Arc<RwLock<u64>>,
    _pending_requests: PendingRequestsMap,   // Unused?
}

// Problems:
// 1. No session tracking for reconnection
// 2. No interceptor support
// 3. Pending requests map not utilized
```

### What We Need

## Target Architecture

### Core Components to Add

```rust
// 1. Shared State Pattern (used by both Client and Server)
pub struct CoreComponents {
    pub session_manager: Arc<SessionManager>,
    pub interceptor_chain: Arc<InterceptorChain>,
    pub pause_controller: Arc<PauseController>,
    pub event_tracker: Arc<EventTracker>,        // For SSE
    pub tape_recorder: Option<Arc<TapeRecorder>>,
}
```

### Redesigned Server

```rust
pub struct Server<C: Connection, H: ServerHandler> {
    // Core components
    components: CoreComponents,
    
    // Handler
    handler: Arc<H>,
    
    // Connection management (improved)
    max_clients: Arc<Semaphore>,                    // Atomic limits
    connection_rx: Mutex<mpsc::Receiver<C>>,        // Bounded, not RwLock
    connection_tx: mpsc::Sender<C>,
    active_connections: JoinSet<()>,                // Track tasks
    
    // Lifecycle
    shutdown: CancellationToken,                    // Clean shutdown
    version: ProtocolVersion,
}
```

### Redesigned Client

```rust
pub struct Client<C: Connection> {
    // Connection pooling (keep existing)
    pool: Arc<Pool<PoolableConnection<C>>>,
    connection_factory: ConnectionFactory<C>,
    
    // Core components
    components: CoreComponents,
    
    // Session tracking
    current_session: Arc<RwLock<Option<SessionId>>>,
    
    // Request tracking (properly used)
    next_id: Arc<AtomicU64>,
    pending_requests: Arc<RwLock<HashMap<JsonRpcId, PendingRequest>>>,
    
    // Protocol
    version: ProtocolVersion,
}
```

## Integration Points

### 1. Message Flow Through Interceptors

```rust
// Before (direct processing)
async fn handle_message(msg: Value) -> Value {
    handler.handle_request(msg).await
}

// After (with pipeline)
async fn handle_message(msg: Value, components: &CoreComponents) -> Value {
    // 1. Parse message
    let protocol_msg = ProtocolMessage::try_from(msg)?;
    
    // 2. Get/create session
    let session = components.session_manager
        .get_or_create(session_id)
        .await?;
    
    // 3. Create context
    let context = InterceptContext::new(
        protocol_msg.clone(),
        Direction::ClientToServer,
        session.id,
        transport_type,
        frame_id,
    );
    
    // 4. Apply interceptors
    let action = components.interceptor_chain.intercept(&context).await?;
    
    // 5. Process based on action
    match action {
        InterceptAction::Continue => {
            // Process normally
            handler.handle_request(protocol_msg).await
        }
        InterceptAction::Modify(new_msg) => {
            // Process modified
            handler.handle_request(new_msg).await
        }
        InterceptAction::Block { reason } => {
            // Return error
            create_error_response(reason)
        }
        InterceptAction::Mock { response } => {
            // Return mock
            response
        }
        InterceptAction::Pause { timeout } => {
            // Wait for manual resume
            components.pause_controller.pause(context, timeout).await
        }
    }
}
```

### 2. Server Connection Handling (Hyper Pattern)

```rust
impl Server {
    pub async fn accept(&mut self, connection: C) -> Result<()> {
        // 1. Acquire permit atomically (no race)
        let permit = self.max_clients.clone()
            .try_acquire_owned()
            .map_err(|_| ServerError::MaxClientsReached)?;
        
        // 2. Create session
        let session = self.components.session_manager
            .create_session(TransportType::from(&connection))
            .await?;
        
        // 3. Single spawn with hyper pattern
        self.active_connections.spawn(async move {
            let _permit = permit; // Hold for connection lifetime
            
            // Setup service with interceptors
            let service = service_fn(move |req| {
                handle_request_with_interceptors(
                    req,
                    session.clone(),
                    self.components.clone(),
                    self.handler.clone(),
                )
            });
            
            // Serve connection (this is the ONLY place doing the work)
            let result = http1::Builder::new()
                .serve_connection(connection, service)
                .await;
            
            // Cleanup
            self.components.session_manager
                .cleanup_session(session.id)
                .await;
        });
        
        Ok(())
    }
}
```

### 3. Client Request Flow

```rust
impl Client {
    pub async fn request(&self, method: &str, params: Value) -> Result<Value> {
        // 1. Get or create session
        let session_id = self.get_or_create_session().await?;
        
        // 2. Create message
        let msg = ProtocolMessage::request(
            self.next_id.fetch_add(1, Ordering::SeqCst),
            method,
            params,
        );
        
        // 3. Apply request interceptors
        let context = InterceptContext::new(
            msg.clone(),
            Direction::ClientToServer,
            session_id,
            TransportType::Http,
            0,
        );
        
        let msg = match self.components.interceptor_chain.intercept(&context).await? {
            InterceptAction::Continue => msg,
            InterceptAction::Modify(new_msg) => new_msg,
            InterceptAction::Block { reason } => return Err(Error::Blocked(reason)),
            _ => msg,
        };
        
        // 4. Get connection from pool
        let conn = self.pool.acquire().await?;
        
        // 5. Send and receive
        let response = conn.send(msg).await?;
        
        // 6. Apply response interceptors
        let response_context = InterceptContext::new(
            response.clone(),
            Direction::ServerToClient,
            session_id,
            TransportType::Http,
            1,
        );
        
        let response = match self.components.interceptor_chain.intercept(&response_context).await? {
            InterceptAction::Continue => response,
            InterceptAction::Modify(new_msg) => new_msg,
            _ => response,
        };
        
        // 7. Update session
        self.components.session_manager
            .update_session(session_id, |s| {
                s.last_activity = Instant::now();
            })
            .await;
        
        Ok(response)
    }
}
```

## Migration Strategy

### Phase 1: Add Core Types (Non-Breaking)
1. Add `SessionManager` struct and traits
2. Add `InterceptorChain` and related types
3. Add `PauseController` and `EventTracker`
4. These are NEW additions, don't break existing code

### Phase 2: Update Server (Breaking but Incremental)
1. Add `components` field to Server
2. Replace `Arc<RwLock<bool>>` with `CancellationToken`
3. Replace manual counting with `Semaphore`
4. Update `accept()` to use hyper pattern
5. Keep old `serve()` working temporarily

### Phase 3: Update Client (Mostly Non-Breaking)
1. Add `components` field
2. Add session tracking
3. Update request methods to use interceptors
4. Existing pool usage remains

### Phase 4: SSE/WebSocket Support
1. Add upgrade detection
2. Implement SSE writer
3. Add event tracking
4. Support reconnection

## Key Design Decisions

### Why Shared Components?
- Sessions and interceptors need to coordinate
- Reduces parameter passing
- Consistent across client/server
- Easy to test with mocks

### Why Hyper Pattern?
- Single spawn per connection (80% reduction)
- Battle-tested in production
- Natural backpressure handling
- Clean separation of concerns

### Why Semaphore for Limits?
- Atomic acquire/release
- No race conditions
- Fair scheduling
- Timeout support

### Why CancellationToken?
- Hierarchical cancellation
- No locks needed
- Clean shutdown propagation
- Tokio best practice

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_interceptor_chain() {
    let chain = InterceptorChain::new();
    chain.register(MockInterceptor::blocking());
    
    let context = InterceptContext::new(...);
    let action = chain.intercept(&context).await?;
    
    assert_matches!(action, InterceptAction::Block { .. });
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_server_with_sessions() {
    let components = create_test_components();
    let server = Server::with_components(components.clone());
    
    // Connect client
    let client = create_test_client();
    server.accept(client).await?;
    
    // Verify session created
    assert_eq!(components.session_manager.count().await, 1);
}
```

## Success Metrics

### Performance
- Task spawns: 5 → 1 per connection ✓
- Memory: < 100KB per session ✓
- Latency: < 5% overhead ✓

### Functionality
- Sessions persist ✓
- Interceptors work ✓
- SSE reconnection ✓
- Graceful shutdown ✓

### Quality
- No breaking changes initially ✓
- Incremental migration ✓
- Full test coverage ✓

## Questions to Resolve

1. **Session Storage**: Start with in-memory only, or include SQLite from beginning?
2. **Interceptor Registration**: Static at startup or dynamic?
3. **Backwards Compatibility**: How long to maintain old APIs?
4. **Configuration**: How to configure components (builder pattern)?

## Next Steps

1. Review this design
2. Agree on approach
3. Start Phase 1 implementation (core types)
4. Test with mock implementations
5. Proceed to Phase 2 (server update)