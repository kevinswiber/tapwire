# Next Session: Complete Phase B of Shadowcat CLI Optimization

## Context

We are optimizing Shadowcat's CLI refactor to transform it from a functional CLI tool into a production-ready library. The work is tracked in `plans/cli-refactor-optimization/cli-optimization-tracker.md`.

## Current Status

**Phase B: 75% Complete** (as of 2025-08-11)

### Completed in Phase B:
- ✅ B.1: Implement Builder Patterns (6h) - All builders working with tests
- ✅ B.2: Add Graceful Shutdown (4h) - Full shutdown system with 681 passing tests
- ✅ B.2.1: Fix Shutdown Integration (4h) - Fixed with real proxy implementations
- ✅ B.3: Create High-Level API (3h) - Clean high-level API with handles, 4 examples
- ✅ B.6: Add Basic Integration Tests (2h) - 781 total tests, all passing

### Remaining in Phase B:
- ⬜ B.4: Extract Transport Factory (3h)
- ⬜ B.5: Standardize Error Handling (2h)

## Working Directory

The work is happening in a git worktree:
```bash
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor
```

This is a separate worktree of the shadowcat submodule on branch `shadowcat-cli-refactor`.

## Test Status

All 781 tests are passing:
- 684 unit tests
- 97 E2E tests  
- 20 active integration tests (14 simple, 6 mock-based)
- 10 ignored integration tests (require stdin/stdout, documented why)

## Next Tasks

### B.4: Extract Transport Factory (3h)

**Goal**: Create a centralized factory for transport creation to reduce code duplication.

**Current Issues**:
- Transport creation logic is scattered across CLI modules
- Each command duplicates stdio/HTTP transport setup
- No consistent error handling for transport creation

**Implementation Plan**:
1. Create `src/transport/factory.rs` with `TransportFactory` struct
2. Implement methods for each transport type:
   - `create_stdio_client()` -> `StdioClientTransport`
   - `create_stdio_server(cmd)` -> `StdioTransport`
   - `create_http_client(url)` -> `HttpTransport`
   - `create_sse_client(url)` -> `SseTransport`
3. Add configuration options (timeouts, buffer sizes, etc.)
4. Update all CLI commands to use factory
5. Add tests for factory methods

**Success Criteria**:
- All transport creation goes through factory
- Consistent error handling
- Reduced code duplication
- Tests for all factory methods

### B.5: Standardize Error Handling (2h)

**Goal**: Ensure consistent error handling throughout the library.

**Current Issues**:
- Mix of `Result<T, ShadowcatError>` and `anyhow::Result<T>`
- Some errors printed to stderr, others returned
- Inconsistent error context and messages

**Implementation Plan**:
1. Audit all public APIs for error types
2. Ensure all use `Result<T, ShadowcatError>`
3. Add proper error context with `.context()`
4. Remove any remaining `println!` or `eprintln!` from library code
5. Ensure errors bubble up properly to CLI layer
6. Add error conversion traits where needed

**Success Criteria**:
- All public APIs return `Result<T, ShadowcatError>`
- No direct printing to stderr in library code
- Consistent error messages with context
- Clean error handling in examples

## Commands to Run

```bash
# Navigate to worktree
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Check current status
git status
cargo test --quiet  # Should show all 781 tests passing

# For B.4 Transport Factory:
# Create src/transport/factory.rs
# Update imports and exports
# Run tests after each change

# For B.5 Error Handling:
# Audit with: rg "anyhow::Result" src/
# Check for: rg "println!|eprintln!" src/ --glob '!main.rs'
# Ensure consistent error types
```

## Important Notes

1. **Worktree**: We're working in `/Users/kevin/src/tapwire/shadowcat-cli-refactor`, not the main shadowcat directory
2. **Tests**: Keep all 781 tests passing as you make changes
3. **Commit Often**: Make small, focused commits for each logical change
4. **Documentation**: Update task files in `plans/cli-refactor-optimization/tasks/` as you complete work

## Phase C Preview

Once B.4 and B.5 are complete, Phase B will be done and we'll move to Phase C (Quality & Testing):
- C.1: Comprehensive Documentation (4h)
- C.2: Configuration File Support (3h)
- C.3: Improve Error Messages (2h)
- C.4: Add Telemetry/Metrics (4h)
- C.5: Performance Optimization (6h)
- C.6: Extensive Test Coverage (6h)
- C.7: CLI Shell Completions (2h)

## Success Metrics

Phase B will be complete when:
- ✅ Transport factory eliminates duplication
- ✅ All public APIs have consistent error handling
- ✅ All 781+ tests still pass
- ✅ Library is usable without any CLI dependencies