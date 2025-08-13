# Shadowcat Developer Implementation Guide

**Project:** Shadowcat Development Guide  
**Version:** v0.1  
**Date:** August 4, 2025  
**Status:** Draft

---

## 1. Getting Started

### 1.1 Prerequisites

- Rust 1.75+ (for async trait support)
- SQLite 3.35+ (for JSON support)
- Git
- Optional: Docker for integration testing

### 1.2 Initial Setup

```bash
# Clone and setup
git clone https://github.com/tapwire/shadowcat.git
cd shadowcat

# Install dependencies
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=shadowcat=debug cargo run -- forward stdio -- echo '{"jsonrpc":"2.0","method":"ping","id":1}'
```

### 1.3 Development Environment

```bash
# Install development tools
cargo install cargo-watch cargo-nextest cargo-tarpaulin

# Watch mode for development
cargo watch -x check -x test -x run

# Coverage reports
cargo tarpaulin --out Html

# Benchmarks
cargo bench
```

---

## 2. Core Abstractions & Code Examples

### 2.1 Transport Trait

```rust
// src/transport/mod.rs
use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum TransportMessage {
    Request { id: String, method: String, params: Value },
    Response { id: String, result: Option<Value>, error: Option<Value> },
    Notification { method: String, params: Value },
}

#[async_trait]
pub trait Transport: Send + Sync {
    async fn connect(&mut self) -> Result<(), TransportError>;
    async fn send(&mut self, msg: TransportMessage) -> Result<(), TransportError>;
    async fn receive(&mut self) -> Result<TransportMessage, TransportError>;
    async fn close(&mut self) -> Result<(), TransportError>;
    
    fn transport_type(&self) -> TransportType;
}

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Receive failed: {0}")]
    ReceiveFailed(String),
    #[error("Protocol error: {0}")]
    ProtocolError(String),
}
```

### 2.2 Stdio Transport Implementation

```rust
// src/transport/stdio.rs
use super::*;
use tokio::process::{Child, Command};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub struct StdioTransport {
    process: Option<Child>,
    stdin_tx: Option<mpsc::Sender<String>>,
    stdout_rx: Option<mpsc::Receiver<String>>,
}

impl StdioTransport {
    pub fn new(command: Command) -> Self {
        Self {
            process: None,
            stdin_tx: None,
            stdout_rx: None,
        }
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        let mut child = self.command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| TransportError::ConnectionFailed(e.to_string()))?;
            
        // Setup channels for communication
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        
        // Spawn tasks for stdin/stdout handling
        let (stdin_tx, mut stdin_rx) = mpsc::channel::<String>(100);
        let (stdout_tx, stdout_rx) = mpsc::channel::<String>(100);
        
        // Stdin writer task
        tokio::spawn(async move {
            let mut stdin = stdin;
            while let Some(msg) = stdin_rx.recv().await {
                let _ = stdin.write_all(msg.as_bytes()).await;
                let _ = stdin.write_all(b"\n").await;
                let _ = stdin.flush().await;
            }
        });
        
        // Stdout reader task
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            while reader.read_line(&mut line).await.unwrap() > 0 {
                let _ = stdout_tx.send(line.clone()).await;
                line.clear();
            }
        });
        
        self.process = Some(child);
        self.stdin_tx = Some(stdin_tx);
        self.stdout_rx = Some(stdout_rx);
        
        Ok(())
    }
    
    async fn send(&mut self, msg: TransportMessage) -> Result<(), TransportError> {
        let json = serde_json::to_string(&msg)
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;
            
        self.stdin_tx
            .as_ref()
            .ok_or_else(|| TransportError::SendFailed("Not connected".into()))?
            .send(json)
            .await
            .map_err(|e| TransportError::SendFailed(e.to_string()))?;
            
        Ok(())
    }
    
    // ... receive and close implementations
}
```

### 2.3 Session Manager

```rust
// src/session/manager.rs
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub server_session_id: Option<String>,
    pub transport: TransportType,
    pub protocol_version: String,
    pub created_at: Instant,
    pub auth_state: AuthState,
    pub frames: Arc<RwLock<Vec<Frame>>>,
}

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, Arc<Session>>>>,
    persistence: Arc<dyn SessionStore>,
    config: SessionConfig,
}

impl SessionManager {
    pub fn new(persistence: Arc<dyn SessionStore>, config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            persistence,
            config,
        }
    }
    
    pub async fn create_session(&self, transport: TransportType) -> Result<Arc<Session>, SessionError> {
        let session = Arc::new(Session {
            id: SessionId(Uuid::new_v4()),
            server_session_id: None,
            transport,
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            created_at: Instant::now(),
            auth_state: AuthState::Unauthenticated,
            frames: Arc::new(RwLock::new(Vec::new())),
        });
        
        // Store in memory
        self.sessions.write().await.insert(session.id.clone(), session.clone());
        
        // Persist to storage
        self.persistence.create(&session).await?;
        
        Ok(session)
    }
    
    pub async fn add_frame(&self, session_id: &SessionId, frame: Frame) -> Result<(), SessionError> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| SessionError::NotFound(session_id.clone()))?;
            
        // Add to memory
        session.frames.write().await.push(frame.clone());
        
        // Persist frame
        self.persistence.add_frame(session_id, &frame).await?;
        
        Ok(())
    }
}
```

### 2.4 Interceptor Pattern

```rust
// src/interceptor/engine.rs
use async_trait::async_trait;

#[async_trait]
pub trait Interceptor: Send + Sync {
    async fn should_intercept(&self, ctx: &InterceptContext) -> bool;
    async fn intercept(&self, ctx: &mut InterceptContext) -> Result<InterceptAction, InterceptError>;
}

pub struct InterceptContext {
    pub session_id: SessionId,
    pub direction: Direction,
    pub message: TransportMessage,
    pub metadata: HashMap<String, Value>,
}

pub enum InterceptAction {
    Continue,
    Modify(TransportMessage),
    Block(String),
    Pause { resume_tx: oneshot::Sender<InterceptAction> },
}

pub struct InterceptorChain {
    interceptors: Vec<Box<dyn Interceptor>>,
}

impl InterceptorChain {
    pub async fn process(&self, mut ctx: InterceptContext) -> Result<TransportMessage, InterceptError> {
        for interceptor in &self.interceptors {
            if interceptor.should_intercept(&ctx).await {
                match interceptor.intercept(&mut ctx).await? {
                    InterceptAction::Continue => continue,
                    InterceptAction::Modify(msg) => {
                        ctx.message = msg;
                    },
                    InterceptAction::Block(reason) => {
                        return Err(InterceptError::Blocked(reason));
                    },
                    InterceptAction::Pause { resume_tx } => {
                        // Wait for manual intervention
                        match resume_tx.await {
                            Ok(action) => match action {
                                InterceptAction::Modify(msg) => ctx.message = msg,
                                InterceptAction::Block(reason) => return Err(InterceptError::Blocked(reason)),
                                _ => {}
                            },
                            Err(_) => return Err(InterceptError::PauseTimeout),
                        }
                    }
                }
            }
        }
        
        Ok(ctx.message)
    }
}
```

### 2.5 Recording Engine

```rust
// src/recorder/tape.rs
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Tape {
    pub version: String,
    pub tape_id: Uuid,
    pub session_id: SessionId,
    pub metadata: TapeMetadata,
    pub frames: Vec<RecordedFrame>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TapeMetadata {
    pub created_at: DateTime<Utc>,
    pub duration_ms: u64,
    pub transport: TransportType,
    pub protocol_version: String,
    pub server_capabilities: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordedFrame {
    pub timestamp_ms: u64,
    pub direction: Direction,
    pub edge: TransportEdge,
    pub content: Value,
    pub metadata: FrameMetadata,
}

pub struct TapeRecorder {
    tape: Tape,
    start_time: Instant,
    storage: Arc<dyn TapeStorage>,
}

impl TapeRecorder {
    pub fn new(session_id: SessionId, transport: TransportType) -> Self {
        Self {
            tape: Tape {
                version: TAPE_VERSION.to_string(),
                tape_id: Uuid::new_v4(),
                session_id,
                metadata: TapeMetadata {
                    created_at: Utc::now(),
                    duration_ms: 0,
                    transport,
                    protocol_version: MCP_PROTOCOL_VERSION.to_string(),
                    server_capabilities: None,
                },
                frames: Vec::new(),
            },
            start_time: Instant::now(),
            storage: Arc::new(SqliteTapeStorage::new("tapes.db")?),
        }
    }
    
    pub async fn record_frame(&mut self, direction: Direction, edge: TransportEdge, content: Value) {
        let timestamp_ms = self.start_time.elapsed().as_millis() as u64;
        
        let frame = RecordedFrame {
            timestamp_ms,
            direction,
            edge,
            content: content.clone(),
            metadata: FrameMetadata::default(),
        };
        
        self.tape.frames.push(frame);
        
        // Update duration
        self.tape.metadata.duration_ms = timestamp_ms;
        
        // Check for server capabilities
        if let Some(caps) = extract_server_capabilities(&content) {
            self.tape.metadata.server_capabilities = Some(caps);
        }
    }
    
    pub async fn finalize(&mut self) -> Result<(), RecorderError> {
        self.storage.save(&self.tape).await?;
        Ok(())
    }
}
```

---

## 3. Unit Testing Patterns

### 3.1 Transport Testing

```rust
// src/transport/stdio.rs (test module)
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    #[test]
    async fn test_stdio_echo() {
        let mut transport = StdioTransport::new(
            Command::new("sh")
                .arg("-c")
                .arg("while read line; do echo $line; done")
        );
        
        transport.connect().await.unwrap();
        
        let msg = TransportMessage::Request {
            id: "1".to_string(),
            method: "test".to_string(),
            params: json!({}),
        };
        
        transport.send(msg.clone()).await.unwrap();
        let received = transport.receive().await.unwrap();
        
        assert_eq!(received, msg);
    }
    
    #[test]
    async fn test_transport_error_handling() {
        let mut transport = StdioTransport::new(
            Command::new("nonexistent-command")
        );
        
        let result = transport.connect().await;
        assert!(matches!(result, Err(TransportError::ConnectionFailed(_))));
    }
}
```

### 3.2 Session Manager Testing

```rust
// src/session/manager.rs (test module)
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    
    mock! {
        Store {}
        
        #[async_trait]
        impl SessionStore for Store {
            async fn create(&self, session: &Session) -> Result<(), StorageError>;
            async fn get(&self, id: &SessionId) -> Result<Option<Session>, StorageError>;
            async fn add_frame(&self, id: &SessionId, frame: &Frame) -> Result<(), StorageError>;
        }
    }
    
    #[test]
    async fn test_session_creation() {
        let mut mock_store = MockStore::new();
        mock_store
            .expect_create()
            .times(1)
            .returning(|_| Ok(()));
            
        let manager = SessionManager::new(
            Arc::new(mock_store),
            SessionConfig::default()
        );
        
        let session = manager.create_session(TransportType::Stdio).await.unwrap();
        assert!(session.id.0 != Uuid::nil());
        assert_eq!(session.transport, TransportType::Stdio);
    }
    
    #[test]
    async fn test_concurrent_session_access() {
        let manager = SessionManager::new(
            Arc::new(MemoryStore::new()),
            SessionConfig::default()
        );
        
        let session = manager.create_session(TransportType::Http).await.unwrap();
        let session_id = session.id.clone();
        
        // Simulate concurrent frame additions
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let manager = manager.clone();
                let session_id = session_id.clone();
                tokio::spawn(async move {
                    let frame = Frame::new(
                        Direction::ClientToServer,
                        json!({ "test": i })
                    );
                    manager.add_frame(&session_id, frame).await
                })
            })
            .collect();
            
        for handle in handles {
            handle.await.unwrap().unwrap();
        }
        
        let frames = session.frames.read().await;
        assert_eq!(frames.len(), 10);
    }
}
```

### 3.3 Interceptor Testing

```rust
// src/interceptor/engine.rs (test module)
#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestInterceptor {
        should_intercept: bool,
        action: InterceptAction,
    }
    
    #[async_trait]
    impl Interceptor for TestInterceptor {
        async fn should_intercept(&self, _ctx: &InterceptContext) -> bool {
            self.should_intercept
        }
        
        async fn intercept(&self, _ctx: &mut InterceptContext) -> Result<InterceptAction, InterceptError> {
            Ok(self.action.clone())
        }
    }
    
    #[test]
    async fn test_interceptor_chain_modify() {
        let chain = InterceptorChain {
            interceptors: vec![
                Box::new(TestInterceptor {
                    should_intercept: true,
                    action: InterceptAction::Modify(TransportMessage::Notification {
                        method: "modified".to_string(),
                        params: json!({}),
                    }),
                }),
            ],
        };
        
        let ctx = InterceptContext {
            session_id: SessionId(Uuid::new_v4()),
            direction: Direction::ClientToServer,
            message: TransportMessage::Notification {
                method: "original".to_string(),
                params: json!({}),
            },
            metadata: HashMap::new(),
        };
        
        let result = chain.process(ctx).await.unwrap();
        match result {
            TransportMessage::Notification { method, .. } => {
                assert_eq!(method, "modified");
            }
            _ => panic!("Expected notification"),
        }
    }
}
```

### 3.4 Integration Testing

```rust
// tests/integration/proxy_flow.rs
use shadowcat::prelude::*;

#[tokio::test]
async fn test_full_proxy_flow() {
    // Start a mock MCP server
    let mock_server = MockMcpServer::start().await;
    
    // Create forward proxy
    let proxy = ForwardProxy::builder()
        .target(mock_server.url())
        .recording(true)
        .build();
        
    let proxy_handle = tokio::spawn(async move {
        proxy.run().await
    });
    
    // Create MCP client
    let client = MpcClient::connect(proxy.url()).await.unwrap();
    
    // Send initialize request
    let response = client.initialize(InitializeParams {
        protocol_version: MCP_PROTOCOL_VERSION.to_string(),
        capabilities: ClientCapabilities::default(),
        client_info: ClientInfo {
            name: "test-client".to_string(),
            version: "1.0".to_string(),
        },
    }).await.unwrap();
    
    assert!(response.capabilities.tools.is_some());
    
    // Verify recording
    let tape = proxy.get_tape().await.unwrap();
    assert!(!tape.frames.is_empty());
    
    // Cleanup
    proxy_handle.abort();
}
```

---

## 4. Error Handling Patterns

### 4.1 Error Types

```rust
// src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShadowcatError {
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),
    
    #[error("Session error: {0}")]
    Session(#[from] SessionError),
    
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    
    #[error("Auth error: {0}")]
    Auth(#[from] AuthError),
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Protocol error: {0}")]
    Protocol(String),
}

pub type Result<T> = std::result::Result<T, ShadowcatError>;
```

### 4.2 Error Context Pattern

```rust
use anyhow::{Context, Result};

pub async fn process_request(req: Request) -> Result<Response> {
    let session = get_session(&req)
        .await
        .context("Failed to retrieve session")?;
        
    let validated = validate_request(&req, &session)
        .await
        .with_context(|| format!("Validation failed for session {}", session.id))?;
        
    let response = forward_request(validated)
        .await
        .with_context(|| {
            format!("Failed to forward request {} to upstream", req.id)
        })?;
        
    Ok(response)
}
```

---

## 5. Logging and Debugging

### 5.1 Structured Logging

```rust
// src/main.rs
use tracing::{info, debug, error, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true);
        
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "shadowcat=info".into());
        
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}

#[instrument(skip(transport))]
pub async fn handle_message(
    transport: &mut dyn Transport,
    msg: TransportMessage,
) -> Result<()> {
    debug!(?msg, "Processing message");
    
    match msg {
        TransportMessage::Request { id, method, params } => {
            info!(%id, %method, "Handling request");
            // Process request
        }
        TransportMessage::Notification { method, params } => {
            debug!(%method, "Handling notification");
            // Process notification
        }
        _ => {}
    }
    
    Ok(())
}
```

### 5.2 Debug Helpers

```rust
// src/debug.rs
pub mod debug {
    use std::fmt;
    
    pub struct MessageDebug<'a>(&'a TransportMessage);
    
    impl fmt::Debug for MessageDebug<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self.0 {
                TransportMessage::Request { id, method, .. } => {
                    write!(f, "Request(id={}, method={})", id, method)
                }
                TransportMessage::Response { id, .. } => {
                    write!(f, "Response(id={})", id)
                }
                TransportMessage::Notification { method, .. } => {
                    write!(f, "Notification(method={})", method)
                }
            }
        }
    }
    
    pub trait DebugExt {
        fn debug(&self) -> MessageDebug;
    }
    
    impl DebugExt for TransportMessage {
        fn debug(&self) -> MessageDebug {
            MessageDebug(self)
        }
    }
}
```

---

## 6. Development Workflow

### 6.1 Feature Development

```bash
# 1. Create feature branch
git checkout -b feature/add-websocket-transport

# 2. Implement with TDD
# Write failing test first
echo "Write test in src/transport/websocket.rs"

# 3. Run tests in watch mode
cargo watch -x "test transport::websocket"

# 4. Implement until tests pass

# 5. Run full test suite
cargo test

# 6. Check formatting and lints
cargo fmt --check
cargo clippy -- -D warnings

# 7. Update documentation
cargo doc --open
```

### 6.2 Debugging Tips

```rust
// Use debug_assertions for expensive checks
#[cfg(debug_assertions)]
fn validate_frame(frame: &Frame) {
    assert!(frame.timestamp_ms > 0);
    assert!(matches!(frame.direction, Direction::ClientToServer | Direction::ServerToClient));
}

// Conditional compilation for debug features
#[cfg(feature = "debug-ui")]
pub mod debug_ui {
    pub fn show_frame_inspector(frame: &Frame) {
        // TUI or web UI for frame inspection
    }
}
```

### 6.3 Performance Profiling

```rust
// Cargo.toml
[profile.release-with-debug]
inherits = "release"
debug = true

// Run with profiling
// cargo build --profile release-with-debug
// perf record --call-graph=dwarf ./target/release-with-debug/shadowcat
// perf report

// Or use flamegraph
// cargo install flamegraph
// cargo flamegraph --bin shadowcat -- forward stdio -- your-mcp-server
```

---

## 7. Common Patterns & Best Practices

### 7.1 Builder Pattern for Complex Types

```rust
pub struct ProxyBuilder {
    config: ProxyConfig,
    interceptors: Vec<Box<dyn Interceptor>>,
    recorder: Option<TapeRecorder>,
}

impl ProxyBuilder {
    pub fn new() -> Self {
        Self {
            config: ProxyConfig::default(),
            interceptors: Vec::new(),
            recorder: None,
        }
    }
    
    pub fn with_config(mut self, config: ProxyConfig) -> Self {
        self.config = config;
        self
    }
    
    pub fn add_interceptor(mut self, interceptor: impl Interceptor + 'static) -> Self {
        self.interceptors.push(Box::new(interceptor));
        self
    }
    
    pub fn with_recording(mut self, enabled: bool) -> Self {
        if enabled {
            self.recorder = Some(TapeRecorder::new());
        }
        self
    }
    
    pub fn build(self) -> Result<Proxy> {
        Ok(Proxy {
            config: self.config,
            interceptor_chain: InterceptorChain::new(self.interceptors),
            recorder: self.recorder,
        })
    }
}
```

### 7.2 Graceful Shutdown

```rust
pub struct ProxyServer {
    shutdown: broadcast::Sender<()>,
}

impl ProxyServer {
    pub async fn run(&self) -> Result<()> {
        let mut shutdown_rx = self.shutdown.subscribe();
        
        tokio::select! {
            result = self.serve() => {
                result?
            }
            _ = shutdown_rx.recv() => {
                info!("Received shutdown signal");
                self.graceful_shutdown().await?;
            }
        }
        
        Ok(())
    }
    
    async fn graceful_shutdown(&self) -> Result<()> {
        // 1. Stop accepting new connections
        // 2. Wait for ongoing requests to complete (with timeout)
        // 3. Flush recordings
        // 4. Close transports
        Ok(())
    }
}
```

### 7.3 Type State Pattern for Session Lifecycle

```rust
pub struct Session<S: SessionState> {
    id: SessionId,
    _state: PhantomData<S>,
}

pub struct Uninitialized;
pub struct Initialized { capabilities: ServerCapabilities }
pub struct Active;
pub struct Terminated;

impl Session<Uninitialized> {
    pub fn initialize(self, caps: ServerCapabilities) -> Session<Initialized> {
        Session {
            id: self.id,
            _state: PhantomData,
        }
    }
}

impl Session<Initialized> {
    pub fn activate(self) -> Session<Active> {
        Session {
            id: self.id,
            _state: PhantomData,
        }
    }
}
```

---

## 8. Benchmarking

### 8.1 Benchmark Setup

```rust
// benches/proxy_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use shadowcat::proxy::*;

fn benchmark_message_processing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("process_simple_request", |b| {
        b.to_async(&rt).iter(|| async {
            let msg = TransportMessage::Request {
                id: "1".to_string(),
                method: "test".to_string(),
                params: json!({}),
            };
            
            let proxy = create_test_proxy();
            black_box(proxy.process_message(msg).await)
        })
    });
}

criterion_group!(benches, benchmark_message_processing);
criterion_main!(benches);
```

---

## 9. CI/CD Configuration

### 9.1 GitHub Actions

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt, clippy
    
    - uses: Swatinem/rust-cache@v2
    
    - name: Check formatting
      run: cargo fmt -- --check
    
    - name: Clippy
      run: cargo clippy -- -D warnings
    
    - name: Test
      run: cargo test
    
    - name: Test with all features
      run: cargo test --all-features
    
    - name: Doc tests
      run: cargo test --doc

  coverage:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: dtolnay/rust-toolchain@stable
    
    - name: Install tarpaulin
      run: cargo install cargo-tarpaulin
    
    - name: Coverage
      run: cargo tarpaulin --out Xml
    
    - name: Upload coverage
      uses: codecov/codecov-action@v3
```

---

## 10. Release Checklist

- [ ] All tests passing
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Security audit: `cargo audit`
- [ ] Benchmarks show no regression
- [ ] Integration tests with major MCP servers
- [ ] Manual testing of CLI commands
- [ ] Release notes drafted
- [ ] Git tag created

---

## 11. Troubleshooting Guide

### Common Issues

1. **Process spawning fails on Windows**
   ```rust
   // Use explicit shell on Windows
   #[cfg(target_os = "windows")]
   let cmd = Command::new("cmd").arg("/C").arg(command);
   
   #[cfg(not(target_os = "windows"))]
   let cmd = Command::new("sh").arg("-c").arg(command);
   ```

2. **Deadlock in stdio transport**
   - Ensure separate tasks for reading/writing
   - Use bounded channels to prevent memory growth
   - Add timeouts for all operations

3. **Session leaks**
   - Implement session timeout
   - Clean up on transport disconnect
   - Monitor session count in metrics

---

## 12. Resources

- [MCP Specification](https://modelcontextprotocol.io/specification)
- [rmcp Documentation](https://docs.rs/rmcp)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Error Handling in Rust](https://nick.groenen.me/posts/rust-error-handling/)