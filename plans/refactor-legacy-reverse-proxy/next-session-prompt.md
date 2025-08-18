# Next Session: Continue Legacy Reverse Proxy Refactoring - Phase C

## Context
You are continuing the refactoring of the shadowcat reverse proxy's `legacy.rs` file, following the architecture defined in `@plans/refactor-legacy-reverse-proxy/analysis/final-architecture.md`.

## Current Status
- **Session**: 3 complete, starting session 4
- **Branch**: `refactor/legacy-reverse-proxy` in shadowcat submodule
- **Progress**: legacy.rs reduced from 3,465 → 2,894 lines (571 lines extracted)
- **Phase C**: 4.5 hours complete, 5 hours remaining
- **All 20 tests passing** ✅

## What Was Done (Session 3)
1. Created handlers/ module with thin orchestrator pattern
2. Extracted MCP handlers to handlers/mcp.rs (486 lines - TOO BIG)
3. Created handlers/health.rs (21 lines)
4. Extracted interceptor logic to pipeline.rs (236 lines)
5. Renamed session_helpers.rs to session_ops.rs
6. Updated router.rs to use new handlers

## Critical Next Task: Thin the MCP Handler
**PRIORITY**: The handlers/mcp.rs file is 486 lines but should be <150 lines per architecture.

Move logic OUT of handlers/mcp.rs to appropriate modules:
- Session version tracking → session_ops.rs
- Frame recording logic → pipeline.rs or dedicated module
- Upstream routing/selection → upstream modules
- Request/response processing → upstream modules

The handler should ONLY:
1. Parse/validate request
2. Get/create session
3. Call pipeline for processing
4. Call upstream for execution
5. Format and return response

## Next Tasks (Priority Order)

### Task C.5: Thin MCP Handler (2 hours) 🔄
```bash
cd shadowcat
# The handlers/mcp.rs file needs to be reduced from 486 to <150 lines
# Move business logic to appropriate modules
```

### Task C.6: Extract Upstream Logic (3 hours)
Create upstream module structure per final-architecture.md:
```
upstream/
├── mod.rs               # UpstreamService trait
├── selector.rs          # Already exists
├── stdio.rs            # Extract from legacy.rs
└── http/
    ├── mod.rs          # HttpUpstream impl
    ├── client.rs       # Move process_via_http_hyper
    ├── relay.rs        # JSON response handling
    └── sse_adapter.rs  # Move proxy_sse_from_upstream
```

Functions to move from legacy.rs:
- `process_via_http_hyper()` → upstream/http/client.rs
- `proxy_sse_from_upstream()` → upstream/http/sse_adapter.rs
- `process_via_stdio_pooled()` → upstream/stdio.rs
- `process_message()` → distribute appropriately

### Phase D: Final Cleanup
- Move tests from legacy.rs to appropriate modules
- Delete legacy.rs when empty
- Update all imports
- Validate performance

## Commands to Run
```bash
# Navigate to shadowcat
cd shadowcat

# Check current status
git status
wc -l src/proxy/reverse/legacy.rs
wc -l src/proxy/reverse/handlers/mcp.rs

# Run tests frequently
cargo test --lib proxy::reverse

# Before committing
cargo fmt
cargo clippy --all-targets -- -D warnings
```

## Success Criteria for This Session
- [ ] handlers/mcp.rs reduced to <150 lines
- [ ] Upstream modules created and populated
- [ ] legacy.rs reduced to <2000 lines
- [ ] All 20 tests still passing
- [ ] No clippy warnings

## Important Files
- `shadowcat/src/proxy/reverse/legacy.rs` - Main file being refactored
- `shadowcat/src/proxy/reverse/handlers/mcp.rs` - Needs thinning
- `plans/refactor-legacy-reverse-proxy/analysis/final-architecture.md` - Architecture guide
- `plans/refactor-legacy-reverse-proxy/refactor-legacy-reverse-proxy-tracker.md` - Progress tracking

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
```

Remember: The goal is to completely empty legacy.rs so it can be deleted!