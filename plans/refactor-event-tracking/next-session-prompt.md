# Event Tracking Consolidation - Ready for Implementation

## Quick Context

We've completed deep analysis of 5 Last-Event-Id tracking systems in Shadowcat and discovered:
1. **ReverseProxySseManager is DEAD CODE** - never used in production, only in tests
2. **Transport EventTracker is perfect** - already has deduplication, just needs wiring
3. **SessionStore trait already has the methods we need** - `store_last_event_id()` and `get_last_event_id()`
4. **Simple 6-hour fix** - mostly deleting dead code and wiring callbacks

## Your Mission: Execute Phase B (2.5 hours)

Start immediately with implementation - all analysis is complete.

### Task B.1: Delete Dead Code (30 minutes)

Delete the dead ReverseProxySseManager module:

```bash
cd shadowcat
git rm src/proxy/reverse/sse_resilience.rs
# Edit src/proxy/reverse/mod.rs to remove: pub mod sse_resilience;
cargo test  # Ensure nothing breaks
```

### Task B.2: Add EventTracker Callbacks (1 hour)

Modify `src/transport/sse/reconnect.rs`:

```rust
pub struct EventTracker {
    // ... existing fields ...
    // ADD THIS:
    on_new_event: Option<Arc<dyn Fn(&str) + Send + Sync>>,
}

impl EventTracker {
    // ADD THIS METHOD:
    pub fn with_callback<F>(mut self, callback: F) -> Self 
    where 
        F: Fn(&str) + Send + Sync + 'static 
    {
        self.on_new_event = Some(Arc::new(callback));
        self
    }
    
    // MODIFY record_event() to call callback:
    pub fn record_event(&self, event: &SseEvent) -> bool {
        // ... existing dedup logic ...
        if !is_duplicate {
            if let Some(ref callback) = self.on_new_event {
                if let Some(ref id) = event.id {
                    callback(id);
                }
            }
        }
        !is_duplicate
    }
}
```

### Task B.3: Wire to SessionStore (1 hour)

In `src/session/manager.rs`, add:

```rust
impl SessionManager {
    pub fn create_event_tracker(&self, session_id: SessionId) -> Arc<EventTracker> {
        let session_id = session_id.clone();
        let store = self.store.clone(); // Already Arc<dyn SessionStore>!
        
        Arc::new(
            EventTracker::new(1000) // or from config
                .with_callback(move |event_id| {
                    let store = store.clone();
                    let session_id = session_id.clone();
                    let event_id = event_id.to_string();
                    tokio::spawn(async move {
                        // Using EXISTING SessionStore method!
                        let _ = store.store_last_event_id(&session_id, event_id).await;
                    });
                })
        )
    }
    
    pub async fn get_last_event_id(&self, session_id: &SessionId) -> Option<String> {
        self.store.get_last_event_id(session_id).await.ok().flatten()
    }
}
```

## Phase C: Integration (2.5 hours) - If Time Permits

### C.1: Update Reverse Proxy (1.5 hours)
- Modify reverse proxy SSE handler to use `session_manager.create_event_tracker()`
- Handle Last-Event-Id header from client reconnections
- Use tracker for deduplication

### C.2: Remove Redundant Tracking (1 hour)
- Remove `last_event_id` field from `ConnectionInfo` in `src/session/sse_integration.rs`
- Remove related methods
- Update all call sites

## Testing Commands

```bash
# After each task:
cargo test --lib
cargo clippy --all-targets -- -D warnings

# Integration test:
cargo test transport::sse
cargo test session::

# Manual test with MCP Inspector if available
```

## Success Criteria

- [ ] ReverseProxySseManager deleted
- [ ] EventTracker has callback support
- [ ] SessionManager creates trackers with persistence
- [ ] All tests pass
- [ ] No clippy warnings

## Key Files to Reference

- **Analysis**: `plans/refactor-event-tracking/analysis/consolidation-design.md`
- **Task Details**: `plans/refactor-event-tracking/tasks/A.3-revised-implementation-plan.md`
- **Tracker**: `plans/refactor-event-tracking/refactor-event-tracking-tracker.md`

## Important Context

### Why This Matters
- Unblocks reverse proxy SSE resilience 
- Enables distributed session storage (Redis)
- Reduces 5 tracking systems to 1
- Works automatically with future SessionStore implementations

### SessionStore Trait Already Perfect
The existing trait has:
- `async fn store_last_event_id(&self, session_id: &SessionId, event_id: String)`
- `async fn get_last_event_id(&self, session_id: &SessionId) -> Option<String>`

No modifications needed - just use these existing methods!

### Architecture Benefits
- Transport EventTracker: Runtime deduplication
- SessionStore: Persistence abstraction
- Clean separation of concerns
- Zero changes needed for new storage backends

## If You Complete Early

Move on to Phase C tasks or:
1. Write integration tests for the new event tracking
2. Test with MCP Inspector if available
3. Document the new architecture in code comments

## Notes for Next Session

- Total remaining work: 6 hours (Phase B: 2.5h, Phase C: 2.5h, Phase D: 1h)
- This unblocks the reverse proxy refactor
- Delete first, then add callbacks, then wire persistence
- Keep it simple - the abstractions are already perfect