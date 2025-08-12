# Task C.6: Extensive Test Coverage

## Overview
Achieve comprehensive test coverage (70%+) for the Shadowcat library, focusing on error paths, edge cases, and integration scenarios.

## Duration
6 hours

## Dependencies
- Phase B complete (library implementation)
- C.5 complete (performance optimizations)

## Objectives
1. Analyze current test coverage with tarpaulin
2. Add integration tests for error paths
3. Test shutdown scenarios comprehensively
4. Add property-based tests for builders
5. Test configuration loading edge cases
6. Achieve 70%+ overall test coverage

## Deliverables

### 1. Coverage Analysis
- [ ] Install and configure cargo-tarpaulin
- [ ] Generate baseline coverage report
- [ ] Identify gaps in test coverage
- [ ] Prioritize critical untested paths

### 2. Error Path Testing
- [ ] Transport connection failures
- [ ] Invalid message parsing
- [ ] Session limit exceeded
- [ ] Timeout scenarios
- [ ] Network interruptions
- [ ] Invalid configuration

### 3. Shutdown Testing
- [ ] Graceful shutdown with active connections
- [ ] Forced shutdown scenarios
- [ ] Shutdown with pending operations
- [ ] Multi-component shutdown coordination
- [ ] Shutdown timeout handling

### 4. Property-Based Tests
- [ ] Builder invariants (using proptest)
- [ ] Message serialization/deserialization roundtrips
- [ ] Session ID generation uniqueness
- [ ] Configuration validation properties

### 5. Configuration Edge Cases
- [ ] Missing configuration files
- [ ] Invalid TOML/YAML syntax
- [ ] Environment variable overrides
- [ ] Type mismatches
- [ ] Partial configurations
- [ ] Default value handling

### 6. Integration Test Scenarios
- [ ] Full proxy flow with errors
- [ ] Multiple concurrent sessions
- [ ] Rate limiting enforcement
- [ ] Interceptor chain processing
- [ ] Recording and replay cycles
- [ ] Auth gateway flows

## Success Criteria
- [ ] 70%+ overall test coverage
- [ ] All critical paths have tests
- [ ] Error scenarios properly tested
- [ ] No untested public APIs
- [ ] Property tests for key invariants
- [ ] Integration tests passing

## Implementation Steps

### Step 1: Setup Coverage Tools
```bash
cargo install cargo-tarpaulin

# Generate initial coverage report
cargo tarpaulin --out Html --output-dir coverage

# For CI integration
cargo tarpaulin --out Xml
```

### Step 2: Identify Coverage Gaps
```bash
# Generate detailed coverage
cargo tarpaulin --verbose --line-coverage --branch-coverage

# Focus on specific modules
cargo tarpaulin --packages shadowcat --exclude-files "*/tests/*"
```

### Step 3: Add Property Tests
```toml
# Add to Cargo.toml
[dev-dependencies]
proptest = "1.0"
```

Example property test:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_builder_invariants(
        timeout in 0u64..10000,
        max_sessions in 0usize..10000,
    ) {
        let builder = SessionManagerBuilder::new()
            .with_timeout(Duration::from_millis(timeout))
            .with_max_sessions(max_sessions);
        
        let manager = builder.build().unwrap();
        assert!(manager.config.timeout_duration.as_millis() == timeout);
        assert!(manager.config.max_sessions == Some(max_sessions));
    }
}
```

### Step 4: Error Path Tests
```rust
#[tokio::test]
async fn test_transport_connection_failure() {
    let transport = StdioTransport::new("nonexistent", &[]);
    assert!(matches!(
        transport.connect().await,
        Err(TransportError::ConnectionFailed(_))
    ));
}

#[tokio::test]
async fn test_session_limit_exceeded() {
    let manager = SessionManagerBuilder::new()
        .with_max_sessions(1)
        .build()
        .unwrap();
    
    let session1 = manager.create_session(SessionId::generate()).await;
    assert!(session1.is_ok());
    
    let session2 = manager.create_session(SessionId::generate()).await;
    assert!(matches!(
        session2,
        Err(SessionError::TooManySessions(_))
    ));
}
```

### Step 5: Integration Tests
Create `tests/integration_test_coverage.rs`:
```rust
use shadowcat::{Shadowcat, ShadowcatBuilder};

#[tokio::test]
async fn test_full_proxy_error_recovery() {
    // Test proxy behavior with upstream failures
}

#[tokio::test]
async fn test_concurrent_session_handling() {
    // Test multiple sessions simultaneously
}
```

## Testing Checklist

### Core Components
- [ ] Transport trait implementations
- [ ] Session manager operations
- [ ] Proxy forward/reverse modes
- [ ] Interceptor chain
- [ ] Error types and conversions
- [ ] Builder patterns
- [ ] Configuration loading
- [ ] Shutdown system

### Edge Cases
- [ ] Empty messages
- [ ] Oversized messages
- [ ] Malformed JSON
- [ ] Unicode handling
- [ ] Concurrent operations
- [ ] Resource exhaustion

### Performance Tests
- [ ] Load testing scenarios
- [ ] Memory leak detection
- [ ] Stress testing
- [ ] Benchmark regression tests

## Notes
- Use `#[should_panic]` for expected panic tests
- Mock external dependencies appropriately
- Test both success and failure paths
- Ensure tests are deterministic
- Keep test execution time reasonable
- Document complex test scenarios

## References
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- [proptest](https://github.com/proptest-rs/proptest)
- [Rust testing best practices](https://doc.rust-lang.org/book/ch11-00-testing.html)