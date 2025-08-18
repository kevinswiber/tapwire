# Task E.1: Complete Hyper Migration from Legacy

## Objective
Complete the migration from the 3,465-line `legacy.rs` to clean hyper-based modules as originally planned.

## Current State
Despite claims in the tracker that the refactor is "complete", the reality is:
- `legacy.rs` still contains 3,465 lines (unchanged from start)
- Hyper modules exist but are minimally used
- Main request handling still goes through legacy code
- The refactor was started but never finished

## What Exists vs What's Used

### Created Hyper Modules
1. **hyper_client.rs** - Basic hyper client setup
2. **hyper_raw_streaming.rs** - Raw streaming support
3. **hyper_sse_intercepted.rs** - SSE with interceptors (has block_on bug)
4. **json_processing.rs** - JSON response handling
5. **upstream_response.rs** - Response wrapper

### Still in Legacy
- Main server setup and configuration (~500 lines)
- Request routing and handling (~800 lines)
- Session management integration (~400 lines)
- Authentication/authorization (~600 lines)
- Interceptor integration (~400 lines)
- Metrics and monitoring (~200 lines)
- Admin endpoints (~500 lines)

## Migration Plan

### Phase 1: Core Server Structure
Extract from legacy.rs:
- `ReverseProxyServer` struct → `server.rs`
- `ReverseProxyConfig` types → `config.rs`
- Router creation → `router.rs`
- App state management → `state.rs`

### Phase 2: Request Handling
Move request processing:
- `handle_mcp_request` → Use hyper modules fully
- `handle_mcp_sse_request` → Integrate with hyper_sse_intercepted
- Remove duplicate `process_via_http_hyper` logic

### Phase 3: Middleware Stack
Extract middleware components:
- Authentication middleware → `middleware/auth.rs`
- Rate limiting middleware → `middleware/rate_limit.rs`
- CORS/tracing setup → `middleware/setup.rs`

### Phase 4: Admin Interface
The 876-line admin UI should be:
- Moved to `admin/` submodule
- Split into routes, handlers, and UI components
- Made optional via feature flag

### Phase 5: Cleanup
- Delete legacy.rs
- Update mod.rs exports
- Ensure all tests still pass

## Implementation Strategy

### Incremental Approach
1. Start with leaf functions (no dependencies)
2. Move pure data types next
3. Extract stateless handlers
4. Refactor stateful components last
5. Keep tests passing at each step

### Module Size Targets
- No module over 500 lines
- Average module ~200-300 lines
- Clear single responsibility per module

## Files to Create
```
src/proxy/reverse/
├── config.rs          (~200 lines)
├── server.rs          (~300 lines)
├── router.rs          (~200 lines)
├── state.rs           (~150 lines)
├── handlers/
│   ├── mcp.rs         (~400 lines)
│   ├── sse.rs         (~300 lines)
│   └── health.rs      (~100 lines)
├── middleware/
│   ├── auth.rs        (~200 lines)
│   ├── rate_limit.rs  (~150 lines)
│   └── setup.rs       (~100 lines)
├── admin/
│   ├── mod.rs         (~100 lines)
│   ├── routes.rs      (~200 lines)
│   ├── handlers.rs    (~300 lines)
│   └── ui.rs          (~276 lines)
└── upstream/
    ├── selection.rs   (~200 lines)
    └── pool.rs        (~150 lines)
```

## Success Criteria
- [ ] legacy.rs deleted
- [ ] All functionality preserved
- [ ] All tests passing
- [ ] No module over 500 lines
- [ ] Clean module boundaries
- [ ] Improved testability

## Estimated Time
- Phase 1: 3-4 hours
- Phase 2: 4-5 hours
- Phase 3: 2-3 hours
- Phase 4: 3-4 hours
- Phase 5: 1-2 hours
- **Total**: 13-18 hours

## Risk Mitigation
- Keep legacy.rs until all exports migrated
- Run tests after each extraction
- Use git commits for each module migration
- Maintain backwards compatibility