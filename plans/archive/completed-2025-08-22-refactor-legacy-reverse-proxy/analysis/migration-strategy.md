# Migration Strategy

## Overview
Incremental migration from monolithic `legacy.rs` (3,682 lines) to modular architecture while maintaining full functionality and test coverage.

## Migration Principles

1. **Zero Downtime**: All changes must maintain working state
2. **Test Coverage**: No reduction in test coverage at any step
3. **Incremental Steps**: Small, reversible changes
4. **Compatibility Layers**: Temporary bridges during transition
5. **Feature Flags**: Control rollout of new modules

## Phase-by-Phase Migration

### Phase 1: Foundation Setup (Day 1-2)
**Risk Level: Low**
**Rollback Time: < 5 minutes**

#### Step 1.1: Create Module Structure
```bash
# Create directory structure
mkdir -p src/proxy/reverse/{config,server,handlers,transport,session,middleware,response}

# Create mod.rs files
touch src/proxy/reverse/{config,server,handlers,transport,session,middleware,response}/mod.rs
```

#### Step 1.2: Extract Error Types
```rust
// src/proxy/reverse/error.rs
pub enum ReverseProxyError { ... }
pub type Result<T> = std::result::Result<T, ReverseProxyError>;

// In legacy.rs - temporary re-export
pub use error::{ReverseProxyError, Result};
```

#### Step 1.3: Extract Configurations
```rust
// src/proxy/reverse/config/mod.rs
mod upstream;
mod session;
mod server;

pub use upstream::*;
pub use session::*;
pub use server::*;

// Move config structs to respective files
// Keep legacy.rs using them via imports
```

#### Verification Steps:
- [ ] `cargo build` succeeds
- [ ] `cargo test --lib` passes
- [ ] `cargo clippy` no warnings

### Phase 2: Simple Extractions (Day 3-4)
**Risk Level: Low-Medium**
**Rollback Time: < 10 minutes**

#### Step 2.1: Extract Metrics
```rust
// src/proxy/reverse/metrics.rs
pub struct ReverseProxyMetrics { ... }

// Legacy.rs compatibility
use metrics::ReverseProxyMetrics;
```

#### Step 2.2: Extract Builder
```rust
// src/proxy/reverse/server/builder.rs
pub struct ReverseProxyServerBuilder { ... }

// Update imports in legacy.rs
```

#### Step 2.3: Extract Admin Handlers
```rust
// src/proxy/reverse/handlers/admin.rs
pub async fn handle_health() -> impl IntoResponse { ... }
pub async fn handle_metrics(...) -> impl IntoResponse { ... }

// In legacy.rs router setup
use handlers::admin::{handle_health, handle_metrics};
```

### Phase 3: Interface Introduction (Day 5-6)
**Risk Level: Medium**
**Rollback Time: < 30 minutes**

#### Step 3.1: Define Core Traits
```rust
// src/proxy/reverse/handlers/mod.rs
#[async_trait]
pub trait RequestHandler: Send + Sync {
    async fn handle(&self, ctx: RequestContext) -> Result<Response>;
}

// Create wrapper for existing functions
pub struct LegacyHandler;

#[async_trait]
impl RequestHandler for LegacyHandler {
    async fn handle(&self, ctx: RequestContext) -> Result<Response> {
        // Call existing handle_mcp_request
        super::legacy::handle_mcp_request_internal(ctx).await
    }
}
```

#### Step 3.2: Introduce Compatibility Layer
```rust
// src/proxy/reverse/compat.rs
/// Temporary compatibility layer during migration
pub mod compat {
    use super::*;
    
    /// Convert old function signature to new trait
    pub fn wrap_handler<F>(f: F) -> Box<dyn RequestHandler>
    where
        F: Fn(Request) -> Response + Send + Sync + 'static
    {
        Box::new(FunctionHandler(f))
    }
}
```

### Phase 4: Transport Extraction (Day 7-9)
**Risk Level: Medium-High**
**Rollback Time: < 1 hour**

#### Step 4.1: Extract Transport Processing
```rust
// src/proxy/reverse/transport/stdio.rs
pub async fn process_via_stdio_pooled(...) -> Result<Value> {
    // Move function from legacy.rs
}

// src/proxy/reverse/transport/http.rs
pub async fn process_via_http(...) -> Result<Value> {
    // Move function from legacy.rs
}
```

#### Step 4.2: Create Transport Router
```rust
// src/proxy/reverse/transport/router.rs
pub struct TransportRouter { ... }

impl TransportRouter {
    pub async fn route(&self, upstream: &UpstreamConfig, msg: Value) -> Result<Value> {
        match upstream.transport_type {
            TransportType::Stdio => stdio::process_via_stdio_pooled(...).await,
            TransportType::Http => http::process_via_http(...).await,
        }
    }
}
```

#### Step 4.3: Update Legacy to Use Router
```rust
// In legacy.rs
use transport::router::TransportRouter;

// Replace inline transport selection with router
let router = TransportRouter::new(app_state.clone());
let response = router.route(&upstream, message).await?;
```

### Phase 5: SSE Handler Extraction (Day 10-12)
**Risk Level: High**
**Rollback Time: < 2 hours**

#### Step 5.1: Create SSE Module
```rust
// src/proxy/reverse/handlers/sse.rs
pub struct SseHandler { ... }

pub async fn handle_sse_request(...) -> Result<Sse<impl Stream>> {
    // Move SSE handling logic
}
```

#### Step 5.2: Extract Streaming Logic
```rust
// src/proxy/reverse/handlers/sse.rs
async fn proxy_sse_from_upstream(...) -> Result<()> {
    // Move streaming implementation
}
```

#### Step 5.3: Integration Testing
```rust
#[cfg(test)]
mod sse_tests {
    // Create comprehensive SSE tests
    // Ensure behavior matches legacy exactly
}
```

### Phase 6: Core Handler Refactoring (Day 13-15)
**Risk Level: Very High**
**Rollback Time: < 4 hours**

#### Step 6.1: Break Down handle_mcp_request
```rust
// src/proxy/reverse/handlers/mcp.rs
pub struct McpHandler { ... }

impl McpHandler {
    async fn parse_request(&self, req: Request) -> Result<ParsedRequest> { ... }
    async fn validate_session(&self, parsed: &ParsedRequest) -> Result<Session> { ... }
    async fn process_message(&self, msg: Value, session: &Session) -> Result<Value> { ... }
    async fn format_response(&self, result: Value) -> Result<Response> { ... }
}
```

#### Step 6.2: Parallel Implementation
```rust
// Keep legacy.rs handle_mcp_request active
// Add feature flag for new implementation
#[cfg(feature = "new_handler")]
use handlers::mcp::McpHandler;

#[cfg(not(feature = "new_handler"))]
use legacy::handle_mcp_request;
```

#### Step 6.3: A/B Testing
```rust
// Gradual rollout with percentage
if rand::random::<f64>() < 0.1 {  // 10% traffic to new handler
    new_handler.handle(request).await
} else {
    legacy_handler.handle(request).await
}
```

### Phase 7: Server Module Migration (Day 16-18)
**Risk Level: High**
**Rollback Time: < 2 hours**

#### Step 7.1: Extract Server Implementation
```rust
// src/proxy/reverse/server/mod.rs
pub struct ReverseProxyServer {
    inner: Arc<ServerInner>,
}

// src/proxy/reverse/server/state.rs
pub(crate) struct AppState { ... }
```

#### Step 7.2: Move Router Configuration
```rust
// src/proxy/reverse/server/router.rs
pub fn create_router(state: AppState) -> Router {
    // Move router setup from legacy.rs
}
```

### Phase 8: Cleanup and Optimization (Day 19-20)
**Risk Level: Low**
**Rollback Time: N/A (final state)**

#### Step 8.1: Remove Legacy File
```bash
# After all tests pass with new modules
git mv src/proxy/reverse/legacy.rs src/proxy/reverse/legacy.rs.bak
```

#### Step 8.2: Update Public API
```rust
// src/proxy/reverse/mod.rs
pub use server::ReverseProxyServer;
pub use config::{ReverseProxyConfig, UpstreamConfig};
pub use error::{ReverseProxyError, Result};
// ... other public exports
```

#### Step 8.3: Documentation
```rust
//! # Reverse Proxy Module
//! 
//! Modular MCP reverse proxy implementation
//! 
//! ## Architecture
//! - `config/`: Configuration types
//! - `server/`: Server lifecycle management
//! - `handlers/`: Request handlers
//! - `transport/`: Transport implementations
```

## Testing Strategy

### Test Migration Approach
1. **Keep existing tests**: Don't modify until module works
2. **Add integration tests**: Test new modules against legacy
3. **Parallel test suites**: Run both old and new tests
4. **Gradual migration**: Move tests with their code

### Test Verification Matrix
| Phase | Unit Tests | Integration Tests | E2E Tests | Performance |
|-------|-----------|------------------|-----------|-------------|
| 1 | ✅ Existing | ✅ Existing | ✅ Existing | Baseline |
| 2 | ✅ + New | ✅ Existing | ✅ Existing | No change |
| 3 | ✅ + New | ✅ + Compat | ✅ Existing | No change |
| 4 | ✅ + New | ✅ + Transport | ✅ Existing | Monitor |
| 5 | ✅ + New | ✅ + SSE | ✅ Existing | Monitor |
| 6 | ✅ Migrated | ✅ New | ✅ + New | Benchmark |
| 7 | ✅ Migrated | ✅ New | ✅ New | Benchmark |
| 8 | ✅ Final | ✅ Final | ✅ Final | Validate |

## Rollback Procedures

### Phase 1-2 Rollback
```bash
# Simple file deletion
git checkout -- src/proxy/reverse/{error,config,metrics}.rs
# Update imports in legacy.rs
```

### Phase 3-4 Rollback
```bash
# Remove trait implementations
git checkout -- src/proxy/reverse/handlers/
# Restore direct function calls
```

### Phase 5-6 Rollback
```bash
# Switch feature flag
cargo build --no-default-features --features=legacy_handler
# Or revert commits
git revert HEAD~5..HEAD
```

### Phase 7-8 Rollback
```bash
# Restore from backup
mv src/proxy/reverse/legacy.rs.bak src/proxy/reverse/legacy.rs
# Update mod.rs to use legacy
```

## Monitoring During Migration

### Key Metrics to Track
1. **Response Time**: p50, p95, p99 latencies
2. **Error Rate**: 4xx and 5xx responses
3. **Memory Usage**: Heap and stack allocation
4. **CPU Usage**: Per-request processing time
5. **Test Coverage**: Maintain >80% coverage

### Alert Thresholds
```yaml
alerts:
  - name: latency_increase
    condition: p95_latency > baseline * 1.1  # 10% increase
    action: investigate
    
  - name: error_spike
    condition: error_rate > baseline * 1.5  # 50% increase
    action: rollback
    
  - name: memory_leak
    condition: memory_growth > 100MB/hour
    action: rollback
```

## Success Criteria

### Per-Phase Success
- [ ] All tests passing
- [ ] No performance regression
- [ ] No increase in error rate
- [ ] Code compiles without warnings
- [ ] Clippy passes

### Overall Success
- [ ] Legacy.rs eliminated
- [ ] All modules < 500 lines
- [ ] Test coverage maintained
- [ ] Performance improved or unchanged
- [ ] Clean module boundaries
- [ ] Documentation complete

## Risk Mitigation

### High-Risk Operations
1. **SSE Handler Migration**: Test with synthetic load
2. **Core Handler Refactor**: Use feature flags
3. **State Management Changes**: Extensive concurrent testing

### Contingency Plans
1. **Performance Degradation**: Keep legacy.rs as fallback
2. **Unexpected Bugs**: Feature flag for quick disable
3. **Integration Issues**: Compatibility layer for gradual migration

## Timeline Summary

| Week | Phase | Risk | Key Deliverable |
|------|-------|------|-----------------|
| 1 | Phase 1-2 | Low | Config/Error extraction |
| 2 | Phase 3-4 | Medium | Transport extraction |
| 3 | Phase 5-6 | High | Handler refactoring |
| 4 | Phase 7-8 | Medium | Server module, cleanup |

**Total Duration**: 4 weeks (20 working days)
**Buffer Time**: +1 week for unexpected issues
**Parallel Work**: Documentation can proceed alongside