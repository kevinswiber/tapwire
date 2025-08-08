# Transport Context Refactor Tracker

## Overview

This tracker coordinates the refactoring of Shadowcat's transport layer to properly separate protocol concerns (JSON-RPC messages) from transport-specific metadata (HTTP headers, SSE events, stdio). This is a prerequisite for SSE proxy integration and must be completed before continuing with the proxy-sse-message-tracker.md work.

### ⚡ USE THE SIMPLIFIED APPROACH
**We have NO external users** - Shadowcat hasn't been released yet! This means:
- ✅ Use the [Simplified Migration Strategy](analysis/migration-strategy-simplified.md)
- ❌ IGNORE the original complex migration strategy with compatibility layers
- ✅ Break APIs freely and delete old code immediately
- ✅ Complete in 30-40 hours instead of 60

**Last Updated**: 2025-08-08  
**Total Estimated Duration**: ~~60 hours~~ **30-40 hours** (simplified - no external users!)  
**Status**: Phase 0 Analysis 100% Complete ✅

⚠️ **CRITICAL UPDATE**: Since Shadowcat hasn't been released yet, we have NO external users. This allows us to:
- Skip all backward compatibility layers
- Delete old code immediately
- Make breaking changes freely
- Complete the refactor in HALF the time

See [Simplified Migration Strategy](analysis/migration-strategy-simplified.md) for the aggressive approach.

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

The `TransportMessage` type is used in 34 files with 330 occurrences (corrected from initial estimate), making this a significant but manageable architectural change.

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
| A.0 | **Analyze MCP Protocol Specifications** | 2h | None | ✅ Completed | | [📄 Task Details](tasks/A.0-mcp-protocol-analysis.md) |
| A.1 | **Analyze TransportMessage Usage** | 3h | A.0 | ✅ Completed | | [📄 Task Details](tasks/A.1-transport-message-usage-analysis.md) |
| A.2 | **Design MessageEnvelope Structure** | 2h | A.0, A.1 | ✅ Completed | | [📄 Task Details](tasks/A.2-design-message-envelope.md) |
| A.3 | **Create Migration Strategy** | 2h | A.2 | ✅ Completed | | [📄 Task Details](tasks/A.3-create-migration-strategy.md) |
| A.4 | **Document Breaking Changes** | 1h | A.3 | ✅ Completed | | [📄 Task Details](tasks/A.4-document-breaking-changes.md) |

**Phase 0 Total**: 10 hours

**Phase 0 Completed Deliverables**: 
- ✅ [MCP Protocol Layers Analysis](analysis/mcp-protocol-layers.md) - Notifications ARE bidirectional, direction is implicit
- ✅ [Architecture Clarification](analysis/architecture-clarification.md) - Clear separation of transport, MCP, and JSON-RPC layers
- ✅ [TransportMessage Usage Analysis](analysis/transport-message-usage.md) - 34 files, 330 occurrences, no dead imports
- ✅ [Migration Impact Assessment](analysis/migration-impact.md) - 60 hour total estimate, phased approach
- ✅ [Current Workarounds Catalog](analysis/current-workarounds.md) - 17 workaround patterns identified
- ✅ [MessageEnvelope Design](analysis/message-envelope-design.md) - Complete type definitions and architecture
- ✅ [Migration Strategy](analysis/migration-strategy.md) - 6-phase incremental migration plan
- ✅ [Breaking Changes Documentation](analysis/breaking-changes.md) - All breaking changes cataloged with timelines

**Key Findings**:
1. **Notifications ARE bidirectional** - Both client→server and server→client, confirming our core assumption
2. **Session Manager is central** - 44 occurrences, manages all routing and must be migrated carefully
3. **No dead code** - All 34 files actively use TransportMessage (no simple wins)
4. **17 workaround patterns** - Complex state management and context reconstruction throughout codebase
5. **Direction is implicit** - Currently inferred from transport edge, breaks for notifications
6. **Headers are lost** - HTTP/SSE metadata extracted but not propagated with messages

**Phase 0 Complete** ✅:
All analysis and design work has been completed. The MessageEnvelope architecture has been fully designed with:
- Complete type definitions for MessageEnvelope, MessageContext, and TransportContext
- ~~Comprehensive migration strategy spanning 6 phases over ~60 hours~~ **SIMPLIFIED: 3 phases, 30-40 hours**
- ~~Full documentation of all breaking changes with mitigation paths~~ **NOT NEEDED: No external users**
- ~~Zero-downtime migration approach using compatibility layers~~ **SIMPLIFIED: Direct replacement**

## SIMPLIFIED IMPLEMENTATION PLAN (No External Users!)

### Phase 1: Add New Types & Migrate Core (10-15 hours)
Create new system and directly replace core components.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| S.1 | **Create MessageEnvelope Types** | 2h | A.2 | ⬜ Not Started | | Direct implementation, no compatibility |
| S.2 | **Replace TransportMessage → ProtocolMessage** | 2h | S.1 | ⬜ Not Started | | Global rename, update all imports |
| S.3 | **Update Transport Trait (BREAKING)** | 2h | S.2 | ⬜ Not Started | | Direct change to use MessageEnvelope |
| S.4 | **Migrate All Transports** | 4h | S.3 | ⬜ Not Started | | Stdio, HTTP, HttpMcp - no compatibility |
| S.5 | **Update SessionManager** | 4h | S.3 | ⬜ Not Started | | Delete Frame, use MessageEnvelope |

**Phase 1 Total**: 14 hours

### Phase 2: Migrate Everything Else (10-15 hours)
Direct conversion of all remaining components.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| S.6 | **Update Proxy (Forward & Reverse)** | 5h | S.5 | ⬜ Not Started | | Direct update, break freely |
| S.7 | **Update All Interceptors** | 3h | S.5 | ⬜ Not Started | | New interface, no compatibility |
| S.8 | **Update Peripherals** | 5h | S.5 | ⬜ Not Started | | Recorder, metrics, audit, rate limiting |

**Phase 2 Total**: 13 hours

### Phase 3: Delete Old Code & Cleanup (5 hours)
Remove all legacy code and workarounds.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| S.9 | **Delete Old Types** | 1h | S.8 | ⬜ Not Started | | TransportMessage, Direction, Frame |
| S.10 | **Remove 17 Workarounds** | 2h | S.8 | ⬜ Not Started | | Clean up all identified patterns |
| S.11 | **Update All Tests** | 2h | S.9-S.10 | ⬜ Not Started | | Fix to use new types only |

**Phase 3 Total**: 5 hours

**TOTAL: 32 hours** (vs 60 hours in original plan)

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

## What NOT to Do (Old Conservative Approach)

❌ **DON'T** create compatibility layers - we have no users to support  
❌ **DON'T** use type aliases for TransportMessage - just rename it  
❌ **DON'T** keep old methods with deprecation warnings - delete them  
❌ **DON'T** use feature flags for gradual rollout - change everything directly  
❌ **DON'T** maintain backward compatibility - break freely  
❌ **DON'T** create migration guides for external users - there are none  

## Simplified Risk Assessment

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| ~~Breaking changes in 90 files~~ 34 files to update | MEDIUM | Compiler will find them all | Active |
| Performance regression | LOW | Benchmark before/after | Planned |
| ~~SSE integration delays~~ | N/A | This enables SSE | Active |
| ~~Incomplete migration~~ | LOW | Compiler ensures completeness | Active |
| ~~Context memory overhead~~ | LOW | Already optimized in design | Resolved |

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

## Next Session Focus

### Immediate Next Steps (Phase 0 Completion)
Complete the remaining Phase 0 analysis and design tasks:

1. **Task A.2: Design MessageEnvelope Structure** (2 hours)
   - Use the protocol layer analysis to inform design
   - Address the 17 workarounds identified
   - Create concrete Rust type definitions
   
2. **Task A.3: Create Migration Strategy** (2 hours)
   - Build on the migration impact assessment
   - Define specific migration phases and checkpoints
   - Create compatibility layer design
   
3. **Task A.4: Document Breaking Changes** (1 hour)
   - Summarize for stakeholders
   - Create migration guide for developers
   - Define deprecation timeline

These tasks will complete Phase 0 and provide everything needed to begin implementation in Phase 1.

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