# Library Readiness Analysis

## Overview

This document assesses whether Shadowcat can be effectively used as a Rust library after the CLI refactor, examining API design, usability, and integration patterns.

## Public API Surface

### Current Exports (`lib.rs`)

```rust
// Core modules
pub mod audit;
pub mod auth;
pub mod cli;        // ⚠️ Should this be public?
pub mod config;
pub mod error;
pub mod interceptor;
pub mod mcp;
pub mod metrics;
pub mod protocol;
pub mod proxy;
pub mod rate_limiting;
pub mod recorder;
pub mod retry;
pub mod session;
pub mod transport;

// Convenience re-exports
pub use error::{Result, ShadowcatError};
pub use transport::{MessageDirection, ProtocolMessage, SessionId, Transport, TransportType};

// Prelude
pub mod prelude {
    pub use crate::error::{Result, ShadowcatError};
    pub use crate::transport::{
        MessageDirection, ProtocolMessage, SessionId, Transport, TransportType,
    };
}
```

### API Assessment

#### Strengths
1. **Clear Module Organization**: Each domain has its module
2. **Prelude Pattern**: Convenient imports for common usage
3. **Error Handling**: Unified error type with Result alias
4. **Core Types Exposed**: Essential types are re-exported

#### Weaknesses
1. **CLI Module Public**: Exposes CLI internals to library users
2. **Missing Builder APIs**: No ergonomic construction patterns
3. **Limited Documentation**: Public APIs lack comprehensive docs
4. **No Feature Flags**: Cannot exclude CLI dependencies

## Library Usage Patterns

### Current Usage Example

```rust
use shadowcat::prelude::*;
use shadowcat::proxy::ForwardProxy;
use shadowcat::transport::stdio::StdioTransport;

// ❌ Currently difficult - requires too much setup
async fn use_as_library() -> Result<()> {
    // Must manually create all components
    let session_config = /* complex setup */;
    let session_manager = Arc::new(SessionManager::with_config(session_config));
    
    // Must know internal details
    let mut cmd = Command::new("mcp-server");
    let transport = StdioTransport::new(cmd);
    
    // No convenient API
    let proxy = ForwardProxy::new()
        .with_session_manager(session_manager);
    
    proxy.start(transport, upstream).await?;
    Ok(())
}
```

### Desired Usage Pattern

```rust
use shadowcat::prelude::*;

// ✅ Should be this simple
async fn use_as_library() -> Result<()> {
    let proxy = shadowcat::proxy()
        .forward()
        .stdio_client("mcp-server")
        .http_upstream("https://api.example.com")
        .with_rate_limiting(100) // rpm
        .build()?;
    
    proxy.start().await?;
    Ok(())
}
```

## Dependency Analysis

### Current Dependencies

```toml
[dependencies]
# Core
tokio = { version = "1.35", features = ["full"] }
axum = "0.7"                    # ⚠️ Heavy for library users
clap = { version = "4", features = ["derive"] }  # ⚠️ CLI only

# Many more...
```

### Recommended Structure

```toml
[dependencies]
# Core only
tokio = { version = "1.35", features = ["rt", "net", "io-util"] }
serde = { version = "1.0", features = ["derive"] }

[dependencies.axum]
version = "0.7"
optional = true

[dependencies.clap]
version = "4"
features = ["derive"]
optional = true

[features]
default = []
cli = ["clap", "axum"]
http = ["axum"]
full = ["cli", "http"]
```

## API Design Issues

### 1. Configuration Complexity

**Current**: Users must understand internal config structures
```rust
let session_config = SessionConfig {
    timeout_duration: Duration::from_secs(300),
    max_sessions: Some(1000),
    cleanup_interval: Duration::from_secs(60),
    max_idle_time: Some(Duration::from_secs(300)),
    max_session_age: Some(Duration::from_secs(86400)),
    cleanup_on_shutdown: true,
    max_pending_per_session: 1000,
    max_pending_total: 10000,
    max_requests_per_second: 100,
};
```

**Desired**: Builder pattern with defaults
```rust
let config = SessionConfig::builder()
    .timeout_secs(300)
    .max_sessions(1000)
    .build();
```

### 2. Transport Creation

**Current**: Direct instantiation with internals exposed
**Desired**: Factory methods with clear interfaces

### 3. Async Lifecycle

**Current**: No clear lifecycle management
**Desired**: Proper start/stop/shutdown methods

## Integration Patterns

### As a Dependency

```toml
# In user's Cargo.toml
[dependencies]
shadowcat = { version = "0.1", default-features = false, features = ["http"] }
```

### Embedding in Application

```rust
struct MyApp {
    shadowcat_proxy: shadowcat::Proxy,
}

impl MyApp {
    async fn new() -> Result<Self> {
        let proxy = shadowcat::Proxy::builder()
            .mode(ProxyMode::Forward)
            .build()?;
        
        Ok(Self { 
            shadowcat_proxy: proxy 
        })
    }
}
```

## Documentation Requirements

### Missing Documentation

1. **Module-level docs**: Most modules lack overview documentation
2. **Example code**: No examples in doc comments
3. **Integration guides**: No guides for library usage
4. **API stability markers**: No indication of stable vs unstable APIs

### Recommended Documentation

```rust
//! # Shadowcat MCP Proxy Library
//! 
//! High-performance Model Context Protocol proxy with recording and interception.
//! 
//! ## Quick Start
//! 
//! ```rust
//! use shadowcat::prelude::*;
//! 
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let proxy = Proxy::forward()
//!         .stdio_to_http("https://mcp.example.com")
//!         .build()?;
//!     
//!     proxy.run().await
//! }
//! ```
```

## Testing as a Library

### Current Testing
- Tests focus on CLI functionality
- Limited library-level integration tests
- No examples directory

### Required Testing
```rust
#[cfg(test)]
mod library_tests {
    #[tokio::test]
    async fn test_library_basic_usage() {
        // Test that library can be used without CLI
    }
    
    #[tokio::test]
    async fn test_builder_patterns() {
        // Test all builder APIs
    }
}
```

## Breaking Changes Risk

### Current Risk Areas
1. **Public CLI module**: Changes to CLI affect library users
2. **Direct type exports**: Can't evolve internals without breaking changes
3. **Missing version policy**: No clear semantic versioning commitment

### Mitigation Strategies
1. Move CLI to separate crate
2. Use facade pattern for public API
3. Document stability guarantees

## Library Readiness Score

| Aspect | Score | Notes |
|--------|-------|-------|
| API Design | C+ | Needs builder patterns and simplification |
| Documentation | D | Minimal docs, no examples |
| Dependencies | C | CLI dependencies not optional |
| Usability | C | Too much complexity exposed |
| Stability | B- | Core is stable, API needs work |
| **Overall** | **C+** | Usable but not ergonomic |

## Recommendations

### Immediate (P0)
1. **Make CLI optional**: Use feature flags
2. **Add builder APIs**: For Proxy, Transport, Config
3. **Hide cli module**: Make it private or separate crate
4. **Add examples/**: Create library usage examples

### Short-term (P1)
1. **Documentation**: Add comprehensive API docs
2. **Integration tests**: Test as library dependency
3. **Prelude expansion**: Include more common types
4. **Config presets**: Provide common configurations

### Long-term (P2)
1. **Separate crates**: shadowcat-core, shadowcat-cli
2. **Stability markers**: Use #[stable] annotations
3. **API guidelines**: Follow Rust API guidelines
4. **Benchmarks**: Library-level performance tests

## Conclusion

Shadowcat is **marginally ready** for library use but needs significant improvements for good developer experience. The core functionality is solid, but the API design, documentation, and dependency management need work before it can be considered a production-ready library.

The refactor has created the foundation for library usage, but additional work is needed to make it truly library-first. The current state would frustrate library users due to complexity and lack of documentation.

**Recommended Path**: 
1. Implement P0 recommendations immediately
2. Release as 0.1.0-alpha with clear warnings
3. Iterate based on user feedback
4. Target 0.2.0 for stable library API