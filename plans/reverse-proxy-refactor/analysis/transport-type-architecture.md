# Transport Type Architecture Analysis

## Problem Statement
We have a code smell with `is_sse_session` boolean field in the Session struct. This indicates a deeper architectural issue with how we model transport types in the proxy, especially for bidirectional proxying where incoming and outgoing transports may differ.

## Key Discovery: Existing Directional Transport Architecture

After investigation, we found that shadowcat ALREADY has a sophisticated directional transport system in place:

1. **IncomingTransport/OutgoingTransport traits** - Well-defined abstractions for directional transports
2. **Forward proxy uses them** - The forward proxy already uses these directional transports
3. **Reverse proxy doesn't** - The reverse proxy has its own implementation, not using these traits

### Current Architecture Findings

#### TransportType Enum
- Used for **session categorization**, not transport implementation
- Lives at the session/configuration level
- Values: Stdio, Http, Sse
- Used by both forward and reverse proxies for session tracking

#### Directional Transports (transport/directional/)
- **IncomingTransport**: Accepts connections from clients
  - StdioIncoming, HttpServerIncoming, StreamableHttpIncoming
- **OutgoingTransport**: Initiates connections to servers
  - SubprocessOutgoing, HttpClientOutgoing, StreamableHttpOutgoing
- Used by forward proxy, NOT by reverse proxy

#### Reverse Proxy Transport Handling
- Uses `ReverseUpstreamConfig` with `transport_type` field
- Directly spawns processes or makes HTTP requests
- Doesn't use the directional transport abstractions
- Has its own connection pooling for stdio (PoolableOutgoingTransport)

## Current State

### Session Struct Issue
```rust
pub struct Session {
    // ...
    pub transport_type: TransportType,  // Single transport type
    pub is_sse_session: bool,          // CODE SMELL: Redundant/wrong abstraction
    pub last_event_id: Option<String>, // Legitimate SSE state
}
```

### Current TransportType Enum
```rust
// Found in src/transport/mod.rs
pub enum TransportType {
    Stdio,
    Http,
    Sse,  // Exists but confusingly named - should be StreamableHttp
}
```

## MCP Protocol Reality

According to the MCP spec (modelcontextprotocol/specs/draft/basic/transports.mdx):
1. **stdio** - Standard input/output streams
2. **Streamable HTTP** - HTTP POST requests with optional SSE responses (replaces old HTTP+SSE)

The spec no longer lists plain "http" as a separate transport. The new "Streamable HTTP" transport:
- Uses HTTP POST for client→server messages
- Can return either `application/json` OR `text/event-stream` responses
- Supports SSE for server→client streaming
- Replaces the old HTTP+SSE transport from protocol 2024-11-05

## The Real Problem

The issue isn't just about `is_sse_session`. We have **two separate transport architectures**:

1. **Forward Proxy**: Uses clean directional transport traits
2. **Reverse Proxy**: Has its own implementation, duplicates logic

This leads to:
- Code duplication between forward and reverse proxies
- The `is_sse_session` hack because reverse proxy doesn't model transports properly
- Inconsistent transport handling across the codebase

## Architectural Requirements

### 1. Unify Transport Handling
- Reverse proxy should use IncomingTransport/OutgoingTransport
- Share transport implementations between forward and reverse proxies
- Eliminate duplicate transport logic

### 2. Fix Session Transport Modeling
The Session struct conflates several concepts:
- **Transport Type**: What protocol (stdio, http, sse)
- **Transport Direction**: Client→Proxy vs Proxy→Upstream
- **Response Mode**: Whether currently streaming SSE

### 3. Proper Bidirectional Tracking
For reverse proxy, we need:
- **Incoming**: Always HTTP/StreamableHttp (from clients)
- **Outgoing**: Could be stdio OR http (to upstream)
- **Response Mode**: JSON vs SSE (dynamic per request)

## Design Options

### Option 1: Leverage Existing Directional Transports ✅ RECOMMENDED
Instead of creating new types, use what already exists:

```rust
pub struct Session {
    pub id: SessionId,
    pub client_transport_type: TransportType,   // What type of client connection
    pub upstream_transport_type: TransportType, // What type of upstream connection
    pub response_mode: ResponseMode,           // Current response mode
    pub last_event_id: Option<String>,        // SSE state (if applicable)
    // Remove is_sse_session - derive from response_mode
}

pub enum ResponseMode {
    Unknown,           // Not yet determined
    Json,             // application/json response
    SseStream,        // text/event-stream response
}
```

**Benefits**:
- Reuses existing TransportType enum
- Clear separation of concerns
- Explicit response mode tracking
- No breaking changes to existing code

### Option 2: Transport Context Pattern
```rust
pub struct Session {
    pub id: SessionId,
    pub transport_context: TransportContext,
    // ...
}

pub struct TransportContext {
    pub incoming: TransportEndpoint,
    pub outgoing: TransportEndpoint,
    pub stream_state: Option<StreamState>,
}

pub enum TransportEndpoint {
    Stdio,
    StreamableHttp {
        accepts_sse: bool,  // Client's Accept header includes text/event-stream
    },
}

pub struct StreamState {
    pub last_event_id: Option<String>,
    pub is_streaming: bool,
}
```

### Option 3: Response-Aware Transport State
```rust
pub struct Session {
    pub id: SessionId,
    pub client_transport: McpTransportType,
    pub upstream_transport: McpTransportType,
    pub response_mode: ResponseMode,
    // ...
}

pub enum ResponseMode {
    NotYetDetermined,
    Json,              // application/json response
    SseStream {        // text/event-stream response
        last_event_id: Option<String>,
    },
}
```

### Option 4: Simplified Boolean Flags (Current, but improved)
```rust
pub struct Session {
    pub id: SessionId,
    pub transport_type: TransportType,
    pub is_streamable_http: bool,  // Better name than is_sse_session
    pub accepts_sse: bool,         // Client capability
    pub is_streaming: bool,        // Current state
    pub last_event_id: Option<String>,
}
```

## Impact Analysis

### Files to Modify
1. `src/transport/mod.rs` - Update TransportType enum to align with spec
2. `src/session/store.rs` - Remove is_sse_session, add incoming/outgoing transports
3. `src/proxy/reverse/legacy.rs` - Update transport detection logic
4. `src/proxy/reverse/sse_resilience.rs` - Use transport type instead of boolean
5. `src/session/memory.rs` - Update in-memory store implementation
6. Tests - Update all tests that create sessions

### Backward Compatibility Considerations
- Current TransportType::Sse should map to StreamableHttp
- Current TransportType::Http could also map to StreamableHttp
- is_sse_session=true means StreamableHttp transport
- Need to handle existing sessions during migration

## Recommendation

We should take a **two-phase approach**:

### Phase 1: Quick Fix (1-2 hours)
**Implement Option 1** with minimal changes:
1. Add `client_transport_type` and `upstream_transport_type` to Session
2. Add `ResponseMode` enum for tracking response type
3. Remove `is_sse_session` boolean
4. Update code to check `response_mode == ResponseMode::SseStream`

This fixes the immediate code smell without major refactoring.

### Phase 2: Architectural Unification (Future)
**Unify transport handling** across proxies:
1. Refactor reverse proxy to use IncomingTransport/OutgoingTransport traits
2. Share transport implementations between forward and reverse proxies
3. Eliminate duplicate transport logic
4. Move to a more unified transport architecture

This is a larger refactor that should be done separately.

## Implementation Plan (Phase 1 - Quick Fix)

### Step 1: Add ResponseMode Enum (30 min)
```rust
// src/session/store.rs

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseMode {
    Unknown,          // Not yet determined
    Json,            // application/json response
    SseStream,       // text/event-stream response
}
```

### Step 2: Update Session Structure (1 hour)
```rust
// src/session/store.rs

pub struct Session {
    pub id: SessionId,
    pub transport_type: TransportType,          // Keep for now (backward compat)
    pub client_transport_type: TransportType,   // NEW: Type of client connection
    pub upstream_transport_type: TransportType, // NEW: Type of upstream connection
    pub response_mode: ResponseMode,           // NEW: Current response mode
    pub status: SessionStatus,
    pub state: SessionState,
    pub created_at: u64,
    pub last_activity: u64,
    pub frame_count: usize,
    pub client_info: Option<String>,
    pub server_info: Option<String>,
    pub version_state: VersionState,
    pub tags: Vec<String>,
    pub last_event_id: Option<String>,
    // REMOVE: pub is_sse_session: bool,
}

impl Session {
    pub fn new_with_transports(
        id: SessionId,
        client_transport: TransportType,
        upstream_transport: TransportType,
    ) -> Self {
        let now = current_timestamp();
        Self {
            id,
            transport_type: client_transport, // For backward compat
            client_transport_type: client_transport,
            upstream_transport_type: upstream_transport,
            response_mode: ResponseMode::Unknown,
            // ... rest of fields
        }
    }
    
    pub fn is_sse(&self) -> bool {
        self.response_mode == ResponseMode::SseStream
    }
    
    pub fn set_response_mode(&mut self, mode: ResponseMode) {
        self.response_mode = mode;
        self.update_activity();
    }
}
```

### Step 3: Update Usage Sites (30 min)
```rust
// src/proxy/reverse/legacy.rs

// Instead of: session.mark_as_sse_session()
session.set_response_mode(ResponseMode::SseStream);

// Instead of: if session.is_sse_session
if session.response_mode == ResponseMode::SseStream {
    // Handle SSE streaming
}

// When detecting response type from headers:
let response_mode = if content_type.contains("text/event-stream") {
    ResponseMode::SseStream
} else if content_type.contains("application/json") {
    ResponseMode::Json
} else {
    ResponseMode::Unknown
};
session.set_response_mode(response_mode);
```

### Step 4: Backward Compatibility (Important!)
```rust
// src/session/store.rs

impl Session {
    // Keep the old constructor for backward compat
    pub fn new(id: SessionId, transport_type: TransportType) -> Self {
        let now = current_timestamp();
        Self {
            id,
            transport_type,
            client_transport_type: transport_type,
            upstream_transport_type: transport_type,
            response_mode: ResponseMode::Unknown,
            // ... rest of fields
            // is_sse_session: false, // REMOVED
        }
    }
    
    // Keep mark_as_sse_session for backward compat (deprecated)
    #[deprecated(note = "Use set_response_mode(ResponseMode::SseStream) instead")]
    pub fn mark_as_sse_session(&mut self) {
        self.response_mode = ResponseMode::SseStream;
        self.update_activity();
    }
}
```

## Key Decisions

1. **Keep existing TransportType**: Don't create new enums, work with what exists
2. **Add ResponseMode**: Separate concept from transport type - it's about response format
3. **Maintain backward compat**: Deprecate old methods but keep them working
4. **Two-phase approach**: Quick fix now, architectural unification later

## Benefits of This Approach

1. **Minimal disruption**: Small, focused changes
2. **Clear semantics**: ResponseMode explicitly tracks what we're actually checking
3. **Backward compatible**: Existing code continues to work
4. **Sets up for future**: Lays groundwork for transport unification

## Summary

The investigation revealed that:
1. **TransportType is for session categorization**, not transport implementation
2. **We already have IncomingTransport/OutgoingTransport traits** that the forward proxy uses
3. **The reverse proxy duplicates transport logic** instead of using these traits
4. **The `is_sse_session` flag is really tracking response mode**, not transport type

The recommended fix:
1. **Phase 1**: Add ResponseMode enum, track client/upstream transports separately
2. **Phase 2**: Future refactor to unify transport handling across proxies

This approach fixes the immediate code smell while setting up for proper architectural improvements.