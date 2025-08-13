# Tapwire - MCP Developer Proxy Platform

## Quick Start
- Clone: `git clone --recursive https://github.com/yourusername/tapwire`
- Setup: `cd shadowcat && cargo build`
- Test: `cargo test`
- Run: `cargo run -- forward stdio -- echo '{"jsonrpc":"2.0","method":"ping","id":1}'`

## Project Structure
- `tapwire/`: Platform coordination and vision
- `shadowcat/`: Core Rust proxy implementation (git submodule)
- `plans/`: Feature planning and tracking
- `specs/`: Technical specifications
- `.claude/`: Modular memory files

## Essential Commands

### Development
- Build: `cd shadowcat && cargo build --release`
- Test all: `cargo test`
- Test specific: `cargo test transport::stdio -- --nocapture`
- Watch mode: `cargo watch -x check -x test`
- Debug run: `RUST_LOG=shadowcat=debug cargo run`

### Code Quality (Run before EVERY commit)
1. Format: `cargo fmt`
2. Lint: `cargo clippy --all-targets -- -D warnings`
3. Test: `cargo test`

## Architecture Overview
- **Protocol**: MCP v2025-11-05
- **Core**: Shadowcat proxy (Rust/Tokio)
- **Storage**: SQLite for sessions and tapes
- **Auth**: OAuth 2.1 compliant gateway
- **Transports**: stdio, HTTP, SSE

## Key Components
- **Transport Layer**: Unified interface for all MCP transports
- **Proxy Engine**: Forward/reverse with circuit breakers
- **Session Manager**: Thread-safe session lifecycle tracking
- **Interceptor Chain**: Pause/modify/block message processing
- **Recording Engine**: Capture and replay MCP sessions
- **Auth Gateway**: OAuth 2.1 with JWT validation

## Critical Rules
- **NEVER** add Claude as git co-author
- **NEVER** forward client tokens upstream
- **NEVER** commit with clippy warnings
- **ALWAYS** commit to shadowcat submodule first
- **ALWAYS** run quality checks before commit

## Performance Targets
- Latency overhead: < 5% p95
- Memory: < 100MB for 1000 sessions
- Throughput: > 10,000 req/sec
- Startup: < 100ms

## Testing Strategy
- Unit: Mock transports and stores
- Integration: Full proxy flows
- Conformance: MCP spec validation
- Performance: Latency benchmarks

## Import Additional Memory
@.claude/git-workflow.md
@.claude/security-requirements.md
@.claude/planning-process.md
@shadowcat/CLAUDE.md

## Key Documentation
- Vision: `plans/001-initial-prd.md`
- Architecture: `plans/002-shadowcat-architecture-plan.md`
- Developer Guide: `plans/003-shadowcat-developer-guide.md`

## Current Focus
Phase 1 - Core Infrastructure:
- Transport abstraction implementation
- Basic forward proxy
- Session management
- Error handling framework