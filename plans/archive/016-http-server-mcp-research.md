# HTTP Server Framework & MCP Protocol Research Report

**Research Period:** August 4, 2025  
**Researcher:** Claude Code Session  
**Status:** Complete - Day 1-2 Research Deliverable  
**Purpose:** Technical analysis for Shadowcat Phase 5 reverse proxy implementation

---

## Executive Summary

**Key Findings:**
- **Axum** emerges as the optimal HTTP framework choice, balancing performance, ergonomics, and ecosystem maturity
- **MCP 2025-06-18** specification introduces Streamable HTTP transport with significant improvements over previous versions
- **Strong ecosystem support** for JWT authentication middleware and reverse proxy patterns in Axum
- **Performance targets achievable** with < 5ms authentication overhead using proper connection pooling and middleware design

**Critical Decisions:**
1. **Framework Choice:** Axum for HTTP server implementation
2. **MCP Transport:** Implement Streamable HTTP transport per 2025-06-18 specification  
3. **Connection Management:** Use reqwest with custom connection pooling for upstream clients
4. **Middleware Strategy:** Layer-based architecture with JWT authentication and policy enforcement

---

## Research Methodology

### Approach and Criteria
- **Performance Benchmarking:** Analysis of 2025 benchmark data across multiple sources
- **Specification Review:** Deep dive into MCP 2025-06-18 Streamable HTTP transport
- **Ecosystem Analysis:** Survey of authentication middleware, reverse proxy crates, and production patterns
- **Integration Assessment:** Compatibility with existing Shadowcat architecture (Phase 4 interceptors)

### Tools and Benchmarks Performed
- Web-based performance analysis from MarkaiCode, LogRocket, and community benchmarks
- MCP specification analysis from modelcontextprotocol.io
- Ecosystem survey via crates.io and GitHub repositories
- Production pattern analysis from Rust community discussions

---

## Detailed Analysis

### HTTP Framework Comparison (2025 Benchmarks)

#### Performance Metrics

**Throughput Rankings:**
1. **Actix Web** - Highest raw throughput, leads in most categories
2. **Axum** - Close second, significantly improved from previous years
3. **Warp** - Good performance but less ergonomic API
4. **Rocket** - Lower performance, better developer experience

**Latency Characteristics:**
- Actix Web and Axum show nearly identical latency profiles
- Both maintain consistently low latency even at high request volumes
- Axum demonstrates most efficient memory usage (12-20MB typical)

**Resource Efficiency:**
- **Axum**: Most efficient memory usage, ideal for container deployments
- **Actix Web**: Handles highest concurrent connections
- **Warp**: Functional programming approach, good for composability

#### Framework Architecture Analysis

**Axum Advantages:**
- Built on Tokio/Hyper stack (same as existing Shadowcat infrastructure)
- Thin layer over hyper with minimal overhead
- Uses Tower middleware ecosystem (timeouts, tracing, compression, authorization)
- Seamless integration with existing async Rust patterns
- Growing community adoption (18.3k GitHub stars, actively maintained)

**Actix Web Considerations:**
- Highest raw performance but uses custom runtime (not Hyper-based)
- More complex generics and learning curve
- Mature ecosystem but different architectural patterns

**Warp Considerations:**
- Filter-based functional approach
- Good composability but steeper learning curve
- Built on Tokio/Hyper like Axum

### MCP Protocol Implementation Requirements

#### MCP 2025-06-18 Streamable HTTP Transport

**Key Changes from Previous Versions:**
- Replaced HTTP+SSE transport with more flexible Streamable HTTP
- Simplified server implementation requirements
- Better support for both basic and streaming-capable servers

**Technical Requirements:**

**Server Implementation:**
```
- Single HTTP endpoint supporting POST and GET methods
- JSON-RPC message handling via HTTP POST requests
- Accept header must list both application/json and text/event-stream
- HTTP 202 Accepted response for valid inputs (no body)
- Optional Server-Sent Events (SSE) for streaming multiple server messages
```

**Header Management:**
```
MCP-Protocol-Version: 2025-06-18
- MUST be included on all requests after initialization
- Allows server to respond based on negotiated protocol version
- Backwards compatibility with 2025-03-26 if header missing

MCP-Session-Id: <session-id>
- Server MAY assign session ID during initialization
- MUST be globally unique and cryptographically secure (UUID, JWT, hash)
- MUST contain only visible ASCII characters (0x21 to 0x7E)
- Client MUST include in all subsequent requests if provided
- Server responds with HTTP 400 if missing on non-initialization requests
```

**Session Lifecycle:**
```
- Server may terminate session anytime (responds with HTTP 404)
- Client SHOULD send HTTP DELETE to explicitly terminate session
- Session management critical for reverse proxy context
```

**Error Handling:**
```
- HTTP 400 Bad Request: Missing required MCP-Session-Id header
- HTTP 404 Not Found: Session terminated or invalid session ID
- HTTP 202 Accepted: Valid message received and accepted
- Standard JSON-RPC error responses within HTTP 200 OK
```

### Integration Architecture Design

#### HTTP Request → TransportMessage Conversion

```rust
// Core conversion pattern for MCP HTTP transport
async fn http_to_transport_message(
    request: HttpRequest,
    headers: &HeaderMap
) -> Result<TransportMessage, TransportError> {
    // 1. Extract MCP headers
    let session_id = headers.get("mcp-session-id")
        .and_then(|h| h.to_str().ok())
        .map(SessionId::from);
    
    let protocol_version = headers.get("mcp-protocol-version")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("2025-03-26"); // Backwards compatibility
    
    // 2. Parse JSON-RPC from HTTP body
    let json_rpc: serde_json::Value = serde_json::from_slice(&body)?;
    
    // 3. Create TransportMessage with HTTP context
    Ok(TransportMessage {
        content: json_rpc,
        session_id,
        transport_type: TransportType::Http,
        metadata: create_http_metadata(headers, protocol_version),
    })
}
```

#### Session Management Integration

```rust
// Integration with existing SessionManager
impl ReverseProxy {
    async fn handle_mcp_request(&self, request: HttpRequest) -> Result<HttpResponse, ProxyError> {
        // 1. Extract session from MCP headers
        let session_id = extract_mcp_session_id(&request.headers())?;
        
        // 2. Use existing SessionManager infrastructure
        let session = self.session_manager
            .get_or_create_session(session_id, TransportType::Http)
            .await?;
        
        // 3. Record frame in existing recording infrastructure
        if let Some(recorder) = &self.tape_recorder {
            recorder.record_frame(&session.id, &transport_message).await?;
        }
        
        // 4. Process through existing interceptor chain
        let intercept_context = InterceptContext {
            session_id: session.id.clone(),
            message: transport_message,
            transport_type: TransportType::Http,
            auth_context: Some(auth_context), // NEW in Phase 5
        };
        
        let action = self.interceptor_chain.intercept(&intercept_context).await?;
        
        // 5. Handle action and route to upstream
        self.handle_intercept_action(action, session, transport_message).await
    }
}
```

### Authentication Middleware Architecture

#### JWT Middleware Integration Pattern

Based on 2025 ecosystem analysis, optimal pattern:

```rust
// Layer-based authentication middleware with Axum
use axum::{
    middleware::{self, Next},
    extract::{Request, State},
    response::Response,
    http::{StatusCode, HeaderMap},
};

// JWT Authentication Layer
async fn jwt_auth_middleware(
    State(auth_gateway): State<Arc<AuthGateway>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Extract Bearer token from Authorization header
    let auth_header = request.headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok());
    
    let token = extract_bearer_token(auth_header)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // 2. Validate token and extract claims
    let auth_context = auth_gateway
        .authenticate_request(&token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // 3. Insert auth context into request extensions
    request.extensions_mut().insert(auth_context);
    
    // 4. Continue to next middleware/handler
    Ok(next.run(request).await)
}

// Router configuration with layered middleware
fn create_router(auth_gateway: Arc<AuthGateway>) -> Router {
    Router::new()
        .route("/mcp", post(handle_mcp_request))
        .layer(middleware::from_fn_with_state(
            auth_gateway.clone(), 
            jwt_auth_middleware
        ))
        .layer(middleware::from_fn_with_state(
            auth_gateway.clone(),
            policy_enforcement_middleware
        ))
        .with_state(auth_gateway)
}
```

### Connection Pooling and Upstream Management

#### Reqwest Client Pool Design

```rust
// Upstream connection pool for reverse proxy
pub struct UpstreamClientPool {
    clients: Arc<RwLock<HashMap<String, Arc<reqwest::Client>>>>,
    config: PoolConfig,
}

impl UpstreamClientPool {
    pub fn new(config: PoolConfig) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    pub async fn get_client(&self, upstream_url: &str) -> Result<Arc<reqwest::Client>, PoolError> {
        // 1. Check existing client
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(upstream_url) {
                return Ok(client.clone());
            }
        }
        
        // 2. Create new client with optimized settings
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(self.config.max_idle_per_host)
            .pool_idle_timeout(self.config.idle_timeout)
            .timeout(self.config.request_timeout)
            .build()?;
        
        // 3. Store and return
        let client = Arc::new(client);
        let mut clients = self.clients.write().await;
        clients.insert(upstream_url.to_string(), client.clone());
        
        Ok(client)
    }
    
    // Periodic connection refresh for load balancing
    pub async fn refresh_connections(&self) {
        let mut clients = self.clients.write().await;
        clients.clear(); // Force recreation of connections
    }
}
```

---

## Recommendations

### Primary Framework Choice: Axum

**Rationale:**
1. **Performance**: Near-identical to Actix Web in 2025 benchmarks with better memory efficiency
2. **Architecture Alignment**: Built on Tokio/Hyper, same stack as existing Shadowcat components
3. **Ecosystem**: Rich middleware ecosystem via Tower, excellent JWT authentication libraries
4. **Developer Experience**: Clean API, good error messages, extensive documentation
5. **Community**: Growing adoption, active maintenance by Tokio team

**Production Readiness:**
- Proven in production environments
- Comprehensive testing and benchmarking
- Enterprise feature support (metrics, tracing, graceful shutdown)

### MCP Transport Implementation Strategy

**Streamable HTTP Transport (2025-06-18):**
1. **Single Endpoint Design**: POST for JSON-RPC, optional GET for health/metadata
2. **Header Management**: Strict MCP-Session-Id and MCP-Protocol-Version handling
3. **Session Lifecycle**: Integration with existing SessionManager, proper cleanup
4. **Error Handling**: HTTP status codes mapping to MCP protocol errors

**Integration Points:**
- Leverage existing `TransportMessage` structure
- Extend `InterceptContext` with HTTP-specific metadata
- Maintain compatibility with stdio transport for development

### Authentication Middleware Architecture

**Layer-Based Design:**
1. **JWT Authentication Layer**: Token validation and claim extraction
2. **Policy Enforcement Layer**: Authorization decisions based on claims
3. **Audit Logging Layer**: Security event recording
4. **Rate Limiting Layer**: Abuse protection

**Implementation Libraries:**
- `axum-jwt-auth` crate for JWT middleware
- `jsonwebtoken` for token validation
- Custom policy engine leveraging Phase 4 rule engine

---

## Risk Assessment

### Known Limitations and Risks

**Framework Risks:**
- **Axum Generics Complexity**: While improved, still more complex than some alternatives
- **Ecosystem Maturity**: Newer than Actix Web, some edge cases may be undiscovered
- **Performance Gap**: Still slightly behind Actix Web in raw throughput

**MCP Transport Risks:**
- **Specification Evolution**: 2025-06-18 is recent, may have implementation edge cases
- **Session Management Complexity**: HTTP session lifecycle more complex than stdio
- **Header Validation**: Strict header requirements may cause client compatibility issues

**Integration Risks:**
- **Interceptor Performance**: HTTP-specific rule evaluation may add latency
- **Connection Pool Complexity**: Load balancing and connection refresh logic
- **Authentication Overhead**: JWT validation may exceed 5ms target under load

### Mitigation Strategies

1. **Performance Monitoring**: Comprehensive benchmarking during implementation
2. **Fallback Options**: Abstract HTTP framework behind trait for easy swapping
3. **Gradual Migration**: Implement alongside existing stdio transport initially
4. **Testing Strategy**: Extensive integration testing with real MCP clients
5. **Connection Management**: Implement connection pool monitoring and health checks

---

## Implementation Impact

### Changes to Implementation Plan

**Week 1 Adjustments:**
- Day 1: Add Axum and JWT middleware dependencies
- Day 2: Implement MCP HTTP transport with session management
- Day 3: JWT authentication middleware integration
- Day 4: Policy enforcement layer implementation
- Day 5: Connection pooling and upstream client management

**Integration Considerations:**
- Extend `TransportType` enum with `Http` variant
- Add HTTP-specific metadata to `InterceptContext`
- Implement HTTP → JSON-RPC conversion utilities
- Create HTTP status code → MCP error mapping

**Testing Requirements:**
- HTTP transport compliance testing
- JWT middleware security testing
- Connection pool performance testing
- MCP client compatibility testing

---

## References

### Specifications and Documentation
- [MCP Specification 2025-06-18](https://modelcontextprotocol.io/specification/2025-06-18)
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [Tower Middleware Guide](https://docs.rs/tower/latest/tower/)
- [OAuth 2.1 Draft Specification](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1)

### Performance Benchmarks
- [Rust Web Frameworks 2025 Benchmarks - MarkaiCode](https://markaicode.com/rust-web-frameworks-performance-benchmark-2025/)
- [Axum vs Actix Performance Analysis - LogRocket](https://blog.logrocket.com/using-rust-axum-build-jwt-authentication-api/)
- [Rust HTTP Framework Comparison - Shuttle](https://www.shuttle.dev/blog/2023/08/23/rust-web-framework-comparison)

### Community Resources
- [Axum JWT Authentication Examples](https://codevoweb.com/jwt-authentication-in-rust-using-axum-framework/)
- [Connection Pooling Best Practices](https://users.rust-lang.org/t/connection-pool-for-http-reverse-proxy-server/86654)
- [MCP Transport Implementation Examples](https://github.com/modelcontextprotocol/servers)

---

**Conclusion:** Axum provides the optimal balance of performance, ecosystem maturity, and architectural alignment for Shadowcat's reverse proxy implementation. The 2025-06-18 MCP specification offers a clear path for HTTP transport implementation, and the available middleware ecosystem supports comprehensive authentication and authorization patterns. Implementation should proceed with confidence while maintaining proper performance monitoring and testing strategies.