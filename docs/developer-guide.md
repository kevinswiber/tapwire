# Shadowcat Developer Guide

## Getting Started

### Prerequisites

- Rust 1.75+ (stable toolchain)
- Git with submodule support
- SQLite3 development libraries
- Optional: Docker for containerized testing

### Initial Setup

```bash
# Clone with submodules
git clone --recursive https://github.com/yourusername/tapwire
cd tapwire/shadowcat

# Build the project
cargo build --release

# Run tests
cargo test

# Check code quality
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
```

### Project Structure

```
shadowcat/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Library root
│   ├── error.rs          # Error types
│   ├── constants.rs      # Global constants
│   ├── cli/              # CLI command modules
│   ├── transport/        # Transport implementations
│   ├── proxy/            # Proxy modes
│   ├── session/          # Session management
│   ├── mcp/              # MCP protocol handling
│   ├── interceptor/      # Message interception
│   ├── recorder/         # Recording/replay
│   ├── auth/             # Authentication
│   ├── config/           # Configuration
│   └── telemetry/        # Observability
├── tests/                # Integration tests
├── benches/              # Performance benchmarks
└── examples/             # Usage examples
```

## Development Workflow

### 1. Feature Development

#### Create a Plan
For significant features, create a plan under `plans/`:

```bash
# Copy template
cp -r plans/template plans/my-feature

# Edit tracker
vim plans/my-feature/my-feature-tracker.md
```

#### Development Cycle
```bash
# 1. Create feature branch
git checkout -b feature/my-feature

# 2. Make changes
vim src/my_module.rs

# 3. Run tests frequently
cargo test my_module

# 4. Check quality before commit
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test

# 5. Commit with conventional format
git commit -m "feat: add my feature"
```

### 2. Testing Strategy

#### Unit Tests
Place unit tests in the same file as the code:

```rust
// src/my_module.rs
pub fn process_message(msg: &str) -> Result<String> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_message() {
        let result = process_message("test").unwrap();
        assert_eq!(result, "expected");
    }
}
```

#### Integration Tests
Create integration tests in `tests/`:

```rust
// tests/integration_my_feature.rs
use shadowcat::proxy::ForwardProxy;

#[tokio::test]
async fn test_forward_proxy_flow() {
    let proxy = ForwardProxy::builder()
        .transport("stdio")
        .build()
        .await
        .unwrap();
    
    // Test implementation
}
```

#### Running Tests Efficiently
```bash
# Run only unit tests (fast)
cargo test --lib

# Run specific module tests
cargo test transport::

# Run single test
cargo test test_exact_name -- --exact

# Run with output
cargo test -- --nocapture

# Run in release mode (performance tests)
cargo test --release
```

### 3. Debugging

#### Enable Debug Logging
```bash
# Basic debug
RUST_LOG=shadowcat=debug cargo run -- forward stdio -- echo

# Detailed trace
RUST_LOG=shadowcat=trace,rmcp=debug cargo run

# With backtrace
RUST_BACKTRACE=1 cargo run
```

#### Using the Inspector
Test your implementation with the MCP Inspector:

```bash
# Start shadowcat in forward proxy mode
cargo run -- forward stdio -- npx @modelcontextprotocol/server-everything

# In another terminal, start the Inspector
npx @modelcontextprotocol/inspector --cli http://localhost:3000
```

## Core Development Patterns

### 1. Error Handling

Use the custom error types with `anyhow` for context:

```rust
use crate::error::{Result, ShadowcatError};
use anyhow::Context;

pub async fn process() -> Result<()> {
    let data = fetch_data()
        .await
        .context("Failed to fetch data")?;
    
    if !validate(&data) {
        return Err(ShadowcatError::Validation(
            "Invalid data format".into()
        ));
    }
    
    Ok(())
}
```

### 2. Builder Pattern

All major components use builders for construction:

```rust
let proxy = ForwardProxy::builder()
    .transport("stdio")
    .interceptor(my_interceptor)
    .recorder(tape_recorder)
    .session_timeout(Duration::from_secs(300))
    .build()
    .await?;
```

### 3. Async/Await Patterns

Always use async for I/O operations:

```rust
use tokio::time::{timeout, Duration};

async fn fetch_with_timeout() -> Result<Data> {
    timeout(
        Duration::from_secs(10),
        fetch_data()
    )
    .await
    .context("Fetch timeout")?
    .context("Fetch failed")
}
```

### 4. Transport Abstraction

Work with the `MessageEnvelope` abstraction:

```rust
use crate::transport::MessageEnvelope;

async fn handle_message(envelope: MessageEnvelope) -> Result<()> {
    // Access raw content
    let content = &envelope.content;
    
    // Check metadata
    if let Some(session_id) = &envelope.metadata.session_id {
        // Handle session-specific logic
    }
    
    // Parse only when needed
    if envelope.metadata.requires_parsing {
        let parsed = parse_mcp(&envelope.content)?;
        // Process parsed message
    }
    
    Ok(())
}
```

### 5. Session Management

Use the session manager for state tracking:

```rust
use crate::session::{SessionManager, Session};

let manager = SessionManager::new(store).await?;

// Create session
let session = manager.create_session(
    "client-123",
    TransportType::Http
).await?;

// Update session
manager.update_session(&session.id, |s| {
    s.metadata.insert("key".into(), json!("value"));
}).await?;
```

## Adding New Features

### 1. Adding a New Transport

Create a new transport implementation:

```rust
// src/transport/my_transport.rs
use crate::transport::{Transport, MessageEnvelope};

pub struct MyTransport {
    // Fields
}

#[async_trait]
impl Transport for MyTransport {
    async fn connect(&mut self) -> Result<()> {
        // Connection logic
    }
    
    async fn send(&mut self, envelope: MessageEnvelope) -> Result<()> {
        // Send implementation
    }
    
    async fn receive(&mut self) -> Result<MessageEnvelope> {
        // Receive implementation
    }
}
```

Register in the transport factory:

```rust
// src/transport/mod.rs
pub fn create_transport(config: &TransportConfig) -> Result<Box<dyn Transport>> {
    match config.transport_type {
        TransportType::MyTransport => {
            Ok(Box::new(MyTransport::new(config)?))
        }
        // Other transports...
    }
}
```

### 2. Adding an Interceptor

Implement the `Interceptor` trait:

```rust
// src/interceptor/my_interceptor.rs
use crate::interceptor::{Interceptor, InterceptorAction};

pub struct MyInterceptor {
    // Configuration
}

#[async_trait]
impl Interceptor for MyInterceptor {
    async fn process(
        &self,
        envelope: MessageEnvelope
    ) -> Result<InterceptorAction> {
        // Processing logic
        
        if should_block(&envelope) {
            return Ok(InterceptorAction::Block {
                reason: "Blocked by policy".into()
            });
        }
        
        if should_modify(&envelope) {
            let modified = transform(envelope)?;
            return Ok(InterceptorAction::Modify {
                envelope: modified
            });
        }
        
        Ok(InterceptorAction::Continue(envelope))
    }
}
```

### 3. Adding a CLI Command

Create a new command module:

```rust
// src/cli/my_command.rs
use clap::Args;

#[derive(Debug, Args)]
pub struct MyCommand {
    #[arg(long, help = "Command option")]
    option: String,
}

impl MyCommand {
    pub async fn execute(self) -> Result<()> {
        // Command implementation
    }
}
```

Register in the CLI:

```rust
// src/cli/mod.rs
#[derive(Subcommand)]
enum Commands {
    MyCommand(my_command::MyCommand),
    // Other commands...
}
```

## Performance Optimization

### 1. Profiling

Use flamegraph for performance analysis:

```bash
# Install flamegraph
cargo install flamegraph

# Profile the application
cargo flamegraph --release --bin shadowcat -- forward stdio -- command

# View the generated flamegraph.svg
```

### 2. Benchmarking

Create benchmarks in `benches/`:

```rust
// benches/my_benchmark.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            // Code to benchmark
        });
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

Run benchmarks:
```bash
cargo bench
```

### 3. Memory Optimization

- Use `Bytes` for zero-copy operations
- Implement buffer pooling for hot paths
- Avoid unnecessary clones
- Use `Arc` for shared immutable data

## Common Tasks

### Running the Proxy

```bash
# Forward proxy (stdio)
cargo run -- forward stdio -- npx @modelcontextprotocol/server-everything

# Forward proxy (HTTP)
cargo run -- forward http --port 8080 --target http://localhost:3000

# Reverse proxy
cargo run -- reverse --bind 127.0.0.1:8080 --upstream http://mcp-server

# Gateway proxy (multiple upstreams)
cargo run -- gateway --config gateway.toml

# Recording session
cargo run -- record --output session.tape -- forward stdio -- server

# Replay session
cargo run -- replay session.tape --port 8080
```

### Configuration Management

```toml
# shadowcat.toml
[proxy]
mode = "forward"
transport = "stdio"

[session]
timeout = 300
max_sessions = 1000

[telemetry]
enabled = true
endpoint = "http://localhost:4317"

[auth]
provider = "oauth2"
client_id = "your-client-id"
```

### Working with MCP Versions

The proxy supports multiple MCP protocol versions:

```rust
// Check version in code
use crate::mcp::version::ProtocolVersion;

match version {
    ProtocolVersion::V20241105 => {
        // Handle legacy version
    }
    ProtocolVersion::V20250326 | ProtocolVersion::V20250618 => {
        // Handle modern versions
    }
}
```

## Troubleshooting

### Common Issues

#### 1. Build Failures
```bash
# Clean build
cargo clean
cargo build

# Update dependencies
cargo update
```

#### 2. Test Failures
```bash
# Run with verbose output
cargo test -- --nocapture

# Run single test for debugging
cargo test test_name -- --exact --nocapture
```

#### 3. Performance Issues
```bash
# Enable release mode
cargo build --release

# Profile with perf
perf record -F 99 -g target/release/shadowcat
perf report
```

### Getting Help

1. Check existing plans: `ls plans/`
2. Review test files for examples
3. Use MCP Inspector for protocol debugging
4. Reference the MCP specification in `~/src/modelcontextprotocol/`

## Contributing

### Before Submitting

1. **Format code**: `cargo fmt`
2. **Check lints**: `cargo clippy --all-targets -- -D warnings`
3. **Run tests**: `cargo test`
4. **Update docs**: Document new features
5. **Commit style**: Use conventional commits

### Pull Request Process

1. Create feature branch from `main`
2. Make changes following patterns above
3. Ensure all checks pass
4. Update relevant documentation
5. Submit PR with clear description

## Resources

- **Active Plans**: `plans/README.md`
- **MCP Spec**: `~/src/modelcontextprotocol/modelcontextprotocol/specs/`
- **TypeScript SDK**: Reference implementation in `~/src/modelcontextprotocol/typescript-sdk/`
- **Inspector**: Testing tool in `~/src/modelcontextprotocol/inspector/`
- **Example Servers**: `~/src/modelcontextprotocol/servers/`