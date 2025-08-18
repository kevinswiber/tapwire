# Refactor Legacy Reverse Proxy Tracker

## Overview

Refactoring the monolithic 3,465-line `legacy.rs` reverse proxy implementation into a clean, modular architecture with proper separation of concerns.

**Last Updated**: 2025-01-18  
**Total Estimated Duration**: 20-25 hours (reduced after removing admin UI)  
**Status**: Phase A Complete - Ready for Implementation
**Working Branch**: `refactor/legacy-reverse-proxy` in shadowcat repo

## Goals

1. **Eliminate Monolith** - Delete `legacy.rs` entirely
2. **Modular Architecture** - No module exceeds 500 lines
3. **Clean Separation** - Single responsibility per module
4. **Maintain Functionality** - All tests continue passing
5. **Improve Testability** - Enable unit testing of components

## Architecture Vision (Refined)

```
src/proxy/reverse/
├── mod.rs                    # Public API exports
├── error.rs                  # ReverseProxyError (50 lines)
├── config.rs                 # All config types (250 lines)
├── state.rs                  # AppState (100 lines)
├── metrics.rs                # Metrics collection (50 lines)
├── server.rs                 # Server + Builder (200 lines)
├── router.rs                 # Router setup (100 lines)
├── handlers/
│   ├── mod.rs               # Handler exports (20 lines)
│   ├── mcp.rs               # /mcp endpoint - THIN (100 lines)
│   └── health.rs            # /health, /metrics (50 lines)
├── pipeline.rs              # Intercept/pause/record (200 lines)
├── session_helpers.rs       # Session operations (150 lines)
├── headers.rs               # Header utilities (100 lines)
└── upstream/
    ├── mod.rs               # UpstreamService trait (50 lines)
    ├── selector.rs          # Load balancing (100 lines)
    ├── stdio.rs             # Stdio upstream (200 lines)
    └── http/
        ├── mod.rs           # HttpUpstream impl (50 lines)
        ├── client.rs        # Hyper client (150 lines)
        ├── relay.rs         # JSON responses (150 lines)
        └── sse_adapter.rs   # Uses transport::sse (100 lines)

REMOVED:
└── admin/                   # Admin UI deleted (~900 lines)
```

**Total Lines**: ~1,970 (down from 3,682)
**Key Changes**: 
- Admin UI removed entirely
- Renamed to avoid conflicts (upstream/ not transport/)
- Leverages transport::sse
- Thin handlers (<150 lines)

## Work Phases

### Phase A: Analysis & Design (Week 1)
Understanding the current architecture and designing the clean solution.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.0 | **Current State Analysis** | 2h | None | ✅ Complete | | [Analysis](analysis/current-structure.md) |
| A.1 | **Dependency Mapping** | 2h | A.0 | ✅ Complete | | [Dependencies](analysis/dependencies.md) |
| A.2 | **Module Design** | 3h | A.1 | ✅ Complete | | [Architecture](analysis/module-architecture.md) |
| A.3 | **Architecture Refinement** | 2h | A.2 | ✅ Complete | | [Final Architecture](analysis/final-architecture.md) |

**Phase A Total**: 9 hours ✅ COMPLETE

### Phase B: Core Extraction (Week 1-2)
Extract foundational components and remove admin UI.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| B.0 | **Transport Overlap Analysis** | 0.5h | A.3 | ⬜ Not Started | | Document reuse opportunities |
| B.1 | **Remove Admin UI** | 0.5h | B.0 | ⬜ Not Started | | Delete ~900 lines |
| B.2 | **Extract Error Types** | 0.5h | B.1 | ⬜ Not Started | | Create error.rs |
| B.3 | **Extract Config Types** | 1h | B.2 | ⬜ Not Started | | Create config.rs |
| B.4 | **Extract Metrics & State** | 0.5h | B.3 | ⬜ Not Started | | metrics.rs, state.rs |
| B.5 | **Extract Helper Modules** | 1h | B.4 | ⬜ Not Started | | pipeline.rs, session_helpers.rs, headers.rs |

**Phase B Total**: 4 hours

### Phase C: Upstream & Handler Implementation (Week 2)
Create upstream abstractions and thin handlers.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.0 | **Create UpstreamService Trait** | 1h | B.5 | ⬜ Not Started | | upstream/mod.rs |
| C.1 | **Implement Upstream Modules** | 3h | C.0 | ⬜ Not Started | | stdio.rs, http/* |
| C.2 | **Create Thin Handlers** | 2h | C.1 | ⬜ Not Started | | mcp.rs, health.rs |
| C.3 | **Wire Router & Server** | 1h | C.2 | ⬜ Not Started | | router.rs, server.rs |

**Phase C Total**: 7 hours

### Phase D: Cleanup & Validation (Week 3)
Remove legacy.rs and validate everything works.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| D.0 | **Delete legacy.rs** | 0.5h | C.3 | ⬜ Not Started | | Final removal |
| D.1 | **Organize Tests** | 1h | D.0 | ⬜ Not Started | | Move to test modules |
| D.2 | **Integration Testing** | 2h | D.1 | ⬜ Not Started | | Full test suite |
| D.3 | **Performance Validation** | 1h | D.2 | ⬜ Not Started | | Benchmark comparison |
| D.4 | **Documentation** | 0.5h | D.3 | ⬜ Not Started | | Update docs |

**Phase D Total**: 5 hours

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Progress Tracking

### Completed
- **2025-01-18**: Phase A - Analysis & Design (9 hours)
  - Comprehensive analysis of legacy.rs
  - Designed refined architecture
  - Created implementation plan

### Week 1 (Starting 2025-01-19)
- [ ] Phase B: Core Extraction (4 hours)
  - Transport overlap analysis
  - Remove admin UI
  - Extract foundation modules

### Week 2
- [ ] Phase C: Upstream & Handler Implementation (7 hours)

### Week 3  
- [ ] Phase D: Cleanup & Validation (5 hours)

## Key Decisions Made

1. **Admin UI**: Removed entirely (~900 lines deleted)
2. **Naming**: `upstream/` instead of `transport/`, `session_helpers.rs` not `session/`
3. **SSE**: Reuse `transport::sse` via adapter
4. **Handlers**: Thin (<150 lines), orchestration only
5. **Pipeline**: Single file for cross-cutting concerns

## Success Criteria

### Functional Requirements
- ✅ All existing tests pass
- ✅ No functionality lost
- ✅ Backward compatibility maintained
- ✅ All proxy modes supported

### Code Quality Requirements
- ✅ No module > 500 lines
- ✅ Single responsibility per module
- ✅ Clean module boundaries
- ✅ No circular dependencies
- ✅ No clippy warnings

### Performance Requirements
- ✅ No performance regression
- ✅ Memory usage unchanged or improved
- ✅ Startup time < 100ms
- ✅ Request latency unchanged

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Breaking existing functionality | HIGH | Incremental refactoring with tests at each step | Planned |
| Circular dependencies | MEDIUM | Design clean interfaces upfront | Planned |
| Performance regression | MEDIUM | Benchmark before and after | Planned |
| Large PR difficult to review | LOW | Split into multiple smaller PRs | Planned |

## Critical Implementation Guidelines

### Migration Strategy
1. **Never break tests** - Each extraction must maintain green tests
2. **Incremental approach** - One module at a time
3. **Compatibility layer** - Keep exports working during migration
4. **Feature flags** - Use for optional components like admin UI

### Module Size Constraints
- **Target**: 200-300 lines per module
- **Maximum**: 500 lines (hard limit)
- **Minimum**: 50 lines (avoid over-fragmentation)

### Testing Requirements
- Unit tests for each extracted module
- Integration tests remain passing
- Performance benchmarks before/after
- Load testing at completion

## Related Documents

### Primary References
- [Original reverse-proxy-refactor plan](../reverse-proxy-refactor/)
- [Event tracking refactor (complete)](../refactor-event-tracking/)
- [Block_on fix details](../reverse-proxy-refactor/tasks/E.0-fix-block-on-deadlock.md)

### Task Files
- [Analysis Tasks](tasks/)
- [Design Documents](analysis/)

### Specifications
- MCP Protocol v2025-11-05
- Rust async/await patterns
- Hyper v1.0 migration guide

## Next Actions

1. **Start with Phase A analysis** - Understand before refactoring
2. **Create detailed module interfaces** - Design clean boundaries
3. **Set up feature flags** - Prepare for admin UI extraction

## Notes

- The `streaming/` modules already exist and are partially integrated
- The block_on deadlock fix in `hyper_sse_intercepted.rs` is complete
- EventTracker and session management already refactored via separate plan
- Admin UI is 876 lines and should be feature-gated
- Must maintain both forward and reverse proxy compatibility

---

**Document Version**: 1.0  
**Created**: 2025-08-18  
**Last Modified**: 2025-08-18  
**Author**: Claude + Kevin

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-08-18 | 1.0 | Initial tracker creation | Claude |