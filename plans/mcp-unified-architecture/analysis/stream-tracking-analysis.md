# Stream Tracking Analysis Report

## Executive Summary

Our current implementation treats events as belonging only to sessions, violating the MCP specification requirement that event replay must be stream-specific. Since we haven't released yet, we can make a clean API break to fix this properly.

## Current Architecture

### Event ID Generation
- **Location**: `crates/mcp/src/events/event_id.rs`
- **Format**: `{session_id}-{node_id}-{json_rpc_id}-{counter}`
- **Problem**: Missing stream_id component

### Event Storage
- **Location**: `crates/mcp/src/events/store.rs`
- **Structure**: `HashMap<SessionId, VecDeque<StoredEvent>>`
- **Problem**: No stream differentiation - all events grouped by session only

### SSE Connection Handling
- **Location**: `crates/mcp/src/transport/http/streamable_incoming.rs`
- **Current**: Each connection has session_id, event_id_generator, event_store
- **Missing**: No stream_id field or stream tracking

## Component Interaction Map

```
StreamableIncomingConnection
    ├── session_id (from Mcp-Session-Id header)
    ├── event_id_generator (generates IDs)
    ├── event_store (stores/retrieves events)
    └── [MISSING: stream_id]

EventIdGenerator.generate()
    Input: session_id, json_rpc_id
    Output: "{session_id}-{node_id}-{json_rpc_id}-{counter}"
    [NEEDS: stream_id parameter]

EventStore.store_event()
    Input: session_id, event_id, data
    Storage: sessions[session_id].push(event)
    [NEEDS: stream_id parameter]

EventStore.get_events_after()
    Input: session_id, after_event_id
    Output: All events after ID in that session
    [NEEDS: stream_id parameter to filter]
```

## Files That Need Changes

### Core Changes (Must Modify)
1. **`crates/mcp/src/events/event_id.rs`**
   - Add stream_id parameter to generate()
   - Add parse_event_id() function
   - Update format to include stream

2. **`crates/mcp/src/events/store.rs`**
   - Change EventStore trait methods to include stream_id
   - Refactor InMemoryEventStore to use nested HashMap
   - Update StoredEvent to include stream_id field

3. **`crates/mcp/src/transport/http/streamable_incoming.rs`**
   - Add stream_id field to StreamableIncomingConnection
   - Generate stream_id on connection creation
   - Pass stream_id to all event operations
   - Parse stream from Last-Event-ID for replay

4. **`crates/mcp/src/session/persistence_worker.rs`**
   - Update PersistenceRequest enum variants with stream_id
   - Update store_event calls
   - Handle stream-specific cleanup

5. **`crates/mcp/src/session/manager.rs`**
   - Integrate StreamManager for stream tracking
   - Clean up streams when sessions end

### New Files to Create
1. **`crates/mcp/src/session/stream.rs`**
   - StreamId type
   - StreamManager implementation
   - Stream lifecycle methods

### Test Files to Update
- All tests using EventStore (8 test functions)
- All tests using EventIdGenerator (5 test functions)
- StreamableIncoming tests

## Implementation Checklist

### Phase 1: Stream Infrastructure (Task 2.5.1)
- [ ] Create `src/session/stream.rs` with StreamId type
- [ ] Implement StreamManager with registration/unregistration
- [ ] Add to session module exports
- [ ] Integrate StreamManager into SessionManager

### Phase 2: EventStore Refactor (Task 2.5.2)
- [ ] Add stream_id to EventStore trait methods
- [ ] Add stream_id field to StoredEvent
- [ ] Refactor InMemoryEventStore to nested HashMap structure
- [ ] Update all EventStore method implementations
- [ ] Update PersistenceWorker integration

### Phase 3: EventId Format Update (Task 2.5.3)
- [ ] Update generate() to include stream_id parameter
- [ ] Change format to `{session_id}-{stream_id}-{node_id}-{json_rpc_id}-{counter}`
- [ ] Implement parse_event_id() function
- [ ] Update all call sites (2 locations)

### Phase 4: Integration (Task 2.5.4)
- [ ] Add stream_id field to StreamableIncomingConnection
- [ ] Generate stream_id on connection creation
- [ ] Pass stream_id to event_id_generator.generate()
- [ ] Pass stream_id to event_store.store_event()
- [ ] Parse stream_id from Last-Event-ID header
- [ ] Implement stream-specific replay logic
- [ ] Unregister stream on disconnect

## Breaking Changes

Since we're not maintaining backwards compatibility:

1. **EventStore trait** - All methods get new stream_id parameter
2. **EventIdGenerator.generate()** - Requires stream_id parameter
3. **Event ID format** - Changes from 4 to 5 components
4. **StoredEvent struct** - Gets new stream_id field

## Benefits of Clean Break

- Simpler, cleaner API
- No migration code needed
- No compatibility layers
- Correct implementation from the start
- Better performance (no compatibility checks)

## Testing Strategy

### Unit Tests
- Test StreamId generation and uniqueness
- Test StreamManager registration/cleanup
- Test nested EventStore structure
- Test event ID parsing

### Integration Tests
- Test multiple streams per session
- Test stream-specific replay
- Verify no cross-stream contamination
- Test stream cleanup on disconnect

### Manual Testing
```bash
# Terminal 1: Start server
cargo run -- forward http --port 8080

# Terminal 2: Connect stream A
curl -N -H "Accept: text/event-stream" \
     -H "Mcp-Session-Id: session-1" \
     http://localhost:8080/mcp

# Terminal 3: Connect stream B (same session)
curl -N -H "Accept: text/event-stream" \
     -H "Mcp-Session-Id: session-1" \
     http://localhost:8080/mcp

# Disconnect stream A, reconnect with Last-Event-ID
# Verify only stream A events are replayed
```

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Performance impact of nested HashMap | Use efficient composite key |
| Memory growth with many streams | Implement stream limits and TTL |
| Stream leaks on abnormal disconnect | Timeout-based cleanup |

## Next Steps

1. ✅ Complete this analysis (Task 2.5.0)
2. Begin StreamManager implementation (Task 2.5.1)
3. Refactor EventStore with clean API (Task 2.5.2)
4. Update EventIdGenerator (Task 2.5.3)
5. Integration and testing (Task 2.5.4)

## Conclusion

The current implementation fundamentally violates the MCP specification by not tracking individual streams. Since we haven't released, we should fix this with a clean API design rather than maintaining backwards compatibility. The implementation is straightforward but touches several core components.