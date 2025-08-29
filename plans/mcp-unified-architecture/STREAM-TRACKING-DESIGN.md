# Stream Tracking Design for MCP SSE Transport

## Problem Statement

The MCP specification defines a critical distinction between **sessions** and **streams**:

- **Session**: An MCP protocol session identified by a session ID (e.g., from Mcp-Session-Id header)
- **Stream**: A transport-level connection (e.g., an SSE connection) within a session

Key requirements from the MCP spec:
1. Event IDs MUST be **globally unique across all streams within a session**
2. Event replay (via Last-Event-ID) MUST only replay messages **from the specific stream that was disconnected**
3. The server MUST NOT replay messages from different streams

## Current Implementation Gap

Our current `EventStore` implementation:
- Uses only `session_id` as the key
- Doesn't track individual streams
- Would incorrectly replay events from ALL streams in a session

## Required Architecture

### 1. Stream Identity

Each SSE stream needs a unique identifier:
```rust
pub struct StreamId(Uuid);

impl StreamId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
```

### 2. Composite Key Structure

Events must be stored with composite keys:
```rust
pub struct EventKey {
    pub session_id: SessionId,
    pub stream_id: StreamId,
}
```

### 3. Event ID Format

Event IDs must encode the stream they belong to:
```
{session_id}-{stream_id}-{node_id}-{json_rpc_id}-{counter}
```

This ensures:
- Global uniqueness across all streams in a session
- Ability to determine which stream an event belongs to

### 4. Updated EventStore Trait

```rust
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Store an event for a specific stream within a session
    async fn store_event(
        &self, 
        session_id: &str, 
        stream_id: &str,
        event_id: String, 
        data: Value
    );
    
    /// Get events after a specific event ID for the SAME stream
    async fn get_events_after(
        &self, 
        session_id: &str,
        stream_id: &str,
        after_event_id: &str
    ) -> Vec<StoredEvent>;
    
    /// Clear all streams for a session
    async fn clear_session(&self, session_id: &str);
    
    /// Clear a specific stream
    async fn clear_stream(&self, session_id: &str, stream_id: &str);
}
```

### 5. Stream Lifecycle Management

```rust
pub struct StreamManager {
    /// Active streams per session
    streams: Arc<RwLock<HashMap<SessionId, HashSet<StreamId>>>>,
    /// Stream metadata (creation time, last activity, etc.)
    metadata: Arc<RwLock<HashMap<StreamId, StreamMetadata>>>,
}

impl StreamManager {
    /// Register a new stream for a session
    pub async fn register_stream(&self, session_id: SessionId) -> StreamId {
        let stream_id = StreamId::new();
        // Add to session's stream set
        // Initialize metadata
        stream_id
    }
    
    /// Unregister a stream (on disconnect)
    pub async fn unregister_stream(&self, session_id: &SessionId, stream_id: &StreamId) {
        // Remove from session's stream set
        // Clean up metadata
    }
    
    /// Get all active streams for a session
    pub async fn get_session_streams(&self, session_id: &SessionId) -> Vec<StreamId> {
        // Return all stream IDs for this session
    }
}
```

### 6. Connection Tracking

Each SSE connection needs to maintain its stream ID:

```rust
pub struct StreamableIncomingConnection {
    session_id: SessionId,
    stream_id: StreamId,  // NEW: Unique per connection
    // ... other fields
}

impl StreamableIncomingConnection {
    pub async fn new(/* params */) -> Self {
        let stream_id = stream_manager.register_stream(session_id).await;
        // ...
    }
}
```

### 7. Event Replay Logic

When a client reconnects with Last-Event-ID header:

1. Parse the event ID to extract the stream ID
2. Look up events ONLY for that specific stream
3. Replay only those events
4. If stream ID not found, start fresh (no replay)

```rust
pub async fn handle_reconnection(
    last_event_id: &str,
    session_id: &SessionId,
    event_store: &dyn EventStore,
) -> Vec<StoredEvent> {
    // Parse stream ID from event ID format
    let stream_id = parse_stream_id_from_event(last_event_id)?;
    
    // Get events only from that specific stream
    event_store.get_events_after(
        &session_id.to_string(),
        &stream_id.to_string(),
        last_event_id
    ).await
}
```

## Implementation Plan

### Phase 1: Core Stream Tracking (4h)
1. Create `StreamId` type and `StreamManager`
2. Update `EventIdGenerator` to include stream ID
3. Add stream registration/unregistration

### Phase 2: EventStore Refactor (4h)
1. Update `EventStore` trait with stream_id parameter
2. Refactor `InMemoryEventStore` to use composite keys
3. Update `StoredEvent` to include stream_id field

### Phase 3: Integration (4h)
1. Update `StreamableIncomingConnection` to track stream ID
2. Modify event storage calls to include stream ID
3. Implement stream-specific replay logic

### Phase 4: Testing (2h)
1. Test multiple streams per session
2. Test stream-specific replay
3. Test stream cleanup on disconnect

## Considerations

### Memory Management
- Need to clean up stream data when connections close
- Consider TTL for inactive streams
- Limit max streams per session

### Performance
- Composite key lookups should remain O(1) with proper HashMap nesting
- Consider using `(SessionId, StreamId)` tuple as key

### Backwards Compatibility
- Migration path for existing event stores
- Default stream ID for non-SSE transports?

## Questions to Resolve

1. **Stream ID persistence**: Should stream IDs be persisted or regenerated on each connection?
2. **Stream limits**: Should we limit concurrent streams per session?
3. **Non-SSE transports**: Do stdio/WebSocket need stream tracking?
4. **Event ID parsing**: Should we make the format pluggable?

## Next Steps

1. Review this design document
2. Create tracking issue for implementation
3. Begin Phase 1 implementation
4. Update tests to cover multi-stream scenarios