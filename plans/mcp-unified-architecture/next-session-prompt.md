# Next Session Prompt - Sprint 1 Foundation

## Session Goal
Implement Sprint 1 from the Critical Path tracker - establish the core foundation with async fixes, observability, and basic hyper patterns.

## Context
- We have two trackers: v1 (comprehensive) and v2 (critical path execution)
- Following v2 for implementation, referencing v1 for details
- Goal: MVP working proxy with metrics in ~38 hours total
- This session: Focus on first 2-3 tasks (aiming for 8 hours work)

## Sprint 1 Tasks

### 1.0: Fix Async Antipatterns (8h) ⭐ CRITICAL
**Reference**: v1 Task B.0 (`tasks/B.0-fix-async-antipatterns.md`)

**Key Issues to Fix**:
- Remove `block_on` calls causing deadlocks
- Fix locks held across await points
- Reduce excessive task spawning
- Fix select! loops that never yield
- Remove unnecessary Arc<Mutex<>> where single-threaded

**Success Criteria**:
- No `block_on` in async contexts
- No locks held across await
- Spawns reduced by 50%+
- All clippy warnings resolved

### 1.1: Basic Observability Setup (6h) ⭐ CRITICAL
**Reference**: v1 Task E.3 (`tasks/E.3-observability.md`)

**Implementation**:
- OpenTelemetry with Prometheus (default)
- Basic metrics: connections, requests, latency
- Metrics endpoint at `/metrics`
- No OTLP initially (avoid tonic dependency)

**Key Metrics**:
- Connection count/duration
- Request rate/latency
- Session active/total
- Error rates

### 1.2: Basic Hyper Server (6h) ⭐ CRITICAL
**Reference**: v1 Task B.1 (partial)

**Implementation**:
- Use hyper v1 serve_connection pattern
- Single spawn per connection
- Basic HTTP/1.1 support
- Integration with session manager stub
- Graceful connection handling

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