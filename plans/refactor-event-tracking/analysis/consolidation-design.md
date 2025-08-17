# Event Tracking Consolidation Design

**Date**: 2025-08-17  
**Analyst**: Claude  
**Status**: Final Design (Revised with SessionStore insights)

## Executive Summary

After deep analysis, the consolidation is simpler than anticipated:
1. **Delete dead code** (ReverseProxySseManager)
2. **Wire existing transport EventTracker** to session persistence via **existing SessionStore trait**
3. **Remove redundant tracking** from ConnectionInfo
4. **No new abstractions needed** - both EventTracker and SessionStore are already well-designed

## Critical Discovery: SessionStore Abstraction Already Exists!

The SessionStore trait already has the perfect abstraction for event ID persistence:
- `store_last_event_id(session_id, event_id)` - async persistence method
- `get_last_event_id(session_id)` - async retrieval method
- Works with InMemory, future Redis, and external session stores
- No modifications needed to the trait!

## Final Architecture

### Simplified Three-Layer Design

```
┌─────────────────────────────────────────────────────────┐
│                   Session Store                          │
│  Purpose: Persistence only (database/Redis ready)        │
│  • Receives updates from Transport via callback          │
│  • Stores last_event_id for recovery                     │
│  • No business logic, just storage                       │
└──────────────────────────┬──────────────────────────────┘
                           │ callback on event
┌──────────────────────────▼──────────────────────────────┐
│              Transport EventTracker                      │
│  Purpose: Single source of truth for tracking            │
│  • Owns all deduplication logic                         │
│  • Maintains circular buffer of seen events             │
│  • Thread-safe with Arc<RwLock>                        │
│  • Notifies session store of updates                    │
└──────────────────────────┬──────────────────────────────┘
                           │ feeds events
┌──────────────────────────▼──────────────────────────────┐
│              SSE Connection Layer                        │
│  Purpose: Wire protocol handling                         │
│  • Reads events from stream                             │
│  • Feeds to EventTracker (no local storage)            │
│  • Manages connection lifecycle                         │
└─────────────────────────────────────────────────────────┘
```

### Data Flow

```
SSE Event arrives → SseConnection reads 
                 → EventTracker.record_event()
                 → If not duplicate:
                     → Callback to SessionStore.update_last_event_id()
                     → Forward event to client
                 → If duplicate:
                     → Drop event (already seen)
```

## API Design

### EventTracker Enhancement
```rust
pub struct EventTracker {
    last_event_id: Arc<RwLock<Option<String>>>,
    seen_events: Arc<RwLock<VecDeque<String>>>,
    max_tracked: usize,
    // NEW: Optional callback for persistence
    on_new_event: Option<Arc<dyn Fn(&str) + Send + Sync>>,
}

impl EventTracker {
    pub fn with_callback<F>(mut self, callback: F) -> Self 
    where 
        F: Fn(&str) + Send + Sync + 'static 
    {
        self.on_new_event = Some(Arc::new(callback));
        self
    }
    
    pub fn record_event(&self, event: &SseEvent) -> bool {
        // ... existing dedup logic ...
        if !is_duplicate {
            if let Some(ref callback) = self.on_new_event {
                callback(&event_id);
            }
        }
        !is_duplicate
    }
}
```

### Session Manager Integration (Using Existing SessionStore Trait!)
```rust
impl SessionManager {
    pub fn create_event_tracker(&self, session_id: SessionId) -> Arc<EventTracker> {
        let session_id_clone = session_id.clone();
        let store = self.store.clone(); // Already Arc<dyn SessionStore>!
        
        Arc::new(
            EventTracker::new(1000)
                .with_callback(move |event_id| {
                    // Fire and forget - persistence is best-effort
                    // Works with ANY SessionStore implementation!
                    let store = store.clone();
                    let session_id = session_id_clone.clone();
                    let event_id = event_id.to_string();
                    tokio::spawn(async move {
                        // Using EXISTING SessionStore trait method!
                        let _ = store.store_last_event_id(&session_id, event_id).await;
                    });
                })
        )
    }
    
    pub async fn get_last_event_id(&self, session_id: &SessionId) -> Option<String> {
        // Using EXISTING SessionStore trait method!
        self.store.get_last_event_id(session_id).await.ok().flatten()
    }
}
```

### Reverse Proxy Integration
```rust
// In reverse proxy SSE handler
let event_tracker = session_manager.create_event_tracker(session_id.clone());

// Use for deduplication on reconnect
if let Some(last_event_id) = request.headers().get("last-event-id") {
    event_tracker.set_last_event_id(last_event_id.to_str()?);
}

// Process events through tracker
while let Some(event) = upstream.next_event().await? {
    if event_tracker.record_event(&event) {
        // Not a duplicate, forward to client
        client.send_event(event).await?;
    }
    // Duplicates are silently dropped
}
```

## Edge Cases Handled

### 1. Multiple Connections Per Session
**Solution**: Each connection gets a reference to the same session EventTracker
```rust
// All connections in a session share the tracker
let tracker = session_trackers.entry(session_id)
    .or_insert_with(|| session_manager.create_event_tracker(session_id));
```

### 2. Session Without Connection
**Solution**: Lazy tracker creation - only create when first connection arrives
```rust
// Don't create tracker until needed
if request.accepts_sse() {
    let tracker = session_manager.create_event_tracker(session_id);
}
```

### 3. Connection Without Session (Forward Proxy)
**Solution**: Forward proxy creates ephemeral tracker without persistence
```rust
// Forward proxy uses tracker without session callback
let tracker = Arc::new(EventTracker::new(1000)); // No callback
```

### 4. Concurrent Event Streams
**Solution**: EventTracker is already thread-safe with Arc<RwLock>
- Multiple streams can safely call record_event()
- RwLock allows concurrent reads of last_event_id

### 5. Tracker Cleanup on Disconnect
**Solution**: Reference counting with Arc handles cleanup
```rust
// When last Arc reference drops, tracker is cleaned up
connection_trackers.remove(&connection_id);
// If this was the last reference, Arc will drop the tracker
```

### 6. Persistence Failures
**Solution**: Fire-and-forget with best-effort semantics
- Persistence callback spawns separate task
- Failures don't block event processing
- Log warnings on persistence errors

### 7. Memory Pressure
**Solution**: Circular buffer with configurable size
- Default 1000 events per tracker
- Oldest events automatically evicted
- Configurable via max_tracked_events

## Performance Considerations

### Memory Usage
- **Per Tracker**: ~24KB (1000 events × 24 bytes average ID)
- **Per Session**: Single tracker shared by all connections
- **Cleanup**: Automatic via Arc reference counting

### Lock Contention
- **Current**: RwLock on each operation
- **Optimization**: Could use lockless ring buffer if needed
- **Mitigation**: Keep critical sections small

### Callback Overhead
- **Async**: Persistence callbacks spawn tasks
- **Non-blocking**: Doesn't delay event forwarding
- **Batching**: Could batch updates every N events

## Migration Plan

### Phase 1: Delete Dead Code (30 minutes)
```bash
# Remove ReverseProxySseManager
git rm src/proxy/reverse/sse_resilience.rs
# Update mod.rs to remove the module
# Run tests to ensure nothing breaks
```

### Phase 2: Add Callback Support (1 hour)
1. Add `on_new_event` field to EventTracker
2. Add `with_callback()` builder method
3. Call callback in `record_event()`
4. Test callback execution

### Phase 3: Wire Session Persistence (1 hour)
1. Add `create_event_tracker()` to SessionManager
2. ~~Update session store trait~~ - Already has `store_last_event_id()`!
3. ~~Implement for InMemorySessionStore~~ - Already implemented!
4. Test persistence updates

### Phase 4: Update Reverse Proxy (2 hours)
1. Get tracker from session manager
2. Use tracker for deduplication
3. Remove any local event tracking
4. Test reconnection scenarios

### Phase 5: Cleanup Redundancy (1 hour)
1. Remove `last_event_id` from ConnectionInfo
2. Update SseSessionState to use shared tracker
3. Remove duplicate tracker creation
4. Run full test suite

## Benefits of Using SessionStore Abstraction

### Future-Proof Design
1. **Automatic Redis Support**: When Redis SessionStore is added, event IDs persist there automatically
2. **External Store Support**: Third-party session stores via API get event tracking for free
3. **No Lock-In**: The abstraction allows any storage backend without code changes
4. **Distributed Ready**: Multi-instance deployments will share event IDs via the store

### Clean Architecture
- **Separation of Concerns**: Transport handles tracking, Store handles persistence
- **Single Responsibility**: EventTracker does deduplication, SessionStore does storage
- **Interface Stability**: Using existing trait methods means no API changes
- **Testing**: Can mock SessionStore for unit tests

## Breaking Changes

### API Changes
- ❌ None for external users
- ❌ None for SessionStore trait (using existing methods!)
- ✅ Internal refactoring only
- ✅ Backward compatible

### Behavior Changes
- ✅ Improved: Deduplication now works in reverse proxy
- ✅ Improved: Session persistence actually updates
- ✅ Improved: Works with any SessionStore implementation
- ✅ Unchanged: Wire protocol and client behavior

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Callback failures block events | Low | High | Use fire-and-forget async tasks |
| Memory leak from trackers | Low | Medium | Arc cleanup + timeout eviction |
| Lock contention | Medium | Low | Keep critical sections minimal |
| Persistence lag | Medium | Low | Best-effort semantics acceptable |

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_event_tracker_callback() {
    let called = Arc::new(AtomicBool::new(false));
    let tracker = EventTracker::new(10)
        .with_callback(move |_| { called.store(true, Ordering::SeqCst); });
    
    let event = SseEvent::new("data").with_id("1");
    tracker.record_event(&event);
    assert!(called.load(Ordering::SeqCst));
}
```

### Integration Tests
1. Test session persistence updates
2. Test reverse proxy deduplication
3. Test multi-connection scenarios
4. Test cleanup on disconnect

### Performance Tests
```rust
#[bench]
fn bench_event_tracking_with_callback(b: &mut Bencher) {
    let tracker = create_tracker_with_callback();
    b.iter(|| {
        let event = create_test_event();
        tracker.record_event(&event);
    });
}
```

## Success Criteria

### Functional
- ✅ Single EventTracker per session
- ✅ Session persistence updates work
- ✅ Reverse proxy deduplication works
- ✅ No duplicate tracking code

### Performance
- ✅ < 1µs overhead per event
- ✅ < 50KB memory per session
- ✅ No blocking operations

### Quality
- ✅ All tests pass
- ✅ No clippy warnings
- ✅ Clear documentation

## Conclusion

The deep analysis revealed a much simpler consolidation path:
1. **Most of the work is already done** - Transport EventTracker is mature
2. **Dead code can be deleted** - ReverseProxySseManager is unused
3. **Simple callbacks solve persistence** - No complex synchronization needed
4. **Reference sharing solves multi-connection** - Arc handles the complexity

**Estimated Time**: 5.5 hours (vs original 8-12 hour estimate)

**Recommendation**: Proceed with implementation immediately. The risk is low and the benefits are clear.