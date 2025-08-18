# Task C.2: Documentation Updates

## Objective

Complete all documentation updates for the error refactoring, including rustdoc comments, changelog, README updates, and example code.

## Background

Good documentation is critical. Since this is pre-release, we need to document:
- How to use the new patterns
- The modular error architecture
- Examples of common patterns

## Key Questions to Answer

1. What documentation needs updating?
2. What examples best demonstrate the new patterns?
3. How do we clearly explain module vs crate-level errors?

## Step-by-Step Process

### 1. Rustdoc Updates (30 min)

Update module and error documentation:

#### 1.1 Module-level docs
```rust
//! # Transport Module
//! 
//! ## Error Handling
//! 
//! This module uses [`Error`] for all error conditions and provides
//! [`Result<T>`] as a convenience alias for `std::result::Result<T, Error>`.
//! 
//! ### Example
//! ```rust
//! use shadowcat::transport::{Error, Result};
//! 
//! fn connect() -> Result<Connection> {
//!     // Returns transport::Result
//! }
//! ```
//! 
//! [`Error`]: self::Error
//! [`Result<T>`]: self::Result
```

#### 1.2 Error type docs
```rust
/// Transport-specific error conditions.
/// 
/// This error type is re-exported as `transport::Error` for module-local use.
/// It automatically converts to [`ShadowcatError`] for high-level APIs.
/// 
/// [`ShadowcatError`]: crate::ShadowcatError
```

### 2. Changelog Entry (10 min)

Write clear changelog entry:

```markdown
## [0.X.0] - 2025-XX-XX

### Added
- Module-local `Error` and `Result` types for better ergonomics
- Each module now exports its own `Error` and `Result` types
- Comprehensive migration guide in docs/MIGRATION.md

### Changed
- Error types are now accessed via their modules (e.g., `transport::Error`)
- Result types are now module-specific (e.g., `transport::Result<T>`)

### Removed
- Old Result type aliases (TransportResult, SessionResult, etc.)
- Direct error exports from error module
```

### 3. README Updates (10 min)

Update main README with new patterns:

```markdown
## Error Handling

Shadowcat uses module-local error types for clarity:

```rust
use shadowcat::transport;

// Module-specific errors
fn transport_operation() -> transport::Result<Data> {
    // Returns transport::Error on failure
}

// High-level operations
use shadowcat::Result;
fn orchestrate() -> Result<()> {
    // Returns ShadowcatError on failure
}
```
```

### 4. Example Code Updates (10 min)

Update all examples to use new patterns:

```bash
# Find example files
find examples -name "*.rs"

# Update each example
sed -i '' 's/use shadowcat::error::/use shadowcat::transport::/g' examples/*.rs
```

### 5. API Documentation (10 min)

Ensure cargo doc output is clear:

```bash
# Generate and review docs
cargo doc --no-deps --open

# Check for broken links
cargo doc --no-deps 2>&1 | grep -i warning
```

## Expected Deliverables

### New Files
- `docs/ERROR_HANDLING.md` - Error handling patterns guide

### Modified Files
- `CHANGELOG.md` - Version entry with deprecations
- `README.md` - Updated error handling section
- `src/lib.rs` - Updated crate-level documentation
- All module `mod.rs` files - Module-level error docs
- `examples/*.rs` - Updated to new patterns

### Documentation Coverage
- All public APIs documented
- Migration path clear
- Examples updated
- Deprecation timeline stated

## Success Criteria Checklist

- [ ] All rustdoc comments updated
- [ ] Migration guide complete
- [ ] Changelog entry written
- [ ] README updated
- [ ] Examples use new patterns
- [ ] cargo doc builds without warnings
- [ ] Deprecation timeline documented
- [ ] Cross-references working

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Incomplete migration guide | HIGH | Test with real user code |
| Broken documentation links | LOW | Use cargo doc to verify |
| Outdated examples | MEDIUM | Test all examples compile |

## Duration Estimate

**Total: 1 hour**
- Rustdoc updates: 20 minutes
- Changelog: 10 minutes
- README updates: 10 minutes
- Example updates: 10 minutes
- API documentation: 10 minutes

## Dependencies

- C.1: Tests updated and passing
- All implementation complete

## Integration Points

- **docs/**: Main documentation directory
- **All modules**: Need doc updates
- **examples/**: Example code
- **README.md**: Project entry point

## Performance Considerations

N/A - Documentation only

## Notes

- Use consistent terminology throughout
- Provide copy-paste examples
- Include platform-specific commands (macOS/Linux)
- Link to this tracker for context

## Commands Reference

```bash
cd shadowcat

# Generate documentation
cargo doc --no-deps

# Check for doc issues
cargo doc --no-deps 2>&1 | grep -i warning

# Serve documentation locally
python3 -m http.server --directory target/doc 8000

# Test examples compile
for example in examples/*.rs; do
  rustc --edition 2021 --crate-type bin "$example" -L target/debug/deps
done
```

## Documentation Templates

### Module Error Documentation
```rust
//! ## Error Handling
//! 
//! This module uses:
//! - [`Error`](self::Error) - Module-specific error type
//! - [`Result<T>`](self::Result) - Convenience alias for `std::result::Result<T, Error>`
//! 
//! ### Examples
//! 
//! ```rust
//! use shadowcat::MODULE_NAME::{Error, Result};
//! 
//! fn operation() -> Result<Data> {
//!     Err(Error::SomeVariant("details".into()))
//! }
//! ```
```

### Migration Example
```markdown
## Before (0.X-1.0)
```rust
use shadowcat::error::{TransportError, TransportResult};

fn old_pattern() -> TransportResult<()> {
    Err(TransportError::ConnectionFailed("...".into()))
}
```

## After (0.X.0+)
```rust
use shadowcat::transport::{Error, Result};

fn new_pattern() -> Result<()> {
    Err(Error::ConnectionFailed("...".into()))
}
```
```

## Follow-up Tasks

After completing this task:
- Create GitHub release with migration notes
- Consider blog post explaining the change
- Monitor GitHub issues for migration problems
- Plan removal of deprecations for 0.X+2.0

---

**Task Status**: ⬜ Not Started
**Created**: 2025-01-18
**Last Modified**: 2025-01-18
**Author**: Kevin