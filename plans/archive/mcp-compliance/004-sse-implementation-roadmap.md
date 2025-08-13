# SSE Implementation Roadmap for MCP Compliance

## Executive Summary
Server-Sent Events (SSE) support is the most critical gap in Shadowcat's MCP specification compliance. This roadmap provides a detailed implementation plan to achieve full SSE support for the Streamable HTTP transport.

## Current State
- **Partial Implementation**: Basic SSE connection code exists in `transport/http.rs:148-216`
- **Missing Features**: No reconnection, no Last-Event-ID, no event types, no proper lifecycle
- **Protocol Version**: Using `2025-11-05` instead of spec version `2025-06-18`

## Implementation Phases

### Phase 1: Core SSE Infrastructure (2-3 days)

#### Task 1.1: Complete SSE Event Handling
**Priority**: CRITICAL
**Location**: `shadowcat/src/transport/http.rs`

```rust
// Add comprehensive SSE event support
enum SseEvent {
    Message { id: Option<String>, data: Value },
    Ping,
    Retry { delay_ms: u64 },
    Custom { event_type: String, data: Value },
}

struct SseConnection {
    event_source: EventSource,
    last_event_id: Option<String>,
    session_id: SessionId,
    reconnect_config: ReconnectConfig,
}
```

**Acceptance Criteria**:
- [ ] Handle all SSE event types (message, ping, retry, custom)
- [ ] Track last_event_id for each connection
- [ ] Support event ID in outgoing messages
- [ ] Parse SSE format correctly (data:, event:, id:, retry:)

#### Task 1.2: Implement SSE Reconnection Logic
**Priority**: CRITICAL
**Location**: `shadowcat/src/transport/http.rs`

```rust
struct ReconnectConfig {
    max_attempts: u32,
    base_delay_ms: u64,
    max_delay_ms: u64,
    jitter: bool,
}

impl SseConnection {
    async fn connect_with_retry(&mut self) -> Result<()> {
        let mut attempts = 0;
        loop {
            match self.establish_connection().await {
                Ok(_) => return Ok(()),
                Err(e) if attempts < self.reconnect_config.max_attempts => {
                    let delay = self.calculate_backoff(attempts);
                    tokio::time::sleep(delay).await;
                    attempts += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }
}
```

**Acceptance Criteria**:
- [ ] Automatic reconnection on connection loss
- [ ] Exponential backoff with jitter
- [ ] Send Last-Event-ID header on reconnection
- [ ] Resume from last received event

#### Task 1.3: Fix Header Handling
**Priority**: HIGH
**Location**: `shadowcat/src/transport/http_mcp.rs`

```rust
// Add proper header validation
const MCP_SESSION_ID_HEADER: &str = "Mcp-Session-Id";
const LAST_EVENT_ID_HEADER: &str = "Last-Event-ID";
const ORIGIN_HEADER: &str = "Origin";

fn validate_and_extract_headers(headers: &HeaderMap) -> Result<McpHeaders> {
    // Validate Origin for CORS
    // Extract and validate Mcp-Session-Id
    // Handle Last-Event-ID for SSE
}
```

**Acceptance Criteria**:
- [ ] Consistent header casing throughout codebase
- [ ] Origin header validation for security
- [ ] Proper Mcp-Session-Id handling
- [ ] Last-Event-ID extraction and storage

### Phase 2: Session Integration (1-2 days)

#### Task 2.1: HTTP Session Lifecycle
**Priority**: HIGH
**Location**: `shadowcat/src/session/manager.rs`

```rust
impl SessionManager {
    async fn create_http_session(&self, headers: HeaderMap) -> Result<Session> {
        // Extract or generate session ID
        // Initialize SSE connection state
        // Setup cleanup handlers
    }
    
    async fn cleanup_http_session(&self, session_id: SessionId) {
        // Close SSE connections
        // Cleanup pending requests
        // Notify connected clients
    }
}
```

**Acceptance Criteria**:
- [ ] Session creation from HTTP headers
- [ ] SSE connection tracking per session
- [ ] Proper cleanup on disconnection
- [ ] Session expiration handling

#### Task 2.2: Multiple SSE Stream Support
**Priority**: MEDIUM
**Location**: `shadowcat/src/transport/http.rs`

```rust
struct SseStreamManager {
    streams: HashMap<SessionId, SseConnection>,
    max_streams_per_session: usize,
}

impl SseStreamManager {
    async fn add_stream(&mut self, session_id: SessionId, connection: SseConnection) -> Result<()> {
        // Enforce stream limits
        // Track connection lifecycle
        // Handle stream multiplexing
    }
}
```

**Acceptance Criteria**:
- [ ] Support multiple concurrent SSE streams
- [ ] Stream limit enforcement
- [ ] Proper stream cleanup
- [ ] Stream-to-session mapping

### Phase 3: Protocol Compliance (1 day)

#### Task 3.1: Update Protocol Version
**Priority**: CRITICAL
**Location**: `shadowcat/src/transport/mod.rs:25`

```rust
// Update to match specification
pub const MCP_PROTOCOL_VERSION: &str = "2025-06-18";

// Support version negotiation
pub fn negotiate_version(client_version: &str) -> Result<String> {
    // Implement version compatibility logic
}
```

**Acceptance Criteria**:
- [ ] Update protocol version constant
- [ ] Implement version negotiation
- [ ] Test with spec-compliant clients
- [ ] Document version support matrix

#### Task 3.2: Unified HTTP Endpoint
**Priority**: MEDIUM
**Location**: `shadowcat/src/proxy/reverse.rs`

```rust
// Implement unified POST/GET endpoint as per spec
async fn handle_mcp_request(
    method: Method,
    headers: HeaderMap,
    body: Option<Bytes>,
) -> Result<Response> {
    match method {
        Method::POST => handle_post_request(headers, body).await,
        Method::GET => handle_sse_upgrade(headers).await,
        _ => Err(Error::MethodNotAllowed),
    }
}
```

**Acceptance Criteria**:
- [ ] Single endpoint handles both POST and GET
- [ ] POST for request/response
- [ ] GET upgrades to SSE
- [ ] Proper HTTP status codes (202, 404, 405)

### Phase 4: Testing and Validation (2 days)

#### Task 4.1: SSE Unit Tests
**Priority**: HIGH
**Location**: `shadowcat/src/transport/http.rs` (test module)

```rust
#[cfg(test)]
mod sse_tests {
    #[tokio::test]
    async fn test_sse_connection_establishment() { }
    
    #[tokio::test]
    async fn test_sse_reconnection_with_backoff() { }
    
    #[tokio::test]
    async fn test_last_event_id_handling() { }
    
    #[tokio::test]
    async fn test_multiple_sse_streams() { }
}
```

**Acceptance Criteria**:
- [ ] Test SSE connection lifecycle
- [ ] Test reconnection scenarios
- [ ] Test event ID tracking
- [ ] Test error handling

#### Task 4.2: Integration Tests
**Priority**: HIGH
**Location**: `shadowcat/tests/e2e_sse_compliance.rs`

```rust
// End-to-end SSE compliance tests
#[tokio::test]
async fn test_mcp_sse_full_flow() {
    // Test complete SSE flow with MCP messages
}

#[tokio::test]
async fn test_sse_with_network_interruption() {
    // Simulate network issues and verify recovery
}
```

**Acceptance Criteria**:
- [ ] Full MCP-over-SSE flow test
- [ ] Network resilience testing
- [ ] Performance benchmarks
- [ ] Spec compliance validation

#### Task 4.3: Conformance Testing
**Priority**: MEDIUM
**Location**: `shadowcat/tests/mcp_conformance.rs`

```rust
// Test against official MCP test suite
async fn run_mcp_conformance_suite() {
    // Validate against reference implementation
}
```

**Acceptance Criteria**:
- [ ] Pass MCP conformance tests
- [ ] Document any deviations
- [ ] Performance within 5% overhead target

## Performance Requirements

- **Latency**: < 5ms additional overhead for SSE
- **Memory**: < 10KB per SSE connection
- **Throughput**: > 1000 messages/second per stream
- **Reconnection**: < 1 second recovery time

## Security Considerations

1. **Origin Validation**: Prevent DNS rebinding attacks
2. **Session Hijacking**: Secure session ID generation and validation
3. **Resource Limits**: Prevent SSE connection exhaustion
4. **CSRF Protection**: Validate tokens on state-changing operations

## Dependencies

- `eventsource-client`: For SSE client implementation
- `futures-util`: For stream processing
- `tower-http`: For HTTP middleware
- Existing: `tokio`, `axum`, `serde_json`

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Breaking existing HTTP transport | Feature flag for SSE enablement |
| Performance regression | Comprehensive benchmarking suite |
| Protocol incompatibility | Test with reference MCP implementations |
| Memory leaks in long connections | Connection timeout and cleanup logic |

## Definition of Done

- [ ] All SSE events properly handled
- [ ] Automatic reconnection with Last-Event-ID
- [ ] Session integration complete
- [ ] Protocol version updated to 2025-06-18
- [ ] Origin validation implemented
- [ ] 100% test coverage for SSE code
- [ ] Performance benchmarks passing
- [ ] Documentation updated
- [ ] No clippy warnings
- [ ] Security review completed

## Timeline

- **Week 1**: Phase 1 (Core SSE) + Phase 2 (Session Integration)
- **Week 2**: Phase 3 (Protocol Compliance) + Phase 4 (Testing)
- **Buffer**: 2-3 days for issues and refinement

## Next Steps

1. Create feature branch: `feature/mcp-sse-compliance`
2. Implement Task 1.1 (SSE Event Handling)
3. Add unit tests incrementally
4. Weekly progress review

## Success Metrics

- SSE connections stable for > 24 hours
- Reconnection success rate > 99%
- Zero memory leaks over extended runs
- Full MCP 2025-06-18 compliance for HTTP transport