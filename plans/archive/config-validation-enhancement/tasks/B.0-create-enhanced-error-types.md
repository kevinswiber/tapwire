# Task B.0: Create Enhanced Error Types

## Objective
Implement the new config::Error enum with rich, specific variants based on the analysis from A.0.

## Key Decisions
1. Which error variants to implement first?
2. What supporting enums do we need (PortError, ResourceType, etc.)?
3. How much context should each variant carry?
4. Should we use #[error(transparent)] for any variants?

## Process

### Step 1: Create New error.rs File
Create `src/config/error.rs` with the enhanced error enum.

### Step 2: Define Core Error Enum
```rust
use thiserror::Error;
use std::path::PathBuf;

#[derive(Debug, Error)]
pub enum Error {
    // Network/Address errors
    #[error("Invalid port {port}: {reason}")]
    InvalidPort {
        port: u16,
        reason: PortError,
    },
    
    #[error("Invalid address '{addr}': {reason}")]
    InvalidAddress {
        addr: String,
        reason: String,
        #[source]
        source: Option<std::net::AddrParseError>,
    },
    
    // Configuration errors
    #[error("Rate limiting misconfigured: {reason}")]
    RateLimiting {
        reason: String,
        suggestion: String,
        rpm: Option<u32>,
        burst: Option<u32>,
    },
    
    // Keep some generic ones
    #[error("Configuration invalid: {0}")]
    Invalid(String),
    
    // Existing useful ones
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Missing required field: {field} in section {section}")]
    MissingField {
        field: String,
        section: String,
    },
}
```

### Step 3: Define Supporting Enums
```rust
#[derive(Debug, Clone, Copy)]
pub enum PortError {
    Privileged,      // Ports 0-1023 need root
    OutOfRange,      // > 65535
    Reserved,        // Well-known service ports
    AlreadyInUse,    // Port binding would fail
}

impl std::fmt::Display for PortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Privileged => write!(f, "port requires elevated privileges"),
            Self::OutOfRange => write!(f, "port number out of valid range"),
            Self::Reserved => write!(f, "port is reserved for well-known service"),
            Self::AlreadyInUse => write!(f, "port is already in use"),
        }
    }
}
```

### Step 4: Add Help Text Method
```rust
impl Error {
    /// Get actionable help text for this error
    pub fn help_text(&self) -> String {
        match self {
            Error::InvalidPort { port, reason } => match reason {
                PortError::Privileged => format!(
                    "Port {} requires elevated privileges. Try:\n\
                     • Use a port above 1024\n\
                     • Run with sudo/admin rights\n\
                     • Set 'allow_privileged = true' in config (not recommended)",
                    port
                ),
                PortError::Reserved => format!(
                    "Port {} is typically used by another service. \
                     Consider using a different port to avoid conflicts.",
                    port
                ),
                _ => String::new(),
            },
            
            Error::RateLimiting { suggestion, .. } => suggestion.clone(),
            
            _ => String::new(),
        }
    }
}
```

### Step 5: Update mod.rs
```rust
// src/config/mod.rs
mod error;
pub use error::{Error, PortError};  // Export the new types

// Remove the old Error enum
```

## Testing

Create tests for:
1. Error construction
2. Display formatting
3. Help text generation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_port_error_display() {
        let err = Error::InvalidPort {
            port: 80,
            reason: PortError::Privileged,
        };
        assert!(err.to_string().contains("80"));
        assert!(err.to_string().contains("elevated privileges"));
    }
    
    #[test]
    fn test_help_text() {
        let err = Error::InvalidPort {
            port: 443,
            reason: PortError::Privileged,
        };
        let help = err.help_text();
        assert!(help.contains("Use a port above 1024"));
        assert!(help.contains("sudo"));
    }
}
```

## Deliverables

### Files Modified/Created:
1. `src/config/error.rs` - New file with enhanced error types
2. `src/config/mod.rs` - Updated to use new error module
3. Tests added to verify functionality

### Documentation:
Update `analysis/error-implementation.md` with:
- Final error variants implemented
- Rationale for included/excluded variants
- Examples of usage

## Success Criteria
- [ ] New error.rs file created with rich error variants
- [ ] Supporting enums (PortError, etc.) defined
- [ ] help_text() method implemented
- [ ] Old Error enum removed from mod.rs
- [ ] Compilation successful
- [ ] Tests pass

## Time Estimate
2 hours

## Dependencies
- A.0 (Audit Current Error Usage) - Need to know what variants to create
- A.1 (Design Error Variants) - Need the design decisions

## Notes
- Start with most common error patterns from analysis
- Don't over-engineer - can always add more variants later
- Consider using #[from] for standard library errors where appropriate
- Keep some generic variants for flexibility