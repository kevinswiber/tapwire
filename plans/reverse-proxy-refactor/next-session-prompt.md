# ⏸️ ON HOLD - Blocked by Event Tracking Refactor

**IMPORTANT**: This work is blocked. See [Event Tracking Refactor](../refactor-event-tracking/refactor-event-tracking-tracker.md) which must be completed first (2-3 hours).

---

# Next Session: Complete SSE Resilience Integration

## Context Update (2025-08-17)

### Major Changes Since Last Session
1. **Transport Architecture Refactor COMPLETE** ✅
   - `TransportContext::Sse` replaced with `Delivery::Sse` 
   - Clean transport types: only Stdio and Http
   - SSE metadata properly at message level
   - No more `is_sse_session` code smell

2. **SSE Resilience Module Created but NOT Integrated** ⚠️
   - `ReverseProxySseManager` exists in `src/proxy/reverse/sse_resilience.rs`
   - Has EventTracker, HealthMonitor, ReconnectionManager integration
   - But it's not being used by the reverse proxy at all!

## Immediate Priority: Wire Up SSE Resilience

The foundation exists but needs to be connected. This is a quick win - likely 1-2 hours to complete.

### Task 1: Integrate ReverseProxySseManager (30 min)
```bash
# Files to modify
shadowcat/src/proxy/reverse/legacy.rs  # Add SSE manager field and initialization
shadowcat/src/proxy/reverse/mod.rs     # Export SSE resilience types
```

**Steps:**
1. Add `sse_manager: Arc<ReverseProxySseManager>` to ReverseProxyServer
2. Initialize in ReverseProxyServer::new()
3. Use manager in SSE streaming paths

### Task 2: Use Existing Last-Event-Id Field (15 min)
```bash
# Session ALREADY has last_event_id field!
shadowcat/src/session/store.rs:92  # pub last_event_id: Option<String>
shadowcat/src/session/sse_integration.rs  # Rich per-connection tracking
```

**Steps:**
1. Session already has the field at line 92 ✅
2. Just need to read/write it in proxy flow
3. Consider using SseSessionState for richer tracking

### Task 3: Wire Up Upstream Reconnection (45 min)
```bash
# Files to modify  
shadowcat/src/proxy/reverse/hyper_sse_intercepted.rs  # Use ReconnectingStream
```

**Steps:**
1. Check for SSE disconnection in streaming loop
2. Call reconnection_manager when connection drops
3. Resume from Last-Event-Id

### Task 4: Support Client Reconnection (30 min)
```bash
# Files to modify
shadowcat/src/proxy/reverse/legacy.rs  # Parse Last-Event-Id header
```

**Steps:**
1. Extract Last-Event-Id from client headers
2. Store in session via sse_manager
3. Use for upstream reconnection

### Task 5: Test with MCP Inspector (15 min)
```bash
# Test commands
cd shadowcat
cargo build --release
./target/release/shadowcat reverse --bind 127.0.0.1:8080 --upstream http://localhost:3000/mcp

# In another terminal
# Start MCP Inspector and connect through proxy
# Simulate disconnection and verify reconnection works
```

## Key Integration Points

### In legacy.rs handle_mcp_request():
```rust
// Around line 1300 where SSE streaming happens
if is_sse_response {
    // NEW: Use SSE manager for resilience
    let last_event_id = self.sse_manager.get_last_event_id(&session_id).await;
    
    // NEW: Parse client's Last-Event-Id if reconnecting
    if let Some(client_last_id) = headers.get("last-event-id") {
        self.sse_manager.set_last_event_id(&session_id, client_last_id).await;
    }
    
    // Continue with existing streaming logic but add reconnection
}
```

## Success Criteria
- [ ] SSE manager integrated and initialized
- [ ] Last-Event-Id tracked per session
- [ ] Upstream disconnections trigger reconnection
- [ ] Client reconnections with Last-Event-Id work
- [ ] MCP Inspector maintains connection through failures
- [ ] No duplicate events after reconnection

## Architecture Reminder
We're not reinventing - just connecting existing pieces:
- `ReverseProxySseManager` - Already wraps transport reconnection logic
- `ReconnectionManager` - Handles exponential backoff
- `EventTracker` - Deduplicates events
- `HealthMonitor` - Detects idle connections

The gap is simply that these aren't wired into the proxy flow yet!

## Time Estimate
Total: 2-3 hours to complete integration and test

This is mostly plumbing work - the hard architectural decisions are done.