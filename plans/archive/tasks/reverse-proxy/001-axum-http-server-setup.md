# Task 001: Axum HTTP Server Setup & MCP Transport Implementation

**Phase:** 5 (Reverse Proxy & Authentication)  
**Week:** 1 (Core Infrastructure)  
**Day:** 1  
**Priority:** Critical  
**Estimated Time:** 6-8 hours

## Overview

Implement the foundation HTTP server using Axum framework with MCP 2025-06-18 Streamable HTTP transport support. This task establishes the core HTTP infrastructure that all subsequent reverse proxy functionality will build upon.

## Success Criteria

- [x] Research validated Axum as optimal HTTP framework
- [ ] HTTP server starts and accepts connections on configured port
- [ ] MCP Streamable HTTP transport fully implemented per 2025-06-18 spec
- [ ] Session management with cryptographically secure session IDs
- [ ] HTTP to TransportMessage conversion working correctly
- [ ] Error handling with proper HTTP status code mapping
- [ ] Integration with existing Phase 4 session manager
- [ ] Basic health check endpoint functional
- [ ] All tests passing (unit + integration)

## Technical Specifications

### HTTP Server Architecture
```rust
// Core server structure based on research findings
pub struct ReverseProxyServer {
    pub bind_address: SocketAddr,
    pub router: Router,
    pub session_manager: Arc<SessionManager>,
    pub config: ReverseProxyConfig,
}

impl ReverseProxyServer {
    pub async fn start(&self) -> Result<(), ReverseProxyError> {
        let listener = TcpListener::bind(self.bind_address).await?;
        axum::serve(listener, self.router.clone()).await?;
        Ok(())
    }
}
```

### MCP Streamable HTTP Transport Implementation

**Required Headers:**
- `MCP-Session-Id`: Cryptographically secure session identifier
- `MCP-Protocol-Version`: Must be `2025-06-18` or compatible

**HTTP Methods:**
- `POST /mcp`: Primary JSON-RPC message endpoint
- `GET /health`: Health check endpoint
- `GET /metrics`: Metrics endpoint (Prometheus format)

**Request Processing Flow:**
```rust
async fn handle_mcp_request(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response<Body>, HttpError> {
    // 1. Extract MCP headers
    let session_id = extract_mcp_session_id(&headers)?;
    let protocol_version = extract_mcp_protocol_version(&headers)?;
    
    // 2. Parse JSON-RPC message
    let json_rpc: serde_json::Value = serde_json::from_slice(&body)?;
    let transport_message = TransportMessage::from_json_rpc(json_rpc)?;
    
    // 3. Create session context
    let session_context = SessionContext::new(session_id, protocol_version);
    
    // 4. Process through existing infrastructure
    let response = app_state.session_manager
        .handle_message(transport_message, session_context)
        .await?;
    
    // 5. Convert back to HTTP response
    Ok(transport_message_to_http_response(response))
}
```

### Session ID Generation
```rust
use ring::rand::{SystemRandom, SecureRandom};

pub fn generate_session_id() -> Result<SessionId, SessionError> {
    let rng = SystemRandom::new();
    let mut session_bytes = [0u8; 32];
    rng.fill(&mut session_bytes)?;
    
    let session_id = hex::encode(session_bytes);
    Ok(SessionId::new(session_id))
}
```

### Error Mapping
```rust
// HTTP status codes mapped to MCP protocol errors
pub fn mcp_error_to_http_status(error: &McpError) -> StatusCode {
    match error.code {
        -32700 => StatusCode::BAD_REQUEST,          // Parse error
        -32600 => StatusCode::BAD_REQUEST,          // Invalid Request
        -32601 => StatusCode::NOT_FOUND,            // Method not found
        -32602 => StatusCode::BAD_REQUEST,          // Invalid params
        -32603 => StatusCode::INTERNAL_SERVER_ERROR, // Internal error
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
```

## Implementation Steps

### Step 1: Dependencies and Project Structure
```toml
# Add to shadowcat/Cargo.toml
[dependencies]
axum = "0.8"
tower = "0.5" 
tower-http = { version = "0.6", features = ["cors", "trace"] }
hyper = { version = "1.0", features = ["full"] }
tokio = { version = "1.0", features = ["full"] } # Already present
serde = { version = "1.0", features = ["derive"] } # Already present
serde_json = "1.0" # Already present
hex = "0.4"
```

### Step 2: Core Module Structure
- `src/transport/http.rs`: HTTP transport implementation (extend existing)
- `src/proxy/reverse.rs`: Reverse proxy core (new file)
- `src/session/http_context.rs`: HTTP-specific session context
- `src/config/reverse_proxy.rs`: Reverse proxy configuration

### Step 3: HTTP Transport Extension
Extend existing HTTP transport to support MCP Streamable HTTP:
- Add MCP header parsing
- Implement session ID extraction and validation
- Add protocol version negotiation
- Integrate with existing TransportMessage system

### Step 4: Router Setup
```rust
pub fn create_reverse_proxy_router(
    session_manager: Arc<SessionManager>,
) -> Router {
    Router::new()
        .route("/mcp", post(handle_mcp_request))
        .route("/health", get(handle_health_check))
        .route("/metrics", get(handle_metrics))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()) // Configure as needed
        )
        .with_state(AppState { session_manager })
}
```

### Step 5: Integration Testing
- Test MCP session establishment
- Test message routing through existing session manager
- Test error handling and HTTP status mapping
- Test concurrent connection handling
- Validate protocol version negotiation

## Dependencies

### Blocked By
- None (foundational task)

### Blocks
- Task 002: OAuth 2.1 Flow Implementation
- Task 003: JWT Validation with JWKS Client
- Task 004: AuthGateway Core Implementation
- Task 005: Connection Pool Implementation

### Integrates With
- Existing Phase 4 SessionManager
- Existing Phase 4 TransportMessage system
- Existing Phase 4 error handling framework

## Testing Requirements

### Unit Tests
- [ ] MCP header parsing and validation
- [ ] Session ID generation and uniqueness
- [ ] TransportMessage conversion accuracy
- [ ] Error mapping completeness
- [ ] Protocol version negotiation

### Integration Tests
- [ ] End-to-end HTTP request processing
- [ ] Session lifecycle management
- [ ] Concurrent request handling
- [ ] Error scenarios and proper HTTP responses
- [ ] Health check endpoint functionality

### Performance Tests
- [ ] Baseline latency measurement (target: < 1ms HTTP overhead)
- [ ] Memory usage per connection (target: < 2KB baseline)
- [ ] Concurrent connection handling (target: 100+ connections)

## Configuration Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseProxyConfig {
    pub bind_address: SocketAddr,
    pub tls_config: Option<TlsConfig>,
    pub session_config: SessionConfig,
    pub cors_config: CorsConfig,
    pub trace_config: TraceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub session_timeout: Duration,
    pub max_sessions: usize,
    pub cleanup_interval: Duration,
}
```

## Documentation Requirements

- [ ] API documentation for all public functions
- [ ] Configuration example files
- [ ] MCP Streamable HTTP transport compliance documentation
- [ ] Error handling guide
- [ ] Performance characteristics documentation

## Risk Assessment

**Low Risk**: Axum is well-established framework with excellent documentation and Tower ecosystem integration.

**Mitigation Strategies**:
- Comprehensive unit test coverage
- Integration testing with existing Phase 4 components
- Performance benchmarking early in development
- Gradual rollout approach for testing

## Completion Checklist

- [ ] HTTP server accepts connections on configured port
- [ ] MCP headers correctly parsed and validated  
- [ ] Session management integrated with existing SessionManager
- [ ] TransportMessage conversion working bidirectionally
- [ ] Error handling with proper HTTP status codes
- [ ] Health check and metrics endpoints functional
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Performance tests meet baseline targets
- [ ] Code review completed
- [ ] Documentation updated

## Files Modified/Created

### New Files
- `src/transport/http_mcp.rs`: MCP Streamable HTTP implementation
- `src/proxy/reverse.rs`: Reverse proxy server core
- `src/config/reverse_proxy.rs`: Configuration structures
- `tests/integration/reverse_proxy_basic.rs`: Integration tests

### Modified Files
- `src/transport/mod.rs`: Export new HTTP MCP transport
- `src/main.rs`: Add reverse proxy CLI command
- `Cargo.toml`: Add Axum and related dependencies
- `src/lib.rs`: Export new reverse proxy modules

## Next Task
Upon completion, proceed to **Task 002: OAuth 2.1 Flow Implementation** which builds upon the HTTP server foundation established in this task.