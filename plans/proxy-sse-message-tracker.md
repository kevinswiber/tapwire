# Unified Proxy-SSE-Message Tracker

## Overview

This is the primary tracker for implementing SSE proxy integration with MCP message handling capabilities. It interleaves work from both initiatives to maximize code reuse and ensure components work together seamlessly.

**Last Updated**: 2025-08-10  
**Total Estimated Duration**: ~~120-140 hours~~ → 118-138 hours (F.5 exists from refactor)  
**Status**: Phase 0 Complete ✅, Phase 1 Complete ✅, Phase 2: 100% Complete ✅

## Transport Naming Clarification

**Important**: The MCP specification terminology can be confusing:
- **MCP 2024-11-05**: Called it "HTTP+SSE transport" (deprecated)
- **MCP 2025-03-26**: Renamed to "Streamable HTTP" (with batching support)
- **MCP 2025-06-18**: Still "Streamable HTTP" (batching removed, version header added)

Both "HTTP+SSE" and "Streamable HTTP" refer to the **same transport mechanism**:
- HTTP POST for client → server messages
- Optional SSE (Server-Sent Events) for server → client streaming
- Session management via headers

**Current CLI Design Issue**: Our CLI has separate `--transport http` and `--transport sse` options, which is confusing since "Streamable HTTP" uses both HTTP and SSE. This should be addressed in a future refactor to have clearer naming like:
- `--transport stdio` - Process stdio communication
- `--transport streamable-http` - HTTP with optional SSE (the MCP remote transport)
- `--transport http-only` - Plain HTTP without SSE (if needed)

## Goals

1. **Enable SSE Transport** in forward and reverse proxies
2. **Add MCP Message Understanding** at the protocol level
3. **Implement Intelligent Interception** based on MCP semantics
4. **Create Recording and Replay** systems for MCP sessions
5. **Support Both MCP Versions** (2025-03-26 with batching, 2025-06-18)

## Architecture Vision

```
┌──────────────────────────────────────────────────────────┐
│                      User CLI                             │
├──────────────────────────────────────────────────────────┤
│                   Forward Proxy                           │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐        │
│  │   Stdio    │  │    SSE     │  │    HTTP    │        │
│  │ Transport  │  │ Transport  │  │ Transport  │        │
│  └────────────┘  └────────────┘  └────────────┘        │
├──────────────────────────────────────────────────────────┤
│                 MCP Message Layer                         │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐        │
│  │   Parser   │  │ Correlator │  │   Batch    │        │
│  │            │  │            │  │  Handler   │        │
│  └────────────┘  └────────────┘  └────────────┘        │
├──────────────────────────────────────────────────────────┤
│              Interceptor / Recorder / Replay              │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐        │
│  │   Rules    │  │   Tapes    │  │  Playback  │        │
│  │   Engine   │  │  Storage   │  │   Engine   │        │
│  └────────────┘  └────────────┘  └────────────┘        │
├──────────────────────────────────────────────────────────┤
│                   Reverse Proxy                           │
│  ┌────────────────────────────────────────────┐          │
│  │        /mcp Endpoint (POST + GET)          │          │
│  └────────────────────────────────────────────┘          │
└──────────────────────────────────────────────────────────┘
```

## Work Phases

### ✅ PREREQUISITE COMPLETE: Transport Context Refactor
**COMPLETED 2025-08-08**: The transport layer has been successfully refactored with the MessageEnvelope system.

**Actual Duration**: 17.5 hours (71% faster than estimate)  
**Result**: MessageEnvelope and TransportContext::Sse ready for immediate use  
See [Transport Context Refactor Tracker](transport-context-refactor/transport-context-tracker.md) for details.

#### Available Foundation from Refactor
- `MessageEnvelope`: Complete message with context wrapper
- `MessageContext`: Session ID, direction, transport metadata, timestamp
- `MessageDirection`: ClientToServer/ServerToClient enum
- `TransportContext::Sse`: SSE-specific fields (event_id, event_type, retry_ms, headers)
- `ProtocolMessage`: Core message type (replaces TransportMessage)

### Phase 0: Foundation Components (Week 1)
Build shared components that both SSE and MCP initiatives need.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| F.1 | **Create Protocol Version Manager** | 2h | None | ✅ Completed | 2025-08-08 | [Task Details](#f1-protocol-version-manager) |
| F.2 | **Build Minimal MCP Parser** | 4h | None | ✅ Completed | 2025-08-08 | [Task Details](#f2-minimal-mcp-parser) |
| F.3 | **Implement Batch Handler** | 3h | F.1, F.2 | ✅ Completed | 2025-08-08 | [Task Details](#f3-batch-handler) |
| F.4 | **Create Unified Event ID Generator** | 2h | None | ✅ Completed | 2025-08-10 | [Task Details](#f4-event-id-generator) |
| F.5 | **~~Build Message Context~~** ⚠️ | ~~2h~~ | ~~F.1~~ | ✅ Exists | Refactor | MessageContext in envelope.rs |

**Phase 0 Total**: ~~13 hours~~ → 11 hours (F.5 already exists)

### Phase 1: SSE Transport with MCP Awareness (Week 1-2)
Implement SSE transport that understands MCP messages from the start.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| S.1 | Add SSE Transport CLI Option | 2h | None | ✅ Completed | 2025-08-10 | [Details](sse-proxy-integration/tasks/task-1.1-cli-sse-option.md) |
| S.2 | **Create MCP-Aware SSE Transport Wrapper** | 4h | F.1-F.4, S.1 | ✅ Completed | 2025-08-10 | Uses MessageContext from refactor |
| S.2.5 | **Fix CLI Transport Naming Confusion** | 1h | S.1, S.2 | ✅ Completed | 2025-08-10 | [Details](sse-proxy-integration/tasks/task-1.2.5-fix-cli-naming.md) |
| S.3 | Integrate with Forward Proxy | 3h | S.2 | ✅ Completed | 2025-08-10 | ForwardProxy now supports SSE |
| S.4 | **Add MCP Parser Hooks to Transport** | 2h | S.2, F.2 | ✅ Completed | 2025-08-10 | Parser integrated in send/receive |

**Phase 1 Total**: ~~11 hours~~ → 12 hours (added S.2.5)

### Phase 2: Reverse Proxy Streamable HTTP (Week 2)
Implement the `/mcp` endpoint with MCP message understanding.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| R.1 | **Create MCP-Aware Dual-Method Endpoint** | 3h | F.1-F.3 | ✅ Complete | 2025-08-10 | [Modified Task 2.1](sse-proxy-integration/tasks/task-2.1-dual-method-endpoint.md) |
| R.2 | Implement SSE Response Handler | 4h | R.1, F.4 | ✅ Complete | 2025-08-10 | [Task 2.2](sse-proxy-integration/tasks/task-2.2-sse-response-handler.md) |
| R.3 | Session-Aware SSE Streaming | 3h | R.2 | ✅ Complete | 2025-08-10 | Integrated with R.1/R.2 |
| R.4 | **Add Early Message Correlation** | 2h | R.1, F.2 | ✅ Complete | 2025-08-10 | Integrated UnifiedEventIdGenerator |

**Phase 2 Total**: 12 hours

### Phase 3: Full MCP Parser and Correlation (Week 3)
Complete the MCP message parser and correlation engine.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| M.1 | Complete MCP Message Types | 4h | F.2 | ⬜ Not Started | | [MCP Task 1.1](mcp-message-handling/mcp-message-handling-tracker.md#task-11-core-message-types) |
| M.2 | Full Message Parser | 3h | M.1 | ⬜ Not Started | | [MCP Task 1.2](mcp-message-handling/mcp-message-handling-tracker.md#task-12-message-parser) |
| M.3 | Message Builder API | 2h | M.1 | ⬜ Not Started | | [MCP Task 1.3](mcp-message-handling/mcp-message-handling-tracker.md#task-13-message-builder) |
| M.4 | Correlation Engine | 5h | M.1 | ⬜ Not Started | | [MCP Task 2.1](mcp-message-handling/mcp-message-handling-tracker.md#task-21-correlation-engine) |
| M.5 | **Wire Correlation to SSE Transport** | 2h | M.4, S.4 | ⬜ Not Started | | [Task Details](#m5-wire-correlation) |

**Phase 3 Total**: 16 hours

### Phase 4: MCP-Aware Interceptor (Week 3-4)
Enable intelligent message interception based on MCP semantics.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| I.1 | Message Interceptor Interface | 4h | M.1 | ⬜ Not Started | | [MCP Task 3.1](mcp-message-handling/interceptor-mcp-spec.md) |
| I.2 | Method-Based Rules Engine | 5h | I.1 | ⬜ Not Started | | [MCP Task 3.2](mcp-message-handling/interceptor-mcp-spec.md) |
| I.3 | Interceptor Chain Integration | 3h | I.2 | ⬜ Not Started | | [MCP Task 3.3](mcp-message-handling/interceptor-mcp-spec.md) |
| I.4 | **SSE Stream Interception** | 3h | I.3, S.4 | ⬜ Not Started | | [Task Details](#i4-stream-interception) |
| I.5 | **Reverse Proxy Interception** | 2h | I.3, R.4 | ⬜ Not Started | | [Task Details](#i5-reverse-interception) |

**Phase 4 Total**: 17 hours

### Phase 5: MCP-Aware Recorder (Week 4)
Record MCP sessions with full semantic understanding.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.1 | MCP Tape Format | 4h | M.1 | ⬜ Not Started | | [MCP Task 4.1](mcp-message-handling/recorder-mcp-spec.md) |
| C.2 | Session Recorder | 5h | C.1, M.4 | ⬜ Not Started | | [MCP Task 4.2](mcp-message-handling/recorder-mcp-spec.md) |
| C.3 | Storage Backend | 3h | C.1 | ⬜ Not Started | | [MCP Task 4.3](mcp-message-handling/recorder-mcp-spec.md) |
| C.4 | **SSE Recording Integration** | 2h | C.2, S.4 | ⬜ Not Started | | [Task Details](#c4-sse-recording) |
| C.5 | **Reverse Proxy Recording** | 2h | C.2, R.4 | ⬜ Not Started | | [Task Details](#c5-reverse-recording) |

**Phase 5 Total**: 16 hours

### Phase 6: MCP-Aware Replay (Week 5)
Enable intelligent replay of recorded sessions.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| P.1 | Replay Engine Core | 5h | C.1 | ⬜ Not Started | | [MCP Task 5.1](mcp-message-handling/replay-mcp-spec.md) |
| P.2 | Replay Controller | 4h | P.1 | ⬜ Not Started | | [MCP Task 5.2](mcp-message-handling/replay-mcp-spec.md) |
| P.3 | Message Transformations | 3h | P.1 | ⬜ Not Started | | [MCP Task 5.3](mcp-message-handling/replay-mcp-spec.md) |
| P.4 | **SSE Replay Support** | 3h | P.1, S.2 | ⬜ Not Started | | [Task Details](#p4-sse-replay) |

**Phase 6 Total**: 15 hours

### Phase 7: Testing and Integration (Week 5-6)
Comprehensive testing of the integrated system.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| T.1 | Forward Proxy SSE Tests | 2h | S.3 | ⬜ Not Started | | [SSE Task 4.1](sse-proxy-integration/sse-proxy-integration-tracker.md#task-41-forward-proxy-sse-tests) |
| T.2 | Reverse Proxy Streamable HTTP Tests | 3h | R.3 | ⬜ Not Started | | [SSE Task 4.2](sse-proxy-integration/sse-proxy-integration-tracker.md#task-42-reverse-proxy-streamable-http-tests) |
| T.3 | End-to-End MCP Flow Tests | 3h | All | ⬜ Not Started | | [SSE Task 4.3](sse-proxy-integration/sse-proxy-integration-tracker.md#task-43-end-to-end-mcp-flow-tests) |
| T.4 | MCP Parser Conformance Tests | 2h | M.2 | ⬜ Not Started | | Validate against spec |
| T.5 | Correlation Engine Tests | 2h | M.4 | ⬜ Not Started | | Request-response matching |
| T.6 | Interceptor Integration Tests | 3h | I.5 | ⬜ Not Started | | Rule processing |
| T.7 | Recorder/Replay Tests | 3h | P.4 | ⬜ Not Started | | Full cycle testing |
| T.8 | Performance Benchmarks | 4h | All | ⬜ Not Started | | < 5% overhead target |

**Phase 7 Total**: 22 hours

## Next Steps: Begin Phase 3

With Phase 2 complete, we're ready to implement the full MCP parser and correlation engine:

1. **P.1: Create Full MCP Parser** (6h) - Build comprehensive message parser with validation
2. **P.2: Add Schema Validation** (4h) - Validate against MCP JSON-RPC schema
3. **P.3: Implement Correlation Store** (5h) - Track request/response pairs
4. **P.4: Add Request/Response Matching** (4h) - Match responses to requests
5. **P.5: Integrate with Proxy** (5h) - Wire parser into forward/reverse proxies

## Glue Tasks Details

These are the new tasks created specifically to connect the two initiatives:

### F.1: Protocol Version Manager ✅
**File**: `src/mcp/protocol.rs` (Completed 2025-08-08)
- Single source of truth for protocol version handling
- Used by both SSE transport and MCP parser
- Handles version negotiation and capability detection
- Enum-based version management with capability checks
- 22 comprehensive tests

### F.2: Minimal MCP Parser ✅
**File**: `src/mcp/early_parser.rs` (Completed 2025-08-08)
- Lightweight parser for immediate use by SSE
- Extract method, ID, and batch detection
- Foundation for full parser in Phase 3
- Handles all message types (request, response, notification)
- SSE data formatting support
- 37 comprehensive tests

### F.3: Batch Handler ✅
**File**: `src/mcp/batch.rs` (Completed 2025-08-08)
- Shared logic for MCP 2025-03-26 batch messages
- Split batches into individual messages
- Combine responses into batches when needed
- Group messages by type and method
- Validate batch structure according to protocol rules
- 18 comprehensive tests covering all functionality

### F.4: Event ID Generator ✅
**File**: `src/mcp/event_id.rs` (Completed 2025-08-10)
- Generate IDs that work for both SSE events and MCP correlation
- Extract correlation info from event IDs  
- Maintain uniqueness across sessions
- Thread-safe ID generation using AtomicU64
- Support for session ID and JSON-RPC ID correlation
- SSE-compatible (newlines replaced with underscores)
- 17 comprehensive tests including thread safety

### R.4: Early Message Correlation ✅
**Implementation**: Integrated into `src/proxy/reverse.rs` (Completed 2025-08-10)
- Added UnifiedEventIdGenerator to reverse proxy AppState
- Enhanced SSE proxy to generate correlation IDs for all events
- Parse MCP messages from SSE data fields to extract JSON-RPC IDs
- Generate session-aware event IDs with format: `{session}-{node}-{json_rpc_id}-{counter}`
- Preserve upstream event IDs while adding correlation info
- Handle both requests/responses and notifications
- Tests updated and passing

### F.5: Message Context
**File**: `src/mcp/context.rs`
- Unified context structure for all message processing
- Tracks session, transport, direction, correlation
- Used by interceptor, recorder, and replay

### S.4: Parser Hooks
**Enhancement to**: `src/transport/sse_transport.rs`
- Integrate minimal parser into SSE transport
- Extract correlation hints from messages
- Prepare context for MCP processing

### R.4: Early Correlation
**Enhancement to**: `src/proxy/reverse/mcp_endpoint.rs`
- Track request IDs in reverse proxy
- Prepare for response matching
- Store correlation context

### M.5: Wire Correlation
**Integration**: Connect correlation engine to SSE transport
- Update SSE transport to use correlation engine
- Track all request-response pairs
- Enable correlation metrics

### I.4: Stream Interception
**File**: `src/interceptor/sse_stream.rs`
- Apply MCP interceptors to SSE event streams
- Handle streaming message sequences
- Maintain stream context

### I.5: Reverse Interception
**Enhancement to**: `src/proxy/reverse.rs`
- Apply interceptors at reverse proxy
- Server-side rule processing
- Response modification

### C.4: SSE Recording
**Integration**: Connect recorder to SSE transport
- Record all SSE messages with MCP context
- Include correlation IDs
- Track session lifecycle

### C.5: Reverse Recording
**Integration**: Connect recorder to reverse proxy
- Record server-side interactions
- Include HTTP metadata
- Track request-response pairs

### P.4: SSE Replay
**File**: `src/replay/sse_replay.rs`
- Replay recorded sessions via SSE transport
- Handle reconnection and resumption
- Transform messages during replay

## Progress Tracking

### Week 1 (Phase 0 + Start Phase 1)
- [ ] F.1: Protocol Version Manager
- [ ] F.2: Minimal MCP Parser
- [ ] F.3: Batch Handler
- [ ] F.4: Event ID Generator
- [ ] F.5: Message Context
- [ ] S.1: CLI Options

### Week 2 (Complete Phase 1 + Phase 2)
- [ ] S.2: SSE Transport Wrapper
- [ ] S.3: Forward Proxy Integration
- [ ] S.4: Parser Hooks
- [ ] R.1: Dual-Method Endpoint
- [ ] R.2: SSE Response Handler
- [ ] R.3: Session Streaming
- [ ] R.4: Early Correlation

### Week 3 (Phase 3 + Start Phase 4)
- [ ] M.1-M.5: Full Parser and Correlation
- [ ] I.1-I.2: Interceptor Foundation

### Week 4 (Complete Phase 4 + Phase 5)
- [ ] I.3-I.5: Interceptor Integration
- [ ] C.1-C.5: Recorder Implementation

### Week 5 (Phase 6 + Start Phase 7)
- [ ] P.1-P.4: Replay System
- [ ] T.1-T.3: Initial Testing

### Week 6 (Complete Phase 7)
- [ ] T.4-T.8: Comprehensive Testing
- [ ] Documentation and Cleanup

## Success Criteria

### Functional Requirements
- ✅ SSE transport works in forward and reverse proxy
- ✅ MCP messages parsed and understood
- ✅ Request-response correlation working
- ✅ Interceptor rules apply to MCP messages
- ✅ Sessions recorded with full context
- ✅ Replay works with transformations
- ✅ Both MCP versions supported

### Performance Requirements
- ✅ < 5% latency overhead
- ✅ < 100MB memory for 1000 sessions
- ✅ Support 10,000 messages/second
- ✅ < 1ms message parsing time

### Quality Requirements
- ✅ 90% test coverage
- ✅ No clippy warnings
- ✅ Full documentation
- ✅ Integration tests passing

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Parser performance | HIGH | Build minimal parser first, optimize later | Planned |
| Complex dependencies | HIGH | Foundation phase builds shared components | Planned |
| Integration issues | MEDIUM | Glue tasks connect components cleanly | Planned |
| Scope creep | MEDIUM | Phased approach, clear success criteria | Active |

## Related Documents

### SSE Proxy Integration
- [SSE Proxy Integration Tracker](sse-proxy-integration/sse-proxy-integration-tracker.md)
- [Task 1.1: CLI Options](sse-proxy-integration/tasks/task-1.1-cli-sse-option.md)
- [Task 1.2: Transport Wrapper](sse-proxy-integration/tasks/task-1.2-sse-transport-wrapper.md)
- [Task 2.1: Dual-Method Endpoint](sse-proxy-integration/tasks/task-2.1-dual-method-endpoint.md)
- [Task 2.2: Response Handler](sse-proxy-integration/tasks/task-2.2-sse-response-handler.md)
- [MCP 2025-03-26 Compatibility](sse-proxy-integration/tasks/compatibility-2025-03-26.md)

### MCP Message Handling
- [MCP Message Handling Tracker](mcp-message-handling/mcp-message-handling-tracker.md)
- [Interceptor Specification](mcp-message-handling/interceptor-mcp-spec.md)
- [Recorder Specification](mcp-message-handling/recorder-mcp-spec.md)
- [Replay Specification](mcp-message-handling/replay-mcp-spec.md)

### Coordination
- [Integration Coordination Guide](integration-coordination.md)

## Session Planning Guidelines

### Optimal Session Structure
1. **Review** (5 min): Check this tracker and relevant task files
2. **Implementation** (2-3 hours): Complete the task deliverables
3. **Testing** (30 min): Run tests, fix issues
4. **Documentation** (15 min): Update tracker, create PR if needed
5. **Handoff** (10 min): Update NEXT_SESSION_PROMPT.md if needed

### Using the rust-code-reviewer
For complex Rust implementation tasks, consider using the `rust-code-reviewer` subagent to:
- Review memory safety and ownership patterns
- Validate async/await correctness with tokio
- Check for performance optimizations
- Ensure proper error handling with Result types
- Verify test coverage for critical paths

### Context Window Management
- Each task is designed to require < 50% context window
- If approaching 70% usage, create NEXT_SESSION_PROMPT.md
- Keep focus on single task to avoid context bloat
- Reference documentation only when needed

### Task Completion Criteria
- [ ] All deliverables checked off
- [ ] Tests passing
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Tracker status updated

## Critical Implementation Guidelines

### Proxy Mode Parity
**ALWAYS implement changes in BOTH proxy modes:**
- **Forward Proxy** (`src/proxy/forward.rs`): Client → Shadowcat → Server
- **Reverse Proxy** (`src/proxy/reverse.rs`): Client → Shadowcat (HTTP) → Server

When implementing any MCP compliance feature:
1. ✅ Implement in forward proxy
2. ✅ Implement in reverse proxy  
3. ✅ Add tests for both modes
4. ✅ Verify behavior consistency

**Common oversights:**
- Version tracking (must track in both modes)
- Error handling (must be consistent)
- Session state management (must be synchronized)
- Protocol validation (must enforce equally)

## Communication Protocol

### Status Updates
After completing each task, update:
1. Task status in this tracker
2. Completion date and notes
3. Any new issues discovered
4. Next recommended task

### Handoff Notes
If context window becomes limited:
1. Save progress to NEXT_SESSION_PROMPT.md
2. Include:
   - Current task status
   - Completed deliverables
   - Remaining work
   - Any blockers or decisions needed

## Next Actions

1. **Review and approve this unified plan**
2. **Start Phase 0: Foundation Components**
3. **Build Protocol Version Manager (F.1)**
4. **Build Minimal MCP Parser (F.2)**
5. **Continue with Phase 1 once foundations ready**

## Session History

### 2025-08-10 Session
**Duration**: ~4 hours  
**Completed**:
- ✅ S.1: Add SSE Transport CLI Option
  - Added `ForwardTransport::Sse` variant with URL and retry configuration
  - Implemented `run_sse_forward_proxy` handler function
- ✅ S.2: Create MCP-Aware SSE Transport Wrapper
  - Created `SseTransport` implementing the `Transport` trait
  - Integrated `UnifiedEventIdGenerator` for correlation
  - Connected with existing `SseHttpClient` and `SseConnectionManager`
  - Added MCP protocol version tracking
  - Implemented send/receive with proper SSE context wrapping

**Key Decisions**:
- Used existing SSE components rather than reimplementing
- Event IDs embed correlation info for request-response matching
- SSE transport initialization defers actual connection to first use

**Issues Identified**:
- CLI naming confusion between `http` and `sse` transports
- Both actually implement "Streamable HTTP" from MCP spec
- Should be refactored to clearer naming in future

## Notes

- This tracker supersedes individual SSE and MCP trackers for execution
- Original trackers remain as reference documentation
- Glue tasks ensure smooth integration between initiatives
- Phases can overlap where dependencies allow
- Testing is integrated throughout, not just at the end

---

**Document Version**: 1.1  
**Created**: 2025-08-08  
**Last Modified**: 2025-08-10  
**Author**: Development Team