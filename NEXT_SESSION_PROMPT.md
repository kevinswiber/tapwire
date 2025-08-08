# Next Session Prompt - MCP Compliance Task 1.4

## Current Status
- **Project**: Tapwire/Shadowcat MCP Compliance Initiative
- **Progress**: 27.6% complete (8 of 29 tasks)
- **Phase 1**: 60% complete (3 of 5 tasks)
- **Just Completed**: Task 1.3 - SSE Reconnection Logic (production-ready)

## Completed in Previous Session (Task 1.3)
1. ✅ Implemented complete SSE reconnection with exponential backoff
2. ✅ Fixed critical `block_on()` anti-pattern with proper async state machine
3. ✅ Added comprehensive documentation with ASCII art diagrams
4. ✅ Applied performance optimizations (eliminated allocations, cached Arcs)
5. ✅ Refactored HTTP errors to use structured status codes
6. ✅ All 67 SSE tests passing, no clippy warnings

## Key Implementation Notes
- **AsyncOperation Pattern**: The async state machine pattern from Task 1.3 is well-documented and should be reused for Task 1.4's session lifecycle management
- **Documentation**: `/Users/kevin/src/tapwire/plans/mcp-compliance/implementation-notes/task-1.3-async-patterns.md` contains critical patterns to follow
- **Retry Enhancement TODO**: Design for Retry-After header support is in `/Users/kevin/src/tapwire/SSE_RECONNECT_RETRY_REFACTOR.md` (for future work)

## Next Task: 1.4 - SSE Session Integration

**File**: `/Users/kevin/src/tapwire/plans/mcp-compliance/tasks/phase-1-task-004-sse-session-integration.md`

### Objectives
1. Link SSE connections to MCP sessions
2. Track SSE streams per session
3. Handle session-scoped event IDs
4. Coordinate session lifecycle with connections
5. Implement session-aware reconnection

### Foundation from Task 1.3
- `ReconnectingStream` with proper async state machine
- `EventTracker` ready for session-scoped enhancement
- `HealthMonitor` for session lifecycle coordination
- Cached Arc pattern for performance

### Critical Patterns to Follow
- **NO `block_on()`** in Stream implementations
- Use AsyncOperation state machine for async operations in poll context
- Use `mem::replace` for clean state transitions
- Cache Arc references to avoid repeated cloning

## Key Files
- **Main work area**: `/Users/kevin/src/tapwire/shadowcat/src/transport/sse/`
- **Session system**: `/Users/kevin/src/tapwire/shadowcat/src/session/`
- **Compliance tracker**: `/Users/kevin/src/tapwire/plans/mcp-compliance/compliance-tracker.md`
- **Task spec**: `/Users/kevin/src/tapwire/plans/mcp-compliance/tasks/phase-1-task-004-sse-session-integration.md`

## Commands to Run First
```bash
cd /Users/kevin/src/tapwire/shadowcat
cargo test sse -- --quiet  # Verify all 67 tests still pass
cargo clippy --all-targets -- -D warnings  # Check for warnings
```

## Important Context
- Shadowcat is a git submodule - commit changes in shadowcat repo first, then update parent
- Target < 5% performance overhead for all operations
- Session timeout defaults: 30 minutes total, 5 minutes idle
- Max 10 connections per session by default

## Success Criteria for Task 1.4
- [ ] SSE connections properly associated with sessions
- [ ] Session ID headers included in all SSE requests
- [ ] Event IDs scoped to sessions as per spec
- [ ] Clean connection cleanup on session termination
- [ ] Session state preserved across reconnections
- [ ] Multiple streams per session properly managed
- [ ] Session expiry handled gracefully
- [ ] Proper isolation between sessions
- [ ] Test coverage for session scenarios

## Notes
- The SSE implementation is now very robust with excellent error handling
- Performance has been optimized based on rust-code-reviewer analysis
- Documentation is comprehensive - use it as a reference
- The async patterns are critical - follow them carefully to avoid deadlocks