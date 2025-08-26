# Next Session Prompt - Sprint 1 Task 1.1 Observability

## Session Goal
Continue Sprint 1 - Add basic observability to MCP client and server using OpenTelemetry with Prometheus.

## Context
- ✅ Task 1.0 Complete: Async patterns are already optimal (no changes needed)
- Now implementing Task 1.1: Basic Observability Setup
- Following v2 tracker, referencing v1 Task E.3 for details
- Time saved from 1.0 can be used for comprehensive metrics

## Current Status

### ✅ Completed (Task 1.0)
- Analyzed all async patterns in MCP crate
- Found spawns are already optimized (bounded executor pattern)
- Fixed minor clippy warning (_connection_sender)
- All tests passing (93/93)

## Sprint 1 Task 1.1: Basic Observability Setup (6h) ⭐ CRITICAL

**Reference**: v1 Task E.3 (`tasks/E.3-observability.md`)

### Implementation Plan

1. **Add Dependencies** (30 min)
```toml
[dependencies]
opentelemetry = { version = "0.24", features = ["metrics"] }
opentelemetry_sdk = { version = "0.24", features = ["rt-tokio"] }
opentelemetry-prometheus = "0.17"
prometheus = "0.13"
```

2. **Create Metrics Module** (1 hour)
- `src/metrics/mod.rs` - Metrics registry
- `src/metrics/server.rs` - Server-specific metrics
- `src/metrics/client.rs` - Client-specific metrics
- `src/metrics/pool.rs` - Pool metrics integration

3. **Core Metrics to Implement** (2 hours)

**Server Metrics:**
- `mcp_server_connections_total` - Total connections accepted
- `mcp_server_connections_active` - Currently active connections
- `mcp_server_requests_total` - Total requests by method
- `mcp_server_request_duration_seconds` - Request processing time
- `mcp_server_errors_total` - Errors by type

**Client Metrics:**
- `mcp_client_requests_total` - Total requests sent
- `mcp_client_request_duration_seconds` - Request round-trip time
- `mcp_client_pool_connections_active` - Active pooled connections
- `mcp_client_pool_connections_created_total` - New connections created
- `mcp_client_errors_total` - Client errors by type

**Pool Metrics (existing):**
- Already has metrics in `pool::metrics` module
- Need to expose via OpenTelemetry

4. **Metrics Endpoint** (1.5 hours)
- Add `/metrics` HTTP endpoint
- Use existing HTTP server infrastructure
- Serve Prometheus text format

5. **Integration & Testing** (1 hour)
- Add metrics to key code paths
- Create example with metrics
- Test metrics collection

### Success Criteria
- [ ] Metrics endpoint serves at `/metrics`
- [ ] Basic metrics visible in Prometheus format
- [ ] No performance regression (< 2% overhead)
- [ ] Example program demonstrates metrics
- [ ] Tests verify metric updates

## Execution Plan

### If 8-Hour Session:
1. Complete Task 1.0 (Fix Async Antipatterns)
2. Run comprehensive tests
3. Document changes in tracker

### If 12-Hour Session:
1. Complete Task 1.0 (8h)
2. Complete Task 1.1 (4h) - Just core metrics setup

### If 16-Hour Session:
1. Complete Task 1.0 (8h)
2. Complete Task 1.1 (6h)
3. Start Task 1.2 (2h) - Basic setup

## Files to Review First

1. Read existing async patterns:
   - `/crates/mcp/src/server/mod.rs`
   - `/crates/mcp/src/client/mod.rs`
   - `/crates/mcp/src/transport/`

2. Check task details:
   - `/plans/mcp-unified-architecture/tasks/B.0-fix-async-antipatterns.md`
   - `/plans/mcp-unified-architecture/analysis/spawn-audit.md`

3. Review shadowcat patterns:
   - `/src/proxy/forward.rs` (good hyper patterns)
   - `/src/server/` (reference implementation)

## Key Patterns to Apply

### Remove block_on:
```rust
// ❌ BAD
runtime.block_on(async_function())

// ✅ GOOD
async_function().await
```

### Fix lock scope:
```rust
// ❌ BAD
let guard = mutex.lock().await;
something_async().await; // Lock held!

// ✅ GOOD
let data = {
    let guard = mutex.lock().await;
    guard.clone()
}; // Lock released
something_async().await;
```

### Reduce spawns:
```rust
// ❌ BAD
tokio::spawn(handle_connection());
tokio::spawn(process_request());
tokio::spawn(send_response());

// ✅ GOOD
tokio::spawn(async move {
    handle_connection().await;
    process_request().await;
    send_response().await;
});
```

## Success Metrics

### After Task 1.0:
- [ ] All `block_on` removed from async code
- [ ] No locks held across await points  
- [ ] Task spawns reduced by 50%+
- [ ] All async clippy warnings fixed
- [ ] Tests passing

### After Task 1.1:
- [ ] Prometheus endpoint serving at `/metrics`
- [ ] Basic metrics visible
- [ ] No external dependencies added
- [ ] Metrics have minimal overhead (<2% CPU)

### After Task 1.2:
- [ ] Hyper server accepting connections
- [ ] One spawn per connection
- [ ] Basic request handling working
- [ ] Integration test passing

## Commands to Run

```bash
# Start in MCP crate
cd /crates/mcp

# Check current issues
cargo clippy --all-targets -- -D warnings 2>&1 | grep -E "(block_on|held across)"

# Find block_on usage
rg "block_on" --type rust

# Find lock issues  
rg "\.lock\(\)|\.write\(\)|\.read\(\)" --type rust -A 3 | grep -B 2 "\.await"

# Test after fixes
cargo test --lib
cargo test --test integration

# Verify spawn reduction
rg "tokio::spawn|task::spawn" --type rust --count
```

## Notes
- Start with Task 1.0 as it's foundational
- Don't worry about perfection - we want working code
- Reference shadowcat's patterns but adapt for MCP
- Keep changes incremental and testable
- Update tracker after each task completion

## If You Get Stuck
1. Check shadowcat's forward proxy for hyper patterns
2. Review the spawn audit document
3. Look at v1 task files for detailed requirements
4. Focus on critical path - skip nice-to-haves

Ready to start with Sprint 1!