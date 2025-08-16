# Response Mode Investigation

**Created**: 2025-08-16  
**Purpose**: Understand what `is_sse_session` actually tracks and design ResponseMode enum replacement

## Executive Summary

The investigation confirms that `is_sse_session` is tracking **response format** (JSON vs SSE streaming), not transport type. However, the flag is **never actually set** in the current codebase - the `mark_as_sse_session()` method exists but is never called. The reverse proxy detects SSE responses via `Content-Type: text/event-stream` header and handles them differently, but doesn't update the session flag. This is a vestigial field that should be replaced with proper response mode tracking.

## Current is_sse_session Usage

### Field Definition
- **Location**: `src/session/store.rs:68` - Boolean field in Session struct
- **Default**: Always `false` (set in constructor at line 91)

### Setting Points
**NONE FOUND** - The `mark_as_sse_session()` method exists but is never called!

### Checking Points
- **Location**: `src/proxy/reverse/sse_resilience.rs` - Checks for SSE reconnection (inactive)
- **Location**: `src/session/store.rs:255-257` - `is_sse()` getter method (unused)

### Critical Finding
The flag exists but is **completely unused** in practice. SSE detection happens at runtime via Content-Type headers, not via session flags.

## Response Mode Patterns

### JSON Response Pattern
**Detection**: 
- Content-Type: `application/json`
- Default response type

**Handling**:
```rust
// From legacy.rs:1527-1528
debug!("Returning JSON format response");
Ok((StatusCode::OK, response_headers, Json(json_response)).into_response())
```

**Characteristics**:
- Single request-response
- Buffered and parsed
- Connection closes after response
- Uses standard JSON-RPC format

### SSE Stream Pattern
**Detection**:
```rust
// From hyper_client.rs:162-168
pub fn is_sse(&self) -> bool {
    self.response
        .headers()
        .get(hyper::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("text/event-stream"))
        .unwrap_or(false)
}
```

**Handling**:
- Streamed via `forward_sse_raw` or `forward_sse_with_interceptors`
- Long-lived connection
- Multiple events over time
- Uses Server-Sent Events format

**Key Code Paths**:
1. `process_via_http_hyper()` detects SSE via `hyper_response.is_sse()`
2. Routes to SSE streaming functions
3. Maintains open connection for event stream

### Other Potential Patterns
Currently not implemented but could be added:
- **Binary**: `application/octet-stream` for file transfers
- **WebSocket**: Bidirectional streaming (future)
- **gRPC**: Protocol buffer streaming (future)

## Accept Header Analysis

### Client Capabilities
The reverse proxy sets Accept headers based on expectations:

```rust
// From hyper_client.rs
req = if accept_sse {
    req.header(ACCEPT, "application/json, text/event-stream")
} else {
    req.header(ACCEPT, "application/json")
};
```

### Server Response Choice
The server chooses response type based on:
1. What the client accepts (Accept header)
2. What the server supports
3. The nature of the request (some methods may require streaming)

### Current Problem
The proxy **always** sets `accept_sse = true`, meaning it always accepts both JSON and SSE, but the session doesn't track which mode was chosen.

## Session vs Request Scope

### Current Approach Problems
- `is_sse_session` implies session-wide scope
- But a session can have mixed response types
- First request might be JSON (initialize)
- Later requests might be SSE (subscribe to events)

### Reality in Code
The code actually handles this **per-response**, not per-session:
```rust
// Each response is checked independently
if hyper_response.is_sse() {
    // Handle as SSE
} else {
    // Handle as JSON
}
```

### Recommended Approach
Track response mode **per-request** with session-level **capabilities** using bitflags:

```rust
pub struct Session {
    // Remove: is_sse_session
    // Add:
    pub client_capabilities: ClientCapabilities, // Bitflags, not struct
    pub last_response_mode: Option<ResponseMode>,
}

// See implementation-recommendations.md for full ClientCapabilities bitflags design
// Using bitflags provides type-safe, efficient capability tracking
```

## Proposed ResponseMode Design

### Enum Definition
```rust
/// Represents the format of a response from the upstream server
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseMode {
    /// Response format not yet determined
    Unknown,
    
    /// Standard JSON response (application/json)
    Json,
    
    /// Server-Sent Events stream (text/event-stream)
    SseStream,
    
    /// Binary data (application/octet-stream) - future
    Binary,
    
    /// Plain text (text/plain) - future
    Text,
    
    /// WebSocket upgrade - future
    WebSocket,
}

impl ResponseMode {
    /// Detect response mode from Content-Type header
    pub fn from_content_type(content_type: &str) -> Self {
        if content_type.contains("application/json") {
            Self::Json
        } else if content_type.contains("text/event-stream") {
            Self::SseStream
        } else if content_type.contains("application/octet-stream") {
            Self::Binary
        } else if content_type.contains("text/plain") {
            Self::Text
        } else {
            Self::Unknown
        }
    }
}
```

### Integration Points

**Where to store**:
1. In MessageEnvelope metadata for per-message tracking
2. In Session as `last_response_mode` for session history
3. In UpstreamResponse struct for processing

**When to determine**:
- After receiving response headers from upstream
- Before choosing processing path (JSON vs streaming)

**How to use**:
```rust
// In process_via_http_hyper
let response_mode = ResponseMode::from_content_type(
    hyper_response.content_type().unwrap_or("")
);

match response_mode {
    ResponseMode::Json => process_json_response(...),
    ResponseMode::SseStream => forward_sse_stream(...),
    _ => forward_raw_response(...),
}
```

## Benefits of ResponseMode

### Type Safety
- **Explicit states** instead of boolean flag
- **Compile-time checking** of all response types
- **No ambiguity** about what's being tracked

### Extensibility
- Easy to add WebSocket, gRPC, binary formats
- Each mode can have specific handling
- Future-proof for new protocols

### Clarity
- Clear separation between transport (how) and format (what)
- Self-documenting code
- Easier debugging and logging

## Implementation Strategy

### Phase 1: Add ResponseMode (1 hour)
1. Add enum to `src/transport/mod.rs`
2. Add `from_content_type()` detection method
3. Add to MessageEnvelope metadata

### Phase 2: Parallel Usage (2 hours)
1. Keep `is_sse_session` for now
2. Set ResponseMode alongside existing logic
3. Add logging to verify correctness

### Phase 3: Migration (2 hours)
1. Replace `hyper_response.is_sse()` with ResponseMode check
2. Update processing paths to use ResponseMode
3. Remove SSE-specific methods in favor of generic response mode

### Phase 4: Cleanup (1 hour)
1. Remove `is_sse_session` field
2. Remove `mark_as_sse_session()` method
3. Remove `is_sse()` method
4. Update tests

## Testing Considerations

### Test Scenarios
1. **JSON-only session**: Initialize, methods, shutdown
2. **SSE subscription**: Subscribe to events, receive stream
3. **Mixed session**: JSON initialize, then SSE subscribe
4. **Mode detection**: Verify Content-Type parsing
5. **Unknown formats**: Handle unexpected Content-Types gracefully

### Backward Compatibility
- No external API changes needed
- Internal refactoring only
- Tests continue to work with gradual migration

## Key Insights

1. **is_sse_session is dead code** - Never set, barely checked
2. **Response mode is per-response** - Not per-session
3. **Content-Type drives behavior** - Runtime detection, not configuration
4. **Proxies handle formats differently** - Buffered JSON vs streamed SSE
5. **Clean architecture exists** - Just needs proper typing

## Recommendations

### Immediate Actions
1. Add ResponseMode enum with proper detection
2. Use it in hyper response processing
3. Remove unused is_sse_session code

### Architecture Alignment
1. Align with directional transport model
2. Make response mode orthogonal to transport type
3. Support multiple response modes per session

### Future Extensions
1. Add WebSocket support with ResponseMode::WebSocket
2. Add binary streaming for file transfers
3. Support protocol negotiation and upgrades

## Conclusion

The investigation reveals that `is_sse_session` is vestigial code that was intended to track SSE streaming but was never properly implemented. The actual SSE detection happens via Content-Type headers at response time. The proposed ResponseMode enum provides a clean, extensible solution that properly models the different response formats while maintaining clear separation from transport types. This aligns perfectly with the broader transport architecture refactoring goals.