# Breaking Changes Documentation

## 1. ShadowcatConfig → Config Rename

### Affected Files (5 files, 19 references)
```
src/api.rs
src/config/loader.rs
src/config/mod.rs
src/config/schema.rs
src/config/validator.rs
```

### Import Changes Required
```rust
// Before
use shadowcat::config::ShadowcatConfig;
use crate::config::schema::ShadowcatConfig;

// After
use shadowcat::config::Config;
use crate::config::schema::Config;
```

### Public API Impact
- **Low Risk**: The type is mostly used internally
- **api.rs**: May affect public API if ShadowcatConfig is exposed
- Need to check if any examples or documentation reference it

## 2. Error Enum Changes

### Removed/Changed Variants
```rust
// These will be removed or changed:
Error::Invalid(String)  // Will remain but usage reduced
Error::MissingField(String)  // Changed to structured variant
```

### New Variants to Handle
Code that matches on config::Error will need updates:
```rust
// Before
match error {
    Error::Invalid(msg) => println!("Invalid: {}", msg),
    Error::MissingField(field) => println!("Missing: {}", field),
    _ => {}
}

// After
match error {
    Error::InvalidPort { port, reason } => println!("Port {} invalid: {}", port, reason),
    Error::InvalidAddress { addr, .. } => println!("Address {} invalid", addr),
    Error::MissingField { field, section } => println!("Missing {} in {}", field, section),
    Error::Invalid(msg) => println!("Invalid: {}", msg),  // Fallback
    _ => {}
}
```

## 3. Error Construction Changes

### Validation Code Updates
All 68 instances of `Error::Invalid` construction will change:

```rust
// Before
return Err(Error::Invalid(format!("Invalid port: {}", port)));

// After
return Err(Error::InvalidPort {
    port,
    reason: PortError::OutOfRange,
});
```

## 4. New Dependencies

### Internal Types
Code using config errors may need to import new types:
```rust
use shadowcat::config::{Error, PortError, ResourceType};
```

## 5. Behavioral Changes

### Error Messages
Error messages will change format:
```
// Before
"Invalid port in server bind address 'localhost:80': permission denied"

// After
"Invalid port 80: port requires elevated privileges"
```

### New Methods
Code can now use new error methods:
```rust
// New capability
if let Err(e) = config.validate() {
    eprintln!("Error: {}", e);
    eprintln!("Help: {}", e.help_text());  // NEW
}
```

## Migration Strategy

### Phase 1: Add New, Keep Old
1. Add new error variants alongside existing
2. Add deprecation warnings to old patterns
3. Migrate internal usage gradually

### Phase 2: Update Dependents
1. Update all internal validators
2. Update examples
3. Update documentation

### Phase 3: Remove Deprecated
1. Remove old variants (if any)
2. Final cleanup

## Rollback Plan

If issues arise:
1. The `Invalid(String)` variant remains as fallback
2. Can temporarily type alias: `type ShadowcatConfig = Config;`
3. Git revert is straightforward since changes are localized

## Testing Requirements

### Before Release
- [ ] All existing config tests pass
- [ ] New error variants have tests
- [ ] Help text tested for usefulness
- [ ] Example configs validated
- [ ] API compatibility verified

### Integration Testing
- [ ] Test with real config files
- [ ] Test with invalid configs to see new errors
- [ ] Verify error messages are improvements

## Communication

### Changelog Entry
```markdown
### Breaking Changes
- Renamed `ShadowcatConfig` to `Config` for cleaner imports
- Enhanced config error types with specific variants for better debugging
- Added `help_text()` method to config errors for actionable guidance

### Migration
- Change imports from `ShadowcatConfig` to `Config`
- Error matching code may need updates for new variants
- Most code will continue to work with `Error::Invalid` fallback
```

## Risk Assessment

| Change | Risk | Impact | Mitigation |
|--------|------|--------|------------|
| Type rename | Low | Import changes only | Simple find/replace |
| Error variants | Medium | Match statements break | Keep Invalid fallback |
| Error messages | Low | Display changes only | Improvements only |
| New methods | None | Additive only | No breaking changes |

## Estimated Effort

- **Rename ShadowcatConfig**: 30 minutes (mechanical change)
- **Update error construction**: 2-3 hours (68 instances)
- **Testing**: 1 hour
- **Documentation**: 30 minutes

Total: ~4-5 hours of focused work