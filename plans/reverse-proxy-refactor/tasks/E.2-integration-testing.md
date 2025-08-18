# Task E.2: Integration Testing for Refactored Components

## Objective
Verify that the refactored reverse proxy components work correctly together, especially the new EventTracker integration and hyper modules.

## Context
- EventTracker refactor is complete (via `refactor-event-tracking` plan)
- Hyper modules exist but need integration testing
- block_on fix must be validated under load
- SSE resilience needs end-to-end verification

## Test Scenarios

### 1. SSE Resilience with EventTracker
Test that the new EventTracker properly handles:
- Client reconnection with Last-Event-Id
- Event deduplication across reconnects
- Persistence of event IDs
- Memory bounds under sustained streaming

### 2. Concurrent Connection Handling
After fixing block_on:
- 100 concurrent SSE streams
- 500 concurrent JSON requests
- Mixed SSE and JSON traffic
- Verify no thread blocking or deadlocks

### 3. Custom SessionStore Integration
- Create mock SessionStore implementation
- Verify it receives event ID updates
- Test batch operations work correctly
- Ensure Redis-ready interface

### 4. Interceptor Chain with SSE
- Test interceptors modify SSE events correctly
- Verify no blocking in event processing
- Check pause/resume functionality
- Validate event ordering preserved

### 5. Hyper Module Integration
- JSON responses via json_processing.rs
- SSE streaming via hyper_sse_intercepted.rs
- Raw streaming via hyper_raw_streaming.rs
- Proper content-type routing

## Implementation Steps

### Step 1: Unit Tests for Components
```rust
#[tokio::test]
async fn test_event_tracker_with_persistence() {
    let session_manager = SessionManager::new_in_memory();
    let tracker = session_manager.create_event_tracker(session_id).await;
    
    // Test event recording
    let event = SseEvent::new("data").with_id("123");
    assert!(!tracker.record_event_with_dedup(&event).await);
    
    // Test duplicate detection
    assert!(tracker.record_event_with_dedup(&event).await);
}
```

### Step 2: Integration Tests
Create `tests/integration_reverse_proxy_refactored.rs`:
```rust
#[tokio::test]
async fn test_sse_resilience_end_to_end() {
    // Start reverse proxy
    let proxy = setup_test_proxy().await;
    
    // Connect SSE client
    let mut client = SseClient::connect(&proxy.url("/mcp")).await;
    
    // Receive some events
    let event1 = client.next_event().await;
    assert_eq!(event1.id, Some("session-1-1"));
    
    // Disconnect and reconnect with Last-Event-Id
    client.disconnect();
    client.reconnect_with_last_event_id().await;
    
    // Should not receive duplicate
    let event2 = client.next_event().await;
    assert_ne!(event2.id, event1.id);
}
```

### Step 3: Load Tests
```rust
#[tokio::test]
async fn test_concurrent_sse_streams() {
    let proxy = setup_test_proxy().await;
    
    // Spawn 100 SSE clients
    let mut handles = vec![];
    for i in 0..100 {
        let url = proxy.url("/mcp");
        handles.push(tokio::spawn(async move {
            let client = SseClient::connect(&url).await;
            // Receive events for 10 seconds
            tokio::time::timeout(
                Duration::from_secs(10),
                client.receive_events()
            ).await
        }));
    }
    
    // All should complete without panic
    for handle in handles {
        assert!(handle.await.is_ok());
    }
}
```

### Step 4: Custom Store Test
```rust
#[tokio::test]
async fn test_custom_session_store() {
    // Create custom store that tracks calls
    let store = Arc::new(MockSessionStore::new());
    
    // Build proxy with custom store
    let proxy = ReverseProxyServerBuilder::new(config)
        .with_session_store(store.clone())
        .build()
        .await?;
    
    // Generate SSE traffic
    // ... 
    
    // Verify store was called
    assert!(store.event_ids_stored() > 0);
    assert!(store.batch_operations_used());
}
```

## Files to Create/Modify

### New Test Files
- `tests/integration_reverse_proxy_refactored.rs` - Main integration tests
- `tests/helpers/mock_session_store.rs` - Mock store for testing
- `tests/load/concurrent_connections.rs` - Load testing scenarios

### Existing Files to Update
- `tests/integration_reverse_proxy.rs` - Add new test cases
- `benches/reverse_proxy.rs` - Add benchmarks for refactored code

## Success Criteria
- [ ] All integration tests pass
- [ ] No deadlocks at 100+ connections
- [ ] EventTracker properly deduplicates
- [ ] Custom SessionStore integration works
- [ ] Memory stays bounded under load
- [ ] Performance meets targets (<5% overhead)

## Tools & Commands

### Run Integration Tests
```bash
cargo test --test integration_reverse_proxy_refactored
```

### Run with Logging
```bash
RUST_LOG=shadowcat=debug cargo test test_sse_resilience -- --nocapture
```

### Check for Deadlocks
```bash
# Use tokio-console to monitor runtime
cargo install tokio-console
TOKIO_CONSOLE=1 cargo test test_concurrent_sse_streams
```

### Memory Profiling
```bash
cargo test --release test_memory_bounds
valgrind --leak-check=full target/release/test_memory_bounds
```

## Estimated Time
- Unit tests: 1 hour
- Integration tests: 1 hour  
- Load tests: 30 minutes
- Debugging/fixes: 30 minutes
- **Total**: 3 hours

## Dependencies
- E.0 (block_on fix) must be complete
- E.1 (hyper migration) can be partial

## Risk Mitigation
- Start with simple scenarios
- Add complexity incrementally
- Use timeouts to prevent hanging tests
- Monitor with tokio-console during development