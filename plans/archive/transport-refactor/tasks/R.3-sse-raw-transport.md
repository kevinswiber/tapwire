# Task R.3: Implement SseRawTransport

## Objective
Implement raw Server-Sent Events transport for streaming data, handling SSE protocol at the byte level.

## Key Requirements
1. Implement the `StreamingRawTransport` trait
2. Support both SSE client and server modes
3. Handle SSE event framing (data:, event:, id:, retry:)
4. No JSON-RPC or MCP knowledge
5. Integrate with HTTP transport for initial connection

## Implementation Steps

### 1. Create SseRawClient
For receiving SSE events:
```rust
pub struct SseRawClient {
    event_stream: Pin<Box<dyn Stream<Item = Result<Event, Error>>>>,
    connected: bool,
}
```

### 2. Create SseRawServer
For sending SSE events:
```rust
pub struct SseRawServer {
    event_tx: mpsc::Sender<Event>,
    client_connected: Arc<AtomicBool>,
}
```

### 3. SSE Event Structure
```rust
pub struct Event {
    pub data: Vec<u8>,
    pub event_type: Option<String>,
    pub id: Option<String>,
    pub retry: Option<Duration>,
}
```

### 4. Key Implementation Details
- Use eventsource-client for client
- Use axum's SSE support for server
- Handle reconnection logic
- Parse SSE format correctly
- Support event IDs and types

## Testing Requirements
- Test event parsing
- Test streaming data flow
- Test reconnection handling
- Test event types and IDs
- Test server-side event generation

## Success Criteria
- [ ] Implements StreamingRawTransport trait
- [ ] SSE client receives events correctly
- [ ] SSE server sends events correctly
- [ ] Event framing works properly
- [ ] Reconnection logic works
- [ ] All tests pass

## Files to Create/Modify
- `src/transport/raw/sse.rs` - New implementation
- `src/transport/raw/mod.rs` - Export new types
- Integration with HTTP for initial connection

## Dependencies
- Requires Phase 1 foundation (✅ Complete)
- Requires R.2 (HTTP transport) for initial connection

## Estimated Duration: 3 hours