# Task R.4: Implement StreamableHttpRawTransport

## Objective
Implement the unified Streamable HTTP transport that combines HTTP POST requests with SSE responses, as specified in the MCP protocol.

## Key Requirements
1. Combine HTTP and SSE into single transport
2. HTTP POST for requests, SSE for streaming responses
3. Handle upgrade from HTTP to SSE seamlessly
4. Support both client and server modes
5. No protocol knowledge - just bytes

## Implementation Steps

### 1. Create StreamableHttpRawClient
For MCP client (sends HTTP POST, receives SSE):
```rust
pub struct StreamableHttpRawClient {
    http_client: HttpRawClient,
    sse_client: Option<SseRawClient>,
    mode: StreamableMode,
}

enum StreamableMode {
    Request,     // Sending HTTP POST
    Streaming,   // Receiving SSE events
}
```

### 2. Create StreamableHttpRawServer
For MCP server (receives HTTP POST, sends SSE):
```rust
pub struct StreamableHttpRawServer {
    http_server: HttpRawServer,
    sse_connections: HashMap<SessionId, SseRawServer>,
}
```

### 3. Connection Flow
1. Client sends HTTP POST with Accept: text/event-stream
2. Server processes request
3. Server responds with Content-Type: text/event-stream
4. Connection upgrades to SSE for streaming responses
5. Client receives multiple responses via SSE

### 4. Key Implementation Details
- Seamless transition from HTTP to SSE
- Session tracking for multiple connections
- Proper header negotiation
- Connection lifecycle management
- Error handling for partial upgrades

## Testing Requirements
- Test HTTP to SSE upgrade
- Test request/stream-response pattern
- Test multiple concurrent sessions
- Test connection failures mid-stream
- Test header negotiation

## Success Criteria
- [ ] Unified transport for Streamable HTTP
- [ ] Seamless HTTP to SSE transition
- [ ] Client sends POST, receives SSE
- [ ] Server receives POST, sends SSE
- [ ] Session management works
- [ ] All tests pass

## Files to Create/Modify
- `src/transport/raw/streamable_http.rs` - New implementation
- `src/transport/raw/mod.rs` - Export new types
- Compose HttpRawTransport and SseRawTransport

## Dependencies
- Requires R.2 (HttpRawTransport)
- Requires R.3 (SseRawTransport)

## Estimated Duration: 4 hours

## Notes
This is the key innovation that fixes the current architecture's inability to properly handle MCP's Streamable HTTP protocol as a single unified transport.