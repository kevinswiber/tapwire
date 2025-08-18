# 🔴 CRITICAL: Event Tracking System - Phase E FINAL

## Critical Issues + Solution Approach

**Architecture review revealed SEVERE performance problems. GPT-5 review found additional CRITICAL BUGS. Final approach uses channels instead of callbacks for simplicity.**

### Critical Issues to Fix
1. **Task Explosion**: 1000 tasks/sec = system instability  
2. **Silent Failures**: All errors ignored = data loss
3. **Memory Inefficiency**: 20KB per session (should be < 5KB)
4. **Retry Queue Bug**: VecDeque not sorted, assumes ordering that doesn't exist
5. **No Coalescing**: 10x write amplification under burst loads

### Solution: Channel-Based EventTracker (No Callbacks!)

**Key Insight**: EventTracker should own the persistence channel sender directly, providing natural backpressure through standard Rust channel semantics.

```rust
pub struct EventTracker {
    session_id: SessionId,  // Knows its session
    persistence_tx: Option<mpsc::Sender<PersistenceRequest>>,  // Direct ownership!
    // ... other fields
}
```

Benefits:
- Idiomatic Rust (channels are standard async pattern)
- Natural backpressure via channel blocking
- Simpler than callbacks
- Consistent with rest of Shadowcat codebase
- Easier to test

## Your Mission: Execute Phase E (6.5 hours)

**THIS MUST BE COMPLETED BEFORE ANY PRODUCTION DEPLOYMENT**

### Task E.1: Worker Pattern with Channel-Based EventTracker (3 hours) - 🔴 CRITICAL

Read: `plans/refactor-event-tracking/tasks/E.1-worker-pattern-final-v2.md`

Implement worker with ALL critical fixes:
1. **EventTracker owns channel** - Mandatory sender, not Option
2. **Short timeout (100ms)** - Prevents SSE stalls
3. **Latest-only buffer** - tokio::sync::watch for newest event
4. **O(1) duplicate detection** - HashSet instead of VecDeque
5. **BinaryHeap for retries** - Proper time-based ordering
6. **recv_many batching** - Atomic, efficient batch reads
7. **Coalescing everywhere** - In worker AND retries
8. **Skip interval bursts** - MissedTickBehavior::Skip
9. **SSE heartbeats** - Prevent connection timeouts
10. **Comprehensive metrics** - Full observability

Key changes:
- EventTracker includes session_id field
- EventTracker has MANDATORY persistence_tx (not Option)
- Latest-only buffer using tokio::sync::watch
- HashSet for O(1) duplicate detection
- SSE heartbeats every 15 seconds
- No callbacks anywhere!

Key code location: `src/session/manager.rs:961-974`

### Task E.2: Fix Activity Tracking (1.5 hours) - 🟡 HIGH

Read: `plans/refactor-event-tracking/tasks/E.2-fix-activity-tracking.md`

Eliminate task spawning for activity updates:
1. Extend worker pattern from E.1
2. Batch activity updates with HashSet deduplication
3. Use channels instead of spawning tasks
4. Add activity-specific metrics

Key code location: `src/transport/sse/session.rs:377-398`

### Task E.3: Optimize Memory Usage (2 hours) - 🟡 HIGH

Read: `plans/refactor-event-tracking/tasks/E.3-optimize-memory-usage.md`

Reduce memory overhead by 75%:
1. Switch from String to Arc<str> for event IDs
2. Implement string interning with LRU cache
3. Add LRU eviction for sessions
4. Comprehensive memory metrics

Target: < 5KB per session (from 20KB)

## Implementation Notes

### Storage Strategy
- **Use in-memory store** - Already implemented
- **No SQLite** - Skip database complexity
- **Redis later** - Future enhancement

### Why No Callbacks?
- Not idiomatic Rust
- Would be only callback in entire codebase
- Channels provide same functionality more simply
- Easier to test and reason about

## Testing Commands

```bash
# After E.1 - Test critical fixes
cargo test test_event_tracker_channel_backpressure
cargo test test_persistence_worker
cargo test test_retry_heap_ordering
cargo test test_event_coalescing
cargo bench event_persistence

# After E.2 - Test activity tracking
cargo test session::activity_batching

# After E.3 - Test memory optimization
cargo test session::memory_usage
cargo bench memory_overhead

# Final load test (MUST PASS)
cargo test test_load_1000_events_per_second -- --nocapture
```

## Critical Test Cases

```rust
// EventTracker must apply backpressure via channel
#[tokio::test]
async fn test_channel_backpressure() {
    let (tx, rx) = mpsc::channel(1);
    tx.send(dummy_request()).await.unwrap(); // Fill
    
    let tracker = EventTracker::new("session-1", 100)
        .with_persistence(tx);
    
    // Should timeout due to full channel
    let start = Instant::now();
    tracker.record_event(&event).await.unwrap();
    assert!(start.elapsed() >= Duration::from_millis(90));
}

// BinaryHeap must maintain time ordering
#[tokio::test]
async fn test_retry_ordering() {
    // Add in random order, pop in time order
    assert_eq!(heap.pop().unwrap().event_id, "first");
    assert_eq!(heap.pop().unwrap().event_id, "second");
}

// Coalescing must reduce writes
#[tokio::test]
async fn test_coalescing() {
    // 3 events for same session -> 1 write
    assert_eq!(coalesced.len(), 1);
    assert_eq!(coalesced[0].event_id, "latest");
}
```

## Success Metrics

### Before Phase E (CURRENT - BROKEN)
- Task spawn rate: 1000+/second 🔴
- Backpressure: NONE 🔴
- Retry ordering: BROKEN (unsorted) 🔴
- Write amplification: 10x (no coalescing) 🔴
- Memory per session: 20KB 🟡

### After Phase E (TARGET)
- Task spawn rate: < 1/second ✅
- Backpressure: Natural via channels ✅
- Retry ordering: Guaranteed by BinaryHeap ✅
- Write reduction: 2-10x via coalescing ✅
- Memory per session: < 5KB ✅
- Full metrics/monitoring ✅

## Implementation Order

1. **E.1** - Worker with channel-based EventTracker (3 hours) - MOST CRITICAL
2. **E.2** - Fix activity tracking (1.5 hours)
3. **E.3** - Memory optimization (2 hours)

Total: 6.5 hours (reduced from 7.5 by removing callback complexity)

## Key Documents

- **Final Approach**: `plans/refactor-event-tracking/analysis/revised-channel-based-approach.md`
- **GPT-5 Review**: `plans/refactor-event-tracking/analysis/gpt5-worker-review-and-revised-plan.md`
- **Critical Analysis**: `plans/refactor-event-tracking/analysis/critical-architecture-review.md`
- **Tracker**: `plans/refactor-event-tracking/refactor-event-tracking-tracker.md`

## Production Readiness Checklist

**DO NOT DEPLOY UNTIL ALL ITEMS ARE COMPLETE:**

- [ ] E.1: EventTracker with channel ownership implemented
- [ ] Backpressure verified (producers block on full channel)
- [ ] Retry ordering verified (BinaryHeap time-based)
- [ ] Coalescing verified (>2x write reduction)
- [ ] Task spawn rate < 10/second
- [ ] Memory usage < 5KB/session
- [ ] Load test passes (1000 events/sec)
- [ ] All metrics exposed and integrated

## Why This Approach Is Better

**Simpler**: No callback traits, no async closures, just channels
**Idiomatic**: Channels are THE Rust async communication pattern
**Consistent**: Matches patterns used throughout Shadowcat
**Testable**: Easy to mock channels and verify behavior
**Efficient**: Same performance, less complexity

## Starting Point

1. Read the channel-based approach document
2. Start with E.1 - implement EventTracker with channel ownership
3. Implement worker with ALL critical fixes (BinaryHeap, coalescing, recv_many)
4. Use `tokio-console` to monitor task spawning
5. Verify each fix with targeted tests

The system is currently **unsuitable for production** and will fail catastrophically without these fixes.