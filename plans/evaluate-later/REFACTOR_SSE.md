# SSE Refactoring Plan for Reverse Proxy

## Current Issues

1. **File Size**: `reverse.rs` has grown to 3400+ lines - too large to maintain
2. **Duplicate Code**: SSE handling logic is duplicated instead of using existing infrastructure
3. **Architectural Problem**: Current approach consumes response body to parse SSE, preventing streaming
4. **No Session Mapping**: Not handling session mapping between client and upstream
5. **Missing Interceptor Support**: SSE responses bypass interceptor chain
6. **Event Buffering Required**: Need to buffer SSE events in memory for:
   - Interceptor processing (each complete event must go through chain)
   - Session mapping (translate IDs between client and upstream)
   - Reconnection support (replay events from Last-Event-Id)

## Existing Infrastructure We Should Use

- `src/transport/sse/` - Complete SSE transport module
  - `SseStream` - Parses SSE from any async reader
  - `SseConnectionManager` - Manages multiple SSE connections
  - `SseParser` - Robust SSE event parsing
  - `SseBuffer` - Buffering and stream management
  - Session-aware SSE handling

## Proposed Solution

### 1. Create New Module: `src/proxy/reverse_sse.rs`
Move all SSE-specific reverse proxy logic to a dedicated module:
- SSE response detection
- Stream proxying without buffering
- Event ID correlation
- Session mapping for SSE

### 2. Refactor `process_via_http` 
Instead of consuming the response body:
```rust
// Return response without consuming body for SSE
enum HttpUpstreamResponse {
    Json(ProtocolMessage),
    SseStream(reqwest::Response), // Keep response for streaming
}
```

### 3. Use Existing SSE Infrastructure
```rust
use crate::transport::sse::{SseStream, SseConnectionManager};

// In reverse_sse.rs
pub async fn proxy_sse_response(
    response: reqwest::Response,
    session: &Session,
    interceptors: Arc<InterceptorChain>,
) -> Result<impl IntoResponse> {
    // Use SseStream to parse without consuming
    let stream = SseStream::new(response.bytes_stream());
    // Apply interceptors to each event
    // Handle session mapping
    // Return streaming response
}
```

### 4. Session Mapping Architecture
As per `/plans/reverse-proxy-session-mapping/`:
- Maintain mapping between client sessions and upstream sessions
- Handle multiple clients connecting to same upstream
- Properly correlate events between sessions

## Implementation Steps

1. **Phase 1: Extract SSE code** (Immediate)
   - Create `reverse_sse.rs` module
   - Move existing SSE functions
   - Reduce `reverse.rs` size

2. **Phase 2: Refactor Response Handling** (Next Sprint)
   - Change `process_via_http` to not consume SSE bodies
   - Return response object for streaming
   - Update all call sites
   - Add event buffering for interceptors

3. **Phase 3: Integrate SSE Infrastructure** (Following Sprint)
   - Replace custom SSE parsing with `SseStream`
   - Use `SseConnectionManager` for connection tracking
   - Add proper event correlation
   - Implement event buffering with configurable size
   - Process each complete SSE event through interceptor chain

4. **Phase 4: Session Mapping** (As per roadmap)
   - Implement session mapping architecture (see `/plans/reverse-proxy-session-mapping/`)
   - Handle multiple client scenarios
   - Add session state synchronization
   - Buffer events for Last-Event-Id replay
   - Translate session IDs in SSE event data

## Benefits

1. **Maintainability**: Smaller, focused modules
2. **Reusability**: Leverage existing tested SSE code
3. **Performance**: True streaming without buffering
4. **Features**: Interceptors, session mapping, reconnection
5. **Testing**: Easier to test isolated modules

## Current Workaround

The current implementation uses `ReverseProxyError::SseStreamingRequired` as a special signal from `process_via_http` to indicate that an SSE response was detected. When this error is returned, the handler makes a second request to get a fresh response stream for proxying. 

**Note**: This is a temporary approach that will likely be removed once proper SSE handling is implemented:
- The `SseStreamingRequired` error is a workaround for the fact that we can't return both a ProtocolMessage and a raw response stream
- Making a duplicate request to upstream is inefficient and incorrect
- This should be replaced with a better architecture where SSE responses are handled separately from the start

### Issues with Current Approach:
1. **Duplicate Requests**: We detect SSE, then make the same request again to get a fresh stream
2. **Error as Control Flow**: Using an error variant for control flow is non-idiomatic
3. **No Interceptor Support**: SSE events bypass the interceptor chain
4. **No Session Mapping**: Client and upstream session IDs aren't mapped
5. **No Event Buffering**: Can't replay events for reconnection

## Files to Refactor

- `src/proxy/reverse.rs` - Extract SSE logic
- `src/proxy/mod.rs` - Add new module exports
- `src/proxy/reverse_sse.rs` - New module (to create)
- Update tests to use new module structure