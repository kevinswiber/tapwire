# MCP Architecture Design Decisions

## Confirmed Decisions ✅

### 1. Session Storage
- **Decision**: Start with in-memory implementation using trait-based design
- **Implementation**: Port `SessionStore` trait and `InMemorySessionStore` from shadowcat
- **Configuration**: Configurable through library API, not hardcoded
- **Persistence**: Include persistence worker for batched operations
```rust
// Library API
let store = InMemorySessionStore::new(); // or SqliteStore, RedisStore later
let session_manager = SessionManager::with_store(store);
```

### 2. Interceptor Registration
- **Decision**: Dynamic registration at runtime
- **Implementation**: Can add/remove interceptors after creation
- **Configuration**: Through library API
```rust
let chain = InterceptorChain::new();
chain.register_interceptor(auth_interceptor).await;
chain.register_interceptor(rate_limiter).await;
// Can add more later dynamically
```

### 3. Backward Compatibility
- **Decision**: NO backward compatibility constraints
- **Rationale**: Not released yet, get the design right
- **Impact**: Can make breaking changes freely

### 4. Configuration Management
- **Decision**: Centralized config module like shadowcat
- **Location**: `crates/mcp/src/config/`
- **Pattern**: Structured configs with validation
```rust
pub mod config {
    pub struct ServerConfig { ... }
    pub struct ClientConfig { ... }
    pub struct SessionConfig { ... }
    pub struct InterceptorConfig { ... }
    
    impl ServerConfig {
        pub fn validate(&self) -> Result<()> { ... }
        pub fn with_env_overrides(self) -> Self { ... }
    }
}
```

## Remaining Design Questions ❓

### 1. Connection Trait vs Transport Trait
**Context**: MCP crate has `Connection` trait, shadowcat has `Transport` traits

**Options**:
a) Keep MCP's Connection trait, adapt shadowcat components
b) Replace with Transport pattern from shadowcat
c) Have both with adapters between them

**Considerations**:
- Connection trait is simpler and already integrated with pool
- Transport pattern is more flexible for different protocols
- Need to support stdio, HTTP, SSE, WebSocket

**Question**: Should we keep Connection or migrate to Transport?

### 2. SessionManager and Connection Pool Integration
**Context**: Pool is already optimized, SessionManager needs connection tracking

**Options**:
a) SessionManager uses the existing pool for connections
b) Keep pool and sessions separate (current shadowcat approach)
c) Hybrid - pool for connections, sessions reference pool entries

**Question**: How tightly should sessions and connections be coupled?

### 3. SSE/WebSocket Support Location
**Context**: Need server push capabilities for MCP notifications

**Options**:
a) Core feature in MCP crate
b) Optional feature flag (`features = ["sse", "websocket"]`)
c) Separate crate (`mcp-streaming`)

**Question**: Core or optional feature?

### 4. Error Handling Strategy
**Context**: Shadowcat has comprehensive errors, MCP has simpler approach

**Options**:
a) Port shadowcat's error hierarchy completely
b) Keep MCP's simple errors, extend as needed
c) Hybrid - simple public API, detailed internal errors

**Question**: How comprehensive should error types be?

### 5. Persistence Worker Lifecycle
**Context**: Persistence worker handles batched storage operations

**Options**:
a) Start automatically when SessionManager created
b) Require explicit `start_persistence()` call
c) Start on first persistence operation (lazy)

**Question**: Automatic or explicit initialization?

### 6. Dual Session ID Tracking
**Context**: Reverse proxy needs both client and upstream session IDs

**Options**:
a) Add to core Session struct (upstream_session_id: Option<SessionId>)
b) Use metadata map for flexibility
c) Separate ProxySession extends Session

**Question**: How to handle proxy-specific session needs?

### 7. Interceptor Async Model
**Context**: Interceptors may need async operations

**Options**:
a) All interceptors must be async
b) Support both sync and async with traits
c) Sync by default, async wrapper available

**Question**: Async-only or mixed model?

### 8. Testing Strategy
**Context**: Need comprehensive testing of all components

**Options**:
a) Mock implementations (`MockSessionStore`, `MockInterceptor`)
b) Real implementations with test-only features
c) Separate test utilities module

**Question**: Mocks or real implementations for testing?

### 9. EventTracker Integration
**Context**: SSE needs event tracking for reconnection

**Options**:
a) Part of SessionManager
b) Separate component that SessionManager uses
c) Optional feature of Session struct

**Question**: Where should EventTracker live?

### 10. Metrics and Telemetry
**Context**: Production systems need observability

**Options**:
a) Include from start with metrics traits
b) Add later when needed
c) Optional feature flag

**Question**: Build in metrics support now or later?

## Proposed Answers (for Discussion)

Based on shadowcat patterns and MCP requirements:

1. **Connection Trait**: Keep it, but make it async-friendly for SSE/WebSocket
2. **Pool Integration**: Keep separate - pool for connections, sessions reference them
3. **SSE/WebSocket**: Core feature (required for MCP notifications)
4. **Errors**: Hybrid approach - simple public API, detailed internals
5. **Persistence Worker**: Lazy initialization on first use
6. **Dual Sessions**: Use metadata map (flexible, non-breaking)
7. **Interceptors**: Async-only (simpler, futures are cheap)
8. **Testing**: Mock implementations in `#[cfg(test)]`
9. **EventTracker**: Separate component that SessionManager coordinates
10. **Metrics**: Optional feature flag from start

## Implementation Priority

Given these decisions, the implementation order should be:

### Phase 1: Core Infrastructure (Week 1)
1. Create config module with all config types
2. Port SessionStore trait and InMemorySessionStore
3. Port InterceptorChain with async interceptors
4. Create basic SessionManager without persistence

### Phase 2: Integration (Week 2)
1. Update Server to use new components
2. Add hyper serve_connection pattern
3. Update Client with session tracking
4. Wire interceptors into message flow

### Phase 3: Advanced Features (Week 3)
1. Add persistence worker
2. Implement SSE support with EventTracker
3. Add WebSocket upgrade handling
4. Implement pause controller

### Phase 4: Production Features (Week 4)
1. Add comprehensive error handling
2. Implement metrics (if feature enabled)
3. Add integration tests
4. Performance benchmarks

## Configuration Schema

```rust
// crates/mcp/src/config/mod.rs
pub struct Config {
    pub server: ServerConfig,
    pub client: ClientConfig,
    pub session: SessionConfig,
    pub interceptor: InterceptorConfig,
    pub pool: PoolConfig,  // Already exists
}

pub struct ServerConfig {
    pub max_clients: usize,
    pub shutdown_timeout: Duration,
    pub enable_sse: bool,
    pub enable_websocket: bool,
}

pub struct ClientConfig {
    pub request_timeout: Duration,
    pub enable_reconnection: bool,
    pub max_retries: u32,
}

pub struct SessionConfig {
    pub cleanup_interval: Duration,
    pub max_idle_time: Option<Duration>,
    pub max_session_age: Option<Duration>,
    pub persistence_batch_size: usize,
}

pub struct InterceptorConfig {
    pub enable_pause_controller: bool,
    pub default_timeout: Duration,
    pub max_chain_length: usize,
}
```

## Next Steps

1. Review and finalize remaining design questions
2. Update plan with final decisions
3. Begin Phase 1 implementation
4. Create integration test scenarios