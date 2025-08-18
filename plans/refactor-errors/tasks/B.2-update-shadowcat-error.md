# Task B.2: Update ShadowcatError

## Objective

Update the ShadowcatError enum to consume module errors via their new module paths, establishing the pattern of module errors flowing into the top-level error.

## Background

Now that modules re-export their errors as `module::Error`, we need to update ShadowcatError's From implementations to use these module paths instead of direct error names. This reinforces the modular structure.

## Key Questions to Answer

1. Should we change From implementations or leave them as-is?
2. How do we handle the transition period?
3. Do we need to update error messages?
4. Should we improve error context while we're at it?

## Step-by-Step Process

### 1. Analysis Phase (10 min)

Review current ShadowcatError structure:

```bash
cd shadowcat
# Review current From implementations
rg "impl From<.*Error>" src/error.rs
rg "#\[from\]" src/error.rs -B 2
```

### 2. Implementation Phase (30 min)

Update ShadowcatError variants to use module paths:

```rust
// src/error.rs
#[derive(thiserror::Error, Debug)]
pub enum ShadowcatError {
    #[error("Transport error: {0}")]
    Transport(#[from] crate::transport::Error),
    
    #[error("Session error: {0}")]
    Session(#[from] crate::session::Error),
    
    #[error("Storage error: {0}")]
    Storage(#[from] crate::storage::Error),
    
    #[error("Auth error: {0}")]
    Auth(#[from] crate::auth::Error),
    
    #[error("Config error: {0}")]
    Config(#[from] crate::config::Error),
    
    #[error("Interceptor error: {0}")]
    Interceptor(#[from] crate::interceptor::Error),
    
    #[error("Recorder error: {0}")]
    Recorder(#[from] crate::recorder::Error),
    
    #[error("Proxy error: {0}")]
    Proxy(#[from] crate::proxy::Error),
    
    #[error("Reverse proxy error: {0}")]
    ReverseProxy(#[from] crate::proxy::reverse::Error),
    
    // Other variants...
}
```

### 3. Testing Phase (15 min)

Ensure From implementations still work:

```bash
# Compile and test
cargo build
cargo test --lib

# Specifically test error conversions
cargo test error::tests

# Check that error messages are still good
cargo test -- --nocapture 2>&1 | grep -i error
```

### 4. Documentation Phase (5 min)

Update error documentation to reflect new structure.

## Expected Deliverables

### Modified Files
- `src/error.rs` - Updated ShadowcatError with module paths

### Tests
- All error conversion tests passing
- No changes to error behavior

### Documentation
- Updated comments explaining module error flow

## Success Criteria Checklist

- [ ] ShadowcatError uses module::Error paths
- [ ] All From implementations working
- [ ] No compilation errors
- [ ] All tests passing
- [ ] Error messages unchanged
- [ ] Documentation updated

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking From implementations | HIGH | Test thoroughly, use type aliases |
| Circular dependencies | MEDIUM | Module errors can't depend on ShadowcatError |
| Import conflicts | LOW | Use fully qualified paths |

## Duration Estimate

**Total: 1 hour**
- Analysis: 10 minutes
- Implementation: 30 minutes
- Testing: 15 minutes
- Documentation: 5 minutes

## Dependencies

- B.1: Add Module Re-exports (must be complete)

## Integration Points

- **error.rs**: Primary modification target
- **All modules**: Via their Error re-exports
- **Error conversion paths**: Must remain intact

## Performance Considerations

- Zero runtime impact (same as before)
- Compilation might be slightly faster (better locality)

## Notes

- The actual error enum definitions stay in error.rs
- We're only changing the From implementation paths
- This is purely organizational, no functional changes

## Commands Reference

```bash
cd shadowcat

# Backup current error.rs
cp src/error.rs src/error.rs.backup

# Update From implementations (manual edit required)
vim src/error.rs

# Test error conversions
cargo test error

# Test a specific conversion
echo "Testing transport::Error -> ShadowcatError conversion"
cargo test test_transport_error_conversion
```

## Example Implementation

```rust
// src/error.rs
use thiserror::Error;

// Keep error enum definitions here
#[derive(Error, Debug)]
pub enum TransportError {
    // ... variants ...
}

// ... other error enums ...

// Updated ShadowcatError with module paths
#[derive(Error, Debug)]
pub enum ShadowcatError {
    #[error("Transport error: {0}")]
    Transport(#[from] crate::transport::Error),
    
    #[error("Session error: {0}")]
    Session(#[from] crate::session::Error),
    
    // ... other variants ...
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

// Result type for high-level operations
pub type Result<T> = std::result::Result<T, ShadowcatError>;
```

## Follow-up Tasks

After completing this task:
- B.3: Migrate internal usage to module Result types
- B.4: Add deprecation warnings to old aliases

---

**Task Status**: ⬜ Not Started
**Created**: 2025-01-18
**Last Modified**: 2025-01-18
**Author**: Kevin