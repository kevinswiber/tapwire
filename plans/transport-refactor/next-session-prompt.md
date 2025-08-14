# Next Session: Builder Pattern Review and Subprocess Tests

## Context
Session 10 successfully fixed two critical security vulnerabilities (memory exhaustion and constructor panics) but uncovered an architectural inconsistency in the builder pattern. Additionally, subprocess tests are still failing and need attention.

## Previous Session (Session 10) Accomplishments
- ✅ Implemented message size limits in all directional transports
- ✅ Added `with_max_message_size()` builder method to all transports
- ✅ Fixed constructor panics - SubprocessOutgoing::new() now returns Result
- ✅ Updated all call sites to handle Result type properly
- ✅ Tests verify size limits and constructor validation work correctly
- ✅ Two critical security vulnerabilities fixed

## Architectural Concern: Builder Pattern Consistency
Current pattern creates inconsistency:
```rust
// Constructor returns Result
let transport = SubprocessOutgoing::new(cmd)?;

// But builder method returns Self
let transport = SubprocessOutgoing::new(cmd)?.with_max_message_size(1024);
```

Should builder methods also return Result for consistency?
```rust
// Option 1: Builder methods return Result
let transport = SubprocessOutgoing::new(cmd)?
    .with_max_message_size(1024)?
    .with_timeout(5000)?;

// Option 2: Separate builder pattern
let transport = SubprocessOutgoing::builder()
    .command(cmd)
    .max_message_size(1024)
    .timeout(5000)
    .build()?;  // Validation happens here
```

## Tasks (4-5 hours)

### 1. Evaluate and Fix Builder Pattern (2h)
- Review all `with_*` methods in directional transports
- Decide on consistent pattern:
  - Option 1: Make builder methods return Result
  - Option 2: Create separate Builder structs
  - Option 3: Keep as-is but document the rationale
- Implement chosen pattern consistently
- Update tests to match new pattern

### 2. Verify Resource Cleanup in Raw Transports (1h)
- Check Drop implementations in all raw transports
- Verify subprocess termination on drop
- Check for any spawned tasks that aren't properly joined
- Test files: `src/transport/raw/tests/subprocess.rs`

### 3. Fix Failing Subprocess Tests (1-2h)
Fix the 5 failing tests in `src/transport/raw/tests/subprocess.rs`:
- `test_subprocess_stdin_stdout_communication`
- `test_subprocess_working_directory`
- `test_subprocess_environment_variables`
- `test_subprocess_handle_crash`
- `test_subprocess_connect_after_drop`

These tests reveal actual bugs in subprocess handling that need fixing.

## Success Criteria
- [ ] Builder pattern is consistent across all transports
- [ ] All subprocess tests pass (5 currently failing)
- [ ] Resource cleanup verified with no leaks
- [ ] Documentation updated to explain builder pattern choice
- [ ] Zero clippy warnings maintained
- [ ] All 788+ unit tests passing

## References
- Tracker: `plans/transport-refactor/transport-refactor-tracker.md` (Phase 10)
- Test coverage: `plans/transport-refactor/analysis/test-coverage-summary.md`
- Current failing tests: Run `cargo test --lib transport::raw::tests::subprocess`

## Notes
- Builder pattern decision impacts API stability - choose carefully
- Subprocess tests may reveal deeper issues in process management
- Consider if we need a formal ADR (Architecture Decision Record) for the builder pattern