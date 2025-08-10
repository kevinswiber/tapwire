# Task B.1: Implement Builder Patterns

## Objective
Create comprehensive builder patterns for all major Shadowcat types (Transport, Proxy, SessionManager) to provide an ergonomic, type-safe API for library users and enable flexible configuration without exposing internal implementation details.

## Background
Currently, creating Shadowcat components requires:
- Direct struct instantiation with internal knowledge
- Multiple steps to configure and connect
- No validation until runtime
- Tight coupling to implementation details

Builder patterns will provide:
- Fluent API for configuration
- Compile-time and runtime validation
- Hidden implementation details
- Sensible defaults with override capability

## Key Questions to Answer
1. Which types need builders?
2. Should builders be consuming (owned) or borrowing?
3. How do we handle async operations in builders?
4. What validation should happen at build time?

## Step-by-Step Process

### 1. Analyze Current Construction Patterns
```bash
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Find direct construction
rg "::new\(" --type rust src/
rg "Arc::new\(" --type rust src/
rg "Transport::.*\{" --type rust src/

# Identify complex initialization
rg "let.*=.*;" -A 5 --type rust src/cli/ | grep -E "(transport|proxy|session)"
```

### 2. Create Transport Builders
```rust
// src/transport/builders.rs (new file)
use super::{Transport, StdioTransport, HttpTransport};
use std::path::PathBuf;
use std::collections::HashMap;

/// Builder for stdio-based transport
pub struct StdioTransportBuilder {
    command: Option<String>,
    args: Vec<String>,
    env: HashMap<String, String>,
    working_dir: Option<PathBuf>,
    inherit_env: bool,
}

impl StdioTransportBuilder {
    pub fn new() -> Self {
        Self {
            command: None,
            args: Vec::new(),
            env: HashMap::new(),
            working_dir: None,
            inherit_env: true,
        }
    }
    
    /// Set the command to execute
    pub fn command(mut self, cmd: impl Into<String>) -> Self {
        self.command = Some(cmd.into());
        self
    }
    
    /// Add command arguments
    pub fn args<I, S>(mut self, args: I) -> Self 
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }
    
    /// Set environment variable
    pub fn env(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.env.insert(key.into(), val.into());
        self
    }
    
    /// Set working directory
    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
    
    /// Connect and build the transport
    pub async fn connect(self) -> Result<StdioTransport> {
        let command = self.command
            .ok_or_else(|| ShadowcatError::Config(
                ConfigError::MissingField("command".to_string())
            ))?;
        
        let mut cmd = Command::new(&command);
        cmd.args(&self.args);
        
        if !self.inherit_env {
            cmd.env_clear();
        }
        
        for (key, val) in self.env {
            cmd.env(key, val);
        }
        
        if let Some(dir) = self.working_dir {
            cmd.current_dir(dir);
        }
        
        StdioTransport::from_command(cmd).await
    }
}

/// Builder for HTTP-based transport
pub struct HttpTransportBuilder {
    url: Option<String>,
    headers: HashMap<String, String>,
    timeout_secs: u64,
    retry_count: u32,
    tls_config: Option<TlsConfig>,
}

impl HttpTransportBuilder {
    pub fn new() -> Self {
        Self {
            url: None,
            headers: HashMap::new(),
            timeout_secs: 30,
            retry_count: 3,
            tls_config: None,
        }
    }
    
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
    
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
    
    pub fn timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }
    
    pub async fn connect(self) -> Result<HttpTransport> {
        let url = self.url
            .ok_or_else(|| ShadowcatError::Config(
                ConfigError::MissingField("url".to_string())
            ))?;
        
        HttpTransport::connect_with_config(
            &url,
            self.headers,
            self.timeout_secs,
            self.retry_count,
        ).await
    }
}
```

### 3. Create Proxy Builders
```rust
// src/proxy/builders.rs (new file)
use super::{ForwardProxy, ReverseProxy};
use crate::config::ProxyConfig;

pub struct ForwardProxyBuilder {
    config: ProxyConfig,
    session_manager: Option<Arc<SessionManager>>,
    rate_limiter: Option<Arc<MultiTierRateLimiter>>,
    interceptors: Vec<Box<dyn Interceptor>>,
    recorder: Option<Arc<Recorder>>,
}

impl ForwardProxyBuilder {
    pub fn new() -> Self {
        Self {
            config: ProxyConfig::default(),
            session_manager: None,
            rate_limiter: None,
            interceptors: Vec::new(),
            recorder: None,
        }
    }
    
    /// Use a specific configuration
    pub fn with_config(mut self, config: ProxyConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Apply sensible defaults for development
    pub fn development_defaults(self) -> Self {
        self.with_config(ProxyConfig {
            enable_rate_limit: false,
            session_timeout_secs: 3600,
            enable_recording: true,
            ..Default::default()
        })
    }
    
    /// Apply sensible defaults for production
    pub fn production_defaults(self) -> Self {
        self.with_config(ProxyConfig {
            enable_rate_limit: true,
            rate_limit_rpm: 1000,
            session_timeout_secs: 300,
            enable_recording: false,
            ..Default::default()
        })
    }
    
    /// Add an interceptor
    pub fn interceptor(mut self, interceptor: impl Interceptor + 'static) -> Self {
        self.interceptors.push(Box::new(interceptor));
        self
    }
    
    /// Enable recording to a file
    pub fn recording(mut self, path: impl Into<PathBuf>) -> Self {
        let recorder = Recorder::new(path.into());
        self.recorder = Some(Arc::new(recorder));
        self
    }
    
    /// Build and start the proxy
    pub async fn start(
        self,
        client: impl Transport + 'static,
        server: impl Transport + 'static,
    ) -> Result<ForwardProxy> {
        let session_manager = self.session_manager
            .unwrap_or_else(|| Arc::new(SessionManager::new()));
        
        let rate_limiter = if self.config.enable_rate_limit {
            Some(Arc::new(MultiTierRateLimiter::from_config(&self.config)?))
        } else {
            None
        };
        
        let proxy = ForwardProxy::new(
            session_manager,
            rate_limiter,
            self.interceptors,
            self.recorder,
        );
        
        proxy.run(client, server).await?;
        Ok(proxy)
    }
}

pub struct ReverseProxyBuilder {
    // Similar structure for reverse proxy
    bind_addr: Option<SocketAddr>,
    upstream_url: Option<String>,
    config: ProxyConfig,
    // ... other fields
}
```

### 4. Create SessionManager Builder
```rust
// src/session/builder.rs (new file)
pub struct SessionManagerBuilder {
    storage: SessionStorage,
    cleanup_interval_secs: u64,
    max_sessions: usize,
    session_timeout: Duration,
}

impl SessionManagerBuilder {
    pub fn new() -> Self {
        Self {
            storage: SessionStorage::Memory,
            cleanup_interval_secs: 60,
            max_sessions: 1000,
            session_timeout: Duration::from_secs(300),
        }
    }
    
    pub fn storage(mut self, storage: SessionStorage) -> Self {
        self.storage = storage;
        self
    }
    
    pub fn max_sessions(mut self, max: usize) -> Self {
        self.max_sessions = max;
        self
    }
    
    pub fn session_timeout(mut self, timeout: Duration) -> Self {
        self.session_timeout = timeout;
        self
    }
    
    pub fn build(self) -> Result<SessionManager> {
        SessionManager::with_config(
            self.storage,
            self.max_sessions,
            self.session_timeout,
            self.cleanup_interval_secs,
        )
    }
}
```

### 5. Update CLI to Use Builders
```rust
// src/cli/forward.rs
pub async fn execute(self) -> Result<()> {
    // Use builders instead of direct construction
    let client = StdioTransport::builder()
        .command(&command_args[0])
        .args(&command_args[1..])
        .connect()
        .await
        .context("Failed to create stdio transport")?;
    
    let server = HttpTransport::builder()
        .url(&self.target)
        .timeout(30)
        .connect()
        .await
        .context("Failed to connect to server")?;
    
    let proxy = ForwardProxy::builder()
        .with_config(self.to_config()?)
        .development_defaults()
        .start(client, server)
        .await
        .context("Failed to start forward proxy")?;
    
    proxy.wait().await
}
```

### 6. Add Builder Tests
```rust
// tests/builders.rs
#[tokio::test]
async fn test_stdio_transport_builder() {
    let transport = StdioTransport::builder()
        .command("echo")
        .args(vec!["test"])
        .env("TEST_VAR", "value")
        .connect()
        .await;
    
    assert!(transport.is_ok());
}

#[tokio::test]
async fn test_forward_proxy_builder() {
    let client = create_mock_transport();
    let server = create_mock_transport();
    
    let proxy = ForwardProxy::builder()
        .development_defaults()
        .interceptor(LoggingInterceptor::new())
        .recording("test.tape")
        .start(client, server)
        .await;
    
    assert!(proxy.is_ok());
}

#[test]
fn test_builder_validation() {
    // Missing required field
    let result = StdioTransport::builder()
        .connect()
        .await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("command"));
}
```

## Expected Deliverables

### New Files
- `shadowcat/src/transport/builders.rs` - Transport builders
- `shadowcat/src/proxy/builders.rs` - Proxy builders
- `shadowcat/src/session/builder.rs` - SessionManager builder
- `shadowcat/tests/builders.rs` - Builder tests

### Modified Files
- `shadowcat/src/transport/mod.rs` - Export builders
- `shadowcat/src/proxy/mod.rs` - Export builders
- `shadowcat/src/session/mod.rs` - Export builder
- `shadowcat/src/cli/*.rs` - Use builders instead of direct construction

### Documentation
```rust
// Example in lib.rs or README
//! ## Quick Start with Builders
//! 
//! ```rust
//! use shadowcat::prelude::*;
//! 
//! // Create a forward proxy with builders
//! let proxy = ForwardProxy::builder()
//!     .development_defaults()
//!     .recording("session.tape")
//!     .start(client, server)
//!     .await?;
//! ```
```

## Success Criteria Checklist
- [ ] All major types have builders
- [ ] Builders validate configuration
- [ ] Builders provide sensible defaults
- [ ] CLI uses builders exclusively
- [ ] Library users can construct without internals
- [ ] Builder API is documented with examples
- [ ] Tests cover builder validation

## Risk Assessment
- **Risk**: Builder API becomes too complex
  - **Mitigation**: Keep simple things simple
  - **Mitigation**: Provide preset methods (development_defaults)

- **Risk**: Breaking changes to internal APIs
  - **Mitigation**: Builders insulate users from internals
  - **Mitigation**: Mark internal methods as pub(crate)

## Duration Estimate
**6 hours**
- 1 hour: Design builder APIs
- 2 hours: Implement transport builders
- 1.5 hours: Implement proxy builders
- 1 hour: Update CLI to use builders
- 30 min: Write tests and documentation

## Dependencies
- Phase A complete (clean module boundaries)

## Notes
- This is the most important task for library usability
- Consider typestate pattern for compile-time validation
- Builders should be the primary construction method
- Keep direct constructors but mark as internal

## Commands Reference
```bash
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Create builder modules
touch src/transport/builders.rs
touch src/proxy/builders.rs
touch src/session/builder.rs

# Test builders
cargo test builders

# Check API documentation
cargo doc --no-deps --open

# Verify CLI still works with builders
cargo run -- forward stdio -- echo test
```