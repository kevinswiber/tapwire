# Task R.2: Implement HttpRawTransport

## Objective
Implement raw HTTP transport for both client and server modes, handling HTTP request/response at the byte level.

## Key Requirements
1. Implement the `RawTransport` trait
2. Support both HTTP client and server modes
3. Handle HTTP headers and body separately
4. No JSON-RPC or MCP protocol knowledge
5. Reuse existing axum/reqwest infrastructure

## Implementation Steps

### 1. Create HttpRawClient
For outgoing HTTP requests:
```rust
pub struct HttpRawClient {
    client: reqwest::Client,
    url: String,
    headers: HeaderMap,
    connected: bool,
}
```

### 2. Create HttpRawServer
For incoming HTTP requests:
```rust
pub struct HttpRawServer {
    request_rx: mpsc::Receiver<(Bytes, oneshot::Sender<Bytes>)>,
    bind_addr: SocketAddr,
    server_handle: Option<JoinHandle<()>>,
}
```

### 3. Key Implementation Details
- Use reqwest for client
- Use axum for server
- Support custom headers
- Handle connection pooling
- Proper error mapping

## Testing Requirements
- Test client request/response
- Test server request handling
- Test header preservation
- Test connection lifecycle
- Test error scenarios

## Success Criteria
- [ ] Implements RawTransport trait
- [ ] HTTP client works correctly
- [ ] HTTP server works correctly
- [ ] Headers preserved
- [ ] Connection pooling works
- [ ] All tests pass

## Files to Create/Modify
- `src/transport/raw/http.rs` - New implementation
- `src/transport/raw/mod.rs` - Export new types
- Integration with existing http modules

## Dependencies
- Requires Phase 1 foundation (✅ Complete)
- May reuse parts of existing HttpTransport

## Estimated Duration: 3 hours