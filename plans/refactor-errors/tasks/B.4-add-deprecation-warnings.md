# Task B.4: Add Deprecation Warnings

## Objective

Add deprecation warnings to old Result type aliases and error re-exports, providing clear migration guidance to users while maintaining backward compatibility.

## Background

With internal usage migrated, we can now mark the old patterns as deprecated. This guides external users to adopt the new patterns while giving them time to migrate.

## Key Questions to Answer

1. What deprecation message is most helpful?
2. Should we provide migration commands in the message?
3. How do we handle the deprecation timeline?
4. Should we deprecate error re-exports too?

## Step-by-Step Process

### 1. Design Deprecation Messages (15 min)

Create helpful, actionable deprecation messages:

```rust
#[deprecated(
    since = "0.X.0",
    note = "Use `crate::transport::Result` instead. \
            To migrate: `sed -i 's/TransportResult</crate::transport::Result</g' src/**/*.rs` \
            or with cargo: `cargo fix --edition`"
)]
```

### 2. Implementation Phase (30 min)

Add deprecations to error.rs:

#### 2.1 Deprecate Result aliases
```rust
// src/error.rs

#[deprecated(
    since = "0.X.0",
    note = "Use `crate::transport::Result` instead. \
            To migrate: `sed -i 's/TransportResult</crate::transport::Result</g' src/**/*.rs`"
)]
pub type TransportResult<T> = crate::transport::Result<T>;

#[deprecated(
    since = "0.X.0",
    note = "Use `crate::session::Result` instead. \
            To migrate: `sed -i 's/SessionResult</crate::session::Result</g' src/**/*.rs`"
)]
pub type SessionResult<T> = crate::session::Result<T>;

// Continue for all Result aliases...
```

#### 2.2 Optionally deprecate error re-exports
```rust
#[deprecated(
    since = "0.X.0",
    note = "Use `crate::transport::Error` instead"
)]
pub use crate::transport::Error as TransportError;
```

### 3. Testing Phase (10 min)

Verify deprecations work correctly:

```bash
# Build and check for deprecation warnings
cargo build 2>&1 | grep deprecated

# Ensure we can suppress warnings for our own use
RUSTFLAGS="-A deprecated" cargo build

# Check that external usage gets warnings
cd examples/
cargo build 2>&1 | grep deprecated
```

### 4. Documentation Phase (5 min)

Update documentation with migration guide.

## Expected Deliverables

### Modified Files
- `src/error.rs` - Added deprecation attributes to old aliases

### New Files
- `analysis/migration-guide.md` - User-facing migration guide

### Documentation Updates
- CHANGELOG.md entry about deprecations
- README.md note about new error patterns

## Success Criteria Checklist

- [ ] All old Result aliases deprecated
- [ ] Helpful migration messages included
- [ ] Migration commands tested and working
- [ ] Documentation updated
- [ ] Examples still compile (with warnings)
- [ ] Internal code doesn't trigger warnings

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Too many warnings overwhelm users | MEDIUM | Clear, actionable messages |
| Migration commands don't work on all platforms | LOW | Provide multiple options |
| Users ignore deprecations | MEDIUM | Blog post, clear timeline |

## Duration Estimate

**Total: 1 hour**
- Design messages: 15 minutes
- Implementation: 30 minutes
- Testing: 10 minutes
- Documentation: 5 minutes

## Dependencies

- B.3: Internal usage migrated (no internal warnings)

## Integration Points

- **error.rs**: Add deprecation attributes
- **CI/CD**: May need to allow deprecations temporarily
- **Documentation**: Multiple files need updates

## Performance Considerations

- Zero runtime impact
- Compile-time warnings (expected)

## Notes

- Keep messages concise but helpful
- Include migration commands for common platforms
- Consider different sed syntax for macOS vs Linux
- Make sure version number is correct

## Commands Reference

```bash
cd shadowcat

# Add deprecation to a type (example)
cat >> src/error.rs << 'EOF'
#[deprecated(
    since = "0.X.0",
    note = "Use `crate::transport::Result` instead. See migration guide: docs/MIGRATION.md"
)]
pub type TransportResult<T> = crate::transport::Result<T>;
EOF

# Test deprecation warnings
cargo build 2>&1 | grep deprecated

# Count deprecation warnings
cargo build 2>&1 | grep -c deprecated

# Build without warnings (for CI)
RUSTFLAGS="-A deprecated" cargo build
```

## Example Implementation

```rust
// src/error.rs

// Re-export errors (optionally deprecated)
#[deprecated(since = "0.X.0", note = "Import `crate::transport::Error` instead")]
pub use crate::transport::Error as TransportError;

// Deprecated Result aliases
#[deprecated(
    since = "0.X.0",
    note = "Use `crate::transport::Result` instead.\n\
            \n\
            To migrate automatically:\n\
            - Run: `cargo fix --edition`\n\
            - Or: `sed -i 's/TransportResult</crate::transport::Result</g' src/**/*.rs`\n\
            \n\
            For more information, see: https://github.com/yourusername/shadowcat/blob/main/docs/MIGRATION.md"
)]
pub type TransportResult<T> = crate::transport::Result<T>;

// Keep this for high-level APIs
pub type Result<T> = std::result::Result<T, ShadowcatError>;
```

## Migration Guide Template

```markdown
# Migrating to Module-Local Error Types

As of version 0.X.0, Shadowcat uses module-local Error and Result types for better ergonomics and clarity.

## What Changed

**Before:**
```rust
use shadowcat::error::{TransportError, TransportResult};
```

**After:**
```rust
use shadowcat::transport::{Error, Result};
// Or for clarity:
use shadowcat::transport;
// Then use transport::Result<T> and transport::Error
```

## Migration Steps

### Automated Migration
```bash
cargo fix --edition
```

### Semi-Automated Migration
```bash
# For each module (example: transport)
sed -i 's/TransportResult</crate::transport::Result</g' src/**/*.rs
sed -i 's/use crate::error::TransportResult/use crate::transport::Result/g' src/**/*.rs
```

### Manual Migration
1. Update imports at the top of each file
2. Replace Result type usage
3. Run `cargo build` to find remaining issues

## Timeline
- Version 0.X.0: Deprecation warnings introduced
- Version 0.X+2.0: Deprecated items removed
```

## Follow-up Tasks

After completing this task:
- C.1: Update test suite
- C.2: Complete documentation updates
- Create GitHub issue for tracking migration

---

**Task Status**: ⬜ Not Started
**Created**: 2025-01-18
**Last Modified**: 2025-01-18
**Author**: Kevin