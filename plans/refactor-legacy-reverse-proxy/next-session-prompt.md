# Next Session: Extract Large Functions from legacy.rs (Phase D Preparation)

## Project Context

Refactoring the monolithic 3,137-line `legacy.rs` reverse proxy into clean modules. Phase B and C are COMPLETE.

**Project**: Refactor Legacy Reverse Proxy
**Tracker**: `plans/refactor-legacy-reverse-proxy/refactor-legacy-reverse-proxy-tracker.md`
**Branch**: `refactor/legacy-reverse-proxy` in shadowcat repo
**Status**: Need to extract ~637 more lines to reach target of < 2,500 lines

## Current Status

### Completed
- ✅ Phase A: Analysis & Design
- ✅ Phase B: Foundation modules extracted (state, headers, session_helpers, selector)
- ✅ Phase C: Upstream abstractions and handlers created
- ✅ All 20 tests passing
- ✅ Clean module structure established

### Progress
- **Starting point**: 3,465 lines
- **After Session 1**: 3,307 lines (158 lines removed)
- **After Session 2**: 3,137 lines (170 lines removed)
- **Total removed**: 328 lines
- **Target**: < 2,500 lines (need to remove ~637 more)

## Your Mission

### Priority: Extract Large Functions

The largest remaining functions in legacy.rs that need extraction:

1. **handle_mcp_request** (~551 lines) - Already have thin version in handlers/mcp.rs
   - Extract the actual processing logic to appropriate modules
   - Move interceptor handling to pipeline.rs
   - Move upstream processing to upstream modules

2. **handle_mcp_sse_request** (~400 lines) - SSE handling
   - Create sse_handler.rs or integrate with existing SSE modules
   - Extract SSE-specific logic

3. **process_message** and related upstream processing
   - Move to upstream modules
   - Consolidate with existing upstream implementations

### Strategy

1. **Start with handle_mcp_request internals**
   - Move interceptor logic to pipeline.rs
   - Move session tracking logic to session_helpers.rs
   - Move upstream communication to upstream modules
   - Keep handler thin (< 150 lines)

2. **Extract SSE handling**
   - Consider reusing transport::sse modules
   - Create clean abstraction for SSE responses

3. **Consolidate upstream processing**
   - Move process_via_stdio_pooled to upstream/stdio.rs
   - Move process_via_http_hyper to upstream/http/
   - Remove duplication

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
# Currently: 3,137 lines

# Find large functions
grep -n "^async fn\|^fn" src/proxy/reverse/legacy.rs | head -20
```

## Success Criteria
- [ ] legacy.rs < 2,500 lines
- [ ] All 20 tests still passing
- [ ] No clippy warnings
- [ ] Each extracted module < 500 lines

## Time Estimate
- Extract handle_mcp_request logic: 2 hours
- Extract SSE handling: 1.5 hours
- Consolidate upstream processing: 1.5 hours
- Testing/validation: 30 min
**Total: 5.5 hours**

---
**Remember**: 
- Incremental refactoring - keep tests green at each step
- Use temporary re-exports in legacy.rs during migration
- Commit frequently with clear messages