# SSE Proxy Integration Tracker

## Project Overview

This tracker coordinates the integration of the completed SSE transport implementation with Shadowcat's forward proxy, reverse proxy, and interceptor systems to enable end-to-end MCP Streamable HTTP support.

**Last Updated**: 2025-08-08  
**MCP Target Versions**: 2025-06-18 and 2025-03-26 (Streamable HTTP)  
**Prerequisites**: Phase 1 SSE Implementation (80% complete)  
**Compatibility**: See [MCP 2025-03-26 Compatibility Guide](tasks/compatibility-2025-03-26.md)  
**Coordination**: See [Integration Coordination Guide](../integration-coordination.md) for MCP message handling synergies

## Executive Summary

While Shadowcat has a robust SSE transport implementation (Phase 1), it currently exists in isolation from the proxy layers. This integration effort will bridge that gap to enable actual MCP client/server communication over SSE.

### Current Status
- ✅ **SSE Transport Layer**: Complete with parser, connection management, reconnection, and session integration
- ❌ **Forward Proxy Integration**: No SSE transport option or integration
- ❌ **Reverse Proxy Integration**: No Streamable HTTP endpoints or SSE response handling
- ❌ **Interceptor Support**: No streaming message awareness

### Integration Goals
1. Enable forward proxy to use SSE transport for MCP communication
2. Implement Streamable HTTP specification in reverse proxy
3. Add interceptor support for SSE event streams
4. Achieve full MCP compliance for both 2025-06-18 and 2025-03-26 versions

## MCP Streamable HTTP Requirements

Based on [MCP 2025-06-18 Specification](../../specs/mcp/docs/specification/2025-06-18/transports/http-sse.md) and [MCP 2025-03-26 Specification](../../specs/mcp/docs/specification/2025-03-26/basic/transports.mdx):

### Core Requirements (Both Versions)
- **Single `/mcp` endpoint** supporting both POST and GET methods
- **POST**: Send JSON-RPC messages, may return JSON or SSE stream
- **GET**: Open server-initiated SSE stream (requires session ID)
- **Session continuity** via `Mcp-Session-Id` header
- **Stream resumability** via `Last-Event-ID` header

### Request/Response Flow
```
POST /mcp HTTP/1.1
Accept: application/json, text/event-stream
MCP-Protocol-Version: 2025-06-18
Mcp-Session-Id: <session-id>
Content-Type: application/json

{"jsonrpc": "2.0", "method": "tools/list", "id": 1}

→ Response Options:
1. HTTP 200 + Content-Type: application/json (immediate response)
2. HTTP 200 + Content-Type: text/event-stream (SSE stream)
3. HTTP 202 Accepted (for notifications)
```

### Server-Initiated Events
```
GET /mcp HTTP/1.1
Accept: text/event-stream
MCP-Protocol-Version: 2025-06-18
Mcp-Session-Id: <session-id>
Last-Event-ID: <last-id>

→ Response:
HTTP 200 OK
Content-Type: text/event-stream

data: {"jsonrpc": "2.0", "method": "notification", "params": {...}}
id: evt-123

data: {"jsonrpc": "2.0", "result": {...}, "id": 1}
id: evt-124
```

### Version-Specific Differences
- **2025-03-26**: Supports JSON-RPC batching (arrays of messages)
- **2025-06-18**: Single messages only (no batching)
- **Default Version**: 2025-03-26 when no MCP-Protocol-Version header

## Architecture Overview

### Component Integration Map
```
┌─────────────────────────────────────────────────────────────┐
│                     CLI Interface                           │
├─────────────────────────────────────────────────────────────┤
│                    Forward Proxy                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │  Stdio   │  │   HTTP   │  │   SSE    │  │Streamable│  │
│  │Transport │  │Transport │  │Transport │  │   HTTP   │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │
├─────────────────────────────────────────────────────────────┤
│                    Interceptor Chain                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Request/Response | SSE Events | Stream Context       │  │
│  └──────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                    Reverse Proxy                            │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────────┐    │
│  │   HTTP   │  │ POST Handler  │  │  GET SSE Handler │    │
│  │  Server  │  │   (JSON/SSE)  │  │ (Event Stream)   │    │
│  └──────────┘  └──────────────┘  └──────────────────┘    │
├─────────────────────────────────────────────────────────────┤
│                   Session Management                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Unified Session Store | SSE Session State | Expiry   │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Phase 1: Forward Proxy SSE Integration

### Goal
Enable the forward proxy to use SSE transport for connecting to MCP servers.

### Tasks

#### Task 1.1: Add SSE Transport CLI Option
**Duration**: 2 hours  
**Priority**: HIGH  
**Files**: `src/main.rs`, `src/cli.rs`

**Deliverables**:
- [ ] Add SSE subcommand to forward proxy CLI
- [ ] Parse SSE-specific options (URL, headers)
- [ ] Validate SSE endpoint URL format
- [ ] Pass configuration to forward proxy

**CLI Example**:
```bash
shadowcat forward sse --url https://mcp.server.com/mcp
shadowcat forward streamable-http --endpoint https://server.com/mcp
```

#### Task 1.2: Create SSE Transport Wrapper
**Duration**: 3-4 hours  
**Priority**: HIGH  
**Files**: `src/transport/sse_transport.rs` (new)

**Deliverables**:
- [ ] Implement Transport trait for SSE
- [ ] Integrate SessionAwareSseManager
- [ ] Handle bi-directional message flow
- [ ] Map TransportMessage to/from SSE events
- [ ] Implement connect/disconnect lifecycle

#### Task 1.3: Integrate with Forward Proxy
**Duration**: 3 hours  
**Priority**: HIGH  
**Files**: `src/proxy/forward.rs`

**Deliverables**:
- [ ] Add SSE transport variant to ForwardTransport
- [ ] Initialize SSE transport in create_transport()
- [ ] Handle SSE-specific session management
- [ ] Support concurrent SSE connections
- [ ] Add SSE-specific error handling

## Phase 2: Reverse Proxy Streamable HTTP

### Goal
Implement full Streamable HTTP specification in the reverse proxy.

### Tasks

#### Task 2.1: Dual-Method `/mcp` Endpoint
**Duration**: 3 hours  
**Priority**: HIGH  
**Files**: `src/proxy/reverse.rs`

**Deliverables**:
- [ ] Add GET handler for `/mcp` endpoint
- [ ] Route based on HTTP method
- [ ] Validate Accept headers
- [ ] Check session ID requirements
- [ ] Return 405 for unsupported methods

#### Task 2.2: SSE Response Handler
**Duration**: 4 hours  
**Priority**: HIGH  
**Files**: `src/proxy/reverse.rs`, `src/transport/sse/server.rs` (new)

**Deliverables**:
- [ ] Detect when to return SSE vs JSON
- [ ] Create SSE response streams
- [ ] Convert TransportMessage to SSE events
- [ ] Implement proper Content-Type handling
- [ ] Support chunked transfer encoding

#### Task 2.3: Session-Aware SSE Streaming
**Duration**: 3 hours  
**Priority**: HIGH  
**Files**: `src/proxy/reverse.rs`

**Deliverables**:
- [ ] Extract Mcp-Session-Id from headers
- [ ] Link SSE streams to sessions
- [ ] Handle Last-Event-ID for resumption
- [ ] Implement session expiry for SSE
- [ ] Clean up streams on session end

#### Task 2.4: Server-Initiated Events
**Duration**: 3 hours  
**Priority**: MEDIUM  
**Files**: `src/proxy/reverse.rs`

**Deliverables**:
- [ ] Queue server notifications per session
- [ ] Send queued events on GET /mcp
- [ ] Handle connection drops gracefully
- [ ] Implement event buffering limits
- [ ] Add heartbeat/keepalive support

## Phase 3: Interceptor Streaming Support

### Goal
Enable the interceptor to handle SSE event streams.

### Tasks

#### Task 3.1: Streaming Context
**Duration**: 3 hours  
**Priority**: MEDIUM  
**Files**: `src/interceptor/context.rs`

**Deliverables**:
- [ ] Extend InterceptContext for streams
- [ ] Add stream state tracking
- [ ] Support multi-message sequences
- [ ] Handle stream lifecycle events
- [ ] Add SSE-specific metadata

#### Task 3.2: SSE Event Interception
**Duration**: 4 hours  
**Priority**: MEDIUM  
**Files**: `src/interceptor/engine.rs`, `src/interceptor/actions.rs`

**Deliverables**:
- [ ] Intercept individual SSE events
- [ ] Support event modification
- [ ] Enable event filtering/dropping
- [ ] Add stream pause/resume actions
- [ ] Implement event injection

#### Task 3.3: Streaming Rules
**Duration**: 2 hours  
**Priority**: LOW  
**Files**: `src/interceptor/rules.rs`

**Deliverables**:
- [ ] Add SSE-specific rule conditions
- [ ] Support event type matching
- [ ] Enable stream-based patterns
- [ ] Add rate limiting for events
- [ ] Document streaming rule syntax

## Phase 4: Integration Testing

### Goal
Ensure end-to-end SSE functionality works correctly.

### Tasks

#### Task 4.1: Forward Proxy SSE Tests
**Duration**: 2 hours  
**Priority**: HIGH  
**Files**: `tests/integration/forward_proxy_sse.rs` (new)

**Test Scenarios**:
- [ ] Connect to SSE endpoint
- [ ] Send request, receive SSE stream
- [ ] Handle connection drops
- [ ] Verify session continuity
- [ ] Test concurrent connections

#### Task 4.2: Reverse Proxy Streamable HTTP Tests
**Duration**: 3 hours  
**Priority**: HIGH  
**Files**: `tests/integration/reverse_proxy_sse.rs` (new)

**Test Scenarios**:
- [ ] POST returning JSON
- [ ] POST returning SSE stream
- [ ] GET opening SSE stream
- [ ] Session ID validation
- [ ] Stream resumption

#### Task 4.3: End-to-End MCP Flow Tests
**Duration**: 3 hours  
**Priority**: HIGH  
**Files**: `tests/e2e/mcp_sse_flow.rs` (new)

**Test Scenarios**:
- [ ] Full MCP session over SSE
- [ ] Bidirectional communication
- [ ] Error handling
- [ ] Performance benchmarks
- [ ] Compliance validation

## Implementation Guidelines

### Design Principles
1. **Reuse existing SSE components** from Phase 1
2. **Maintain transport abstraction** - SSE should be swappable
3. **Session consistency** across HTTP and SSE
4. **Graceful degradation** when SSE not available
5. **Performance target**: < 5% overhead vs direct connection

### Code Organization
```
src/
├── transport/
│   ├── sse/           # Existing SSE implementation
│   ├── sse_transport.rs  # New: Transport trait impl
│   └── streamable_http.rs # New: Combined HTTP+SSE
├── proxy/
│   ├── forward.rs     # Modified: Add SSE support
│   └── reverse.rs     # Modified: Streamable HTTP
└── interceptor/
    ├── context.rs     # Modified: Streaming context
    └── streaming.rs   # New: SSE interception
```

### Session Management Strategy
1. **Unified session store** for HTTP and SSE
2. **Session ID mapping** between internal and MCP IDs
3. **Connection tracking** per session
4. **Automatic cleanup** on expiry/disconnect

### Error Handling
- **Connection failures**: Automatic reconnection with backoff
- **Session expiry**: Return 404, require re-initialization
- **Invalid requests**: Return appropriate HTTP status
- **Stream errors**: Send error events, don't drop connection

## Success Criteria

### Functional Requirements
- [ ] Forward proxy can connect to SSE MCP servers
- [ ] Reverse proxy implements full Streamable HTTP spec
- [ ] Interceptor can process SSE event streams
- [ ] Sessions work consistently across transports
- [ ] All MCP 2025-06-18 requirements met

### Performance Requirements
- [ ] < 5% latency overhead for SSE streams
- [ ] Support 1000+ concurrent SSE connections
- [ ] < 100MB memory for 1000 sessions
- [ ] Reconnection within 5 seconds

### Quality Requirements
- [ ] 90%+ test coverage for new code
- [ ] No clippy warnings
- [ ] Comprehensive documentation
- [ ] Integration test suite passing

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Complex state management | HIGH | Leverage existing session infrastructure |
| Performance degradation | MEDIUM | Benchmark early and often |
| Breaking changes to existing code | MEDIUM | Feature flags for gradual rollout |
| SSE browser compatibility | LOW | Follow W3C SSE specification |

## Dependencies

### External Dependencies
- Existing Phase 1 SSE implementation (80% complete)
- Session management system (complete)
- Protocol version negotiation (complete)

### Internal Dependencies
- Forward proxy architecture
- Reverse proxy HTTP server
- Interceptor engine
- Transport abstraction

## Timeline Estimate

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Phase 1: Forward Proxy | 8-10 hours | Phase 1 SSE complete |
| Phase 2: Reverse Proxy | 13-15 hours | None |
| Phase 3: Interceptor | 9-10 hours | Phase 1 & 2 |
| Phase 4: Testing | 8 hours | All phases |
| **Total** | **38-43 hours** | |

## Next Steps

### Immediate Actions
1. Complete Phase 1 Task 1.5 (Performance Optimization)
2. Create SSE transport CLI design
3. Plan transport abstraction refactoring

### Prerequisites for Starting
- [ ] Phase 1 SSE implementation complete (1 task remaining)
- [ ] Review MCP 2025-06-18 specification
- [ ] Design transport wrapper interface
- [ ] Plan testing strategy

## References

### MCP Specifications
- [MCP 2025-06-18 Streamable HTTP](../../specs/mcp/docs/specification/2025-06-18/transports/http-sse.md)
- [MCP 2025-06-18 Transport Overview](../../specs/mcp/docs/specification/2025-06-18/transports/index.mdx)
- [MCP 2025-03-26 Basic Transport](../../specs/mcp/docs/specification/2025-03-26/basic/transports.mdx)

### Implementation References
- [Phase 1 SSE Implementation](../mcp-compliance/compliance-tracker.md)
- [SSE Module Documentation](../../shadowcat/src/transport/sse/mod.rs)
- [Session Management](../../shadowcat/src/session/mod.rs)

### Related Documents
- [Task 1.4 Session Integration Review](../mcp-compliance/implementation-notes/task-1.4-session-integration-review.md)
- [MCP Compliance Tracker](../mcp-compliance/compliance-tracker.md)

---

## Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2025-08-08 | 1.0 | Initial tracker creation based on gap analysis |