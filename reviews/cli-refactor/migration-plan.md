# Migration Plan: CLI Refactor Improvements

## Overview

This document provides a detailed migration plan to address the issues identified in the CLI refactor review, transforming Shadowcat into a production-ready library and CLI tool.

## Migration Goals

1. **Library-First Architecture**: Make Shadowcat usable as a Rust library
2. **Clean Module Boundaries**: Separate CLI from library concerns
3. **Improved Developer Experience**: Better APIs, documentation, and testing
4. **Production Readiness**: Error handling, monitoring, and stability

## Phase 1: Foundation (Days 1-3)

### Day 1: Critical Fixes

#### Task 1.1: Hide CLI Module (2 hours)
```rust
// src/lib.rs
-pub mod cli;
+#[cfg(feature = "cli")]
+pub(crate) mod cli;

// Cargo.toml
+[features]
+default = []
+cli = ["clap", "directories"]
```

**Verification**:
```bash
# Library builds without CLI
cargo build --no-default-features

# CLI still works
cargo build --features cli
```

#### Task 1.2: Fix Error Handling (3 hours)
```rust
// src/cli/forward.rs
-if command_args.is_empty() {
-    eprintln!("Error: No command specified");
-    exit(1);
-}
+if command_args.is_empty() {
+    return Err(ShadowcatError::Config(
+        ConfigError::MissingArgument("command".to_string())
+    ));
+}

// src/main.rs
#[tokio::main]
-async fn main() {
+async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    init_logging(cli.log_level, cli.verbose);
    
-   let result = match cli.command {
+   match cli.command {
        Commands::Forward(cmd) => cmd.execute().await,
        // ...
-   };
-   
-   if let Err(e) = result {
-       error!("Error: {}", e);
-       exit(1);
-   }
+   }?;
+   
+   Ok(())
}
```

#### Task 1.3: Extract ProxyConfig (3 hours)
```rust
// src/config/proxy.rs (new file)
pub struct ProxyConfig {
    // ... fields
}

pub struct ProxyConfigBuilder {
    config: ProxyConfig,
}

impl ProxyConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: ProxyConfig::default(),
        }
    }
    
    pub fn rate_limiting(mut self, enabled: bool, rpm: u32) -> Self {
        self.config.enable_rate_limit = enabled;
        self.config.rate_limit_rpm = rpm;
        self
    }
    
    pub fn build(self) -> Result<ProxyConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

// Move from cli/common.rs to library
// Update all references
```

### Day 2: Builder Patterns

#### Task 2.1: Transport Builders (4 hours)
```rust
// src/transport/builders.rs
pub struct StdioTransportBuilder {
    command: Option<Vec<String>>,
    env: HashMap<String, String>,
    working_dir: Option<PathBuf>,
}

impl StdioTransportBuilder {
    pub fn new() -> Self { ... }
    
    pub fn command(mut self, cmd: impl Into<String>, args: Vec<String>) -> Self {
        self.command = Some([vec![cmd.into()], args].concat());
        self
    }
    
    pub async fn connect(self) -> Result<StdioTransport> {
        let command = self.command
            .ok_or_else(|| ShadowcatError::Config(
                ConfigError::MissingField("command".to_string())
            ))?;
        
        let mut transport = StdioTransport::new_internal(command);
        transport.connect().await?;
        Ok(transport)
    }
}

// Usage in CLI
let transport = StdioTransport::builder()
    .command("mcp-server", vec!["--arg".to_string()])
    .connect()
    .await?;
```

#### Task 2.2: Proxy Builders (4 hours)
```rust
// src/proxy/builders.rs
pub struct ForwardProxyBuilder {
    session_manager: Option<Arc<SessionManager>>,
    rate_limiter: Option<Arc<MultiTierRateLimiter>>,
    interceptors: Vec<Box<dyn Interceptor>>,
}

impl ForwardProxyBuilder {
    pub fn new() -> Self { ... }
    
    pub fn with_defaults(self) -> Self {
        self.session_manager(SessionManager::new())
            .rate_limiting(RateLimitPreset::Development)
    }
    
    pub async fn start(
        self,
        client: impl Transport,
        server: impl Transport,
    ) -> Result<ForwardProxy> {
        let proxy = ForwardProxy {
            session_manager: self.session_manager
                .unwrap_or_else(|| Arc::new(SessionManager::new())),
            rate_limiter: self.rate_limiter,
            interceptors: self.interceptors,
        };
        
        proxy.run(client, server).await?;
        Ok(proxy)
    }
}
```

### Day 3: Testing Infrastructure

#### Task 3.1: Integration Test Framework (4 hours)
```rust
// tests/common/mod.rs
pub struct TestServer {
    port: u16,
    shutdown: Option<oneshot::Sender<()>>,
}

impl TestServer {
    pub async fn start() -> Self {
        // Start mock MCP server
    }
    
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

// tests/integration/forward_proxy.rs
#[tokio::test]
async fn test_forward_proxy_stdio_to_http() {
    let server = TestServer::start().await;
    
    let proxy = ForwardProxy::builder()
        .with_defaults()
        .build()?;
    
    let client = StdioTransport::builder()
        .command("echo", vec!["test"])
        .connect()
        .await?;
    
    let server = HttpTransport::builder()
        .url(server.url())
        .connect()
        .await?;
    
    proxy.start(client, server).await?;
    
    // Verify communication
    assert!(proxy.session_count() > 0);
}
```

#### Task 3.2: CLI Test Harness (2 hours)
```rust
// tests/cli/mod.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_forward_command_validation() {
    let mut cmd = Command::cargo_bin("shadowcat").unwrap();
    cmd.arg("forward")
        .arg("stdio");
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No command specified"));
}

#[test]
fn test_forward_command_success() {
    let mut cmd = Command::cargo_bin("shadowcat").unwrap();
    cmd.arg("forward")
        .arg("stdio")
        .arg("--")
        .arg("echo")
        .arg("test");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("initialize"));
}
```

## Phase 2: Library API (Days 4-6)

### Day 4: Public API Design

#### Task 4.1: Facade Pattern (4 hours)
```rust
// src/shadowcat.rs (new file)
/// Main entry point for library users
pub struct Shadowcat {
    config: ShadowcatConfig,
}

impl Shadowcat {
    /// Create a new Shadowcat instance with default configuration
    pub fn new() -> Self {
        Self {
            config: ShadowcatConfig::default(),
        }
    }
    
    /// Create a forward proxy
    pub fn forward_proxy(&self) -> ForwardProxyBuilder {
        ForwardProxyBuilder::new()
            .with_config(self.config.clone())
    }
    
    /// Create a reverse proxy
    pub fn reverse_proxy(&self) -> ReverseProxyBuilder {
        ReverseProxyBuilder::new()
            .with_config(self.config.clone())
    }
    
    /// Start recording a session
    pub fn recorder(&self) -> RecorderBuilder {
        RecorderBuilder::new()
            .with_config(self.config.clone())
    }
}

// Simple library usage
let shadowcat = Shadowcat::new();
let proxy = shadowcat
    .forward_proxy()
    .stdio_client("mcp-server")
    .http_server("https://api.example.com")
    .start()
    .await?;
```

#### Task 4.2: Configuration System (4 hours)
```rust
// src/config/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowcatConfig {
    pub proxy: ProxyConfig,
    pub transport: TransportConfig,
    pub recording: RecordingConfig,
    pub telemetry: TelemetryConfig,
}

impl ShadowcatConfig {
    /// Load from file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(Into::into)
    }
    
    /// Load from environment
    pub fn from_env() -> Result<Self> {
        envy::from_env().map_err(Into::into)
    }
    
    /// Merge multiple sources
    pub fn merged() -> Result<Self> {
        let mut config = Self::default();
        
        // Try file
        if let Ok(file_config) = Self::from_file("shadowcat.toml") {
            config.merge(file_config);
        }
        
        // Override with env
        if let Ok(env_config) = Self::from_env() {
            config.merge(env_config);
        }
        
        Ok(config)
    }
}
```

### Day 5: Documentation

#### Task 5.1: API Documentation (4 hours)
```rust
//! # Shadowcat
//! 
//! High-performance Model Context Protocol (MCP) proxy with recording and interception.
//! 
//! ## Quick Start
//! 
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! shadowcat = "0.1"
//! ```
//! 
//! Basic usage:
//! ```rust
//! use shadowcat::prelude::*;
//! 
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Create a forward proxy
//!     let proxy = shadowcat::forward_proxy()
//!         .stdio_client("mcp-server")
//!         .http_server("https://api.example.com")
//!         .with_recording("session.tape")
//!         .start()
//!         .await?;
//!     
//!     // Proxy runs until stopped
//!     proxy.wait().await?;
//!     
//!     Ok(())
//! }
//! ```

// Document all public types and functions
```

#### Task 5.2: Examples (2 hours)
```rust
// examples/forward_proxy.rs
//! Example: Forward proxy with recording

use shadowcat::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let proxy = shadowcat::forward_proxy()
        .stdio_client("mcp-server")
        .http_server("https://api.example.com")
        .with_rate_limiting(100) // 100 requests per minute
        .with_recording("session.tape")
        .start()
        .await?;
    
    println!("Proxy running on session {}", proxy.session_id());
    
    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    
    println!("Shutting down...");
    proxy.shutdown().await?;
    
    Ok(())
}
```

### Day 6: Graceful Shutdown

#### Task 6.1: Shutdown Handler (4 hours)
```rust
// src/shutdown.rs
pub struct ShutdownController {
    shutdown_tx: broadcast::Sender<()>,
    tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl ShutdownController {
    pub fn new() -> (Self, ShutdownReceiver) {
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        
        let controller = Self {
            shutdown_tx,
            tasks: Arc::new(Mutex::new(Vec::new())),
        };
        
        let receiver = ShutdownReceiver {
            shutdown_rx,
        };
        
        (controller, receiver)
    }
    
    pub fn register_task(&self, task: JoinHandle<()>) {
        self.tasks.lock().unwrap().push(task);
    }
    
    pub async fn shutdown(self) -> Result<()> {
        info!("Initiating graceful shutdown");
        
        // Send shutdown signal
        let _ = self.shutdown_tx.send(());
        
        // Wait for tasks with timeout
        let tasks = self.tasks.lock().unwrap().drain(..).collect::<Vec<_>>();
        
        let timeout = tokio::time::sleep(Duration::from_secs(30));
        tokio::select! {
            _ = Self::await_tasks(tasks) => {
                info!("All tasks completed");
            }
            _ = timeout => {
                warn!("Shutdown timeout, forcing termination");
            }
        }
        
        Ok(())
    }
}

// Integration in main
let (shutdown_controller, shutdown_receiver) = ShutdownController::new();

let proxy_task = tokio::spawn(async move {
    proxy.run_with_shutdown(shutdown_receiver).await
});

shutdown_controller.register_task(proxy_task);

tokio::select! {
    _ = tokio::signal::ctrl_c() => {
        shutdown_controller.shutdown().await?;
    }
}
```

## Phase 3: Polish (Days 7-9)

### Day 7: Performance Optimization

#### Task 7.1: Connection Pooling (4 hours)
```rust
// src/transport/pool.rs
pub struct TransportPool<T: Transport> {
    idle: Vec<T>,
    busy: HashMap<Uuid, T>,
    max_size: usize,
}

impl<T: Transport> TransportPool<T> {
    pub async fn get(&mut self) -> Result<PooledTransport<T>> {
        if let Some(transport) = self.idle.pop() {
            let id = Uuid::new_v4();
            self.busy.insert(id, transport);
            Ok(PooledTransport { id, pool: self })
        } else if self.busy.len() < self.max_size {
            // Create new transport
            let transport = T::connect().await?;
            let id = Uuid::new_v4();
            self.busy.insert(id, transport);
            Ok(PooledTransport { id, pool: self })
        } else {
            Err(ShadowcatError::PoolExhausted)
        }
    }
}
```

#### Task 7.2: Buffer Reuse (2 hours)
```rust
// src/buffer_pool.rs
thread_local! {
    static BUFFER_POOL: RefCell<Vec<Vec<u8>>> = RefCell::new(Vec::new());
}

pub fn take_buffer() -> Vec<u8> {
    BUFFER_POOL.with(|pool| {
        pool.borrow_mut().pop().unwrap_or_else(|| Vec::with_capacity(8192))
    })
}

pub fn return_buffer(mut buffer: Vec<u8>) {
    buffer.clear();
    BUFFER_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        if pool.len() < 32 { // Keep max 32 buffers
            pool.push(buffer);
        }
    });
}
```

### Day 8: Telemetry

#### Task 8.1: Metrics Integration (4 hours)
```rust
// src/telemetry/metrics.rs
use prometheus::{Counter, Histogram, Registry};

pub struct Metrics {
    pub requests_total: Counter,
    pub request_duration: Histogram,
    pub active_sessions: Gauge,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Result<Self> {
        let requests_total = Counter::new(
            "shadowcat_requests_total",
            "Total number of requests"
        )?;
        registry.register(Box::new(requests_total.clone()))?;
        
        // ... register other metrics
        
        Ok(Self {
            requests_total,
            request_duration,
            active_sessions,
        })
    }
}

// Integration
impl ForwardProxy {
    pub async fn handle_request(&self, req: Request) -> Result<Response> {
        let timer = self.metrics.request_duration.start_timer();
        self.metrics.requests_total.inc();
        
        let result = self.handle_request_internal(req).await;
        
        timer.observe_duration();
        result
    }
}
```

### Day 9: Final Testing

#### Task 9.1: Load Testing (4 hours)
```rust
// tests/load/proxy_load.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_forward_proxy(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("forward_proxy_throughput", |b| {
        b.to_async(&runtime).iter(|| async {
            let proxy = create_test_proxy().await;
            
            // Send 1000 requests
            let mut tasks = vec![];
            for _ in 0..1000 {
                let proxy = proxy.clone();
                tasks.push(tokio::spawn(async move {
                    proxy.handle_request(create_test_request()).await
                }));
            }
            
            futures::future::join_all(tasks).await;
        });
    });
}

criterion_group!(benches, bench_forward_proxy);
criterion_main!(benches);
```

## Phase 4: Release Preparation (Day 10)

### Task 10.1: Version Management
```toml
# Cargo.toml
[package]
name = "shadowcat"
version = "0.1.0-alpha.1"
edition = "2021"
license = "MIT OR Apache-2.0"
description = "High-performance MCP proxy with recording and interception"
repository = "https://github.com/tapwire/shadowcat"
keywords = ["mcp", "proxy", "recording", "model-context-protocol"]
categories = ["network-programming", "development-tools"]

[badges]
maintenance = { status = "actively-developed" }
```

### Task 10.2: CI/CD Pipeline
```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo publish --dry-run
      - run: cargo publish
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_TOKEN }}
```

## Rollback Plan

If issues arise during migration:

1. **Git Tags**: Tag before each phase
   ```bash
   git tag pre-phase-1
   git push --tags
   ```

2. **Feature Flags**: Keep old code behind flags
   ```rust
   #[cfg(feature = "legacy")]
   mod old_implementation;
   ```

3. **Gradual Rollout**: Deploy to staging first

## Success Metrics

### Phase 1 Success
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Library builds without CLI

### Phase 2 Success
- [ ] Example code runs
- [ ] API documentation complete
- [ ] Graceful shutdown works

### Phase 3 Success
- [ ] Performance benchmarks pass
- [ ] Metrics exported correctly
- [ ] Load tests successful

### Overall Success
- [ ] Published to crates.io
- [ ] GitHub release created
- [ ] Documentation deployed
- [ ] No critical bugs in first week

## Timeline Summary

| Day | Phase | Focus | Deliverables |
|-----|-------|-------|--------------|
| 1 | Foundation | Critical fixes | Clean module boundaries |
| 2 | Foundation | Builders | Ergonomic APIs |
| 3 | Foundation | Testing | Test infrastructure |
| 4 | Library API | Public API | Facade pattern |
| 5 | Library API | Documentation | Complete docs |
| 6 | Library API | Shutdown | Graceful termination |
| 7 | Polish | Performance | Optimizations |
| 8 | Polish | Telemetry | Metrics & tracing |
| 9 | Polish | Testing | Load tests |
| 10 | Release | Preparation | Published crate |

## Conclusion

This migration plan transforms Shadowcat from a CLI-centric tool to a library-first architecture over 10 days. Each phase builds on the previous one, with clear success criteria and rollback options. The plan prioritizes the most critical issues first while maintaining backwards compatibility where possible.