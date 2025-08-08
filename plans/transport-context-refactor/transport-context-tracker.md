# Transport Context Refactor Tracker

## Overview

This tracker coordinates the refactoring of Shadowcat's transport layer to properly separate protocol concerns (JSON-RPC messages) from transport-specific metadata (HTTP headers, SSE events, stdio). This is a prerequisite for SSE proxy integration and must be completed before continuing with the proxy-sse-message-tracker.md work.

**Last Updated**: 2025-08-08  
**Total Estimated Duration**: 30-40 hours  
**Status**: Planning

## Problem Statement

The current `TransportMessage` enum conflates multiple concerns across different protocol layers:

### Protocol Layer Confusion
1. **Transport Layer** (HTTP/SSE/stdio): How bytes are moved
   - HTTP: Only has request/response semantics
   - SSE: One-way server-to-client events  
   - stdio: Bidirectional byte streams

2. **MCP Protocol Layer**: Application-level semantics
   - Request/Response/Notification patterns
   - Bidirectional notifications (both client→server and server→client)
   - Session management and correlation

3. **JSON-RPC Layer**: Message framing and structure
   - ID-based request/response correlation
   - Method names and parameters
   - Error handling

The current `TransportMessage` appears to be at the MCP semantic level but lacks critical context:
- **Notifications lack direction** (who is the source/destination?)
- **No transport metadata** (HTTP headers, SSE event IDs)
- **No session context** (which session, what protocol version)

With SSE integration, we need to track transport-specific metadata like:
- SSE event IDs, event types, retry hints
- HTTP headers, status codes, content types
- Stream state and correlation IDs
- Session continuity across transports
- Message direction and routing information

The `TransportMessage` type is used in 90 files with 658 occurrences, making this a significant architectural change.

## Goals

1. **Separate Concerns** - Decouple protocol messages from transport metadata
2. **Maintain Compatibility** - Enable incremental migration without breaking existing code
3. **Enable SSE Integration** - Provide proper abstractions for SSE-specific requirements
4. **Improve Type Safety** - Make transport-specific handling explicit and type-safe
5. **Support Future Transports** - Create extensible architecture for WebSocket, gRPC, etc.

## Architecture Vision

```
Current Architecture:
┌─────────────────────────────────────┐
│        TransportMessage             │
│  (Protocol + Some Transport Mixed)  │
└─────────────────────────────────────┘
           ↓ Used by 90 files

Target Architecture:
┌─────────────────────────────────────┐
│      Protocol Layer                 │
│   TransportMessage (unchanged)      │
│  Request/Response/Notification      │
└─────────────────────────────────────┘
           ↓ Wrapped by
┌─────────────────────────────────────┐
│      Transport Layer                │
│       MessageEnvelope               │
│  ┌────────────┬─────────────┐      │
│  │  Message   │   Context    │      │
│  │            │              │      │
│  │ Transport  │  Transport   │      │
│  │  Message   │  Metadata    │      │
│  └────────────┴─────────────┘      │
└─────────────────────────────────────┘
```

## Work Phases

### Phase 0: Analysis and Design (Week 1, Day 1-2)
Analyze current usage, understand protocol layers, and design migration strategy.

| ID | Task | Duration | Dependencies | Status | Owner | Task File |
|----|------|----------|--------------|--------|-------|-----------|
| A.0 | **Analyze MCP Protocol Specifications** | 2h | None | ⬜ Not Started | | [📄 Task Details](tasks/A.0-mcp-protocol-analysis.md) |
| A.1 | **Analyze TransportMessage Usage** | 3h | A.0 | ⬜ Not Started | | [📄 Task Details](tasks/A.1-transport-message-usage-analysis.md) |
| A.2 | **Design MessageEnvelope Structure** | 2h | A.0, A.1 | ⬜ Not Started | | [📄 Task Details](tasks/A.2-design-message-envelope.md) |
| A.3 | **Create Migration Strategy** | 2h | A.2 | ⬜ Not Started | | [📄 Task Details](tasks/A.3-create-migration-strategy.md) |
| A.4 | **Document Breaking Changes** | 1h | A.3 | ⬜ Not Started | | [📄 Task Details](tasks/A.4-document-breaking-changes.md) |

**Phase 0 Total**: 10 hours

**Analysis Output Directory**: `analysis/`
- All findings and analysis documents will be stored here
- See individual task files for specific deliverables

### Phase 1: Core Infrastructure (Week 1, Day 2-3)
Build the new transport context system alongside existing code.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.1 | **Create MessageEnvelope Types** | 3h | A.2 | ⬜ Not Started | | `src/transport/envelope.rs` |
| C.2 | **Implement Transport Metadata** | 2h | C.1 | ⬜ Not Started | | HTTP, SSE, stdio variants |
| C.3 | **Add Context Extraction** | 2h | C.2 | ⬜ Not Started | | Extract metadata from transports |
| C.4 | **Create Compatibility Layer** | 3h | C.3 | ⬜ Not Started | | Bridge old and new APIs |

**Phase 1 Total**: 10 hours

### Phase 2: Transport Migration (Week 1, Day 4-5)
Migrate transport implementations to use new context system.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| T.1 | **Migrate StdioTransport** | 2h | C.4 | ⬜ Not Started | | Simplest transport to start |
| T.2 | **Migrate HttpTransport** | 3h | C.4 | ⬜ Not Started | | Add header context |
| T.3 | **Migrate HttpMcpTransport** | 2h | T.2 | ⬜ Not Started | | MCP-specific HTTP |
| T.4 | **Update Transport Trait** | 2h | T.1-T.3 | ⬜ Not Started | | Add context-aware methods |

**Phase 2 Total**: 9 hours

### Phase 3: Proxy Layer Migration (Week 2, Day 1-2)
Update proxy implementations to handle transport context.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| P.1 | **Update Forward Proxy** | 3h | T.4 | ⬜ Not Started | | Handle context in forwarding |
| P.2 | **Update Reverse Proxy** | 3h | T.4 | ⬜ Not Started | | Extract/inject HTTP context |
| P.3 | **Session Context Integration** | 2h | P.1-P.2 | ⬜ Not Started | | Link context to sessions |

**Phase 3 Total**: 8 hours

### Phase 4: Testing and Documentation (Week 2, Day 3)
Ensure everything works and is documented.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| D.1 | **Unit Tests for Envelope** | 2h | C.1-C.4 | ⬜ Not Started | | Test new types |
| D.2 | **Integration Tests** | 2h | P.3 | ⬜ Not Started | | End-to-end with context |
| D.3 | **Migration Guide** | 1h | All | ⬜ Not Started | | Document for other components |

**Phase 4 Total**: 5 hours

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Implementation Details

See task files for detailed designs:
- **Type Definitions**: See [A.2-design-message-envelope.md](tasks/A.2-design-message-envelope.md)
- **Migration Strategy**: See [A.3-create-migration-strategy.md](tasks/A.3-create-migration-strategy.md)
- **Breaking Changes**: See [A.4-document-breaking-changes.md](tasks/A.4-document-breaking-changes.md)

Key design decisions will be documented in `analysis/message-envelope-design.md` after Phase 0 completion.

## Success Criteria

### Functional Requirements
- ✅ Transport metadata properly separated from protocol messages
- ✅ SSE-specific metadata can be tracked through the system
- ✅ HTTP headers preserved through proxy layers
- ✅ Session correlation maintained across transports
- ✅ Backward compatibility maintained during migration

### Performance Requirements
- ✅ < 1% additional latency from context handling
- ✅ < 5MB additional memory for context storage
- ✅ No performance regression in existing code paths

### Quality Requirements
- ✅ 95% test coverage for new code
- ✅ No clippy warnings
- ✅ Complete documentation for new types
- ✅ Migration guide for dependent code

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Breaking changes in 90 files | HIGH | Incremental migration with compatibility layer | Planned |
| Performance regression | MEDIUM | Benchmark before/after each phase | Planned |
| SSE integration delays | HIGH | This refactor is now a prerequisite | Active |
| Incomplete migration | MEDIUM | Feature flags to toggle old/new paths | Planned |
| Context memory overhead | LOW | Use Cow<> and Arc<> for shared data | Planned |

## Impact on SSE Integration

This refactor directly enables the following SSE integration tasks:
- **S.2**: SSE Transport Wrapper can properly handle event metadata
- **S.4**: Parser hooks can access transport context
- **R.2**: SSE Response Handler can set proper event IDs
- **R.4**: Early correlation can use transport context
- **I.4**: Stream interception can modify SSE metadata
- **I.5**: Reverse proxy can preserve context

## Session Planning Guidelines

### Phase 0: Analysis Tasks
- **Start with**: Task A.0 - MCP Protocol Analysis
- **Task files**: See `tasks/` directory for detailed instructions
- **Output to**: `analysis/` directory for all findings
- **Duration**: ~10 hours total for all analysis tasks

### Implementation Phases (1-4)
1. **Review** (10 min): Review tracker and relevant task file
2. **Implementation** (2-3 hours): Focus on one component at a time
3. **Testing** (30 min): Test both old and new code paths
4. **Documentation** (15 min): Update analysis/migration documents
5. **Handoff** (10 min): Update tracker and progress metrics

### Critical Documentation
- **All analysis outputs** must be written to `analysis/` directory
- **Task details** are in `tasks/` directory - no need to duplicate
- **Progress tracking** should update both tracker and analysis/README.md

### Using the rust-code-reviewer
For implementation phases, the rust-code-reviewer should focus on:
- Zero-cost abstractions
- Lifetime management for context data
- Avoiding unnecessary clones
- Async trait implementations
- Backward compatibility

## Critical Implementation Guidelines

### Proxy Mode Parity
**ALWAYS implement context handling in BOTH proxy modes:**
- **Forward Proxy**: Must preserve context when forwarding
- **Reverse Proxy**: Must extract context from HTTP and inject into responses

### Common Pitfalls to Avoid
- Don't clone the entire context unnecessarily
- Don't lose context during error handling
- Don't assume all transports have all metadata types
- Don't break existing Transport trait users

## Communication Protocol

### Status Updates
After completing each task:
1. Update task status in this tracker
2. Run benchmarks to ensure no regression
3. Document any unexpected TransportMessage usage found
4. Update count of migrated files

### Handoff Notes
Track migration progress:
- Files migrated: X/90
- Tests passing: Y/Z
- Performance impact: +X% latency, +Y MB memory

## Related Documents

### Primary References
- [SSE Proxy Integration Tracker](../sse-proxy-integration/sse-proxy-integration-tracker.md)
- [Proxy-SSE-Message Tracker](../proxy-sse-message-tracker.md) - **BLOCKED ON THIS REFACTOR**
- [Transport Module](../../shadowcat/src/transport/mod.rs)

### Design Documents
- [Architecture Plan](../002-shadowcat-architecture-plan.md)
- [Developer Guide](../003-shadowcat-developer-guide.md)

### Impacted Components
- All Transport implementations
- Forward and Reverse Proxy
- Session Manager
- Interceptor Engine
- Recorder/Replay systems

## Next Actions

1. **Immediate**: Begin Phase 0 analysis tasks (see `tasks/` directory)
2. **Day 1**: Complete A.0 and A.1 analysis tasks
3. **Day 2**: Complete A.2, A.3, and A.4 design tasks
4. **Day 3-4**: Implement Phase 1 core infrastructure
5. **Week 2**: Complete remaining phases

## Notes

- This refactor is a **prerequisite** for SSE proxy integration
- The compatibility layer is critical for incremental migration
- Performance benchmarks must be run after each phase
- Consider using feature flags to allow rollback if issues arise
- The 658 occurrences of TransportMessage suggest many are imports/uses that won't need changes

---

**Document Version**: 1.0  
**Created**: 2025-08-08  
**Last Modified**: 2025-08-08  
**Author**: Development Team

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-08-08 | 1.0 | Initial tracker creation based on SSE integration requirements | Dev Team |