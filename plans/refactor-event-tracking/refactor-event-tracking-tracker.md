# Event Tracking Refactor Tracker

## Overview

This tracker coordinates the consolidation and cleanup of Last-Event-Id tracking systems across the Shadowcat codebase. ~~We currently have **5 overlapping tracking systems** with no synchronization~~. **UPDATE**: Deep analysis revealed only 3 active systems and 1 dead system - simpler than expected!

**Last Updated**: 2025-08-17  
**Total Estimated Duration**: ~~8-12 hours~~ ~~7 hours~~ **6 hours** (further reduced - SessionStore trait already perfect!)  
**Status**: Ready for Implementation

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

## Deep Analysis Findings (NEW!)

### Key Discoveries
1. **ReverseProxySseManager is DEAD CODE** - Only in tests, never used in production!
2. **Transport EventTracker is mature** - Already has all needed functionality
3. **Systems aren't integrated** - They exist in isolation, no complex merging needed
4. **SessionStore trait already perfect!** - Has `store_last_event_id()` and `get_last_event_id()` methods
5. **Simple wiring needed** - Just callbacks from transport to existing SessionStore methods

### Revised Understanding

| System | Status | Action Required |
|--------|--------|-----------------|
| Session Store | ❌ Not wired | Add update callback |
| SSE Session Integration | ⚠️ Redundant | Remove duplicate tracking |
| **Reverse Proxy SSE Resilience** | **💀 DEAD CODE** | **DELETE ENTIRELY** |
| Transport EventTracker | ✅ Working | Add persistence callback |
| SSE Connection | ✅ Minimal | Leave as-is |

## The Five Systems (Original Analysis)

1. **Session Store Layer** (`session/store.rs` + `memory.rs`)
   - Persistent storage with `Session.last_event_id`
   - ✅ KEEP for persistence only - just needs wiring

2. **SSE Session Integration** (`session/sse_integration.rs`)
   - Per-connection tracking in `ConnectionInfo`
   - ❌ REFACTOR to remove redundant tracking

3. **Reverse Proxy SSE Resilience** (`proxy/reverse/sse_resilience.rs`)
   - ~~Wraps transport EventTracker~~
   - **💀 DELETE - Dead code, never used in production!**

4. **Transport Layer Event Tracking** (`transport/sse/reconnect.rs`)
   - Core `EventTracker` with deduplication
   - ✅ KEEP as single authority - already perfect

5. **SSE Connection Level** (`transport/sse/connection.rs`)
   - Raw connection tracking
   - ✅ KEEP as-is - minimal wire protocol handler

## Work Phases

### Phase A: Deep Analysis & Planning (4.5 hours) - ✅ COMPLETE
Understanding the problem space and designing the solution

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.0 | **Initial analysis** | 1h | None | ✅ Complete | | [Analysis](analysis/last-event-id-tracking-analysis.md) |
| A.1 | **Map dependencies** | 1h | A.0 | ✅ Complete | | Found 5 systems |
| A.2 | **Design approach** | 1h | A.1 | ✅ Complete | | Transport as authority |
| A.3 | **Deep usage analysis** | 2h | A.2 | ✅ Complete | | [Usage mapping](analysis/usage-mapping.md) |
| A.4 | **Functionality matrix** | 1h | A.3 | ✅ Complete | | [Matrix](analysis/functionality-matrix.md) |
| A.5 | **Consolidation design** | 1h | A.4 | ✅ Complete | | [Design](analysis/consolidation-design.md) |
| A.6 | **Revised plan** | 30m | A.5 | ✅ Complete | | [Revised tasks](tasks/A.3-revised-implementation-plan.md) |

**Phase A Total**: 4.5 hours (COMPLETE)

### Phase B: Core Implementation (2.5 hours) - FURTHER REVISED ✏️
Delete dead code and add callback mechanism using EXISTING SessionStore trait

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| B.1 | **Delete dead code** | 30m | None | ⬜ Not Started | | Remove sse_resilience.rs entirely |
| B.2 | **Add EventTracker callbacks** | 1h | None | ⬜ Not Started | | Add persistence notification |
| B.3 | **Wire to SessionStore** | 1h | B.2 | ⬜ Not Started | | Use existing trait methods! |

**Phase B Total**: 2.5 hours (reduced - no trait modifications needed)

### Phase C: Integration & Cleanup (2.5 hours) - REVISED ✏️
Wire reverse proxy and remove redundancy

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.1 | **Update reverse proxy** | 1.5h | B.3 | ⬜ Not Started | | Use shared EventTracker |
| C.2 | **Remove redundant tracking** | 1h | C.1 | ⬜ Not Started | | Clean ConnectionInfo |

**Phase C Total**: 2.5 hours

### Phase D: Testing & Validation (1 hour) - SIMPLIFIED ✏️
Ensure everything works correctly

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| D.1 | **Integration testing** | 1h | C.2 | ⬜ Not Started | | Test all scenarios |

**Phase D Total**: 1 hour

**GRAND TOTAL**: 4.5 (Phase A) + 2.5 (Phase B) + 2.5 (Phase C) + 1 (Phase D) = **10.5 hours**
- Phase A already complete: **6 hours remaining work**

### ~~Deprecated Phases~~ (No longer needed)
- ~~Original Phase C.3~~: Update connection tracking - Not needed
- ~~Original Phase D.1~~: Document architecture - Already documented in analysis
- ~~Original Phase D.2~~: Separate integration tests - Combined into D.1

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

## Why SessionStore Abstraction is Critical

### Future Storage Backends (Automatic Support!)
With our consolidation using the existing SessionStore trait:
- **Redis Backend**: When added, event IDs automatically persist to Redis
- **External Stores**: Third-party session stores via API get event tracking for free
- **Distributed Proxy**: Multiple instances share event IDs via the store
- **No Code Changes**: New backends work without touching event tracking code

### Architecture Benefits
- **Clean Separation**: Transport does tracking, Store does persistence
- **Interface Stability**: Using existing trait methods, no API changes
- **Testing**: Can mock SessionStore for unit tests
- **Zero Technical Debt**: Proper abstraction from the start

## Implementation Strategy

### Option A: Minimal Change (SELECTED & IMPROVED)
**Approach**: Wire transport EventTracker to SessionStore trait methods
- **Duration**: ~~2-3 hours~~ **2 hours** (simpler with existing trait)
- **Risk**: Very Low (using existing abstractions)
- **Benefits**: Quick unblock, future-proof design
- **Drawbacks**: ~~Some redundancy remains~~ None with proper abstraction!

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