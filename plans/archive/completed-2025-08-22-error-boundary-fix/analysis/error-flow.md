# Error Flow Analysis

## Current Error Flows (Problematic)

### Direct Violations
These modules bypass their error hierarchy and construct `crate::Error` directly:

```
auth::gateway.rs:
    OAuth error ──> auth::Error::OAuth ──> crate::Error::Auth ❌
                                            (constructs directly)

auth::middleware.rs:
    Auth error ──> crate::Error::Auth ❌
                   (imports and uses crate::Error)

proxy::reverse::upstream::stdio.rs:
    IO error ──> transport::Error ──> crate::Error::Transport ❌
                                       (maps directly)

pool (traits):
    Operations ──> crate::Result<T> ❌
                   (uses crate::Result in trait definitions)
```

### Import Violations
Modules importing `crate::Result` as aliases:

```
auth::policy.rs:
    use crate::Result as ShadowcatResult ❌

auth::rate_limit.rs:
    use crate::Result as ShadowcatResult ❌

audit::logger.rs:
    use crate::Result as ShadowcatResult ❌
```

## Expected Error Flows (Target)

### Proper Error Propagation Chains

#### Forward Proxy Flow
```
io::Error 
    ↓ [#from]
transport::Error 
    ↓ [#from]
pool::Error 
    ↓ [#from]
proxy::forward::Error 
    ↓ [#from]
crate::Error::ForwardProxy ✅
```

#### Reverse Proxy Flow
```
io::Error 
    ↓ [#from]
transport::Error 
    ↓ [#from]
auth::Error 
    ↓ [#from]
proxy::reverse::Error 
    ↓ [#from]
crate::Error::ReverseProxy ✅
```

#### Session Management Flow
```
io::Error
    ↓ [#from]
transport::Error
    ↓ [#from]
session::Error
    ↓ [#from]
crate::Error::Session ✅
```

#### Recording Flow
```
io::Error
    ↓ [#from]
mcp::Error
    ↓ [#from]
recorder::Error
    ↓ [#from]
crate::Error::Recorder ✅
```

## Error Conversion Patterns

### Current Anti-Patterns ❌

```rust
// BAD: Direct construction of crate::Error
return Err(crate::Error::Auth(auth::Error::OAuth(msg)));

// BAD: Using crate::Result in module
pub async fn do_something() -> crate::Result<()> { }

// BAD: Importing crate::Result with alias
use crate::Result as ShadowcatResult;
```

### Target Patterns ✅

```rust
// GOOD: Use module error
return Err(auth::Error::OAuth(msg));
// Let #[from] handle conversion at boundaries

// GOOD: Use module Result
pub async fn do_something() -> Result<()> { }
// Where Result = std::result::Result<T, auth::Error>

// GOOD: No need for aliases
use super::Result; // or just Result if in same module
```

## Module-Specific Error Flows

### Modules Missing Error Types
These need Error types created:

```
mcp:
    Current: Returns crate::Result directly
    Target:  mcp::Error -> (used by many) -> crate::Error

telemetry:
    Current: Uses crate types
    Target:  telemetry::Error -> transport::Error -> crate::Error

process:
    Current: Uses crate types
    Target:  process::Error -> transport::Error -> crate::Error

audit:
    Current: Uses crate::Result alias
    Target:  audit::Error -> crate::Error::Audit
```

### Modules With Broken Flows
These have Error types but bypass them:

```
auth:
    Current: Has auth::Error but constructs crate::Error::Auth
    Fix:     Always return auth::Error, let conversion happen at boundary

pool:
    Current: Has pool::Error but traits use crate::Result
    Fix:     Use associated types in traits or generic bounds

proxy::reverse:
    Current: Has Error but maps to crate::Error::Transport
    Fix:     Return proxy::reverse::Error::Transport instead
```

## Trait Error Handling

### Current Problem
```rust
// pool/traits.rs
trait PooledResource {
    async fn close(&mut self) -> crate::Result<()>;  // ❌
}
```

### Solution Options

#### Option 1: Associated Type
```rust
trait PooledResource {
    type Error;
    async fn close(&mut self) -> Result<(), Self::Error>;  // ✅
}
```

#### Option 2: Generic Error
```rust
trait PooledResource<E> {
    async fn close(&mut self) -> Result<(), E>;  // ✅
}
```

#### Option 3: Module Error
```rust
trait PooledResource {
    async fn close(&mut self) -> pool::Result<()>;  // ✅
}
```

## Boundary Conversion Points

These are the ONLY places where conversion to `crate::Error` should happen:

1. **api.rs** - Public API functions
2. **main.rs** - Binary entry points
3. **Public trait implementations** - When required by public traits

Example:
```rust
// In api.rs (public boundary)
impl Shadowcat {
    pub async fn forward_proxy(&self) -> crate::Result<()> {
        proxy::forward::run()
            .await
            .map_err(crate::Error::from)  // ✅ Conversion at boundary
    }
}

// In proxy/forward/mod.rs (internal)
pub async fn run() -> Result<()> {  // ✅ Module Result
    // ... implementation
}
```

## Testing Error Flows

After migration, test error propagation:

```rust
#[test]
fn test_error_propagation() {
    // Create low-level error
    let io_err = io::Error::new(io::ErrorKind::NotFound, "test");
    
    // Convert through chain
    let transport_err = transport::Error::from(io_err);
    let session_err = session::Error::from(transport_err);
    let crate_err = crate::Error::from(session_err);
    
    // Verify proper chain
    assert!(matches!(crate_err, crate::Error::Session(_)));
}
```

## Summary

### Key Problems
1. Direct construction of `crate::Error` variants (4 instances)
2. Using `crate::Result` in module functions (13 instances)
3. Importing `crate::Result` with aliases (3 instances)
4. Missing Error types in core modules (5 modules)

### Migration Focus
1. Stop constructing `crate::Error` directly
2. Use module-local Result types
3. Let `#[from]` handle conversions
4. Only convert at API boundaries