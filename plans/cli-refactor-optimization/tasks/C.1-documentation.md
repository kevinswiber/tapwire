# Task C.1: Comprehensive Documentation

## Objective
Create complete, professional documentation for Shadowcat covering library API, CLI usage, architecture, examples, and deployment guides to ensure users can effectively use and integrate Shadowcat into their projects.

## Background
Current documentation gaps:
- Only 15% of public functions documented
- No architecture overview
- Missing usage examples
- No deployment guides
- No troubleshooting section
- No API reference

Good documentation is critical for:
- Library adoption
- Reducing support burden
- Onboarding new contributors
- Building trust in the project

## Key Questions to Answer
1. What documentation formats do we need (rustdoc, README, guides)?
2. Who are the primary audiences?
3. What examples best demonstrate capabilities?
4. How do we keep docs in sync with code?

## Step-by-Step Process

### 1. Document All Public APIs
```rust
// Add comprehensive rustdoc to all public items
// src/shadowcat.rs
/// High-level interface for the Shadowcat MCP proxy.
/// 
/// `Shadowcat` provides a simple, ergonomic API for creating Model Context Protocol
/// proxies with support for recording, interception, and rate limiting.
/// 
/// # Quick Start
/// 
/// ```rust
/// use shadowcat::Shadowcat;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a forward proxy from stdio to HTTP
///     let proxy = Shadowcat::new()
///         .forward_stdio_to_http("mcp-server", "https://api.example.com")
///         .await?;
///     
///     // Proxy runs until stopped
///     proxy.wait().await?;
///     Ok(())
/// }
/// ```
/// 
/// # Configuration
/// 
/// Shadowcat can be configured through:
/// - Builder pattern for programmatic configuration
/// - Configuration files (TOML, JSON, YAML)
/// - Environment variables
/// 
/// # Proxy Modes
/// 
/// ## Forward Proxy
/// Client → Shadowcat → Server
/// 
/// Used when you control the client and want to intercept/record traffic
/// to an MCP server.
/// 
/// ## Reverse Proxy
/// Client → Shadowcat (HTTP) → Server
/// 
/// Used when you control the server and want to provide an HTTP gateway
/// to stdio-based MCP servers.
pub struct Shadowcat {
    // ...
}
```

### 2. Create README.md
```markdown
# Shadowcat

[![Crates.io](https://img.shields.io/crates/v/shadowcat.svg)](https://crates.io/crates/shadowcat)
[![Documentation](https://docs.rs/shadowcat/badge.svg)](https://docs.rs/shadowcat)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](LICENSE)
[![Build Status](https://github.com/tapwire/shadowcat/workflows/CI/badge.svg)](https://github.com/tapwire/shadowcat/actions)

High-performance Model Context Protocol (MCP) proxy with recording, interception, and rate limiting.

## Features

- 🚀 **High Performance** - Less than 5% latency overhead
- 📼 **Recording & Replay** - Capture and replay MCP sessions
- 🔍 **Interception** - Inspect and modify messages in-flight
- ⚡ **Rate Limiting** - Multi-tier rate limiting with burst support
- 🔐 **Auth Gateway** - OAuth 2.1 authentication for reverse proxy
- 📊 **Metrics & Telemetry** - Prometheus metrics and OpenTelemetry support
- 🛠️ **Developer Friendly** - Simple API with sensible defaults

## Quick Start

### Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
shadowcat = "0.1"
tokio = { version = "1", features = ["full"] }
```

Create a simple forward proxy:

```rust
use shadowcat::Shadowcat;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proxy = Shadowcat::new()
        .forward_stdio_to_http("mcp-server", "https://api.example.com")
        .await?;
    
    proxy.wait().await?;
    Ok(())
}
```

### CLI Usage

```bash
# Install
cargo install shadowcat

# Forward proxy (stdio to HTTP)
shadowcat forward stdio -- mcp-server --arg1 value1

# Reverse proxy (HTTP gateway to stdio)
shadowcat reverse --bind 127.0.0.1:8080 --upstream stdio -- mcp-server

# Record a session
shadowcat record stdio --output session.tape -- mcp-server

# Replay a session
shadowcat replay session.tape --port 8080
```

## Documentation

- [API Documentation](https://docs.rs/shadowcat)
- [Architecture Guide](docs/architecture.md)
- [Configuration Reference](docs/configuration.md)
- [Deployment Guide](docs/deployment.md)
- [Examples](examples/)

## Examples

See the [examples](examples/) directory for:
- [Simple forward proxy](examples/simple_forward.rs)
- [Reverse proxy with auth](examples/reverse_with_auth.rs)
- [Recording sessions](examples/recording.rs)
- [Custom interceptors](examples/custom_interceptor.rs)
- [Rate limiting](examples/rate_limiting.rs)

## Performance

Shadowcat is designed for minimal overhead:

| Metric | Target | Actual |
|--------|--------|--------|
| Latency overhead | < 5% | 3.2% |
| Memory per session | < 100KB | 87KB |
| Throughput | > 10k req/s | 12k req/s |

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
```

### 3. Create Architecture Documentation
```markdown
# docs/architecture.md

# Shadowcat Architecture

## Overview

Shadowcat is built as a modular, async-first MCP proxy with clear separation of concerns.

## Component Architecture

```
┌─────────────────────────────────────────┐
│            Shadowcat Library            │
├─────────────────────────────────────────┤
│          High-Level API Layer           │
│         (shadowcat::Shadowcat)          │
├─────────────────────────────────────────┤
│           Core Components               │
│  ┌──────────┐ ┌──────────┐ ┌─────────┐ │
│  │Transport │ │  Proxy   │ │Session  │ │
│  │  Layer   │ │  Engine  │ │Manager  │ │
│  └──────────┘ └──────────┘ └─────────┘ │
│  ┌──────────┐ ┌──────────┐ ┌─────────┐ │
│  │Recorder  │ │Intercept │ │  Auth   │ │
│  │  Engine  │ │  Chain   │ │Gateway  │ │
│  └──────────┘ └──────────┘ └─────────┘ │
├─────────────────────────────────────────┤
│          Foundation Layer               │
│  ┌──────────┐ ┌──────────┐ ┌─────────┐ │
│  │  Config  │ │  Error   │ │Metrics  │ │
│  │  System  │ │ Handling │ │Telemetry│ │
│  └──────────┘ └──────────┘ └─────────┘ │
└─────────────────────────────────────────┘
```

## Data Flow

### Forward Proxy Mode
1. Client connects via Transport (stdio/HTTP)
2. Session Manager creates new session
3. Interceptor chain processes request
4. Rate limiter checks quotas
5. Proxy forwards to upstream server
6. Recorder captures traffic (if enabled)
7. Response flows back through chain

### Reverse Proxy Mode
1. HTTP request received on bind address
2. Auth gateway validates credentials
3. Session lookup or creation
4. Transform HTTP to MCP protocol
5. Forward to stdio upstream
6. Transform response back to HTTP

## Key Design Decisions

### Async-First
All I/O operations are async using Tokio, ensuring high concurrency without thread overhead.

### Builder Pattern
Complex objects use builders for ergonomic construction with validation.

### Transport Abstraction
Unified `Transport` trait allows seamless switching between stdio, HTTP, and SSE.

### Session-Centric
All operations are organized around sessions for consistency and tracking.

## Performance Considerations

### Zero-Copy Where Possible
Message forwarding uses zero-copy techniques to minimize overhead.

### Connection Pooling
HTTP transports use connection pooling to reduce latency.

### Buffer Reuse
Thread-local buffer pools reduce allocation overhead.
```

### 4. Create Configuration Guide
```markdown
# docs/configuration.md

# Configuration Reference

Shadowcat can be configured through multiple sources (in priority order):
1. Command-line arguments
2. Environment variables
3. Configuration file
4. Defaults

## Configuration File

```toml
# shadowcat.toml

[proxy]
enable_rate_limit = true
rate_limit_rpm = 1000
rate_limit_burst = 100
session_timeout_secs = 300
max_sessions = 1000

[transport]
retry_count = 3
retry_interval_ms = 1000
connect_timeout_secs = 10
request_timeout_secs = 30

[recording]
enabled = false
directory = "./recordings"
format = "jsonl"  # or "binary"
compression = true

[interceptor]
enabled = false
rules_file = "./interceptor-rules.yaml"

[metrics]
enabled = true
port = 9090
path = "/metrics"

[logging]
level = "info"  # trace, debug, info, warn, error
format = "json"  # or "pretty"
```

## Environment Variables

All configuration options can be set via environment variables:

```bash
SHADOWCAT_PROXY_RATE_LIMIT=true
SHADOWCAT_PROXY_RATE_LIMIT_RPM=500
SHADOWCAT_TRANSPORT_TIMEOUT_SECS=60
SHADOWCAT_RECORDING_ENABLED=true
SHADOWCAT_RECORDING_DIRECTORY=/var/shadowcat/recordings
```

## CLI Arguments

```bash
shadowcat forward stdio \
  --rate-limit \
  --rate-limit-rpm 500 \
  --session-timeout 600 \
  --config /etc/shadowcat/config.toml \
  -- mcp-server
```
```

### 5. Create API Examples
```rust
// examples/README.md
//! # Shadowcat Examples
//! 
//! This directory contains examples demonstrating various Shadowcat features.
//! 
//! ## Running Examples
//! 
//! ```bash
//! cargo run --example simple_forward
//! cargo run --example recording
//! cargo run --example custom_interceptor
//! ```

// examples/custom_interceptor.rs
use shadowcat::prelude::*;
use shadowcat::interceptor::{Interceptor, InterceptorContext, InterceptorAction};

struct LoggingInterceptor;

#[async_trait]
impl Interceptor for LoggingInterceptor {
    async fn intercept(&self, ctx: &mut InterceptorContext) -> Result<InterceptorAction> {
        println!("Request: {:?}", ctx.request());
        Ok(InterceptorAction::Continue)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proxy = Shadowcat::new()
        .forward_proxy_builder()
        .interceptor(LoggingInterceptor)
        .start(client, server)
        .await?;
    
    proxy.wait().await?;
    Ok(())
}
```

### 6. Create Deployment Guide
```markdown
# docs/deployment.md

# Deployment Guide

## Docker

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/shadowcat /usr/local/bin/
EXPOSE 8080 9090
ENTRYPOINT ["shadowcat"]
```

```bash
docker build -t shadowcat .
docker run -p 8080:8080 shadowcat reverse --bind 0.0.0.0:8080
```

## Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: shadowcat
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: shadowcat
        image: shadowcat:latest
        args:
        - reverse
        - --bind=0.0.0.0:8080
        - --upstream=stdio
        ports:
        - containerPort: 8080
        - containerPort: 9090  # metrics
```

## Systemd

```ini
[Unit]
Description=Shadowcat MCP Proxy
After=network.target

[Service]
Type=simple
User=shadowcat
ExecStart=/usr/local/bin/shadowcat reverse --config /etc/shadowcat/config.toml
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```
```

### 7. Generate Documentation
```bash
#!/bin/bash
# scripts/generate-docs.sh

# Generate API documentation
cargo doc --no-deps --all-features

# Generate mdbook if using
mdbook build docs/

# Check documentation coverage
cargo doc-coverage

# Lint markdown
markdownlint docs/*.md
```

## Expected Deliverables

### Documentation Files
- `shadowcat/README.md` - Project overview
- `shadowcat/docs/architecture.md` - Architecture guide
- `shadowcat/docs/configuration.md` - Configuration reference
- `shadowcat/docs/deployment.md` - Deployment guide
- `shadowcat/docs/troubleshooting.md` - Common issues
- `shadowcat/CONTRIBUTING.md` - Contribution guidelines

### Code Documentation
- All public APIs with rustdoc
- Examples in doc comments
- Module-level documentation
- Crate-level documentation

### Examples
- `examples/simple_forward.rs`
- `examples/reverse_with_auth.rs`
- `examples/recording.rs`
- `examples/custom_interceptor.rs`
- `examples/rate_limiting.rs`

## Success Criteria Checklist
- [ ] 100% public API documented
- [ ] README complete with badges
- [ ] Architecture documented with diagrams
- [ ] Configuration reference complete
- [ ] 5+ working examples
- [ ] Deployment guides for Docker/K8s
- [ ] Documentation builds without warnings
- [ ] Examples compile and run

## Risk Assessment
- **Risk**: Documentation becomes outdated
  - **Mitigation**: Link docs to code with examples
  - **Mitigation**: CI checks for doc tests

- **Risk**: Too much/too little detail
  - **Mitigation**: Progressive disclosure
  - **Mitigation**: Separate guides by audience

## Duration Estimate
**4 hours**
- 1.5 hours: API documentation
- 1 hour: README and guides
- 1 hour: Examples
- 30 min: Review and polish

## Dependencies
- Phase B complete (stable API to document)

## Notes
- Documentation is marketing for libraries
- Examples are the best documentation
- Keep it scannable with good structure
- Test all code in documentation

## Commands Reference
```bash
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Generate and view docs
cargo doc --no-deps --open

# Test documentation examples
cargo test --doc

# Run examples
for example in examples/*.rs; do
    cargo run --example $(basename $example .rs)
done

# Check doc coverage
cargo doc-coverage  # if installed
```