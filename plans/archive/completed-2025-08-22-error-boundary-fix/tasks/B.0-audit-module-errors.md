# Task B.0: Audit Module Errors

## Objective
Create proper Error and Result types for the audit module and remove all references to `crate::Error`.

## Current State
- The audit module has 20+ references to crate::Error (based on screenshot)
- No module-specific Error type exists
- Module handles audit logging and compliance tracking

## Process

### Step 1: Analyze Current Usage
```bash
# Find all crate::Error usage in audit module
grep -rn "crate::Error\|crate::Result" src/audit/

# Understand what errors the module needs to handle
rg "return Err\|\.map_err\|\?" src/audit/
```

### Step 2: Create Module Error Type

Create `src/audit/error.rs`:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Audit log write failed: {0}")]
    LogWriteFailed(String),
    
    #[error("Audit store error: {0}")]
    StorageError(String),
    
    #[error("Event serialization failed: {0}")]
    SerializationFailed(String),
    
    #[error("Audit policy violation: {0}")]
    PolicyViolation(String),
    
    // From dependencies
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Step 3: Update Module Exports

In `src/audit/mod.rs`:
```rust
mod error;
pub use error::{Error, Result};

// Remove any: use crate::{Error, Result};
```

### Step 4: Update Function Signatures

Change all functions from:
```rust
pub async fn audit_event(...) -> crate::Result<()> {
```

To:
```rust
pub async fn audit_event(...) -> Result<()> {
```

### Step 5: Update Error Construction

Change from:
```rust
return Err(crate::Error::Audit(format!("Failed: {}", e)));
```

To:
```rust
return Err(Error::LogWriteFailed(format!("Failed: {}", e)));
```

### Step 6: Update Parent Module

In `src/lib.rs`, ensure:
```rust
#[derive(Error, Debug)]
pub enum Error {
    // ...
    #[error("Audit error: {0}")]
    Audit(#[from] audit::Error),
    // ...
}
```

### Step 7: Fix Tests

Update any tests that were expecting `crate::Error`:
```rust
// Before
assert!(matches!(result.unwrap_err(), crate::Error::Audit(_)));

// After  
assert!(matches!(result.unwrap_err(), audit::Error::LogWriteFailed(_)));
```

## Validation

```bash
# Ensure no more crate::Error references
grep -rn "crate::Error\|crate::Result" src/audit/ | grep -v "^src/audit/error.rs"

# Run module tests
cargo test audit::

# Check for clippy warnings
cargo clippy --all-targets -- -D warnings
```

## Success Criteria
- [ ] audit::Error enum created with appropriate variants
- [ ] audit::Result type alias created
- [ ] All function signatures updated
- [ ] All error construction updated
- [ ] Parent module updated with #[from]
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] No references to crate::Error remain

## Time Estimate
3 hours

## Dependencies
- A.2 (Migration Strategy) must be complete

## Common Issues & Solutions

### Issue: Error variant naming
- Use descriptive names that indicate what failed
- Don't just wrap other errors, add context

### Issue: Missing error context
- Include relevant details in error messages
- Use format strings to add runtime values

### Issue: Test compilation failures
- Update test error matching
- May need to import module Error type in tests

## Notes
- Audit module is high-priority due to many violations
- Be careful with public API - may need to maintain compatibility
- Consider if audit errors should be logged differently