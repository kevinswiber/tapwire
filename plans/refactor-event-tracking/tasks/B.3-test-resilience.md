# Task B.3: Test SSE Resilience

## Objective

Validate that the consolidated event tracking system works correctly with SSE resilience features, including deduplication, reconnection, and Last-Event-Id persistence.

## Background

After wiring the transport EventTracker to the proxy (B.1) and connecting session persistence (B.2), we need to verify that:
- SSE reconnection works with Last-Event-Id
- Event deduplication prevents duplicates after reconnection
- Session persistence survives proxy restarts
- MCP Inspector can maintain connections through failures

## Key Questions to Answer

1. Does deduplication work after reconnection?
2. Is Last-Event-Id correctly sent on reconnection?
3. Does session persistence survive restarts?
4. Can MCP Inspector reconnect successfully?

## Step-by-Step Process

### 1. Setup Phase (10 min)
Prepare test environment

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Build reverse proxy with changes
cargo build --release

# Start a test SSE server (if needed)
# Or use MCP example server with SSE support

# Prepare MCP Inspector for testing
```

### 2. Manual Testing Phase (30 min)

#### 2.1 Test Basic SSE Streaming
```bash
# Terminal 1: Start reverse proxy
./target/release/shadowcat reverse \
    --bind 127.0.0.1:8080 \
    --upstream http://localhost:3000/mcp

# Terminal 2: Connect MCP Inspector
# Configure to use http://localhost:8080
```

Verify:
- [ ] SSE events stream successfully
- [ ] No duplicate events
- [ ] Event IDs tracked correctly

#### 2.2 Test Reconnection
```bash
# Simulate disconnection:
# 1. Kill upstream server
# 2. Restart upstream server
# 3. Observe reconnection

# OR simulate network failure:
# Use iptables or similar to drop packets temporarily
```

Verify:
- [ ] Client reconnects automatically
- [ ] Last-Event-Id header sent
- [ ] No duplicate events after reconnection
- [ ] Events resume from correct position

#### 2.3 Test Session Persistence
```bash
# 1. Start proxy, receive some events
# 2. Note the last event ID
# 3. Stop proxy (Ctrl+C)
# 4. Start proxy again
# 5. Check session has last_event_id preserved
```

Verify:
- [ ] Session survives restart
- [ ] Last-Event-Id preserved
- [ ] Can resume from saved position

### 3. Automated Testing Phase (15 min)

#### 3.1 Create Integration Test
```rust
// In tests/integration_event_tracking.rs
#[tokio::test]
async fn test_event_tracking_consolidation() {
    // Setup proxy with SSE
    let proxy = setup_test_proxy().await;
    
    // Send events with IDs
    let events = vec![
        SseEvent::new("data1").with_id("1"),
        SseEvent::new("data2").with_id("2"),
        SseEvent::new("data3").with_id("3"),
    ];
    
    // Process events
    for event in &events {
        proxy.process_event(event).await;
    }
    
    // Verify tracking
    assert_eq!(proxy.get_last_event_id().await, Some("3".to_string()));
    
    // Test deduplication
    proxy.process_event(&events[1]).await; // Duplicate
    assert_eq!(proxy.events_processed(), 3); // Still 3, not 4
}
```

#### 3.2 Test Reconnection Scenario
```rust
#[tokio::test]
async fn test_reconnection_with_last_event_id() {
    // Setup and process initial events
    let mut connection = create_sse_connection().await;
    connection.process_events(&["1", "2", "3"]).await;
    
    // Simulate disconnect
    connection.disconnect().await;
    
    // Reconnect with Last-Event-Id
    let new_connection = connection.reconnect().await;
    assert_eq!(new_connection.last_event_id_header(), Some("3"));
    
    // Verify no duplicates
    new_connection.process_events(&["2", "3", "4"]).await;
    assert_eq!(new_connection.unique_events(), vec!["1", "2", "3", "4"]);
}
```

### 4. Performance Testing Phase (10 min)
```bash
# Measure deduplication overhead
cargo bench event_deduplication

# Test with high event rate
# Generate 1000 events/second and verify no memory leaks

# Profile memory usage
valgrind --leak-check=full ./target/release/shadowcat reverse ...
```

Verify:
- [ ] < 1ms deduplication overhead
- [ ] Memory usage stable
- [ ] No memory leaks

### 5. Documentation Phase (5 min)
- Document test results
- Update architecture notes
- Create troubleshooting guide

## Expected Deliverables

### New Files
- `tests/integration_event_tracking.rs` - Integration tests

### Test Results
- Manual test checklist completed
- Automated tests passing
- Performance benchmarks recorded

### Documentation
- Test results summary
- Known issues or limitations
- Troubleshooting guide

## Success Criteria Checklist

- [ ] SSE streaming works with consolidated tracker
- [ ] Deduplication prevents duplicate events
- [ ] Reconnection sends Last-Event-Id header
- [ ] No events lost or duplicated after reconnection
- [ ] Session persistence survives restarts
- [ ] MCP Inspector works correctly
- [ ] All automated tests passing
- [ ] Performance within targets
- [ ] No memory leaks

## Risk Assessment

| Risk | Impact | Mitigation | 
|------|--------|------------|
| Test environment issues | MEDIUM | Use docker for consistent setup |
| MCP Inspector compatibility | HIGH | Test with multiple versions |
| Edge cases missed | LOW | Comprehensive test scenarios |

## Duration Estimate

**Total: 1 hour**
- Setup: 10 minutes
- Manual testing: 30 minutes
- Automated testing: 15 minutes
- Performance testing: 10 minutes
- Documentation: 5 minutes

## Dependencies

- B.1 and B.2 must be complete
- Test SSE server available
- MCP Inspector installed

## Test Scenarios

### Scenario 1: Happy Path
1. Connect client
2. Stream events
3. Verify tracking

### Scenario 2: Disconnection
1. Stream events
2. Disconnect
3. Reconnect
4. Verify no duplicates

### Scenario 3: Proxy Restart
1. Stream events
2. Restart proxy
3. Verify persistence

### Scenario 4: Concurrent Connections
1. Multiple clients
2. Each gets unique tracker
3. No interference

## Notes

- Focus on real-world scenarios
- Test with actual MCP Inspector
- Document any quirks or workarounds
- Consider edge cases like empty event IDs

## Commands Reference

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Run all SSE tests
cargo test sse

# Run specific integration test
cargo test test_event_tracking_consolidation

# Test with real server
./target/release/shadowcat reverse \
    --bind 127.0.0.1:8080 \
    --upstream http://localhost:3000/mcp

# Monitor memory usage
htop -p $(pgrep shadowcat)

# Check for memory leaks
valgrind --leak-check=full --show-leak-kinds=all \
    ./target/release/shadowcat reverse ...
```

## Test Output Example

```
Running 5 tests
test test_basic_streaming ... ok
test test_deduplication ... ok
test test_reconnection ... ok
test test_persistence ... ok
test test_concurrent ... ok

Test result: ok. 5 passed; 0 failed
```

## Follow-up Tasks

After successful testing:
- Resume reverse proxy SSE resilience integration
- Plan Phase C for removing redundant code
- Consider Redis backend testing

---

**Task Status**: ⬜ Not Started
**Created**: 2025-08-17
**Last Modified**: 2025-08-17
**Author**: Claude