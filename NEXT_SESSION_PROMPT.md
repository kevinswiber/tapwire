# Task 009.1: Fix Critical Session Cleanup Design Flaws

## Context

You are working on the Shadowcat project, a high-performance Model Context Protocol (MCP) proxy written in Rust. Task 009 (Session Cleanup) was completed but a comprehensive code review revealed **CRITICAL design flaws** that must be fixed immediately before production deployment.

### Current Status

- **Phase 1 (Critical Safety)**: ✅ COMPLETE - All 4 tasks finished
- **Phase 2 (Core Features)**: 6/7 tasks complete - Task 009.1 CRITICAL fixes needed
  - ✅ Task 005: Record Command implemented
  - ✅ Task 006: Replay Command implemented
  - ✅ Task 007: Rate Limiting implemented
  - ✅ Task 008: Session Matching implemented
  - ✅ Task 008.1: Session Matching Design Flaws fixed
  - ✅ Task 009: Session Cleanup implemented
  - 🔴 **Task 009.1: Fix Critical Session Cleanup Design Flaws** (YOUR CURRENT TASK)

- **Production Readiness**: 90/100 ⚠️ (reduced from 98 due to critical issues)

### Working Directory

```
/Users/kevin/src/tapwire/shadowcat
```

## Critical Issues to Fix

### 🔴 CRITICAL Issue 1: Deadlock in LRU Eviction

**Location**: `src/session/manager.rs:383-402`

The `evict_lru_sessions()` method holds a write lock on `lru_queue` while calling `delete_session()`, which also tries to acquire the same write lock. This creates a **guaranteed deadlock** that will freeze the system.

### 🔴 CRITICAL Issue 2: Race Condition in Metrics

**Location**: `src/session/manager.rs:341-346`

Metrics incorrectly assume all deleted sessions were active, leading to incorrect or negative session counts.

### 🟠 HIGH Priority Issues

- **Memory Leak in LRU Queue**: Duplicate entries cause unbounded growth
- **O(n) Cleanup Performance**: Linear scan of all sessions
- **Missing Backpressure**: No rate limiting on request tracking

## Essential Files to Review

1. **Code Review Document**: `/Users/kevin/src/tapwire/reviews/session-cleanup-review-2025-08-07.md`
   - Contains detailed analysis and recommended fixes for all issues
2. **Task Definition**: `/Users/kevin/src/tapwire/plans/refactors/task-009.1-session-cleanup-fixes.md`
3. **Session Manager**: `src/session/manager.rs` - Primary focus for fixes
4. **Refactor Tracker**: `/Users/kevin/src/tapwire/plans/refactors/shadowcat-refactor-tracker.md`

## Implementation Priority

### Immediate (Must fix before ANY other work):

1. **Fix the deadlock in `evict_lru_sessions`**
   - Release lock before calling `delete_session`
   - See review document for exact fix

2. **Fix race condition in metrics**
   - Track active sessions separately
   - Only decrement active count for actually active sessions

### Short-term (Fix in this session):

3. **Replace VecDeque with LinkedHashMap for LRU**
   - Prevents duplicate entries
   - Maintains O(1) operations

4. **Improve cleanup performance**
   - Consider priority queue for O(log n) cleanup
   - Or at minimum, optimize the linear scan

5. **Add backpressure for request tracking**
   - Implement rate limiting per session
   - Prevent system overload from rapid requests

## Success Criteria Checklist

- [ ] No deadlocks in LRU eviction (test with concurrent operations)
- [ ] Metrics accurately track active/cleaned sessions
- [ ] LRU queue maintains unique entries only
- [ ] Cleanup performance improved from O(n)
- [ ] Request tracking has proper backpressure
- [ ] All existing tests still pass
- [ ] New concurrent tests added and passing
- [ ] Memory usage remains bounded under load
- [ ] Clean `cargo fmt` output
- [ ] Clean `cargo clippy --all-targets -- -D warnings` output

## Commands to Use

### Testing Commands

```bash
# Test for deadlocks
cargo test evict_lru --release -- --test-threads=1

# Test concurrent operations
cargo test concurrent_cleanup

# Check for memory leaks (if valgrind available)
valgrind --leak-check=full ./target/debug/shadowcat session cleanup --all

# Verify metrics accuracy
cargo test metrics_accuracy

# Run all session tests
cargo test session

# Format and lint
cargo fmt
cargo clippy --all-targets -- -D warnings
```

### Development Workflow

1. Create todo list with TodoWrite tool
2. Read the comprehensive review document first
3. Fix CRITICAL issues first (deadlock and race condition)
4. Test each fix incrementally
5. Add concurrent tests for each fix
6. Fix HIGH priority issues
7. Run full test suite
8. Update refactor tracker when complete

## Important Notes

- **This is a CRITICAL blocker** - The system cannot be deployed with these issues
- **The deadlock will freeze production** - Must be fixed immediately
- **Follow the exact fixes in the review document** - They have been carefully designed
- **Test with concurrent operations** - Single-threaded tests won't catch these issues
- **Consider using `parking_lot::RwLock`** - Better performance and deadlock detection
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

1. **Fixed `evict_lru_sessions`** - No deadlock possibility
2. **Fixed metrics tracking** - Accurate active/cleaned counts
3. **LinkedHashMap LRU** - No duplicate entries
4. **Improved cleanup algorithm** - Better than O(n) if possible
5. **Backpressure mechanism** - Rate limiting on requests
6. **Comprehensive concurrent tests** - Prove fixes work under load
7. **Updated documentation** - Document the fixes and why they were needed

## Start Here

1. First, create a TodoWrite list with the critical fixes
2. Read `/Users/kevin/src/tapwire/reviews/session-cleanup-review-2025-08-07.md` thoroughly
3. Fix the deadlock IMMEDIATELY - this is the highest priority
4. Test the deadlock fix with concurrent operations
5. Then proceed with the race condition fix
6. Continue with HIGH priority issues

Remember: These are not theoretical issues - they are **guaranteed bugs** that will cause production failures. The fixes in the review document have been carefully designed to resolve them properly.

Good luck with Task 009.1!