# Task 008: Complete Session Matching for Shadowcat

## Context

You are working on the Shadowcat project, a high-performance Model Context Protocol (MCP) proxy written in Rust. The project is undergoing systematic refactoring based on a comprehensive review.

### Current Status

- **Phase 1 (Critical Safety)**: ✅ COMPLETE - All 4 tasks finished
  - Removed 35 production unwraps
  - Fixed duplicate error types
  - Added request size limits
  - Fixed blocking I/O in async contexts
- **Phase 2 (Core Features)**: IN PROGRESS - 3/5 tasks complete
  - ✅ Task 005: Implement Record Command
  - ✅ Task 006: Implement Replay Command
  - ✅ Task 007: Implement Rate Limiting
  - 🔄 **Task 008: Complete Session Matching** (YOUR CURRENT TASK)
  - ⏳ Task 009: Implement Session Cleanup

### Working Directory

```
/Users/kevin/src/tapwire/shadowcat
```

## Objective: Complete Session Matching Implementation

### Problem Statement

The session matching logic is marked as CRITICAL but has a TODO stub at `src/session/manager.rs:108`. This functionality is essential for:

- Associating MCP requests with responses
- Tracking session lifecycle
- Managing session state transitions
- Proper cleanup and resource management

### Essential Context Files to Read

1. **Task Definition**: `/Users/kevin/src/tapwire/plans/refactors/task-008-session-matching.md`
2. **Session Manager**: `src/session/manager.rs` - Current implementation with TODO
3. **Session Module**: `src/session/mod.rs` - Session struct and types
4. **Refactor Tracker**: `/Users/kevin/src/tapwire/plans/refactors/shadowcat-refactor-tracker.md`
5. **Transport Types**: `src/transport/mod.rs` - TransportMessage enum
6. **MCP Protocol**: Review MCP message types and session handling

## Implementation Strategy

### Phase 1: Analysis (Start Here)

1. Use TodoWrite tool to create task tracking list
2. Examine `src/session/manager.rs` to understand current implementation
3. Find the TODO at line 108 and analyze what's missing
4. Review how sessions are currently created and tracked
5. Understand MCP message flow (initialize, requests, responses, shutdown)

### Phase 2: Core Implementation

1. **Session State Machine**

   - Add SessionState enum (Initializing, Active, ShuttingDown, Closed, Failed)
   - Implement state transitions
   - Add validation for invalid transitions

2. **Session ID Extraction**

   - Extract session IDs from MCP messages
   - Handle initialize requests (generate new session)
   - Extract from headers (Mcp-Session-Id)
   - Match responses to pending requests

3. **Request-Response Correlation**
   - Track pending requests with session IDs
   - Match responses to their originating sessions
   - Clean up completed request-response pairs

### Phase 3: Lifecycle Management

1. **Timeout Handling**

   - Implement cleanup for stale requests (30-second timeout)
   - Log warnings for timed-out requests
   - Prevent memory leaks from orphaned requests

2. **Session Cleanup**
   - Handle shutdown messages properly
   - Clean up resources on session close
   - Remove pending requests for closed sessions

### Phase 4: Testing & Validation

1. Write unit tests for:

   - Session creation from initialize
   - Request-response matching
   - State transitions
   - Timeout handling

2. Write integration tests for:
   - Full session lifecycle
   - Concurrent session handling
   - Error recovery

## Success Criteria Checklist

- [ ] TODO comment at `src/session/manager.rs:108` removed
- [ ] Session matching handles all MCP message types:
  - [ ] initialize/initialized
  - [ ] ping/pong
  - [ ] Tool calls and responses
  - [ ] Resource operations
  - [ ] Prompt operations
  - [ ] shutdown
- [ ] Request-response correlation working
- [ ] Session state transitions implemented
- [ ] Stale request cleanup working
- [ ] Comprehensive tests passing
- [ ] No performance degradation
- [ ] All existing 349+ tests still passing

## Commands to Use

### Development Commands

```bash
# Find the TODO
rg "TODO.*session matching" --type rust

# Run session tests
cargo test session::manager
cargo test session::

# Test with actual MCP messages
cargo run -- forward stdio -- echo '{"jsonrpc":"2.0","method":"initialize","id":1}'

# Watch for changes
cargo watch -x check -x test

# Format and lint
cargo fmt
cargo clippy --all-targets -- -D warnings
```

### Verification Commands

```bash
# Ensure TODO is removed
rg "TODO" src/session/manager.rs

# Run all tests
cargo test

# Check for warnings
cargo clippy --all-targets -- -D warnings

# Performance check
cargo test --release
```

## Important Notes

- **Always use TodoWrite tool** to track your progress through the task
- **Start with examining existing code** to understand current architecture
- **Follow established patterns** from previous implementations
- **Test incrementally** as you build each component
- **Run `cargo fmt`** after implementing new functionality
- **Run `cargo clippy -- -D warnings`** before any commit
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

1. **Completed Session Matching Logic**

   - Session ID extraction from all message types
   - Request-response correlation
   - State machine for session lifecycle
   - Cleanup mechanisms for stale data

2. **Comprehensive Tests**

   - Unit tests for all new functions
   - Integration tests for session lifecycle
   - Edge case handling tests

3. **Documentation Updates**

   - Remove TODO comment
   - Add inline documentation for complex logic
   - Update refactor tracker with completion status

4. **Clean Code**
   - No clippy warnings
   - Properly formatted with cargo fmt
   - Following established project patterns

## Critical Patterns to Follow

Based on completed tasks, follow these patterns:

- Use `anyhow::Context` for error context
- Return `Result<T, SessionError>` from session functions
- Use `Arc<RwLock<>>` for shared state (but justify usage)
- Add tracing logs for debugging
- Handle all error cases explicitly (no unwraps)

## Start Here

1. First, create a TodoWrite list with the major implementation steps
2. Read `src/session/manager.rs` to find the TODO and understand context
3. Examine how sessions are currently created and managed
4. Begin implementing the session state machine
5. Test frequently as you build

Remember: Session matching is CRITICAL functionality that affects recording, replay, and interception features. Take care to implement it correctly and thoroughly test all scenarios.

Good luck with Task 008!

