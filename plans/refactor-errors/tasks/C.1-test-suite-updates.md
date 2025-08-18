# Task C.1: Test Suite Updates

## Objective

Update the test suite to use the new module-local error patterns, ensure all tests pass, and add specific tests for error conversions.

## Background

Tests may need updates to use the new error patterns. We also want to add tests that verify:
- Error conversions work correctly
- Module boundaries are respected

## Key Questions to Answer

1. Which tests need updates?
2. Should tests use old or new patterns?
3. What new tests should we add?
4. How do we test deprecation warnings?

## Step-by-Step Process

### 1. Test Inventory (20 min)

Identify tests needing updates:

```bash
cd shadowcat

# Find test files using old patterns
rg "TransportResult|SessionResult|StorageResult" tests/ --type rust
rg "use.*error::" tests/ --type rust

# Find integration tests
ls tests/*.rs

# Find module tests
find src -name "*.rs" -exec grep -l "#\[cfg(test)\]" {} \;
```

### 2. Update Test Imports (40 min)

Update each test file to use new patterns:

```rust
// Old test
#[cfg(test)]
mod tests {
    use crate::error::{TransportError, TransportResult};
    
    #[test]
    fn test_transport() -> TransportResult<()> {
        // ...
    }
}

// New test
#[cfg(test)]
mod tests {
    use crate::transport::{Error, Result};
    
    #[test]
    fn test_transport() -> Result<()> {
        // ...
    }
}
```

### 3. Add Error Conversion Tests (30 min)

Create tests for error conversions:

```rust
#[cfg(test)]
mod error_conversion_tests {
    use crate::{transport, session, ShadowcatError};
    
    #[test]
    fn test_transport_error_converts_to_shadowcat_error() {
        let transport_err = transport::Error::ConnectionFailed("test".into());
        let shadowcat_err: ShadowcatError = transport_err.into();
        assert!(matches!(shadowcat_err, ShadowcatError::Transport(_)));
    }
    
    #[test]
    fn test_module_result_converts_to_crate_result() {
        fn returns_transport_result() -> transport::Result<()> {
            Err(transport::Error::ConnectionFailed("test".into()))
        }
        
        fn returns_crate_result() -> crate::Result<()> {
            returns_transport_result()?;
            Ok(())
        }
        
        assert!(returns_crate_result().is_err());
    }
}
```

### 4. Run Full Test Suite (10 min)

```bash
# Run all tests
cargo test

# Run with deprecation warnings visible
cargo test 2>&1 | grep deprecated

# Run specific test categories
cargo test --lib
cargo test --tests
cargo test --doc
```

## Expected Deliverables

### Modified Files
- `tests/*.rs` - Updated integration tests
- `src/*/tests.rs` - Updated unit tests
- `src/*/mod.rs` - Updated module tests

### New Tests
- Error conversion tests in `tests/error_conversions.rs`
- Deprecation compatibility tests
- Module boundary tests

### Test Coverage
- All existing tests passing
- New error conversion tests
- Deprecation warning tests

## Success Criteria Checklist

- [ ] All tests updated to new patterns
- [ ] All tests passing
- [ ] Error conversion tests added
- [ ] Deprecation tests added
- [ ] No unexpected deprecation warnings in tests
- [ ] Test coverage maintained or improved
- [ ] Integration tests updated

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Tests using old patterns break | HIGH | Update systematically, test after each file |
| Missing test coverage | MEDIUM | Add new tests for conversions |
| Flaky tests due to changes | LOW | Run tests multiple times |

## Duration Estimate

**Total: 1 hour**
- Test inventory: 10 minutes
- Update imports: 20 minutes
- Add conversion tests: 20 minutes
- Full test run: 10 minutes

## Dependencies

- B.4: Old aliases removed
- All implementation complete

## Integration Points

- **tests/**: Integration tests
- **src/*/tests.rs**: Unit tests
- **examples/**: Example code tests

## Performance Considerations

- Test runtime should not increase
- May compile slightly faster (better locality)

## Notes

- Use `#[allow(deprecated)]` sparingly
- Keep tests focused on behavior, not implementation
- Consider property-based tests for error conversions

## Commands Reference

```bash
cd shadowcat

# Update test imports (example for one file)
sed -i '' 's/use crate::error::TransportResult/use crate::transport::Result/g' tests/transport_test.rs

# Run tests without deprecation warnings
RUSTFLAGS="-A deprecated" cargo test

# Run tests with all output
cargo test -- --nocapture

# Run only error-related tests
cargo test error

# Check test coverage
cargo tarpaulin --out Html
```

## Example Test Updates

```rust
// tests/error_conversions.rs
use shadowcat::{transport, session, storage, ShadowcatError, Result};

#[test]
fn transport_error_to_shadowcat_error() {
    let err = transport::Error::ConnectionFailed("test".into());
    let result: Result<()> = Err(err.into());
    
    match result {
        Err(ShadowcatError::Transport(_)) => (),
        _ => panic!("Expected Transport variant"),
    }
}

#[test]
fn module_result_chain() {
    fn transport_op() -> transport::Result<String> {
        Err(transport::Error::Timeout)
    }
    
    fn high_level_op() -> Result<String> {
        let data = transport_op()?;  // Auto-converts
        Ok(data)
    }
    
    assert!(high_level_op().is_err());
}

#[test]
#[allow(deprecated)]
fn deprecated_alias_compatibility() {
    use shadowcat::error::TransportResult;
    
    fn old_api() -> TransportResult<()> {
        Ok(())
    }
    
    // Should still compile and work
    assert!(old_api().is_ok());
}
```

## Follow-up Tasks

After completing this task:
- C.2: Update documentation
- Run benchmarks to ensure no performance regression
- Create example migrations for users

---

**Task Status**: ⬜ Not Started
**Created**: 2025-01-18
**Last Modified**: 2025-01-18
**Author**: Kevin