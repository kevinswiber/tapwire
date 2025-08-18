# Next Session: Continue Legacy Reverse Proxy Refactoring - Phase D

## Context
You are continuing the refactoring of the shadowcat reverse proxy's `legacy.rs` file, following the architecture defined in `@plans/refactor-legacy-reverse-proxy/analysis/final-architecture.md`.

## Current Status
- **Session**: 4 complete, starting session 5
- **Branch**: `refactor/legacy-reverse-proxy` in shadowcat submodule
- **Progress**: legacy.rs at 2,897 lines (down from 3,465)
- **Phase C**: Complete ✅
- **Phase D**: Started - Upstream extraction
- **All 20 tests passing** ✅

## What Was Accomplished (Session 4)
1. ✅ Thinned handlers/mcp.rs from 492 → 192 lines (proper orchestrator pattern)
2. ✅ Added session version tracking functions to session_helpers.rs
3. ✅ Added frame recording functions to pipeline.rs  
4. ✅ Created upstream/http/sse.rs placeholder (delegates to legacy)
5. ✅ Renamed session_ops.rs → session_helpers.rs per architecture
6. ✅ Created upstream/http/ module structure

## Priority Tasks for Next Session

### Task D.1: Extract HTTP Client Logic (2 hours)
Move the actual HTTP upstream implementation from legacy.rs:
- Extract `process_via_http_hyper()` → upstream/http/client.rs
- Create proper Hyper client wrapper
- Remove duplicated logic between legacy and upstream/http.rs
- Update handlers to use the new upstream service

### Task D.2: Extract Stdio Processing (1.5 hours)
Move stdio upstream implementation:
- Extract `process_via_stdio_pooled()` → enhance upstream/stdio.rs
- Implement connection pooling properly
- Remove from legacy.rs
- Update handlers to use the new upstream service

### Task D.3: Complete SSE Extraction (2 hours)
Replace placeholder with actual implementation:
- Move full `proxy_sse_from_upstream()` logic from legacy.rs
- Implement in upstream/http/sse.rs (not just a placeholder)
- Handle reqwest_eventsource dependencies
- Remove from legacy.rs

### Task D.4: Handler Integration (1 hour)
Update handlers to use upstream services properly:
- Remove direct calls to legacy functions
- Use UpstreamService trait
- Ensure proper error handling
- Test all proxy modes

## Key Files to Work With
- `shadowcat/src/proxy/reverse/legacy.rs` - Main file being refactored (goal: reduce significantly)
- `shadowcat/src/proxy/reverse/handlers/mcp.rs` - Should use upstream services, not legacy
- `shadowcat/src/proxy/reverse/upstream/http/` - Implement client.rs, complete sse.rs
- `shadowcat/src/proxy/reverse/upstream/stdio.rs` - Enhance with pooling logic

## Architecture Reminders
Per `analysis/final-architecture.md`:
- **Handlers** should be thin orchestrators (<200 lines)
- **Upstream modules** handle all communication with upstream servers
- **No direct legacy.rs calls** from handlers after this phase
- **UpstreamService trait** is the abstraction boundary

## Commands to Run
```bash
# Navigate to shadowcat
cd shadowcat

# Check current status
git status
wc -l src/proxy/reverse/legacy.rs

# Run tests frequently
cargo test --lib proxy::reverse

# Before committing
cargo fmt
cargo clippy --all-targets -- -D warnings
```

## Success Criteria for This Session
- [ ] legacy.rs reduced to <2400 lines (remove ~500 lines)
- [ ] HTTP client logic in upstream/http/client.rs
- [ ] Stdio processing in upstream/stdio.rs  
- [ ] SSE logic in upstream/http/sse.rs (not placeholder)
- [ ] Handlers using UpstreamService trait
- [ ] All 20 tests still passing
- [ ] No clippy warnings

## Important Notes
- **Focus on extraction**, not perfection - we can refine later
- **Keep tests passing** at each step
- **Small, incremental commits** make review easier
- The goal is to empty legacy.rs so it can be deleted
- Follow the architecture in final-architecture.md strictly

## Git Workflow
```bash
# You're on branch refactor/legacy-reverse-proxy
cd shadowcat
git add -A
git commit -m "refactor: [description of extraction]"

# After significant progress
cd ..  # back to tapwire
git add shadowcat plans/
git commit -m "chore: update shadowcat submodule and tracking docs"
git push
```

Remember: The goal is to completely eliminate legacy.rs by moving all its functionality to proper modules!