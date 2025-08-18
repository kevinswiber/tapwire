# Refactor Legacy Reverse Proxy Tracker

## Overview

Refactoring the monolithic 3,465-line `legacy.rs` reverse proxy implementation into a clean, modular architecture with proper separation of concerns.

**Last Updated**: 2025-08-18  
**Total Estimated Duration**: 25-35 hours  
**Status**: Planning

## Goals

1. **Eliminate Monolith** - Delete `legacy.rs` entirely
2. **Modular Architecture** - No module exceeds 500 lines
3. **Clean Separation** - Single responsibility per module
4. **Maintain Functionality** - All tests continue passing
5. **Improve Testability** - Enable unit testing of components

## Architecture Vision

```
src/proxy/reverse/
├── mod.rs              # Public API (re-exports)
├── config/             # Configuration (~600 lines total)
│   ├── mod.rs          # Config traits
│   ├── upstream.rs     # Upstream configs
│   ├── session.rs      # Session configs
│   └── middleware.rs   # Auth, rate limit configs
├── server/             # Server lifecycle (~800 lines)
│   ├── mod.rs          # Server struct
│   ├── builder.rs      # Builder pattern
│   └── state.rs        # Shared app state
├── router/             # Routing (~400 lines)
│   ├── mod.rs          # Router setup
│   ├── routes.rs       # Route definitions
│   └── middleware.rs   # Middleware stack
├── handlers/           # Request handlers (~1400 lines)
│   ├── mod.rs          # Handler traits
│   ├── mcp.rs          # MCP requests
│   ├── sse.rs          # SSE streaming
│   ├── health.rs       # Health/metrics
│   └── admin.rs        # Admin endpoints
├── streaming/          # Already exists (~850 lines)
│   ├── hyper_client.rs
│   ├── hyper_raw_streaming.rs
│   ├── hyper_sse_intercepted.rs ✅ (fixed)
│   └── json_processing.rs
├── upstream/           # Upstream management (~500 lines)
│   ├── mod.rs          # Upstream traits
│   ├── selection.rs    # Load balancing
│   ├── pool.rs         # Connection pooling
│   └── health.rs       # Health checking
└── admin/              # Admin UI (feature-gated, ~876 lines)
    ├── mod.rs          
    ├── dashboard.rs    # HTML UI
    └── api.rs          # Admin API
```

## Work Phases

### Phase A: Analysis & Design (Week 1)
Understanding the current architecture and designing the clean solution.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| A.0 | **Current State Analysis** | 2h | None | ⬜ Not Started | | [Details](tasks/A.0-current-state-analysis.md) |
| A.1 | **Dependency Mapping** | 2h | A.0 | ⬜ Not Started | | [Details](tasks/A.1-dependency-mapping.md) |
| A.2 | **Module Design** | 3h | A.1 | ⬜ Not Started | | [Details](tasks/A.2-module-design.md) |
| A.3 | **Interface Design** | 2h | A.2 | ⬜ Not Started | | [Details](tasks/A.3-interface-design.md) |

**Phase A Total**: 9 hours

### Phase B: Core Extraction (Week 1-2)
Extract foundational components that everything else depends on.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| B.0 | **Extract Config Types** | 2h | A.3 | ⬜ Not Started | | [Details](tasks/B.0-extract-config.md) |
| B.1 | **Extract Server Core** | 3h | B.0 | ⬜ Not Started | | [Details](tasks/B.1-extract-server.md) |
| B.2 | **Extract App State** | 2h | B.1 | ⬜ Not Started | | [Details](tasks/B.2-extract-state.md) |

**Phase B Total**: 7 hours

### Phase C: Handler Extraction (Week 2)
Move request handling logic to dedicated modules.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| C.0 | **Extract MCP Handler** | 4h | B.2 | ⬜ Not Started | | [Details](tasks/C.0-extract-mcp-handler.md) |
| C.1 | **Extract SSE Handler** | 3h | B.2 | ⬜ Not Started | | [Details](tasks/C.1-extract-sse-handler.md) |
| C.2 | **Extract Health/Admin** | 2h | B.2 | ⬜ Not Started | | [Details](tasks/C.2-extract-health-admin.md) |

**Phase C Total**: 9 hours

### Phase D: Infrastructure (Week 2-3)
Extract supporting infrastructure components.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| D.0 | **Extract Upstream Management** | 3h | C.0 | ⬜ Not Started | | [Details](tasks/D.0-extract-upstream.md) |
| D.1 | **Extract Router/Middleware** | 2h | C.0, C.1, C.2 | ⬜ Not Started | | [Details](tasks/D.1-extract-router.md) |
| D.2 | **Feature-gate Admin UI** | 2h | C.2 | ⬜ Not Started | | [Details](tasks/D.2-feature-gate-admin.md) |

**Phase D Total**: 7 hours

### Phase E: Cleanup & Validation (Week 3)
Remove legacy code and ensure everything works.

| ID | Task | Duration | Dependencies | Status | Owner | Notes |
|----|------|----------|--------------|--------|-------|-------|
| E.0 | **Delete Legacy.rs** | 1h | D.0, D.1, D.2 | ⬜ Not Started | | [Details](tasks/E.0-delete-legacy.md) |
| E.1 | **Integration Testing** | 2h | E.0 | ⬜ Not Started | | [Details](tasks/E.1-integration-testing.md) |
| E.2 | **Performance Validation** | 2h | E.1 | ⬜ Not Started | | [Details](tasks/E.2-performance-validation.md) |

**Phase E Total**: 5 hours

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Progress Tracking

### Week 1 (Starting 2025-08-19)
- [ ] A.0: Current State Analysis
- [ ] A.1: Dependency Mapping
- [ ] A.2: Module Design
- [ ] A.3: Interface Design
- [ ] B.0: Extract Config Types

### Completed Tasks
- [x] Fixed block_on deadlock in hyper_sse_intercepted.rs - Completed 2025-08-18

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