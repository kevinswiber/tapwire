# Final Design Decisions for MCP Architecture

## Confirmed Architectural Decisions ✅

### Core Design Choices

#### 1. Connection Trait (Keep Current)
- **Decision**: Keep the existing Connection trait
- **Rationale**: Simpler, already integrated with pool, works well
- **Enhancement**: Ensure it supports SSE/WebSocket upgrades
```rust
// Keep this pattern
pub trait Connection: Send + Sync {
    async fn send(&mut self, msg: Value) -> Result<()>;
    async fn receive(&mut self) -> Result<Value>;
    async fn close(&mut self) -> Result<()>;
}
```

#### 2. SSE Core, WebSocket Optional
- **Decision**: SSE is core feature, WebSocket is feature-gated
- **Rationale**: SSE is in MCP spec, WebSocket is experimental
```toml
[features]
default = ["sse"]
sse = []  # Always included in default
websocket = ["tokio-tungstenite", "hyper-tungstenite"]
```

#### 3. Persistence Worker (Auto-Start)
- **Decision**: Start persistence worker automatically
- **Rationale**: 99% of use cases need it for SSE
- **Note**: Minor overhead for HTTP-only scenarios is acceptable
```rust
impl SessionManager {
    pub fn new(store: Arc<dyn SessionStore>) -> Self {
        let manager = Self { ... };
        manager.start_persistence_worker(); // Automatic
        manager
    }
}
```

#### 4. Async-Only Interceptors
- **Decision**: All interceptors must be async
- **Rationale**: Consistency, futures are cheap, most need async anyway
```rust
#[async_trait]
pub trait Interceptor: Send + Sync {
    async fn intercept(&self, context: &InterceptContext) -> Result<InterceptAction>;
}
```

#### 5. Metrics On by Default
- **Decision**: Metrics enabled by default (not feature-gated)
- **Rationale**: Production readiness, observability is critical
- **Implementation**: Use lightweight counters, make collectors optional
```rust
// Always track metrics
pub struct Metrics {
    requests_total: AtomicU64,
    sessions_active: AtomicU64,
    // Collector is optional
    collector: Option<Box<dyn MetricsCollector>>,
}
```

#### 6. Session ID Mapping (Proxy Concern)
- **Decision**: Keep dual session tracking OUT of core MCP client/server
- **Rationale**: This is proxy-specific behavior
- **Implementation**: Proxy uses Session metadata or wrapper
```rust
// In proxy code (not MCP crate)
pub struct ProxySession {
    client_session_id: SessionId,      // Proxy-generated
    upstream_session_id: Option<SessionId>, // From upstream
    session: Session,                   // Core MCP session
}
```

#### 7. EventTracker as Separate Component
- **Decision**: EventTracker is separate from SessionManager
- **Rationale**: Single responsibility, optional usage
```rust
pub struct EventTracker { ... }  // Separate

pub struct SessionManager {
    // Coordinates with EventTracker when needed
    event_tracker: Option<Arc<EventTracker>>,
}
```

#### 8. Comprehensive Error Types
- **Decision**: Port shadowcat's comprehensive error hierarchy
- **Rationale**: Better debugging, production readiness
```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),
    
    #[error("Session error: {0}")]
    Session(#[from] SessionError),
    
    #[error("Interceptor error: {0}")]
    Interceptor(#[from] InterceptorError),
    // ... more specific errors
}
```

#### 9. Dual Testing Strategy
- **Decision**: Both mock and real implementations
- **Usage**: Mocks for unit tests, real for integration
```rust
// For unit tests
#[cfg(test)]
pub struct MockSessionStore { ... }

// For integration tests
pub fn create_test_session_manager() -> SessionManager {
    SessionManager::new(Arc::new(InMemorySessionStore::new()))
}
```

### Configuration Architecture

#### Centralized Configuration
```rust
// crates/mcp/src/config/mod.rs
pub mod server;
pub mod client;
pub mod session;
pub mod interceptor;
pub mod pool;  // Already exists

pub struct Config {
    pub server: ServerConfig,
    pub client: ClientConfig,
    pub session: SessionConfig,
    pub interceptor: InterceptorConfig,
    pub pool: PoolConfig,
}

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> { ... }
    pub fn with_env_overrides(self) -> Self { ... }
}
```

### No Backward Compatibility Constraints
- **Decision**: Breaking changes are acceptable
- **Rationale**: Not released yet, get it right
- **Impact**: Can redesign APIs freely

## Implementation Architecture

### Phase Structure

#### Phase 1: Core Types
```rust
// Port from shadowcat
- SessionStore trait
- InMemorySessionStore
- InterceptorChain
- InterceptContext
- PersistenceWorker
```

#### Phase 2: Server Integration
```rust
pub struct Server<C: Connection, H: ServerHandler> {
    // New unified components
    session_manager: Arc<SessionManager>,
    interceptor_chain: Arc<InterceptorChain>,
    event_tracker: Arc<EventTracker>,
    
    // Improved lifecycle
    max_clients: Arc<Semaphore>,
    shutdown: CancellationToken,
    active_connections: JoinSet<()>,
    
    // Handler
    handler: Arc<H>,
}
```

#### Phase 3: Client Integration
```rust
pub struct Client<C: Connection> {
    // Existing pool
    pool: Arc<Pool<PoolableConnection<C>>>,
    
    // New components
    session_manager: Arc<SessionManager>,
    interceptor_chain: Arc<InterceptorChain>,
    current_session: Arc<RwLock<Option<SessionId>>>,
}
```

## Design Validations

### Proxy Compatibility Check
The dual session mapping plan is achievable with this design:
```rust
// Proxy can extend the base session
impl ReverseProxy {
    fn handle_request(&self, client_session_id: SessionId) {
        // Get core session
        let session = self.session_manager.get(client_session_id);
        
        // Map to upstream (proxy's responsibility)
        let upstream_id = self.session_mappings.get(client_session_id);
        
        // Use metadata for tracking
        session.metadata.insert("upstream_session_id", upstream_id);
    }
}
```

### SSE Reconnection Support
```rust
// EventTracker handles SSE reconnection
impl EventTracker {
    pub async fn track_event(&self, session_id: SessionId, event_id: String) { ... }
    pub async fn get_events_since(&self, session_id: SessionId, last_event_id: String) -> Vec<Event> { ... }
}

// SessionManager coordinates
impl SessionManager {
    pub async fn handle_sse_reconnect(&self, session_id: SessionId, last_event_id: Option<String>) {
        if let Some(event_tracker) = &self.event_tracker {
            let events = event_tracker.get_events_since(session_id, last_event_id).await;
            // Replay events
        }
    }
}
```

## Remaining Questions

### 1. Pool and SessionManager Coordination
**Question**: How should connection pool interact with SessionManager?

**Options**:
a) Pool notifies SessionManager on connection events
b) SessionManager queries pool for connection state
c) Keep them independent with external coordination

**Recommendation**: Option C - Keep independent, let Server/Client coordinate

### 2. Feature Flag Granularity
**Question**: How granular should feature flags be?

**Current Plan**:
```toml
[features]
default = ["sse", "metrics"]
sse = []
websocket = ["tokio-tungstenite"]
metrics = []
sqlite = ["sqlx"]
redis = ["redis-rs"]
```

**Question**: Should we have meta-features like `full` or `production`?

### 3. Config Validation Timing
**Question**: When should config validation happen?

**Options**:
a) On creation (fail fast)
b) On first use (lazy)
c) Explicit `validate()` call

**Recommendation**: Option A - Validate on creation

### 4. Interceptor Priority/Ordering ✅
**Decision**: Registration order with optional version filtering

**Implementation**:
- Registration order determines execution order
- Interceptors self-select based on context
- Optional `supported_versions()` for performance optimization
- Version checking happens in interceptor, not chain

### 5. Session Cleanup Triggers
**Question**: What triggers session cleanup?

**Confirmed**:
- Idle timeout ✓
- Max age ✓
- Explicit close ✓

**Question**: Should we also cleanup on:
- Memory pressure?
- Max session count?
- Connection pool exhaustion?

## Final Implementation Plan

### Week 1: Foundation
1. Create config module structure
2. Port SessionStore and InMemorySessionStore
3. Port InterceptorChain with async model
4. Create SessionManager with persistence worker
5. Port EventTracker as separate component

### Week 2: Server Refactoring
1. Add components to Server struct
2. Implement hyper serve_connection pattern
3. Add Semaphore for connection limits
4. Replace Arc<RwLock<bool>> with CancellationToken
5. Integrate interceptor pipeline

### Week 3: Client Enhancement
1. Add SessionManager to Client
2. Implement session tracking
3. Add interceptor support
4. Update request/response flow

### Week 4: SSE and Testing
1. Implement SSE with core support
2. Add WebSocket behind feature flag
3. Create comprehensive test suite
4. Add performance benchmarks

## Success Validation

### Functional Checkpoints
- [ ] Single spawn per connection
- [ ] Sessions persist across requests
- [ ] Interceptors process all messages
- [ ] SSE reconnection works
- [ ] Graceful shutdown completes
- [ ] Metrics tracked accurately

### Performance Targets
- [ ] < 1 task per connection (down from 5)
- [ ] < 100KB memory per session
- [ ] < 5% latency overhead from interceptors
- [ ] > 10,000 concurrent sessions supported

### Quality Gates
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Documentation complete
- [ ] Integration examples working

## Next Steps

1. Review remaining questions
2. Create config module structure
3. Begin porting SessionStore trait
4. Set up test infrastructure