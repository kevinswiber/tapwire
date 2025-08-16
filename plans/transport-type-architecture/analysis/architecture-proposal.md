# Transport Type Architecture Proposal

**Created**: 2025-08-16  
**Purpose**: Comprehensive architecture proposal for eliminating code smells and unifying transport handling in shadowcat

## Executive Summary

After analyzing 174 TransportType usages across 32 files and discovering that `is_sse_session` is completely dead code, this proposal presents a clean architecture that eliminates duplication, establishes clear domain boundaries, and provides a solid foundation for shadowcat's transport layer.

**Key Changes**:
1. Replace `is_sse_session` boolean with `ResponseMode` enum
2. Unify transport handling using existing DirectionalTransport traits
3. Create shared transport abstractions for both proxy types
4. Establish clear separation between transport, protocol, session, and proxy layers

**Impact**: ~500 lines of duplicate code eliminated, type-safe response handling, extensible architecture for future protocols.

## Problem Statement

### Current Issues

1. **Dead Code**: `is_sse_session` field exists but is never set (mark_as_sse_session() is unused)
2. **Code Duplication**: Forward and reverse proxies implement transport logic separately
3. **Conflated Concepts**: TransportType mixes session origin with response format
4. **Monolithic Design**: Reverse proxy (1000+ lines) mixes transport, protocol, and business logic
5. **Inconsistent Abstractions**: Forward proxy uses traits, reverse proxy uses direct implementations

### Root Cause Analysis

The core issue is that shadowcat evolved two separate architectures:
- **Forward Proxy**: Clean directional transport traits with proper separation
- **Reverse Proxy**: Direct HTTP client usage with manual SSE detection

This divergence created maintenance burden and architectural debt.

## Target Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         Application Layer                        │
├───────────────────────────────┬─────────────────────────────────┤
│         Forward Proxy         │         Reverse Proxy           │
│    (uses shared abstractions) │    (uses shared abstractions)   │
└───────────────────────────────┴─────────────────────────────────┘
                                │
┌───────────────────────────────┴─────────────────────────────────┐
│                          Proxy Core                              │
│        (Shared routing, session management, interceptors)        │
└──────────────────────────────────────────────────────────────────┘
                                │
┌───────────────────────────────┴─────────────────────────────────┐
│                        Protocol Layer                            │
│         (MCP message handling, serialization, validation)        │
└──────────────────────────────────────────────────────────────────┘
                                │
┌───────────────────────────────┴─────────────────────────────────┐
│                        Transport Layer                           │
├─────────────────┬──────────────────────┬────────────────────────┤
│  Core Types     │  Directional Traits   │   Implementations     │
├─────────────────┼──────────────────────┼────────────────────────┤
│ • TransportType │ • IncomingTransport   │ • StdioIncoming       │
│ • ResponseMode  │ • OutgoingTransport   │ • HttpIncoming        │
│ • PoolConfig    │ • BidirectionalTrans. │ • SseIncoming         │
│                 │                        │ • StdioOutgoing       │
│                 │                        │ • HttpOutgoing        │
│                 │                        │ • SseOutgoing         │
└─────────────────┴──────────────────────┴────────────────────────┘
                                │
┌───────────────────────────────┴─────────────────────────────────┐
│                     Infrastructure Layer                         │
│            (Connection pooling, buffering, metrics)              │
└──────────────────────────────────────────────────────────────────┘
```

### Module Structure

```
src/
├── transport/
│   ├── core/
│   │   ├── mod.rs              # Core types and traits
│   │   ├── transport_type.rs   # TransportType enum (renamed from Sse to StreamableHttp)
│   │   └── response_mode.rs    # NEW: ResponseMode enum
│   │
│   ├── directional/
│   │   ├── mod.rs              # Trait definitions (existing)
│   │   ├── incoming.rs         # IncomingTransport implementations
│   │   ├── outgoing.rs         # OutgoingTransport implementations
│   │   └── bidirectional.rs    # Future: WebSocket support
│   │
│   ├── implementations/
│   │   ├── stdio.rs            # Shared stdio logic
│   │   ├── http.rs             # Shared HTTP logic  
│   │   ├── sse.rs              # Shared SSE logic
│   │   └── pool.rs             # Generic connection pooling
│   │
│   └── factory/
│       ├── mod.rs              # Unified transport factory
│       ├── builder.rs          # Transport builders
│       └── config.rs           # Transport configuration
│
├── protocol/
│   ├── mcp.rs                  # MCP protocol handling
│   ├── serialization.rs        # Message serialization
│   └── validation.rs           # Protocol validation
│
├── proxy/
│   ├── core/
│   │   ├── mod.rs              # Shared proxy logic
│   │   ├── router.rs           # Message routing
│   │   ├── session_mapper.rs   # Session management
│   │   └── interceptor.rs      # Interceptor chain
│   │
│   ├── forward/
│   │   └── mod.rs              # Forward proxy specialization
│   │
│   └── reverse/
│       ├── mod.rs              # Reverse proxy specialization
│       ├── auth_gateway.rs     # OAuth handling
│       └── sse_resilience.rs   # SSE reconnection
│
└── session/
    ├── store.rs                # Updated Session struct
    ├── manager.rs              # Session lifecycle
    └── tracking.rs             # Response mode tracking
```

## Core Type Definitions

### ResponseMode Enum

```rust
/// Represents the format of a response from the upstream server
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseMode {
    /// Response format not yet determined
    Unknown,
    
    /// Standard JSON-RPC response (application/json)
    Json,
    
    /// Server-Sent Events stream (text/event-stream)
    SseStream,
    
    /// Binary data (application/octet-stream)
    Binary,
    
    /// WebSocket upgrade (future)
    WebSocket,
}

impl ResponseMode {
    /// Detect response mode from Content-Type header
    pub fn from_content_type(content_type: &str) -> Self {
        if content_type.contains("application/json") {
            Self::Json
        } else if content_type.contains("text/event-stream") {
            Self::SseStream
        } else if content_type.contains("application/octet-stream") {
            Self::Binary
        } else {
            Self::Unknown
        }
    }
    
    /// Check if this mode requires streaming
    pub fn is_streaming(&self) -> bool {
        matches!(self, Self::SseStream | Self::WebSocket)
    }
}
```

### Updated Session Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)] // Must stay serializable for distributed storage
pub struct Session {
    pub id: SessionId,
    
    // Transport configuration
    pub client_transport: TransportType,    // How client connects to proxy
    pub upstream_transport: TransportType,   // How proxy connects to upstream
    
    // Session ID mapping (reverse proxy only - see reverse-proxy-session-mapping plan)
    pub upstream_session_id: Option<SessionId>, // Maps to upstream server's session ID
    
    // Response tracking (serializable for distributed storage)
    pub response_mode: Option<ResponseMode>, // Current response format
    pub supports_streaming: bool,            // Client capability
    
    // Remove: is_sse_session field
    
    // ... other existing fields ...
}

impl Session {
    /// Update response mode based on upstream response
    /// Note: Caller must persist via SessionStore for distributed scenarios
    pub fn set_response_mode(&mut self, mode: ResponseMode) {
        self.response_mode = Some(mode);
        self.update_activity(); // Use existing method from store.rs
    }
    
    /// Check if session is handling streaming response
    pub fn is_streaming(&self) -> bool {
        self.response_mode
            .map(|m| m.is_streaming())
            .unwrap_or(false)
    }
}
```

### Unified Transport Factory

```rust
pub struct TransportFactory {
    buffer_pools: Arc<BufferPools>,
    connection_pool: Arc<ConnectionPool>,
    config: TransportConfig,
}

impl TransportFactory {
    /// Create incoming transport based on configuration
    pub async fn create_incoming(
        &self,
        transport_type: TransportType,
    ) -> Result<Box<dyn IncomingTransport>> {
        match transport_type {
            TransportType::Stdio => {
                Ok(Box::new(StdioIncoming::new(self.buffer_pools.clone())))
            }
            TransportType::StreamableHttp => {
                Ok(Box::new(HttpIncoming::new(
                    self.config.bind_addr,
                    self.buffer_pools.clone(),
                )))
            }
        }
    }
    
    /// Create outgoing transport with connection pooling
    pub async fn create_outgoing(
        &self,
        transport_type: TransportType,
        target: &str,
    ) -> Result<Box<dyn OutgoingTransport>> {
        // Check pool first
        if let Some(pooled) = self.connection_pool.acquire(target).await? {
            return Ok(pooled);
        }
        
        // Create new connection
        match transport_type {
            TransportType::Stdio => {
                Ok(Box::new(StdioOutgoing::new(
                    target,
                    self.buffer_pools.clone(),
                )))
            }
            TransportType::StreamableHttp => {
                Ok(Box::new(HttpOutgoing::new(
                    target,
                    self.buffer_pools.clone(),
                )))
            }
        }
    }
}
```

## Module Organization Clarification

### transport/raw vs transport/directional

The distinction between these modules is important:

**transport/raw/** - Low-level transport primitives
- Contains raw I/O operations (read/write bytes)
- No knowledge of MCP protocol or message framing
- Shared by multiple directional implementations
- Examples: Raw stdio operations, HTTP client/server setup, SSE event parsing

**transport/directional/** - High-level transport implementations
- Implements IncomingTransport/OutgoingTransport traits
- Handles MCP message serialization/deserialization
- Uses raw primitives for actual I/O
- Examples: StdioIncoming (uses raw/stdio.rs), HttpOutgoing (uses raw/http.rs)

This separation allows code reuse while maintaining clean abstractions.

## Shared Abstractions

### Generic Transport Wrapper

```rust
/// Wraps any transport with common functionality
pub struct TransportWrapper<T> {
    inner: T,
    session_id: SessionId,
    response_mode: ResponseMode,
    metrics: Arc<Metrics>,
}

impl<T: IncomingTransport> IncomingTransport for TransportWrapper<T> {
    async fn receive_request(&mut self) -> TransportResult<MessageEnvelope> {
        let start = Instant::now();
        let result = self.inner.receive_request().await;
        self.metrics.record_request_duration(start.elapsed());
        result
    }
    
    async fn send_response(&mut self, response: MessageEnvelope) -> TransportResult<()> {
        // Detect response mode from envelope metadata
        if let Some(content_type) = response.metadata.get("content-type") {
            self.response_mode = ResponseMode::from_content_type(content_type);
        }
        
        self.inner.send_response(response).await
    }
    
    // ... delegate other methods ...
}
```

### ProxyCore: Shared Pipeline Logic

```rust
/// Shared proxy pipeline logic used by both ForwardProxy and ReverseProxy
/// Note: This is NOT a unified proxy class, but shared core logic that
/// each proxy type uses while maintaining their distinct identities
pub struct ProxyCore {
    session_manager: Arc<SessionManager>,
    interceptor_chain: Arc<InterceptorChain>,
    transport_factory: Arc<TransportFactory>,
}

impl ProxyCore {
    /// Process a request through the proxy pipeline
    pub async fn process_request(
        &self,
        incoming: &mut dyn IncomingTransport,
        outgoing: &mut dyn OutgoingTransport,
    ) -> Result<()> {
        // Receive request from client
        let request = incoming.receive_request().await?;
        
        // Process through interceptors
        let request = self.interceptor_chain.process_request(request).await?;
        
        // Forward to upstream
        outgoing.send_request(request).await?;
        
        // Handle response based on mode
        let response = outgoing.receive_response().await?;
        let response_mode = self.detect_response_mode(&response);
        
        match response_mode {
            ResponseMode::Json => {
                self.handle_json_response(incoming, response).await
            }
            ResponseMode::SseStream => {
                self.handle_sse_stream(incoming, outgoing).await
            }
            _ => self.handle_raw_response(incoming, response).await
        }
    }
    
    fn detect_response_mode(&self, response: &MessageEnvelope) -> ResponseMode {
        response.metadata
            .get("content-type")
            .map(|ct| ResponseMode::from_content_type(ct))
            .unwrap_or(ResponseMode::Unknown)
    }
}
```

## Migration Plan

### Phase 1: Add ResponseMode (4 hours)

**Objective**: Introduce ResponseMode enum without breaking existing code

1. **Add ResponseMode enum** (30 min)
   - Create `src/transport/core/response_mode.rs`
   - Add detection methods
   - Add tests

2. **Update Session struct** (1 hour)
   - Add `response_mode: Option<ResponseMode>` field
   - Keep `is_sse_session` temporarily for compatibility
   - Add setter/getter methods

3. **Parallel tracking** (1.5 hours)
   - Update hyper_client to set ResponseMode
   - Log both old and new values for verification
   - Ensure consistency

4. **Remove dead code** (1 hour)
   - Delete `is_sse_session` field
   - Remove `mark_as_sse_session()` method
   - Update all references

### Phase 2: Extract Shared Transport Logic (6 hours)

**Objective**: Create reusable transport implementations

1. **Create transport/implementations module** (2 hours)
   - Extract common stdio logic
   - Extract common HTTP logic
   - Extract common SSE logic

2. **Update directional transports** (2 hours)
   - Refactor to use shared implementations
   - Maintain existing interfaces
   - Add comprehensive tests

3. **Create unified factory** (2 hours)
   - Implement TransportFactory
   - Add configuration support
   - Wire into existing code

### Phase 3: Unify Proxy Architecture (8 hours)

**Objective**: Adopt directional transports in reverse proxy

1. **Create ProxyCore abstraction** (2 hours)
   - Extract common proxy logic
   - Define shared interfaces
   - Implement core pipeline

2. **Refactor reverse proxy** (4 hours)
   - Adopt IncomingTransport for client connections
   - Adopt OutgoingTransport for upstream connections
   - Remove duplicate transport code

3. **Update connection pooling** (2 hours)
   - Create generic pool implementation
   - Support all transport types
   - Add health checks and reconnection

## Implementation Order

### Immediate (Phase 1 - Quick Fix)
1. Add ResponseMode enum ✓
2. Update Session struct ✓
3. Remove is_sse_session ✓
4. Update tests ✓

### Short-term (Phase 2 - Consolidation)
1. Extract shared transport logic
2. Create unified factory
3. Standardize error handling
4. Add metrics collection

### Medium-term (Phase 3 - Unification)
1. Refactor reverse proxy
2. Unify connection pooling
3. Consolidate configuration
4. Complete test coverage

## API Design

### Transport Traits (Enhanced)

```rust
#[async_trait]
pub trait IncomingTransport: Send + Sync {
    // Core methods (existing)
    async fn accept(&mut self) -> TransportResult<()>;
    async fn receive_request(&mut self) -> TransportResult<MessageEnvelope>;
    async fn send_response(&mut self, response: MessageEnvelope) -> TransportResult<()>;
    async fn close(&mut self) -> TransportResult<()>;
    
    // Enhanced for streaming
    async fn send_stream_start(&mut self) -> TransportResult<()> {
        Ok(()) // Default no-op
    }
    
    async fn send_stream_chunk(&mut self, chunk: &[u8]) -> TransportResult<()> {
        Err(TransportError::NotSupported("Streaming not supported"))
    }
    
    async fn send_stream_end(&mut self) -> TransportResult<()> {
        Ok(()) // Default no-op
    }
    
    // Metadata
    fn transport_type(&self) -> TransportType;
    fn supports_streaming(&self) -> bool {
        false // Default
    }
}
```

### Session API (Updated)

```rust
impl SessionManager {
    /// Create session with bidirectional transport tracking
    pub async fn create_session(
        &self,
        client_transport: TransportType,
        upstream_transport: TransportType,
    ) -> Result<Session> {
        let session = Session {
            id: SessionId::new(),
            client_transport,
            upstream_transport,
            response_mode: None,
            supports_streaming: client_transport.supports_streaming(),
            // ... other fields ...
        };
        
        self.store.create(session).await
    }
    
    /// Update response mode for active session
    pub async fn set_response_mode(
        &self,
        session_id: &SessionId,
        mode: ResponseMode,
    ) -> Result<()> {
        self.store.update_response_mode(session_id, mode).await
    }
}
```

## Session ID Mapping Consideration

The reverse proxy needs to maintain dual session IDs (as documented in the reverse-proxy-session-mapping plan):

```rust
// For reverse proxy sessions
pub struct ReverseProxySession {
    pub proxy_session_id: SessionId,      // Our ID for client connection
    pub upstream_session_id: Option<SessionId>, // Upstream server's session ID
    // ... other fields ...
}
```

For the forward proxy, only a single session ID is needed. This suggests we might want:
- A base `Session` trait with common fields
- `ForwardSession` and `ReverseSession` implementations
- Or conditional fields using feature flags or Options

This will be addressed in detail in the reverse-proxy-session-mapping implementation, but the architecture should accommodate both models.

## Design Decisions

### Why ResponseMode Instead of is_sse_session?

1. **Type Safety**: Enum provides compile-time checking of all response types
2. **Extensibility**: Easy to add new formats (WebSocket, gRPC, etc.)
3. **Clarity**: Explicitly models what we're tracking (response format)
4. **Correctness**: Aligns with actual runtime behavior (per-response detection)

### Why Unify on DirectionalTransports?

1. **Code Reuse**: Eliminate ~500 lines of duplicate transport logic
2. **Consistency**: Same behavior across both proxy types
3. **Maintainability**: Single source of truth for transport handling
4. **Testability**: Shared test suites for all transports

### Why Keep TransportType Separate from ResponseMode?

1. **Orthogonal Concerns**: Transport is "how", response is "what"
2. **Mixed Sessions**: A session can have different response modes over time
3. **Configuration vs Runtime**: TransportType is config, ResponseMode is runtime
4. **Future Flexibility**: Allows protocol negotiation and upgrades

### Module Organization Rationale

1. **transport/core**: Fundamental types used everywhere
2. **transport/directional**: Trait abstractions for direction
3. **transport/implementations**: Shared concrete logic
4. **proxy/core**: Shared proxy pipeline
5. **proxy/{forward,reverse}**: Specializations only

## Stream Trait Considerations

Using `futures::Stream` for streaming responses provides better async/await ergonomics:

**Performance Impact**: 
- Minimal overhead - Stream is zero-cost abstraction
- Better than manual chunking - automatic backpressure handling
- Integrates with tokio's async runtime efficiently

**Developer Experience**:
- More idiomatic Rust - follows ecosystem patterns
- Works with combinators - `.map()`, `.filter()`, `.take()` etc.
- Compatible with async ecosystem - can use with `tokio_stream` utilities
- Example already in codebase: `transport::sse::buffer::SseStream`

```rust
// Example streaming response handler
if let Some(stream) = transport.response_stream() {
    pin_mut!(stream);
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(bytes) => process_chunk(bytes),
            Err(e) => handle_error(e),
        }
    }
}
```

## Architecture Clarification: ProxyCore vs Unified Proxy

### What We're Building
- **ProxyCore**: Shared message pipeline logic
- **ForwardProxy**: Uses ProxyCore + forward-specific features
- **ReverseProxy**: Uses ProxyCore + reverse-specific features (auth, SSE resilience)

### What We're NOT Building
- Single UnifiedProxy class that handles both modes
- Complete merger of forward and reverse proxies
- Loss of specialization capabilities

This design allows code reuse while maintaining the distinct identity and capabilities of each proxy type.

## Example Usage

### Forward Proxy (Minimal Changes)

```rust
pub async fn handle_forward_proxy(
    client_transport: Box<dyn IncomingTransport>,
    server_transport: Box<dyn OutgoingTransport>,
) -> Result<()> {
    let proxy_core = ProxyCore::new(/* deps */);
    
    // Use shared implementation
    proxy_core.run_proxy(client_transport, server_transport).await
}
```

### Reverse Proxy (After Refactor)

```rust
pub async fn handle_reverse_proxy(
    bind_addr: SocketAddr,
    upstream_config: UpstreamConfig,
) -> Result<()> {
    let factory = TransportFactory::new(/* config */);
    let proxy_core = ProxyCore::new(/* deps */);
    
    // Create transports using factory
    let incoming = factory.create_incoming(TransportType::StreamableHttp).await?;
    let outgoing = factory.create_outgoing(
        upstream_config.transport_type,
        &upstream_config.target,
    ).await?;
    
    // Use same shared implementation
    proxy_core.run_proxy(incoming, outgoing).await
}
```

## Benefits

### Immediate Benefits
- **Eliminate Dead Code**: Remove unused is_sse_session
- **Type Safety**: Explicit ResponseMode tracking
- **Clear Semantics**: Proper separation of concerns

### Long-term Benefits
- **Reduced Maintenance**: ~500 lines less duplicate code
- **Easier Extension**: Add new transports/protocols easily
- **Better Testing**: Shared test infrastructure
- **Performance**: Unified connection pooling and buffering

### Architectural Benefits
- **Clean Boundaries**: Clear layer separation
- **Single Responsibility**: Each module has one job
- **Dependency Inversion**: Depend on abstractions, not concretions
- **Open/Closed**: Open for extension, closed for modification

## Risk Assessment

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking existing behavior | Low | High | Phased migration with compatibility layer |
| Performance regression | Low | Medium | Benchmark critical paths, optimize if needed |
| Complex refactor | Medium | Medium | Incremental changes with tests at each step |
| SSE reconnection issues | Low | High | Careful testing of resilience module |

### Mitigation Strategies

1. **Incremental Migration**: Small, tested changes
2. **Feature Flags**: Toggle new/old implementations
3. **Extensive Testing**: Unit, integration, and E2E tests
4. **Performance Monitoring**: Benchmark before/after
5. **Rollback Plan**: Git tags at each phase

## Success Metrics

### Quantitative Metrics
- ✅ 0 instances of is_sse_session in codebase
- ✅ <5% code duplication between proxies (from ~30%)
- ✅ 100% test coverage for new abstractions
- ✅ <5% performance overhead
- ✅ <100ms startup time maintained

### Qualitative Metrics
- ✅ Clear separation of concerns
- ✅ Consistent behavior across proxies
- ✅ Easy to add new transport types
- ✅ Simplified debugging and maintenance
- ✅ Team satisfaction with architecture

## Recommendations

### Do Immediately (This Week)
1. **Implement Phase 1** - Add ResponseMode, remove dead code
2. **Update documentation** - Explain new architecture
3. **Create migration guide** - Help team understand changes

### Do Soon (Next Sprint)
1. **Extract raw transport primitives** - Phase 2 consolidation
2. **Benchmark performance** - Ensure no regression
3. **Update examples** - Show best practices

### Do Later (Next Month)
1. **Complete unification** - Phase 3 reverse proxy refactor
2. **Add WebSocket support** - Leverage new architecture
3. **Optimize hot paths** - Based on profiling data

## Critical Questions Answered

### 1. ResponseMode Placement
**Answer**: Place in `transport/core/` as it's fundamental to transport behavior but keep it orthogonal to TransportType. It affects how transports handle responses but isn't tied to a specific transport implementation.

### 2. Session-Transport Relationship
**Answer**: Sessions should track transport configuration (TransportType) and current state (ResponseMode) but not own transport instances. Use correlation IDs for linking without tight coupling.

### 3. Pooling Architecture
**Answer**: Create a generic `PoolableTransport<T: OutgoingTransport>` wrapper that works with any transport. Handle SSE reconnection at the pool level with health checks and automatic recovery.

### 4. Error Boundaries
**Answer**: Each layer handles its own errors and wraps them appropriately:
- Transport layer: Connection and I/O errors
- Protocol layer: Serialization and validation errors
- Proxy layer: Business logic and routing errors
- Each layer converts lower-level errors to its domain

## Conclusion

This architecture proposal addresses all identified issues while maintaining shadowcat's performance targets and extensibility goals. The phased migration approach ensures we can deliver value incrementally while maintaining system stability. By unifying on the proven directional transport model and establishing clear domain boundaries, we create a maintainable foundation for future growth.

The key insight is that the current problems stem from architectural divergence, not fundamental design flaws. By converging on a unified architecture, we eliminate complexity while preserving the best aspects of both current implementations.

---

**Next Steps**:
1. Review proposal with team
2. Get buy-in on approach
3. Begin Phase 1 implementation
4. Track progress in transport-type-architecture-tracker.md