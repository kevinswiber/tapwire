# Shadowcat Error and Module Structure Guide

## Overview
This document outlines the recommended error handling and module organization for the Shadowcat MCP proxy library, focusing on creating a clean public API while maintaining internal flexibility.

## Error Structure

### Top-Level Error Design
The `crate::Error` enum should mirror your public API operations, not internal module structure:

```rust
// src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Configuration or builder errors
    #[error("Configuration error")]
    Config(#[from] config::Error),
    
    /// Forward proxy operation errors
    #[error("Forward proxy error")]
    ForwardProxy(#[from] crate::proxy::forward::Error),
    
    /// Reverse proxy operation errors  
    #[error("Reverse proxy error")]
    ReverseProxy(#[from] crate::proxy::reverse::Error),
    
    /// Recording or replay errors
    #[error("Recording error")]
    Recording(#[from] crate::recorder::Error),
    
    /// Storage backend errors
    #[error("Storage error")]
    Storage(#[from] crate::storage::Error),
    
    /// Transport creation/operation errors
    #[error("Transport error")]
    Transport(#[from] crate::transport::Error),
    
    /// Shutdown or lifecycle errors
    #[error("Shutdown error")]
    Shutdown(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Internal Error Bubbling
Internal modules should have their own error types that bubble up through operation-specific errors:

```rust
// src/proxy/forward/error.rs
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Connection pool error")]
    Pool(#[from] crate::pool::Error),
    
    #[error("Transport error")]
    Transport(#[from] crate::transport::Error),
    
    #[error("Session error")]
    Session(#[from] crate::session::Error),
    
    #[error("Configuration error")]
    Config(#[from] crate::config::Error),
    
    // Forward proxy specific errors
    #[error("Client disconnected")]
    ClientDisconnected,
}

pub type Result<T> = std::result::Result<T, Error>;

// src/proxy/reverse/error.rs  
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Connection pool error")]
    Pool(#[from] crate::pool::Error),
    
    #[error("Configuration error")]
    Config(#[from] crate::config::Error),
    
    #[error("Upstream error: {0}")]
    Upstream(String),
    
    #[error("Load balancing error")]
    LoadBalancing(String),
    
    // Reverse proxy specific errors
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Error Flow Example
```
pool::Error -> forward::Error::Pool -> crate::Error::ForwardProxy
             ↘ reverse::Error::Pool -> crate::Error::ReverseProxy
```

## Module Structure

### Directory Layout
Keep the current proxy structure with forward and reverse as submodules:

```
src/
├── api.rs              # High-level Shadowcat API
├── lib.rs              # Module exports and re-exports
├── error.rs            # Top-level Error enum
├── proxy/
│   ├── mod.rs          # Proxy module organization
│   ├── forward/
│   │   ├── mod.rs
│   │   ├── builder.rs  # ForwardProxyBuilder
│   │   ├── error.rs    # Forward-specific errors
│   │   └── ...
│   ├── reverse/
│   │   ├── mod.rs
│   │   ├── builder.rs  # ReverseProxyBuilder
│   │   ├── error.rs    # Reverse-specific errors
│   │   └── ...
│   ├── circuit_breaker.rs  # Shared utilities
│   ├── health_checker.rs
│   └── load_balancer.rs
├── transport/
├── recorder/
├── session/
└── ... (other modules)
```

### src/lib.rs - Public API Organization

```rust
//! Shadowcat MCP Proxy Library
//! 
//! # Quick Start
//! ```
//! use shadowcat::Shadowcat;
//! 
//! #[tokio::main]
//! async fn main() -> shadowcat::Result<()> {
//!     let sc = Shadowcat::production();
//!     sc.forward_stdio(vec!["mcp-server".into()], None).await?;
//!     Ok(())
//! }
//! ```
//! 
//! # Advanced Usage
//! For fine-grained control, access modules directly:
//! ```
//! use shadowcat::proxy::forward::ForwardProxyBuilder;
//! ```

// ===== Primary High-Level API =====
// Most users should start here
pub use api::{
    Shadowcat, 
    ShadowcatBuilder,
    ForwardProxyHandle,
    ReverseProxyHandle,
    RecordingHandle,
    ReplayHandle,
};

// ===== Core Types =====
pub use error::{Error, Result};

// ===== Convenience Re-exports =====
// Direct access to commonly used types
pub use proxy::forward::{ForwardProxy, ForwardProxyBuilder};
pub use proxy::reverse::{ReverseProxy, ReverseProxyBuilder};

// ===== Public Modules =====
// Expose modules for advanced users
pub mod api;
pub mod config;
pub mod error;

// Proxy module with submodules
pub mod proxy {
    pub mod forward {
        pub use crate::proxy::forward::{
            ForwardProxy,
            ForwardProxyBuilder,
            Error,
            Result,
        };
    }
    
    pub mod reverse {
        pub use crate::proxy::reverse::{
            ReverseProxy,
            ReverseProxyBuilder,  
            Error,
            Result,
        };
    }
    
    // Optionally expose shared utilities
    pub mod resilience {
        pub use crate::proxy::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
        pub use crate::proxy::health_checker::{HealthChecker, HealthCheckConfig};
    }
}

pub mod transport {
    pub use crate::transport::{
        Transport,
        IncomingTransport,
        OutgoingTransport,
        TransportFactory,
        Error,
        Result,
    };
}

pub mod recorder {
    pub use crate::recorder::{
        TapeRecorder,
        RecorderConfig,
        Error,
        Result,
    };
}

pub mod session {
    pub use crate::session::{
        SessionManager,
        SessionConfig,
        Session,
        Error,
        Result,
    };
}

// ===== Internal Modules =====
// Not exposed in public API
mod audit;
mod auth;
mod interceptor;
mod mcp;
mod pool;
mod process;
mod rate_limiting;
mod replay;
mod retry;
mod telemetry;

// ===== Optional CLI =====
#[cfg(feature = "cli")]
pub mod cli;
```

### src/proxy/mod.rs - Proxy Module Organization

```rust
//! Proxy implementations and utilities

pub mod forward;
pub mod reverse;

// Internal shared utilities
pub(crate) mod circuit_breaker;
pub(crate) mod health_checker;
pub(crate) mod load_balancer;

// Re-export key types at proxy module level for convenience
pub use forward::{ForwardProxy, ForwardProxyBuilder};
pub use reverse::{ReverseProxy, ReverseProxyBuilder};

// Optionally expose configuration types for utilities
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
pub use health_checker::{HealthChecker, HealthCheckConfig};
pub use load_balancer::{LoadBalancer, LoadBalancingStrategy};
```

### API Method to Error Mapping

Your `api.rs` methods should map to specific error variants:

```rust
impl Shadowcat {
    // Can return Error::Config or Error::ForwardProxy
    pub async fn forward_stdio(&self, ...) -> Result<()> { ... }
    
    // Can return Error::Config or Error::ForwardProxy
    pub async fn forward_http(&self, ...) -> Result<ForwardProxyHandle> { ... }
    
    // Can return Error::Config or Error::ReverseProxy  
    pub async fn reverse_proxy(&self, ...) -> Result<ReverseProxyHandle> { ... }
    
    // Can return Error::Config or Error::Recording
    pub async fn record_stdio(&self, ...) -> Result<()> { ... }
    
    // Can return Error::Config or Error::Storage
    pub async fn configure_storage(&self, ...) -> Result<()> { ... }
    
    // Can return Error::Config (build-time errors only)
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> { ... }
}
```

## Usage Examples

### Simple API Usage
```rust
use shadowcat::Shadowcat;

// Users primarily interact with high-level API
let shadowcat = Shadowcat::production();
shadowcat.forward_stdio(vec!["server".into()], None).await?;
```

### Advanced Module Access
```rust
use shadowcat::proxy::forward::ForwardProxyBuilder;
use shadowcat::transport::TransportFactory;
use shadowcat::session::SessionManager;

// Power users can build custom configurations
let proxy = ForwardProxyBuilder::new()
    .with_custom_interceptor(my_interceptor)
    .with_session_manager(custom_manager)
    .build()
    .await?;
```

### Error Handling
```rust
use shadowcat::{Shadowcat, Error};

match shadowcat.reverse_proxy(...).await {
    Ok(handle) => println!("Proxy started on {}", handle.addr()),
    Err(Error::ReverseProxy(e)) => eprintln!("Reverse proxy error: {}", e),
    Err(Error::Config(e)) => eprintln!("Configuration error: {}", e),
    Err(e) => eprintln!("Unexpected error: {}", e),
}
```

## Key Principles

1. **API-Driven Errors**: Top-level errors match public API operations, not internal structure
2. **Progressive Disclosure**: Simple high-level API with module access for advanced users
3. **Semantic Grouping**: Keep forward/reverse under proxy/ since they ARE proxies
4. **Internal Flexibility**: Hide implementation modules (pool, mcp, telemetry) from public API
5. **Ergonomic Access**: Provide convenient re-exports while maintaining logical structure
6. **Error Context**: Preserve error context by bubbling through operation-specific types

## Benefits

- **Easy Onboarding**: New users start with `Shadowcat::new()`
- **Power When Needed**: Advanced users access `proxy::forward::ForwardProxyBuilder`
- **Stable API**: High-level API rarely changes even with internal refactoring
- **Clear Errors**: Users understand errors in context of what they were doing
- **Good Documentation**: Tutorial-style docs for `Shadowcat`, reference docs for modules