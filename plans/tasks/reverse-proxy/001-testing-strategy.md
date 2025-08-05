# Task 001: Comprehensive Testing Strategy

**Phase:** 5 (Reverse Proxy & Authentication)  
**Component:** Axum HTTP Server & MCP Transport  
**Created:** August 5, 2025

## Testing Philosophy

Task 001 follows a **Test-Driven Development (TDD)** approach with incremental validation at each step. Every component is tested in isolation (unit tests) and as part of the integrated system (integration tests).

## Test Categories

### 1. Unit Tests (40+ tests)

#### Transport Layer Tests (`src/transport/http_mcp.rs`)
```rust
#[cfg(test)]
mod tests {
    // Session ID Generation
    #[test]
    fn test_secure_session_id_uniqueness() {
        // Generate 1000 IDs and verify uniqueness
    }
    
    #[test]
    fn test_session_id_format() {
        // Verify UUID v4 format compliance
    }
    
    // Header Parsing
    #[test]
    fn test_mcp_header_extraction_success() {
        // Valid headers
    }
    
    #[test]
    fn test_mcp_header_missing_session_id() {
        // Error case
    }
    
    #[test]
    fn test_mcp_header_invalid_version() {
        // Version mismatch
    }
    
    #[test]
    fn test_protocol_version_compatibility() {
        // Test all supported versions
    }
    
    // Message Conversion
    #[test]
    fn test_transport_message_to_json_rpc() {
        // All message types
    }
    
    #[test]
    fn test_json_rpc_to_transport_message() {
        // All valid formats
    }
    
    #[test]
    fn test_malformed_json_rpc_handling() {
        // Error cases
    }
}
```

#### Error Handling Tests (`src/error.rs`)
```rust
#[cfg(test)]
mod reverse_proxy_error_tests {
    #[test]
    fn test_error_to_http_status_mapping() {
        assert_eq!(
            ReverseProxyError::InvalidHeaders("test".into()).to_http_status(),
            StatusCode::BAD_REQUEST
        );
        // Test all error variants
    }
    
    #[test]
    fn test_mcp_error_code_mapping() {
        assert_eq!(mcp_error_to_http_status(-32700), StatusCode::BAD_REQUEST);
        assert_eq!(mcp_error_to_http_status(-32601), StatusCode::NOT_FOUND);
        // Test all MCP error codes
    }
    
    #[test]
    fn test_error_display_messages() {
        // Verify user-friendly error messages
    }
}
```

#### Server Core Tests (`src/proxy/reverse.rs`)
```rust
#[cfg(test)]
mod tests {
    // Configuration
    #[test]
    fn test_config_defaults() {
        let config = ReverseProxyConfig::default();
        assert_eq!(config.session_config.session_timeout_secs, 300);
    }
    
    #[test]
    fn test_config_validation() {
        // Invalid configurations should fail
    }
    
    // Request Processing
    #[test]
    fn test_json_rpc_request_parsing() {
        // All request types
    }
    
    #[test]
    fn test_json_rpc_response_parsing() {
        // Success and error responses
    }
    
    #[test]
    fn test_json_rpc_notification_parsing() {
        // Notifications (no ID)
    }
    
    // Session Integration
    #[tokio::test]
    async fn test_session_creation_on_new_request() {
        // Mock session manager
    }
    
    #[tokio::test]
    async fn test_session_reuse_on_existing() {
        // Mock session manager
    }
}
```

#### Configuration Tests (`src/config/reverse_proxy.rs`)
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_config_from_yaml() {
        let yaml = r#"
server:
  bind_address: "0.0.0.0:9090"
  max_connections: 500
session:
  timeout_secs: 600
"#;
        let config: ReverseProxySettings = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.server.max_connections, 500);
    }
    
    #[test]
    fn test_config_environment_override() {
        // Test env var overrides
    }
    
    #[test]
    fn test_config_validation_rules() {
        // Test invalid configs fail appropriately
    }
}
```

### 2. Integration Tests (10+ scenarios)

#### Basic Server Tests (`tests/integration/reverse_proxy_basic.rs`)
```rust
#[tokio::test]
async fn test_server_lifecycle() {
    // Start, verify running, shutdown cleanly
}

#[tokio::test]
async fn test_all_endpoints_accessible() {
    // GET /health, GET /metrics, POST /mcp
}

#[tokio::test]
async fn test_cors_headers_present() {
    // Verify CORS middleware working
}

#[tokio::test]
async fn test_trace_layer_headers() {
    // Verify tracing headers added
}
```

#### MCP Protocol Tests
```rust
#[tokio::test]
async fn test_mcp_initialize_request() {
    // Full initialize handshake
}

#[tokio::test]
async fn test_mcp_method_invocation() {
    // Standard method call
}

#[tokio::test]
async fn test_mcp_notification_handling() {
    // One-way notifications
}

#[tokio::test]
async fn test_mcp_error_response() {
    // Server returns error
}
```

#### Session Management Tests
```rust
#[tokio::test]
async fn test_session_persistence_across_requests() {
    // Multiple requests, same session
}

#[tokio::test]
async fn test_concurrent_sessions() {
    // 100+ concurrent sessions
}

#[tokio::test]
async fn test_session_timeout_handling() {
    // Session expires after timeout
}
```

#### Error Scenario Tests
```rust
#[tokio::test]
async fn test_malformed_json_body() {
    // Invalid JSON in request
}

#[tokio::test]
async fn test_missing_required_headers() {
    // No MCP-Session-Id
}

#[tokio::test]
async fn test_unsupported_protocol_version() {
    // Old/future version
}

#[tokio::test]
async fn test_request_size_limit() {
    // > 1MB request body
}
```

### 3. Performance Tests

#### Benchmark Suite (`benches/reverse_proxy_bench.rs`)
```rust
// Latency Benchmarks
fn bench_request_response_cycle(c: &mut Criterion) {
    // Measure full HTTP request/response time
    // Target: < 1ms overhead
}

fn bench_session_lookup_time(c: &mut Criterion) {
    // Time to find existing session
    // Target: < 100μs
}

fn bench_header_parsing_time(c: &mut Criterion) {
    // MCP header extraction
    // Target: < 50μs
}

// Throughput Benchmarks
fn bench_requests_per_second(c: &mut Criterion) {
    // Maximum RPS on single core
    // Target: > 10,000 RPS
}

// Memory Benchmarks
fn bench_memory_per_connection(c: &mut Criterion) {
    // Memory usage per active connection
    // Target: < 2KB baseline
}

fn bench_session_memory_overhead(c: &mut Criterion) {
    // Memory per session in manager
    // Target: < 1KB per session
}
```

#### Load Tests
```rust
#[tokio::test]
async fn load_test_concurrent_connections() {
    // Spawn 1000 concurrent connections
    // Verify all succeed
    // Measure resource usage
}

#[tokio::test]
async fn load_test_sustained_traffic() {
    // 100 RPS for 60 seconds
    // Monitor latency degradation
    // Check memory leaks
}
```

### 4. Property-Based Tests

Using `proptest` for exhaustive testing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_any_valid_json_rpc_parses(
        id in prop::option::of(any::<String>()),
        method in "[a-zA-Z][a-zA-Z0-9_]*",
        params in any::<serde_json::Value>()
    ) {
        let json = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });
        
        let result = parse_json_rpc_to_transport(&json);
        prop_assert!(result.is_ok());
    }
    
    #[test]
    fn test_round_trip_conversion(msg in arb_transport_message()) {
        let json = transport_to_json_rpc(&msg).unwrap();
        let parsed = parse_json_rpc_to_transport(&json).unwrap();
        prop_assert_eq!(msg, parsed);
    }
}
```

## Testing Infrastructure

### Mock Utilities
```rust
// Mock session manager for testing
pub struct MockSessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    fail_on_create: bool,
}

impl MockSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            fail_on_create: false,
        }
    }
    
    pub fn with_failure(mut self) -> Self {
        self.fail_on_create = true;
        self
    }
}

// Mock HTTP client for integration tests
pub struct TestClient {
    base_url: String,
    default_headers: HeaderMap,
}

impl TestClient {
    pub async fn mcp_request(&self, body: Value) -> Response {
        // Helper for standard MCP requests
    }
}
```

### Test Fixtures
```rust
// Common test data
pub mod fixtures {
    pub fn valid_mcp_request() -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": "test-1",
            "method": "initialize",
            "params": {
                "capabilities": {}
            }
        })
    }
    
    pub fn valid_mcp_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("mcp-session-id", "test-session".parse().unwrap());
        headers.insert("mcp-protocol-version", "2025-11-05".parse().unwrap());
        headers
    }
}
```

## Coverage Requirements

### Minimum Coverage Targets
- Overall: 80%
- Core modules: 90%
  - `http_mcp.rs`: 90%
  - `reverse.rs`: 90%
  - Error handling: 95%
- Integration tests: 100% endpoint coverage

### Coverage Commands
```bash
# Install coverage tools
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage

# Check coverage meets targets
cargo tarpaulin --print-summary --fail-under 80
```

## Continuous Testing

### Pre-commit Hooks
```bash
#!/bin/bash
# .git/hooks/pre-commit

# Run format check
cargo fmt -- --check || exit 1

# Run clippy
cargo clippy -- -D warnings || exit 1

# Run tests
cargo test || exit 1

# Check coverage on changed files
cargo tarpaulin --print-summary --fail-under 80
```

### CI Pipeline
```yaml
# .github/workflows/shadowcat-tests.yml
name: Shadowcat Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      
      - name: Run tests
        run: |
          cd shadowcat
          cargo test --all-features
          
      - name: Run benchmarks (smoke test)
        run: |
          cd shadowcat
          cargo bench --no-run
          
      - name: Check coverage
        run: |
          cd shadowcat
          cargo tarpaulin --fail-under 80
```

## Test Execution Strategy

### Development Workflow
1. **Write test first** (TDD)
2. **Run focused test**: `cargo test test_name`
3. **Run module tests**: `cargo test module_name::`
4. **Run all tests**: `cargo test`
5. **Check coverage**: `cargo tarpaulin`

### Test Organization
```
shadowcat/
├── src/
│   ├── transport/
│   │   └── http_mcp.rs      # Unit tests in same file
│   ├── proxy/
│   │   └── reverse.rs        # Unit tests in same file
│   └── error.rs              # Unit tests in same file
├── tests/
│   ├── integration/
│   │   ├── reverse_proxy_basic.rs
│   │   ├── mcp_protocol.rs
│   │   └── performance.rs
│   └── common/
│       ├── mod.rs           # Shared test utilities
│       └── fixtures.rs      # Test data
└── benches/
    └── reverse_proxy_bench.rs
```

## Debugging Failed Tests

### Diagnostic Tools
```bash
# Run with test output
cargo test -- --nocapture

# Run specific test with backtrace
RUST_BACKTRACE=1 cargo test test_name

# Run with debug logging
RUST_LOG=shadowcat=debug cargo test

# Run single test in isolation
cargo test test_name -- --test-threads=1
```

### Common Test Failures

1. **Port Binding Issues**
   - Use dynamic port allocation in tests
   - Clean up servers properly

2. **Async Test Timeouts**
   - Use `tokio::time::timeout` wrapper
   - Set reasonable timeouts (5s default)

3. **Session State Pollution**
   - Create fresh session manager per test
   - Use unique session IDs

## Performance Testing Guidelines

### Baseline Establishment
1. Run benchmarks on clean machine state
2. Record baseline metrics
3. Set up performance regression alerts

### Performance Test Environment
- Dedicated hardware or VM
- Consistent CPU governor settings
- Minimal background processes
- Multiple runs for statistical significance

### Metrics to Track
- **Latency**: p50, p95, p99, p99.9
- **Throughput**: requests/second
- **Memory**: RSS, heap allocations
- **CPU**: usage percentage, context switches

## Security Testing

### Input Validation Tests
- Oversized headers
- Malformed UTF-8
- Injection attempts
- Resource exhaustion

### Session Security Tests
- Session ID entropy verification
- Session fixation prevention
- Concurrent session limits

This comprehensive testing strategy ensures Task 001 delivers a robust, performant, and secure foundation for the Shadowcat Phase 5 reverse proxy implementation.