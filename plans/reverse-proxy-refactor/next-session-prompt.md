# Next Session: Phase C.5.1 - Replace Reqwest with Hyper for SSE Streaming

## Critical Issue
Client cannot establish successful SSE connections through the proxy. The issue is not with the upstream server but with how the proxy handles SSE streaming. Reqwest's abstraction layer is preventing proper SSE handling.

## Current Status
- **Phase A**: ✅ COMPLETE - Full analysis
- **Phase B**: ✅ COMPLETE - SessionStore abstraction  
- **Phase C**: ✅ COMPLETE - SSE routing fixed, but streaming broken
- **Phase C.5**: 🚧 IN PROGRESS - Fix SSE streaming with hyper
- **Phase D**: ⬜ Blocked - Modularization
- **Phase E**: ⬜ Blocked - Integration & testing

## Your Task: Replace Reqwest with Hyper

### Why This Is Necessary
1. **Reqwest limitations discovered**:
   - Cannot access raw hyper::Body
   - `bytes_stream()` completes prematurely
   - No control over chunk handling
   - Abstraction prevents proper SSE streaming

2. **What hyper gives us**:
   - Direct access to Body stream
   - Full control via `poll_data(cx)`
   - Proper chunked transfer encoding
   - Ability to keep connection alive

### Implementation Plan

#### Step 1: Create Hyper HTTP Client Module (2 hours)
Create `src/proxy/reverse/hyper_client.rs`:
```rust
use hyper::{Body, Client, Request, Response};
use hyper::client::HttpConnector;

pub struct HyperHttpClient {
    client: Client<HttpConnector>,
}

impl HyperHttpClient {
    pub async fn send_request(
        &self,
        method: Method,
        url: &str,
        headers: HeaderMap,
        body: Vec<u8>,
    ) -> Result<Response<Body>> {
        // Build hyper request
        // Send and return raw response
    }
}
```

#### Step 2: Create New process_via_http_hyper (2 hours)
Replace `process_via_http_new` with hyper-based version:
1. Use hyper::Client instead of reqwest::Client
2. Build Request<Body> directly
3. Return Response<Body> without wrapping
4. Let caller handle body streaming

#### Step 3: Update SSE Streaming (2 hours)
Modify `sse_streaming.rs` to work with hyper::Body:
1. Use `poll_data(cx)` to read chunks
2. Keep polling until None (connection closed)
3. Parse SSE events from chunks as they arrive
4. No more EOF issues from reqwest

#### Step 4: Integration & Testing (2 hours)
1. Update handle_mcp_request to use new hyper client
2. Test with MCP Inspector
3. Verify client can connect successfully
4. Confirm SSE events are received

### Files to Create/Modify
- NEW: `src/proxy/reverse/hyper_client.rs` - Hyper-based HTTP client
- NEW: `src/proxy/reverse/hyper_streaming.rs` - Body stream handling
- UPDATE: `src/proxy/reverse/legacy.rs` - Use hyper instead of reqwest
- UPDATE: `src/proxy/reverse/sse_streaming.rs` - Work with hyper::Body
- UPDATE: `Cargo.toml` - Ensure hyper dependencies are correct

### Success Criteria
- [ ] Client can establish connection through proxy
- [ ] SSE events are received properly
- [ ] No premature stream termination
- [ ] Connection stays open as long as upstream keeps it open
- [ ] MCP Inspector shows successful communication

### Key Code Pattern from eventsource-client
```rust
// This is how eventsource-client polls hyper::Body
match ready!(body.poll_data(cx)) {
    Some(Ok(chunk)) => {
        // Process chunk
        continue;
    }
    Some(Err(e)) => {
        // Handle error
    }
    None => {
        // Stream ended
    }
}
```

### Testing Commands
```bash
# Build
cargo build --release

# Run proxy with debug
RUST_LOG=debug ./target/release/shadowcat reverse \
  --bind 127.0.0.1:8081 \
  --upstream http://localhost:3001/mcp

# Test with MCP Inspector
# Connect to http://127.0.0.1:8081
# Should successfully initialize and receive responses
```

### Important Notes
1. Hyper is already a dependency (via reqwest), so no new deps needed
2. This gives us the low-level control needed for proper SSE
3. We can reuse much of the existing code structure
4. Focus on getting basic HTTP working first, then SSE

### Time Estimate
- Phase C.5.1: 6-8 hours
- This unblocks proper SSE streaming functionality

## Why This Will Work
Hyper gives us direct access to the response body stream, allowing us to:
- Poll for chunks continuously
- Handle chunked transfer encoding properly
- Keep connections alive as long as data is being sent
- Have full control over the streaming behavior

This is exactly what eventsource-client does successfully, and we can apply the same pattern.