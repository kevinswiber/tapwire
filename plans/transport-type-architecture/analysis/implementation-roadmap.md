# Implementation Roadmap

**Created**: 2025-08-16  
**Purpose**: Step-by-step implementation guide for transport architecture refactor

## Overview

This roadmap provides concrete implementation steps for the transport architecture refactor, organized into three phases with specific tasks, file changes, and validation steps.

## Phase 1: Quick Fix - Add ResponseMode (4 hours)

### Objective
Eliminate the `is_sse_session` code smell by introducing proper response mode tracking.

### Task 1.1: Create ResponseMode Enum (30 min)

**Files to Create**:
```
src/transport/core/response_mode.rs
```

**Implementation**:
```rust
// src/transport/core/response_mode.rs
use serde::{Deserialize, Serialize};

use mime::Mime;

/// Represents the format of a response from the upstream server
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseMode {
    /// Standard JSON-RPC response
    Json,
    /// Server-Sent Events stream
    SseStream,
    /// Any other content type - passthrough without processing
    Passthrough,
}

impl ResponseMode {
    pub fn from_content_type(content_type: &str) -> Self {
        match content_type.parse::<Mime>() {
            Ok(mime) => {
                match (mime.type_(), mime.subtype()) {
                    (mime::APPLICATION, mime::JSON) => Self::Json,
                    (mime::TEXT, subtype) if subtype == "event-stream" => Self::SseStream,
                    _ => Self::Passthrough,
                }
            }
            Err(_) => Self::Passthrough,
        }
    }
    
    pub fn is_streaming(&self) -> bool {
        matches!(self, Self::SseStream)
        // Note: Will include WebSocket when MCP spec supports it
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_response_mode_detection() {
        assert_eq!(
            ResponseMode::from_content_type("application/json"),
            ResponseMode::Json
        );
        assert_eq!(
            ResponseMode::from_content_type("text/event-stream"),
            ResponseMode::SseStream
        );
    }
}
```

**Files to Modify**:
```
src/transport/core/mod.rs - Export ResponseMode
src/transport/mod.rs - Re-export from core
```

### Task 1.2: Update Session Structure (1 hour)

**Files to Modify**:
```
src/session/store.rs
```

**Changes**:
```rust
// src/session/store.rs

use crate::transport::core::ResponseMode;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub transport_type: TransportType,
    // Remove: pub is_sse_session: bool,
    pub response_mode: Option<ResponseMode>, // Add this
    pub supports_streaming: bool, // Add this
    // ... other fields ...
}

impl Session {
    pub fn new(id: SessionId, transport_type: TransportType) -> Self {
        Session {
            id,
            transport_type,
            response_mode: None,
            supports_streaming: transport_type.supports_streaming(),
            // ... other fields ...
        }
    }
    
    // Remove: mark_as_sse_session() method
    // Remove: is_sse() method
    
    // Add new methods
    pub fn set_response_mode(&mut self, mode: ResponseMode) {
        self.response_mode = Some(mode);
        self.last_activity = Instant::now();
    }
    
    pub fn is_streaming(&self) -> bool {
        self.response_mode
            .map(|m| m.is_streaming())
            .unwrap_or(false)
    }
}
```

### Task 1.3: Update Response Detection (1.5 hours)

**Files to Modify**:
```
src/proxy/reverse/hyper_client.rs
src/proxy/reverse/legacy.rs
```

**Changes in hyper_client.rs**:
```rust
// src/proxy/reverse/hyper_client.rs

use crate::transport::core::ResponseMode;

impl HyperResponse {
    // Remove is_sse() completely - no compatibility needed
    // Only keep the new method
    pub fn response_mode(&self) -> ResponseMode {
        self.response
            .headers()
            .get(hyper::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(ResponseMode::from_content_type)
            .unwrap_or(ResponseMode::Passthrough)
    }
}
```

**Changes in legacy.rs**:
```rust
// src/proxy/reverse/legacy.rs

// In process_via_http_hyper function
let response_mode = hyper_response.response_mode();

// Update session with response mode
if let Some(session) = session_manager.get_mut(&session_id).await {
    session.set_response_mode(response_mode);
}

// Use response mode for routing
match response_mode {
    ResponseMode::SseStream => {
        // Handle SSE streaming
        forward_sse_with_interceptors(/* ... */).await
    }
    ResponseMode::Json | ResponseMode::Unknown => {
        // Handle JSON response
        Ok((StatusCode::OK, response_headers, Json(json_response)).into_response())
    }
    _ => {
        // Handle other formats
        forward_raw_response(/* ... */).await
    }
}
```

### Task 1.4: Clean Up Dead Code (1 hour)

**Files to Modify**:
```
src/session/store.rs - Remove is_sse_session field and methods
src/proxy/reverse/sse_resilience.rs - Update to use response_mode
tests/integration_*.rs - Update test fixtures
```

**Validation Steps**:
1. Run `cargo build` - Must compile without errors
2. Run `cargo test session` - Session tests must pass
3. Run `cargo test reverse_proxy` - Reverse proxy tests must pass
4. Run `cargo clippy` - No new warnings

## Phase 2: Extract Shared Transport Logic (6 hours)

### Task 2.1: Create Raw Transport Primitives Module (2 hours)

**Files to Create**:
```
src/transport/raw/mod.rs
src/transport/raw/stdio.rs
src/transport/raw/http.rs
src/transport/raw/sse.rs
```

**Implementation Structure**:
```rust
// src/transport/raw/stdio.rs

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use crate::transport::buffer_pool::BytesPool;

/// Raw stdio I/O operations (no MCP knowledge)
/// Already well-implemented using tokio::io - no changes needed
pub struct StdioCore {
    buffer_pool: Arc<BytesPool>,
    stdin: Option<BufReader<tokio::io::Stdin>>,
    stdout: Option<tokio::io::Stdout>,
    process: Option<Child>,  // For subprocess mode
}

impl StdioCore {
    pub async fn send_bytes(&mut self, data: &[u8]) -> Result<()> {
        // Already using tokio::io::stdout() - optimal
        let stdout = self.stdout.as_mut().ok_or(Error::NotConnected)?;
        stdout.write_all(data).await?;
        stdout.flush().await?;
        Ok(())
    }
    
    pub async fn receive_bytes(&mut self) -> Result<Vec<u8>> {
        // Already using tokio::io::stdin() with buffer pooling - optimal
        let stdin = self.stdin.as_mut().ok_or(Error::NotConnected)?;
        // ... existing efficient implementation
    }
}
```

### Task 2.2: Refactor Directional Transports (2 hours)

**Files to Modify**:
```
src/transport/directional/incoming.rs
src/transport/directional/outgoing.rs
```

**Refactor Pattern**:
```rust
// src/transport/directional/incoming.rs

use crate::transport::raw::stdio::StdioCore;

pub struct StdioIncoming {
    core: StdioCore,
    session_id: SessionId,
}

impl IncomingTransport for StdioIncoming {
    async fn receive_request(&mut self) -> TransportResult<MessageEnvelope> {
        let bytes = self.core.receive_bytes().await?;
        // Deserialize using shared protocol layer
        protocol::deserialize(&bytes)
    }
    
    // Delegate other methods to core
}
```

### Task 2.3: Create Unified Factory (2 hours)

**Files to Create**:
```
src/transport/factory/mod.rs
src/transport/factory/builder.rs
src/transport/factory/config.rs
```

**Implementation**:
```rust
// src/transport/factory/mod.rs

pub struct TransportFactory {
    buffer_pools: Arc<BufferPools>,
    connection_pool: Arc<ConnectionPool>,
    config: TransportConfig,
}

impl TransportFactory {
    pub async fn create_incoming(
        &self,
        transport_type: TransportType,
    ) -> Result<Box<dyn IncomingTransport>> {
        let transport = match transport_type {
            TransportType::Stdio => {
                Box::new(StdioIncoming::new(self.buffer_pools.clone()))
                    as Box<dyn IncomingTransport>
            }
            TransportType::StreamableHttp => {
                Box::new(HttpIncoming::new(
                    self.config.bind_addr,
                    self.buffer_pools.clone(),
                )) as Box<dyn IncomingTransport>
            }
        };
        
        Ok(transport)
    }
    
    // Similar for create_outgoing with pooling support
}
```

**Integration Points**:
```
src/cli/forward.rs - Use factory for transport creation
src/cli/reverse.rs - Use factory for transport creation
```

## Phase 3: Unify Proxy Architecture (8 hours)

### Task 3.1: Create ProxyCore Abstraction (2 hours)

**Files to Create**:
```
src/proxy/core/mod.rs
src/proxy/core/router.rs
src/proxy/core/pipeline.rs
```

**Implementation**:
```rust
// src/proxy/core/mod.rs

pub struct ProxyCore {
    session_manager: Arc<SessionManager>,
    interceptor_chain: Arc<InterceptorChain>,
    transport_factory: Arc<TransportFactory>,
}

impl ProxyCore {
    pub async fn run_proxy(
        &self,
        mut incoming: Box<dyn IncomingTransport>,
        mut outgoing: Box<dyn OutgoingTransport>,
    ) -> Result<()> {
        // Shared proxy pipeline
        loop {
            let request = incoming.receive_request().await?;
            let request = self.interceptor_chain.process_request(request).await?;
            
            outgoing.send_request(request).await?;
            
            let response = outgoing.receive_response().await?;
            let response_mode = detect_response_mode(&response);
            
            self.handle_response(incoming.as_mut(), response, response_mode).await?;
        }
    }
}
```

### Task 3.2: Refactor Reverse Proxy (4 hours)

**Files to Modify**:
```
src/proxy/reverse/mod.rs - New cleaner entry point
src/proxy/reverse/legacy.rs - Gradually migrate logic
```

**Migration Strategy**:
1. Create new `mod.rs` with clean implementation
2. Move logic piece by piece from legacy.rs
3. Update routing to use directional transports
4. Delete legacy.rs when complete

**New Structure**:
```rust
// src/proxy/reverse/mod.rs

use crate::proxy::core::ProxyCore;
use crate::transport::factory::TransportFactory;

pub struct ReverseProxy {
    core: ProxyCore,
    auth_gateway: Option<AuthGateway>,
    sse_resilience: SseResilience,
}

impl ReverseProxy {
    pub async fn serve(
        &self,
        bind_addr: SocketAddr,
        upstream_config: UpstreamConfig,
    ) -> Result<()> {
        let factory = &self.core.transport_factory;
        
        // Create incoming transport for HTTP server
        let incoming = factory.create_incoming(
            TransportType::StreamableHttp
        ).await?;
        
        // Create outgoing based on config
        let outgoing = factory.create_outgoing(
            upstream_config.transport_type,
            &upstream_config.target,
        ).await?;
        
        // Use shared proxy core
        self.core.run_proxy(incoming, outgoing).await
    }
}
```

### Task 3.3: Update Connection Pooling (2 hours)

**Files to Create**:
```
src/transport/implementations/pool.rs
```

**Implementation**:
```rust
// src/transport/implementations/pool.rs

pub struct GenericPool<T: OutgoingTransport> {
    connections: Arc<Mutex<HashMap<String, Vec<T>>>>,
    max_per_target: usize,
    health_check_interval: Duration,
}

impl<T: OutgoingTransport> GenericPool<T> {
    pub async fn acquire(&self, target: &str) -> Option<T> {
        let mut connections = self.connections.lock().await;
        connections.get_mut(target)?.pop()
    }
    
    pub async fn release(&self, target: String, transport: T) {
        if transport.is_healthy().await {
            let mut connections = self.connections.lock().await;
            connections.entry(target)
                .or_insert_with(Vec::new)
                .push(transport);
        }
    }
}
```

## Validation and Testing Strategy

### Unit Test Updates

**Phase 1 Tests**:
```rust
// tests/unit/response_mode_test.rs
#[test]
fn test_response_mode_detection() { /* ... */ }

#[test]
fn test_session_response_tracking() { /* ... */ }
```

**Phase 2 Tests**:
```rust
// tests/unit/shared_transport_test.rs
#[test]
fn test_stdio_core_operations() { /* ... */ }

#[test]
fn test_transport_factory_creation() { /* ... */ }
```

**Phase 3 Tests**:
```rust
// tests/unit/proxy_core_test.rs
#[test]
fn test_unified_proxy_pipeline() { /* ... */ }

#[test]
fn test_connection_pooling() { /* ... */ }
```

### Integration Test Updates

```bash
# Run after each phase
cargo test --test integration_forward_proxy
cargo test --test integration_reverse_proxy
cargo test --test e2e_complete_flow
```

### Performance Validation

```bash
# Benchmark before changes
cargo bench --bench transport_performance > baseline.txt

# Benchmark after each phase
cargo bench --bench transport_performance > phase_N.txt

# Compare results
diff baseline.txt phase_N.txt
```

## Rollback Plan

### Git Tags
```bash
# Tag before starting
git tag pre-transport-refactor

# Tag after each phase
git tag transport-refactor-phase-1
git tag transport-refactor-phase-2
git tag transport-refactor-phase-3
```

### Feature Flags (Optional)
```rust
// If risk is high, use feature flags
#[cfg(feature = "new-transport")]
mod new_implementation;

#[cfg(not(feature = "new-transport"))]
mod old_implementation;
```

## Timeline

### Week 1
- **Day 1**: Phase 1 implementation (4 hours)
- **Day 2**: Phase 1 testing and validation
- **Day 3-4**: Phase 2 implementation (6 hours)
- **Day 5**: Phase 2 testing and documentation

### Week 2
- **Day 1-3**: Phase 3 implementation (8 hours)
- **Day 4**: Integration testing
- **Day 5**: Performance validation and optimization

## Success Criteria Checklist

### Phase 1 Complete When:
- [ ] ResponseMode enum implemented and tested
- [ ] Session struct updated without is_sse_session
- [ ] All references to is_sse_session removed
- [ ] All existing tests pass
- [ ] No clippy warnings

### Phase 2 Complete When:
- [ ] Shared transport implementations created
- [ ] Directional transports refactored to use shared logic
- [ ] Transport factory operational
- [ ] Code duplication reduced by >50%
- [ ] Performance benchmarks show <5% overhead

### Phase 3 Complete When:
- [ ] ProxyCore abstraction implemented
- [ ] Reverse proxy uses directional transports
- [ ] Connection pooling works for all transport types
- [ ] All integration tests pass
- [ ] Architecture documentation updated

## Risk Monitoring

### Key Metrics to Watch
1. **Test Coverage**: Must not drop below 80%
2. **Performance**: P95 latency must stay <5% overhead
3. **Memory Usage**: Must stay within 100MB for 1000 sessions
4. **Error Rate**: Must not increase in production

### Canary Testing
1. Deploy to staging after each phase
2. Run load tests with production-like traffic
3. Monitor for 24 hours before proceeding

## Documentation Updates

### After Each Phase
1. Update API documentation
2. Update architecture diagrams
3. Update developer guide
4. Create migration notes

### Final Documentation
1. Complete architecture guide
2. Best practices document
3. Troubleshooting guide
4. Performance tuning guide

---

**Next Step**: Begin Phase 1 implementation with Task 1.1