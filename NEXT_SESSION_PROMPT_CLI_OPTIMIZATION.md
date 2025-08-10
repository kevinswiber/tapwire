# Next Session: Shadowcat CLI Optimization - Phase A Implementation

## Context
We've completed the planning phase for the CLI refactor optimization. A comprehensive project plan has been created in `plans/cli-refactor-optimization/` with detailed task files for all three phases. Now it's time to begin implementation.

## Current State
- **Planning**: ✅ Complete
  - Main tracker: `plans/cli-refactor-optimization/cli-optimization-tracker.md`
  - Prioritization analysis: `plans/cli-refactor-optimization/analysis/prioritization.md`
  - Task files: Created for Phase A (critical fixes), Phase B (library readiness), and Phase C (quality & testing)
- **Branch**: `shadowcat-cli-refactor` (git worktree at `/Users/kevin/src/tapwire/shadowcat-cli-refactor`)
- **Implementation**: Not started

## Your Task: Implement Phase A (Critical Fixes)

### Overview
Phase A removes immediate blockers to library usage. These are simple but high-impact changes that can be completed in 1-2 days (7 hours total).

### Tasks to Complete (in order)

#### 1. Task A.1: Make CLI Module Private (2 hours)
**File**: `plans/cli-refactor-optimization/tasks/A.1-make-cli-private.md`

Start here:
```bash
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor
```

Key steps:
- Change `pub mod cli` to `pub(crate) mod cli` in `src/lib.rs`
- Optionally add feature flag for CLI
- Verify library builds without CLI
- Ensure CLI binary still works

#### 2. Task A.2: Remove Exit() Calls (2 hours)
**File**: `plans/cli-refactor-optimization/tasks/A.2-remove-exit-calls.md`

Key steps:
- Find all `exit()` calls in CLI modules
- Replace with proper `Result` error propagation
- Update `main()` to return `Result`
- Fix any `unwrap()` calls while you're at it

#### 3. Task A.3: Fix Configuration Duplication (3 hours)
**File**: `plans/cli-refactor-optimization/tasks/A.3-fix-config-duplication.md`

Key steps:
- Extract `ProxyConfig` from `src/cli/common.rs` to library
- Create `src/config/` module with proper structure
- Add basic builder pattern for configuration
- Update CLI to use library configuration

### Success Criteria for This Session
- [ ] All three Phase A tasks completed
- [ ] Library builds without CLI dependencies: `cargo build --lib --no-default-features`
- [ ] CLI still works correctly: `cargo run -- forward stdio -- echo test`
- [ ] No direct `exit()` calls remain in codebase
- [ ] Single `ProxyConfig` definition in library (no duplication)
- [ ] All tests pass: `cargo test --all-features`
- [ ] No clippy warnings: `cargo clippy --all-targets -- -D warnings`

### Verification Commands
```bash
# After A.1: Verify library independence
cargo build --lib --no-default-features
cargo doc --no-deps  # CLI module should not appear in docs

# After A.2: Verify error handling
rg "exit\(" --type rust src/  # Should return nothing
cargo run -- forward stdio  # Should show nice error, not panic

# After A.3: Verify configuration
rg "struct ProxyConfig" --type rust  # Should only appear once in library
cargo test config

# Final verification
cargo test --all-features
cargo clippy --all-targets -- -D warnings
```

### Important Notes
1. **Work in the shadowcat-cli-refactor worktree**, not the main shadowcat submodule
2. Each task has detailed step-by-step instructions in its task file
3. Update the tracker status as you complete each task
4. If you encounter blockers, document them in the task file
5. These changes lay the foundation for all future improvements

### What Comes Next (Phase B Preview)
After Phase A is complete, Phase B will:
- Implement comprehensive builder patterns (B.1)
- Add graceful shutdown handling (B.2)
- Create a high-level library facade (B.3)
- Extract transport factories (B.4)
- Standardize error handling patterns (B.5)
- Add integration tests (B.6)

But focus on Phase A first - it's the critical path to making Shadowcat usable as a library.

## Commands to Get Started
```bash
# Navigate to the refactor worktree
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Start with task A.1
cat /Users/kevin/src/tapwire/plans/cli-refactor-optimization/tasks/A.1-make-cli-private.md

# Check current state
rg "pub mod cli" src/lib.rs
```

## Estimated Duration
- Phase A implementation: 7 hours (can be split across sessions if needed)
- Recommend completing all three tasks in sequence for best results

Good luck with the implementation! The detailed task files have everything you need to succeed.