# Migration Roadmap

## Overview
This document outlines the step-by-step process for refactoring main.rs into modular CLI components. The migration is designed to be incremental, testable, and reversible at each stage.

## Migration Phases

### Phase 1: Foundation Setup (Day 1, 3 hours)
**Goal**: Establish shared infrastructure without breaking existing code

#### Step 1.1: Create Common Module (1 hour)
- Create `src/cli/common.rs`
- Implement ProxyConfig with all methods
- Add JSON conversion utilities
- Add rate limiter factory
- Add session manager factory

#### Step 1.2: Add Common Tests (30 min)
- Unit tests for ProxyConfig
- Tests for utility functions
- Mock helpers for testing

#### Step 1.3: Update main.rs to Use Common (30 min)
- Import common module
- Replace ProxyConfig with common::ProxyConfig
- Move utility functions to common
- Verify all tests pass

#### Step 1.4: Integration Testing (1 hour)
- Run all existing CLI commands
- Verify identical behavior
- Create baseline test outputs

**Checkpoint**: All commands work with common module

### Phase 2: Simple Command Extraction (Day 2, 3 hours)
**Goal**: Extract simpler commands first to establish patterns

#### Step 2.1: Extract Replay Command (1 hour)
- Create `src/cli/replay.rs`
- Move `run_replay_server` and `handle_replay_request`
- Move `messages_match` helper
- Update main.rs to call replay module
- Test replay functionality

#### Step 2.2: Extract Record Commands (1.5 hours)
- Create `src/cli/record.rs`
- Move `run_stdio_recording` and `run_http_recording`
- Move `handle_recording_request`
- Update main.rs dispatch
- Test both stdio and HTTP recording

#### Step 2.3: Create Handlers Module (30 min)
- Create `src/cli/handlers.rs`
- Extract shared HTTP handlers
- Move error response utilities
- Update modules to use handlers

**Checkpoint**: Record and replay commands fully modularized

### Phase 3: Complex Command Extraction (Day 3, 3 hours)
**Goal**: Extract the most complex commands with shared logic

#### Step 3.1: Extract Forward Commands (1.5 hours)
- Create `src/cli/forward.rs`
- Move `run_stdio_forward` and `run_http_forward_proxy`
- Handle command spawning logic
- Update main.rs dispatch
- Extensive testing of both transports

#### Step 3.2: Extract Reverse Command (1 hour)
- Create `src/cli/reverse.rs`
- Move `run_reverse_proxy`
- Integrate with ReverseProxyServer
- Update main.rs dispatch
- Test reverse proxy functionality

#### Step 3.3: Update Module Exports (30 min)
- Update `src/cli/mod.rs` with all new modules
- Clean up public API exports
- Ensure consistent naming

**Checkpoint**: All commands extracted to modules

### Phase 4: Cleanup and Optimization (Day 4, 3 hours)
**Goal**: Final cleanup and optimization

#### Step 4.1: Minimize main.rs (1 hour)
- Remove all extracted functions
- Keep only CLI parsing and dispatch
- Clean up imports
- Target < 200 lines

#### Step 4.2: Comprehensive Testing (1 hour)
- Run full test suite
- Integration tests for all commands
- Performance benchmarks
- Binary size comparison

#### Step 4.3: Documentation Update (1 hour)
- Update CLAUDE.md with new structure
- Update README if needed
- Create migration notes
- Document new module APIs

**Checkpoint**: Refactoring complete

## Implementation Order Rationale

### Why This Order?

1. **Common First**: Establishes shared foundation
2. **Replay Before Record**: Simpler, fewer dependencies
3. **Record Before Forward**: Less complex transport handling
4. **Forward Before Reverse**: More complex but standalone
5. **Reverse Last**: Uses most infrastructure

### Dependency Graph
```
common.rs (no deps)
    ↓
handlers.rs (uses common)
    ↓
replay.rs (uses common, handlers)
    ↓
record.rs (uses common, handlers)
    ↓
forward.rs (uses common, handlers)
    ↓
reverse.rs (uses common, handlers)
```

## Git Strategy

### Branch Structure
```
main
  ↓
cli-refactor (feature branch)
  ├── cli-refactor-common
  ├── cli-refactor-replay
  ├── cli-refactor-record
  ├── cli-refactor-forward
  ├── cli-refactor-reverse
  └── cli-refactor-cleanup
```

### Commit Strategy
- One commit per extraction step
- Descriptive commit messages
- Include tests in same commit
- Squash before final merge

### Example Commits
```
feat(cli): create common module with shared configuration
feat(cli): extract replay command to dedicated module
feat(cli): extract record commands to dedicated module
feat(cli): extract forward proxy commands to module
feat(cli): extract reverse proxy command to module
refactor(cli): minimize main.rs to dispatch-only
```

## Testing Checkpoints

### After Each Phase
1. **Functional Tests**: All commands work identically
2. **Unit Tests**: Module-specific tests pass
3. **Integration Tests**: End-to-end scenarios work
4. **Performance Tests**: No regression in performance
5. **Binary Size**: Minimal change in binary size

### Test Commands
```bash
# After each extraction
cargo test --all
cargo clippy -- -D warnings
cargo fmt -- --check

# Integration tests
./test_all_commands.sh  # Create this script

# Performance check
cargo bench

# Binary size check
cargo build --release
ls -lh target/release/shadowcat
```

## Risk Mitigation

### Rollback Points
Each phase is a safe rollback point. If issues arise:
1. Git revert to previous checkpoint
2. Fix issues in isolation
3. Re-attempt migration

### Gradual Rollout
- Keep old functions with deprecation warnings initially
- Can run both old and new code paths
- Remove old code after validation period

### Feature Flags (if needed)
```rust
#[cfg(feature = "new-cli")]
mod cli;

#[cfg(not(feature = "new-cli"))]
mod old_cli;
```

## Success Metrics

### Quantitative
- ✅ main.rs < 200 lines (from 1294)
- ✅ Each module < 250 lines
- ✅ No performance regression
- ✅ Binary size within 5% of original
- ✅ All tests passing

### Qualitative
- ✅ Cleaner code organization
- ✅ Easier to test
- ✅ Better maintainability
- ✅ Clear module boundaries
- ✅ No breaking changes

## Timeline

### Estimated Schedule
- **Day 1**: Phase 1 (Foundation) - 3 hours
- **Day 2**: Phase 2 (Simple Commands) - 3 hours
- **Day 3**: Phase 3 (Complex Commands) - 3 hours
- **Day 4**: Phase 4 (Cleanup) - 3 hours
- **Total**: 12 hours over 4 days

### Buffer Time
- Add 20% buffer for unexpected issues
- Total with buffer: 14-15 hours

## Next Steps

1. Create feature branch
2. Start with Phase 1.1 (common module)
3. Follow roadmap strictly
4. Document any deviations
5. Update tracker after each phase