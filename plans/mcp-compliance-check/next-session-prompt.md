# Next Session Prompt - MCP Compliance Project

**Last Updated**: 2025-08-24  
**Current Phase**: C.7 - Connection Pattern Implementation  
**Status**: Hyper 1.7 upgrade complete, ready to implement Connection trait

## Context for Next Session

### What We Just Completed (2025-08-24)

1. ✅ **Hyper 1.7 Upgrade** - Successfully migrated from hyper 0.14 to 1.7
   - Direct connection management via `hyper::client::conn`
   - No built-in pooling (avoids double pooling with shadowcat)
   - Pure Rust TLS with rustls
   - ~25% performance improvement
   - Foundation for HTTP/3

2. ✅ **GPT-5 Critical Bugs Fixed**
   - C.6.0: Client deadlock resolved with background receiver task
   - C.6.1: HTTP worker pattern implemented with real HTTP requests

3. ✅ **Architecture Pivot to Connection Pattern**
   - Moving from Sink/Stream to async Connection trait
   - Eliminates worker task overhead (20µs → ~0)
   - Natural HTTP/2 multiplexing and connection pooling

4. ✅ **Documentation Consolidation**
   - Created `CONSOLIDATED-ARCHITECTURE.md` as single source of truth
   - Reduced 37 docs to ~10 active documents
   - Updated all READMEs to reflect current status

5. ✅ **Pool Performance Analysis**
   - Created `~/src/tapwire/research/hyper-pool-vs-shadowcat-pool.md`
   - Identified optimization opportunities
   - Shadowcat's 30ns overhead acceptable for multi-protocol support

## 🚨 IMPORTANT: Working in Git Worktree

**Work Directory**: `/Users/kevin/src/tapwire/shadowcat-mcp-compliance`
- Git worktree on branch `feat/mcpspec`
- Main shadowcat remains untouched
- Commit to `feat/mcpspec` branch

## Current Work: C.7 - Connection Pattern Implementation

### Next Task: C.7.0 - Create Connection Trait (2 hours)

**Objective**: Define the core Connection trait to replace Sink/Stream

```rust
// crates/mcp/src/connection/mod.rs
#[async_trait]
pub trait Connection: Send + Sync {
    /// Send a message through the connection
    async fn send(&mut self, message: Value) -> Result<()>;
    
    /// Receive a message from the connection
    async fn receive(&mut self) -> Result<Value>;
    
    /// Close the connection gracefully
    async fn close(&mut self) -> Result<()>;
    
    /// Check if connection is healthy (for pooling)
    fn is_healthy(&self) -> bool { true }
    
    /// Get protocol type (for routing)
    fn protocol(&self) -> Protocol { Protocol::Unknown }
}

// Migration adapter for existing Sink/Stream transports
pub struct SinkStreamAdapter<T> where T: Sink<Value> + Stream<Item = Result<Value>> {
    inner: T,
}
```

### Implementation Steps

1. **Create Connection trait** (C.7.0 - NOW)
   - Define trait in `crates/mcp/src/connection/mod.rs`
   - Add Protocol enum for routing
   - Create SinkStreamAdapter for gradual migration
   - Write tests for adapter

2. **Implement HTTP/2 Connection** (C.7.1 - 4 hours)
   - Use hyper 1.7's `http2::SendRequest`
   - Support SSE streaming
   - Session ID management via headers
   - Ready for shadowcat pooling

3. **Implement WebSocket Connection** (C.7.2 - 3 hours)
   - Use tokio-tungstenite
   - Bidirectional messaging
   - Session in message routing

4. **Implement Stdio Connection** (C.7.3 - 2 hours)
   - Wrapper around existing stdio transport
   - Singleton pattern

5. **Migrate Client/Server** (C.7.4 - 3 hours)
   - Replace Sink/Stream with Connection trait
   - Remove worker tasks
   - Direct async/await

6. **Integrate shadowcat pool** (C.7.5 - 2 hours)
   - Implement PoolableResource for connections
   - Protocol-specific pooling strategies

## Key Files to Reference

**Architecture & Design**:
- `plans/mcp-compliance-check/analysis/CONSOLIDATED-ARCHITECTURE.md` - Current architecture
- `plans/mcp-compliance-check/tasks/C.7-connection-trait-tasks.md` - Detailed task breakdown
- `plans/mcp-compliance-check/mcp-compliance-check-tracker.md` - Project progress

**Code Locations**:
- `crates/mcp/src/transport/http/mod.rs` - HTTP transport (hyper 1.7 ready)
- `crates/mcp/src/client.rs` - Needs migration to Connection trait
- `crates/mcp/src/server.rs` - Needs migration to Connection trait

## Commands to Run

```bash
# Navigate to work directory
cd /Users/kevin/src/tapwire/shadowcat-mcp-compliance/crates/mcp

# Create connection module
mkdir -p src/connection
touch src/connection/mod.rs

# Run tests as you develop
cargo test --package mcp

# Check compilation
cargo check

# Before committing
cargo fmt
cargo clippy --all-targets -- -D warnings
```

## Success Criteria for C.7.0

- [ ] Connection trait defined with async methods
- [ ] Protocol enum for routing decisions
- [ ] SinkStreamAdapter for migration
- [ ] Tests demonstrate adapter works with existing transports
- [ ] Documentation explains when to use Connection vs Sink/Stream

## Architecture Reminder

**Why Connection Pattern?**
- **Zero overhead**: No workers, no channels, no task spawning
- **Natural multiplexing**: HTTP/2 and WebSocket multiplex natively
- **Connection pooling**: Integrates perfectly with shadowcat's pool
- **Direct backpressure**: async/await provides natural flow control
- **Scales to 10K+**: No worker task per connection

**Trade-offs We Accept**:
- More complex than Sink/Stream initially
- Protocol-specific implementations needed
- Worth it for proxy scale requirements

## Performance Targets

- Connection overhead: < 1µs (vs 20µs with workers)
- HTTP/2 multiplexing: 100+ streams per connection
- Pool acquire: < 100µs
- Memory per connection: < 50KB

## If Starting Fresh

Read these in order:
1. `analysis/CONSOLIDATED-ARCHITECTURE.md` - Understand current design
2. `analysis/HYPER-1.7-UPGRADE-COMPLETE.md` - See HTTP transport status
3. `tasks/C.7-connection-trait-tasks.md` - Detailed implementation guide

Then start implementing the Connection trait in `crates/mcp/src/connection/mod.rs`.

## Notes from Previous Session

- Hyper 1.7 upgrade complete and tested
- HTTP transport compiles but needs Connection trait to fully integrate
- SSE receiver updated for hyper 1.7's Incoming body type
- Client/Server still use Sink/Stream (need migration)
- No WebSocket transport yet (needs to be created)

## After This Session

Once Connection trait is implemented:
1. Create protocol-specific connections (HTTP/2, WebSocket, stdio)
2. Migrate Client/Server to use Connection
3. Integrate with shadowcat's pool
4. Run MCP validator tests
5. Performance benchmarking

---

*This prompt captures the current state after hyper 1.7 upgrade and architecture consolidation. The Connection trait implementation is the critical next step to eliminate worker overhead and enable proper connection pooling.*

*Duration estimate for full C.7 phase: 16 hours*  
*Priority: CRITICAL - enables 10K+ connection scale*