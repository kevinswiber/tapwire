# Final Architecture Plan - Refactor Legacy Reverse Proxy

## Overview
Final refined architecture incorporating feedback from multiple reviews. This plan removes admin UI, avoids naming conflicts, leverages existing transport modules, and creates thin handlers with clear separation of concerns.

## Core Principles

1. **Thin handlers** - Handlers only orchestrate, don't implement business logic
2. **Reuse transport** - Leverage existing transport::sse and analyze other overlaps
3. **No naming conflicts** - Clear distinction from core modules
4. **Pipeline pattern** - Cross-cutting concerns in dedicated pipeline module
5. **Single responsibility** - Each module does one thing well

## Final Module Structure

```
src/proxy/reverse/
├── mod.rs                    // Public API exports
├── error.rs                  // ReverseProxyError (50 lines)
├── config.rs                 // All config types (250 lines)
├── state.rs                  // AppState (100 lines)
├── metrics.rs                // ReverseProxyMetrics (50 lines)
├── server.rs                 // ReverseProxyServer + Builder (200 lines)
├── router.rs                 // Router setup with layers (100 lines)
├── handlers/
│   ├── mod.rs               // Handler exports (20 lines)
│   ├── mcp.rs               // /mcp endpoint - THIN (100 lines)
│   └── health.rs            // /health, /metrics endpoints (50 lines)
├── pipeline.rs              // Intercept/pause/record logic (200 lines)
├── session_helpers.rs       // Session operations (150 lines)
├── headers.rs               // Header utilities (100 lines)
└── upstream/
    ├── mod.rs               // UpstreamService trait (50 lines)
    ├── selector.rs          // Load balancing strategies (100 lines)
    ├── stdio.rs             // Stdio upstream impl (200 lines)
    └── http/
        ├── mod.rs           // HttpUpstream impl (50 lines)
        ├── client.rs        // Hyper client wrapper (150 lines)
        ├── relay.rs         // JSON/non-SSE responses (150 lines)
        └── sse_adapter.rs   // Adapts transport::sse (100 lines)
```

**Total Lines**: ~1,970 (down from 3,682 after removing ~900 admin lines)
**Largest Module**: 250 lines (config.rs)
**All modules**: Under 300 lines ✅

## Key Design Decisions

### 1. Naming Solutions

| Original Concern | Solution |
|-----------------|----------|
| `forward.rs` confusion | → `relay.rs` (clearer for reverse proxy) |
| `transport/` conflict | → `upstream/` (specific to upstream servers) |
| `session/` conflict | → `session_helpers.rs` (single file, clear purpose) |

### 2. Leveraging Existing Transport

```rust
// upstream/http/sse_adapter.rs - Thin adapter
use crate::transport::sse::{SseParser, SseEvent};

pub async fn create_sse_stream(
    body: hyper::Body,
    session: &Session,
) -> impl Stream<Item = Result<SseEvent>> {
    // Reuse transport::sse infrastructure
    let parser = SseParser::new(body);
    
    // Add reverse-proxy specific transforms
    apply_reverse_proxy_transforms(parser, session)
}
```

### 3. Pipeline Pattern for Cross-Cutting Concerns

```rust
// pipeline.rs - Single file for all pipeline operations
pub mod intercept {
    pub async fn apply_inbound(...) -> Result<Message> { }
    pub async fn apply_outbound(...) -> Result<Response> { }
}

pub mod pause {
    pub async fn check_pause_state(...) -> Result<()> { }
}

pub mod record {
    pub async fn record_request(...) -> Result<()> { }
    pub async fn record_response(...) -> Result<()> { }
}
```

### 4. Thin Handler Example

```rust
// handlers/mcp.rs - Only orchestration, no business logic
pub async fn handle_mcp_request(
    State(app): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response> {
    // 1. Session setup
    let session = session_helpers::get_or_create(&app, &headers).await?;
    
    // 2. Pipeline processing
    let msg = pipeline::intercept::apply_inbound(&app, body).await?;
    pipeline::pause::check_pause_state(&app, &session).await?;
    
    // 3. Upstream selection and sending
    let upstream = app.upstream_selector.select(&session).await?;
    let response = upstream.send(msg, &session).await?;
    
    // 4. Response handling
    pipeline::record::record_response(&app, &response).await?;
    
    match response {
        UpstreamResponse::Json(val, hdrs) => 
            Ok(create_json_response(val, hdrs)),
        UpstreamResponse::SseStream(stream) => 
            Ok(Sse::new(stream).into_response()),
    }
}
```

## The UpstreamService Trait

```rust
// upstream/mod.rs
#[async_trait]
pub trait UpstreamService: Send + Sync {
    async fn send(
        &self,
        msg: ProtocolMessage,
        session: &Session,
    ) -> Result<UpstreamResponse>;
    
    async fn health_check(&self) -> Result<HealthStatus>;
}

pub enum UpstreamResponse {
    Json {
        status: StatusCode,
        headers: HeaderMap,
        body: Value,
    },
    SseStream(Box<dyn Stream<Item = Result<SseEvent>> + Send>),
}

// Implementations
pub struct StdioUpstream { /* ... */ }
pub struct HttpUpstream { /* ... */ }
```

## Router with Explicit Layer Ordering

```rust
// router.rs - All middleware in one place, order explicit
use tower::ServiceBuilder;

pub fn build_router(app: AppState, cfg: &ReverseProxyConfig) -> Router {
    let routes = Router::new()
        .route("/mcp", post(handlers::mcp::post).get(handlers::mcp::get))
        .route("/health", get(handlers::health::get))
        .route("/metrics", get(handlers::health::metrics))
        .with_state(app.clone());
    
    // Layer order matters! First added = outermost
    let middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(DefaultBodyLimit::max(cfg.max_body_size));
    
    if let Some(auth) = &app.auth_gateway {
        middleware = middleware.layer(auth.into_layer());
    }
    
    if let Some(rate_limiter) = &app.rate_limiter {
        middleware = middleware.layer(rate_limiter.into_layer());
    }
    
    routes.layer(middleware)
}
```

## Migration Strategy (Updated)

### Phase 0: Analysis & Admin Removal (Day 1)
- [x] Complete architecture analysis
- [ ] Analyze transport overlap (see transport-overlap-analysis.md)
- [ ] Remove admin UI (~900 lines)
- [ ] Remove admin tests
- [ ] Verify remaining tests pass

### Phase 1: Type Extraction (Day 1-2)
- [ ] Extract error.rs
- [ ] Extract config.rs
- [ ] Extract metrics.rs
- [ ] Extract state.rs

### Phase 2: Infrastructure (Day 2-3)
- [ ] Create pipeline.rs (intercept/pause/record)
- [ ] Create session_helpers.rs
- [ ] Create headers.rs
- [ ] Create UpstreamService trait

### Phase 3: Upstream Implementation (Day 3-4)
- [ ] Implement upstream/stdio.rs (using transport where possible)
- [ ] Implement upstream/http/client.rs
- [ ] Implement upstream/http/relay.rs
- [ ] Implement upstream/http/sse_adapter.rs (using transport::sse)
- [ ] Implement upstream/selector.rs

### Phase 4: Handler Thinning (Day 4-5)
- [ ] Extract router.rs
- [ ] Create thin handlers/mcp.rs
- [ ] Create handlers/health.rs
- [ ] Update server.rs to use new router

### Phase 5: Cleanup (Day 5)
- [ ] Delete legacy.rs
- [ ] Organize tests
- [ ] Update documentation
- [ ] Performance validation

## Critical Success Factors

1. **Tests pass at each phase** - Never break functionality
2. **Transport reuse** - Identify and eliminate duplication
3. **Handler thinness** - Handlers under 150 lines
4. **Clear boundaries** - No module does two things
5. **Performance maintained** - No regression from refactor

## Open Questions (To Resolve During Implementation)

1. **Transport overlap**: How much of stdio/http can reuse transport module?
2. **Pool management**: Should connection pools move to transport?
3. **Interceptor chain**: Should this be a general transport feature?
4. **Session abstraction**: Is session_helpers the right boundary?

## Comparison with Previous Designs

| Aspect | Original | GPT-5 | Final |
|--------|----------|--------|-------|
| Admin UI | Extract to module | Not addressed | Delete entirely |
| SSE handling | Separate handler | upstream/http/sse.rs | sse_adapter using transport::sse |
| forward.rs naming | Not considered | Potential confusion | relay.rs instead |
| Transport reuse | Not considered | Not considered | Explicit reuse strategy |
| Handler size | 400-500 lines | "Thin" | <150 lines target |
| Pipeline pattern | Not considered | pipeline/ directory | Single pipeline.rs file |

## Conclusion

This final architecture:
- Removes ~1,700 lines from legacy.rs
- Creates 14 focused modules, all under 300 lines
- Maximizes reuse of existing transport infrastructure
- Provides clear, thin handlers
- Avoids all naming conflicts
- Sets up for future transport consolidation

Ready for implementation with clear phases and success criteria.