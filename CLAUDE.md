# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tapwire is a Model Context Protocol (MCP) developer proxy platform with two main components:
1. **Tapwire**: The overall platform vision for MCP inspection, recording, and observability
2. **Shadowcat**: The core Rust proxy implementation handling forward/reverse proxy, recording, and interception

**Important**: Shadowcat is a git submodule with its own repository. Changes to Shadowcat must be committed in its own repository, not in the main Tapwire repository.

## Essential Commands

### Git Submodule Management
```bash
# Clone with submodules
git clone --recursive <tapwire-repo>

# If already cloned, initialize submodule
git submodule init
git submodule update

# Work on Shadowcat
cd shadowcat
git checkout main  # Or appropriate branch
# Make changes...
git add .
git commit -m "feat: implement feature"
git push

# Update parent repo to point to new commit
cd ..
git add shadowcat
git commit -m "Update shadowcat submodule"
git push
```

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

## Git Commit Guidelines

**Important**: When creating git commits:
- **DO NOT** add Claude as a co-author of commits
- **DO NOT** mention Claude Code in commit messages
- Keep commit messages focused on the technical changes and their purpose
- Use standard conventional commit format when appropriate

## Rust Code Review Guidelines

When reviewing Rust code in this project, use the specialized `rust-code-reviewer` agent for:
- Memory safety verification and unsafe code auditing
- Performance optimization and zero-cost abstractions
- Async/await patterns with tokio
- Error handling with Result types and custom errors
- Trait design and generic programming

The agent follows specific quality gates:
- Flag any unsafe code lacking safety documentation
- Ensure public APIs have documentation
- Check for unwrap()/expect() in production code
- Verify test coverage for critical paths
- Monitor performance against 5% overhead target

**Important** Make sure there are no clippy warnings with `cargo clippy -- -Dwarnings` after significant code changes or before committing code. Remember that `cargo clippy --fix -- -Dwarnings` can help fix a lot of the problems. Also be sure to run `cargo fmt` after significant changes and/or before committing.

## Current Implementation Status

### Shadowcat Core Modules
- `src/transport/`: Transport abstraction with stdio, HTTP, and HTTP-MCP implementations
- `src/proxy/`: Forward and reverse proxy implementations with circuit breakers and health checking
- `src/session/`: Session management and storage
- `src/interceptor/`: Rule-based message interception engine
- `src/recorder/`: Tape recording and replay functionality
- `src/auth/`: OAuth 2.1 authentication gateway
- `src/audit/`: Event logging and audit trails
- `src/rate_limiting/`: Multi-tier rate limiting
- `src/metrics/`: Performance metrics collection

### CLI Commands
```bash
# Forward proxy modes
shadowcat forward stdio -- command args
shadowcat forward http --port 8080 --target http://server

# Reverse proxy
shadowcat reverse --bind 127.0.0.1:8080 --upstream http://mcp-server

# Recording and replay
shadowcat record --output session.tape -- command
shadowcat replay session.tape --port 8080

# Tape management
shadowcat tape list
shadowcat tape info <tape-id>
shadowcat tape export <tape-id>

# Interception management
shadowcat intercept list-rules
shadowcat intercept add-rule --file rule.yaml
```

### Key Dependencies
- **rmcp**: MCP protocol implementation
- **tokio**: Async runtime with full features
- **axum**: HTTP server framework
- **sqlx**: Database access with SQLite
- **jsonwebtoken**: JWT handling
- **oauth2**: OAuth 2.1 implementation
- **governor**: Rate limiting
- **tracing**: Structured logging

## Security Requirements

### Authentication and Authorization
- OAuth 2.1 compliance for auth gateway
- JWT validation with proper audience checking
- PKCE (Proof Key for Code Exchange) support
- **Never forward client tokens to upstream servers**
- Resource server metadata discovery (RFC 9728)

### Transport Security
- Localhost binding by default for development
- Origin validation for HTTP transport
- DNS rebinding protection
- TLS termination for production deployments

### Audit and Compliance
- Comprehensive event logging
- Session tracking and replay capabilities
- Rate limiting with configurable tiers
- Policy enforcement at multiple layers

## Performance Targets

- **Latency overhead**: < 5% p95 for typical tool calls
- **Memory usage**: < 100MB for 1000 concurrent sessions
- **Throughput**: > 10,000 requests/second
- **Startup time**: < 100ms
- **Recording overhead**: < 10% additional latency

## Contributing Guidelines

1. **Code Style**: Follow standard Rust conventions (cargo fmt, cargo clippy)
2. **Testing**: Comprehensive unit and integration tests required
3. **Documentation**: Public APIs must be documented with examples
4. **Performance**: Profile changes that may affect latency
5. **Security**: All auth-related changes need security review
6. **Compatibility**: Maintain MCP protocol compliance

## Important Notes

- **Submodule Workflow**: Always commit Shadowcat changes in the shadowcat repository first, then update the parent repository's submodule reference
- **Never commit secrets**: Use proper configuration management for sensitive data
- **Protocol Version**: Currently targeting MCP `2025-11-05`
- **Transport Priority**: stdio for development, HTTP for production deployments
- **Session Lifecycle**: All operations are session-scoped with proper cleanup
- **Error Context**: Use `anyhow::Context` for rich error messages throughout the codebase