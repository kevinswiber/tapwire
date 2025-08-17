# Event Tracking Refactor Tracker

## Overview

This tracker coordinates the consolidation and cleanup of Last-Event-Id tracking systems across the Shadowcat codebase. We currently have **5 overlapping tracking systems** with no synchronization, creating significant complexity and potential for bugs.

**Last Updated**: 2025-08-17  
**Total Estimated Duration**: 8-12 hours  
**Status**: Planning

## Goals

1. **Consolidate Tracking** - Reduce from 5 systems to a single authoritative source
2. **Establish Clear Ownership** - Transport layer owns tracking, Session layer owns persistence
3. **Enable SSE Resilience** - Unblock reverse proxy SSE reconnection features
4. **Improve Code Quality** - Remove redundant code and clarify interfaces

## Architecture Vision

### Current State (5 Overlapping Systems)
```
┌─────────────────────────────────────────────┐
│         5 TRACKING SYSTEMS                  │
│       NO SYNCHRONIZATION!                   │
└─────────────────────────────────────────────┘
    ↓            ↓            ↓
Session A    Session A    Session A
ID: "123"    ID: "456"    ID: null
(out of sync!)
```

### Target State (Unified Architecture)
```
┌─────────────────────────────────────────────┐
│       Session Store (Persistence)           │
│  • Saves last_event_id for recovery        │
│  • Updated from transport layer            │
└────────────────────┬───────────────────────┘
                     │ persists
┌────────────────────▼───────────────────────┐
│    Transport EventTracker (Authority)      │
│  • Single source of truth                 │
│  • Deduplication and resumption          │
│  • Updates flow to session store         │
└────────────────────────────────────────────┘
```

## The Five Systems (Analysis Summary)

1. **Session Store Layer** (`session/store.rs` + `memory.rs`)
   - Persistent storage with `Session.last_event_id`
   - ✅ KEEP for persistence only

2. **SSE Session Integration** (`session/sse_integration.rs`)
   - Per-connection tracking in `ConnectionInfo`
   - ❌ REFACTOR to reference transport tracker

3. **Reverse Proxy SSE Resilience** (`proxy/reverse/sse_resilience.rs`)
   - Wraps transport EventTracker
   - ❌ REMOVE duplicate tracker creation

4. **Transport Layer Event Tracking** (`transport/sse/reconnect.rs`)
   - Core `EventTracker` with deduplication
   - ✅ KEEP as single authority

5. **SSE Connection Level** (`transport/sse/connection.rs`)
   - Raw connection tracking
   - 🔄 REFACTOR to feed transport tracker

## Work Phases

### Phase A: Analysis & Planning (3 hours) - ✅ COMPLETE
Understanding the problem space and designing the solution

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.0 | **Analyze tracking systems** | 1h | None | ✅ Complete | | [Analysis](analysis/last-event-id-tracking-analysis.md) |
| A.1 | **Map dependencies** | 1h | A.0 | ✅ Complete | | Found 5 systems |
| A.2 | **Design unified approach** | 1h | A.1 | ✅ Complete | | Transport as authority |

**Phase A Total**: 3 hours (COMPLETE)

### Phase B: Minimal Integration (2-3 hours)
Quick fix to unblock reverse proxy SSE resilience

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| B.1 | **Wire transport tracker to proxy** | 1h | None | ⬜ Not Started | | [Details](tasks/B.1-wire-transport-tracker.md) |
| B.2 | **Connect session persistence** | 1h | B.1 | ⬜ Not Started | | [Details](tasks/B.2-connect-persistence.md) |
| B.3 | **Test SSE resilience** | 1h | B.2 | ⬜ Not Started | | [Details](tasks/B.3-test-resilience.md) |

**Phase B Total**: 3 hours

### Phase C: Remove Redundancy (3-4 hours)
Clean up duplicate tracking systems

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.1 | **Remove proxy duplicate trackers** | 1h | B.3 | ⬜ Not Started | | [Details](tasks/C.1-remove-proxy-trackers.md) |
| C.2 | **Refactor SSE session integration** | 2h | C.1 | ⬜ Not Started | | [Details](tasks/C.2-refactor-session-integration.md) |
| C.3 | **Update connection tracking** | 1h | C.2 | ⬜ Not Started | | [Details](tasks/C.3-update-connection.md) |

**Phase C Total**: 4 hours

### Phase D: Documentation & Testing (2 hours)
Ensure quality and maintainability

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| D.1 | **Document architecture** | 1h | C.3 | ⬜ Not Started | | [Details](tasks/D.1-document-architecture.md) |
| D.2 | **Integration tests** | 1h | C.3 | ⬜ Not Started | | [Details](tasks/D.2-integration-tests.md) |

**Phase D Total**: 2 hours

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Progress Tracking

### Completed Tasks
- [x] A.0: Analyze tracking systems - Completed 2025-08-17
- [x] A.1: Map dependencies - Completed 2025-08-17
- [x] A.2: Design unified approach - Completed 2025-08-17

### Next Session Tasks
- [ ] B.1: Wire transport tracker to proxy
- [ ] B.2: Connect session persistence
- [ ] B.3: Test SSE resilience

## Success Criteria

### Functional Requirements
- ✅ Single source of truth for event IDs
- ✅ No duplicate tracking logic
- ✅ Clear data flow from transport to persistence
- ✅ SSE resilience unblocked in reverse proxy

### Quality Requirements
- ✅ All tests passing
- ✅ No clippy warnings
- ✅ Clear documentation of architecture
- ✅ Backward compatibility maintained

### Performance Requirements
- ✅ No additional memory overhead
- ✅ < 1ms tracking overhead per event
- ✅ Efficient deduplication

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Breaking existing SSE functionality | HIGH | Incremental changes with testing | Active |
| Session persistence issues | MEDIUM | Test with both in-memory and future Redis | Planned |
| Reverse proxy integration conflicts | MEDIUM | Coordinate with reverse proxy refactor | Active |
| Missing edge cases | LOW | Comprehensive integration tests | Planned |

## Implementation Strategy

### Option A: Minimal Change (SELECTED)
**Approach**: Wire existing transport EventTracker to proxy, update session from transport
- **Duration**: 2-3 hours
- **Risk**: Low
- **Benefits**: Quick unblock of SSE resilience
- **Drawbacks**: Some redundancy remains temporarily

### Option B: Full Refactor (NOT SELECTED)
**Approach**: Complete redesign of tracking architecture
- **Duration**: 8-12 hours  
- **Risk**: High
- **Benefits**: Clean architecture
- **Drawbacks**: Blocks other work

### Option C: Gradual Migration (FUTURE)
**Approach**: Start with Option A, deprecate redundant systems over time
- **Duration**: Spread over 2-3 releases
- **Benefits**: Low risk, clean end state
- **Drawbacks**: Temporary complexity

## Related Documents

### Primary References
- [Last-Event-Id Tracking Analysis](analysis/last-event-id-tracking-analysis.md)
- [Reverse Proxy Refactor](../reverse-proxy-refactor/tracker.md) - ON HOLD pending this work
- [Transport SSE Implementation](../../shadowcat/src/transport/sse/)

### Task Files
- [Phase B Tasks](tasks/) - Minimal integration
- [Phase C Tasks](tasks/) - Redundancy removal
- [Phase D Tasks](tasks/) - Documentation

### Key Code Locations
- `src/transport/sse/reconnect.rs` - Core EventTracker (KEEP)
- `src/session/store.rs` - Session persistence (KEEP)
- `src/proxy/reverse/sse_resilience.rs` - Duplicate tracking (REFACTOR)
- `src/session/sse_integration.rs` - Connection tracking (REFACTOR)

## Next Actions

1. **Start Phase B.1** - Wire transport tracker to reverse proxy
2. **Test with MCP Inspector** - Verify SSE resilience works
3. **Resume reverse proxy refactor** - Once tracking is consolidated

## Notes

- This refactor was discovered during reverse proxy SSE resilience integration
- The transport layer's EventTracker is the most mature implementation
- Session persistence should be one-way: Transport → Session
- Future Redis storage will use the same interface
- Gradual deprecation preferred over big-bang refactor

---

**Document Version**: 1.0  
**Created**: 2025-08-17  
**Last Modified**: 2025-08-17  
**Author**: Claude + Kevin

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-08-17 | 1.0 | Initial plan created from analysis | Claude + Kevin |