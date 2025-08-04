# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tapwire is a Model Context Protocol (MCP) developer proxy platform with two main components:
1. **Tapwire**: The overall platform vision for MCP inspection, recording, and observability
2. **Shadowcat**: The core Rust proxy implementation handling forward/reverse proxy, recording, and interception

## Essential Commands

### Shadowcat Development
```bash
# Navigate to the Rust project
cd shadowcat

# Run tests
cargo test
cargo test transport::stdio::tests  # Run specific test module
cargo test -- --nocapture          # Show println! output

# Development mode with auto-reload
cargo watch -x check -x test -x run

# Run with debug logging
RUST_LOG=shadowcat=debug cargo run -- forward stdio -- echo '{"jsonrpc":"2.0","method":"ping","id":1}'

# Benchmarks
cargo bench
cargo flamegraph

# Code quality
cargo fmt
cargo clippy -- -D warnings
```

### Initial Setup (if dependencies not installed)
```bash
cd shadowcat
cargo add rmcp tokio --features tokio/full
cargo add axum tower tower-http
cargo add serde serde_json
cargo add tracing tracing-subscriber
cargo add clap --features derive
cargo add thiserror anyhow
```

## Architecture Overview

### High-Level Components
```
Tapwire (Platform)
└── Shadowcat (Rust Proxy)
    ├── Transport Layer (stdio, HTTP/SSE)
    ├── Proxy Engine (forward/reverse)
    ├── Session Manager
    ├── Recording Engine (tapes)
    ├── Interceptor (rules, actions)
    └── Auth Gateway (OAuth 2.1)
```

### Key Design Decisions
- **rmcp**: Official Rust MCP SDK for protocol implementation
- **Tokio**: Async runtime (required by rmcp)
- **SQLite**: Embedded storage for recordings and sessions
- **Transport Abstraction**: Unified interface for stdio/HTTP/SSE
- **Session-Centric**: All operations organized around MCP sessions

### Module Communication Flow
1. **Transport** receives MCP messages (stdio process or HTTP request)
2. **Session Manager** tracks session lifecycle and associates frames
3. **Interceptor Chain** processes messages (can pause/modify/block)
4. **Proxy** forwards to destination (with auth for reverse proxy)
5. **Recorder** captures all traffic to persistent tapes

### Critical Files to Understand Architecture
- `plans/001-initial-prd.md`: Overall Tapwire vision and requirements
- `plans/002-shadowcat-architecture-plan.md`: Detailed technical design
- `plans/003-shadowcat-developer-guide.md`: Implementation patterns and examples
- `src/transport/mod.rs` (when created): Core Transport trait defining all transports
- `src/proxy/mod.rs` (when created): Proxy trait unifying forward/reverse behavior

## MCP Protocol Implementation

Key constants and requirements:
- Protocol version: `2025-11-05`
- Session header: `Mcp-Session-Id`
- Version header: `MCP-Protocol-Version`
- **Critical**: Never pass through client tokens to upstream servers
- OAuth 2.1 compliance required for auth gateway

## Development Phases

Currently in Phase 1 (Core Infrastructure):
1. Transport abstraction and stdio implementation
2. Basic forward proxy
3. Session management
4. Error handling framework

Week 1 target: `cargo run -- forward stdio -- echo '{"jsonrpc":"2.0","method":"initialize","id":1}'`

## Testing Approach

- **Unit Tests**: Mock transports, session stores, interceptors using `mockall`
- **Integration Tests**: Full proxy flows with mock MCP servers
- **Conformance Tests**: Validate against MCP specification
- **Performance Tests**: < 5% latency overhead target

Run tests for specific modules during development:
```bash
cargo test session::manager::tests
cargo test interceptor::
```

## Error Handling Pattern

All functions return `Result<T, ShadowcatError>` with context:
```rust
use anyhow::Context;
something.await.context("Failed to do something")?;
```

## Debugging

Enable debug logging:
```bash
RUST_LOG=shadowcat=debug,rmcp=trace cargo run
```

For performance issues:
```bash
cargo flamegraph --bin shadowcat -- forward stdio -- your-command
```