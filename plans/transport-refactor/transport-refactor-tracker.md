# Transport Layer Refactor: IncomingTransport/OutgoingTransport Architecture

## Overview

This tracker manages the refactoring of Shadowcat's transport layer to introduce clearer `IncomingTransport` and `OutgoingTransport` abstractions, addressing current architectural confusion and enabling proper support for MCP's Streamable HTTP protocol.

**Last Updated**: 2025-08-13  
**Total Estimated Duration**: 40-50 hours  
**Status**: Phase 0 Complete - Ready for Phase 1  
**Priority**: High (Now that SSE/MCP is complete)

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

## Session Planning Guidelines

### Next Session Prompt
This plan has a corresponding `next-session-prompt.md` file in this directory, based on the template in `plans/template/next-session-prompt.md`. This file should be updated at the end of each session to set up the next session with proper context.

### Optimal Session Structure
1. **Review** (5 min): Check this tracker and relevant task files
2. **Implementation** (2-3 hours): Complete the task deliverables
3. **Testing** (30 min): Run tests, fix issues
4. **Documentation** (15 min): Update tracker, create PR if needed
5. **Handoff** (10 min): Update next-session-prompt.md in this directory

## Work Phases

### Phase 0: Prerequisites and Analysis (Week 1)
Understand the current state and prepare for safe refactoring.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.0 | ~~Complete SSE/MCP work~~ | - | None | ✅ Complete | | Done 2025-08-13 |
| A.1 | Document existing transport patterns | 3h | None | ✅ Complete | 2025-08-13 | Comprehensive analysis created |
| A.2 | Create test suite for current behavior | 4h | A.1 | ✅ Complete | 2025-08-13 | 16 regression tests created |
| A.3 | Identify breaking change risks | 2h | A.1 | ✅ Complete | 2025-08-13 | Risk assessment complete |

**Phase 0 Total**: 9 hours

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

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

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

## Progress Tracking

### Week 1 (Starting 2025-08-13)
- [ ] A.1: Document existing transport patterns
- [ ] A.2: Create test suite for current behavior
- [ ] A.3: Identify breaking change risks
- [ ] F.1: Design RawTransport trait hierarchy
- [ ] F.2: Design ProtocolHandler abstraction

### Completed Tasks
- [x] A.0: Complete SSE/MCP work - Completed 2025-08-13

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

### Task Files
- [Task F.3: Incoming/Outgoing Traits](tasks/F.3-incoming-outgoing-traits.md) - Core trait design
- Task files should follow the structure defined in `plans/template/task.md`

### Primary References
- [SSE/MCP Tracker](../proxy-sse-message-tracker.md) - Recently completed work
- [Original Architecture Plan](../002-shadowcat-architecture-plan.md) - Original design
- [Transport Context Refactor](../transport-context-refactor/transport-context-tracker.md) - Previous refactor

---

**Document Version**: 1.0  
**Created**: 2025-08-12  
**Author**: Architecture Team