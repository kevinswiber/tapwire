# Task C.3: Integration Testing

## Objective
Comprehensive integration testing of the refactored transport layer. Validate that the extracted raw primitives, refactored directional transports, and unified factory work correctly together while maintaining performance targets.

## Design Reference
This task validates the implementation of **Phase 2** from `analysis/implementation-roadmap.md` (lines 207-332).

Testing strategy from implementation-roadmap.md lines 463-516:
- Unit tests for each component
- Integration tests for complete flows
- Performance benchmarks to ensure no regression

## Prerequisites
- [x] C.0 complete (raw primitives created)
- [x] C.1 complete (directional transports refactored)
- [x] C.2 complete (unified factory implemented)
- [ ] Test environment ready

## Test Implementation Steps

### Step 1: Unit Tests for Raw Primitives (20 min)

Following implementation-roadmap.md lines 467-474:

```rust
// tests/unit/raw_transport_test.rs

use shadowcat::transport::raw::{StdioCore, HttpCore, SseCore};

#[tokio::test]
async fn test_stdio_core_operations() {
    let pool = Arc::new(BytesPool::new(8192, 10));
    let mut core = StdioCore::new(pool);
    
    // Test send
    let data = b"test data";
    core.send_bytes(data).await.unwrap();
    
    // Test receive
    let received = core.receive_bytes().await.unwrap();
    assert!(!received.is_empty());
}

#[test]
fn test_sse_core_parsing() {
    let mut core = SseCore::new();
    
    // Test event parsing
    let event = core.parse_event("data: {\"test\": true}").unwrap();
    assert_eq!(event.data, "{\"test\": true}");
    
    // Test ID parsing
    let event = core.parse_event("id: 123").unwrap();
    assert_eq!(event.id, Some("123".to_string()));
}

#[tokio::test]
async fn test_http_core_request() {
    let core = HttpCore::new(Arc::new(BytesPool::new(16384, 10)));
    
    // Test request sending
    let request = Request::builder()
        .method("POST")
        .uri("http://localhost:8080")
        .body(Body::from("test"))
        .unwrap();
    
    // Would need mock server for full test
}
```

### Step 2: Test Shared Transport Logic (20 min)

Following implementation-roadmap.md lines 476-484:

```rust
// tests/unit/shared_transport_test.rs

#[tokio::test]
async fn test_directional_uses_raw_primitives() {
    // Verify StdioIncoming uses StdioCore
    let incoming = StdioIncoming::new(Arc::new(BytesPool::default()));
    
    // The implementation should delegate to core
    // This is more of a compilation test - if it compiles, delegation works
}

#[tokio::test]
async fn test_no_duplicate_io_logic() {
    // This test validates our refactoring
    // Count lines of I/O code in directional transports
    // Should be significantly reduced after refactoring
}
```

### Step 3: Test Transport Factory (25 min)

Following implementation-roadmap.md lines 480-484:

```rust
// tests/unit/transport_factory_test.rs

use shadowcat::transport::factory::{TransportFactory, TransportBuilder};

#[tokio::test]
async fn test_factory_creation() {
    let factory = TransportBuilder::new()
        .pool_size(5)
        .stdio_buffer_size(4096)
        .build();
    
    // Test incoming creation
    let incoming = factory.create_incoming(TransportType::Stdio).await.unwrap();
    assert_eq!(incoming.transport_type(), TransportType::Stdio);
    
    // Test outgoing creation
    let outgoing = factory.create_outgoing(
        TransportType::StreamableHttp,
        "http://localhost:8080"
    ).await.unwrap();
    assert_eq!(outgoing.transport_type(), TransportType::StreamableHttp);
}

#[tokio::test]
async fn test_connection_pooling() {
    let factory = TransportFactory::new(TransportConfig::default());
    
    // Create first connection
    let conn1 = factory.create_outgoing(
        TransportType::Stdio,
        "echo"
    ).await.unwrap();
    
    // Return to pool
    drop(conn1);
    
    // Should get pooled connection
    let conn2 = factory.create_outgoing(
        TransportType::Stdio,
        "echo"
    ).await.unwrap();
    
    // Verify it's from pool (would need instrumentation)
}

#[tokio::test] 
async fn test_buffer_pool_management() {
    let factory = TransportBuilder::new()
        .stdio_buffer_size(8192)
        .http_buffer_size(16384)
        .build();
    
    // Create multiple transports
    let _t1 = factory.create_incoming(TransportType::Stdio).await.unwrap();
    let _t2 = factory.create_incoming(TransportType::StreamableHttp).await.unwrap();
    
    // Verify buffer pools are shared (would need metrics)
}
```

### Step 4: Integration Tests (30 min)

Following implementation-roadmap.md lines 497-503:

```rust
// tests/integration_transport_layer.rs

#[tokio::test]
async fn test_end_to_end_with_factory() {
    // Create factory
    let factory = TransportFactory::new(TransportConfig::default());
    
    // Create transports
    let mut incoming = factory.create_incoming(TransportType::Stdio).await.unwrap();
    let mut outgoing = factory.create_outgoing(
        TransportType::Stdio,
        "echo"
    ).await.unwrap();
    
    // Connect
    incoming.accept().await.unwrap();
    outgoing.connect().await.unwrap();
    
    // Send message
    let request = MessageEnvelope::new(
        ProtocolMessage::new_request("1", "test", json!({})),
        SessionId::new(),
    );
    outgoing.send_request(request.clone()).await.unwrap();
    
    // Receive response
    let response = outgoing.receive_response().await.unwrap();
    assert_eq!(response.message.id(), request.message.id());
}

#[tokio::test]
async fn test_forward_proxy_with_refactored_transports() {
    // Test that forward proxy still works with refactored transports
    let factory = TransportFactory::new(TransportConfig::default());
    
    let client = factory.create_incoming(TransportType::Stdio).await.unwrap();
    let server = factory.create_outgoing(
        TransportType::Stdio,
        "test-server"
    ).await.unwrap();
    
    let mut proxy = ForwardProxy::new(/* config */);
    proxy.start(client, server).await.unwrap();
    
    // Proxy should work exactly as before
}

#[tokio::test]
async fn test_reverse_proxy_uses_factory() {
    // Verify reverse proxy can use factory
    // (This is prep work for Phase 3)
    
    let factory = TransportFactory::new(TransportConfig::default());
    
    // Should be able to create transports for reverse proxy
    let _incoming = factory.create_incoming(
        TransportType::StreamableHttp
    ).await.unwrap();
}
```

### Step 5: Performance Validation (25 min)

Following implementation-roadmap.md lines 506-516:

```rust
// benches/transport_performance.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_transport_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("raw_stdio_send", |b| {
        b.to_async(&rt).iter(|| async {
            let mut core = StdioCore::new(Arc::new(BytesPool::default()));
            core.send_bytes(black_box(b"test data")).await.unwrap()
        });
    });
    
    c.bench_function("factory_create_transport", |b| {
        b.to_async(&rt).iter(|| async {
            let factory = TransportFactory::new(TransportConfig::default());
            factory.create_incoming(black_box(TransportType::Stdio)).await.unwrap()
        });
    });
    
    c.bench_function("directional_message_flow", |b| {
        b.to_async(&rt).iter(|| async {
            // Benchmark complete message flow
            let factory = TransportFactory::new(TransportConfig::default());
            let mut transport = factory.create_outgoing(
                TransportType::Stdio,
                "echo"
            ).await.unwrap();
            
            let msg = MessageEnvelope::new(
                ProtocolMessage::new_request("1", "test", json!({})),
                SessionId::new(),
            );
            
            transport.send_request(black_box(msg)).await.unwrap();
        });
    });
}

criterion_group!(benches, benchmark_transport_operations);
criterion_main!(benches);
```

Run and compare:

```bash
# Baseline before refactor
git checkout main
cargo bench --bench transport_performance > baseline.txt

# After refactor  
git checkout refactor/phase-c
cargo bench --bench transport_performance > after.txt

# Compare - should see <5% regression
diff baseline.txt after.txt
```

### Step 6: Validate Code Reduction (10 min)

Verify we've eliminated duplication:

```bash
# Count lines before/after
echo "Before refactor:"
git checkout main
find src/transport -name "*.rs" | xargs wc -l

echo "After refactor:"
git checkout refactor/phase-c
find src/transport -name "*.rs" | xargs wc -l

# Should see significant reduction in directional/ modules
# And new code in raw/ modules

# Check for duplication
echo "Checking for duplicate I/O implementations:"
rg "AsyncWriteExt|AsyncBufReadExt" src/transport/directional/ -A 5

# Should only see imports and delegation, not implementations
```

## Success Criteria

From implementation-roadmap.md lines 567-575:
- [ ] Shared transport implementations created
- [ ] Directional transports refactored to use shared logic
- [ ] Transport factory operational
- [ ] Code duplication reduced by >50%
- [ ] Performance benchmarks show <5% overhead
- [ ] All existing tests pass (873+ tests)
- [ ] New tests for refactored components

## Test Commands

```bash
# Run all tests
cargo test

# Run transport-specific tests
cargo test transport::

# Run new unit tests
cargo test --test unit_raw_transport_test
cargo test --test unit_shared_transport_test
cargo test --test unit_transport_factory_test

# Run integration tests
cargo test --test integration_transport_layer

# Run benchmarks
cargo bench --bench transport_performance

# Check code coverage
cargo tarpaulin --out Html --output-dir coverage/

# Verify no clippy warnings
cargo clippy --all-targets -- -D warnings
```

## Duration Estimate
**Total: 2 hours**
- Raw primitive tests: 20 min
- Shared logic tests: 20 min
- Factory tests: 25 min
- Integration tests: 30 min
- Performance validation: 25 min
- Code reduction verification: 10 min

## Notes
- Focus on validating the refactoring worked correctly
- Ensure no behavioral changes
- Performance should be same or better
- Code should be significantly cleaner
- This completes Phase 2 of the implementation roadmap

---

**Task Status**: Ready for implementation
**References**: analysis/implementation-roadmap.md (Phase 2 validation)
**Risk Level**: Low - Testing and validation only