# Task B.2: Connect Session Persistence

## Objective

Establish a one-way data flow from the transport EventTracker to the Session store, ensuring Last-Event-Id is persisted for recovery while maintaining transport as the authoritative source.

## Background

Currently, the Session struct has a `last_event_id` field, but it's not being updated from the transport layer's EventTracker. We need to:
- Update Session.last_event_id when transport receives events
- Ensure atomic, one-way updates (Transport → Session)
- Maintain persistence for session recovery
- Avoid circular dependencies

## Key Questions to Answer

1. Where in the transport flow should we update the session?
2. How do we get session reference from transport layer?
3. Should updates be synchronous or async?
4. How do we handle session not found errors?

## Step-by-Step Process

### 1. Analysis Phase (10 min)
Find integration points between transport and session

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Find where EventTracker records events
grep -n "record_event" src/transport/sse/reconnect.rs

# Find session update methods
grep -n "last_event_id" src/session/store.rs

# See where session manager is available
grep -n "SessionManager" src/proxy/reverse/*.rs
```

### 2. Design Phase (10 min)
Design the update flow

Key design considerations:
- Update after successful deduplication
- Use session manager for atomic updates
- Handle missing sessions gracefully
- Keep updates async to avoid blocking

### 3. Implementation Phase (30 min)

#### 3.1 Add Update Method to Session
```rust
// In src/session/store.rs
impl Session {
    /// Update last event ID from transport layer
    pub fn update_last_event_id(&mut self, event_id: String) {
        self.last_event_id = Some(event_id);
        self.update_activity();
    }
}
```

#### 3.2 Create Update Hook in Transport
```rust
// In transport handling code (where EventTracker is used)
// After successful event recording:
if let Some(event_id) = event.id.as_ref() {
    // Record in transport tracker
    tracker.record_event(&event).await;
    
    // Update session persistence
    if let Some(session_manager) = &self.session_manager {
        if let Ok(mut session) = session_manager.get_session(&session_id).await {
            session.update_last_event_id(event_id.clone());
            session_manager.update_session(session).await?;
        }
    }
}
```

#### 3.3 Wire Session Manager to Transport
```rust
// In proxy initialization, pass session manager to transport layer
// OR use a callback/channel approach

// Option 1: Direct reference
transport.set_session_manager(Arc::clone(&session_manager));

// Option 2: Callback
transport.on_event_received(move |session_id, event_id| {
    // Update session
});

// Option 3: Channel
let (tx, rx) = mpsc::channel();
// Transport sends updates, session manager receives
```

### 4. Testing Phase (15 min)
```bash
# Test persistence works
cargo test test_session_persistence

# Test concurrent updates
cargo test test_concurrent_event_updates

# Verify no memory leaks
cargo test test_session_cleanup
```

Test cases to verify:
- [ ] Event ID persisted to session
- [ ] Survives session reload
- [ ] Handles missing sessions gracefully
- [ ] Thread-safe updates

### 5. Documentation Phase (5 min)
- Document the one-way data flow
- Add sequence diagram in comments
- Update architecture notes

## Expected Deliverables

### Modified Files
- `src/session/store.rs` - Add update_last_event_id method
- `src/session/memory.rs` - Ensure persistence works
- Transport integration point (varies by approach)

### Tests
- Session persistence test
- Concurrent update test
- Missing session handling test

### Documentation
- Clear comments about data flow direction
- Architecture diagram updated

## Success Criteria Checklist

- [ ] Transport updates session after deduplication
- [ ] One-way flow: Transport → Session only
- [ ] Session.last_event_id persisted correctly
- [ ] Atomic updates (no partial states)
- [ ] Handles missing sessions gracefully
- [ ] No circular dependencies
- [ ] All tests passing
- [ ] No clippy warnings

## Risk Assessment

| Risk | Impact | Mitigation | 
|------|--------|------------|
| Circular dependency | HIGH | Careful layering, one-way flow |
| Race conditions | MEDIUM | Use atomic operations |
| Performance impact | LOW | Async updates, batch if needed |

## Duration Estimate

**Total: 1 hour**
- Analysis: 10 minutes
- Design: 10 minutes
- Implementation: 30 minutes
- Testing: 15 minutes
- Documentation: 5 minutes

## Dependencies

- B.1 must be complete (transport tracker wired)
- Session manager must be accessible
- EventTracker must expose event IDs

## Integration Points

- **Transport Layer**: Source of events
- **Session Manager**: Persistence coordinator
- **Session Store**: Final persistence

## Performance Considerations

- Async updates to avoid blocking
- Batch updates if high frequency
- Consider using channels for decoupling

## Notes

- This establishes the authoritative data flow
- Future Redis backend will use same interface
- Keep updates atomic and idempotent
- Log but don't fail on missing sessions

## Commands Reference

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Test session functionality
cargo test session::

# Test with real SSE stream
cargo run -- forward stdio -- your-sse-server

# Check persistence
# Start proxy, receive events, restart, verify ID persisted

# Validation
cargo clippy --all-targets -- -D warnings
```

## Example Implementation

```rust
// Clean update flow example
pub async fn handle_sse_event(
    event: SseEvent,
    tracker: Arc<EventTracker>,
    session_manager: Arc<SessionManager>,
    session_id: SessionId,
) -> Result<()> {
    // 1. Transport layer deduplication
    if let Some(event_id) = &event.id {
        if !tracker.is_duplicate(event_id).await {
            tracker.record_event(&event).await;
            
            // 2. Update session persistence
            let _ = session_manager
                .update_last_event_id(&session_id, event_id.clone())
                .await
                .map_err(|e| debug!("Session update failed: {}", e));
        }
    }
    
    // 3. Process event normally
    process_event(event).await
}
```

## Follow-up Tasks

After completing this task:
- B.3: Test SSE resilience end-to-end
- Verify with MCP Inspector
- Phase C: Remove redundant systems

---

**Task Status**: ⬜ Not Started
**Created**: 2025-08-17
**Last Modified**: 2025-08-17
**Author**: Claude