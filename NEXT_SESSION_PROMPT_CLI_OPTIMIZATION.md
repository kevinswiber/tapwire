# Next Session: Shadowcat CLI Optimization - Phase C Continuation

## Context
We're continuing the Shadowcat CLI optimization in the **shadowcat-cli-refactor** worktree. 

**IMPORTANT**: Work in the `/Users/kevin/src/tapwire/shadowcat-cli-refactor` directory, NOT the main shadowcat directory.

**Current Progress: 60% Complete** (49 of 73 hours completed)

## What Was Just Completed: Phase C.3 and C.4

### C.3 - Improve Error Messages (2 hours) ✅
- Created comprehensive error formatter with context and suggestions
- Enhanced transport error messages (stdio and HTTP)
- Integrated enhanced formatting into CLI
- Fixed all clippy warnings

### C.4 - Telemetry and Metrics (4 hours) ✅
- Implemented OpenTelemetry tracing with OTLP export
- Added Prometheus metrics collection
- Created telemetry demo example
- Zero overhead when disabled

## Next Tasks to Complete

### Priority 1: C.5 - Performance Optimization (6 hours) [High Impact]
Optimize for production loads:
- Profile with flamegraph
- Reduce allocations in hot paths
- Optimize buffer sizes
- Implement connection pooling
- Target: < 5% latency overhead

### Priority 2: C.6 - Extensive Test Coverage (6 hours)
Achieve 70%+ test coverage:
- Analyze current coverage with tarpaulin
- Add integration tests for error paths
- Test shutdown scenarios comprehensively
- Add property-based tests for builders
- Test configuration loading edge cases

### Priority 3: C.7 - CLI Shell Completions (2 hours)
Add shell completion support:
- Implement completions for bash, zsh, fish
- Add installation instructions
- Test completion scenarios

## Current Status

### Completed Phases
- **Phase A**: Critical Fixes (100% - 7 hours) ✅
- **Phase B**: Library Readiness (100% - 24 hours) ✅
- **Phase B.7**: Code Review Fixes (100% - 5 hours) ✅
- **Phase C**: Quality & Testing (35% - 13 of 37 hours)
  - ✅ C.1: Comprehensive Documentation
  - ✅ C.2: Configuration File Support
  - ✅ C.3: Improve Error Messages (just completed)
  - ✅ C.4: Telemetry/Metrics
  - ✅ C.8: Example Programs

### Remaining Work (24 hours)
- C.5: Performance Optimization (6h)
- C.6: Extensive Test Coverage (6h)
- C.7: CLI Shell Completions (2h)
- C.9: Connection Pooling (3h)
- C.10: Load Testing (2h)
- C.11: Release Preparation (2h)
- Unallocated: 3h

## Tracker Location
Review the full tracker at: `plans/cli-refactor-optimization/cli-optimization-tracker.md`

## Commands to Run at Start
```bash
# Switch to the correct worktree
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Verify we're on the right branch
git status

# Check what's changed
git diff

# Run tests to ensure everything is working
cargo test --quiet

# Check for any remaining clippy issues (should be none)
cargo clippy --all-targets -- -D warnings
```

## Key Files to Reference
- `plans/cli-refactor-optimization/cli-optimization-tracker.md` - Main tracker
- `plans/cli-refactor-optimization/tasks/C.5-performance.md` - Next task details
- `src/cli/error_formatter.rs` - Just completed error formatter
- `examples/` - Example programs to test with

## Important Notes
1. All work should be done in the shadowcat-cli-refactor worktree
2. The branch has been rebased onto main and is up to date
3. All clippy warnings have been fixed
4. Focus on performance optimization next as it's critical for production readiness
5. Consider using the rust-code-reviewer agent for complex performance optimizations

## Success Criteria for Next Session
- [ ] Complete C.5 Performance Optimization
- [ ] Achieve < 5% latency overhead in benchmarks
- [ ] Begin C.6 Test Coverage if time permits
- [ ] Update tracker with progress
- [ ] Keep all tests passing and clippy clean