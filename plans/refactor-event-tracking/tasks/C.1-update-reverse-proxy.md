# C.1: Update Reverse Proxy to Use Shared EventTracker

**Task ID**: C.1  
**Phase**: Integration  
**Duration**: 1.5 hours  
**Dependencies**: B.1, B.2, B.3  
**Status**: ⬜ Not Started

## Objective

Update the reverse proxy SSE handler to use the shared EventTracker from SessionManager for deduplication and reconnection support.

## Current State

The reverse proxy currently:
- Has no event tracking or deduplication
- Doesn't handle Last-Event-Id header from client reconnections
- Doesn't persist event IDs for recovery

## Implementation Steps

### 1. Locate Reverse Proxy SSE Handler (30 min)
```bash
# Find SSE handling code in reverse proxy
grep -r "text/event-stream" src/proxy/reverse/
grep -r "SseEvent" src/proxy/reverse/
grep -r "Last-Event-Id" src/proxy/reverse/
```

### 2. Update SSE Stream Handler (45 min)

In the reverse proxy SSE handler:

```rust
// Get or create tracker for session
let tracker = session_manager.create_event_tracker(session_id.clone());

// Handle Last-Event-Id header from client reconnection
if let Some(last_event_id) = headers.get("last-event-id") {
    let event_id = last_event_id.to_str()?.to_string();
    tracker.set_last_event_id(event_id).await;
}

// Process events through tracker for deduplication
while let Some(event) = upstream.next_event().await? {
    // Use the new combined method for efficiency
    if tracker.record_event_with_dedup(&event).await {
        // Not a duplicate, forward to client
        downstream.send_event(event).await?;
    }
    // Duplicates are silently dropped
}
```

### 3. Handle Reconnection Scenarios (15 min)

```rust
// On client reconnection, get stored event ID
let last_event_id = session_manager.get_last_event_id(&session_id).await;

// Include in upstream request if resuming
if let Some(event_id) = last_event_id {
    upstream_headers.insert("Last-Event-Id", event_id.parse()?);
}
```

## Success Criteria

- [ ] Reverse proxy creates EventTracker via SessionManager
- [ ] Last-Event-Id header is read from client requests
- [ ] Events are deduplicated using tracker
- [ ] Event IDs are persisted automatically
- [ ] Reconnection resumes from last event
- [ ] All existing reverse proxy tests pass

## Testing

```bash
# Unit tests
cargo test proxy::reverse

# Integration test with SSE
cargo test --test integration_reverse_proxy

# Manual test with MCP Inspector if available
```

## Files to Modify

- `src/proxy/reverse/legacy.rs` - Main reverse proxy implementation
- `src/proxy/reverse/hyper_sse_intercepted.rs` - SSE interception logic
- Possibly new module for clean SSE handling

## Notes

- The reverse proxy refactor is in progress, so we work with legacy.rs
- Use the new `record_event_with_dedup()` method for efficiency
- Ensure thread-safety with Arc<EventTracker>
- Fire-and-forget persistence is acceptable (best-effort)