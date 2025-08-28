# SSE and Streaming Knowledge Base

**Created**: 2025-01-08  
**Purpose**: Central knowledge repository for SSE and streaming implementation in MCP

## Table of Contents

1. [Overview](#overview)
2. [Key Insights](#key-insights)
3. [Existing Implementations](#existing-implementations)
4. [Architecture Decisions](#architecture-decisions)
5. [Code Locations](#code-locations)
6. [Integration Points](#integration-points)
7. [Future Work](#future-work)
8. [References](#references)

## Overview

We're implementing **Streamable HTTP** transport for MCP, which is NOT a separate transport but rather HTTP with content negotiation:
- **HTTP-only mode**: Returns `application/json` for single responses
- **SSE mode**: Returns `text/event-stream` for streaming responses

The client sends `Accept: application/json, text/event-stream` and the server chooses based on:
1. Client capabilities (Accept header)
2. Server configuration (stateful vs stateless mode)
3. Request type (some benefit from streaming)

## Key Insights

### 1. Streamable HTTP is One Transport, Two Modes

Per the MCP specification (~/src/modelcontextprotocol/modelcontextprotocol/docs/specification/draft/basic/transports.mdx):
- It replaces the old HTTP+SSE transport from protocol version 2024-11-05
- Server decides response type based on Content-Type
- Client MUST support both response types

### 2. We Already Have Sophisticated SSE Support

We have TWO complete SSE implementations in shadowcat:

**A. Transport-level SSE** (`/src/transport/sse/`)
- Full SSE client with reconnection
- Session-aware streaming
- Event parsing and buffering
- Health monitoring
- Exponential backoff reconnection

**B. Reverse Proxy SSE** (`/src/proxy/reverse/upstream/http/streaming/`)
- Upstream SSE handling
- Intercepted streaming
- Reconnection strategies
- Raw streaming support

### 3. Event Tracking Abstraction

We've already abstracted event tracking in the MCP crate to be transport-agnostic:

```
crates/mcp/src/events/
├── tracker.rs         # Generic EventTracker trait
├── mod.rs            # Module exports
└── (implementations in transport modules)
```

Key abstraction points:
- `EventId` is just a String wrapper (works for SSE, WebSocket, etc.)
- `EventTracker` trait for deduplication and resumption
- Transport-specific implementations (e.g., `SseEventTracker`)

## Existing Implementations

### A. Shadowcat SSE Transport (`/src/transport/sse/`)

| File | Purpose | Key Features |
|------|---------|--------------|
| `client.rs` | SSE HTTP client | Request/response handling, stream management |
| `connection.rs` | Connection management | State tracking, error handling |
| `event.rs` | Event types | `SseEvent`, field parsing |
| `parser.rs` | SSE parsing | Streaming parser, error recovery |
| `buffer.rs` | Buffering | Stream buffering for parsing |
| `reconnect.rs` | Reconnection logic | `ReconnectionManager`, strategies, health monitoring |
| `session.rs` | Session awareness | Session-based streaming |
| `manager.rs` | Connection management | Pool-like management of SSE connections |

### B. MCP Crate Streaming (`crates/mcp/src/transport/http/streaming/`)

| File | Purpose | Status |
|------|---------|--------|
| `sse.rs` | SSE parsing and reconnection | Partially implemented |
| `event_tracker.rs` | SSE-specific event tracking | Complete |
| `mod.rs` | Module exports | Basic |

### C. New Streamable HTTP (`crates/mcp/src/transport/http/`)

| File | Purpose | Status |
|------|---------|--------|
| `streamable_config.rs` | Configuration for both modes | ✅ Created |
| `streamable_incoming.rs` | Server-side Streamable HTTP | 🚧 Started (needs SSE body streaming) |
| `streamable_outgoing.rs` | Client-side Streamable HTTP | ❌ Not started |

## Architecture Decisions

### 1. Unified Streaming Abstraction

**Question**: Should streaming be abstracted for SSE, WebSockets, and future transports?

**Answer**: YES! We need:
- Generic `StreamingTransport` trait
- Event tracking abstraction (✅ already have this)
- Reconnection strategies (can reuse from SSE)
- Session management integration

### 2. Response Mode Selection

Server chooses response mode based on:

```rust
enum ResponseDecision {
    Json,     // Single response, close connection
    Sse,      // Stream responses, keep connection
}

fn decide_response_mode(config: &StreamableHttpConfig, accept: &str, request: &Value) -> ResponseDecision {
    if !config.stateful_mode {
        return ResponseDecision::Json; // HTTP-only mode
    }
    
    if !accept.contains("text/event-stream") {
        return ResponseDecision::Json; // Client doesn't support SSE
    }
    
    // Could be SSE - decide based on request type
    if is_initialization_request(request) {
        ResponseDecision::Sse // Keep stream for future messages
    } else {
        ResponseDecision::Json // Simple request/response
    }
}
```

### 3. Reuse vs Rewrite

**Reuse from shadowcat SSE**:
- ✅ Event parsing (`parser.rs`, `event.rs`)
- ✅ Reconnection strategies (`reconnect.rs`)
- ✅ Health monitoring
- ✅ Buffer management

**Adapt for MCP**:
- 🔄 Session integration (use MCP session manager)
- 🔄 Event tracking (use abstracted trait)
- 🔄 Connection management (integrate with MCP pool)

**New for Streamable HTTP**:
- ❌ Content negotiation logic
- ❌ Dynamic response type selection
- ❌ Hyper 1.7 body streaming for SSE

## Code Locations

### Core References

```bash
# Shadowcat SSE implementation (sophisticated, battle-tested)
~/src/tapwire/shadowcat-mcp-compliance/src/transport/sse/

# Reverse proxy SSE (upstream handling)
~/src/tapwire/shadowcat-mcp-compliance/src/proxy/reverse/upstream/http/streaming/

# MCP event abstraction (transport-agnostic)
~/src/tapwire/shadowcat-mcp-compliance/crates/mcp/src/events/

# MCP HTTP streaming (in progress)
~/src/tapwire/shadowcat-mcp-compliance/crates/mcp/src/transport/http/streaming/

# New Streamable HTTP implementation
~/src/tapwire/shadowcat-mcp-compliance/crates/mcp/src/transport/http/streamable_*.rs
```

### External References

```bash
# MCP Specification (Streamable HTTP definition)
~/src/modelcontextprotocol/modelcontextprotocol/docs/specification/draft/basic/transports.mdx

# RMCP Reference Implementation
~/src/modelcontextprotocol/rust-sdk/crates/rmcp/src/transport/streamable_http_server.rs
~/src/modelcontextprotocol/rust-sdk/crates/rmcp/src/transport/streamable_http_client.rs

# TypeScript SDK (for behavior reference)
~/src/modelcontextprotocol/typescript-sdk/src/transport/
```

## Integration Points

### 1. Session Manager Integration

```rust
// Use existing MCP session manager
use crate::session::{Session, SessionManager};

// For SSE streams, track:
- session_id: SessionId
- last_event_id: Option<EventId>
- stream_state: StreamState
```

### 2. Connection Pool Integration

```rust
// SSE connections need special handling in pool
impl PoolableResource for StreamableConnection {
    // SSE connections may be long-lived
    // Need different health checks
    // May not be reusable after streaming
}
```

### 3. Event Tracker Usage

```rust
// Use the abstracted event tracker
use crate::events::{EventTracker, EventId};

// SSE implementation
let tracker = SseEventTracker::new(session_id, config);

// WebSocket implementation (future)
let tracker = WebSocketEventTracker::new(session_id, config);
```

## Future Work

### Phase 1: Complete Streamable HTTP (Current Sprint)
1. ✅ Configuration structure
2. 🚧 Server-side implementation
   - ❌ Fix SSE body streaming with hyper 1.7
   - ❌ Session integration
3. ❌ Client-side implementation
   - Handle both response types
   - SSE event parsing
   - Reconnection support
4. ❌ Tests for both modes

### Phase 2: WebSocket Support
1. Implement WebSocket transport
2. Reuse event tracking abstraction
3. Add `mcpEventId` field support
4. Integrate with same session manager

### Phase 3: Unified Streaming API
1. Create `StreamingTransport` trait
2. Implement for SSE, WebSocket, others
3. Common reconnection strategies
4. Unified metrics and monitoring

## Implementation Checklist

### Immediate TODOs

- [ ] **Fix SSE body streaming**: Current implementation has TODO for streaming body
  - Use hyper 1.7's streaming body capabilities
  - Look at shadowcat SSE for reference
  
- [ ] **Complete `StreamableIncomingConnection`**:
  - Implement GET request handling for server-initiated streams
  - Add session management
  - Support Last-Event-Id resumption

- [ ] **Create `StreamableOutgoingConnection`**:
  - Implement `Outgoing` trait
  - Handle both JSON and SSE responses
  - Parse SSE events using existing parser
  - Reconnection with Last-Event-Id

- [ ] **Integrate existing SSE code**:
  - Reuse `SseParser` from shadowcat
  - Adapt `ReconnectionManager` for MCP
  - Use `SseEventTracker` for deduplication

### Code to Reuse

From `shadowcat-mcp-compliance/src/transport/sse/`:
```rust
// Can directly reuse:
use crate::transport::sse::{
    SseParser,      // For parsing SSE streams
    SseEvent,       // Event representation
    EventBuilder,   // Building events
    parse_sse_stream, // Stream parsing utility
};

// Adapt for MCP:
use crate::transport::sse::{
    ReconnectionManager,  // Adapt to use MCP session
    SseConnection,       // Adapt to use MCP pool
    SessionStream,       // Integrate with MCP sessions
};
```

## Key Questions and Decisions

### Q1: Should we move shadowcat SSE into MCP crate?

**Current thinking**: No, keep them separate:
- Shadowcat SSE is for proxy/general use
- MCP SSE is specifically for Streamable HTTP
- Share code through careful abstraction

### Q2: How to handle hyper 1.7 streaming bodies?

**Options**:
1. Use `http_body_util::StreamBody` for SSE
2. Create custom body type implementing `Body` trait
3. Use existing SSE streaming from shadowcat

**Recommendation**: Option 1 with StreamBody, referencing shadowcat patterns

### Q3: Session management for SSE streams?

**Decision**: 
- Stateful mode: Full session with persistence
- Stateless mode: No session, each request independent
- Use existing MCP `SessionManager`

## Testing Strategy

### Unit Tests
- Content negotiation logic
- Response type selection
- SSE event parsing
- Reconnection logic

### Integration Tests
- HTTP-only mode end-to-end
- SSE streaming mode end-to-end
- Mode switching based on Accept header
- Session continuity across reconnections

### Performance Tests
- SSE streaming latency
- Memory usage with long-lived streams
- Reconnection overhead
- Pool efficiency with mixed modes

## Notes for Next Session

When continuing this work:

1. **Start here**: `/Users/kevin/src/tapwire/plans/mcp-unified-architecture/SSE-AND-STREAMING-KNOWLEDGE.md`
2. **Current task**: Implementing SSE body streaming in `streamable_incoming.rs`
3. **Next task**: Create `streamable_outgoing.rs` for client-side
4. **Key challenge**: Integrating hyper 1.7 streaming with existing SSE code
5. **Don't forget**: We have sophisticated SSE in shadowcat - reuse it!

## References

### Specifications
- [MCP Streamable HTTP Spec](~/src/modelcontextprotocol/modelcontextprotocol/docs/specification/draft/basic/transports.mdx)
- [SSE Specification](https://html.spec.whatwg.org/multipage/server-sent-events.html)
- [MCP Protocol Spec](https://modelcontextprotocol.io/specification)

### Implementations
- [RMCP Streamable HTTP](~/src/modelcontextprotocol/rust-sdk/crates/rmcp/src/transport/)
- [TypeScript MCP SDK](~/src/modelcontextprotocol/typescript-sdk/)
- [Shadowcat SSE](~/src/tapwire/shadowcat-mcp-compliance/src/transport/sse/)

### Key Files Modified
- `crates/mcp/src/transport/http/streamable_config.rs` - Configuration
- `crates/mcp/src/transport/http/streamable_incoming.rs` - Server implementation (partial)
- `crates/mcp/src/transport/http/mod.rs` - Module exports

---

**Remember**: This is a big lift. We don't need it done fast - we need it done RIGHT. The abstraction should work for SSE, WebSockets, and future streaming transports. Always ask: "Would this work for WebSockets too?"