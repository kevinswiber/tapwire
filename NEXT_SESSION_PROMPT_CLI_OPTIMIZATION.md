# Next Session: Begin Phase C - Quality & Testing

## Context

We are optimizing Shadowcat's CLI refactor to transform it from a functional CLI tool into a production-ready library. The work is tracked in `plans/cli-refactor-optimization/cli-optimization-tracker.md`.

## Current Status

**Phase B: 100% Complete!** 🎉 (as of 2025-08-11)
**Phase C: Ready to Begin**

### Phase B Accomplishments:
- ✅ B.1: Implement Builder Patterns (6h) - All builders working with tests
- ✅ B.2: Add Graceful Shutdown (4h) - Full shutdown system
- ✅ B.2.1: Fix Shutdown Integration (4h) - Fixed with real proxy implementations
- ✅ B.3: Create High-Level API (3h) - Clean API with handles, 4 examples
- ✅ B.4: Extract Transport Factory (3h) - Type-safe factory with TransportSpec enum
- ✅ B.5: Standardize Error Handling (2h) - Domain-specific errors, no anyhow
- ✅ B.6: Add Basic Integration Tests (2h) - 873 total tests, all passing

## Working Directory

The work is happening in a git worktree:
```bash
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor
```

This is a separate worktree of the shadowcat submodule on branch `shadowcat-cli-refactor`.

## Test Status

All 859 tests are passing with the new transport factory implementation.

## Phase C Tasks Overview

Phase C focuses on polish, optimization, and production readiness. Total: 37 hours.

### Priority Order (based on user impact):

1. **C.1: Comprehensive Documentation** (4h) - **START HERE**
   - Document all public APIs with examples
   - Create library usage guide
   - Add module-level documentation
   - Document migration from CLI to library usage

2. **C.2: Configuration File Support** (3h)
   - Add TOML/YAML config file loading
   - Environment variable overrides
   - Config validation and defaults
   - Example config files

3. **C.3: Improve Error Messages** (2h)
   - Make errors actionable with suggestions
   - Add error codes for common issues
   - Improve error display formatting
   - Add troubleshooting guide

4. **C.4: Add Telemetry/Metrics** (4h)
   - OpenTelemetry integration
   - Performance metrics collection
   - Request/response tracing
   - Metrics export options

5. **C.5: Performance Optimization** (6h)
   - Profile critical paths
   - Optimize allocations
   - Improve async performance
   - Benchmark against targets

## Next Task: C.1 - Comprehensive Documentation (4 hours)

**Goal**: Make the library easily adoptable with excellent documentation.

**Deliverables**:
1. Complete API documentation for all public types and functions
2. Module-level documentation explaining architecture
3. Library usage guide with common patterns
4. Migration guide from CLI to library
5. Troubleshooting section

**Implementation Plan**:

### Step 1: API Documentation (1.5h)
```bash
# Find all public items lacking docs
rg "^\s*pub (fn|struct|enum|trait|type)" src/ --glob '!cli/' | grep -v "^\s*///"

# Generate initial docs
cargo doc --open
```

### Step 2: Module Documentation (1h)
- Add module-level docs to lib.rs
- Document each public module's purpose
- Add usage examples in module docs

### Step 3: Usage Guide (1h)
- Create examples/README.md
- Document common patterns
- Show error handling best practices
- Provide complete working examples

### Step 4: Migration Guide (0.5h)
- Document CLI → Library migration
- Show before/after code examples
- List breaking changes and alternatives

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

## Key Achievements from Phase B

- **Builder Patterns**: All major types have ergonomic builders
- **Graceful Shutdown**: Complete shutdown system with Ctrl+C handling
- **High-Level API**: Simple Shadowcat struct with handles for lifecycle management
- **Transport Factory**: Type-safe factory with TransportSpec enum
- **Error Handling**: Standardized to domain-specific errors, no anyhow in library
- **Integration Tests**: 873 tests total, all passing
- **Zero Clippy Warnings**: Full compliance with Rust best practices

## Important Notes

1. **Worktree**: We're in `/Users/kevin/src/tapwire/shadowcat-cli-refactor`
2. **Focus**: B.5 is the LAST task in Phase B - completing it means Phase B is done!
3. **Library vs CLI**: Focus on library code; CLI can keep using anyhow
4. **Backwards Compatibility**: Preserve where possible

## Success Criteria for C.1

- [ ] All public APIs have documentation with examples
- [ ] Every module has clear purpose documentation
- [ ] Usage guide covers 5+ common scenarios
- [ ] Migration guide helps CLI users adopt library
- [ ] `cargo doc` generates clean, navigable docs
- [ ] No `missing_docs` warnings when enabled

## Session Planning

Estimated time: 4 hours

Suggested session structure:
1. Start with C.1 documentation (this session)
2. Next session: C.2 + C.3 (config files + error messages) - 5 hours
3. Following session: C.4 + start C.5 (telemetry + optimization) - 6-8 hours
4. Final session: Complete C.5 + remaining C tasks - 4-6 hours

## Overall Progress

- **Phase A**: 100% Complete ✅
- **Phase B**: 100% Complete ✅
- **Phase C**: 0% Complete (starting now)
- **Total Progress**: 38% (31 of 68 hours)