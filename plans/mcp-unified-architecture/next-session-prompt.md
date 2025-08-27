# Next Session Prompt - Sprint 1 Task 1.2 Basic Hyper Server

## Session Goal
Continue Sprint 1 - Implement a basic Hyper server for the MCP library with proper connection handling patterns.

## Context
- ✅ Task 1.0 Complete: Async patterns already optimal (2h instead of 8h)
- ✅ Task 1.1 Complete: OpenTelemetry observability with Prometheus implemented
- 🎯 Task 1.2: Basic Hyper Server (6h)
- Following v2 tracker in `mcp-tracker-v2-critical-path.md`
- Time saved from 1.0 & 1.1 can be used for more robust implementation

## Current Status

### ✅ Completed (Sprint 1)
1. **Task 1.0 - Async Patterns**
   - Analyzed all async patterns
   - Found bounded executor pattern preventing spawn explosion
   - One-spawn-per-client is correct for stateful MCP
   - No refactoring needed

2. **Task 1.1 - Observability**
   - Added OpenTelemetry + Prometheus metrics
   - Server metrics: connections, requests, duration, errors
   - Client metrics: requests, round-trip time, pool connections
   - Export via `export_metrics()` method
   - Safe error handling (no panics)

## Sprint 1 Task 1.2: Basic Hyper Server (6h) ⭐ CRITICAL

### Goal
Replace or augment the existing server implementation with proper Hyper 1.x patterns for HTTP-based MCP transport.

### Key Requirements
1. **Use Hyper 1.x** (already in dependencies)
2. **One spawn per connection** (pattern already validated)
3. **Integrate with existing metrics** from Task 1.1
4. **Support HTTP/1.1 and HTTP/2**
5. **Proper graceful shutdown**

### Implementation Plan

1. **Review Existing Server** (30 min)
   - Check `src/server.rs` current implementation
   - Understand connection handling patterns
   - Review `src/connection/http.rs` for HTTP support

2. **Create Hyper Service** (2 hours)
   ```rust
   // Create a service that handles MCP over HTTP
   struct McpHttpService<H: ServerHandler> {
       handler: Arc<H>,
       metrics: Arc<ServerMetrics>,
   }
   
   impl<H> Service<Request<Body>> for McpHttpService<H> {
       // Handle HTTP requests containing MCP messages
   }
   ```

3. **Implement HTTP Transport** (2 hours)
   - Handle POST requests with JSON-RPC bodies
   - Support SSE for server-sent events
   - Maintain session state across requests
   - Add proper CORS headers for browser clients

4. **Connection Management** (1 hour)
   - Use hyper's `serve_connection` pattern
   - Graceful shutdown with drain
   - Connection limit enforcement
   - Integrate with existing session tracking

5. **Testing & Integration** (30 min)
   - Create HTTP integration test
   - Verify metrics are recorded
   - Test graceful shutdown
   - Benchmark performance

### Success Criteria
- [ ] HTTP server accepts MCP requests
- [ ] Supports both HTTP/1.1 and HTTP/2
- [ ] Metrics track HTTP connections/requests
- [ ] Graceful shutdown works properly
- [ ] One spawn per connection pattern maintained
- [ ] Integration test passes

## Files to Review First

1. Current implementation:
   - `/crates/mcp/src/server.rs` - Existing server
   - `/crates/mcp/src/connection/http.rs` - HTTP connection support
   - `/crates/mcp/src/transport/` - Transport implementations

2. Reference shadowcat patterns:
   - `/src/proxy/forward.rs` - Good hyper patterns
   - `/src/server/` - Reference server implementation

## Key Patterns from Shadowcat

### Connection Handling:
```rust
let (service, connection) = hyper::server::conn::http1()
    .serve_connection(stream, service)
    .with_upgrades();

tokio::spawn(async move {
    if let Err(e) = connection.await {
        error!("Connection error: {}", e);
    }
});
```

### Graceful Shutdown:
```rust
let graceful = connection.graceful_shutdown();
shutdown_rx.await;
graceful.await;
```

## Commands to Run

```bash
# Navigate to MCP crate
cd /crates/mcp

# Check existing HTTP implementation
rg "hyper::" --type rust src/

# Run tests after implementation
cargo test --lib server::
cargo test --test integration_server

# Check metrics work
cargo run --example http_server_demo
```

## Next Steps After 1.2

- Task 1.3: Basic Hyper Client (6h)
- Task 1.4: Session Manager Core (8h)
- Task 1.5: Memory Session Store (4h)

Sprint 1 should deliver a working proxy foundation with proper async patterns, observability, and Hyper-based HTTP transport.

## Notes
- We're ahead of schedule (saved 6h from Task 1.0)
- Focus on clean Hyper integration
- Reuse existing connection abstractions where possible
- Keep metrics integration from Task 1.1
- Document any deviations from the plan