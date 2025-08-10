# CLI Refactor Rebase Plan

## Overview
The shadowcat-cli-refactor branch needs to be rebased onto main to incorporate recent transport naming changes. This is a complex rebase due to conflicting structural changes in both branches.

## Current Situation

### Changes in Main (8 commits ahead)
Key commit: **4e52043** - "refactor(cli): align transport naming with MCP specification"
- **Removed**: `http` and `sse` commands from ForwardTransport enum
- **Added**: `streamable-http` command combining HTTP POST + optional SSE
- **Simplified**: ~150 lines of redundant code removed
- **Renamed**: `run_sse_forward_proxy()` → `run_streamable_http_forward_proxy()`

Other commits:
- Transport improvements (SSE/MCP integration)
- Event ID generator optimizations
- Clippy fixes and formatting

### Changes in Our Branch (shadowcat-cli-refactor)
- **Massive refactor**: main.rs reduced from 1294 → 141 lines
- **Modularized**: All commands moved to cli/ modules:
  - cli/forward.rs (328 lines)
  - cli/reverse.rs (229 lines)
  - cli/record.rs (400 lines)
  - cli/replay.rs (408 lines)
  - cli/common.rs (605 lines)
- **Current ForwardTransport**: Still has `stdio` and `http` variants

## Conflict Analysis

### Files with Conflicts
Based on diff analysis, the following files will have conflicts:
- `src/main.rs` - Heavy conflicts (completely different structures)
- `src/cli/forward.rs` - Needs transport enum updates
- `src/cli/*.rs` - New files in our branch, not in main
- `src/transport/sse_transport.rs` - Transport improvements in main
- `src/mcp/event_id.rs` - New file in main
- `src/mcp/batch.rs` - New file in main

### Primary Conflicts
1. **ForwardTransport enum structure**
   - Main: Has `Stdio` and `StreamableHttp` 
   - Our branch: Has `Stdio` and `Http`
   - Need to: Update to `StreamableHttp` with `enable_sse` flag

2. **Function naming**
   - Main: `run_streamable_http_forward_proxy()`
   - Our branch: `run_http_forward()` in cli/forward.rs
   - Need to: Rename and update signature

3. **Command arguments**
   - Main: StreamableHttp has `url`, `enable_sse`, retry options
   - Our branch: Http has `port`, `target`, `command`
   - Need to: Align with new structure

### Secondary Conflicts
- Transport module changes (SSE transport improvements)
- Event ID generator (new in main, not in our refactor)
- Various clippy fixes

## Rebase Strategy

### Option 1: Interactive Rebase (Recommended)
**Pros:**
- Maintains clean history
- Can resolve conflicts commit by commit
- Preserves our refactor structure

**Cons:**
- Complex conflict resolution
- Risk of introducing bugs

**Steps:**
1. Create backup branch: `git branch cli-refactor-backup`
2. Start rebase: `git rebase -i main`
3. Resolve conflicts for each commit:
   - Apply transport changes to cli/forward.rs
   - Update ForwardTransport enum
   - Rename functions appropriately
4. Run tests after each resolution

### Option 2: Merge with Squash
**Pros:**
- Simpler conflict resolution
- Can review all changes at once

**Cons:**
- Loses individual commit history
- Harder to track specific changes

### Option 3: Cherry-pick and Reconstruct
**Pros:**
- Maximum control
- Can reorganize commits logically

**Cons:**
- Most time-consuming
- Manual process

## Detailed Migration Steps

### Phase 1: Prepare
1. Create backup branch
2. Document current ForwardTransport structure
3. Review all affected files

### Phase 2: Update Forward Module
1. **Update ForwardTransport enum** in cli/forward.rs:
   ```rust
   pub enum ForwardTransport {
       Stdio { 
           command_args: Vec<String>,
       },
       StreamableHttp {
           url: String,
           enable_sse: bool,
           retry_interval_ms: u64,
           max_retries: u32,
           command: Vec<String>, // for stdio backend
       },
   }
   ```

2. **Rename and update function**:
   - `run_http_forward()` → `run_streamable_http_forward()`
   - Add SSE transport support
   - Update function signature

3. **Update execute() method**:
   ```rust
   match self.transport {
       ForwardTransport::Stdio { .. } => { ... }
       ForwardTransport::StreamableHttp { .. } => { 
           run_streamable_http_forward(...).await
       }
   }
   ```

### Phase 3: Handle Transport Changes
1. Ensure SSE transport module changes are compatible
2. Update any references to transport types
3. Integrate MCP parser changes if needed

### Phase 4: Testing
1. Run all unit tests
2. Test each command variant:
   - `forward stdio`
   - `forward streamable-http`
3. Verify rate limiting still works
4. Check clippy warnings

### Phase 5: Cleanup
1. Remove any obsolete code
2. Update documentation
3. Ensure consistent naming throughout

## Risk Assessment

### High Risk Areas
1. **Forward command structure** - Major changes needed
2. **Transport initialization** - Different parameters
3. **Test compatibility** - May need test updates

### Medium Risk Areas
1. **Rate limiting integration** - Should be mostly compatible
2. **Session management** - Minimal changes expected

### Low Risk Areas
1. **Other commands** (reverse, record, replay) - Should be unaffected
2. **Common utilities** - Should remain compatible

## Estimated Timeline
- **Preparation**: 30 minutes
- **Rebase execution**: 2-3 hours
- **Testing and fixes**: 1-2 hours
- **Total**: 3.5-5.5 hours

## Rollback Plan
If rebase becomes too complex:
1. Reset to backup branch
2. Create new branch from main
3. Manually re-apply refactor with new transport structure
4. This would take longer but be cleaner

## Success Criteria
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] All CLI commands work as expected
- [ ] Forward proxy supports both stdio and streamable-http
- [ ] Clean git history maintained
- [ ] No functionality regression

## Recommendation

Given the analysis, I recommend **Option 1: Interactive Rebase** with the following approach:

1. **Start with a test rebase** to assess actual conflicts
2. **Focus on forward.rs changes first** since that's the main conflict
3. **Keep our modular structure** and adapt the new transport naming to it
4. **Run tests frequently** during the rebase process

The conflicts are manageable because:
- Our refactor is mostly structural (moving code to modules)
- Main's changes are mostly renaming and consolidation
- The two change sets are somewhat orthogonal

However, this will require careful attention to:
- Ensuring the new `StreamableHttp` transport works correctly
- Preserving all the new MCP/SSE improvements from main
- Maintaining our clean module separation

## Alternative: Incremental Approach

If the full rebase proves too complex, consider:
1. Merge main into our branch first (to see all conflicts at once)
2. Resolve conflicts while preserving our structure
3. Then do a clean rebase to organize history

## Next Steps
1. Review this plan
2. **Decision point**: Choose rebase strategy
3. Create backup branch: `git branch cli-refactor-backup`
4. Begin chosen approach
5. Document any issues encountered
6. Update tracker with results

---

**Created**: 2025-08-10  
**Status**: Planning  
**Complexity**: High  
**Estimated Effort**: 3.5-5.5 hours
**Recommendation**: Interactive rebase with focus on forward.rs transport updates