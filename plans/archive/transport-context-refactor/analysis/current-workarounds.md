# Current Workarounds Catalog

## Overview

This document catalogs the existing patterns and workarounds that have been implemented to deal with the limitations of the current `TransportMessage` structure, particularly around missing context and direction information.

## Direction Tracking Workarounds

### 1. External Direction in Frame
**Location**: `session/manager.rs`, `session/store.rs`

```rust
pub struct Frame {
    pub session_id: SessionId,
    pub direction: Direction,  // Tracked separately from message
    pub message: TransportMessage,
    pub timestamp: SystemTime,
}
```

**Problem Solved**: Need to know if message is client→server or server→client
**Limitation**: Direction is lost when message leaves the Frame context
**Impact**: Can't properly route notifications through proxy

### 2. Transport Edge Inference
**Location**: `proxy/forward.rs`, `proxy/reverse.rs`

The proxy infers direction based on which transport received the message:
- Message from client transport → must be client-to-server
- Message from server transport → must be server-to-client

**Problem**: Works for request/response but fails for notifications
**Workaround**: Assumes all notifications follow request/response pattern

## Session Context Workarounds

### 3. Session ID Extraction from Messages
**Location**: `session/manager.rs:574-613`

```rust
fn extract_session_from_message(&self, message: &TransportMessage, fallback: Option<&SessionId>) -> Option<SessionId>
```

Complex logic to extract session ID from:
1. Initialize request → create new session
2. Response → look up by request ID
3. Notification → use fallback or active session

**Problem**: Session ID should be carried with message
**Impact**: Complex state tracking, potential for session confusion

### 4. HTTP Headers Separate Handling
**Location**: `transport/http_mcp.rs`

```rust
pub struct McpHeaders {
    pub session_id: Option<String>,
    pub protocol_version: Option<String>,
}

// Headers extracted but not attached to message
pub fn extract_mcp_headers(headers: &HeaderMap) -> McpHeaders
```

**Problem**: Headers contain critical context but aren't propagated
**Workaround**: Re-extract headers at each layer
**Impact**: Lost metadata, repeated parsing

## Transport Metadata Workarounds

### 5. SSE Event Context Loss
**Location**: `transport/sse/manager.rs`

SSE event IDs and retry information are extracted but not preserved:
- Event IDs needed for resumability
- Retry-After headers for reconnection
- Currently discarded after parsing

**Problem**: Can't properly implement SSE resumption
**Impact**: No fault tolerance for SSE connections

### 6. Retry Information in Separate Channel
**Location**: `transport/sse/retry_strategy.rs`

```rust
// Retry info passed out-of-band
pub struct HttpRetryInfo {
    pub retry_after: Option<Duration>,
    pub retry_after_header: Option<String>,
}
```

**Workaround**: Side channel for retry information
**Problem**: Not associated with specific messages/responses
**Impact**: Complex coordination between components

## Method-Based Routing Heuristics

### 7. Initialize Request Detection
**Location**: `session/manager.rs:615-617`

```rust
fn is_initialize_request(&self, message: &TransportMessage) -> bool {
    matches!(message, TransportMessage::Request { method, .. } if method == "initialize")
}
```

**Problem**: Special-casing specific methods throughout codebase
**Workaround**: Pattern matching on method names
**Impact**: Brittle, doesn't scale to other special messages

### 8. Notification Method Routing
**Location**: Multiple files

Attempting to determine notification direction by method name:
- `notifications/initialized` → must be from client
- `notifications/resources/*` → must be from server
- Problem: Not all notifications follow this pattern

## State Management Workarounds

### 9. Global Session State
**Location**: `session/manager.rs`

Maintains complex state maps to track:
- Active sessions
- Pending requests (for response correlation)
- Session capabilities
- Protocol versions

**Problem**: State that should travel with messages is stored globally
**Impact**: Thread safety concerns, state synchronization issues

### 10. Request-Response Correlation Maps
**Location**: `session/manager.rs`, `proxy/forward.rs`

```rust
pending_requests: Arc<RwLock<HashMap<String, RequestContext>>>
```

**Workaround**: Track every request to correlate responses
**Problem**: Memory overhead, cleanup complexity
**Should be**: Context flowing with messages

## Interceptor Workarounds

### 11. Context Reconstruction in Interceptors
**Location**: `interceptor/engine.rs`, `interceptor/actions.rs`

Interceptors must reconstruct context from:
- Global state lookups
- Method name patterns
- Previous message history

**Problem**: Each interceptor reimplements context detection
**Impact**: Inconsistent behavior, performance overhead

### 12. Action Context Preservation
**Location**: `interceptor/actions.rs`

When modifying messages, interceptors must:
- Preserve implicit context
- Maintain session continuity
- Avoid breaking correlation

**Workaround**: Complex state machine in each interceptor
**Should be**: Context preserved automatically

## Recording/Replay Workarounds

### 13. Tape Metadata Storage
**Location**: `recorder/tape.rs`

Stores context separately from messages:
```rust
pub struct TapeFrame {
    pub message: TransportMessage,
    pub direction: Direction,
    pub timestamp: SystemTime,
    // Context that should be part of message
}
```

**Problem**: Reconstruction requires tape-specific logic
**Impact**: Can't replay without full tape context

### 14. Replay State Reconstruction
**Location**: `transport/replay.rs`

Must reconstruct:
- Session state from message sequence
- Direction from message ordering
- Timing from separate metadata

**Workaround**: Complex state machine for replay
**Should be**: Self-contained messages with context

## Testing Workarounds

### 15. Mock Context Injection
**Location**: Various test files

Tests must manually construct context:
```rust
// Tests manually set up direction, session, etc.
let frame = Frame::new(session_id, Direction::ClientToServer, message);
```

**Problem**: Tests don't match production message flow
**Impact**: Tests may not catch context-related bugs

## Performance Workarounds

### 16. Context Caching
**Location**: `session/manager.rs`

Caches frequently accessed context to avoid repeated lookups:
- Session capabilities
- Protocol versions
- Active directions

**Problem**: Cache invalidation complexity
**Should be**: Context travels with message

## Security Workarounds

### 17. Auth Token Stripping
**Location**: `proxy/reverse.rs`

Must carefully strip auth tokens before forwarding:
- Parse message to find tokens
- Remove from params
- Track what was removed

**Problem**: Token handling mixed with message forwarding
**Should be**: Clean separation of auth context

## Summary of Impact

### Code Complexity
- 17+ distinct workaround patterns identified
- Each adds complexity and potential bugs
- Maintenance burden across 34 files

### Performance Impact
- Multiple lookups for context reconstruction
- Unnecessary state tracking
- Cache management overhead

### Correctness Issues
- Lost context in edge cases
- Notification routing failures
- Session confusion possibilities

### Development Velocity
- New features require updating multiple workarounds
- Testing complexity due to implicit context
- Debugging difficulty when context is wrong

## Refactor Benefits

Implementing `MessageEnvelope` with proper context will:
1. Eliminate all 17 workaround patterns
2. Reduce code complexity significantly
3. Enable proper bidirectional notification routing
4. Simplify testing and debugging
5. Improve performance by reducing lookups
6. Enable SSE resumption and retry features
7. Provide foundation for future transport types