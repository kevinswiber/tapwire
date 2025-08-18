# Task B.1: Add Module Re-exports

## Objective

Add module-local Error re-exports and Result type aliases to each domain module, establishing the foundation for the new error handling pattern without breaking existing code.

## Background

This is the first implementation step. We'll add the new patterns alongside existing ones, allowing both to coexist. The actual error enum definitions remain in error.rs to minimize churn.

## Key Questions to Answer

1. Which modules need Error and Result types?
2. How do we handle modules without dedicated errors?
3. Should we add these to mod.rs or separate files?
4. How do we ensure consistency across modules?

## Step-by-Step Process

### 1. Analysis Phase (15 min)

Identify target modules:

```bash
cd shadowcat/src
# List modules that have corresponding errors
ls -la | grep -E "^d" | awk '{print $9}'
# Check which have errors in error.rs
rg "pub enum \w+Error" error.rs
```

### 2. Implementation Phase (1.5 hours)

For each module with a corresponding error type:

#### 2.1 Transport Module
```rust
// src/transport/mod.rs
// Add at the top of the file after other imports
pub use crate::error::TransportError as Error;
pub type Result<T> = std::result::Result<T, Error>;
```

#### 2.2 Session Module
```rust
// src/session/mod.rs
pub use crate::error::SessionError as Error;
pub type Result<T> = std::result::Result<T, Error>;
```

#### 2.3 Storage Module
```rust
// src/storage/mod.rs
pub use crate::error::StorageError as Error;
pub type Result<T> = std::result::Result<T, Error>;
```

#### 2.4 Continue for all modules
- auth/mod.rs
- config/mod.rs
- interceptor/mod.rs
- recorder/mod.rs
- proxy/reverse/mod.rs (special case - nested module)

### 3. Testing Phase (15 min)

Verify additions don't break existing code:

```bash
# Ensure everything still compiles
cargo build
cargo test --lib
cargo clippy --all-targets -- -D warnings

# Check that new types are accessible
cargo doc --no-deps
```

### 4. Documentation Phase (15 min)

Add module-level documentation:

```rust
//! # Errors
//! 
//! This module uses [`Error`] for all error cases and [`Result<T>`] as a convenience alias.
//! 
//! [`Error`]: self::Error
//! [`Result<T>`]: self::Result
```

## Expected Deliverables

### Modified Files
- `src/transport/mod.rs` - Added Error and Result re-exports
- `src/session/mod.rs` - Added Error and Result re-exports
- `src/storage/mod.rs` - Added Error and Result re-exports
- `src/auth/mod.rs` - Added Error and Result re-exports
- `src/config/mod.rs` - Added Error and Result re-exports
- `src/interceptor/mod.rs` - Added Error and Result re-exports
- `src/recorder/mod.rs` - Added Error and Result re-exports
- `src/proxy/reverse/mod.rs` - Added Error and Result re-exports

### Tests
- All existing tests must continue passing
- No new tests needed (additions only)

### Documentation
- Module-level error documentation added
- Rustdoc builds without warnings

## Success Criteria Checklist

- [ ] All target modules have Error re-export
- [ ] All target modules have Result<T> alias
- [ ] No compilation errors
- [ ] No new clippy warnings
- [ ] All tests passing
- [ ] Documentation builds cleanly
- [ ] No changes to existing public API

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Name conflicts with existing Error types | LOW | Use qualified paths if needed |
| Missing a module | MEDIUM | Use checklist, verify with grep |
| Breaking module re-exports | HIGH | Test incrementally |

## Duration Estimate

**Total: 1 hour**
- Analysis: 10 minutes
- Implementation: 30 minutes
- Testing: 10 minutes
- Documentation: 10 minutes

## Dependencies

- A.0: Current State Inventory (must be complete)

## Integration Points

- **error.rs**: Source of error definitions
- **All domain modules**: Need modifications
- **lib.rs**: May need to re-export for convenience

## Performance Considerations

- Zero runtime cost (type aliases and re-exports)
- Slight increase in compilation time (negligible)

## Notes

- Be consistent with placement (top of mod.rs after imports)
- Don't modify error.rs yet (that's B.2)
- Keep changes minimal and focused

## Commands Reference

```bash
cd shadowcat

# Add re-exports (example for one module)
cat >> src/transport/mod.rs << 'EOF'

// Error re-exports for module-local error handling
pub use crate::error::TransportError as Error;
pub type Result<T> = std::result::Result<T, Error>;
EOF

# Verify all modules
for module in transport session storage auth config interceptor recorder; do
  echo "=== $module ==="
  grep -E "(pub use.*Error|pub type Result)" src/$module/mod.rs
done

# Test everything
cargo build && cargo test --lib && cargo clippy
```

## Example Implementation

```rust
// src/transport/mod.rs
// ... existing imports ...

// Error re-exports for module-local error handling
pub use crate::error::TransportError as Error;
pub type Result<T> = std::result::Result<T, Error>;

//! # Transport Module
//! 
//! Handles MCP transport protocols (stdio, HTTP, SSE).
//! 
//! ## Errors
//! 
//! This module uses [`Error`] for all error cases and [`Result<T>`] as a convenience alias.
//! 
//! [`Error`]: self::Error
//! [`Result<T>`]: self::Result

// ... rest of module ...
```

## Follow-up Tasks

After completing this task:
- B.2: Update ShadowcatError to use module paths
- B.3: Begin migrating internal usage

---

**Task Status**: ⬜ Not Started
**Created**: 2025-01-18
**Last Modified**: 2025-01-18
**Author**: Kevin