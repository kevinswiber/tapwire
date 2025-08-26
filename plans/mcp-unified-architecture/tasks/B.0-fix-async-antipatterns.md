# B.0: Fix Async Antipatterns

## Objective
Fix critical async antipatterns in the MCP server implementation that could cause deadlocks, busy-waiting, and race conditions.

## Key Issues to Fix
1. Polling channel in loop instead of awaiting
2. Holding locks across await points
3. Using Arc<RwLock<bool>> for simple flags
4. Race conditions in max_clients enforcement
5. Unbounded channels causing memory issues

## Process

### 1. Replace Polling with Select
Current problem:
```rust
// ❌ BAD: Busy-wait pattern
loop {
    if let Ok(conn) = connection_rx.try_recv() {
        // handle
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```

Fix to:
```rust
// ✅ GOOD: Proper async
loop {
    tokio::select! {
        Some(conn) = connection_rx.recv() => {
            // handle
        }
        _ = shutdown.cancelled() => break,
    }
}
```

### 2. Fix Lock-Across-Await
Identify and fix all instances of:
```rust
// ❌ BAD
let mut sessions = self.sessions.write().await;
client.send(msg).await; // Lock still held!
```

Change to:
```rust
// ✅ GOOD
let client = {
    let sessions = self.sessions.read().await;
    sessions.get(id).cloned()
}; // Lock released
client.send(msg).await;
```

### 3. Replace Arc<RwLock<bool>> with CancellationToken
- [ ] Remove shutdown: Arc<RwLock<bool>>
- [ ] Add shutdown: CancellationToken
- [ ] Update all shutdown checks
- [ ] Add child tokens where needed

### 4. Atomic Connection Limits with Semaphore
- [ ] Replace manual counting with Semaphore
- [ ] Ensure atomic acquire/release
- [ ] No race conditions in limit enforcement
- [ ] Proper error on max reached

### 5. Bounded Channels for Backpressure
- [ ] Replace unbounded channels
- [ ] Add reasonable bounds (100-1000)
- [ ] Handle channel full scenarios
- [ ] Add metrics for monitoring

## Implementation Steps

### Phase 1: Server Core
- [ ] Update Server struct fields
- [ ] Fix serve() method
- [ ] Update accept() method
- [ ] Fix shutdown handling

### Phase 2: Session Management
- [ ] Audit all lock usage
- [ ] Fix notify_client()
- [ ] Fix broadcast methods
- [ ] Update session cleanup

### Phase 3: Connection Handling
- [ ] Fix spawn_client_handler
- [ ] Update message loops
- [ ] Proper error propagation
- [ ] Clean disconnection

## Deliverables

### 1. Updated server.rs
- All antipatterns fixed
- Proper async/await usage
- Clean shutdown path
- No deadlock potential

### 2. Test Suite
Location: `tests/async_correctness.rs`
- Test shutdown under load
- Test connection limits
- Test lock contention
- Test backpressure

### 3. Performance Report
Location: `analysis/async-fixes-performance.md`
- Before/after benchmarks
- Memory usage comparison
- Latency improvements
- Throughput changes

## Testing Checklist
```rust
#[tokio::test]
async fn test_no_deadlock_on_shutdown() {
    // Accept many connections
    // Trigger shutdown
    // Verify completes in <5s
}

#[tokio::test] 
async fn test_connection_limit_atomic() {
    // Spawn 1000 concurrent connects
    // Verify exactly N succeed
    // No race conditions
}

#[tokio::test]
async fn test_no_busy_wait() {
    // Monitor CPU usage
    // Should be ~0% when idle
}
```

## Success Criteria
- [ ] No locks held across await
- [ ] No polling loops
- [ ] Atomic connection limits
- [ ] Clean shutdown in <5s
- [ ] CPU usage ~0% when idle
- [ ] All tests passing

## Duration
8 hours

## Dependencies
- A.3: Migration plan complete

## Notes
- This is critical foundation work
- Must be done before hyper integration
- Every change needs testing
- Consider compatibility during migration