# Proxy-Specific Test Scenarios

## Overview

While the MCP specification doesn't mention proxies, a compliant MCP proxy must maintain **transparency** while adding value through features like authentication, routing, and monitoring. These tests verify proxy-specific behaviors beyond basic MCP compliance.

## Core Proxy Principles

1. **Message Transparency** - Don't modify MCP message content
2. **Session Isolation** - Keep client sessions separate
3. **Error Fidelity** - Propagate errors accurately
4. **Performance Overhead** - Minimize latency impact
5. **Security Boundary** - Don't leak authentication

## Test Categories

### 1. Message Forwarding Integrity

#### Test: Exact Message Forwarding
```rust
#[test]
async fn test_message_content_unchanged() {
    // Send request through proxy
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });
    
    // Capture what proxy sends upstream
    let forwarded = capture_upstream_request();
    
    // Verify exact match (except headers)
    assert_eq!(request, forwarded.body);
}
```

#### Test: ID Preservation
```rust
#[test]
async fn test_request_id_preserved() {
    // Test string IDs, numeric IDs, large IDs
    for id in ["abc", "123", "999999999"] {
        let response = send_through_proxy(id);
        assert_eq!(response.id, id);
    }
}
```

#### Test: Notification Handling
```rust
#[test]
async fn test_notification_forwarding() {
    // Notifications have no ID
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "notifications/progress",
        "params": {"token": "abc", "progress": 50}
    });
    
    // Should forward without expecting response
    send_notification_through_proxy(notification);
    
    // Verify proxy doesn't wait for response
    assert_no_response_expected();
}
```

### 2. Session Management

#### Test: Dual Session ID Tracking
```rust
#[test]
async fn test_dual_session_mapping() {
    // Client creates session with proxy
    let client_session = connect_to_proxy();
    assert!(client_session.id.is_some());
    
    // Proxy creates session with upstream
    let upstream_session = get_upstream_session();
    assert!(upstream_session.id.is_some());
    
    // Verify mapping maintained
    assert_ne!(client_session.id, upstream_session.id);
    
    // Verify correct routing
    let response = send_request(client_session.id);
    assert_eq!(response.session_id, client_session.id);
}
```

#### Test: Session Isolation
```rust
#[test]
async fn test_session_isolation() {
    // Create two client sessions
    let session1 = connect_to_proxy();
    let session2 = connect_to_proxy();
    
    // Initialize differently
    init_session(session1, "client1");
    init_session(session2, "client2");
    
    // Verify no cross-contamination
    let tools1 = get_tools(session1);
    let tools2 = get_tools(session2);
    
    // Each session should have its own state
    assert_independent(tools1, tools2);
}
```

#### Test: Session Cleanup
```rust
#[test]
async fn test_session_cleanup_on_disconnect() {
    let session = connect_to_proxy();
    let upstream = get_upstream_session();
    
    // Disconnect client
    disconnect(session);
    
    // Verify proxy cleans up
    assert!(proxy_session_gone(session.id));
    assert!(upstream_session_closed(upstream.id));
    
    // Verify no resource leaks
    assert_eq!(get_active_sessions(), initial_count);
}
```

### 3. Error Propagation

#### Test: Error Response Forwarding
```rust
#[test]
async fn test_error_propagation() {
    // Trigger various upstream errors
    let test_cases = vec![
        ("unknown_method", -32601, "Method not found"),
        ("invalid_params", -32602, "Invalid params"),
        ("internal_error", -32603, "Internal error"),
    ];
    
    for (method, expected_code, _) in test_cases {
        let error = call_through_proxy(method).unwrap_err();
        assert_eq!(error.code, expected_code);
        // Verify error message preserved
        assert!(error.message.contains(expected_msg_part));
    }
}
```

#### Test: Proxy-Generated Errors
```rust
#[test]
async fn test_proxy_error_attribution() {
    // Disconnect upstream to trigger proxy error
    kill_upstream_server();
    
    let error = call_through_proxy("tools/list").unwrap_err();
    
    // Proxy should indicate it generated the error
    assert_eq!(error.code, -32000); // Server error range
    assert!(error.data.contains("proxy"));
    assert!(error.data.contains("upstream unavailable"));
}
```

### 4. Authentication Handling

#### Test: Token Non-Forwarding
```rust
#[test]
async fn test_client_token_not_forwarded() {
    // Client sends auth token
    let client_headers = HashMap::from([
        ("Authorization", "Bearer client-secret-token"),
    ]);
    
    // Capture upstream request
    let upstream_req = capture_upstream_request();
    
    // Verify client token NOT forwarded
    assert!(!upstream_req.headers.contains_key("Authorization"));
    
    // Verify proxy uses its own auth
    assert_eq!(
        upstream_req.headers.get("Authorization"),
        Some("Bearer proxy-upstream-token")
    );
}
```

#### Test: Auth Error Handling
```rust
#[test]
async fn test_auth_error_handling() {
    // Test unauthorized client
    let response = connect_without_auth();
    assert_eq!(response.status, 401);
    
    // Test upstream auth failure
    configure_bad_upstream_token();
    let error = call_through_proxy("tools/list").unwrap_err();
    
    // Should indicate auth issue without leaking details
    assert!(error.message.contains("authentication"));
    assert!(!error.message.contains("token"));
}
```

### 5. Transport-Specific Behaviors

#### Test: SSE Reconnection
```rust
#[test]
async fn test_sse_auto_reconnect() {
    // Establish SSE connection through proxy
    let sse = connect_sse_through_proxy();
    
    // Simulate upstream disconnect
    kill_upstream_sse();
    
    // Send request while disconnected
    let request_id = send_request(sse);
    
    // Wait for reconnection
    wait_for_upstream_reconnect();
    
    // Verify request completed after reconnect
    let response = await_response(request_id);
    assert!(response.is_ok());
}
```

#### Test: Message Buffering
```rust
#[test]
async fn test_message_buffering_during_reconnect() {
    let sse = connect_sse_through_proxy();
    
    // Send multiple requests
    let ids = (0..5).map(|_| send_request_async(sse)).collect();
    
    // Disconnect upstream mid-flight
    kill_upstream_sse();
    
    // Reconnect
    start_upstream_sse();
    
    // All requests should complete
    for id in ids {
        assert!(await_response(id).is_ok());
    }
}
```

### 6. Performance and Scalability

#### Test: Connection Pooling
```rust
#[test]
async fn test_connection_pooling() {
    // Create multiple client connections
    let clients = (0..10).map(|_| connect_to_proxy()).collect();
    
    // Verify proxy uses fewer upstream connections
    let upstream_conns = count_upstream_connections();
    assert!(upstream_conns < clients.len());
    
    // Verify proper request routing despite pooling
    for client in clients {
        let response = call_through_proxy_with_session(client);
        assert_eq!(response.session_id, client.id);
    }
}
```

#### Test: Latency Overhead
```rust
#[test]
async fn test_proxy_latency_overhead() {
    // Measure direct connection
    let direct_latency = measure_direct_latency();
    
    // Measure through proxy
    let proxy_latency = measure_proxy_latency();
    
    // Calculate overhead
    let overhead = (proxy_latency - direct_latency) / direct_latency;
    
    // Should be < 5% for localhost
    assert!(overhead < 0.05, "Proxy overhead {}% > 5%", overhead * 100.0);
}
```

### 7. Failover and Resilience

#### Test: Upstream Failover
```rust
#[test]
async fn test_upstream_failover() {
    // Configure primary and backup upstreams
    configure_multi_upstream(&["primary:3000", "backup:3001"]);
    
    // Verify using primary
    assert_eq!(get_active_upstream(), "primary:3000");
    
    // Kill primary
    kill_server("primary:3000");
    
    // Next request should use backup
    let response = call_through_proxy("ping");
    assert!(response.is_ok());
    assert_eq!(get_active_upstream(), "backup:3001");
    
    // Client session should be preserved
    assert_eq!(get_client_session_id(), original_session_id);
}
```

#### Test: Circuit Breaker
```rust
#[test]
async fn test_circuit_breaker() {
    // Trigger repeated failures
    for _ in 0..5 {
        make_upstream_fail();
        let _ = call_through_proxy("test");
    }
    
    // Circuit should open
    assert_eq!(get_circuit_state(), CircuitState::Open);
    
    // Requests should fail fast
    let start = Instant::now();
    let error = call_through_proxy("test").unwrap_err();
    assert!(start.elapsed() < Duration::from_millis(100));
    assert!(error.message.contains("circuit open"));
    
    // Wait for half-open
    sleep(Duration::from_secs(30));
    assert_eq!(get_circuit_state(), CircuitState::HalfOpen);
    
    // Successful request should close circuit
    make_upstream_healthy();
    call_through_proxy("test").unwrap();
    assert_eq!(get_circuit_state(), CircuitState::Closed);
}
```

### 8. Proxy Transparency Validation

#### Test: Header Forwarding Rules
```rust
#[test]
async fn test_header_forwarding() {
    let headers = HashMap::from([
        ("Accept", "application/json"),              // Should forward
        ("Content-Type", "application/json"),        // Should forward
        ("Authorization", "Bearer token"),           // Should NOT forward
        ("X-Custom-Header", "value"),               // Should forward
        ("Connection", "keep-alive"),               // Should NOT forward
        ("Host", "client.example.com"),             // Should be replaced
    ]);
    
    let upstream_headers = send_with_headers(headers);
    
    assert_eq!(upstream_headers.get("Accept"), Some("application/json"));
    assert_eq!(upstream_headers.get("Content-Type"), Some("application/json"));
    assert!(!upstream_headers.contains_key("Authorization"));
    assert_eq!(upstream_headers.get("X-Custom-Header"), Some("value"));
    assert!(!upstream_headers.contains_key("Connection"));
    assert_eq!(upstream_headers.get("Host"), Some("upstream.example.com"));
}
```

#### Test: Protocol Version Transparency
```rust
#[test]
async fn test_protocol_version_negotiation_passthrough() {
    // Client requests specific version
    let client_init = json!({
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26"
        }
    });
    
    // Proxy should forward exact version
    let upstream_init = capture_upstream_request();
    assert_eq!(
        upstream_init.params.protocolVersion,
        "2025-03-26"
    );
    
    // Upstream negotiates different version
    let upstream_response = json!({
        "result": {
            "protocolVersion": "2025-06-18"
        }
    });
    
    // Proxy should forward negotiated version
    let client_response = get_client_response();
    assert_eq!(
        client_response.result.protocolVersion,
        "2025-06-18"
    );
}
```

## Proxy Compliance Levels

### Level 1: Basic Proxy (Minimum)
- Message forwarding works
- Sessions are isolated
- Errors are propagated
- No message modification

### Level 2: Production Proxy (Recommended)
- All Level 1 requirements
- Connection pooling
- Session cleanup
- Auth token handling
- Performance < 5% overhead

### Level 3: Enterprise Proxy (Advanced)
- All Level 2 requirements
- Failover support
- Circuit breaker
- SSE reconnection
- Message buffering
- Rate limiting
- Monitoring/metrics

## Testing Strategy

### 1. Isolated Tests
Test each proxy behavior independently:
- Use mock upstream servers
- Control failure scenarios
- Verify specific behaviors

### 2. Integration Tests
Test full proxy flow:
- Real MCP servers
- Complete session lifecycle
- End-to-end scenarios

### 3. Chaos Testing
Test resilience:
- Random disconnections
- Upstream failures
- Network delays
- Resource exhaustion

### 4. Performance Testing
Measure overhead:
- Latency impact
- Throughput limits
- Memory usage
- Connection limits

## Implementation Notes

### Test Utilities Needed

```rust
// Test harness for proxy testing
pub struct ProxyTestHarness {
    proxy: ProxyInstance,
    upstream: MockUpstream,
    client: TestClient,
}

impl ProxyTestHarness {
    pub fn capture_upstream_request(&mut self) -> Request {
        self.upstream.last_request()
    }
    
    pub fn kill_upstream(&mut self) {
        self.upstream.shutdown()
    }
    
    pub fn measure_latency(&self) -> Duration {
        // Measure round-trip time
    }
}
```

### Mock Upstream Server

```rust
pub struct MockUpstream {
    responses: HashMap<String, Response>,
    requests: Vec<Request>,
    failure_mode: Option<FailureMode>,
}

pub enum FailureMode {
    Disconnect,
    Timeout,
    ErrorResponse(i32),
    SlowResponse(Duration),
}
```

## Summary

These proxy-specific tests ensure:

1. **Transparency** - Proxy doesn't break MCP protocol
2. **Isolation** - Client sessions remain independent
3. **Security** - Authentication boundaries maintained
4. **Resilience** - Handles failures gracefully
5. **Performance** - Minimal overhead
6. **Value-Add** - Proxy features work correctly

Total proxy-specific tests needed: **~40-50 tests**

---

*Created: 2025-08-23*
*Purpose: Define proxy-specific test scenarios not in MCP spec*
*Target: MCP proxy implementations like Shadowcat*