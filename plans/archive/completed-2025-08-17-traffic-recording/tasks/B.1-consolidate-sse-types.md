# Task B.1: Consolidate SseEvent Types

## Objective
Consolidate the multiple `SseEvent` definitions into a single canonical type that properly represents the SSE wire format, eliminating duplication and confusion.

## Current State

We have multiple SSE-related types scattered across the codebase:

1. **`transport::sse::event::SseEvent`** (lines 4-50)
   - The most complete implementation
   - Has id, event_type, data, retry fields
   - Includes builder methods

2. **`transport::outgoing::http::SseEvent`** (lines 52-60)
   - Internal struct for buffering
   - Duplicates fields but with different visibility
   - Less complete than the main one

3. **`recorder::tape::SseMetadata`** (lines 186-200)
   - Used for recording metadata
   - Has event_id, event_type, retry_ms, last_event_id
   - Different field names (retry_ms vs retry)

## Target State

Single canonical `SseEvent` in `transport::sse::event` that:
- Represents the wire format accurately
- Can be used by both transport and recording layers
- Has clear semantics for each field
- Includes necessary serialization traits

## Implementation Steps

### 1. Audit All Uses (30 min)
- [ ] Find all references to each SseEvent type
- [ ] Document how each is being used
- [ ] Identify any unique requirements

### 2. Design Canonical Type (30 min)
```rust
// Proposed structure in transport::sse::event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SseEvent {
    /// Optional event ID for reconnection (Last-Event-ID header)
    pub id: Option<String>,
    
    /// Event type (typically "message" for MCP)
    pub event_type: String,
    
    /// Event data containing the JSON-RPC message
    pub data: String,
    
    /// Optional retry delay in milliseconds
    pub retry: Option<u64>,
}

impl SseEvent {
    /// Create a basic SSE event with data
    pub fn new(data: String) -> Self { ... }
    
    /// Parse from wire format
    pub fn from_wire(raw: &[u8]) -> Result<Self> { ... }
    
    /// Serialize to wire format
    pub fn to_wire(&self) -> Vec<u8> { ... }
}
```

### 3. Update Transport Layer (1 hour)
- [ ] Remove `transport::outgoing::http::SseEvent`
- [ ] Update HTTP transport to use canonical type
- [ ] Update SSE parser to create canonical type
- [ ] Ensure all tests still pass

### 4. Update Recording Layer (1 hour)
- [ ] Map from `SseEvent` to `SseMetadata` where needed
- [ ] Consider if SseMetadata is still necessary
- [ ] Update recorder to use canonical type
- [ ] Update replay to use canonical type

### 5. Testing (30 min)
- [ ] Run all transport tests
- [ ] Run recording/replay tests
- [ ] Add tests for wire format parsing
- [ ] Verify backward compatibility

## Migration Considerations

### Field Mapping
- `retry` (SseEvent) ↔ `retry_ms` (SseMetadata)
- `id` (SseEvent) ↔ `event_id` (SseMetadata)
- `event_type` is consistent
- `last_event_id` only in SseMetadata (for reconnection)

### Breaking Changes
- Internal `http::SseEvent` can be removed safely
- `SseMetadata` might need to stay for tape format compatibility
- Consider versioning if tape format changes

## Success Criteria

- [ ] Single `SseEvent` type in `transport::sse::event`
- [ ] No duplicate SSE event types
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Clear documentation on each field's purpose
- [ ] Wire format parsing/serialization works

## Testing Checklist

- [ ] Unit tests for SseEvent creation
- [ ] Wire format parsing tests
- [ ] Round-trip serialization tests
- [ ] HTTP transport SSE handling
- [ ] Recording with SSE metadata
- [ ] Replay of SSE streams

## Dependencies
- Design document completion (A.2)
- Understanding of all current uses

## Notes
- Remember: SSE event ID ≠ JSON-RPC message ID
- The `event_type` field is rarely used in MCP (always "message")
- The `retry` field affects client reconnection behavior
- We may need adapter functions for backward compatibility

## Code Locations

### Files to Modify
- `src/transport/sse/event.rs` - Enhance canonical type
- `src/transport/outgoing/http.rs` - Remove duplicate, update usage
- `src/recorder/tape.rs` - Update SseMetadata mapping
- `src/recorder/session_recorder.rs` - Update extraction logic
- `src/replay/sse_support.rs` - Update replay logic

### Files to Review
- `src/transport/sse/parser.rs` - How events are parsed
- `src/transport/sse/reconnect.rs` - How retry is used
- Tests in `tests/` directory

---

**Estimated Duration**: 3 hours  
**Risk Level**: Medium (affects core transport and recording)  
**Rollback Plan**: Git revert if issues found