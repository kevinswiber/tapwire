# Transport Layer Refactor: IncomingTransport/OutgoingTransport Architecture

## Overview

This tracker manages the refactoring of Shadowcat's transport layer to introduce clearer `IncomingTransport` and `OutgoingTransport` abstractions, addressing current architectural confusion and enabling proper support for MCP's Streamable HTTP protocol.

**Created**: 2025-08-12  
**Status**: Planning  
**Estimated Duration**: 40-50 hours  
**Priority**: Medium (can wait until after current SSE/MCP work)

## Problem Statement

The current transport architecture has several issues:

1. **Mixing of Concerns**: Transport mechanics mixed with protocol semantics and process management
2. **Confusing Abstractions**: `StdioTransport` vs `StdioClientTransport` confusion
3. **Artificial Separation**: HTTP and SSE treated as separate when MCP uses both together
4. **Unclear Direction**: Connection direction confused with data flow direction

## Goals

1. **Clear Separation**: Separate transport mechanics from protocol handling
2. **Better Abstractions**: `IncomingTransport` for proxy-exposed transports, `OutgoingTransport` for upstream connections
3. **Unified Streamable HTTP**: Single transport for MCP's HTTP POST + SSE combination
4. **Improved Testability**: Clean interfaces for mocking and testing

## Architecture Vision

### Current (Confusing)
```
Transport (trait)
├── StdioTransport (spawns subprocess - actually outgoing)
├── StdioClientTransport (reads stdin - actually incoming)
├── HttpTransport (HTTP client - outgoing)
├── HttpMcpTransport (HTTP server - incoming)
├── SseTransport (SSE client - outgoing)
└── [Missing: Unified Streamable HTTP]
```

### Proposed (Clear)
```
IncomingTransport (proxy accepts these)
├── StdioIncoming (read from stdin)
├── HttpServerIncoming (HTTP server)
└── StreamableHttpIncoming (HTTP server + SSE responses)

OutgoingTransport (proxy connects to these)
├── SubprocessOutgoing (spawn subprocess)
├── HttpClientOutgoing (HTTP client)
└── StreamableHttpOutgoing (HTTP POST + SSE client)
```

## Design Principles

### 1. Layer Separation
```rust
// Layer 1: Raw transport (bytes only)
trait RawTransport {
    async fn send_bytes(&mut self, data: &[u8]) -> Result<()>;
    async fn receive_bytes(&mut self) -> Result<Vec<u8>>;
}

// Layer 2: Protocol handling (MCP/JSON-RPC)
trait ProtocolHandler {
    fn serialize(&self, msg: &ProtocolMessage) -> Result<Vec<u8>>;
    fn deserialize(&self, data: &[u8]) -> Result<ProtocolMessage>;
}

// Layer 3: Direction-aware transport
trait IncomingTransport {
    async fn receive_request(&mut self) -> Result<MessageEnvelope>;
    async fn send_response(&mut self, response: MessageEnvelope) -> Result<()>;
}

trait OutgoingTransport {
    async fn send_request(&mut self, request: MessageEnvelope) -> Result<()>;
    async fn receive_response(&mut self) -> Result<MessageEnvelope>;
}
```

### 2. Process Management Separation
```rust
// Separate from transport
trait ProcessManager {
    async fn spawn(&mut self, command: &Command) -> Result<ProcessHandle>;
    async fn terminate(&mut self, handle: ProcessHandle) -> Result<()>;
}
```

## Session Management

**Important**: To avoid collision with ongoing SSE/MCP work, this refactor uses a separate session prompt file:
- **Session Prompt File**: `NEXT_SESSION_PROMPT_TRANSPORT_REFACTOR.md`
- **Do NOT use**: `NEXT_SESSION_PROMPT.md` (reserved for SSE/MCP work)

When working on this refactor:
1. Start sessions with `NEXT_SESSION_PROMPT_TRANSPORT_REFACTOR.md`
2. Update that file at session end
3. Keep work isolated from main SSE/MCP development

## Work Phases

### Phase 0: Prerequisites (Before Starting)
- [ ] Complete current SSE/MCP Phase 3-7 work
- [ ] Document all existing transport usage patterns
- [ ] Create comprehensive test suite for current behavior

### Phase 1: Foundation (Week 1)
| ID | Task | Duration | Dependencies | Status |
|----|------|----------|--------------|--------|
| F.1 | Design RawTransport trait hierarchy | 2h | None | ⬜ |
| F.2 | Design ProtocolHandler abstraction | 2h | None | ⬜ |
| F.3 | Design Incoming/Outgoing traits | 3h | F.1, F.2 | ⬜ |
| F.4 | Create ProcessManager trait | 2h | None | ⬜ |
| F.5 | Design migration strategy | 2h | F.1-F.4 | ⬜ |

**Phase 1 Total**: 11 hours

### Phase 2: Raw Transport Layer (Week 1-2)
| ID | Task | Duration | Dependencies | Status |
|----|------|----------|--------------|--------|
| R.1 | Implement StdioRawTransport | 3h | F.1 | ⬜ |
| R.2 | Implement HttpRawTransport | 3h | F.1 | ⬜ |
| R.3 | Implement SseRawTransport | 3h | F.1 | ⬜ |
| R.4 | Implement StreamableHttpRawTransport | 4h | R.2, R.3 | ⬜ |
| R.5 | Create RawTransport tests | 3h | R.1-R.4 | ⬜ |

**Phase 2 Total**: 16 hours

### Phase 3: Protocol Handler (Week 2)
| ID | Task | Duration | Dependencies | Status |
|----|------|----------|--------------|--------|
| P.1 | Extract McpProtocolHandler | 3h | F.2 | ⬜ |
| P.2 | Remove duplicate parsing code | 2h | P.1 | ⬜ |
| P.3 | Create protocol handler tests | 2h | P.1 | ⬜ |

**Phase 3 Total**: 7 hours

### Phase 4: Direction-Aware Transports (Week 2-3)
| ID | Task | Duration | Dependencies | Status |
|----|------|----------|--------------|--------|
| D.1 | Implement IncomingTransport types | 4h | F.3, R.1-R.4 | ⬜ |
| D.2 | Implement OutgoingTransport types | 4h | F.3, R.1-R.4 | ⬜ |
| D.3 | Update proxy to use new transports | 3h | D.1, D.2 | ⬜ |
| D.4 | Create direction-aware tests | 3h | D.1-D.3 | ⬜ |

**Phase 4 Total**: 14 hours

### Phase 5: Migration and Cleanup (Week 3)
| ID | Task | Duration | Dependencies | Status |
|----|------|----------|--------------|--------|
| M.1 | Migrate forward proxy | 3h | D.3 | ⬜ |
| M.2 | Migrate reverse proxy | 3h | D.3 | ⬜ |
| M.3 | Update CLI and factory | 2h | M.1, M.2 | ⬜ |
| M.4 | Remove old transport code | 1h | M.1-M.3 | ⬜ |
| M.5 | Update documentation | 2h | M.4 | ⬜ |

**Phase 5 Total**: 11 hours

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking existing functionality | HIGH | Comprehensive test suite before starting |
| Disrupting ongoing SSE/MCP work | HIGH | Wait until Phase 3-7 complete |
| Complex migration | MEDIUM | Gradual migration with compatibility layer |
| Performance regression | MEDIUM | Benchmark before/after each phase |

## Success Criteria

### Functional
- [ ] All existing tests pass with new architecture
- [ ] Streamable HTTP works as single transport
- [ ] Clear separation between incoming/outgoing
- [ ] Process management extracted from transports

### Quality
- [ ] Zero clippy warnings
- [ ] 95% test coverage on new code
- [ ] Performance within 2% of current
- [ ] Clear documentation and examples

### Architecture
- [ ] No protocol logic in transport layer
- [ ] No process management in transport layer
- [ ] Clear naming (no more StdioClient confusion)
- [ ] Unified handling of Streamable HTTP

## Dependencies on Current Work

This refactor should **wait until after**:
1. Current SSE/MCP tracker Phase 3-7 complete
2. All integration tests passing
3. Basic recording/replay working

Or run **in parallel** with careful coordination:
1. Create compatibility shim layer
2. New transports alongside old
3. Gradual migration per transport type

## CLI Changes

### Current (Confusing)
```bash
shadowcat forward stdio -- command
shadowcat forward http --url http://server
shadowcat forward sse --url http://server  # Actually Streamable HTTP!
```

### Proposed (Clear)
```bash
# Clear separation of from/to
shadowcat forward --from stdio --to subprocess -- command
shadowcat forward --from stdio --to streamable-http https://server/mcp

# Reverse proxy with clear upstream
shadowcat reverse --listen :8080 --upstream streamable-http https://server/mcp
shadowcat reverse --listen :8080 --upstream subprocess -- local-server
```

## Notes

- This refactor addresses architectural debt accumulated during rapid development
- Should improve maintainability and reduce confusion for new contributors
- Enables proper support for MCP's Streamable HTTP as designed
- Makes the codebase match the mental model of a proxy

## Related Documents

- [Current SSE/MCP Tracker](../proxy-sse-message-tracker.md) - Must coordinate with this
- [Original Architecture Plan](../002-shadowcat-architecture-plan.md) - Original design
- [Transport Context Refactor](../transport-context-refactor/transport-context-tracker.md) - Previous refactor

---

**Document Version**: 1.0  
**Created**: 2025-08-12  
**Author**: Architecture Team