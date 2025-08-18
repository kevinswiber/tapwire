# ✅ UNBLOCKED - Ready to Complete SSE Integration

**UPDATE (2025-08-18)**: Event tracking refactor is COMPLETE. Work can proceed immediately.

## Critical Architecture Changes

### What's Different Now

1. **ReverseProxySseManager is GONE** 
   - Was dead code, never used in production
   - Deleted during event tracking refactor
   - Replace with direct SessionManager integration

2. **SessionManager Has SSE Support**
   - New method: `create_event_tracker(session_id)`
   - Automatic persistence via PersistenceWorker
   - No callbacks needed - all channel-based

3. **Simpler Integration Path**
   - No complex wiring required
   - SessionManager → EventTracker → Done
   - ~2 hours instead of original estimate

## Immediate Tasks: Wire Up SSE Resilience (2 hours)

### Task 1: Integrate SessionManager with Reverse Proxy (30 min)

```rust
// In shadowcat/src/proxy/reverse/legacy.rs

struct ReverseProxyServer {
    session_manager: Arc<SessionManager>,  // Already exists!
    // Remove any ReverseProxySseManager references
}

// In handle_mcp_request around line 1300
if is_sse_response {
    // Create event tracker for this session
    let event_tracker = app_state.session_manager
        .create_event_tracker(session_id.clone())
        .await;
    
    // Check for client reconnection
    if let Some(last_event_id) = headers.get("last-event-id") {
        event_tracker.set_last_event_id(last_event_id.to_string()).await;
    }
    
    // Continue with streaming...
}
```

### Task 2: Update SSE Streaming Loop (45 min)

```rust
// In shadowcat/src/proxy/reverse/hyper_sse_intercepted.rs

// During SSE event processing
while let Some(event) = event_stream.next().await {
    // Record event for deduplication and persistence
    if let Some(ref id) = event.id {
        event_tracker.record_event(&event).await?;
    }
    
    // Forward to client
    tx.send(Event::default()
        .id(event.id)
        .data(event.data))
        .await?;
}
```

### Task 3: Handle Upstream Reconnection (30 min)

```rust
// When upstream connection drops
let last_event_id = event_tracker.get_last_event_id().await;

// Add to upstream request headers for reconnection
if let Some(last_id) = last_event_id {
    upstream_headers.insert("Last-Event-Id", last_id.parse()?);
}
```

### Task 4: Clean Up Dead Code (15 min)

```bash
# Remove references to deleted components
grep -r "ReverseProxySseManager" shadowcat/src/
# Should return nothing - if found, remove

# Check for unused SSE modules
grep -r "SessionAwareSseManager" shadowcat/src/proxy/
# If not used in proxy, we might not need it
```

## Testing Plan

### Manual Testing with MCP Inspector
```bash
# Terminal 1: Start proxy
cd shadowcat
cargo build --release
./target/release/shadowcat reverse \
    --bind 127.0.0.1:8080 \
    --upstream http://localhost:3000/mcp

# Terminal 2: Start MCP server with SSE
npx -y @modelcontextprotocol/server-everything

# Terminal 3: Connect Inspector through proxy
# 1. Open MCP Inspector
# 2. Connect to http://localhost:8080
# 3. Trigger SSE events
# 4. Kill upstream server
# 5. Restart upstream
# 6. Verify reconnection with Last-Event-Id
```

### Automated Tests
```bash
# Run existing SSE tests
cargo test transport::sse

# Run reverse proxy tests  
cargo test proxy::reverse

# Integration test
cargo test --test integration_reverse_proxy_sse
```

## Key Files to Modify

1. **shadowcat/src/proxy/reverse/legacy.rs**
   - Line ~1300: SSE response handling
   - Add EventTracker integration
   - Parse Last-Event-Id header

2. **shadowcat/src/proxy/reverse/hyper_sse_intercepted.rs**  
   - SSE streaming loop
   - Add event recording
   - Handle reconnection

3. **shadowcat/src/proxy/reverse/mod.rs**
   - Remove any ReverseProxySseManager exports
   - Clean up imports

## Architecture After Integration

```
ReverseProxyServer
    ├── session_manager: Arc<SessionManager>
    │   ├── create_event_tracker() → EventTracker
    │   └── PersistenceWorker (background)
    │
    ├── handle_mcp_request()
    │   ├── Detect SSE (Accept header)
    │   ├── Create EventTracker
    │   ├── Parse Last-Event-Id
    │   └── Stream with deduplication
    │
    └── No ReverseProxySseManager needed!
```

## Questions Already Resolved

1. **Do we need SessionAwareSseManager?** 
   - NO - Direct SessionManager is simpler
   - Can remove if unused after integration

2. **How to handle persistence?**
   - Automatic via PersistenceWorker
   - No manual persistence needed

3. **What about multiple upstreams?**
   - Create EventTracker per upstream connection
   - SessionManager handles multiple trackers

## Success Criteria

- [ ] SSE streams work through proxy
- [ ] Client reconnection with Last-Event-Id works
- [ ] Upstream reconnection resumes from last event
- [ ] No duplicate events after reconnection
- [ ] Memory usage bounded (~60KB per session)
- [ ] No task explosion (1 worker total)

## Estimated Time: 2 hours

Much simpler than originally planned thanks to the completed event tracking refactor!