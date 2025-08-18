# Next Session: Complete Foundation Extraction (Phase B) and Handlers (Phase C.2-C.3)

## Project Context

Refactoring the monolithic 3,298-line `legacy.rs` reverse proxy into clean modules. Phase A (Analysis) and Phase C.0-C.1 (Upstream Abstractions) are COMPLETE.

**Project**: Refactor Legacy Reverse Proxy
**Tracker**: `plans/refactor-legacy-reverse-proxy/refactor-legacy-reverse-proxy-tracker.md`
**Branch**: `refactor/legacy-reverse-proxy` in shadowcat repo
**Status**: Upstream abstractions complete, need foundation modules and handlers

## Current Status

### Completed
- ✅ Phase A: Analysis & Design
- ✅ Phase C.0-C.1: Upstream abstractions (trait, HTTP, stdio, selector)
- ✅ Moved hyper client to transport module
- ✅ All 20 tests passing

### Module Structure Created
```
src/proxy/reverse/
├── upstream/
│   ├── mod.rs       # UpstreamService trait ✅
│   ├── selector.rs  # Load balancing ✅
│   ├── http.rs      # HTTP upstream ✅
│   └── stdio.rs     # Stdio upstream ✅
├── config.rs        # Existing (needs expansion)
├── metrics.rs       # Existing (needs expansion)
└── legacy.rs        # 3,298 lines to refactor
```

## Your Mission

### Priority 1: Foundation Extraction (Phase B) - 3 hours

1. **Remove Admin UI** (30 min)
   ```bash
   # Count admin lines
   grep -n "admin" src/proxy/reverse/legacy.rs | wc -l
   ```
   - Delete handle_admin_request function
   - Remove admin routes from router
   - Update tests

2. **Extract Error Types** (30 min)
   - Create `src/proxy/reverse/error.rs`
   - Move ReverseProxyError enum if not using crate::error
   - Add re-exports

3. **Extract State & Expand Config** (1 hour)
   - Create `src/proxy/reverse/state.rs` - AppState struct
   - Expand `config.rs` with missing config types
   - Move all config-related code

4. **Extract Helper Modules** (1 hour)
   - `headers.rs` - Header manipulation utilities
   - `session_helpers.rs` - Session operations
   - `pipeline.rs` - Interceptor/pause/record orchestration

### Priority 2: Thin Handlers (Phase C.2-C.3) - 3 hours

5. **Create Handler Modules** (2 hours)
   - `handlers/mod.rs` - Exports
   - `handlers/mcp.rs` - Main /mcp endpoint (<150 lines)
   - `handlers/health.rs` - Health & metrics endpoints
   - Handlers should orchestrate, not implement logic

6. **Wire Router & Server** (1 hour)
   - Create `router.rs` - Route setup
   - Create `server.rs` - Server builder pattern
   - Update mod.rs exports

## Commands to Run First

```bash
cd /Users/kevin/src/tapwire/shadowcat
git checkout refactor/legacy-reverse-proxy
git pull

# Verify starting point
cargo test proxy::reverse --lib | grep "test result"
# Should show: "test result: ok. 20 passed"

# Check legacy.rs size
wc -l src/proxy/reverse/legacy.rs
# Currently: 3,298 lines

# See what needs extraction
grep -n "struct AppState" src/proxy/reverse/legacy.rs
grep -n "handle_admin" src/proxy/reverse/legacy.rs
```

## Implementation Strategy

### For Each Extraction:
1. Create new file
2. Move code with imports
3. Add temporary re-exports in legacy.rs
4. Run tests immediately
5. Fix compilation errors
6. Commit when green

### Handler Pattern:
```rust
// handlers/mcp.rs - THIN orchestration
pub async fn handle_mcp_request(
    State(app): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response> {
    // 1. Extract/validate headers
    let mcp_headers = headers::extract_mcp_headers(&headers)?;
    
    // 2. Parse body
    let message = parse_request_body(&body)?;
    
    // 3. Get/create session
    let session = session_helpers::get_or_create_session(&app, &mcp_headers)?;
    
    // 4. Apply pipeline (intercept/pause/record)
    let message = pipeline::process_inbound(&app, message, &session).await?;
    
    // 5. Select upstream
    let upstream = app.upstream_selector.select().await
        .ok_or(ReverseProxyError::NoUpstreamsAvailable)?;
    
    // 6. Forward to upstream
    let response = upstream.send_request(message, &session, app.interceptor_chain.clone()).await?;
    
    // 7. Apply outbound pipeline
    pipeline::process_outbound(&app, response, &session).await
}
```

## Success Criteria
- [ ] Admin UI completely removed
- [ ] All foundation modules extracted
- [ ] Handlers < 150 lines each
- [ ] Legacy.rs < 2,500 lines
- [ ] All 20 tests still passing
- [ ] No clippy warnings

## Key Files to Reference
- See `plans/refactor-legacy-reverse-proxy/phase-c-summary.md` for what was just completed
- Check `src/proxy/reverse/upstream/` for the new upstream abstractions
- Transport HTTP client: `src/transport/outgoing/http.rs::send_mcp_request_raw()`

## Time Estimate
- Foundation extraction: 3 hours
- Handler creation: 2 hours  
- Wiring: 1 hour
- Testing/validation: 30 min
**Total: 6.5 hours**

---
**Remember**: The goal is incremental refactoring. Each extraction should maintain green tests!