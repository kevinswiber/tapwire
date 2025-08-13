# Task A.1: Make CLI Module Private

## Objective
Remove the CLI module from Shadowcat's public API to prevent library users from accessing CLI-specific functionality and to maintain clean module boundaries.

## Background
Currently, `src/lib.rs` exposes the CLI module publicly:
```rust
pub mod cli;  // This pollutes the library API
```

This means library users see CLI-specific types and functions that should be internal implementation details. This violates the principle of separation of concerns and makes it impossible to change CLI internals without potentially breaking library users.

## Key Questions to Answer
1. What depends on the public CLI module currently?
2. Should we use feature flags or make it completely private?
3. How do we ensure the CLI binary still works after making the module private?

## Step-by-Step Process

### 1. Analyze Current Dependencies
```bash
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor
rg "use.*cli::" --type rust
rg "shadowcat::cli" --type rust
```

### 2. Update lib.rs
Change the module visibility:
```rust
// src/lib.rs
#[cfg(feature = "cli")]
pub(crate) mod cli;  // Now private to the crate

// Or if no feature flag needed:
pub(crate) mod cli;
```

### 3. Update Cargo.toml
Add feature flag if using conditional compilation:
```toml
[features]
default = []
cli = ["clap", "directories"]

[[bin]]
name = "shadowcat"
required-features = ["cli"]
```

### 4. Update main.rs
Ensure main.rs can still access the CLI module:
```rust
// src/main.rs
// The cli module is now pub(crate) so main.rs can access it
use shadowcat::cli::{Cli, Commands};  // This might need adjustment
```

### 5. Verify Library Build
```bash
# Library should build without CLI
cargo build --lib --no-default-features

# CLI should still work
cargo build --features cli
cargo run --features cli -- forward stdio -- echo test
```

### 6. Run Tests
```bash
cargo test --all-features
cargo clippy --all-targets -- -D warnings
```

## Expected Deliverables

### Modified Files
- `shadowcat/src/lib.rs` - CLI module now private
- `shadowcat/Cargo.toml` - Feature flags added (if needed)
- `shadowcat/src/main.rs` - Updated imports (if needed)

### Verification Commands
```bash
# These should all succeed:
cargo build --lib --no-default-features  # Library without CLI
cargo build --features cli                # With CLI
cargo doc --no-deps                       # Check public API docs
```

## Success Criteria Checklist
- [ ] CLI module is not visible in library documentation
- [ ] Library builds without CLI dependencies
- [ ] CLI binary still works correctly
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Public API documentation shows only library types

## Risk Assessment
- **Risk**: Breaking existing code that depends on public CLI
  - **Mitigation**: Search for any external usage first
  - **Mitigation**: Use deprecation warnings if needed

- **Risk**: Main.rs can't access private CLI module
  - **Mitigation**: Use `pub(crate)` visibility
  - **Mitigation**: Restructure if needed

## Duration Estimate
**2 hours**
- 30 min: Analysis and planning
- 45 min: Implementation
- 30 min: Testing and verification
- 15 min: Documentation updates

## Dependencies
- None (this is the first task)

## Notes
- This is the simplest but most impactful change
- Opens the door for all other library improvements
- Consider whether to use feature flags or just make private
- If using features, document in README

## Commands Reference
```bash
# Navigate to the worktree
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Check current module visibility
grep "pub mod cli" src/lib.rs

# After changes, verify library API
cargo doc --no-deps --open

# Test both configurations
cargo test --no-default-features  # Library only
cargo test --all-features         # Everything
```