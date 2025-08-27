# Next Session Prompt - Sprint 1 Task 1.3 Basic Hyper Client

## Session Goal
Continue Sprint 1 - Implement a basic Hyper client for the MCP library to complement the server we just built.

## Context
- ✅ Task 1.0 Complete: Async patterns already optimal (2h instead of 8h)
- ✅ Task 1.1 Complete: OpenTelemetry observability with Prometheus implemented
- ✅ Task 1.2 Complete: Basic Hyper HTTP server with HTTP/1.1 and HTTP/2 support
- 🎯 Task 1.3: Basic Hyper Client (6h)
- Following v2 tracker in `mcp-tracker-v2-critical-path.md`

## Current Status

### ✅ Completed (Sprint 1)
1. **Task 1.0 - Async Patterns**
   - Bounded executor preventing spawn explosion
   - One-spawn-per-client pattern validated

2. **Task 1.1 - Observability** 
   - OpenTelemetry + Prometheus metrics
   - Server, client, and pool metrics
   - Export via `export_metrics()` method

3. **Task 1.2 - Basic Hyper Server**
   - Hyper 1.x server implementation
   - Support for HTTP/1.1 and HTTP/2
   - One spawn per connection pattern
   - Health, metrics, and MCP endpoints
   - Demo example and integration tests

## Sprint 1 Task 1.3: Basic Hyper Client (6h) ⭐ CRITICAL

### Goal
Create a Hyper 1.x based HTTP client for MCP that matches the server implementation.

### Key Requirements
1. **Use existing HttpConnection** in `src/connection/http.rs`
2. **Integrate with connection pool** from `src/pool/`
3. **Support both HTTP/1.1 and HTTP/2**
4. **Integrate metrics** from Task 1.1
5. **Match server patterns** from Task 1.2

### Implementation Plan

1. **Review Existing HTTP Connection** (30 min)
   - Check `src/connection/http.rs` implementation
   - Understand current HTTP client patterns
   - Review connection pooling integration

2. **Enhance HTTP Client** (2 hours)
   - Update for consistency with server patterns
   - Ensure proper HTTP/2 negotiation
   - Add connection health checks
   - Integrate client metrics

3. **Pool Integration** (2 hours)
   - Ensure HttpConnection works with pool
   - Test connection reuse
   - Handle connection lifecycle properly
   - Track metrics for pooled connections

4. **Client Factory Pattern** (1 hour)
   - Create factory for HTTP connections
   - Support configuration options
   - Handle TLS properly

5. **Testing & Examples** (30 min)
   - Create client demo example
   - Integration tests with server
   - Test connection pooling
   - Verify metrics integration

### Success Criteria
- [ ] HTTP client can connect to HTTP server from Task 1.2
- [ ] Connection pooling works properly
- [ ] Metrics track client requests and connections
- [ ] Both HTTP/1.1 and HTTP/2 work
- [ ] Demo example shows client-server interaction
- [ ] Integration tests pass

## Files to Review

1. Existing implementation:
   - `/crates/mcp/src/connection/http.rs` - Current HTTP connection
   - `/crates/mcp/src/client.rs` - Client implementation
   - `/crates/mcp/src/pool/` - Connection pooling

2. Reference from Task 1.2:
   - `/crates/mcp/src/http_server.rs` - Server patterns to match
   - `/crates/mcp/examples/http_server_demo.rs` - Server example

## Commands to Run

```bash
# Navigate to MCP crate
cd ~/src/tapwire/shadowcat-mcp-compliance
cd crates/mcp

# Check existing HTTP client
rg "HttpConnection" --type rust

# Run tests
cargo test --lib connection::http
cargo test --lib client::

# Test client-server interaction
cargo run --example http_client_demo
```

## Next Steps After 1.3

- Task 1.4: Session Manager Core (8h)
- Task 1.5: Memory Session Store (4h)

Sprint 1 will deliver a working HTTP client-server foundation with proper async patterns, observability, and Hyper 1.x integration.

## Notes
- We're ahead of schedule (saved 10h so far from Tasks 1.0-1.2)
- Focus on reusing existing code where possible
- Ensure client and server work together seamlessly
- Keep metrics integration consistent