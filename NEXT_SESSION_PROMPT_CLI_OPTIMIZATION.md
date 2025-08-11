# Next Session: Complete Phase B with Error Handling

## Context

We are optimizing Shadowcat's CLI refactor to transform it from a functional CLI tool into a production-ready library. The work is tracked in `plans/cli-refactor-optimization/cli-optimization-tracker.md`.

## Current Status

**Phase B: 92% Complete** (as of 2025-08-11)

### Completed in Phase B:
- ✅ B.1: Implement Builder Patterns (6h) - All builders working with tests
- ✅ B.2: Add Graceful Shutdown (4h) - Full shutdown system
- ✅ B.2.1: Fix Shutdown Integration (4h) - Fixed with real proxy implementations
- ✅ B.3: Create High-Level API (3h) - Clean API with handles, 4 examples
- ✅ B.4: Extract Transport Factory (3h) - Type-safe factory with TransportSpec enum
- ✅ B.6: Add Basic Integration Tests (2h) - 859 total tests, all passing

### Remaining in Phase B:
- ⬜ B.5: Standardize Error Handling (2h) - **LAST TASK IN PHASE B!**

## Working Directory

The work is happening in a git worktree:
```bash
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor
```

This is a separate worktree of the shadowcat submodule on branch `shadowcat-cli-refactor`.

## Test Status

All 859 tests are passing with the new transport factory implementation.

## Next Task: B.5 - Standardize Error Handling (2 hours)

**Goal**: Ensure consistent error handling throughout the library.

**Current Issues**:
1. Mix of `Result<T, ShadowcatError>` and `anyhow::Result<T>`
2. Some errors printed to stderr, others returned
3. Inconsistent error context and messages
4. Some library code still uses `println!` or `eprintln!`

**Implementation Plan**:

### Step 1: Audit (30 min)
```bash
# Find anyhow::Result in library code
rg "anyhow::Result" src/ --glob '!main.rs' --glob '!cli/'

# Find direct output in library
rg "println!|eprintln!" src/ --glob '!main.rs' --glob '!cli/'

# Check error types
rg "Result<.*>" src/ --glob '*.rs' | grep -v ShadowcatError | grep -v "Result<()"
```

### Step 2: Standardize (45 min)
- Ensure all public APIs return `Result<T, ShadowcatError>`
- Add error conversion traits where needed
- Update function signatures

### Step 3: Add Context (30 min)
- Add `.context()` calls for fallible operations
- Make error messages descriptive
- Include relevant data (paths, URLs)

### Step 4: Remove Direct Output (15 min)
- Remove `println!`/`eprintln!` from library
- Use tracing instead

### Step 5: Update Examples (30 min)
- Show proper error handling patterns
- Add error recovery examples

## Success Criteria for B.5

- All public APIs return `Result<T, ShadowcatError>`
- No direct printing in library code
- Consistent error messages with context
- All 859+ tests still passing
- No clippy warnings

## After B.5: Start Phase C!

Once B.5 is complete, Phase B will be 100% done and we can start Phase C (Quality & Testing):

1. **C.1: Comprehensive Documentation** (4h) - Document the new APIs
2. **C.2: Configuration File Support** (3h) - TOML/YAML config
3. **C.3: Improve Error Messages** (2h) - Make errors actionable
4. **C.4: Add Telemetry/Metrics** (4h) - Performance monitoring
5. **C.5: Performance Optimization** (6h) - Profile and optimize

## Commands to Run

```bash
# Navigate to worktree
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Check current status
git status
cargo test --quiet  # Should show 859+ tests passing

# Start B.5 implementation
# Follow the audit steps above

# After each change
cargo test --quiet
cargo clippy --all-targets -- -D warnings
```

## Key Achievements from Last Session

- **Consolidated Transport Factory**: Removed duplicate factory, created single comprehensive implementation
- **Type-Safe API**: Replaced invalid URL schemes (stdio://, sse://) with TransportSpec enum
- **Multiple Creation Methods**: Spec-based, direct methods, auto-detection, and builders
- **Clean Design**: Using Rust's type system instead of string parsing
- **All Tests Passing**: 859 tests, no clippy warnings

## Important Notes

1. **Worktree**: We're in `/Users/kevin/src/tapwire/shadowcat-cli-refactor`
2. **Focus**: B.5 is the LAST task in Phase B - completing it means Phase B is done!
3. **Library vs CLI**: Focus on library code; CLI can keep using anyhow
4. **Backwards Compatibility**: Preserve where possible

## Phase B Completion

When B.5 is done, Phase B will be 100% complete:
- Library will be fully usable without CLI
- Clean, ergonomic APIs with builders
- Proper error handling throughout
- Graceful shutdown support
- Comprehensive transport factory
- Integration tests in place

Ready to finish Phase B and move to Phase C!