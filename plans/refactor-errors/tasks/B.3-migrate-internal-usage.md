# Task B.3: Migrate Internal Usage

## Objective

Migrate internal code from using centralized error Result aliases (TransportResult, SessionResult, etc.) to module-local Result types, establishing the new pattern throughout the codebase.

## Background

This is the most substantial task - updating all internal usage to the new pattern. Since we're not maintaining backward compatibility, we can update everything in one go.

## Key Questions to Answer

1. How do we handle cross-module error usage?
2. Should we use qualified paths or imports?
3. How do we ensure we don't miss any usage?

## Step-by-Step Process

### 1. Migration Order Planning (15 min)

Based on dependency analysis, migrate in this order:
1. storage (fewest dependencies)
2. config (simple, isolated)
3. interceptor (well-contained)
4. recorder (depends on storage)
5. auth (depends on config)
6. session (central, many dependencies)
7. transport (most complex)
8. proxy (depends on transport/session)

### 2. Module-by-Module Migration (2 hours)

For each module:

#### 2.1 Update imports
```rust
// Old
use crate::error::TransportResult;

// New
use crate::transport::Result;
// Or for clarity in mixed contexts:
use crate::transport::Result as TransportResult;
```

#### 2.2 Update function signatures
```rust
// Old
pub fn connect() -> TransportResult<Connection>

// New (if in transport module)
pub fn connect() -> Result<Connection>

// New (if in another module)
pub fn connect() -> transport::Result<Connection>
```

#### 2.3 Update type annotations where needed
```rust
// May need to qualify in some contexts
let result: transport::Result<()> = transport_op();
```

### 3. Testing Phase (30 min)

After each module migration:
```bash
# Test specific module
cargo test --lib module_name

# Run clippy
cargo clippy --all-targets -- -D warnings

# Full test suite after all modules
cargo test
```

### 4. Search and Verify (15 min)

Ensure complete migration:
```bash
# Find remaining old usage
rg "TransportResult<" --type rust -g '!error.rs'
rg "SessionResult<" --type rust -g '!error.rs'
# ... for all old aliases

# Verify new usage
rg "use.*::Result" --type rust src/
```

## Expected Deliverables

### Modified Files
Per module (example for transport):
- `src/transport/*.rs` - Updated to use module Result
- `src/transport/tests/*.rs` - Updated test files
- Files using transport errors - Updated imports

### Migration Checklist
- [ ] storage module migrated
- [ ] config module migrated
- [ ] interceptor module migrated
- [ ] recorder module migrated
- [ ] auth module migrated
- [ ] session module migrated
- [ ] transport module migrated
- [ ] proxy module migrated

### Tests
- All tests passing after each module migration
- No new test failures

## Success Criteria Checklist

- [ ] All internal usage migrated
- [ ] No remaining old aliases (except in error.rs)
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Consistent usage patterns
- [ ] Clear import structure

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Missing some usage | MEDIUM | Use multiple search patterns |
| Breaking tests | MEDIUM | Fix as we find them |
| Import confusion | LOW | Use qualified paths where unclear |
| Merge conflicts if long-running | HIGH | Complete in single session/PR |

## Duration Estimate

**Total: 3 hours**
- Planning: 15 minutes
- Module migration: 2 hours
- Testing: 30 minutes
- Verification: 15 minutes

## Dependencies

- B.1: Module re-exports in place
- B.2: ShadowcatError updated

## Integration Points

- **All modules**: Need updates
- **Tests**: May need import changes
- **Examples**: Should be updated for consistency

## Performance Considerations

- No runtime impact
- Might slightly improve compilation (better locality)

## Notes

- Be consistent: use unqualified Result within a module, qualified outside
- Update imports at the top, then fix usage sites
- Run tests after each module to catch issues early

## Commands Reference

```bash
cd shadowcat

# Scripted migration for a module (example: transport)
# Step 1: Update imports
rg -l "use crate::error::TransportResult" src/transport/ | \
  xargs sed -i '' 's/use crate::error::TransportResult/use crate::transport::Result/g'

# Step 2: Update function signatures in module
rg -l "TransportResult<" src/transport/ | \
  xargs sed -i '' 's/TransportResult</Result</g'

# Step 3: Update external usage
rg -l "TransportResult<" src/ --glob '!transport/**' --glob '!error.rs' | \
  xargs sed -i '' 's/TransportResult</transport::Result</g'

# Test after each module
cargo test --lib transport
cargo clippy

# Final verification
rg "TransportResult" --type rust -g '!error.rs'
```

## Example Migration

```rust
// Before (src/transport/stdio.rs)
use crate::error::{TransportError, TransportResult};

impl StdioTransport {
    pub async fn connect(&mut self) -> TransportResult<()> {
        // ...
    }
}

// After (src/transport/stdio.rs)
use crate::transport::{Error, Result};

impl StdioTransport {
    pub async fn connect(&mut self) -> Result<()> {
        // ...
    }
}

// Usage from another module (src/proxy/forward.rs)
// Before
use crate::error::TransportResult;
fn setup_transport() -> TransportResult<Transport> { ... }

// After
use crate::transport;
fn setup_transport() -> transport::Result<Transport> { ... }
```

## Follow-up Tasks

After completing this task:
- B.4: Add deprecation warnings to old aliases
- C.1: Update test suite for new patterns

---

**Task Status**: ⬜ Not Started
**Created**: 2025-01-18
**Last Modified**: 2025-01-18
**Author**: Kevin