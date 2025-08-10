# Next Session: Shadowcat CLI Optimization - Phase B Implementation

## Context
Phase A of the CLI refactor optimization has been successfully completed! All critical fixes are done:
- ✅ **A.1**: CLI module is now hidden from library API (`#[doc(hidden)]`)
- ✅ **A.2**: All `exit()` calls removed, proper error propagation with Result types
- ✅ **A.3**: Configuration centralized in `src/config/proxy.rs` with builder pattern

The foundation is now solid for making Shadowcat a production-ready library.

## Current State
- **Branch**: `shadowcat-cli-refactor` (git worktree at `/Users/kevin/src/tapwire/shadowcat-cli-refactor`)
- **Planning**: Complete in `plans/cli-refactor-optimization/`
  - Main tracker: `plans/cli-refactor-optimization/cli-optimization-tracker.md`
  - Task files: All Phase B tasks ready in `plans/cli-refactor-optimization/tasks/`
- **Code State**: 
  - Library builds independently: `cargo build --lib --no-default-features` ✅
  - All tests passing (756 tests) ✅
  - No clippy warnings ✅
  - CLI works correctly with proper error handling ✅

## Your Task: Implement Phase B (Library Readiness)

### Overview
Phase B creates ergonomic library APIs and core functionality. This phase transforms Shadowcat from a CLI tool into a proper library that can be embedded in other applications.

### Tasks to Complete (20 hours total)

#### 1. Task B.1: Implement Builder Patterns (6 hours)
**File**: `plans/cli-refactor-optimization/tasks/B.1-builder-patterns.md`

Create comprehensive builders for:
- Transport configuration (StdioBuilder, HttpBuilder, SSEBuilder)
- Proxy setup (ForwardProxyBuilder, ReverseProxyBuilder)
- Session management configuration
- Interceptor chain configuration

#### 2. Task B.2: Add Graceful Shutdown (4 hours)
**File**: `plans/cli-refactor-optimization/tasks/B.2-graceful-shutdown.md`

Implement proper shutdown handling:
- Graceful connection draining
- Session cleanup
- Resource deallocation
- Cancellation tokens throughout

#### 3. Task B.3: Create Library Facade (3 hours)
**File**: `plans/cli-refactor-optimization/tasks/B.3-library-facade.md`

Design high-level API for common use cases:
- `Shadowcat::forward_proxy()` convenience method
- `Shadowcat::reverse_proxy()` convenience method
- Simple one-liner setup for basic scenarios

#### 4. Task B.4: Extract Transport Factory (3 hours)
**File**: `plans/cli-refactor-optimization/tasks/B.4-transport-factory.md`

Create factory pattern for transport creation:
- Unified transport creation interface
- Runtime transport selection
- Configuration-driven transport setup

#### 5. Task B.5: Standardize Error Handling (2 hours)
**File**: `plans/cli-refactor-optimization/tasks/B.5-error-handling.md`

Improve error handling patterns:
- Consistent error types across modules
- Better error context and chaining
- User-friendly error messages

#### 6. Task B.6: Add Basic Integration Tests (2 hours)
**File**: `plans/cli-refactor-optimization/tasks/B.6-integration-tests.md`

Create integration tests for library usage:
- Test builder patterns
- Test graceful shutdown
- Test library facade methods
- Test error scenarios

### Success Criteria for Phase B
- [ ] All builders implemented and tested
- [ ] Graceful shutdown works correctly
- [ ] Library facade provides simple API
- [ ] Transport factory eliminates duplication
- [ ] Error handling is consistent and helpful
- [ ] Integration tests demonstrate library usage
- [ ] Documentation shows how to use as library
- [ ] No breaking changes to CLI functionality

### Important Implementation Notes

1. **Current Architecture Insights**:
   - ProxyConfig is now in `src/config/proxy.rs` with builder
   - CLI modules are hidden but still functional
   - Error propagation uses Result types throughout
   - Main.rs handles exit codes properly

2. **Key Files Modified in Phase A**:
   - `src/lib.rs`: CLI module marked as `#[doc(hidden)]`
   - `src/config/proxy.rs`: New centralized configuration
   - `src/cli/common.rs`: Uses library ProxyConfig
   - `src/main.rs`: Proper error handling with exit codes

3. **Testing Approach**:
   - Use existing test infrastructure
   - Add new integration tests in `tests/` directory
   - Ensure CLI tests still pass after changes

### Commands to Get Started
```bash
# Navigate to the refactor worktree
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Verify current state
cargo test --all-features --quiet
cargo clippy --all-targets -- -D warnings

# Start with task B.1
cat /Users/kevin/src/tapwire/plans/cli-refactor-optimization/tasks/B.1-builder-patterns.md

# Check current builder implementation in ProxyConfig
grep -n "ProxyConfigBuilder" src/config/proxy.rs
```

### Development Strategy

1. **Start with B.1 (Builder Patterns)**: This is the foundation for all other improvements
2. **Then B.2 (Graceful Shutdown)**: Critical for production usage
3. **B.3-B.4 can be done in parallel**: Both are independent improvements
4. **B.5 before B.6**: Standardize errors before writing tests
5. **B.6 validates everything**: Integration tests prove the library works

### What NOT to Do
- Don't worry about backward compatibility - Shadowcat hasn't been released
- Don't over-engineer - focus on practical, usable APIs
- Don't break existing CLI functionality - it should still work
- Don't add dependencies without clear benefit

## Estimated Duration
Phase B should take approximately 20 hours of focused work. It can be split across multiple sessions:
- Session 1: B.1 (Builder Patterns) - 6 hours
- Session 2: B.2 (Graceful Shutdown) + B.3 (Library Facade) - 7 hours  
- Session 3: B.4 (Transport Factory) + B.5 (Error Handling) + B.6 (Tests) - 7 hours

## Phase C Preview
After Phase B is complete, Phase C will focus on:
- Comprehensive documentation (C.1)
- Configuration file support (C.2)
- Improved error messages (C.3)
- Telemetry/metrics (C.4)
- Performance optimization (C.5)
- Extensive test coverage (C.6)
- CLI shell completions (C.7)

But focus on Phase B first - it's what makes Shadowcat usable as a library!

Good luck with Phase B implementation!