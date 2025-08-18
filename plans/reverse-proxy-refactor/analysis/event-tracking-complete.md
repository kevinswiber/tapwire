# Event Tracking Refactor Complete - Integration Guide

## Status: ✅ COMPLETE (2025-08-18)

The event tracking refactor is now complete, unblocking the reverse proxy SSE resilience work.

## What Changed

### Removed Components
- **ReverseProxySseManager** - DELETED (was dead code, never used in production)
- **Callback-based tracking** - REMOVED (replaced with channel-based approach)

### New Architecture

```
SessionManager (owns SSE support)
    ├── create_event_tracker() - Creates EventTracker for SSE sessions
    ├── PersistenceWorker - Single worker for all event persistence
    └── SessionStore - Batch persistence operations

EventTracker (per SSE session)
    ├── Persistence channel - Direct ownership for backpressure
    ├── HashSet deduplication - O(1) duplicate detection
    ├── Latest-only buffer - Prevents blocking
    └── Last-Event-Id tracking - For reconnection

SessionAwareSseManager (transport::sse::session)
    ├── Wraps SseConnectionManager
    ├── Integrates with SessionManager
    ├── Per-session EventTrackers
    └── Lifecycle management
```

## Available SSE Components

### Core Components (Use These)

1. **SessionManager** (`src/session/manager.rs`)
   - Has `create_event_tracker(session_id)` method
   - Automatically handles persistence
   - Thread-safe, production-ready

2. **EventTracker** (`src/transport/sse/reconnect.rs`)
   - Created via SessionManager
   - Handles Last-Event-Id tracking
   - Efficient deduplication
   - Channel-based persistence

3. **ReconnectionManager** (`src/transport/sse/reconnect.rs`)
   - Manages SSE reconnection logic
   - Requires EventTracker at construction
   - Handles retry strategies

### Optional Higher-Level Components

4. **SessionAwareSseManager** (`src/transport/sse/session.rs`)
   - Higher-level abstraction over SSE
   - Integrates SessionManager with SSE connections
   - Might be overkill for reverse proxy
   - Consider if you need full session lifecycle management

5. **SseStream** (`src/transport/sse/stream.rs`)
   - Low-level SSE stream handling
   - Probably not needed directly

## Integration Recommendations for Reverse Proxy

### Option 1: Direct SessionManager Integration (RECOMMENDED)

The simplest approach - use SessionManager directly:

```rust
// In ReverseProxyServer
struct ReverseProxyServer {
    session_manager: Arc<SessionManager>,
    // ... other fields
}

// For SSE handling
async fn handle_sse_request(&self, session_id: SessionId) {
    // Create event tracker for this session
    let event_tracker = self.session_manager
        .create_event_tracker(session_id.clone())
        .await;
    
    // Use for deduplication and Last-Event-Id tracking
    if let Some(last_id) = event_tracker.get_last_event_id().await {
        // Resume from last event
    }
    
    // During streaming
    event_tracker.record_event(&event).await?;
}
```

### Option 2: Use SessionAwareSseManager

If you need full session lifecycle management:

```rust
// In ReverseProxyServer
struct ReverseProxyServer {
    sse_manager: Arc<SessionAwareSseManager>,
    // ... other fields
}
```

But this might be overkill - it's designed for managing multiple SSE connections per session.

## Key Integration Points

### 1. Session Creation
```rust
// Session already has last_event_id field
let session = Session {
    id: session_id,
    last_event_id: None,  // Updated by EventTracker
    // ...
};
```

### 2. SSE Stream Setup
```rust
// Parse client's Last-Event-Id header
if let Some(last_event_id) = headers.get("last-event-id") {
    event_tracker.set_last_event_id(last_event_id.to_string()).await;
}
```

### 3. Event Processing
```rust
// During SSE streaming
while let Some(event) = stream.next().await {
    // Record event (handles deduplication and persistence)
    event_tracker.record_event(&event).await?;
    
    // Forward to client
    send_to_client(event).await?;
}
```

### 4. Reconnection
```rust
// On reconnection, resume from last event
let last_id = event_tracker.get_last_event_id().await;
// Include in upstream request headers
```

## Migration from Planned Architecture

The plan mentioned using `ReverseProxySseManager`, but that's been deleted. Replace with:

- **Before**: `ReverseProxySseManager`
- **After**: Direct `SessionManager` + `EventTracker`

## Breaking Changes to Note

1. **EventTracker requires persistence channel** - Created via SessionManager
2. **No more callbacks** - Everything is channel-based
3. **Tests need async context** - SessionManager spawns background task

## Performance Characteristics

- **Memory**: ~60KB per session (bounded)
- **Tasks**: 1 persistence worker total (not per session)
- **Writes**: Coalesced by 10x
- **Backpressure**: Natural via channels
- **Deduplication**: O(1) HashSet lookup

## Next Steps for Reverse Proxy

1. Remove references to `ReverseProxySseManager`
2. Integrate `SessionManager` directly into `ReverseProxyServer`
3. Use `create_event_tracker()` for SSE sessions
4. Implement Last-Event-Id header parsing
5. Test with MCP Inspector

## Questions to Resolve

1. **Do we need SessionAwareSseManager?** 
   - Probably not for reverse proxy
   - Direct SessionManager integration is simpler
   
2. **Should we remove unused SSE components?**
   - SessionAwareSseManager might be unused after integration
   - Keep for now, remove if confirmed unused

3. **How to handle multiple upstreams?**
   - Each upstream connection needs its own EventTracker
   - SessionManager can create multiple trackers