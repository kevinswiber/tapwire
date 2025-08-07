# Next Claude Session Prompt - Shadowcat Refactor Task 003

## Context

You are continuing the systematic refactoring of the Shadowcat Rust proxy codebase. **Tasks 001 and 002 have been successfully completed**:
- Task 001: All 35 production unwrap calls eliminated ✅
- Task 002: Duplicate error types consolidated ✅

## Your Current Objective

**Continue Phase 1 with Task 003: Add Request Size Limits**

Implement configurable request size limits to protect against memory exhaustion attacks and ensure stable operation under load.

## Essential Context Files

Please read these files to understand your current task:

1. **Task Definition**: `/Users/kevin/src/tapwire/plans/refactors/task-003-add-size-limits.md` (create if it doesn't exist)
2. **Overall Refactor Plan**: `/Users/kevin/src/tapwire/plans/refactors/shadowcat-refactor-tracker.md`
3. **Original Review**: `/Users/kevin/src/tapwire/reviews/shadowcat-comprehensive-review-2025-08-06.md`

## Working Directory

`/Users/kevin/src/tapwire/shadowcat`

## Critical Development Practices

**IMPORTANT: Code Quality Standards**
- Run `cargo fmt` after every significant code change
- Run `cargo clippy -- -D warnings` before EVERY commit
- Both commands MUST pass with zero errors/warnings before committing

## What Has Been Accomplished

### Task 001 ✅
- 35 production unwraps eliminated (560 → 525, remaining are test-only)
- Added 4 new error variants: `SystemTime`, `AddressParse`, `RequiredFieldMissing`, `InvalidUpstreamConfig`
- All 341 tests passing
- Clean clippy output

### Task 002 ✅
- Removed duplicate `AuthenticationError(String)` and `ConfigurationError(String)` from `ShadowcatError`
- Updated all usages to use proper `AuthError` and `ConfigError` enums
- Fixed compilation issues in `audit/store.rs`
- Consolidated error handling patterns

## Your Task 003 Objectives

1. **Implement request size limits** at the transport layer
2. **Add configuration options** for max request/response sizes
3. **Create validation middleware** that rejects oversized requests early
4. **Add appropriate error responses** when limits are exceeded
5. **Test with large payloads** to verify protection
6. **Document the configuration** in the appropriate places

## Success Criteria for Task 003

- [ ] Request size limits configurable via config file
- [ ] Default sensible limits (e.g., 10MB for requests, 100MB for responses)
- [ ] Early rejection of oversized requests before full parsing
- [ ] Clear error messages when limits exceeded
- [ ] Tests for boundary conditions
- [ ] All existing tests still pass
- [ ] `cargo fmt` passes
- [ ] `cargo clippy -- -D warnings` passes

## Commands to Use

```bash
# Format code (run after changes)
cargo fmt

# Check for clippy warnings (run before commits)
cargo clippy -- -D warnings

# Run tests
cargo test

# Check specific modules
cargo test transport::
cargo test proxy::

# Check compilation
cargo check
```

## Implementation Guidance

1. **Start with configuration**: Add size limit fields to relevant config structs
2. **Implement at transport layer**: Add size checking in stdio, HTTP transports
3. **Use streaming where possible**: Don't load entire payloads into memory
4. **Consider different limits**: Headers vs body, request vs response
5. **Add metrics**: Track rejected requests due to size limits

## Current Phase Status

**Phase 1: Critical Safety (2/4 tasks complete)**
- ✅ Task 001: Remove All Unwrap Calls (COMPLETED)
- ✅ Task 002: Fix Duplicate Error Types (COMPLETED)
- 🔄 Task 003: Add Request Size Limits (CURRENT)
- ⏳ Task 004: Fix Blocking IO in Async

## Important Notes

- **Always use TodoWrite tool** to track your progress through the task
- **Test frequently** to catch issues early
- **Run `cargo fmt`** after implementing new functionality
- **Run `cargo clippy -- -D warnings`** before any commit
- **Update the refactor tracker** when Task 003 is complete
- **Focus on security**: Size limits are a critical defense against DoS attacks

## Model Usage Strategy

Use **OPUS for**:
- Initial design of size limit architecture
- Complex decisions about where to implement checks
- Security considerations and threat modeling

Use **SONNET for**:
- Mechanical code changes
- Running validation commands
- Writing tests
- Updating documentation

## Development Workflow

1. Create todo list with TodoWrite tool
2. Read existing transport and config code
3. Design size limit approach
4. Implement configuration changes
5. Add size checking to transports
6. Write comprehensive tests
7. Run `cargo fmt`
8. Run `cargo clippy -- -D warnings`
9. Ensure all tests pass
10. Update refactor tracker documentation
11. Commit with clear message

Begin by using the TodoWrite tool to create a task list, then analyze the current transport implementations to understand where size limits should be added.