# Current Error Patterns Analysis

## Summary Statistics
- **Total `Error::Invalid` uses**: 68
- **Total `Error::MissingField` uses**: 2
- **Total `Error::ParseError` uses**: 1
- **Files with most errors**: 
  - `validator.rs`: ~60 errors
  - `proxy.rs`: ~6 errors
  - `reverse_proxy.rs`: ~2 errors

## Common Error Categories

### 1. Port/Address Validation (8 occurrences)
```rust
// Current patterns
Error::Invalid(format!("Invalid server bind address '{}': missing host", self.server.bind))
Error::Invalid(format!("Invalid port in server bind address '{}': {e}", self.server.bind))
Error::Invalid("Port 80 (HTTP) requires elevated privileges or may conflict with existing services")

// Lost context:
- Specific port number as u16
- Type of port error (privileged, out of range, reserved)
- System requirements
```

### 2. Timeout/Duration Validation (12 occurrences)
```rust
// Current patterns
Error::Invalid("Server shutdown timeout must be greater than 0. Example: shutdown_timeout = 30")
Error::Invalid("Proxy request timeout must be greater than 0. Example: request_timeout = 30")
Error::Invalid(format!("Connect timeout ({}) should not exceed request timeout ({})", ...))

// Lost context:
- Actual timeout values
- Relationship between timeouts
- Performance implications
```

### 3. Resource Limits (10 occurrences)
```rust
// Current patterns
Error::Invalid("Server workers must be greater than 0")
Error::Invalid("Server workers must not exceed 1024")
Error::Invalid("Max connections must be greater than 0 when connection pooling is enabled")
Error::Invalid("Buffer size must be greater than 0")

// Lost context:
- Requested vs allowed values
- System capabilities (CPU count, memory)
- Performance impact
```

### 4. Rate Limiting Configuration (6 occurrences)
```rust
// Current patterns
Error::Invalid("Rate limit RPM must be greater than 0 when rate limiting is enabled")
Error::Invalid("Rate limit must be between 1 and 1,000,000 requests per minute")

// Lost context:
- Current RPM and burst values
- Relationship between settings
- Suggestions for typical values
```

### 5. Session/Timeout Configuration (8 occurrences)
```rust
// Current patterns
Error::Invalid("Session timeout must be greater than 0")
Error::Invalid("Session idle timeout should not exceed session timeout")
Error::Invalid("Max sessions must be greater than 0")

// Lost context:
- Timeout relationships
- Memory implications of session limits
```

### 6. Recording/Storage Configuration (4 occurrences)
```rust
// Current patterns
Error::Invalid("Max recording size must be greater than 0")
Error::Invalid("Storage directory path must not be empty")

// Lost context:
- Storage requirements
- Disk space availability
```

### 7. TLS/Security Configuration (3 occurrences)
```rust
// Current patterns
Error::Invalid("TLS cert path must not be empty when TLS is enabled")
Error::Invalid("TLS key path must not be empty when TLS is enabled")

// Lost context:
- File existence checks
- Permission requirements
```

### 8. Compatibility/Relationship Errors (8 occurrences)
```rust
// Current patterns
Error::Invalid("Connect timeout should not exceed request timeout")
Error::Invalid("Idle timeout should not exceed session timeout")
Error::Invalid("Min connections cannot exceed max connections")

// Lost context:
- The actual conflicting values
- Why they conflict
- How to resolve
```

## Patterns by Frequency

### High-Priority (>5 occurrences each)
1. **Timeout validation** - 12 occurrences
2. **Resource limits** - 10 occurrences  
3. **Port/Address parsing** - 8 occurrences
4. **Compatibility checks** - 8 occurrences
5. **Session configuration** - 8 occurrences

### Medium-Priority (3-5 occurrences)
1. **Rate limiting** - 6 occurrences
2. **Recording/Storage** - 4 occurrences
3. **TLS configuration** - 3 occurrences

### Low-Priority (1-2 occurrences)
1. **OAuth configuration** - 2 occurrences
2. **Metrics configuration** - 2 occurrences

## Recommended Error Variants

Based on the analysis, we should create these specific error variants:

### Essential Variants (Phase 1)
```rust
pub enum Error {
    #[error("Invalid port {port}: {reason}")]
    InvalidPort {
        port: u16,
        reason: PortError,
    },
    
    #[error("Invalid address '{addr}': {reason}")]
    InvalidAddress {
        addr: String,
        reason: String,
    },
    
    #[error("Invalid timeout: {message}")]
    InvalidTimeout {
        message: String,
        value: u64,
        suggestion: String,
    },
    
    #[error("Resource limit exceeded: {resource}")]
    ResourceLimit {
        resource: ResourceType,
        requested: usize,
        limit: usize,
        suggestion: String,
    },
    
    #[error("Rate limiting misconfigured: {reason}")]
    RateLimiting {
        reason: String,
        rpm: Option<u32>,
        burst: Option<u32>,
        suggestion: String,
    },
    
    #[error("Incompatible settings: {message}")]
    Incompatible {
        message: String,
        conflicts: Vec<String>,
        resolution: String,
    },
}
```

### Supporting Enums
```rust
pub enum PortError {
    Privileged,      // 0-1023
    OutOfRange,      // > 65535
    Reserved,        // Well-known ports
    AlreadyInUse,    // Runtime check
}

pub enum ResourceType {
    Workers,
    Connections,
    BufferSize,
    Sessions,
    RecordingSize,
}
```

### Keep Generic (for flexibility)
```rust
    #[error("Configuration invalid: {0}")]
    Invalid(String),  // For edge cases
    
    #[error("Missing required field: {field} in section {section}")]
    MissingField { field: String, section: String },
```

## Key Insights

1. **Most errors are range/limit validations** - These would benefit greatly from showing the actual values and valid ranges

2. **Many errors involve relationships** - Timeouts that must be ordered, limits that must be compatible

3. **Examples are already being provided** - Many error messages include "Example: foo = 30", showing users want guidance

4. **Performance warnings exist** - Some validations log warnings instead of errors (e.g., high connect timeout)

5. **Context is consistently lost** - Actual values, system limits, and suggestions are formatted into strings

## Migration Strategy

1. Start with the high-frequency patterns (timeouts, resources, ports)
2. Keep `Invalid(String)` for uncommon cases initially
3. Add help_text() for the most confusing errors
4. Consider making some errors into warnings (performance-related)