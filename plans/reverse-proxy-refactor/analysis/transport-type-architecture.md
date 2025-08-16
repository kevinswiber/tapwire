# Transport Type Architecture Analysis

## Problem Statement
We have a code smell with `is_sse_session` boolean field in the Session struct. This indicates a deeper architectural issue with how we model transport types in the proxy, especially for bidirectional proxying where incoming and outgoing transports may differ.

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

## Architectural Requirements

### 1. Bidirectional Transport Tracking
As a proxy, we need to track BOTH:
- **Incoming transport**: How the client connects to us
- **Outgoing transport**: How we connect to upstream

Example scenarios:
- Client (stdio) → Proxy → Upstream (streamable-http)
- Client (streamable-http) → Proxy → Upstream (stdio)
- Client (streamable-http) → Proxy → Upstream (streamable-http)

### 2. Message-Level Transport Indication
Each message/envelope needs to indicate its transport:
- Messages from stdio transport
- HTTP POST request messages
- HTTP response messages (JSON or SSE stream)
- SSE event messages within a stream

### 3. Protocol Compliance
Must align with MCP's transport model:
- stdio
- streamable-http (HTTP POST + optional SSE response)

### 4. Dynamic Response Type Handling
The same session might handle:
- JSON responses (application/json)
- SSE streams (text/event-stream)
Based on server's response to each request

## Design Options

### Option 1: Separate Incoming/Outgoing Transport Types
```rust
pub struct Session {
    pub id: SessionId,
    pub incoming_transport: McpTransportType,  // Client → Proxy
    pub outgoing_transport: McpTransportType,  // Proxy → Upstream
    pub last_event_id: Option<String>,        // SSE state (if applicable)
    // Remove is_sse_session - derive from transport types
}

pub enum McpTransportType {
    Stdio,
    Http,           // HTTP-only (rare)
    StreamableHttp, // HTTP + SSE (common)
}
```

### Option 2: Transport Pair Pattern
```rust
pub struct TransportPair {
    pub incoming: TransportEndpoint,
    pub outgoing: TransportEndpoint,
}

pub enum TransportEndpoint {
    Stdio(StdioConfig),
    Http(HttpConfig),
    Sse(SseConfig),  // SSE is always preceded by HTTP
}
```

### Option 3: Dynamic Transport State
```rust
pub struct Session {
    pub id: SessionId,
    pub transport_state: TransportState,
    // ...
}

pub enum TransportState {
    Stdio,
    HttpOnly,
    HttpWithSse {
        last_event_id: Option<String>,
        event_tracker: Option<Arc<EventTracker>>,
    },
}
```

### Option 4: Protocol-Aligned Enum
```rust
// Directly match MCP spec
pub enum McpTransport {
    Stdio,
    Http,          // Rarely used
    StreamableHttp {
        mode: StreamableHttpMode,
    },
}

pub enum StreamableHttpMode {
    Request,      // Initial HTTP request
    Response,     // HTTP response
    SseStream,    // Subsequent SSE events
}
```

## Impact Analysis

### Files to Modify
1. `src/transport/mod.rs` - Update TransportType enum
2. `src/session/store.rs` - Remove is_sse_session, update transport tracking
3. `src/proxy/reverse/legacy.rs` - Update transport detection logic
4. `src/proxy/reverse/sse_resilience.rs` - Use transport type instead of boolean
5. Message/Envelope structures - Add transport indication

### Backward Compatibility
- Need migration for existing sessions
- Update all session creation sites
- Fix tests that assume single transport type

## Recommendation

**Implement Option 1** (Separate Incoming/Outgoing) because:
1. Accurately models proxy behavior
2. Aligns with MCP spec (stdio/http/streamable-http)
3. Eliminates the need for `is_sse_session` boolean
4. Provides clear semantics for bidirectional proxying
5. SSE state (`last_event_id`) only exists when relevant

## Implementation Plan

### Phase 1: Define New Types (1 hour)
- Create `McpTransportType` enum matching spec
- Add incoming/outgoing fields to Session
- Keep backward compat with old TransportType

### Phase 2: Update Session Management (2 hours)
- Remove `is_sse_session` field
- Update session creation/update logic
- Migrate existing session handling

### Phase 3: Update Transport Detection (2 hours)
- Fix HTTP vs SSE detection
- Update proxy routing logic
- Ensure proper transport propagation

### Phase 4: Testing & Validation (1 hour)
- Update tests for new transport model
- Verify MCP spec compliance
- Test various proxy configurations

## Key Questions to Answer

1. Should we rename `TransportType` to avoid confusion with the new model?
2. How do we handle transport upgrades (HTTP → SSE) mid-session?
3. Should envelope/message types directly reference transport type?
4. Do we need transport type in the database schema?

## Context for Next Session

This analysis stems from implementing SSE reconnection support (Phase D.0). We discovered that the `is_sse_session` boolean field is a code smell indicating deeper architectural issues with transport type modeling.

The core issue: We're proxying between potentially different transport types (stdio ↔ streamable-http) but our Session model assumes a single transport type. This needs to be fixed to properly support:
- Bidirectional proxying with different transports
- MCP spec compliance (stdio/http/streamable-http)
- Clean SSE state management without boolean flags

Start by reviewing:
1. MCP spec at `~/src/modelcontextprotocol/modelcontextprotocol/spec`
2. Current TransportType usage across codebase
3. How shadowcat's raw transport (`src/transport/raw/streamable_http.rs`) handles this

This is a foundational issue that should be resolved before continuing with Phase D.1-D.3 of SSE reconnection.