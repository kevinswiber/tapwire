# Design Decisions

**Created**: 2025-08-16  
**Purpose**: Document key architectural decisions and their rationale

## Decision Record

### Decision 1: ResponseMode as Separate Enum

**Decision**: Create a new `ResponseMode` enum instead of extending `TransportType`

**Alternatives Considered**:
1. Add response format variants to TransportType (e.g., `TransportType::SseJson`, `TransportType::SseStream`)
2. Use a boolean flag per format (e.g., `is_json`, `is_sse`, `is_binary`)
3. Use string-based format tracking

**Rationale**:
- **Separation of Concerns**: TransportType describes the connection method (stdio, HTTP), while ResponseMode describes the data format (JSON, SSE stream)
- **Orthogonality**: A single transport can handle multiple response modes
- **Runtime vs Configuration**: TransportType is configuration-time, ResponseMode is runtime-detected
- **Extensibility**: Easy to add new formats without affecting transport logic

**Consequences**:
- ✅ Clear semantic separation
- ✅ Type-safe format handling
- ✅ Supports mixed-mode sessions
- ⚠️ Requires tracking two separate enums
- ⚠️ Slightly more complex session state

### Decision 2: Adopt DirectionalTransports for Reverse Proxy

**Decision**: Refactor reverse proxy to use IncomingTransport/OutgoingTransport traits

**Alternatives Considered**:
1. Keep reverse proxy's direct implementation approach
2. Create separate ReverseIncoming/ReverseOutgoing traits
3. Use a hybrid approach with adapters

**Rationale**:
- **Code Reuse**: Eliminate ~500 lines of duplicate transport logic
- **Consistency**: Same behavior and error handling across both proxies
- **Proven Design**: Forward proxy demonstrates the pattern works well
- **Maintainability**: Single implementation to maintain and test

**Consequences**:
- ✅ Significant code reduction
- ✅ Unified testing strategy
- ✅ Consistent behavior
- ⚠️ Reverse proxy refactoring effort
- ⚠️ Potential edge case differences

### Decision 3: Generic Connection Pooling

**Decision**: Create `GenericPool<T: OutgoingTransport>` for all transport types

**Alternatives Considered**:
1. Separate pools per transport type
2. No pooling for HTTP/SSE (current approach)
3. External pooling library

**Rationale**:
- **Efficiency**: Reuse connections for all transport types
- **SSE Support**: Critical for SSE reconnection with Last-Event-ID
- **Uniformity**: Same pooling behavior for all transports
- **Health Checks**: Centralized connection health management

**Consequences**:
- ✅ Better resource utilization
- ✅ SSE reconnection support
- ✅ Unified connection management
- ⚠️ More complex pool implementation
- ⚠️ Need health check strategies per transport

### Decision 4: Shared Transport Implementations

**Decision**: Extract common logic into `transport/implementations/` module

**Alternatives Considered**:
1. Keep implementations in directional modules
2. Use inheritance/composition patterns
3. Macro-based code generation

**Rationale**:
- **DRY Principle**: Single source of truth for transport logic
- **Modularity**: Clear separation between trait and implementation
- **Testability**: Test shared logic once
- **Performance**: Shared buffer pools and optimizations

**Consequences**:
- ✅ Reduced code duplication
- ✅ Easier to optimize
- ✅ Centralized bug fixes
- ⚠️ Additional abstraction layer
- ⚠️ Potential over-generalization

### Decision 5: ProxyCore vs UnifiedProxy Naming

**Decision**: Use ProxyCore as the shared proxy pipeline abstraction, not UnifiedProxy

**Context**: During analysis, "UnifiedProxy" was used to describe the long-term vision of unified proxy architecture. During design, "ProxyCore" emerged as the actual implementation name. These represent the same concept.

**Alternatives Considered**:
1. **UnifiedProxy**: Single proxy class that operates in both modes
2. **ProxyCore**: Shared core logic used by distinct ForwardProxy and ReverseProxy classes
3. **ProxyPipeline**: Emphasizes the pipeline nature of the shared logic
4. **CommonProxy**: Generic name for shared functionality

**Rationale**:
- **ProxyCore** better represents the actual architecture: shared core logic, not a unified proxy
- Forward and Reverse proxies remain distinct classes that use ProxyCore
- "Core" implies foundational shared logic, not a complete proxy implementation
- Allows specialization: each proxy type can add its specific features

**Architecture Clarification**:
```rust
// What we're building:
pub struct ProxyCore {
    // Shared pipeline logic
}

pub struct ForwardProxy {
    core: ProxyCore,  // Uses shared logic
    // Forward-specific fields
}

pub struct ReverseProxy {
    core: ProxyCore,  // Uses shared logic
    auth_gateway: Option<AuthGateway>,  // Reverse-specific
}

// NOT building:
pub struct UnifiedProxy {
    mode: ProxyMode,  // Single class for both modes
}
```

**Consequences**:
- ✅ Clear separation of shared vs specific logic
- ✅ Each proxy type can evolve independently
- ✅ Shared behavior without forcing identical structure
- ⚠️ Must maintain consistency in ProxyCore usage
- ⚠️ Need to document which logic belongs in core vs specific

### Decision 6: Shared ProxyCore Implementation

**Decision**: Implement ProxyCore with shared message pipeline logic

**Alternatives Considered**:
1. Keep proxies completely separate (current state)
2. Use traits for shared behavior
3. Template/generic proxy implementation

**Rationale**:
- **Consistency**: Same message flow for both proxy types
- **Interceptors**: Shared interceptor chain logic
- **Sessions**: Unified session management
- **Maintenance**: Single pipeline to maintain and test

**Consequences**:
- ✅ Unified proxy behavior
- ✅ Shared interceptor chain
- ✅ Consistent error handling
- ✅ ~500 lines of duplicate code eliminated
- ⚠️ Less flexibility per proxy type
- ⚠️ Must carefully decide core vs specific logic

### Decision 8: Keep TransportType Name

**Decision**: Keep `TransportType` name but rename `Sse` variant to `StreamableHttp`

**Alternatives Considered**:
1. Rename to `SessionOrigin` or `ClientTransport`
2. Create separate enums for client and upstream
3. Remove enum entirely in favor of traits

**Rationale**:
- **Minimal Disruption**: Less code churn
- **Accurate Naming**: StreamableHttp better matches MCP spec
- **Clear Purpose**: Type describes transport configuration
- **Backward Compatible**: Easy migration path

**Consequences**:
- ✅ Minimal breaking changes
- ✅ Aligns with MCP spec
- ✅ Clear semantics
- ⚠️ Slight naming inconsistency
- ⚠️ May still conflate concepts slightly

### Decision 9: Session Store Compatibility

**Decision**: Maintain compatibility with existing SessionStore trait

**Alternatives Considered**:
1. Create new session types that bypass SessionStore
2. Add response mode as separate storage outside sessions
3. Modify SessionStore trait to add specific methods

**Rationale**:
- Existing abstraction supports distributed storage
- All methods already async for network operations
- Maintains path to Redis backend implementation
- Consistent session management across storage backends

**Consequences**:
- ✅ Works with future Redis implementation
- ✅ Atomic session updates
- ✅ Consistent API across backends
- ⚠️ Must keep Session serializable
- ⚠️ All updates must be async

### Decision 10: Phased Migration Approach

**Decision**: Implement in three phases rather than single big-bang refactor

**Alternatives Considered**:
1. Complete rewrite in new module
2. Single comprehensive refactor
3. Feature-flag based parallel implementation

**Rationale**:
- **Risk Mitigation**: Smaller, testable changes
- **Continuous Delivery**: Value delivered incrementally
- **Team Velocity**: Work fits in sprints
- **Rollback Safety**: Can revert individual phases

**Consequences**:
- ✅ Lower risk
- ✅ Incremental value
- ✅ Easier review and testing
- ⚠️ Temporary compatibility code
- ⚠️ Longer total duration

### Decision 11: Module Organization Strategy

**Decision**: Organize by technical layer (transport, protocol, proxy, session)

**Alternatives Considered**:
1. Organize by feature (forward-proxy/, reverse-proxy/)
2. Organize by domain (client/, upstream/, core/)
3. Flat structure with prefixed names

**Rationale**:
- **Clear Layers**: Obvious architectural boundaries
- **Dependency Management**: Enforces proper layering
- **Discoverability**: Easy to find functionality
- **Rust Conventions**: Follows community patterns

**Consequences**:
- ✅ Clear architecture
- ✅ Enforced layering
- ✅ Easy navigation
- ⚠️ Some cross-cutting concerns
- ⚠️ Potential circular dependency risks

## Distributed Session Management Considerations

### Current SessionStore Abstraction

Shadowcat already has a well-designed `SessionStore` trait in `src/session/store.rs` that supports distributed storage:

```rust
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create_session(&self, session: Session) -> SessionResult<()>;
    async fn get_session(&self, id: &SessionId) -> SessionResult<Session>;
    async fn update_session(&self, session: Session) -> SessionResult<()>;
    async fn delete_session(&self, id: &SessionId) -> SessionResult<()>;
    // ... other async methods
}
```

**Key Design Points**:
- All methods are already async to support network-based stores
- Trait is Send + Sync for concurrent access
- Batch operations included for efficiency
- SSE-specific operations for event ID tracking

### Impact on Transport Architecture

When updating the Session struct for ResponseMode:

1. **Keep Serializable**: Session must remain `Serialize`/`Deserialize` for Redis
2. **Async Operations**: All session updates must use async store methods
3. **Response Mode Storage**: ResponseMode enum must be serializable
4. **Session ID Mapping**: Consider how dual session IDs affect distributed storage

### Design Decisions for Distributed Support

**Decision**: Keep Session struct serializable and lightweight
- ResponseMode enum derives Serialize/Deserialize
- upstream_session_id field is optional and serializable
- Avoid storing large transient data in Session

**Decision**: Use existing SessionStore abstraction
- Don't bypass SessionStore trait for direct access
- All session updates go through async store methods
- Maintains compatibility with future Redis backend

**Decision**: Session updates should be atomic
- Update entire Session object, not individual fields
- Prevents inconsistency in distributed scenarios
- Example:
  ```rust
  // Good - atomic update
  let mut session = store.get_session(&id).await?;
  session.response_mode = Some(ResponseMode::SseStream);
  store.update_session(session).await?;
  
  // Bad - non-atomic field update
  store.set_response_mode(&id, ResponseMode::SseStream).await?;
  ```

### Future Redis Implementation

The redis-session-storage plan (in `plans/redis-session-storage/`) will:
1. Implement RedisStore conforming to SessionStore trait
2. Use Redis data structures optimized for session data
3. Support session expiry and TTL
4. Enable horizontal scaling across multiple proxies

Our transport architecture changes maintain compatibility by:
- Keeping Session serializable
- Using async store operations
- Not adding non-serializable fields

## Deferred Decisions

### WebSocket Support
**Why Deferred**: Need to complete current refactor first
**When to Revisit**: After Phase 3 completion
**Considerations**: Bidirectional streaming, protocol upgrades

### Binary Protocol Support
**Why Deferred**: Not currently required
**When to Revisit**: When binary MCP extensions are needed
**Considerations**: Efficient serialization, streaming large files

## Anti-Patterns to Avoid

### 1. Transport Logic in Proxy Layer
**Why It's Bad**: Violates separation of concerns
**Instead**: Use transport traits and delegate

### 2. Response Format Assumptions
**Why It's Bad**: Prevents extensibility
**Instead**: Use ResponseMode enum exhaustively

### 3. Direct Transport Creation
**Why It's Bad**: Bypasses configuration and pooling
**Instead**: Always use TransportFactory

### 4. Synchronous Blocking
**Why It's Bad**: Defeats async runtime benefits
**Instead**: Use async throughout, even for small operations

### 5. Tight Coupling to Hyper
**Why It's Bad**: Makes it hard to switch HTTP libraries
**Instead**: Abstract behind transport traits

## Performance Considerations

### Buffer Pooling Strategy
- **Decision**: Thread-local pools for hot paths
- **Rationale**: Reduces allocation overhead
- **Trade-off**: Memory vs CPU

### Async Boundaries
- **Decision**: Keep async boundaries at transport layer
- **Rationale**: Maximize concurrent request handling
- **Trade-off**: Complexity vs throughput

### Connection Keep-Alive
- **Decision**: Aggressive keep-alive with health checks
- **Rationale**: Reduce connection overhead
- **Trade-off**: Resource usage vs latency

## Security Considerations

### Transport Isolation
- **Decision**: Separate transports per session
- **Rationale**: Prevent session hijacking
- **Implementation**: No transport sharing between sessions

### Response Mode Validation
- **Decision**: Validate Content-Type strictly
- **Rationale**: Prevent response smuggling
- **Implementation**: Whitelist allowed content types

### Connection Pool Security
- **Decision**: Pool connections per target
- **Rationale**: Prevent connection confusion
- **Implementation**: Strict target validation

## Testing Strategy Decisions

### Unit Test Granularity
- **Decision**: Test at trait implementation level
- **Rationale**: Balance coverage and maintenance
- **Approach**: Mock dependencies, test contracts

### Integration Test Scope
- **Decision**: Test complete request/response flows
- **Rationale**: Catch interaction issues
- **Approach**: Use real transports, mock servers

### Performance Benchmarks
- **Decision**: Benchmark critical paths only
- **Rationale**: Focus on hot paths
- **Metrics**: Latency, throughput, memory

## Documentation Decisions

### API Documentation
- **Decision**: Document at trait level
- **Rationale**: Users interact with traits
- **Tool**: Rust doc comments

### Architecture Documentation
- **Decision**: Maintain in markdown
- **Rationale**: Version controlled, reviewable
- **Location**: docs/architecture/

### Examples
- **Decision**: Provide for each major use case
- **Rationale**: Show best practices
- **Location**: examples/ directory

---

**Status**: Living document - update as new decisions are made
**Review**: Architecture team should review quarterly