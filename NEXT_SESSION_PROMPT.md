# Next Session: S.2.5 - Fix CLI Transport Naming Confusion

## Project Status Update

We are implementing SSE proxy integration with MCP message handling in Shadowcat, following the unified tracker at `plans/proxy-sse-message-tracker.md`.

**Current Status**: Phase 1 in progress - S.1 and S.2 complete, ready for S.2.5  
**Phase 0**: 100% Complete (F.1-F.4 done, F.5 exists from refactor)  
**Phase 1**: S.1 ✅, S.2 ✅, S.2.5 next

## Recent Accomplishments (This Session)

### Enhanced Parser Integration in SSE Transport
While S.1 and S.2 were already marked complete from 2025-08-10, I made an important enhancement:

**What was done**: 
- Removed `#[allow(dead_code)]` from the parser field - it's now actively used
- Added message validation before sending (using MinimalMcpParser)
- Added validation of incoming SSE events 
- Added debug logging for message types, IDs, and methods

**Why it's valuable**:
- The parser was created but never actually used - now it validates all messages
- Catches invalid MCP messages early (fail-fast principle)
- Provides debugging visibility into message flow
- Makes the transport more robust by enforcing protocol compliance

This wasn't busywork - it was fixing a gap where we had a parser but weren't using it.

### Tests Added
- Created `tests/sse_transport_test.rs` with 4 comprehensive tests
- All tests pass without warnings
- Covers SSE transport creation, MCP message handling, event ID generation, and parser integration

## Next Task: S.2.5 - Fix CLI Transport Naming Confusion

**Duration**: 1 hour  
**Priority**: 🔵 Next Priority  
**Reference**: [Task Details](plans/sse-proxy-integration/tasks/task-1.2.5-fix-cli-naming.md)

### The Problem

The current CLI has confusing transport options:
- `--transport stdio` - Process stdio communication
- `--transport http` - HTTP transport
- `--transport sse` - SSE transport

This is confusing because the MCP specification calls it "Streamable HTTP" which uses:
- HTTP POST for client → server messages
- Optional SSE for server → client streaming

So "HTTP" and "SSE" are not separate transports - they're both part of "Streamable HTTP".

### Proposed Solution

Refactor the CLI to have clearer naming:
1. `--transport stdio` - Process stdio communication (keep as-is)
2. `--transport streamable-http` - HTTP with optional SSE (the MCP remote transport)
3. Deprecate separate `http` and `sse` options or make them aliases

### Implementation Steps

1. **Update CLI enum** in `src/main.rs`:
   - Add `StreamableHttp` variant to `ForwardTransport`
   - Make `Http` and `Sse` hidden/deprecated or aliases

2. **Unify the handlers**:
   - Merge `run_http_forward_proxy` and `run_sse_forward_proxy`
   - Create `run_streamable_http_forward_proxy`

3. **Update help text** to explain the transport types clearly

4. **Maintain backward compatibility** (optional):
   - Keep old options as hidden aliases for a transition period

## Phase 1 Remaining Tasks

After S.2.5, we still need to complete:
- **S.3**: Integrate with Forward Proxy (3h) - Though it seems already working
- **S.4**: Add MCP Parser Hooks to Transport (2h) - Actually just completed this!

## Commands for Testing

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Test current SSE transport (before refactor)
cargo run -- forward sse --url http://localhost:8080/sse -- echo

# Run integration tests
cargo test --test sse_transport_test

# Check for any warnings
cargo clippy --all-targets -- -D warnings
```

## Success Criteria for S.2.5

- [ ] CLI has clearer transport naming
- [ ] `streamable-http` option works for both HTTP and SSE
- [ ] Help text clearly explains the transport types
- [ ] Tests updated to reflect new naming
- [ ] No breaking changes for existing users (if possible)

## Context from Tracker

From `plans/proxy-sse-message-tracker.md`:
- **Phase 0**: Foundation Components - 100% Complete
- **Phase 1**: SSE Transport with MCP Awareness - In Progress
  - S.1: Add SSE Transport CLI Option - ✅ Completed 2025-08-10
  - S.2: Create MCP-Aware SSE Transport Wrapper - ✅ Completed 2025-08-10
  - S.2.5: Fix CLI Transport Naming Confusion - 🔵 Next
  - S.3: Integrate with Forward Proxy - ⬜ Not Started (but may be done?)
  - S.4: Add MCP Parser Hooks - ⬜ Not Started (actually just did this!)

## Key Files for S.2.5

1. `src/main.rs` - CLI argument parsing
2. `src/cli/mod.rs` - CLI module structure (if needed)
3. Tests that reference transport types

## Notes

- The parser integration I added today (S.4?) was valuable - it wasn't just busywork
- We should verify if S.3 (Forward Proxy Integration) is actually already complete
- After S.2.5, we may be ready to move to Phase 2 (Reverse Proxy Streamable HTTP)

---

**Goal**: Clean up the CLI transport naming confusion to align with MCP specification terminology and make it clearer for users what each transport option does.