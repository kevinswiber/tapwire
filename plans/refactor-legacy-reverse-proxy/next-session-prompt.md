# Continue Reverse Proxy Refactoring - Phase F Final Extraction

## Context
We're refactoring shadowcat's monolithic 3,465-line `legacy.rs` reverse proxy into a clean modular architecture. 

**Current Status**: Phase E complete - legacy.rs reduced to 1,749 lines (49.5% reduction)
**Tests**: 19 passing
**Goal**: Extract remaining components and delete legacy.rs entirely

## Phase F Tasks (Priority Order)

### F.0 - Move ReverseProxyServer & Builder (~566 lines)
Extract the main server implementation to `server.rs`:
- ReverseProxyServer struct and its impl
- ReverseProxyServerBuilder and its impl  
- Keep builder pattern intact
- Update imports in main.rs

### F.1 - Move handle_mcp_sse_request (~163 lines)
Extract SSE handler to `handlers/sse.rs`:
- Move the entire handle_mcp_sse_request function
- Update handlers/mcp.rs to import from handlers/sse.rs instead of legacy
- Ensure all SSE streaming modules are properly connected

### F.2 - Move Router Creation (~70 lines)
Extract create_router to `router.rs`:
- Move the create_router function
- Update server to use router::create_router
- Ensure all routes are properly configured

### F.3 - Move Health/Metrics Handlers (~100 lines)
Extract to `handlers/health.rs`:
- handle_health function
- handle_metrics function
- Update router to use handlers::health

### F.4 - Organize Tests (~850 lines)
Create proper test modules:
- Move integration tests to tests/
- Create unit test modules for each component
- Ensure all tests still pass

### F.5 - Delete legacy.rs
Final removal once everything is extracted and tests pass

## Key Files to Reference
- Main tracker: `/Users/kevin/src/tapwire/plans/refactor-legacy-reverse-proxy/refactor-legacy-reverse-proxy-tracker.md`
- Legacy file: `/Users/kevin/src/tapwire/shadowcat/src/proxy/reverse/legacy.rs`
- Phase F tasks: `/Users/kevin/src/tapwire/plans/refactor-legacy-reverse-proxy/tasks/F.0-extract-server.md`

## Success Criteria
- All 19 tests passing
- No clippy warnings
- legacy.rs completely removed
- No module exceeds 500 lines
- Clean module boundaries

## Start Command
Begin with Phase F.0 - extracting ReverseProxyServer to server.rs. This is the largest remaining component (~566 lines) and will significantly reduce legacy.rs.