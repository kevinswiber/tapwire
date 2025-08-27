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
2. Lint: `cargo xtask lint --workspace --all-targets`
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
- **NEVER** commit with lint warnings
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

- **Active Plans**: `plans/README.md` - Main project coordinator (always up-to-date)
- **Architecture Guide**: `docs/architecture.md` - System design and components
- **Developer Guide**: `docs/developer-guide.md` - Development workflow and patterns
- **API Reference**: `docs/api-reference.md` - Library APIs and usage
- **Deployment Guide**: `docs/deployment.md` - Production deployment patterns

## MCP Reference Repositories

We maintain local references to official MCP repositories in `~/src/modelcontextprotocol/` for development and testing:

### Core Specifications (`modelcontextprotocol/`)

- **Purpose**: Official MCP specifications and protocol schemas
- **Key Content**: All protocol versions (2024-11-05, 2025-03-26, 2025-06-18, draft)
- **Use Cases**: Protocol compliance testing, spec validation, schema definitions
- **Location**: `~/src/modelcontextprotocol/modelcontextprotocol/specs/`

### Inspector (`inspector/`)

- **Purpose**: Developer tool for testing and debugging MCP servers
- **Architecture**: Web UI client + protocol bridge proxy server
- **Transports**: stdio, SSE, streamable-http
- **Use Cases**: Testing shadowcat proxy, debugging protocol flows, validating implementations
- **Key Features**: Session export, OAuth support, CLI mode for automation

### TypeScript SDK (`typescript-sdk/`)

- **Purpose**: Official TypeScript/JavaScript SDK (most up-to-date reference)
- **Key Features**: Full protocol implementation, all transport types, extensive examples
- **Use Cases**: Reference implementation, cross-validation with shadowcat, test client
- **Notable**: Used by Inspector, excellent protocol coverage

### Rust SDK (`rust-sdk/`)

- **Purpose**: Official Rust SDK implementation (rmcp crate)
- **Architecture**: Tokio-based async, procedural macros for tools
- **Use Cases**: Reference for Rust patterns, comparison with shadowcat implementation
- **Crates**: rmcp (core), rmcp-macros (code generation)

### Example Servers (`servers/`)

- **Purpose**: Collection of reference MCP server implementations
- **Available Servers**:
  - `everything/`: Complete test server with all MCP features
  - `filesystem/`: File operations with access controls
  - `git/`: Repository manipulation tools
  - `memory/`: Persistent memory with knowledge graphs
  - `fetch/`: Web content fetching
- **Use Cases**: Integration testing targets, protocol conformance testing, real-world examples

### Usage in Development

- **Testing**: Use `servers/everything` for comprehensive integration tests
- **Validation**: Cross-reference protocol behavior with TypeScript SDK
- **Debugging**: Use Inspector to visualize shadowcat's proxy behavior
- **Specs**: Always check `modelcontextprotocol/specs/` for latest protocol changes

## Current Focus

### 🔥 TOP PRIORITY - Critical Proxy Infrastructure

1. **[Reverse Proxy Session Mapping](plans/reverse-proxy-session-mapping/)** - Dual session ID tracking for SSE reconnection/failover
2. **[Multi-Session Forward Proxy](plans/multi-session-forward-proxy/)** - Support multiple concurrent client connections

### Active Development Plans

- **[Better CLI Interface](plans/better-cli-interface/)** - Smart transport detection and improved UX
- **[Full Batch Support](plans/full-batch-support/)** - Complete MCP batch message support
- **[Redis Session Storage](plans/redis-session-storage/)** - Distributed session storage backend
- **[Tape Format JSON Lines](plans/tape-format-json-lines/)** - JSONL tape format for streaming (Phase 1 Complete)
- **[Wassette Integration](plans/wassette-integration/)** - WebAssembly module integration

### Recently Completed

- ✅ **LLM Help Documentation** - Built-in help command with LLM-friendly output (2025-08-15)
- ✅ **Transport Layer Refactor** - Raw transport implementations and builder patterns
- ✅ **SSE/MCP Integration** - Full SSE proxy with MCP message handling (v0.2.0)
- ✅ **CLI Optimization** - Library-first architecture with 802+ tests passing

See `plans/README.md` for complete status, execution order, and detailed progress tracking.

