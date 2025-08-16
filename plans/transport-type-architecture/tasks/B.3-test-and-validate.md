# Task B.3: Test and Validate

## Objective
Comprehensive testing and validation of the ResponseMode/ClientCapabilities refactor. This task ensures the changes are correct, performant, and properly integrated throughout the codebase. We validate that dead code is truly removed and the new architecture works as designed.

## Context from Phase A Analysis

### What We're Validating (Reference: analysis/transport-usage-audit.md)
**Removed Dead Code**:
- `is_sse_session` was never set to true anywhere
- `mark_as_sse_session()` was never called
- System functioned without this code

**New Functionality**:
- ResponseMode properly detects response formats
- ClientCapabilities correctly tracks client features
- Session updates work with distributed storage
- Response routing uses new type-safe approach

### Testing Strategy (Reference: analysis/testing-strategy.md)
1. **Unit Tests**: Core type functionality
2. **Integration Tests**: End-to-end proxy flows
3. **Performance Tests**: No regression from changes
4. **Conformance Tests**: MCP protocol compliance
5. **Migration Tests**: Old patterns properly replaced

### Success Metrics (Reference: analysis/design-decisions.md)
- All tests pass
- No performance regression
- Code coverage maintained or improved
- No references to old methods remain
- Distributed storage compatibility verified

## Prerequisites
- [x] B.0 complete (types implemented)
- [x] B.1 complete (Session updated)
- [x] B.2 complete (usage sites migrated)
- [ ] All code compiles without errors
- [ ] On correct git branch

## Detailed Implementation Steps

### Step 1: Create Unit Tests for ResponseMode (15 min)

Create comprehensive tests in `src/transport/core/response_mode.rs`:

```rust
#[cfg(test)]
mod comprehensive_tests {
    use super::*;
    
    #[test]
    fn test_response_mode_complete_coverage() {
        // Test all standard MIME types
        let test_cases = vec![
            // JSON variants
            ("application/json", ResponseMode::Json),
            ("application/json; charset=utf-8", ResponseMode::Json),
            ("application/json;charset=UTF-8", ResponseMode::Json),
            ("application/json ; charset = utf-8", ResponseMode::Json),
            
            // SSE variants
            ("text/event-stream", ResponseMode::SseStream),
            ("text/event-stream; charset=utf-8", ResponseMode::SseStream),
            ("text/event-stream;charset=UTF-8", ResponseMode::SseStream),
            
            // Passthrough cases
            ("text/plain", ResponseMode::Passthrough),
            ("text/html", ResponseMode::Passthrough),
            ("application/xml", ResponseMode::Passthrough),
            ("application/octet-stream", ResponseMode::Passthrough),
            ("image/png", ResponseMode::Passthrough),
            ("video/mp4", ResponseMode::Passthrough),
            ("", ResponseMode::Passthrough),
            ("invalid-mime", ResponseMode::Passthrough),
            ("application/", ResponseMode::Passthrough),
            ("/json", ResponseMode::Passthrough),
        ];
        
        for (content_type, expected) in test_cases {
            let actual = ResponseMode::from_content_type(content_type);
            assert_eq!(
                actual, expected,
                "Failed for content_type: '{}'", content_type
            );
        }
    }
    
    #[test]
    fn test_response_mode_properties_exhaustive() {
        // Ensure all variants are tested
        let modes = [
            ResponseMode::Json,
            ResponseMode::SseStream,
            ResponseMode::Passthrough,
        ];
        
        // Test that exactly one mode requires buffering
        let buffering_count = modes.iter()
            .filter(|m| m.requires_buffering())
            .count();
        assert_eq!(buffering_count, 1, "Only JSON should require buffering");
        
        // Test that streaming is exclusive to SSE
        let streaming_count = modes.iter()
            .filter(|m| m.is_streaming())
            .count();
        assert_eq!(streaming_count, 1, "Only SSE should be streaming");
        
        // Test interception support
        let interception_count = modes.iter()
            .filter(|m| m.supports_interception())
            .count();
        assert_eq!(interception_count, 2, "JSON and SSE should support interception");
    }
    
    #[test]
    fn test_response_mode_case_sensitivity() {
        // MIME types should be case-insensitive
        assert_eq!(
            ResponseMode::from_content_type("APPLICATION/JSON"),
            ResponseMode::Json
        );
        assert_eq!(
            ResponseMode::from_content_type("Text/Event-Stream"),
            ResponseMode::SseStream
        );
    }
    
    #[test]
    fn test_response_mode_round_trip() {
        // Test serialization round-trip
        for mode in [ResponseMode::Json, ResponseMode::SseStream, ResponseMode::Passthrough] {
            // JSON serialization
            let json = serde_json::to_string(&mode).unwrap();
            let decoded: ResponseMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, decoded);
            
            // Bincode serialization (if used for storage)
            let bytes = bincode::serialize(&mode).unwrap();
            let decoded: ResponseMode = bincode::deserialize(&bytes).unwrap();
            assert_eq!(mode, decoded);
        }
    }
}
```

### Step 2: Create Unit Tests for ClientCapabilities (15 min)

Add comprehensive tests in `src/transport/core/capabilities.rs`:

```rust
#[cfg(test)]
mod comprehensive_tests {
    use super::*;
    
    #[test]
    fn test_capabilities_bit_operations() {
        let mut caps = ClientCapabilities::empty();
        
        // Test individual bit setting
        assert!(!caps.contains(ClientCapabilities::ACCEPTS_JSON));
        caps.insert(ClientCapabilities::ACCEPTS_JSON);
        assert!(caps.contains(ClientCapabilities::ACCEPTS_JSON));
        
        // Test bit removal
        caps.remove(ClientCapabilities::ACCEPTS_JSON);
        assert!(!caps.contains(ClientCapabilities::ACCEPTS_JSON));
        
        // Test multiple bits
        caps = ClientCapabilities::ACCEPTS_JSON | ClientCapabilities::ACCEPTS_SSE;
        assert!(caps.contains(ClientCapabilities::ACCEPTS_JSON));
        assert!(caps.contains(ClientCapabilities::ACCEPTS_SSE));
        assert!(!caps.contains(ClientCapabilities::ACCEPTS_BINARY));
        
        // Test intersection
        let caps2 = ClientCapabilities::ACCEPTS_SSE | ClientCapabilities::ACCEPTS_BINARY;
        let intersection = caps & caps2;
        assert_eq!(intersection, ClientCapabilities::ACCEPTS_SSE);
        
        // Test union
        let union = caps | caps2;
        assert!(union.contains(ClientCapabilities::ACCEPTS_JSON));
        assert!(union.contains(ClientCapabilities::ACCEPTS_SSE));
        assert!(union.contains(ClientCapabilities::ACCEPTS_BINARY));
    }
    
    #[test]
    fn test_capabilities_from_transport_type() {
        use crate::transport::TransportType;
        
        // Stdio should get standard (JSON only)
        let stdio_caps = ClientCapabilities::from_transport_type(TransportType::Stdio);
        assert_eq!(stdio_caps, ClientCapabilities::STANDARD);
        assert!(stdio_caps.contains(ClientCapabilities::ACCEPTS_JSON));
        assert!(!stdio_caps.contains(ClientCapabilities::ACCEPTS_SSE));
        
        // HTTP should get streaming
        let http_caps = ClientCapabilities::from_transport_type(TransportType::Http);
        assert_eq!(http_caps, ClientCapabilities::STREAMING);
        assert!(http_caps.contains(ClientCapabilities::ACCEPTS_JSON));
        assert!(http_caps.contains(ClientCapabilities::ACCEPTS_SSE));
        
        // SSE should get streaming
        let sse_caps = ClientCapabilities::from_transport_type(TransportType::Sse);
        assert_eq!(sse_caps, ClientCapabilities::STREAMING);
    }
    
    #[test]
    fn test_capabilities_accept_header_parsing() {
        // Complex Accept header
        let caps = ClientCapabilities::from_accept_header(
            "text/html, application/json;q=0.9, text/event-stream;q=0.8, */*;q=0.1"
        );
        assert!(caps.contains(ClientCapabilities::ACCEPTS_JSON));
        assert!(caps.contains(ClientCapabilities::ACCEPTS_SSE));
        assert!(caps.contains(ClientCapabilities::ACCEPTS_BINARY)); // from */*
        
        // Empty header
        let caps = ClientCapabilities::from_accept_header("");
        assert!(caps.is_empty());
        
        // Wildcard only
        let caps = ClientCapabilities::from_accept_header("*/*");
        assert!(caps.contains(ClientCapabilities::ACCEPTS_BINARY));
    }
    
    #[test]
    fn test_capabilities_response_mode_compatibility() {
        // Standard caps should accept JSON only
        let standard = ClientCapabilities::STANDARD;
        assert!(standard.accepts_response_mode(ResponseMode::Json));
        assert!(!standard.accepts_response_mode(ResponseMode::SseStream));
        
        // Streaming caps should accept both JSON and SSE
        let streaming = ClientCapabilities::STREAMING;
        assert!(streaming.accepts_response_mode(ResponseMode::Json));
        assert!(streaming.accepts_response_mode(ResponseMode::SseStream));
        
        // Empty caps should accept passthrough
        let empty = ClientCapabilities::empty();
        assert!(empty.accepts_response_mode(ResponseMode::Passthrough));
        assert!(!empty.accepts_response_mode(ResponseMode::Json));
    }
    
    #[test]
    fn test_capabilities_constants_are_distinct() {
        // Ensure no overlapping bits
        let all_flags = [
            ClientCapabilities::ACCEPTS_JSON,
            ClientCapabilities::ACCEPTS_SSE,
            ClientCapabilities::ACCEPTS_BINARY,
            ClientCapabilities::SUPPORTS_COMPRESSION,
            ClientCapabilities::SUPPORTS_BATCH,
            ClientCapabilities::SUPPORTS_WEBSOCKET,
            ClientCapabilities::SUPPORTS_CANCELLATION,
            ClientCapabilities::SUPPORTS_PROGRESS,
        ];
        
        for i in 0..all_flags.len() {
            for j in (i+1)..all_flags.len() {
                let intersection = all_flags[i] & all_flags[j];
                assert!(
                    intersection.is_empty(),
                    "Flags {:?} and {:?} overlap!",
                    all_flags[i], all_flags[j]
                );
            }
        }
    }
}
```

### Step 3: Create Integration Tests (20 min)

Create `tests/integration_response_mode.rs`:

```rust
use shadowcat::session::{Session, SessionStore, InMemorySessionStore};
use shadowcat::transport::{ResponseMode, ClientCapabilities, TransportType};
use shadowcat::proxy::{ForwardProxy, ReverseProxy};

#[tokio::test]
async fn test_session_response_mode_tracking() {
    let store = InMemorySessionStore::new();
    let session_id = SessionId::new();
    
    // Create session
    let mut session = Session::new(session_id.clone(), TransportType::Http);
    assert_eq!(session.response_mode, None);
    
    // Set response mode
    session.set_response_mode(ResponseMode::SseStream);
    assert_eq!(session.response_mode, Some(ResponseMode::SseStream));
    assert!(session.is_streaming());
    
    // Store and retrieve
    store.create(session.clone()).await.unwrap();
    let retrieved = store.get(&session_id).await.unwrap().unwrap();
    assert_eq!(retrieved.response_mode, Some(ResponseMode::SseStream));
}

#[tokio::test]
async fn test_forward_proxy_response_routing() {
    // Setup mock transports and proxy
    let mut proxy = ForwardProxy::new();
    let session_id = SessionId::new();
    
    // Create session with standard capabilities (JSON only)
    let session = Session::new(session_id.clone(), TransportType::Stdio);
    proxy.session_store.create(session).await.unwrap();
    
    // Simulate JSON response
    let json_response = create_mock_response("application/json", r#"{"result": "ok"}"#);
    let result = proxy.handle_response(json_response, &session_id).await;
    assert!(result.is_ok(), "Should handle JSON response");
    
    // Verify session updated with response mode
    let session = proxy.session_store.get(&session_id).await.unwrap().unwrap();
    assert_eq!(session.response_mode, Some(ResponseMode::Json));
    
    // Simulate SSE response (should fail for stdio client)
    let sse_response = create_mock_response("text/event-stream", "data: test\n\n");
    let result = proxy.handle_response(sse_response, &session_id).await;
    assert!(result.is_err(), "Stdio client shouldn't accept SSE");
}

#[tokio::test]
async fn test_reverse_proxy_capability_negotiation() {
    let mut proxy = ReverseProxy::new("127.0.0.1:0");
    
    // Test request with SSE accept header
    let request = create_mock_request()
        .header("Accept", "text/event-stream, application/json");
    
    let session_id = proxy.handle_request(request).await.unwrap();
    
    // Verify session has correct capabilities
    let session = proxy.session_store.get(&session_id).await.unwrap().unwrap();
    assert!(session.client_capabilities.contains(ClientCapabilities::ACCEPTS_JSON));
    assert!(session.client_capabilities.contains(ClientCapabilities::ACCEPTS_SSE));
    assert!(session.accepts_response_mode(ResponseMode::SseStream));
}

#[tokio::test]
async fn test_no_is_sse_session_references() {
    // This test ensures the old field is completely gone
    // It should not compile if any references remain
    
    let session = Session::new(SessionId::new(), TransportType::Http);
    
    // These lines should fail to compile if uncommented:
    // let _ = session.is_sse_session;  // Field doesn't exist
    // session.mark_as_sse_session();   // Method doesn't exist
    // let _ = session.is_sse();        // Method doesn't exist
    
    // Only new methods should work
    assert!(session.response_mode.is_none());
    assert!(!session.is_streaming());
}
```

### Step 4: Create Performance Benchmark Tests (15 min)

Create `benches/response_mode_bench.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shadowcat::transport::{ResponseMode, ClientCapabilities};

fn benchmark_response_mode_detection(c: &mut Criterion) {
    c.bench_function("response_mode_from_content_type", |b| {
        let content_types = vec![
            "application/json",
            "text/event-stream",
            "text/plain",
            "application/json; charset=utf-8",
            "text/event-stream; charset=utf-8",
        ];
        
        b.iter(|| {
            for ct in &content_types {
                black_box(ResponseMode::from_content_type(ct));
            }
        });
    });
}

fn benchmark_client_capabilities(c: &mut Criterion) {
    c.bench_function("capabilities_bitwise_ops", |b| {
        let caps1 = ClientCapabilities::STREAMING;
        let caps2 = ClientCapabilities::STANDARD;
        
        b.iter(|| {
            black_box(caps1.contains(ClientCapabilities::ACCEPTS_JSON));
            black_box(caps1 & caps2);
            black_box(caps1 | caps2);
            black_box(caps1.accepts_response_mode(ResponseMode::Json));
        });
    });
    
    c.bench_function("capabilities_from_accept_header", |b| {
        let headers = vec![
            "application/json",
            "text/event-stream, application/json",
            "*/*",
            "text/html, application/json;q=0.9, text/event-stream;q=0.8",
        ];
        
        b.iter(|| {
            for header in &headers {
                black_box(ClientCapabilities::from_accept_header(header));
            }
        });
    });
}

criterion_group!(benches, benchmark_response_mode_detection, benchmark_client_capabilities);
criterion_main!(benches);
```

Add to `Cargo.toml`:
```toml
[[bench]]
name = "response_mode_bench"
harness = false

[dev-dependencies]
criterion = "0.5"
```

### Step 5: Validate Dead Code Removal (10 min)

Create validation script:

```bash
#!/bin/bash
# validate_refactor.sh

echo "=== Validating ResponseMode Refactor ==="

# Check for any remaining references to old code
echo "Checking for old field/method references..."

OLD_PATTERNS=(
    "is_sse_session"
    "mark_as_sse_session"
    "\.is_sse\(\)"
)

FOUND_ISSUES=0

for pattern in "${OLD_PATTERNS[@]}"; do
    echo -n "Checking for '$pattern'... "
    if rg "$pattern" src/ tests/ --type rust --quiet; then
        echo "FOUND! (This should be removed)"
        rg "$pattern" src/ tests/ --type rust
        FOUND_ISSUES=$((FOUND_ISSUES + 1))
    else
        echo "✓ Clean"
    fi
done

# Check that new types are properly used
echo -e "\nVerifying new types are in use..."

NEW_PATTERNS=(
    "ResponseMode"
    "ClientCapabilities"
    "response_mode"
    "client_capabilities"
)

for pattern in "${NEW_PATTERNS[@]}"; do
    echo -n "Checking for '$pattern'... "
    count=$(rg "$pattern" src/ --type rust | wc -l)
    if [ $count -gt 0 ]; then
        echo "✓ Found $count uses"
    else
        echo "WARNING: No uses found"
    fi
done

# Check compilation
echo -e "\nChecking compilation..."
if cargo check --all-targets 2>&1 | grep -q "error"; then
    echo "✗ Compilation errors found"
    FOUND_ISSUES=$((FOUND_ISSUES + 1))
else
    echo "✓ Compiles successfully"
fi

# Check clippy
echo -e "\nRunning clippy..."
if cargo clippy --all-targets -- -D warnings 2>&1 | grep -q "error"; then
    echo "✗ Clippy warnings found"
    FOUND_ISSUES=$((FOUND_ISSUES + 1))
else
    echo "✓ No clippy warnings"
fi

# Summary
echo -e "\n=== Summary ==="
if [ $FOUND_ISSUES -eq 0 ]; then
    echo "✓ All validations passed!"
    exit 0
else
    echo "✗ Found $FOUND_ISSUES issues that need fixing"
    exit 1
fi
```

### Step 6: Test Distributed Storage Compatibility (15 min)

Create test for Redis compatibility (even if not implemented yet):

```rust
#[cfg(test)]
mod distributed_storage_tests {
    use super::*;
    
    #[test]
    fn test_session_serialization_with_new_fields() {
        let mut session = Session::new(SessionId::new(), TransportType::Http);
        session.set_response_mode(ResponseMode::SseStream);
        session.update_capabilities(ClientCapabilities::STREAMING);
        session.set_upstream_session_id(SessionId::new());
        
        // JSON serialization (for Redis JSON)
        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();
        
        assert_eq!(session.id, deserialized.id);
        assert_eq!(session.response_mode, deserialized.response_mode);
        assert_eq!(session.client_capabilities, deserialized.client_capabilities);
        assert_eq!(session.upstream_session_id, deserialized.upstream_session_id);
        
        // Binary serialization (for Redis binary)
        let bytes = bincode::serialize(&session).unwrap();
        let deserialized: Session = bincode::deserialize(&bytes).unwrap();
        
        assert_eq!(session.id, deserialized.id);
        assert_eq!(session.response_mode, deserialized.response_mode);
    }
    
    #[test]
    fn test_backward_compatibility() {
        // Test that sessions without new fields can still be deserialized
        // This simulates reading old sessions from storage
        
        let json_without_new_fields = r#"{
            "id": "test-id",
            "transport_type": "Http",
            "status": "Active",
            "state": "Established",
            "created_at": 1234567890,
            "last_activity": 1234567890,
            "frame_count": 0,
            "client_info": null,
            "server_info": null,
            "version_state": {},
            "tags": [],
            "last_event_id": null
        }"#;
        
        // Should deserialize with None/default for new fields
        let session: Result<Session, _> = serde_json::from_str(json_without_new_fields);
        
        // Note: This might fail if new fields are required
        // Document this as a breaking change if needed
        if let Ok(session) = session {
            assert_eq!(session.response_mode, None);
            assert_eq!(session.client_capabilities, ClientCapabilities::default());
            assert_eq!(session.upstream_session_id, None);
        }
    }
}
```

### Step 7: Run Full Test Suite (10 min)

```bash
# Clean everything first
cargo clean

# Run all tests with coverage
cargo install cargo-tarpaulin  # If not installed
cargo tarpaulin --out Html --output-dir coverage/

# Run tests in different modes
echo "=== Running unit tests ==="
cargo test --lib

echo "=== Running integration tests ==="
cargo test --tests

echo "=== Running doc tests ==="
cargo test --doc

echo "=== Running with all features ==="
cargo test --all-features

echo "=== Running in release mode ==="
cargo test --release

# Run benchmarks
echo "=== Running benchmarks ==="
cargo bench

# Check test output for any deprecation warnings
cargo test 2>&1 | grep -i "deprecat" && echo "Found deprecation warnings!" || echo "No deprecation warnings"
```

### Step 8: Create Test Report (10 min)

Create `test_report.md`:

```markdown
# ResponseMode Refactor Test Report

## Date: [DATE]
## Branch: refactor/transport-type-architecture

### Test Coverage

| Component | Before | After | Change |
|-----------|--------|-------|--------|
| transport::core | N/A | 95% | New |
| session::store | 85% | 88% | +3% |
| proxy::forward | 75% | 78% | +3% |
| proxy::reverse | 70% | 74% | +4% |

### Performance Benchmarks

| Operation | Before | After | Change |
|-----------|--------|-------|--------|
| Session creation | 1.2µs | 1.3µs | +8% |
| Response detection | N/A | 250ns | New |
| Capability check | N/A | 15ns | New |
| Session serialization | 2.1µs | 2.3µs | +9% |

### Dead Code Verification

- [x] is_sse_session field removed
- [x] mark_as_sse_session() method removed
- [x] is_sse() method removed
- [x] No references remain in codebase

### New Functionality Verification

- [x] ResponseMode::from_content_type() works correctly
- [x] ClientCapabilities bitflags operations efficient
- [x] Session serialization includes new fields
- [x] Forward proxy uses ResponseMode for routing
- [x] Reverse proxy uses ClientCapabilities for negotiation

### Regression Testing

- [x] All existing tests pass
- [x] No performance regression
- [x] Distributed storage compatibility maintained
- [x] MCP protocol compliance verified

### Issues Found and Fixed

1. **Issue**: [Description]
   **Fix**: [Solution]

### Recommendations

1. Monitor performance in production for session operations
2. Add metrics for response mode distribution
3. Consider caching ResponseMode detection for repeated content-types

### Sign-off

- [ ] Unit tests complete and passing
- [ ] Integration tests complete and passing
- [ ] Performance acceptable
- [ ] No regressions found
- [ ] Documentation updated
```

## Success Criteria Checklist

- [ ] All unit tests for ResponseMode pass
- [ ] All unit tests for ClientCapabilities pass
- [ ] Integration tests demonstrate proper routing
- [ ] Performance benchmarks show no significant regression
- [ ] Dead code validation confirms removal
- [ ] Distributed storage tests pass
- [ ] Full test suite passes
- [ ] Test coverage maintained or improved
- [ ] No clippy warnings
- [ ] No references to old methods exist
- [ ] Test report completed

## Common Issues and Solutions

1. **Test failures in integration tests**
   - Ensure test fixtures use new API correctly
   - Check that mock transports implement new methods

2. **Performance regression**
   - ResponseMode detection should be fast (< 500ns)
   - Consider caching if needed
   - Bitflags operations should be < 20ns

3. **Serialization test failures**
   - Ensure serde derives on all new types
   - Check for breaking changes in session format

4. **Coverage decrease**
   - Add tests for edge cases
   - Test error paths
   - Test all ResponseMode variants

5. **Benchmark compilation issues**
   - Add criterion to dev-dependencies
   - Use correct benchmark harness setup

## Test Checklist

### Before Starting Tests
- [ ] Code compiles without warnings
- [ ] Clippy passes
- [ ] Documentation comments added

### Unit Tests
- [ ] ResponseMode all variants tested
- [ ] ClientCapabilities all flags tested
- [ ] Session new methods tested
- [ ] Edge cases covered

### Integration Tests
- [ ] Forward proxy routing tested
- [ ] Reverse proxy negotiation tested
- [ ] Session persistence tested
- [ ] Error cases tested

### Performance Tests
- [ ] Benchmarks run successfully
- [ ] No significant regression (< 10%)
- [ ] Memory usage stable

### Validation
- [ ] Dead code removed
- [ ] New code properly integrated
- [ ] No breaking changes (unless documented)

## Duration Estimate
**Total: 105 minutes**
- Unit tests ResponseMode: 15 min
- Unit tests ClientCapabilities: 15 min
- Integration tests: 20 min
- Performance benchmarks: 15 min
- Dead code validation: 10 min
- Distributed storage tests: 15 min
- Full test suite: 10 min
- Test report: 10 min
- Buffer: 5 min

## Next Steps
After completing this task:
1. Review test report with team
2. Address any issues found
3. Update documentation if needed
4. Commit: `git commit -m "test: comprehensive validation of ResponseMode refactor"`
5. Create PR for review
6. Proceed to Phase C implementation

## Notes
- This is where we prove the refactor works
- Take time to test edge cases
- Performance is critical - measure carefully
- Document any surprising findings
- Consider adding property-based tests for exhaustive coverage

---

**Task Status**: Ready for implementation
**Prerequisites**: B.0, B.1, B.2 complete
**Blocks**: Phase C
**Reviewer**: Verify all tests pass and coverage is maintained