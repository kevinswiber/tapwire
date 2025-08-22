# Test Organization and Naming Conventions

## Overview
Consistent test organization and naming is critical for maintainability, discoverability, and CI/CD integration. This document defines the structure and conventions for E2E tests in the Shadowcat project.

## Directory Structure

```
shadowcat/
├── src/                         # Source code
├── tests/                       # All test code
│   ├── unit/                   # Unit tests (if separated from src/)
│   │   └── ...
│   │
│   ├── integration/            # Component integration tests
│   │   ├── mod.rs             # Module exports
│   │   ├── transport/         # Transport-specific tests
│   │   ├── session/           # Session management tests
│   │   ├── auth/              # Authentication tests
│   │   └── proxy/             # Proxy logic tests
│   │
│   ├── e2e/                   # End-to-end tests (NEW)
│   │   ├── mod.rs             # Module configuration
│   │   ├── harness/           # Test infrastructure
│   │   │   ├── mod.rs
│   │   │   ├── process.rs    # Process management
│   │   │   ├── ports.rs      # Port allocation
│   │   │   ├── logs.rs       # Log collection
│   │   │   └── fixtures.rs   # Test data
│   │   │
│   │   ├── scenarios/         # Test scenarios
│   │   │   ├── mod.rs
│   │   │   ├── basic/        # Basic functionality
│   │   │   ├── mcp/          # MCP protocol tests
│   │   │   ├── performance/  # Performance tests
│   │   │   ├── resilience/   # Error recovery
│   │   │   └── security/     # Security tests
│   │   │
│   │   ├── validators/        # MCP validator tests
│   │   │   ├── mod.rs
│   │   │   ├── compliance.rs # Protocol compliance
│   │   │   └── interop.rs    # Interoperability
│   │   │
│   │   └── helpers/           # Test utilities
│   │       ├── mod.rs
│   │       ├── clients.rs    # Test clients
│   │       ├── servers.rs    # Mock servers
│   │       └── assertions.rs # Custom assertions
│   │
│   ├── common/                 # Shared test utilities (existing)
│   │   ├── mod.rs
│   │   └── mock_servers.rs
│   │
│   ├── fixtures/               # Test data files
│   │   ├── configs/           # Test configurations
│   │   ├── messages/          # Sample MCP messages
│   │   ├── tapes/             # Recorded sessions
│   │   └── certs/             # Test certificates
│   │
│   └── benchmarks/             # Performance benchmarks
│       ├── mod.rs
│       ├── latency.rs
│       └── throughput.rs
```

## Naming Conventions

### 1. Test Files

```rust
// Format: test_[component]_[scenario].rs

// Good examples:
test_proxy_basic_flow.rs
test_mcp_compliance.rs
test_sse_reconnection.rs
test_auth_oauth_flow.rs
test_rate_limiting_enforcement.rs

// Bad examples:
test1.rs                    // Not descriptive
proxy_tests.rs              // Too generic
test_everything.rs          // Too broad
MyProxyTest.rs             // Wrong casing
```

### 2. Test Functions

```rust
// Format: test_[component]_[action]_[expected_result]

// Basic tests
#[test]
fn test_port_allocator_allocate_returns_unique_port() { }

#[tokio::test]
async fn test_proxy_forward_request_succeeds() { }

// Negative tests
#[test]
fn test_auth_invalid_token_returns_401() { }

// Edge cases
#[test]
fn test_session_manager_cleanup_removes_expired_sessions() { }

// Performance tests
#[test]
fn test_proxy_throughput_exceeds_10k_rps() { }
```

### 3. Test Modules

```rust
// Group related tests in modules
mod proxy_tests {
    mod forward_mode {
        #[test]
        fn test_basic_forwarding() { }
        
        #[test]
        fn test_with_authentication() { }
    }
    
    mod reverse_mode {
        #[test]
        fn test_upstream_routing() { }
        
        #[test]
        fn test_load_balancing() { }
    }
}
```

### 4. Test Categories

Use attributes to categorize tests:

```rust
// Unit tests (fast, isolated)
#[test]
#[cfg(test)]
fn test_unit_parse_message() { }

// Integration tests (component interaction)
#[test]
#[ignore] // Run with --ignored
fn test_integration_database_connection() { }

// E2E tests (full system)
#[test]
#[cfg(feature = "e2e-tests")]
fn test_e2e_complete_flow() { }

// Performance tests
#[test]
#[cfg(feature = "bench")]
fn test_perf_latency_under_load() { }

// Flaky tests (temporarily disabled)
#[test]
#[ignore = "Flaky on CI - see issue #123"]
fn test_timing_sensitive_operation() { }
```

## Test Organization Patterns

### 1. Scenario-Based Organization

```rust
// tests/e2e/scenarios/basic/proxy_flow.rs
mod basic_proxy_flow {
    use crate::e2e::harness::*;
    
    #[tokio::test]
    async fn test_simple_request_response() {
        // Arrange
        let harness = TestHarness::new().await;
        let upstream = harness.start_mock_upstream().await?;
        let proxy = harness.start_proxy(&upstream).await?;
        
        // Act
        let response = proxy.send_request("ping").await?;
        
        // Assert
        assert_eq!(response.status(), 200);
        harness.assert_no_errors().await?;
    }
}
```

### 2. Feature-Based Organization

```rust
// tests/e2e/scenarios/mcp/protocol_compliance.rs
mod mcp_protocol_compliance {
    #[tokio::test]
    async fn test_initialize_handshake() { }
    
    #[tokio::test]
    async fn test_session_management() { }
    
    #[tokio::test]
    async fn test_batch_requests() { }
}
```

### 3. Transport-Based Organization

```rust
// tests/e2e/scenarios/transport/stdio_tests.rs
mod stdio_transport {
    #[tokio::test]
    async fn test_stdio_subprocess_spawn() { }
    
    #[tokio::test]
    async fn test_stdio_bidirectional_communication() { }
}

// tests/e2e/scenarios/transport/http_tests.rs
mod http_transport {
    #[tokio::test]
    async fn test_http_request_routing() { }
    
    #[tokio::test]
    async fn test_http_keepalive() { }
}
```

## Test Documentation

### 1. Test Function Documentation

```rust
/// Tests that the proxy correctly forwards MCP requests to upstream servers.
/// 
/// This test:
/// 1. Starts a mock MCP server on a dynamic port
/// 2. Configures the proxy to route to the mock server
/// 3. Sends a test request through the proxy
/// 4. Verifies the response matches expectations
/// 
/// Related issues: #123, #456
#[tokio::test]
async fn test_proxy_forwards_mcp_requests() {
    // Test implementation
}
```

### 2. Module Documentation

```rust
//! # MCP Protocol Compliance Tests
//! 
//! This module contains tests that verify Shadowcat's compliance with
//! the MCP protocol specification (version 2025-11-05).
//! 
//! ## Test Coverage
//! - Protocol handshake and initialization
//! - Session management lifecycle
//! - Message routing and responses
//! - Error handling and recovery
//! 
//! ## Running These Tests
//! ```bash
//! cargo test --test e2e_mcp_compliance
//! ```
```

## Test Fixtures and Helpers

### 1. Fixture Naming

```
fixtures/
├── configs/
│   ├── proxy_minimal.toml      # Minimal valid config
│   ├── proxy_full.toml         # All features enabled
│   └── proxy_invalid.toml      # For error testing
├── messages/
│   ├── mcp_initialize.json     # Valid initialization
│   ├── mcp_batch.json          # Batch request
│   └── mcp_malformed.json      # Invalid message
└── tapes/
    ├── session_normal.tape      # Normal session
    └── session_error.tape       # Error scenarios
```

### 2. Helper Functions

```rust
// tests/e2e/helpers/builders.rs

/// Creates a standard test configuration
pub fn test_config() -> ProxyConfig {
    ProxyConfig::default()
        .with_port(0) // Dynamic allocation
        .with_timeout(Duration::from_secs(5))
}

/// Creates a test MCP request
pub fn test_request(method: &str) -> MpcRequest {
    MpcRequest::new(method)
        .with_id(uuid::Uuid::new_v4())
        .with_params(json!({}))
}
```

## Test Execution Patterns

### 1. Test Grouping for CI

```toml
# Cargo.toml
[features]
default = ["unit-tests", "integration-tests"]
unit-tests = []
integration-tests = []
e2e-tests = ["integration-tests"]
all-tests = ["unit-tests", "integration-tests", "e2e-tests"]
ci-tests = ["unit-tests", "integration-tests"]
nightly-tests = ["all-tests", "bench"]
```

### 2. Test Commands

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test 'integration_*'

# Run only E2E tests
cargo test --test 'e2e_*' --features e2e-tests

# Run specific scenario
cargo test --test e2e_mcp_compliance

# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Run tests in parallel (default)
cargo test -- --test-threads=4

# Run tests serially (for debugging)
cargo test -- --test-threads=1
```

## Test Data Management

### 1. Test Data Builders

```rust
pub struct TestDataBuilder {
    messages: Vec<MpcMessage>,
    config: ProxyConfig,
}

impl TestDataBuilder {
    pub fn new() -> Self { }
    
    pub fn with_message(mut self, msg: MpcMessage) -> Self {
        self.messages.push(msg);
        self
    }
    
    pub fn with_random_messages(mut self, count: usize) -> Self {
        // Generate random but valid messages
        self
    }
    
    pub fn build(self) -> TestData {
        TestData {
            messages: self.messages,
            config: self.config,
        }
    }
}
```

### 2. Fixture Loading

```rust
pub struct Fixtures;

impl Fixtures {
    pub fn load_config(name: &str) -> Result<ProxyConfig> {
        let path = format!("tests/fixtures/configs/{}.toml", name);
        ProxyConfig::from_file(&path)
    }
    
    pub fn load_message(name: &str) -> Result<MpcMessage> {
        let path = format!("tests/fixtures/messages/{}.json", name);
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
    }
}
```

## Assertion Patterns

### 1. Custom Assertions

```rust
pub trait ProxyAssertions {
    async fn assert_listening(&self) -> Result<()>;
    async fn assert_healthy(&self) -> Result<()>;
    async fn assert_request_count(&self, expected: usize) -> Result<()>;
}

pub trait LogAssertions {
    async fn assert_contains(&self, pattern: &str) -> Result<()>;
    async fn assert_no_errors(&self) -> Result<()>;
    async fn assert_sequence(&self, patterns: &[&str]) -> Result<()>;
}
```

### 2. Assertion Macros

```rust
#[macro_export]
macro_rules! assert_response_ok {
    ($response:expr) => {
        assert!(
            $response.is_success(),
            "Expected success response, got: {:?}",
            $response.status()
        )
    };
}

#[macro_export]
macro_rules! assert_within_duration {
    ($duration:expr, $limit:expr) => {
        assert!(
            $duration <= $limit,
            "Operation took {:?}, expected under {:?}",
            $duration,
            $limit
        )
    };
}
```

## CI/CD Integration

### 1. GitHub Actions Test Matrix

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  e2e-tests:
    strategy:
      matrix:
        scenario: [basic, mcp, performance, resilience]
        os: [ubuntu-latest, macos-latest]
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v2
      
      - name: Run E2E Test Scenario
        run: |
          cargo test --test "e2e_${{ matrix.scenario }}_*" \
            --features e2e-tests \
            -- --test-threads=2
      
      - name: Upload Logs on Failure
        if: failure()
        uses: actions/upload-artifact@v2
        with:
          name: test-logs-${{ matrix.scenario }}-${{ matrix.os }}
          path: target/test-logs/
```

### 2. Test Sharding

```rust
// Use environment variables for test sharding
fn should_run_test(test_name: &str) -> bool {
    let shard = std::env::var("TEST_SHARD").ok();
    let total = std::env::var("TEST_TOTAL_SHARDS").ok();
    
    match (shard, total) {
        (Some(s), Some(t)) => {
            let shard: usize = s.parse().unwrap();
            let total: usize = t.parse().unwrap();
            let hash = calculate_hash(test_name);
            (hash % total) == shard
        }
        _ => true // Run all tests if not sharding
    }
}
```

## Best Practices

1. **Test Independence**: Each test should be completely independent
2. **Resource Cleanup**: Always clean up resources, even on panic
3. **Meaningful Names**: Test names should describe what is being tested
4. **Fast Feedback**: Organize tests from fast to slow
5. **Clear Failures**: Assertion messages should explain what went wrong
6. **Deterministic**: Avoid time-dependent or random behaviors
7. **Documentation**: Document complex test scenarios
8. **Reusable Helpers**: Extract common setup into helper functions

## Summary

This organization provides:
- **Clear Structure**: Easy to find and understand tests
- **Consistent Naming**: Predictable test and file names  
- **Flexible Execution**: Run subsets of tests easily
- **CI/CD Ready**: Integrated with build pipelines
- **Maintainable**: Clear patterns for adding new tests
- **Discoverable**: Easy to find relevant tests