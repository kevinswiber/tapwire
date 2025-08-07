# Task 008.1: Fix Session Matching Design Flaws

## Context

You are working on the Shadowcat project, a high-performance Model Context Protocol (MCP) proxy written in Rust. The project is undergoing systematic refactoring based on a comprehensive review.

### Current Status

- **Phase 1 (Critical Safety)**: ✅ COMPLETE - All 4 tasks finished
  - Removed 35 production unwraps
  - Fixed duplicate error types
  - Added request size limits
  - Fixed blocking I/O in async contexts
  
- **Phase 2 (Core Features)**: IN PROGRESS - 4/5 core tasks complete + 1 critical fix needed
  - ✅ Task 005: Record Command implemented
  - ✅ Task 006: Replay Command implemented
  - ✅ Task 007: Rate Limiting implemented
  - ✅ Task 008: Session Matching implemented (but with critical flaws)
  - 🔴 **Task 008.1: Fix Session Matching Design Flaws** (YOUR CURRENT TASK - BLOCKER)
  - ⏳ Task 009: Session Cleanup (blocked by 008.1)

- **Production Readiness**: 96/100 (-2 points due to critical issues)

### Working Directory

```
/Users/kevin/src/tapwire/shadowcat
```

## Objective: Fix Critical Design Flaws in Session Matching

### Problem Statement

Task 008 successfully implemented session matching functionality, but review discovered 10 critical design flaws that make the feature non-functional and potentially dangerous in production. These MUST be fixed before proceeding to Task 009.

### Critical Issues (Priority Order)

1. **🔴 Memory Leak**: Pending requests never cleaned up for ended sessions
2. **🔴 Race Condition**: Shutdown detection has TOCTOU bug between check and process
3. **🔴 InterceptContext Metadata Never Populated**: Session matching is completely non-functional
4. **🔴 No Session Recovery**: Lost session context cannot be recovered
5. **🟡 State Confusion**: SessionState and SessionStatus overlap and conflict
6. **🟡 No DoS Protection**: Unbounded pending_requests HashMap
7. **🟡 Fragile Detection**: Initialized response detection assumes specific structure
8. **🟡 Error Handling**: State modifications continue after failures
9. **🟢 Session Tags Never Set**: Feature exists but is never used
10. **🟢 No Transactional Guarantees**: Multiple state updates without atomicity

### Essential Context Files to Read

1. **Task Definition**: `/Users/kevin/src/tapwire/plans/refactors/task-008-1-session-matching-fixes.md`
2. **Session Manager**: `src/session/manager.rs` - Contains memory leak and race conditions
3. **Session Store**: `src/session/store.rs` - Has state confusion issues
4. **Forward Proxy**: `src/proxy/forward.rs` - Where InterceptContext is created without metadata
5. **Interceptor Rules**: `src/interceptor/rules.rs` - Expects metadata that's never provided
6. **Refactor Tracker**: `/Users/kevin/src/tapwire/plans/refactors/shadowcat-refactor-tracker.md`

## Implementation Strategy

### Phase 1: Critical Memory & Safety Fixes (MUST DO FIRST)

1. **Fix Memory Leak in pending_requests**
   - Add session-scoped cleanup when sessions end
   - Modify `complete_session()` and `fail_session()` to clean up pending requests
   - Consider two-level HashMap: `HashMap<SessionId, HashMap<RequestId, PendingRequest>>`

2. **Fix Race Condition in Shutdown Detection**
   - Make `is_shutdown_response()` atomic with removal
   - Use `remove()` instead of separate `get()` and later `remove()`
   - Store request type in PendingRequest

3. **Populate InterceptContext Metadata**
   - In `src/proxy/forward.rs`, look up session before creating InterceptContext
   - Add metadata for frame_count, session_duration_ms, session_tags
   - Make session matching actually functional

4. **Add Session Recovery Mechanism**
   - Add fallback in `extract_session_id()` to use current session context
   - Pass session ID through transport layer
   - Log warnings when session context is lost

### Phase 2: Design Improvements (IMPORTANT)

1. **Consolidate State Management**
   - Either remove SessionStatus or SessionState (pick one)
   - If keeping both, clearly define relationship
   - Add invariant checks

2. **Add DoS Protection**
   - Limit pending requests per session (e.g., max 1000)
   - Add global limit
   - Return errors when exceeded

3. **Improve Response Detection**
   - Track request types in pending_requests
   - Match responses based on original request type
   - Remove fragile protocol-specific assumptions

### Phase 3: Robustness (NICE TO HAVE)

1. **Transactional State Updates**
   - Collect all changes first
   - Apply atomically or rollback
   - Use Unit of Work pattern

2. **Implement Session Tagging**
   - Define automatic tags (e.g., "long-running", "high-volume")
   - Add tagging logic in process_message_for_session

## Success Criteria Checklist

- [ ] No memory leaks - pending_requests cleaned up with sessions
- [ ] No race conditions - atomic operations for concurrent access
- [ ] InterceptContext metadata properly populated from session
- [ ] Session matching actually works (test with interceptor rules)
- [ ] Session recovery mechanism in place
- [ ] State consistency guaranteed (SessionState vs SessionStatus resolved)
- [ ] DoS protection with request limits
- [ ] All 359+ existing tests still passing
- [ ] New tests for all fixes passing
- [ ] Clean `cargo fmt` output
- [ ] Clean `cargo clippy --all-targets -- -D warnings` output

## Commands to Use

### Development Commands

```bash
# Find the memory leak location
rg "pending_requests" --type rust

# Check where InterceptContext is created
rg "InterceptContext::new" --type rust -A 5

# Run session tests
cargo test session::
cargo test intercept::

# Test with debug logging
RUST_LOG=shadowcat=debug cargo test session_matching

# Check for race conditions
cargo test --release -- --test-threads=1

# Watch for changes
cargo watch -x check -x test

# Format and lint
cargo fmt
cargo clippy --all-targets -- -D warnings
```

### Verification Commands

```bash
# Ensure no memory leaks
valgrind --leak-check=full ./target/debug/shadowcat forward stdio -- echo '{}'

# Run all tests
cargo test

# Check session matching works
cargo run -- forward stdio -- echo '{"jsonrpc":"2.0","method":"initialize","id":1}'

# Stress test for DoS protection
for i in {1..10000}; do echo '{"jsonrpc":"2.0","method":"test","id":'$i'}'; done | cargo run -- forward stdio -- cat
```

## Important Notes

- **Always use TodoWrite tool** to track your progress through the task
- **Start with examining existing code** to understand current architecture
- **Follow established patterns** from previous implementations
- **Test incrementally** as you build each component
- **Run `cargo fmt`** after implementing new functionality
- **Run `cargo clippy --all-targets -- -D warnings`** before any commit
- **Update the refactor tracker** when the task is complete
- **Focus on the current phase objectives**

## Model Usage Guidelines

- **IMPORTANT** Be mindful of model capabilities. Assess whether Claude Opus or Claude Sonnet would be best for each step. When there's a benefit to a model change, pause and recommend it. Be mindful of the context window. When the context window has less than 15% availability, suggest creating a new Claude session and output a good prompt, referencing all available plans, tasks, and completion files that are relevant. Save the prompt into NEXT_SESSION_PROMPT.md.

## Development Workflow

1. Create todo list with TodoWrite tool to track progress
2. Examine existing codebase architecture and established patterns
3. Study current implementations related to the task
4. Design the solution approach and identify key components
5. Implement functionality incrementally with frequent testing
6. Add comprehensive error handling following project patterns
7. Create tests demonstrating functionality works correctly
8. Run tests after each significant change to catch issues early
9. Run `cargo fmt` to ensure consistent code formatting
10. Run `cargo clippy -- -D warnings` to catch potential issues
11. Update project documentation and tracker as needed
12. Commit changes with clear, descriptive messages

## Expected Deliverables

1. **Fixed Memory Management**
   - No memory leaks in pending_requests
   - Proper cleanup on session end
   - DoS protection with limits

2. **Working Session Matching**
   - InterceptContext metadata populated
   - Session matching rules functional
   - Session recovery mechanism

3. **Consistent State Management**
   - SessionState/SessionStatus confusion resolved
   - Atomic state updates
   - No race conditions

4. **Comprehensive Tests**
   - Tests for memory leak prevention
   - Tests for race condition fixes
   - Tests for session matching with metadata
   - Tests for DoS protection

5. **Clean Code**
   - No clippy warnings
   - Properly formatted with cargo fmt
   - Following established project patterns

## Critical Patterns to Follow

Based on completed tasks, follow these patterns:

- Use `anyhow::Context` for error context
- Return proper Result types from all functions
- Use Arc<RwLock<>> only when justified (document why)
- Add tracing logs for debugging
- Handle all error cases explicitly (no unwraps)
- Use #[instrument] on key functions

## Start Here

1. First, create a TodoWrite list with the major implementation steps
2. Read `src/session/manager.rs` to understand the memory leak
3. Check `src/proxy/forward.rs` to see where InterceptContext lacks metadata
4. Begin with Phase 1 critical fixes (memory leak first)
5. Test frequently as you build

Remember: This is a CRITICAL BLOCKER. Task 009 cannot proceed until these issues are fixed. The session matching feature is currently non-functional and has memory leaks that could cause production outages.

Good luck with Task 008.1!