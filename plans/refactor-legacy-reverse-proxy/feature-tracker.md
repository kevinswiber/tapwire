# Feature Tracker: Refactor Legacy Reverse Proxy

## Status
**Current Phase**: E - Cleanup & Consolidation  
**Sessions**: 5 complete  
**Branch**: `refactor/legacy-reverse-proxy` in shadowcat submodule  
**Legacy.rs**: 2,196 lines (goal: 0)  
**Tests**: All 20 passing ✅

## Problem Statement
The `legacy.rs` file in shadowcat's reverse proxy has grown to 3,465 lines, making it difficult to maintain, test, and extend. It violates single responsibility principle with mixed concerns including server setup, routing, SSE handling, interceptors, and business logic.

## Goals
1. ✅ Break down legacy.rs into focused modules (<300 lines each)
2. ✅ Maintain all existing functionality and tests
3. ✅ Eliminate reqwest dependency (use hyper exclusively)
4. ⚠️ Create clean module boundaries with clear responsibilities
5. ⚠️ Enable future extensibility for admin UI and monitoring

## Success Criteria
- [x] All 20 existing tests pass
- [x] Legacy.rs reduced below 2,400 lines
- [x] No reqwest dependencies in reverse proxy
- [ ] All modules under 300 lines
- [ ] Clean separation of concerns
- [ ] Legacy.rs can be deleted

## Progress Summary

### ✅ Phase A: Analysis & Planning (Session 1)
- Analyzed legacy.rs structure and dependencies
- Created extraction plan with 6 phases
- Identified key modules to extract

### ✅ Phase B: Core Extractions (Sessions 2-3)
- Extracted config (250 lines), state (150 lines), metrics (60 lines)
- Extracted router (125 lines), server (175 lines)
- Created pipeline module for interceptors/pause/record
- Created session_helpers and headers modules
- Reduced legacy.rs: 3,465 → 2,897 lines

### ✅ Phase C: Handler Extraction (Session 4)
- Thinned handlers/mcp.rs to 192 lines (proper orchestrator)
- Added session version tracking to session_helpers
- Created upstream/http module structure
- Reduced legacy.rs: 2,897 → 2,734 lines

### ✅ Phase D: Upstream & Reqwest Elimination (Session 5)
- **Completely eliminated reqwest dependency!**
- Extracted HTTP client logic to upstream/http/client.rs
- Extracted stdio processing to upstream/stdio.rs
- Created hyper-based SSE initiator (288 lines)
- Deleted unused modules (upstream_response.rs, json_processing.rs)
- Deleted dead functions: proxy_sse_response, process_via_http, proxy_sse_from_upstream
- Reduced legacy.rs: 2,734 → 2,196 lines (-538 lines!)

### ⚠️ Phase E: Cleanup & Consolidation (Current)
**Issues to address:**
1. Duplicate selector modules (selector.rs vs upstream/selector.rs)
2. hyper_ prefixed files should be renamed/reorganized
3. Old handler files (mcp_old.rs, mcp_original.rs) need removal
4. Some functions still in legacy.rs that could be extracted
5. Module organization could be cleaner

### Phase F: Final Verification
- [ ] Delete legacy.rs entirely
- [ ] Verify all tests still pass
- [ ] Update documentation
- [ ] Performance validation

## Key Findings

### What's Left in Legacy.rs (2,196 lines)
1. **Core Server Logic** (~400 lines)
   - ReverseProxyServer struct and builder
   - Main server run loop
   
2. **Request Handlers** (~800 lines)
   - handle_mcp_request (main POST handler)
   - handle_mcp_sse_request (SSE GET handler)
   - Supporting handler functions

3. **Message Processing** (~600 lines)
   - process_message
   - Various helper functions
   
4. **Tests** (~400 lines)
   - Test implementations

### Modules Successfully Extracted
- ✅ config.rs (250 lines)
- ✅ state.rs (150 lines)
- ✅ metrics.rs (60 lines)
- ✅ router.rs (125 lines)
- ✅ server.rs (175 lines)
- ✅ pipeline.rs (200 lines)
- ✅ session_helpers.rs (150 lines)
- ✅ headers.rs (100 lines)
- ✅ upstream/http/client.rs (135 lines)
- ✅ upstream/http/sse_initiator.rs (288 lines)
- ✅ upstream/stdio.rs (249 lines)

### Technical Debt Addressed
- ✅ Eliminated reqwest dependency completely
- ✅ Unified on hyper for all HTTP operations
- ✅ Reused transport layer components (SseParser, EventTracker)
- ⚠️ Need to consolidate duplicate modules
- ⚠️ Need better module organization

## Risk Assessment
- **Low Risk**: Refactoring with all tests passing
- **Medium Risk**: Some module duplication needs resolution
- **Mitigated**: Incremental approach with continuous testing

## Next Steps (Phase E)
1. Consolidate selector modules
2. Rename hyper_ prefixed files  
3. Remove old handler files
4. Extract remaining handler logic from legacy.rs
5. Final module organization

## Notes
- Successful elimination of reqwest is a major win
- Reusing transport layer components shows good architecture
- Module size goals mostly achieved (<300 lines)
- Ready for final cleanup phase