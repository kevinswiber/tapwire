# Refactor Legacy Reverse Proxy Tracker

## Overview

Refactoring the monolithic 3,465-line `legacy.rs` reverse proxy implementation into a clean, modular architecture with proper separation of concerns.

**Last Updated**: 2025-01-18 (Session 6 - Phase E Complete)  
**Total Estimated Duration**: 20-25 hours  
**Status**: Phase E Complete - 1,749 lines remaining in legacy.rs (49.5% reduction)
**Working Branch**: `refactor/legacy-reverse-proxy` in shadowcat repo

## Goals

1. **Eliminate Monolith** - Delete `legacy.rs` entirely
2. **Modular Architecture** - No module exceeds 500 lines
3. **Clean Separation** - Single responsibility per module
4. **Maintain Functionality** - All tests continue passing
5. **Improve Testability** - Enable unit testing of components

## Progress Summary

- **Starting Point**: 3,465 lines in legacy.rs
- **Current State**: 1,749 lines (1,716 lines removed, 49.5% reduction)
- **Tests**: 19 passing (was 20, removed 1 unused test)
- **Modules Created**: 22 well-organized files

## Architecture Vision (Current State)

```
src/proxy/reverse/
├── mod.rs                    # Public API exports
├── config.rs                 # Config types (250 lines)
├── state.rs                  # AppState (50 lines)
├── metrics.rs                # Metrics collection (60 lines)
├── server.rs                 # Basic server (51 lines)
├── router.rs                 # Router setup (75 lines)
├── handlers/
│   ├── mod.rs               # Handler exports
│   ├── mcp.rs               # /mcp endpoint (150 lines) ✅
│   └── health.rs            # /health, /metrics (20 lines) ✅
├── pipeline.rs              # Intercept/pause/record (250 lines) ✅
├── session_helpers.rs       # Session operations (200 lines) ✅
├── headers.rs               # Header utilities (50 lines) ✅
├── legacy.rs                # REMAINING: 1,749 lines 🔥
└── upstream/
    ├── mod.rs               # UpstreamService trait + simple selector
    ├── selector.rs          # Advanced load balancing (117 lines) ✅
    ├── stdio.rs             # Stdio upstream (200 lines) ✅
    └── http/
        ├── mod.rs           # HTTP exports
        ├── client.rs        # Hyper client (135 lines) ✅
        └── streaming/       # SSE streaming modules ✅
            ├── initiator.rs # SSE connection setup (288 lines)
            ├── intercepted.rs # Parsed & intercepted (405 lines)
            └── raw.rs       # Direct byte streaming (122 lines)
```

## Work Phases

### Phase A: Analysis & Design ✅ COMPLETE (9 hours)
### Phase B: Core Extraction ✅ COMPLETE (3.5 hours)
### Phase C: Handler Extraction ✅ COMPLETE (9.5 hours)
### Phase D: Upstream Modules ✅ COMPLETE (4 hours)

### Phase E: Cleanup & Consolidation ✅ COMPLETE (Week 3)
Final cleanup to make legacy.rs deletable.

| ID | Task | Duration | Status | Notes |
|----|------|----------|--------|-------|
| E.0 | **Consolidate Selectors** | 0.5h | ✅ Complete | Kept upstream/selector.rs, removed duplicate |
| E.1 | **Rename Hyper Modules** | 0.5h | ✅ Complete | raw_streaming.rs, sse_intercepted.rs |
| E.2 | **Clean Up Old Files** | 0.25h | ✅ Complete | No backup files found |
| E.3 | **Extract Remaining Handlers** | 2h | ✅ Complete | Removed duplicate handle_mcp_request (320 lines) |
| E.4 | **Consolidate SSE Modules** | 1h | ✅ Complete | Moved to upstream/http/streaming/ |
| E.5 | **Remove Redundant Functions** | 1h | ✅ Complete | Removed process_message, echo_response, etc. |

**Phase E Total**: 5.25 hours ✅ COMPLETE
**Lines Removed**: 448 lines (legacy.rs: 2,197 → 1,749)

### Phase F: Final Extraction (Remaining)
Extract the remaining large components from legacy.rs.

| ID | Task | Duration | Status | Notes |
|----|------|----------|--------|-------|
| F.0 | **Move ReverseProxyServer** | 3h | ⬜ Not Started | ~566 lines to server.rs |
| F.1 | **Move handle_mcp_sse_request** | 2h | ⬜ Not Started | ~163 lines to handlers/sse.rs |
| F.2 | **Move Router Creation** | 1h | ⬜ Not Started | create_router to router.rs |
| F.3 | **Move Health/Metrics Handlers** | 0.5h | ⬜ Not Started | To handlers/health.rs |
| F.4 | **Organize Tests** | 1h | ⬜ Not Started | Move to test modules |
| F.5 | **Delete legacy.rs** | 0.5h | ⬜ Not Started | Final removal |

**Phase F Total**: 8 hours estimated

## What Remains in legacy.rs

1. **ReverseProxyServer & Builder** (~566 lines) - Main server implementation
2. **handle_mcp_sse_request** (~163 lines) - SSE endpoint handler
3. **create_router** (~70 lines) - Router configuration
4. **handle_health/handle_metrics** (~100 lines) - Health endpoints
5. **Test module** (~850 lines) - All tests

## Key Achievements

### Module Organization
- ✅ Consolidated selectors (removed duplicate)
- ✅ Moved SSE streaming to `upstream/http/streaming/`
- ✅ Consolidated SSE modules (removed redundant wrappers)
- ✅ Clean module hierarchy with clear responsibilities

### Code Cleanup
- ✅ Renamed modules (removed `hyper_` prefix)
- ✅ Removed unused `handle_mcp_request` (320 lines)
- ✅ Removed redundant proxy functions (57 lines)
- ✅ Removed `echo_response` and test (68 lines)
- ✅ Fixed all unused imports

### Architecture Improvements
```
upstream/
├── selector.rs          # Advanced selector with strategies
├── http/
│   ├── client.rs        # HTTP client logic
│   └── streaming/       # All SSE/streaming logic
│       ├── initiator.rs # SSE connection initiation
│       ├── intercepted.rs # Parsed & intercepted streaming
│       └── raw.rs      # Direct byte streaming
└── stdio/               # Stdio transport
```

## Next Steps

1. **Extract ReverseProxyServer** - Move server and builder to server.rs
2. **Extract SSE Handler** - Move handle_mcp_sse_request to handlers/sse.rs
3. **Consolidate Router** - Move all routing logic to router.rs
4. **Organize Tests** - Create proper test modules
5. **Delete legacy.rs** - Final removal once everything is extracted

## Success Metrics

- [x] All tests passing (19/19) ✅
- [x] No clippy warnings ✅
- [ ] legacy.rs deleted
- [ ] No module > 500 lines
- [ ] Clear module boundaries

## Risk Assessment

**Low Risk** - We've successfully:
- Extracted 49.5% of legacy.rs
- Maintained all functionality
- Kept tests passing
- Improved code organization

**Remaining Risk**: 
- ReverseProxyServer extraction is complex (~566 lines)
- Need to carefully handle dependencies
- Test module organization needs planning

## Notes

- Session 6 focused on cleanup and consolidation
- Excellent progress on module organization
- Ready for final extraction phase