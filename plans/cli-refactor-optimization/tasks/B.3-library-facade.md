# Task B.3: Create Library Facade

## Objective
Design and implement a high-level `Shadowcat` facade that provides a simple, intuitive entry point for library users, hiding complexity while exposing the full power of the proxy system through a clean API.

## Background
Currently, library users must:
- Understand internal module structure
- Know which types to import
- Manage dependencies between components
- Configure multiple objects separately

The facade pattern will provide:
- Single entry point (`Shadowcat` struct)
- Intuitive method names
- Sensible defaults
- Progressive disclosure of complexity

## Key Questions to Answer
1. What should be the primary use cases for the facade?
2. How much should the facade hide vs expose?
3. Should the facade be stateful or stateless?
4. How do we handle different proxy modes?

## Step-by-Step Process

### 1. Design the Facade API
```rust
// src/shadowcat.rs (new file)
//! High-level API for Shadowcat proxy operations
//!
//! This module provides the main entry point for library users.

use crate::{
    config::{ProxyConfig, ShadowcatConfig},
    proxy::{ForwardProxy, ReverseProxy, ForwardProxyBuilder, ReverseProxyBuilder},
    transport::{Transport, StdioTransportBuilder, HttpTransportBuilder},
    session::SessionManager,
    recorder::Recorder,
    Result,
};

/// Main entry point for Shadowcat library
/// 
/// # Examples
/// 
/// ```rust
/// use shadowcat::Shadowcat;
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     // Simple forward proxy
///     let proxy = Shadowcat::new()
///         .forward_stdio_to_http("mcp-server", "https://api.example.com")
///         .await?;
///     
///     proxy.wait().await?;
///     Ok(())
/// }
/// ```
pub struct Shadowcat {
    config: ShadowcatConfig,
    session_manager: Option<Arc<SessionManager>>,
}

impl Shadowcat {
    /// Create a new Shadowcat instance with default configuration
    pub fn new() -> Self {
        Self {
            config: ShadowcatConfig::default(),
            session_manager: None,
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: ShadowcatConfig) -> Self {
        Self {
            config,
            session_manager: None,
        }
    }
    
    /// Load configuration from file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let config = ShadowcatConfig::from_file(path)?;
        Ok(Self::with_config(config))
    }
    
    /// Use development-optimized settings
    pub fn development() -> Self {
        Self::with_config(ShadowcatConfig::development())
    }
    
    /// Use production-optimized settings
    pub fn production() -> Self {
        Self::with_config(ShadowcatConfig::production())
    }
}
```

### 2. Add Convenience Methods
```rust
// src/shadowcat.rs (continued)
impl Shadowcat {
    // ===== Forward Proxy Convenience Methods =====
    
    /// Create a forward proxy from stdio to HTTP
    pub async fn forward_stdio_to_http(
        self,
        command: impl Into<String>,
        target: impl Into<String>,
    ) -> Result<ForwardProxyHandle> {
        let client = StdioTransportBuilder::new()
            .command(command)
            .connect()
            .await?;
        
        let server = HttpTransportBuilder::new()
            .url(target)
            .connect()
            .await?;
        
        self.forward(client, server).await
    }
    
    /// Create a forward proxy from HTTP to HTTP
    pub async fn forward_http_to_http(
        self,
        listen: impl Into<String>,
        target: impl Into<String>,
    ) -> Result<ForwardProxyHandle> {
        let client = HttpTransportBuilder::new()
            .url(listen)
            .as_server()
            .connect()
            .await?;
        
        let server = HttpTransportBuilder::new()
            .url(target)
            .connect()
            .await?;
        
        self.forward(client, server).await
    }
    
    /// Generic forward proxy
    pub async fn forward(
        self,
        client: impl Transport + 'static,
        server: impl Transport + 'static,
    ) -> Result<ForwardProxyHandle> {
        let proxy = self.forward_proxy_builder()
            .start(client, server)
            .await?;
        
        Ok(ForwardProxyHandle { proxy })
    }
    
    // ===== Reverse Proxy Convenience Methods =====
    
    /// Create a reverse proxy
    pub async fn reverse(
        self,
        bind: impl Into<SocketAddr>,
        upstream: impl Into<String>,
    ) -> Result<ReverseProxyHandle> {
        let proxy = self.reverse_proxy_builder()
            .bind(bind)
            .upstream(upstream)
            .start()
            .await?;
        
        Ok(ReverseProxyHandle { proxy })
    }
    
    // ===== Recording Methods =====
    
    /// Record a session
    pub async fn record<F, Fut>(
        self,
        output: impl Into<PathBuf>,
        f: F,
    ) -> Result<RecordingHandle>
    where
        F: FnOnce(Shadowcat) -> Fut,
        Fut: Future<Output = Result<()>>,
    {
        let recorder = Recorder::new(output.into());
        let shadowcat = self.with_recorder(recorder.clone());
        
        let handle = tokio::spawn(f(shadowcat));
        
        Ok(RecordingHandle {
            recorder,
            task: handle,
        })
    }
    
    /// Replay a recorded session
    pub async fn replay(
        self,
        tape: impl Into<PathBuf>,
        port: u16,
    ) -> Result<ReplayHandle> {
        let replayer = Replayer::new(tape.into());
        replayer.serve(port).await?;
        
        Ok(ReplayHandle { replayer })
    }
}
```

### 3. Add Builder Access Methods
```rust
// src/shadowcat.rs (continued)
impl Shadowcat {
    /// Get a forward proxy builder for advanced configuration
    pub fn forward_proxy_builder(&self) -> ForwardProxyBuilder {
        ForwardProxyBuilder::new()
            .with_config(self.config.proxy.clone())
            .session_manager(self.session_manager())
    }
    
    /// Get a reverse proxy builder for advanced configuration
    pub fn reverse_proxy_builder(&self) -> ReverseProxyBuilder {
        ReverseProxyBuilder::new()
            .with_config(self.config.proxy.clone())
            .session_manager(self.session_manager())
    }
    
    /// Get a stdio transport builder
    pub fn stdio_transport(&self) -> StdioTransportBuilder {
        StdioTransportBuilder::new()
    }
    
    /// Get an HTTP transport builder
    pub fn http_transport(&self) -> HttpTransportBuilder {
        HttpTransportBuilder::new()
    }
    
    /// Access the session manager
    fn session_manager(&self) -> Arc<SessionManager> {
        self.session_manager.get_or_insert_with(|| {
            Arc::new(SessionManager::from_config(&self.config.session))
        }).clone()
    }
}
```

### 4. Create Handle Types
```rust
// src/shadowcat.rs (continued)

/// Handle to a running forward proxy
pub struct ForwardProxyHandle {
    proxy: ForwardProxy,
}

impl ForwardProxyHandle {
    /// Wait for the proxy to complete
    pub async fn wait(self) -> Result<()> {
        self.proxy.wait().await
    }
    
    /// Shutdown the proxy gracefully
    pub async fn shutdown(self) -> Result<()> {
        self.proxy.shutdown().await
    }
    
    /// Get proxy statistics
    pub fn stats(&self) -> ProxyStats {
        self.proxy.stats()
    }
}

/// Handle to a running reverse proxy
pub struct ReverseProxyHandle {
    proxy: ReverseProxy,
}

impl ReverseProxyHandle {
    /// Get the listening address
    pub fn addr(&self) -> SocketAddr {
        self.proxy.addr()
    }
    
    /// Shutdown the proxy
    pub async fn shutdown(self) -> Result<()> {
        self.proxy.shutdown().await
    }
}

/// Handle to a recording session
pub struct RecordingHandle {
    recorder: Arc<Recorder>,
    task: JoinHandle<Result<()>>,
}

impl RecordingHandle {
    /// Stop recording and save the tape
    pub async fn stop(self) -> Result<PathBuf> {
        self.recorder.stop().await?;
        self.task.await??;
        Ok(self.recorder.output_path())
    }
}
```

### 5. Create Prelude Module
```rust
// src/prelude.rs (new file)
//! Common imports for Shadowcat users
//!
//! # Usage
//! ```rust
//! use shadowcat::prelude::*;
//! ```

pub use crate::{
    Shadowcat,
    Result,
    ShadowcatError,
    config::{ProxyConfig, ShadowcatConfig},
};

// Re-export commonly used types
pub use crate::transport::Transport;
pub use crate::interceptor::Interceptor;
```

### 6. Update lib.rs
```rust
// src/lib.rs (modify)
mod shadowcat;
pub mod prelude;

pub use shadowcat::{
    Shadowcat,
    ForwardProxyHandle,
    ReverseProxyHandle,
    RecordingHandle,
    ReplayHandle,
};

// Keep modules available for advanced users
pub mod config;
pub mod transport;
pub mod proxy;
pub mod session;
// ... other modules
```

### 7. Add Examples
```rust
// examples/simple_forward.rs
use shadowcat::Shadowcat;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simplest possible forward proxy
    let proxy = Shadowcat::development()
        .forward_stdio_to_http("mcp-server", "https://api.anthropic.com")
        .await?;
    
    println!("Proxy running, press Ctrl+C to stop");
    proxy.wait().await?;
    
    Ok(())
}

// examples/advanced.rs
use shadowcat::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Advanced configuration with builder
    let shadowcat = Shadowcat::production();
    
    let proxy = shadowcat
        .forward_proxy_builder()
        .rate_limiting(100)  // 100 requests per minute
        .interceptor(MyCustomInterceptor::new())
        .recording("session.tape")
        .start(client, server)
        .await?;
    
    proxy.wait().await?;
    Ok(())
}
```

### 8. Add Facade Tests
```rust
// tests/facade.rs
use shadowcat::Shadowcat;

#[tokio::test]
async fn test_simple_forward_proxy() {
    let result = Shadowcat::development()
        .forward_stdio_to_http("echo", "http://localhost:8080")
        .await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_config_loading() {
    let shadowcat = Shadowcat::from_file("test-config.toml").unwrap();
    assert_eq!(shadowcat.config.proxy.rate_limit_rpm, 500);
}

#[tokio::test]
async fn test_recording() {
    let handle = Shadowcat::new()
        .record("test.tape", |sc| async move {
            let proxy = sc.forward_stdio_to_http("echo", "http://localhost:8080").await?;
            // Do some work...
            proxy.shutdown().await?;
            Ok(())
        })
        .await
        .unwrap();
    
    let tape_path = handle.stop().await.unwrap();
    assert!(tape_path.exists());
}
```

## Expected Deliverables

### New Files
- `shadowcat/src/shadowcat.rs` - Main facade implementation
- `shadowcat/src/prelude.rs` - Common imports
- `shadowcat/examples/simple_forward.rs` - Simple example
- `shadowcat/examples/advanced.rs` - Advanced example
- `shadowcat/tests/facade.rs` - Facade tests

### Modified Files
- `shadowcat/src/lib.rs` - Export facade and prelude

### Documentation
- Full rustdoc comments on all public methods
- Examples in documentation
- README updated with new API

## Success Criteria Checklist
- [ ] Simple use cases require < 5 lines of code
- [ ] All proxy modes accessible via facade
- [ ] Progressive disclosure (simple → advanced)
- [ ] Examples demonstrate common patterns
- [ ] Facade methods well documented
- [ ] Prelude provides essential imports
- [ ] Tests cover main use cases

## Risk Assessment
- **Risk**: Facade becomes too complex
  - **Mitigation**: Keep simple things simple
  - **Mitigation**: Delegate to builders for complexity

- **Risk**: Facade limits flexibility
  - **Mitigation**: Expose builders for advanced use
  - **Mitigation**: Keep modules public for power users

## Duration Estimate
**3 hours**
- 1 hour: Design and implement facade
- 45 min: Add convenience methods
- 30 min: Create handle types
- 30 min: Write examples
- 15 min: Documentation

## Dependencies
- B.1: Builder patterns (facade uses builders)
- B.2: Graceful shutdown (for handle methods)

## Notes
- This is the primary API users will interact with
- Keep the common case simple
- Progressive disclosure is key
- Consider adding async builder pattern later

## Commands Reference
```bash
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Create facade module
touch src/shadowcat.rs
touch src/prelude.rs

# Run examples
cargo run --example simple_forward
cargo run --example advanced

# Check documentation
cargo doc --no-deps --open

# Test facade
cargo test facade
```