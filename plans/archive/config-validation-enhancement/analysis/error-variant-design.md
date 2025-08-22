# Error Variant Design

## Design Principles

1. **No redundant "Error" suffix** - Use `InvalidPort`, not `InvalidPortError`
2. **Preserve context** - Include actual values, not just messages
3. **Actionable guidance** - Each error should suggest how to fix it
4. **Progressive enhancement** - Start with common cases, expand as needed

## Final Error Enum Design

```rust
use thiserror::Error;
use std::path::PathBuf;

#[derive(Debug, Error)]
pub enum Error {
    // ===== Network/Address Errors =====
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
    
    // ===== Validation Errors =====
    #[error("{field} value {value} is out of range: {reason}")]
    OutOfRange {
        field: String,
        value: String,  // String to handle different numeric types
        reason: String,
        valid_range: Option<String>,  // e.g., "1-65535", "0.0-1.0"
    },
    
    #[error("Invalid timeout configuration: {reason}")]
    InvalidTimeout {
        reason: String,
        suggestion: String,
    },
    
    // ===== Resource Errors =====
    #[error("Resource limit exceeded for {resource}: requested {requested}, limit {limit}")]
    ResourceLimit {
        resource: ResourceType,
        requested: usize,
        limit: usize,
        suggestion: String,
    },
    
    // ===== Configuration Logic Errors =====
    #[error("Incompatible configuration: {message}")]
    Incompatible {
        message: String,
        conflicts: Vec<String>,
        resolution: String,
    },
    
    #[error("Rate limiting misconfigured: {reason}")]
    RateLimiting {
        reason: String,
        rpm: Option<u32>,
        burst: Option<u32>,
        suggestion: String,
    },
    
    // ===== File/Path Errors =====
    #[error("File not found: {path}")]
    FileNotFound {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    #[error("Missing required field: {field} in section {section}")]
    MissingField {
        field: String,
        section: String,
    },
    
    // ===== Parsing Errors =====
    #[error("Failed to parse configuration: {reason}")]
    ParseError {
        reason: String,
        line: Option<usize>,
        column: Option<usize>,
    },
    
    #[error("Address parse error")]
    AddressParse(#[from] std::net::AddrParseError),
    
    // ===== Generic Fallback =====
    #[error("Configuration invalid: {0}")]
    Invalid(String),  // Keep for edge cases and gradual migration
}

// ===== Supporting Enums =====

#[derive(Debug, Clone, Copy)]
pub enum PortError {
    Privileged,      // Ports 0-1023 need root/admin
    OutOfRange,      // > 65535
    Reserved,        // Well-known service ports
    AlreadyInUse,    // Port is already bound (runtime check)
}

impl std::fmt::Display for PortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Privileged => write!(f, "port requires elevated privileges"),
            Self::OutOfRange => write!(f, "port number out of valid range (1-65535)"),
            Self::Reserved => write!(f, "port is reserved for well-known service"),
            Self::AlreadyInUse => write!(f, "port is already in use"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ResourceType {
    Workers,
    Connections,
    BufferSize,
    Sessions,
    RecordingSize,
    FileDescriptors,
    Memory,
}

impl std::fmt::Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Workers => write!(f, "worker threads"),
            Self::Connections => write!(f, "connections"),
            Self::BufferSize => write!(f, "buffer size"),
            Self::Sessions => write!(f, "sessions"),
            Self::RecordingSize => write!(f, "recording size"),
            Self::FileDescriptors => write!(f, "file descriptors"),
            Self::Memory => write!(f, "memory"),
        }
    }
}
```

## Help Text Implementation

```rust
impl Error {
    /// Get actionable help text for this error
    pub fn help_text(&self) -> String {
        match self {
            Error::InvalidPort { port, reason } => match reason {
                PortError::Privileged => format!(
                    "Port {} requires elevated privileges. Try one of:\n\
                     • Use a port above 1024 (recommended)\n\
                     • Run the application with sudo/admin rights\n\
                     • Configure port forwarding from a high port to {}",
                    port, port
                ),
                PortError::Reserved => format!(
                    "Port {} is commonly used by another service. \
                     Consider using:\n\
                     • Port 8080 or 8081 for HTTP\n\
                     • Port 8443 for HTTPS\n\
                     • Any port above 10000 for custom services",
                    port
                ),
                PortError::OutOfRange => {
                    "Valid port range is 1-65535. Common choices:\n\
                     • 8080 for development HTTP\n\
                     • 3000-3999 for Node.js style apps\n\
                     • 50000+ for high ports".to_string()
                },
                PortError::AlreadyInUse => format!(
                    "Port {} is already in use. Try:\n\
                     • Stop the other process: lsof -i :{} or netstat -an | grep {}\n\
                     • Use a different port\n\
                     • Use port 0 to let the OS assign an available port",
                    port, port, port
                ),
            },
            
            Error::ResourceLimit { resource, requested, limit, suggestion } => {
                format!(
                    "Cannot allocate {} {} (system limit: {}).\n\
                     {}\n\
                     To check system limits: ulimit -a",
                    requested, resource, limit, suggestion
                )
            },
            
            Error::RateLimiting { suggestion, .. } => suggestion.clone(),
            
            Error::Incompatible { resolution, .. } => resolution.clone(),
            
            Error::InvalidTimeout { suggestion, .. } => suggestion.clone(),
            
            Error::OutOfRange { field, valid_range, .. } => {
                if let Some(range) = valid_range {
                    format!("{} must be within {}", field, range)
                } else {
                    String::new()
                }
            },
            
            _ => String::new(),
        }
    }
    
    /// Check if this error is a warning that could be bypassed
    pub fn is_warning(&self) -> bool {
        // Future: Some errors might be demoted to warnings
        false
    }
}
```

## Migration Examples

### Before
```rust
return Err(Error::Invalid(format!(
    "Invalid port in server bind address '{}': {}",
    self.server.bind, e
)));
```

### After
```rust
return Err(Error::InvalidPort {
    port: parsed_port,
    reason: if parsed_port < 1024 { 
        PortError::Privileged 
    } else { 
        PortError::OutOfRange 
    },
});
```

### Before
```rust
return Err(Error::Invalid(
    "Rate limit RPM must be greater than 0 when rate limiting is enabled. \
     Example: rate_limit_rpm = 600".to_string()
));
```

### After
```rust
return Err(Error::RateLimiting {
    reason: "RPM must be greater than 0 when rate limiting is enabled".to_string(),
    rpm: Some(self.proxy.rate_limit_rpm),
    burst: Some(self.proxy.rate_limit_burst),
    suggestion: "Common values: 600 RPM (10/sec) for APIs, 6000 RPM (100/sec) for web apps".to_string(),
});
```

## Implementation Priority

### Phase 1 - Core Variants (Must Have)
1. `InvalidPort` - Very common, needs specific handling
2. `InvalidAddress` - Parse errors need context  
3. `OutOfRange` - Covers many validation errors
4. `ResourceLimit` - Important for production
5. `Incompatible` - Relationship errors

### Phase 2 - Enhanced Variants (Should Have)
1. `RateLimiting` - Complex configuration
2. `InvalidTimeout` - Timeout relationships
3. `FileNotFound` - Better than generic IO error

### Phase 3 - Nice to Have
1. Performance warnings
2. Suggestions for all errors
3. Severity levels

## Notes

- Keep `Invalid(String)` for gradual migration and edge cases
- Don't create too many specific variants initially
- Focus on the errors users actually hit frequently
- Make help_text() genuinely helpful with concrete examples