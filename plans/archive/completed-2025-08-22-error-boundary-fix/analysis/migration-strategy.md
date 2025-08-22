# Migration Strategy

## Executive Summary

We have 18 direct violations across 9 modules, much fewer than the 161 initially expected. The main issues are:
1. Modules with Error types still constructing `crate::Error` directly (auth)
2. Missing Error types in foundation modules (mcp, telemetry, process, audit)
3. Traits using `crate::Result` instead of generic approaches (pool)

## Migration Order

Based on dependency analysis, here's the optimal migration sequence:

### Phase 1: Foundation Modules (8 hours)
Start with modules that have no internal dependencies.

#### 1.1 Create telemetry::Error (2 hours)
- **Files**: src/telemetry/mod.rs
- **Dependencies**: None
- **Complexity**: Low - simple module
- **Actions**: Create Error enum, Result type, update functions

#### 1.2 Create process::Error (2 hours)
- **Files**: src/process/mod.rs  
- **Dependencies**: telemetry (done), transport (has Error)
- **Complexity**: Low - simple module
- **Actions**: Create Error enum, Result type, update functions

#### 1.3 Create mcp::Error (3 hours)
- **Files**: validation.rs, handshake.rs, handler.rs, encoding.rs, builder.rs
- **Dependencies**: transport (has Error)
- **Complexity**: Medium - multiple files
- **Actions**: Create Error enum, Result type, update all files

#### 1.4 Create shutdown::Error (1 hour)
- **Files**: src/shutdown.rs
- **Dependencies**: None
- **Complexity**: Low - single file
- **Actions**: Create Error enum, Result type, or leave as boundary

### Phase 2: Service Modules (6 hours)

#### 2.1 Create audit::Error (3 hours)
- **Files**: logger.rs, store.rs
- **Dependencies**: auth, mcp (will have Errors), rate_limiting (has Error)
- **Complexity**: Medium
- **Actions**: Create Error enum, remove ShadowcatResult alias

#### 2.2 Fix pool traits (3 hours)
- **Files**: mod.rs, traits.rs
- **Current issue**: Traits return `crate::Result`
- **Solution**: Use associated types or module Result
- **Complexity**: Medium - need to update trait implementations

### Phase 3: Fix Existing Modules (6 hours)

#### 3.1 Fix auth module (4 hours)
- **Files**: gateway.rs, middleware.rs, policy.rs, rate_limit.rs
- **Current issue**: Constructs `crate::Error::Auth` directly
- **Solution**: Always return `auth::Error`, remove aliases
- **Complexity**: High - OAuth handling, middleware

#### 3.2 Fix proxy modules (2 hours)
- **Files**: forward/single_session.rs, forward/multi_session.rs, reverse/upstream/stdio.rs
- **Current issue**: Using `crate::Result` or constructing `crate::Error`
- **Solution**: Use module-local types
- **Complexity**: Medium

### Phase 4: Cleanup (2 hours)

#### 4.1 Minor fixes (2 hours)
- transport/factory.rs - might be at boundary
- session/builder.rs - single import
- Verify all tests pass
- Run clippy

## Migration Patterns

### Standard Module Structure

Every module needing errors should follow this pattern:

```rust
// src/module/error.rs (or in mod.rs for small modules)
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // Module-specific variants
    #[error("Module-specific error: {0}")]
    SpecificError(String),
    
    // From dependencies (NOT crate::Error)
    #[error("Transport error")]
    Transport(#[from] transport::Error),
    
    // From std
    #[error("IO error")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Conversion at Boundaries

Only at public API boundaries (api.rs):

```rust
// Internal module
pub async fn do_work() -> module::Result<Data> {
    // Return module::Error on failure
    Err(module::Error::Failed("reason".into()))
}

// At API boundary (api.rs)
pub async fn public_api() -> crate::Result<Data> {
    module::do_work()
        .await
        .map_err(Into::into)  // Auto-conversion via #[from]
}
```

### Fixing Traits

For the pool module traits:

```rust
// Before (BAD)
trait PooledResource {
    async fn close(&mut self) -> crate::Result<()>;
}

// After (GOOD) - Option 1: Module Result
trait PooledResource {
    async fn close(&mut self) -> pool::Result<()>;
}

// After (GOOD) - Option 2: Associated Type
trait PooledResource {
    type Error;
    async fn close(&mut self) -> Result<(), Self::Error>;
}
```

## Testing Strategy

### After Each Module
```bash
# Run module tests
cargo test module_name::

# Check for compilation
cargo check --all-targets

# Verify no new violations
grep -r "crate::Error" src/module_name/
```

### After Each Phase
```bash
# Full test suite
cargo test --all

# Clippy check
cargo clippy --all-targets -- -D warnings

# Count remaining violations
grep -r "crate::Error\|crate::Result" src/ | grep -v "src/lib.rs" | grep -v "src/main.rs" | wc -l
```

## Risk Mitigation

### Risk: Circular dependency (auth ↔ interceptor)
**Mitigation**: These modules already have Error types. Focus on fixing usage, not structure.

### Risk: Breaking changes
**Mitigation**: 
- Each module in separate commit
- Test after each module
- Can revert individual modules

### Risk: Merge conflicts
**Mitigation**:
- Complete quickly (22 hours over 2-3 days)
- Focus on one module at a time
- Coordinate with team on auth/proxy changes

## Rollback Plan

Each phase can be reverted independently:
1. Phase commits are atomic
2. Each module change is isolated
3. Tests verify no regressions
4. Can cherry-pick successful changes

## Success Metrics

### Per Module
- ✅ No `crate::Error` references (except lib.rs)
- ✅ No `crate::Result` references (except lib.rs)
- ✅ Module has Error and Result types
- ✅ All tests pass
- ✅ No clippy warnings

### Overall
- ✅ 0 violations (down from 18)
- ✅ All modules have proper Error types
- ✅ Clean error propagation chains
- ✅ No performance regression
- ✅ Binary size increase < 5%

## Quick Reference Checklist

For each module migration:

- [ ] Create `Error` enum with appropriate variants
- [ ] Create `Result<T>` type alias
- [ ] Update all function signatures from `crate::Result` to `Result`
- [ ] Update all error construction to use module Error
- [ ] Add `#[from]` for dependency errors
- [ ] Update parent module if new Error type
- [ ] Run module tests
- [ ] Check for remaining violations
- [ ] Fix any clippy warnings
- [ ] Commit with clear message

## Timeline Estimate

**Total: 22 hours**

- Phase 1 (Foundation): 8 hours
- Phase 2 (Services): 6 hours  
- Phase 3 (Fix Existing): 6 hours
- Phase 4 (Cleanup): 2 hours

Can be completed in 2-3 focused days or spread over a week.

## Next Steps

1. ✅ Complete analysis (DONE)
2. Start with Phase 1.1 (telemetry module)
3. Follow checklist for each module
4. Update tracker after each phase
5. Create PR after full completion