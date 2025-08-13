# Task 002: Fix Duplicate Error Types

## Overview
Remove duplicate error type definitions in `src/error.rs` that are causing confusion and potential compilation issues.

## Context
The [comprehensive review](../../reviews/shadowcat-comprehensive-review-2025-08-06.md) identified duplicate definitions of `ConfigurationError` and `AuthenticationError` in the same file.

## Problem
**Location**: `src/error.rs:7-15`

The file contains two sets of error definitions:
1. Plain enums without derives
2. Same enums with `#[derive(Error, Debug)]`

## Current State
```rust
// First definition (lines 7-10)
pub enum ConfigurationError {
    InvalidConfig,
    MissingField(String),
}

// Second definition (lines 12-15) 
pub enum AuthenticationError {
    InvalidToken,
    Expired,
}

// Later in the same file...
#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("Invalid configuration")]
    InvalidConfig,
    #[error("Missing required field: {0}")]
    MissingField(String),
}

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("Invalid authentication token")]
    InvalidToken,
    #[error("Authentication expired")]
    Expired,
}
```

## Solution

### Step 1: Identify Which Definition to Keep
Keep the `#[derive(Error, Debug)]` versions as they:
- Implement the `Error` trait properly
- Include error messages
- Work with `thiserror` crate

### Step 2: Remove Duplicate Definitions

1. Open `src/error.rs`
2. Delete the plain enum definitions (without derives)
3. Keep only the derived versions

### Step 3: Verify No Breaking Changes

Check all usages of these error types:
```bash
# Find all uses of ConfigurationError
rg "ConfigurationError" --type rust

# Find all uses of AuthenticationError  
rg "AuthenticationError" --type rust
```

### Step 4: Update Import Statements
If any imports are affected, update them:
```rust
// Ensure imports are correct
use crate::error::{ConfigurationError, AuthenticationError};
```

## Expected Final State

```rust
// src/error.rs - Only these definitions should remain:

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("Invalid configuration")]
    InvalidConfig,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid value for {field}: {value}")]
    InvalidValue { field: String, value: String },
}

#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("Invalid authentication token")]
    InvalidToken,
    #[error("Authentication expired")]
    Expired,
    #[error("Insufficient permissions")]
    Unauthorized,
    #[error("Authentication failed: {0}")]
    Failed(String),
}

// ... rest of error types ...
```

## Additional Improvements

While fixing duplicates, also:

1. **Add missing variants** that might be needed:
```rust
#[derive(Error, Debug)]
pub enum ConfigurationError {
    // ... existing variants ...
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),
    #[error("Failed to parse configuration: {0}")]
    ParseError(String),
}
```

2. **Ensure proper From implementations**:
```rust
impl From<std::io::Error> for ConfigurationError {
    fn from(err: std::io::Error) -> Self {
        ConfigurationError::Failed(err.to_string())
    }
}
```

3. **Check error conversions in ShadowcatError**:
```rust
#[derive(Error, Debug)]
pub enum ShadowcatError {
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigurationError),
    
    #[error("Authentication error: {0}")]
    Authentication(#[from] AuthenticationError),
    
    // ... other variants ...
}
```

## Validation

### Before Starting
```bash
# Check current compilation status
cargo check 2>&1 | grep -E "(ConfigurationError|AuthenticationError)"
```

### After Completion
```bash
# Should compile without errors
cargo check

# Should build successfully
cargo build

# Run tests to ensure nothing broke
cargo test

# Check for any warnings
cargo clippy -- -D warnings
```

## Common Issues

1. **Ambiguous imports**: If you see "ambiguous import" errors, ensure only one definition exists
2. **Match patterns**: Update any match statements that might rely on specific variants
3. **Error conversions**: Verify `?` operator still works where these errors are used

## Files Likely Affected

- `src/error.rs` - Main changes
- `src/config.rs` - Uses ConfigurationError
- `src/auth/mod.rs` - Uses AuthenticationError
- `src/auth/oauth.rs` - Uses AuthenticationError
- `src/cli/mod.rs` - May import these errors

## Success Criteria

- [ ] Only one definition of each error type exists
- [ ] All error types have proper `#[derive(Error, Debug)]`
- [ ] Code compiles without errors or warnings
- [ ] All tests pass
- [ ] Error messages are descriptive and helpful

## Time Estimate
30 minutes - This is a straightforward fix that mainly involves deleting duplicate code.