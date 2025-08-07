# Task 010: Remove Dead Code

## Overview
Remove all unused code identified in the comprehensive review, including unused enums, fields, functions, and test helpers.

## Context
The [comprehensive review](../../reviews/shadowcat-comprehensive-review-2025-08-06.md) identified several pieces of dead code that add confusion and maintenance burden.

## Dead Code Inventory

### 1. TransportEdge Enum
**Location**: `src/transport/mod.rs:61-65`
```rust
pub enum TransportEdge {
    Client,
    Server,
}
```
**Action**: Delete entirely - no references found

### 2. SessionManager cleanup_interval Field  
**Location**: `src/session/manager.rs:41`
```rust
pub struct SessionManager {
    // ...
    cleanup_interval: Duration,  // NEVER USED
    // ...
}
```
**Action**: Remove field and update constructor

### 3. Unused Test Helpers
**Location**: Various test modules
- Files with `#[allow(dead_code)]` annotations
- Mock builders that are never called
- Assertion helpers that aren't used

**Action**: Remove or document why they're kept

### 4. Unused Imports
**Location**: Throughout codebase
```rust
use std::collections::HashSet;  // unused import
```
**Action**: Remove all unused imports

## Step-by-Step Process

### Step 1: Enable Stricter Warnings

Add to `src/lib.rs` and `src/main.rs`:
```rust
#![warn(
    dead_code,
    unused_imports,
    unused_variables,
    unused_mut,
    unreachable_code,
)]
```

### Step 2: Build and Collect Warnings

```bash
# Clean build to see all warnings
cargo clean
cargo build --all-targets 2>&1 | tee build_warnings.txt

# Extract dead code warnings
grep "warning:.*never" build_warnings.txt > dead_code.txt
grep "warning:.*unused" build_warnings.txt >> dead_code.txt
```

### Step 3: Remove TransportEdge Enum

**File**: `src/transport/mod.rs`

```diff
- /// Represents which side of the transport connection
- #[derive(Debug, Clone, Copy, PartialEq, Eq)]
- pub enum TransportEdge {
-     Client,
-     Server,
- }
```

### Step 4: Remove cleanup_interval Field

**File**: `src/session/manager.rs`

```diff
 pub struct SessionManager {
     store: Arc<SessionStore>,
     active_sessions: Arc<RwLock<HashMap<SessionId, SessionHandle>>>,
-    cleanup_interval: Duration,
     max_sessions: usize,
 }

 impl SessionManager {
     pub fn new(config: SessionConfig) -> Self {
         Self {
             store: Arc::new(SessionStore::new()),
             active_sessions: Arc::new(RwLock::new(HashMap::new())),
-            cleanup_interval: config.cleanup_interval,
             max_sessions: config.max_sessions,
         }
     }
```

Also update `SessionConfig`:
```diff
 pub struct SessionConfig {
     pub max_sessions: usize,
-    pub cleanup_interval: Duration,
     pub session_timeout: Duration,
 }
```

### Step 5: Clean Up Test Code

**Approach**: Keep test utilities that might be useful, remove truly dead code

```rust
// Instead of removing, consider:
#[cfg(test)]
mod test_helpers {
    #![allow(dead_code)]  // Remove this
    
    // If truly unused, delete:
    pub fn unused_helper() { ... }  // DELETE
    
    // If might be useful later, document:
    /// Helper for future integration tests
    /// TODO: Use in upcoming test scenarios
    pub fn future_helper() { ... }  // KEEP WITH COMMENT
}
```

### Step 6: Remove Unused Imports

```bash
# Use cargo fix to automatically remove
cargo fix --allow-dirty --allow-staged

# Or manually with clippy
cargo clippy --fix -- -W clippy::unused_imports
```

### Step 7: Check for Unused Dependencies

```bash
# Install cargo-udeps
cargo install cargo-udeps

# Check for unused dependencies
cargo +nightly udeps

# Remove from Cargo.toml if truly unused
```

### Step 8: Remove Commented Code

```bash
# Find commented code blocks
rg "^\s*//.*fn " --type rust  # Commented functions
rg "^\s*/\*" --type rust       # Block comments

# Review and remove if obsolete
```

## Special Cases

### Keep These Despite Warnings

1. **Error Variants** - May be unused now but needed for completeness:
```rust
#[allow(dead_code)]  // Will be used when feature X is implemented
pub enum Error {
    NotImplemented,  // Keep for future use
}
```

2. **Public API** - Unused internally but part of public interface:
```rust
pub fn public_utility() {  // Keep even if unused internally
    // ...
}
```

3. **FFI Functions** - Called from external code:
```rust
#[no_mangle]
extern "C" fn ffi_function() {  // Keep
    // ...
}
```

## Validation

### Before Removal
```bash
# Count warnings
cargo build 2>&1 | grep -c "warning:"

# Save list of symbols
nm target/debug/shadowcat > symbols_before.txt
```

### After Removal
```bash
# Should see fewer warnings
cargo build 2>&1 | grep -c "warning:"  # Should be much lower

# Ensure no build errors
cargo build --release

# Run all tests
cargo test --all

# Check binary size reduction
ls -lh target/release/shadowcat  # Should be smaller
```

## Automated Tools

### Use cargo-machete for unused dependencies
```bash
cargo install cargo-machete
cargo machete
```

### Use cargo-bloat to find large unused sections
```bash
cargo install cargo-bloat
cargo bloat --release --crates
```

## Documentation

Update docs after removal:
```rust
// If removing a public item, document in CHANGELOG
## Breaking Changes
- Removed `TransportEdge` enum (was unused)
- Removed `cleanup_interval` from `SessionConfig`
```

## Common Pitfalls

1. **Don't remove error variants** that might be needed
2. **Keep example code** in docs even if "unused"
3. **Preserve benchmarks** even if not run regularly
4. **Keep macro-generated code** that appears unused
5. **Don't remove trait methods** required by trait definition

## Success Criteria

- [ ] Zero dead code warnings (or documented exceptions)
- [ ] All unused imports removed
- [ ] Unused dependencies removed from Cargo.toml
- [ ] Binary size reduced by >5%
- [ ] All tests still pass
- [ ] Documentation updated for removed items
- [ ] No functional regression

## Metrics

### Before
- Dead code warnings: ~50
- Binary size: X MB
- Compile time: Y seconds
- Dependencies: Z

### After (Target)
- Dead code warnings: <5 (documented)
- Binary size: <0.95X MB
- Compile time: <Y seconds
- Dependencies: <Z

## Follow-up Tasks

1. Add CI check for dead code:
```yaml
# .github/workflows/ci.yml
- name: Check for dead code
  run: |
    cargo build 2>&1 | grep "warning:.*never" && exit 1 || exit 0
```

2. Regular dead code audits (monthly)
3. Document why certain "dead" code is kept