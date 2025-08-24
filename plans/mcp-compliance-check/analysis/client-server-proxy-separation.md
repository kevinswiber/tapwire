# Client-Server-Proxy Test Separation

## Critical Insight

**Shadowcat is both an MCP client AND an MCP server:**
- **MCP Server**: Accepts connections from downstream clients
- **MCP Client**: Connects to upstream MCP servers
- **Proxy**: Bridges the two roles with additional behaviors

This three-way distinction fundamentally improves our testing architecture.

## The Three Test Categories

### 1. Client Compliance Tests

Tests whether an MCP client implementation correctly:

```rust
pub async fn test_client_compliance(
    client: &dyn McpClient,
    test_server: &TestServer,
) -> ClientComplianceReport {
    // Test client behaviors
}
```

#### Client Requirements to Test

**Initialization**
- Client MUST send initialize request first
- Client MUST send initialized notification after response
- Client MUST include protocol version
- Client SHOULD handle version negotiation

**Request Formation**
- Client MUST use correct JSON-RPC format
- Client MUST use unique request IDs
- Client MUST NOT send batch with initialize
- Client SHOULD set appropriate timeouts

**Response Handling**
- Client MUST match response IDs to requests
- Client MUST handle error responses
- Client SHOULD handle partial responses
- Client MAY handle progress notifications

**Session Management**
- Client MUST include session ID in subsequent requests (HTTP)
- Client SHOULD handle session expiration
- Client SHOULD reconnect on disconnect (SSE)

**Error Recovery**
- Client SHOULD retry on network failures
- Client SHOULD handle rate limiting
- Client MUST handle protocol errors gracefully

### 2. Server Compliance Tests

Tests whether an MCP server implementation correctly:

```rust
pub async fn test_server_compliance(
    server: &TestableServer,
    test_client: &TestClient,
) -> ServerComplianceReport {
    // Test server behaviors
}
```

#### Server Requirements to Test

**Initialization**
- Server MUST respond to initialize with capabilities
- Server MUST accept initialized notification
- Server MUST negotiate protocol version
- Server SHOULD NOT accept requests before initialized

**Request Processing**
- Server MUST handle JSON-RPC format
- Server MUST return same ID in response
- Server MUST return result XOR error
- Server SHOULD validate parameters

**Capability Implementation**
- Server MUST implement advertised capabilities
- Server MUST NOT claim unsupported features
- Server SHOULD handle missing optional features

**Error Handling**
- Server MUST use correct error codes
- Server MUST include error message
- Server MAY include error data
- Server SHOULD NOT expose internal details

### 3. Proxy Compliance Tests

Tests proxy-specific behaviors beyond client/server roles:

```rust
pub async fn test_proxy_compliance(
    proxy: &ShadowcatProxy,
    mock_upstream: &MockServer,
    test_client: &TestClient,
) -> ProxyComplianceReport {
    // Test proxy-specific behaviors
}
```

#### Proxy Requirements to Test

**Message Forwarding**
- Proxy MUST NOT modify message content
- Proxy MUST preserve request IDs
- Proxy MUST maintain message ordering
- Proxy SHOULD minimize latency

**Session Management**
- Proxy MUST track dual sessions
- Proxy MUST map client↔upstream sessions
- Proxy MUST clean up on disconnect
- Proxy SHOULD handle session timeout

**Error Propagation**
- Proxy MUST forward upstream errors
- Proxy MUST indicate proxy-generated errors
- Proxy SHOULD NOT leak sensitive error details

**Security Boundary**
- Proxy MUST NOT forward client auth tokens
- Proxy MUST use its own upstream auth
- Proxy SHOULD validate Origin headers

## How This Separation Helps

### 1. Cleaner Test Architecture

```rust
// Before: Mixed concerns
#[test]
async fn test_proxy_initialize() {
    // Tests client sending, proxy forwarding, server responding
    // Hard to identify which component failed
}

// After: Separated concerns
#[test]
async fn test_client_sends_initialize() {
    // Only tests client behavior
}

#[test]
async fn test_server_handles_initialize() {
    // Only tests server behavior
}

#[test]
async fn test_proxy_forwards_initialize() {
    // Only tests forwarding behavior
}
```

### 2. Reusable Test Components

```rust
pub struct ComplianceChecker {
    client_tests: ClientTestSuite,
    server_tests: ServerTestSuite,
    proxy_tests: ProxyTestSuite,
}

impl ComplianceChecker {
    /// Test any MCP client implementation
    pub async fn test_client(&self, client: &dyn McpClient) -> Report {
        self.client_tests.run(client).await
    }
    
    /// Test any MCP server implementation
    pub async fn test_server(&self, server: &dyn McpServer) -> Report {
        self.server_tests.run(server).await
    }
    
    /// Test Shadowcat as both client and server
    pub async fn test_shadowcat(&self, shadowcat: &Shadowcat) -> Report {
        let mut report = Report::new();
        
        // Test Shadowcat's client behavior (upstream connection)
        report.client = self.test_client(&shadowcat.as_client()).await;
        
        // Test Shadowcat's server behavior (downstream API)
        report.server = self.test_server(&shadowcat.as_server()).await;
        
        // Test proxy-specific behaviors
        report.proxy = self.proxy_tests.run(shadowcat).await;
        
        report
    }
}
```

### 3. Better Diagnostics

When a proxy test fails, we can identify exactly where:

```rust
// Test output example
Shadowcat Compliance Report:
  Client Compliance (upstream): 98% ✅
    ✅ Sends correct initialize
    ✅ Handles version negotiation
    ❌ Missing timeout on requests
    
  Server Compliance (downstream): 95% ✅
    ✅ Responds to initialize
    ✅ Advertises capabilities
    ❌ Wrong error code for invalid method
    
  Proxy Compliance: 100% ✅
    ✅ Forwards messages unchanged
    ✅ Maps sessions correctly
    ✅ Doesn't leak auth tokens
```

## Implementation Strategy

### 1. Create Test Traits

```rust
/// Trait for testable MCP clients
pub trait TestableClient {
    async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse>;
    async fn connect(&mut self, server_url: &str) -> Result<()>;
    fn get_session_id(&self) -> Option<String>;
}

/// Trait for testable MCP servers  
pub trait TestableServer {
    async fn start(&mut self) -> Result<String>; // Returns URL
    async fn shutdown(&mut self) -> Result<()>;
    fn get_capabilities(&self) -> ServerCapabilities;
}

/// Shadowcat implements both traits
impl TestableClient for Shadowcat {
    // Test Shadowcat's upstream client behavior
}

impl TestableServer for Shadowcat {
    // Test Shadowcat's downstream server behavior
}
```

### 2. Adapt Shadowcat for Testing

```rust
// In Shadowcat
impl Shadowcat {
    /// Expose client behavior for testing
    pub fn as_client(&self) -> impl TestableClient {
        ShadowcatClientAdapter { 
            upstream_handler: &self.upstream_handler 
        }
    }
    
    /// Expose server behavior for testing
    pub fn as_server(&self) -> impl TestableServer {
        ShadowcatServerAdapter {
            downstream_handler: &self.downstream_handler
        }
    }
}
```

### 3. Test Organization

```
shadowcat-compliance/
├── src/
│   ├── client/              # Client compliance tests
│   │   ├── mod.rs
│   │   ├── initialization.rs
│   │   ├── requests.rs
│   │   ├── responses.rs
│   │   └── errors.rs
│   ├── server/              # Server compliance tests
│   │   ├── mod.rs
│   │   ├── initialization.rs
│   │   ├── capabilities.rs
│   │   ├── methods.rs
│   │   └── errors.rs
│   └── proxy/               # Proxy-specific tests
│       ├── mod.rs
│       ├── forwarding.rs
│       ├── sessions.rs
│       ├── security.rs
│       └── performance.rs
```

## Benefits for Shadowcat Testing

### 1. Test in Isolation

```rust
#[test]
async fn test_shadowcat_client_behavior() {
    // Test against mock server - no real upstream needed
    let mock_server = MockMcpServer::new();
    let shadowcat = Shadowcat::new();
    
    let report = test_client_compliance(
        &shadowcat.as_client(),
        &mock_server
    ).await;
    
    assert!(report.is_compliant());
}
```

### 2. Test Both Directions

```rust
#[test]
async fn test_shadowcat_bidirectional() {
    let shadowcat = Shadowcat::new();
    
    // Test downstream (server) behavior
    let server_report = test_server_compliance(&shadowcat.as_server()).await;
    
    // Test upstream (client) behavior  
    let client_report = test_client_compliance(&shadowcat.as_client()).await;
    
    // Both should be compliant
    assert!(server_report.is_compliant());
    assert!(client_report.is_compliant());
}
```

### 3. Identify Issues Precisely

```rust
// Instead of: "Proxy test failed"
// We get: "Client-side timeout handling failed when connecting upstream"

#[test]
async fn diagnose_proxy_issue() {
    let result = test_proxy_initialize().await;
    if result.is_err() {
        // Run targeted tests to isolate issue
        if !test_client_sends_initialize().await.is_ok() {
            println!("Issue: Client-side initialize is malformed");
        } else if !test_server_accepts_initialize().await.is_ok() {
            println!("Issue: Server-side initialize handling");
        } else {
            println!("Issue: Proxy-specific forwarding logic");
        }
    }
}
```

## Updated Test Count Estimates

With client/server/proxy separation:

| Category | Tests | Description |
|----------|-------|-------------|
| **Client Tests** | 60 | Client-side compliance |
| **Server Tests** | 60 | Server-side compliance |
| **Proxy Tests** | 50 | Proxy-specific behaviors |
| **Protocol Tests** | 80 | Version-specific, shared by client/server |
| **Total** | 250 | Comprehensive coverage |

## Updated Library API

```rust
pub struct ComplianceChecker {
    // Test suites
    client_suite: ClientTestSuite,
    server_suite: ServerTestSuite,
    proxy_suite: ProxyTestSuite,
}

impl ComplianceChecker {
    /// Test an MCP client implementation
    pub async fn test_client(
        &self,
        client: &dyn TestableClient,
        options: ClientTestOptions,
    ) -> Result<ClientComplianceReport> {
        self.client_suite.run(client, options).await
    }
    
    /// Test an MCP server implementation
    pub async fn test_server(
        &self,
        server: &dyn TestableServer,
        options: ServerTestOptions,
    ) -> Result<ServerComplianceReport> {
        self.server_suite.run(server, options).await
    }
    
    /// Test an MCP proxy (tests all three aspects)
    pub async fn test_proxy(
        &self,
        proxy: &dyn TestableProxy,
        options: ProxyTestOptions,
    ) -> Result<ProxyComplianceReport> {
        let mut report = ProxyComplianceReport::new();
        
        // Test client-side compliance
        report.client_compliance = self.test_client(
            &proxy.as_client(),
            options.client_options
        ).await?;
        
        // Test server-side compliance
        report.server_compliance = self.test_server(
            &proxy.as_server(),
            options.server_options
        ).await?;
        
        // Test proxy-specific behaviors
        report.proxy_compliance = self.proxy_suite.run(
            proxy,
            options.proxy_options
        ).await?;
        
        Ok(report)
    }
}
```

## CLI Updates

```bash
# Test a client implementation
mcp-compliance client <client-command> -v 2025-06-18

# Test a server implementation  
mcp-compliance server <server-url> -v 2025-06-18

# Test a proxy (all three aspects)
mcp-compliance proxy <proxy-url> <upstream-url> -v 2025-06-18

# Test specific aspect of proxy
mcp-compliance proxy <proxy-url> --only-client   # Just client-side
mcp-compliance proxy <proxy-url> --only-server   # Just server-side
mcp-compliance proxy <proxy-url> --only-proxy    # Just proxy behaviors
```

## Conclusion

This three-way separation (client/server/proxy) provides:

1. **Better coverage** - Tests both sides of proxy behavior
2. **Cleaner architecture** - Separated concerns, reusable components
3. **Precise diagnostics** - Identify exactly which aspect failed
4. **Flexibility** - Test components in isolation
5. **Reusability** - Same tests for any MCP client/server/proxy

This is a significant architectural improvement that makes our compliance framework more powerful and maintainable.

---

*Created: 2025-08-24*
*Insight: Proxy = Client + Server + Proxy-specific behaviors*
*Impact: Cleaner, more reusable test architecture*