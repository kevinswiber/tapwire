# Next Session: Phase E - Cleanup & Consolidation

## Context
You are continuing the refactoring of shadowcat's reverse proxy `legacy.rs` file. We've successfully eliminated reqwest and extracted major components. Now we need final cleanup.

## Current Status
- **Session**: Starting session 6
- **Branch**: `refactor/legacy-reverse-proxy` in shadowcat submodule  
- **Progress**: legacy.rs at 2,196 lines (down from 3,465)
- **All 20 tests passing** ✅
- **Reqwest eliminated** ✅

## What Was Accomplished (Session 5)
1. ✅ Completely eliminated reqwest dependency
2. ✅ Created hyper-based SSE initiator (288 lines)
3. ✅ Deleted 538 lines from legacy.rs
4. ✅ Removed unused modules (upstream_response.rs, json_processing.rs)
5. ✅ All HTTP/SSE now uses hyper exclusively

## Priority Tasks for Phase E

### Task E.0: Consolidate Selectors (30 min)
- Compare `selector.rs` vs `upstream/selector.rs`
- Merge duplicate functionality
- Single source of truth for upstream selection

### Task E.1: Rename Hyper Modules (30 min)
- Rename `hyper_raw_streaming.rs` → better name
- Rename `hyper_sse_intercepted.rs` → better name
- Create proper module structure (streaming/ or sse/)

### Task E.2: Clean Up Old Files (15 min)
- Delete `handlers/mcp_old.rs`
- Delete `handlers/mcp_original.rs`
- Remove any .bak files

### Task E.3: Extract Remaining Handlers (2 hours)
- Move `handle_mcp_request` from legacy.rs to handlers/
- Move `handle_mcp_sse_request` from legacy.rs to handlers/
- Create handlers/helpers.rs for shared logic
- Target: legacy.rs under 1,800 lines

### Task E.4: Final Server Extraction (1 hour)
- Move ReverseProxyServer implementation to server.rs
- Move builder pattern to server.rs
- Target: legacy.rs under 1,000 lines

## Commands to Run
```bash
# Navigate to shadowcat
cd shadowcat

# Check current status
git status
wc -l src/proxy/reverse/legacy.rs
ls -la src/proxy/reverse/*.bak* 2>/dev/null

# Run tests frequently
cargo test --lib proxy::reverse

# Before committing
cargo fmt
cargo clippy --all-targets -- -D warnings
```

## Module Structure Issues to Fix
```
Current Issues:
- Duplicate: selector.rs vs upstream/selector.rs
- Poor naming: hyper_raw_streaming.rs, hyper_sse_intercepted.rs  
- Old files: mcp_old.rs, mcp_original.rs
- Large legacy.rs still has handlers and server code
```

## Success Criteria for This Session
- [ ] No duplicate selector modules
- [ ] No hyper_ prefixed files
- [ ] No old/backup files
- [ ] legacy.rs under 1,000 lines
- [ ] All 20 tests still passing
- [ ] No clippy warnings

## Git Workflow
```bash
# You're on branch refactor/legacy-reverse-proxy
cd shadowcat
git add -A
git commit -m "refactor: [description]"

# After significant progress
cd ..  # back to tapwire
git add shadowcat plans/
git commit -m "chore: update shadowcat submodule and tracking"
```

## Important Notes
- We're in cleanup phase - focus on organization
- Don't break anything that's working
- Small, incremental commits
- Goal is to make legacy.rs deletable