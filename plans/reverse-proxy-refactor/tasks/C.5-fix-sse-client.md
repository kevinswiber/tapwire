# Task C.5: Fix SSE Client for Long-lived Connections

## Problem Statement

The current SSE implementation closes connections after receiving a single event. Investigation revealed that reqwest's `bytes_stream()` method is not designed for long-lived SSE connections - it completes when initial data is consumed rather than keeping the connection open for future events.

### Root Cause
- Reqwest lacks native SSE support ([issue #2677](https://github.com/seanmonstar/reqwest/issues/2677))
- `bytes_stream()` treats end of available data as stream completion
- This breaks the SSE model of keeping connections open indefinitely

### Impact
- SSE connections close prematurely after first event
- Client hangs waiting for more events that never arrive
- Breaks real-time updates in MCP Inspector and other SSE clients

## Solution: Use eventsource-client Library

After evaluating options, we've chosen LaunchDarkly's eventsource-client library because:
- Production-ready and actively maintained
- Built for Tokio (which we already use)
- Handles reconnection with exponential backoff
- Provides proper SSE protocol support

## Implementation Plan

### Step 1: Create SSE Client Module (1.5 hours)
Create `src/proxy/reverse/sse_client.rs`:
```rust
use eventsource_client as es;
use futures::stream::Stream;

pub struct SseUpstreamClient {
    url: String,
    headers: HeaderMap,
    session_id: SessionId,
}

impl SseUpstreamClient {
    pub async fn connect(&self) -> Result<impl Stream<Item = SseEvent>> {
        // Build eventsource client with headers
        // Return stream of SSE events
    }
}
```

### Step 2: Modify Upstream Request Logic (1.5 hours)
In `process_via_http_new()`:
1. Detect SSE early via Accept header check
2. Branch to SSE-specific path:
   - For SSE: Create SseUpstreamClient and connect
   - For non-SSE: Continue using reqwest as before

### Step 3: Update SSE Streaming Module (1 hour)
Modify `sse_streaming.rs`:
1. Accept stream from eventsource-client
2. Convert events to our internal format
3. Maintain existing interceptor support
4. Pass through to client

### Step 4: Handle Reconnection (1 hour)
- Store Last-Event-Id in SessionStore
- Pass Last-Event-Id header on reconnection
- Handle connection failures gracefully

### Step 5: Testing (1 hour)
- Test with MCP Inspector
- Verify multiple events are received
- Test reconnection scenarios
- Measure performance impact

## Success Criteria
- [ ] SSE connections remain open indefinitely
- [ ] Multiple events are received over time
- [ ] Reconnection works with Last-Event-Id
- [ ] No performance regression for non-SSE requests
- [ ] MCP Inspector shows continuous event stream

## Code Changes Required

### Files to Create:
- `src/proxy/reverse/sse_client.rs` - New SSE client using eventsource-client

### Files to Modify:
- `src/proxy/reverse/legacy.rs` - Update process_via_http_new() for SSE detection
- `src/proxy/reverse/sse_streaming.rs` - Consume eventsource-client stream
- `Cargo.toml` - Already added eventsource-client dependency

## Testing Strategy

1. **Unit Tests**:
   - Mock eventsource-client stream
   - Test event conversion
   - Test error handling

2. **Integration Tests**:
   - Use test SSE server
   - Verify long-lived connections
   - Test reconnection logic

3. **Manual Testing**:
   - MCP Inspector with everything server
   - Monitor connection lifetime
   - Verify event delivery

## Risks and Mitigations

**Risk**: eventsource-client may have bugs (early stage release)
**Mitigation**: Thorough testing, fallback to custom hyper implementation if needed

**Risk**: Performance impact from additional library
**Mitigation**: Benchmark before/after, optimize hot paths

**Risk**: Breaking existing non-SSE functionality
**Mitigation**: Clear branching logic, comprehensive tests

## References
- [reqwest SSE issue](https://github.com/seanmonstar/reqwest/issues/2677)
- [eventsource-client docs](https://github.com/launchdarkly/rust-eventsource-client)
- [SSE protocol spec](https://html.spec.whatwg.org/multipage/server-sent-events.html)