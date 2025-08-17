# D.1: Integration Testing

**Task ID**: D.1  
**Phase**: Testing & Validation  
**Duration**: 1 hour  
**Dependencies**: C.1, C.2  
**Status**: ⬜ Not Started

## Objective

Comprehensive integration testing of the consolidated event tracking system to ensure all components work together correctly.

## Test Scenarios

### 1. End-to-End SSE Reconnection (20 min)

Create an integration test that:
- Establishes SSE connection through reverse proxy
- Receives events with IDs
- Simulates disconnection
- Reconnects with Last-Event-Id header
- Verifies deduplication works
- Confirms resumption from correct point

```rust
#[tokio::test]
async fn test_sse_reconnection_with_deduplication() {
    // Setup reverse proxy with SSE upstream
    let proxy = setup_reverse_proxy_with_sse().await;
    
    // Connect client
    let mut client = connect_sse_client(&proxy.url).await;
    
    // Receive some events
    let event1 = client.next_event().await.unwrap();
    let event2 = client.next_event().await.unwrap();
    assert_eq!(event1.id, Some("1".into()));
    assert_eq!(event2.id, Some("2".into()));
    
    // Disconnect
    client.disconnect().await;
    
    // Reconnect with Last-Event-Id
    let mut client = reconnect_sse_client(&proxy.url, "2").await;
    
    // Should not receive events 1 or 2 (duplicates)
    let event3 = client.next_event().await.unwrap();
    assert_eq!(event3.id, Some("3".into()));
}
```

### 2. Multi-Connection Session Test (15 min)

Test multiple connections sharing the same session:

```rust
#[tokio::test]
async fn test_multiple_connections_shared_tracker() {
    let manager = SessionManager::new();
    let session_id = SessionId::new();
    
    // Create two trackers for same session
    let tracker1 = manager.create_event_tracker(session_id.clone());
    let tracker2 = manager.create_event_tracker(session_id.clone());
    
    // Record event on tracker1
    let event = SseEvent::new("data").with_id("shared-1");
    tracker1.record_event(&event).await;
    
    // Both trackers should see it as duplicate
    assert!(tracker1.is_duplicate("shared-1").await);
    assert!(tracker2.is_duplicate("shared-1").await);
}
```

### 3. Persistence Recovery Test (15 min)

Test that event IDs persist and recover:

```rust
#[tokio::test]
async fn test_event_id_persistence_recovery() {
    let manager = SessionManager::new();
    let session_id = SessionId::new();
    
    // Create session and tracker
    manager.create_session(session_id.clone(), TransportType::Http).await?;
    let tracker = manager.create_event_tracker(session_id.clone());
    
    // Record events
    for i in 1..=5 {
        let event = SseEvent::new("data").with_id(format!("persist-{i}"));
        tracker.record_event(&event).await;
    }
    
    // Wait for async persistence
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Simulate crash - drop tracker
    drop(tracker);
    
    // Recover last event ID from store
    let recovered = manager.get_last_event_id(&session_id).await;
    assert_eq!(recovered, Some("persist-5".to_string()));
    
    // New tracker should have the history
    let new_tracker = manager.create_event_tracker(session_id.clone());
    assert_eq!(new_tracker.get_last_event_id().await, Some("persist-5".to_string()));
}
```

### 4. Performance Test (10 min)

Verify no significant performance degradation:

```rust
#[tokio::test]
async fn test_event_tracking_performance() {
    let tracker = EventTracker::new(1000);
    let start = Instant::now();
    
    // Process 10,000 events
    for i in 0..10_000 {
        let event = SseEvent::new("data").with_id(format!("{i}"));
        tracker.record_event(&event).await;
    }
    
    let elapsed = start.elapsed();
    
    // Should process 10k events in under 100ms
    assert!(elapsed < Duration::from_millis(100));
    
    // Deduplication should still work
    assert!(tracker.is_duplicate("9999").await);
}
```

## Manual Testing with MCP Inspector

If MCP Inspector is available:

1. Start reverse proxy with SSE upstream
2. Connect MCP Inspector as client
3. Send several MCP messages
4. Kill the connection
5. Reconnect Inspector
6. Verify no duplicate messages received
7. Verify session continues from last point

## Success Criteria

- [ ] All integration tests pass
- [ ] SSE reconnection works with deduplication
- [ ] Multiple connections share tracker state
- [ ] Event IDs persist across restarts
- [ ] Performance meets targets (<5% overhead)
- [ ] Manual testing with Inspector successful

## Commands

```bash
# Run integration tests
cargo test --test integration_event_tracking

# Run with logging for debugging
RUST_LOG=shadowcat=debug cargo test --test integration_event_tracking -- --nocapture

# Run performance tests
cargo test test_performance --release

# Manual test with Inspector (if available)
cargo run -- reverse --bind 127.0.0.1:8080 --upstream http://localhost:3000
```

## Notes

- Integration tests may need mock SSE servers
- Use tokio::time::sleep() to allow async callbacks to complete
- Consider using test fixtures for consistent event streams
- Document any flaky tests for future investigation