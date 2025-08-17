# Task B.1: Wire Transport Tracker to Proxy

## Objective

Modify the reverse proxy's SSE resilience manager to use the transport layer's EventTracker as the single source of truth, eliminating duplicate tracker creation.

## Background

The `ReverseProxySseManager` currently creates its own HashMap of EventTrackers per session, but the transport layer already has a mature EventTracker implementation. This duplication causes:
- Multiple sources of truth for event IDs
- Potential synchronization issues
- Unnecessary memory overhead
- Confusion about which tracker is authoritative

## Key Questions to Answer

1. How does the transport EventTracker get created and passed around?
2. Can we share a single tracker instance safely across threads?
3. What's the lifecycle of tracker vs session vs connection?
4. How do we handle tracker cleanup when sessions end?

## Step-by-Step Process

### 1. Analysis Phase (15 min)
Understand current tracker creation and usage

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Find where transport EventTracker is created
grep -n "EventTracker::new" src/transport/sse/*.rs

# See how ReverseProxySseManager creates trackers
grep -n "EventTracker" src/proxy/reverse/sse_resilience.rs

# Check if transport tracker is accessible
grep -n "pub.*EventTracker" src/transport/sse/reconnect.rs
```

### 2. Design Phase (15 min)
Decide on integration approach

Key design considerations:
- Transport owns the EventTracker
- Proxy gets reference to transport's tracker
- Use Arc for thread-safe sharing
- Tracker lifecycle tied to stream, not session

### 3. Implementation Phase (30 min)

#### 3.1 Remove Duplicate Tracker Storage
```rust
// In src/proxy/reverse/sse_resilience.rs
// REMOVE this field:
// session_trackers: Arc<RwLock<HashMap<SessionId, Arc<EventTracker>>>>,

// KEEP the ReconnectionManager which has its own tracker
reconnection_manager: Arc<ReconnectionManager>,
```

#### 3.2 Update Methods to Use Transport Tracker
```rust
// Instead of creating new tracker:
pub async fn get_event_tracker(&self, session_id: &SessionId) -> Arc<EventTracker> {
    // OLD: Create new tracker
    // NEW: Get from ReconnectionManager or accept as parameter
    self.reconnection_manager.get_tracker() // or similar
}

// Methods that need updating:
// - get_event_tracker()
// - is_duplicate()
// - record_event()
// - get_last_event_id()
// - set_last_event_id()
```

#### 3.3 Wire Transport Tracker Through
```rust
// When creating SSE streams, pass transport's tracker
// In hyper_sse_intercepted.rs or similar:
let tracker = /* get from transport layer */;
sse_manager.with_tracker(tracker);
```

### 4. Testing Phase (15 min)
```bash
# Test that deduplication still works
cargo test test_event_deduplication

# Test session cleanup doesn't break
cargo test test_session_cleanup

# Run clippy
cargo clippy --all-targets -- -D warnings
```

Test cases to verify:
- [ ] Single tracker instance per stream
- [ ] Deduplication works correctly
- [ ] No memory leaks from removed HashMap
- [ ] Thread safety maintained

### 5. Documentation Phase (5 min)
- Update inline comments about tracker ownership
- Document the data flow
- Update analysis doc with implementation

## Expected Deliverables

### Modified Files
- `src/proxy/reverse/sse_resilience.rs` - Remove duplicate tracker storage
- `src/proxy/reverse/hyper_sse_intercepted.rs` - Use transport tracker
- `src/proxy/reverse/legacy.rs` - Pass tracker through if needed

### Tests
- Existing tests should still pass
- No new tests needed for this refactor

### Documentation
- Updated comments explaining tracker ownership
- Clear TODOs for Phase C cleanup

## Success Criteria Checklist

- [ ] No more HashMap of trackers in ReverseProxySseManager
- [ ] Transport EventTracker used consistently
- [ ] All SSE tests passing
- [ ] No clippy warnings
- [ ] No duplicate EventTracker instances created
- [ ] Thread safety maintained with Arc
- [ ] Memory usage reduced (no HashMap overhead)
- [ ] Tracker updated with completion status

## Risk Assessment

| Risk | Impact | Mitigation | 
|------|--------|------------|
| Breaking deduplication | HIGH | Test thoroughly with duplicate events |
| Thread safety issues | MEDIUM | Use Arc consistently |
| Lifecycle mismatch | LOW | Document tracker ownership clearly |

## Duration Estimate

**Total: 1 hour**
- Analysis: 15 minutes
- Design: 15 minutes
- Implementation: 30 minutes
- Testing: 15 minutes
- Documentation: 5 minutes

## Dependencies

- Understanding of transport EventTracker API
- Access to ReconnectionManager's tracker

## Integration Points

- **Transport Layer**: Source of EventTracker
- **Reverse Proxy**: Consumer of tracker
- **Session Store**: Will be updated in B.2

## Performance Considerations

- Reduced memory: No HashMap of trackers
- Better cache locality: Single tracker instance
- No synchronization overhead: One source of truth

## Notes

- This is a minimal change for Option A approach
- Full cleanup happens in Phase C
- Focus on making it work, not perfect architecture
- Keep backward compatibility where possible

## Commands Reference

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Build and test
cargo build --release
cargo test sse

# Run reverse proxy to test
./target/release/shadowcat reverse --bind 127.0.0.1:8080 --upstream http://localhost:3000/mcp

# Validation
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Follow-up Tasks

After completing this task:
- B.2: Connect session persistence
- B.3: Test SSE resilience
- Phase C: Full cleanup of redundant code

---

**Task Status**: ⬜ Not Started
**Created**: 2025-08-17
**Last Modified**: 2025-08-17
**Author**: Claude