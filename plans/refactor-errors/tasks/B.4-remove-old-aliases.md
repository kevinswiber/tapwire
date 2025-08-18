# Task B.4: Remove Old Aliases

## Objective

Remove all old Result type aliases and direct error exports from error.rs, completing the migration to module-local error patterns.

## Background

With all internal usage migrated to module-local patterns, we can now remove the old centralized Result aliases entirely. Since Shadowcat hasn't been released yet, we don't need deprecation warnings or backward compatibility.

## Key Questions to Answer

1. Are all internal usages migrated?
2. Do any tests still use old patterns?
3. Are there any examples using old patterns?
4. Can we simplify error.rs further?

## Step-by-Step Process

### 1. Verification Phase (15 min)

Ensure no remaining usage of old patterns:

```bash
cd shadowcat

# Find any remaining old Result aliases
rg "TransportResult|SessionResult|StorageResult|AuthResult|ConfigResult" --type rust
rg "InterceptResult|RecorderResult|ProxyResult|ReverseProxyResult" --type rust

# Check for direct error imports from error module
rg "use crate::error::\w+Error" --type rust -g '!error.rs'
```

### 2. Removal Phase (30 min)

Remove from error.rs:

#### 2.1 Remove Result type aliases
```rust
// DELETE these lines from error.rs:
pub type TransportResult<T> = std::result::Result<T, TransportError>;
pub type SessionResult<T> = std::result::Result<T, SessionError>;
pub type StorageResult<T> = std::result::Result<T, StorageError>;
pub type AuthResult<T> = std::result::Result<T, AuthError>;
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;
pub type InterceptResult<T> = std::result::Result<T, InterceptError>;
pub type RecorderResult<T> = std::result::Result<T, RecorderError>;
pub type ProxyResult<T> = std::result::Result<T, ProxyError>;
pub type ReverseProxyResult<T> = std::result::Result<T, ReverseProxyError>;
```

#### 2.2 Clean up any direct error exports if present
Remove any `pub use` statements that directly export individual errors (keep the enum definitions).

### 3. Testing Phase (10 min)

Ensure everything still compiles:

```bash
# Full build
cargo build

# Run all tests
cargo test

# Check clippy
cargo clippy --all-targets -- -D warnings

# Build documentation
cargo doc --no-deps
```

### 4. Cleanup Phase (5 min)

Clean up error.rs structure:
- Remove any unused imports
- Organize remaining code
- Add clear section comments

## Expected Deliverables

### Modified Files
- `src/error.rs` - Removed all Result aliases and cleaned up

### Tests
- All existing tests still passing
- No compilation errors

### Documentation
- error.rs is cleaner and more focused

## Success Criteria Checklist

- [ ] All old Result aliases removed
- [ ] No remaining usage of old patterns
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Documentation builds successfully
- [ ] error.rs is clean and organized

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Missed usage site | HIGH | Compiler will catch, thorough search first |
| Tests break | MEDIUM | Fix as found |
| Documentation references | LOW | Update if found |

## Duration Estimate

**Total: 1 hour**
- Verification: 15 minutes
- Removal: 30 minutes
- Testing: 10 minutes
- Cleanup: 5 minutes

## Dependencies

- B.3: All internal usage migrated

## Integration Points

- **error.rs**: Primary modification target
- **All modules**: Should already be using module patterns

## Performance Considerations

- Might slightly improve compilation (less to parse)
- No runtime impact

## Notes

- This is a clean break - no backward compatibility needed
- Make sure to remove ALL aliases, not just some
- Keep the error enum definitions intact

## Commands Reference

```bash
cd shadowcat

# Verify no remaining usage
rg "\w+Result<" src/error.rs
rg "pub type.*Result" src/error.rs

# After removal, verify clean build
cargo clean
cargo build
cargo test

# Check for any missed references
cargo build 2>&1 | grep -i "cannot find"
```

## Example Final State

```rust
// src/error.rs after cleanup
use thiserror::Error;

// Error enum definitions (kept)
#[derive(Error, Debug)]
pub enum TransportError {
    // ... variants ...
}

#[derive(Error, Debug)]
pub enum SessionError {
    // ... variants ...
}

// ... other error enums ...

// Top-level error with From implementations
#[derive(Error, Debug)]
pub enum ShadowcatError {
    #[error("Transport error: {0}")]
    Transport(#[from] crate::transport::Error),
    
    #[error("Session error: {0}")]
    Session(#[from] crate::session::Error),
    
    // ... other variants ...
}

// Only keep the high-level Result type
pub type Result<T> = std::result::Result<T, ShadowcatError>;
```

## Follow-up Tasks

After completing this task:
- C.1: Update test suite
- C.2: Update documentation
- Consider if error enums can be moved to their modules

---

**Task Status**: ⬜ Not Started
**Created**: 2025-01-18
**Last Modified**: 2025-01-18
**Author**: Kevin