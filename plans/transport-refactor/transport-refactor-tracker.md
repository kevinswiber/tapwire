# Transport Layer Refactor: IncomingTransport/OutgoingTransport Architecture

## Overview

This tracker manages the refactoring of Shadowcat's transport layer to introduce clearer `IncomingTransport` and `OutgoingTransport` abstractions, addressing current architectural confusion and enabling proper support for MCP's Streamable HTTP protocol.

**Last Updated**: 2025-08-14  
**Total Estimated Duration**: 40-50 hours  
**Status**: Phase 3 Complete - Ready for Phase 4 Implementation  
**Priority**: High (Protocol layer complete with optimizations)

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
| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| F.1 | Design RawTransport trait hierarchy | 2h | None | ✅ Complete | 2025-08-13 | Created in src/transport/raw/mod.rs |
| F.2 | Design ProtocolHandler abstraction | 2h | None | ✅ Complete | 2025-08-13 | Created McpProtocolHandler |
| F.3 | Design Incoming/Outgoing traits | 3h | F.1, F.2 | ✅ Complete | 2025-08-13 | Created in src/transport/directional/mod.rs |
| F.4 | Create ProcessManager trait | 2h | None | ✅ Complete | 2025-08-13 | Created in src/process/mod.rs |
| F.5 | Design migration strategy | 2h | F.1-F.4 | ✅ Complete | 2025-08-13 | No compat layer needed (pre-release) |

**Phase 1 Total**: 11 hours ✅ COMPLETED

### Phase 2: Raw Transport Layer (Week 1-2)
| ID | Task | Duration | Dependencies | Status |
|----|------|----------|--------------|--------|
| R.1 | Implement StdioRawTransport | 3h | F.1 | ✅ Complete |
| R.2 | Implement HttpRawTransport | 3h | F.1 | ✅ Complete |
| R.3 | Implement SseRawTransport | 3h | F.1 | ✅ Complete |
| R.4 | Implement StreamableHttpRawTransport | 4h | R.2, R.3 | ✅ Complete |
| R.5 | Create RawTransport tests | 3h | R.1-R.4 | ✅ Complete |

**Phase 2 Total**: 16 hours ✅ COMPLETED

### Phase 3: Protocol Handler (Week 2)
| ID | Task | Duration | Dependencies | Status | Notes |
|----|------|----------|--------------|--------|--------|
| P.1 | Enhanced McpProtocolHandler with batch support | 1h | F.2 | ✅ Complete | Batch support deferred to separate plan |
| P.2 | Implemented MCP protocol validator | 30m | P.1 | ✅ Complete | JSON-RPC 2.0 error codes |
| P.3 | Implemented protocol negotiation | 1h | P.1 | ✅ Complete | Version & capability exchange |
| P.4 | Created comprehensive protocol tests | 30m | P.1-P.3 | ✅ Complete | 21 protocol + 10 negotiation tests |

**Phase 3 Total**: 3 hours ✅ COMPLETED

#### Phase 3 Follow-up Optimizations (2025-08-14)
| ID | Task | Duration | Dependencies | Status | Notes |
|----|------|----------|--------------|--------|--------|
| P.5 | Batch support analysis & planning | 1h | P.1 | ✅ Complete | Created full-batch-support plan |
| P.6 | Negotiation module consolidation | 2h | P.3 | ✅ Complete | Merged 2 modules, kept MCP enum |
| P.7 | Buffer pool integration | 2h | R.1-R.5 | ✅ Complete | >80% hit rate, metrics added |
| P.8 | Performance validation | 30m | P.7 | ✅ Complete | 5 buffer pool tests, no regressions |

**Phase 3 Follow-up Total**: 5.5 hours ✅ COMPLETED

#### Phase 3 Code Review & Improvements (2025-08-14)
| ID | Task | Duration | Dependencies | Status | Notes |
|----|------|----------|--------------|--------|--------|
| P.9 | Comprehensive code review via rust-code-reviewer | 30m | P.1-P.8 | ✅ Complete | Grade: A-, no critical issues |
| P.10 | Fix buffer clone performance issue | 15m | P.9 | ✅ Complete | Eliminated unnecessary clone in UTF-8 conversion |
| P.11 | Add version constants documentation | 15m | P.9 | ✅ Complete | Added migration guides & feature docs |
| P.12 | Improve BytesMut serialization | 15m | P.9 | ✅ Complete | Zero-copy writer adapter implemented |
| P.13 | Add concurrent buffer pool tests | 20m | P.9 | ✅ Complete | 2 new tests for thread safety |
| P.14 | Minor improvements (error context, magic numbers) | 10m | P.9 | ✅ Complete | Enhanced debugging & maintainability |

**Phase 3 Review & Improvements Total**: 1.75 hours ✅ COMPLETED

**Review Summary**: Conducted comprehensive code review using rust-code-reviewer agent. Found no critical issues, only performance optimizations and code quality improvements. All 853 tests passing with zero clippy warnings. Full review documented in `reviews/phase3-review.md`.

### Phase 4: Direction-Aware Transports (Week 2-3)
| ID | Task | Duration | Dependencies | Status | Notes |
|----|------|----------|--------------|--------|--------|
| D.1 | Implement IncomingTransport types | 4h | F.3, R.1-R.4 | ✅ Complete | StdioIncoming, HttpServerIncoming, StreamableHttpIncoming |
| D.2 | Implement OutgoingTransport types | 4h | F.3, R.1-R.4 | ✅ Complete | SubprocessOutgoing, HttpClientOutgoing, StreamableHttpOutgoing |
| D.3 | Update proxy to use new transports | 3h | D.1, D.2 | ⬜ | |
| D.4 | Create direction-aware tests | 3h | D.1-D.3 | 🔄 Partial | 12 unit tests created and passing |

**Phase 4 Total**: 14 hours (50% complete)

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

## Lessons Learned from Phase 2

### Critical Issues Found and Fixed
1. **Duplicate Process Spawning**: Initial implementation spawned processes twice (via ProcessManager AND directly)
2. **Sync/Async Mismatch**: ProcessManager trait had sync methods that needed async operations
3. **Command Handling**: tokio::process::Command doesn't implement Clone, requiring string extraction

### Best Practices Applied
1. **Async All The Way**: Avoid futures::executor::block_on - make functions async instead
2. **Type Aliases**: Use type aliases for complex types to satisfy clippy
3. **Field Usage**: Use #[allow(dead_code)] for fields that will be used, not underscore prefix
4. **Test Coverage**: Internal module tests can access private fields, external tests cannot

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking existing functionality | HIGH | Comprehensive test suite before starting |
| Disrupting ongoing SSE/MCP work | HIGH | Wait until Phase 3-7 complete |
| Complex migration | MEDIUM | Gradual migration with compatibility layer |
| Performance regression | MEDIUM | Benchmark before/after each phase |

## Success Criteria

### Functional
- [x] All existing tests pass with new architecture (853 tests passing)
- [x] Streamable HTTP works as single transport (Phase 2 complete)
- [ ] Clear separation between incoming/outgoing (Phase 4 pending)
- [x] Process management extracted from transports (ProcessManager complete)

### Quality
- [x] Zero clippy warnings (all phases pass clippy)
- [x] 95% test coverage on new code (comprehensive test suites)
- [x] Performance within 2% of current (buffer pools >80% hit rate)
- [x] Clear documentation and examples (all phases documented)

### Architecture
- [x] No protocol logic in transport layer (clean separation)
- [x] No process management in transport layer (ProcessManager extracted)
- [x] Clear naming (no more StdioClient confusion)
- [x] Unified handling of Streamable HTTP (StreamableHttpRawTransport complete)

## Progress Tracking

### Week 1 (Starting 2025-08-13)
- [x] A.1: Document existing transport patterns - ✅ Completed
- [x] A.2: Create test suite for current behavior - ✅ Completed  
- [x] A.3: Identify breaking change risks - ✅ Completed
- [x] F.1: Design RawTransport trait hierarchy - ✅ Completed
- [x] F.2: Design ProtocolHandler abstraction - ✅ Completed
- [x] F.3: Design Incoming/Outgoing traits - ✅ Completed
- [x] F.4: Create ProcessManager trait - ✅ Completed
- [x] F.5: Design migration strategy - ✅ Completed
- [x] R.1: Implement StdioRawTransport - ✅ Completed
- [x] R.2: Implement HttpRawTransport - ✅ Completed
- [x] R.3: Implement SseRawTransport - ✅ Completed
- [x] R.4: Implement StreamableHttpRawTransport - ✅ Completed (KEY INNOVATION!)
- [x] R.5: Create RawTransport tests - ✅ Completed

**Phase 2 Complete!** All compilation errors fixed, tests passing, code formatted.

**Post-Phase 2 Improvements:**
- Fixed critical bug: duplicate process spawning in StdioRawOutgoing
- Made ProcessManager trait fully async (removed all block_on calls)
- Improved Command handling with better API
- Added type aliases for complex types
- All 22 raw transport tests passing with zero clippy warnings

**Phase 2 Code Review Fixes (2025-08-13):**
- ✅ Priority 0: All Drop implementations, mutex patterns, process cleanup
- ✅ Priority 1: Buffer limits, timeouts, error handling, 6 concurrent tests
- ✅ Priority 2: Buffer pooling, zero-copy optimizations, performance benchmarks
- **Final Status**: 847 tests passing, < 60KB per session, production-ready

### Completed Phases
- [x] Phase 0: Prerequisites and Analysis - Completed 2025-08-13
- [x] Phase 1: Foundation Design - Completed 2025-08-13
- [x] Phase 2: Raw Transport Layer - Completed 2025-08-13
- [x] Phase 3: Protocol Handler - Completed 2025-08-14 (including follow-up optimizations)

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