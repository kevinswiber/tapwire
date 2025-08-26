# Sprint 1.0: Async Antipatterns Found in MCP Crate

## Summary
- **block_on**: ✅ None in src/ (only in benchmarks - acceptable)
- **tokio::spawn**: ❌ 26 instances found in src/
- **Locks across await**: ⚠️ Several potential issues found

## Spawn Analysis

### 1. Pool Module (src/pool/) - 14 spawns
**Location**: Most spawns are in the pool module for resource management

#### Connection Returns (3 spawns)
- `connection.rs:95` - Spawns to close old resources when replacing
- `connection.rs:101` - Spawns async return when not in runtime context  
- `connection.rs:119` - Spawns to close excess idle connections

**Issue**: These spawns are for fire-and-forget cleanup operations. Could be handled better with a dedicated cleanup task.

#### Tests (4 spawns)
- `tests.rs` - Various test spawns (acceptable in tests)

#### Maintenance Task (1 spawn)
- `mod.rs:497` - Spawns maintenance task for periodic cleanup
**Issue**: This is acceptable - single long-running maintenance task per pool

#### Cleanup Operations (3 spawns)
- `mod.rs:537` - Spawns for async cleanup during drop
- `mod.rs:654` - Spawns to close expired resources

**Issue**: Fire-and-forget cleanup could accumulate tasks

#### Bounded Executor (3 spawns)
- `bounded_executor.rs:51` - Spawns executor worker (acceptable - single worker)
- `bounded_executor.rs:86,90` - Fallback spawns when queue full

**Issue**: Fallback spawns could indicate overload

### 2. Server Module (src/server.rs) - 1 spawn
- Line 218: Spawns handler for each client connection
```rust
let handle = tokio::spawn(async move {
    // Main message loop for this client
    // ...
});
```
**Issue**: This follows the one-spawn-per-connection pattern, which is what we want to achieve. However, we should consider using hyper's serve_connection pattern instead.

### 3. Connection Module (src/connection/) - 3 spawns
- `http.rs:223` - Spawns HTTP/1.1 connection handler
- `http.rs:267` - Spawns HTTP/2 connection handler
- Both follow hyper pattern (acceptable)

### 4. Transport Module - 8 spawns
- SSE and streaming related spawns
- Need investigation

## Lock Issues

### 1. Pool Module
```rust
// src/pool/mod.rs:497
let handle = self.inner.maintenance_handle.lock().take();
if let Some(handle) = handle {
    let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;
}
```
**Issue**: Lock is dropped before await (✅ GOOD)

### 2. Connection Module
```rust
// src/connection/http.rs:216
let mut sender = sender.lock().await;
sender.send_request(request).await
```
**Issue**: Lock held across await (❌ BAD) - but this is a Tokio Mutex, which is designed for this

### 3. Transport SSE
Multiple instances of locks across await in SSE parser - needs investigation

## Recommendations

### Priority 1: Reduce Pool Spawns
**Current**: 14 spawns for various cleanup operations
**Target**: Single cleanup task that processes a queue
**Approach**: 
```rust
// Instead of:
tokio::spawn(async move { resource.close().await; });

// Use:
cleanup_queue.send(resource).await;
// With single worker processing the queue
```

### Priority 2: Server Connection Handling
**Current**: One spawn per client connection
**Target**: Keep this pattern but ensure it's optimal
**Note**: This is actually the correct pattern for MCP server - one task per client is appropriate

### Priority 3: Fix Lock Patterns
**Current**: Some locks held across await (mostly Tokio Mutex - acceptable)
**Target**: Verify all are Tokio Mutex or refactor to drop before await

### Priority 4: Connection Module
**Current**: Spawns for HTTP connection handling
**Target**: These follow hyper patterns - likely acceptable

## Action Plan

1. **Pool Cleanup Consolidation** (2 hours)
   - Create single cleanup worker with queue
   - Replace individual spawns with queue sends
   - Keep maintenance task spawn (it's a singleton)

2. **Server Pattern Review** (1 hour)
   - Current pattern is actually correct for MCP
   - Document why one-spawn-per-client is appropriate
   - No changes needed

3. **Lock Audit** (1 hour)
   - Verify all async locks are Tokio Mutex
   - Document any sync Mutex usage
   - Fix any problematic patterns

4. **Transport SSE Review** (1 hour)
   - Investigate SSE spawns
   - Determine if consolidation possible

## Metrics
- Current spawns: 26
- Target spawns: ~10-15 (mostly connection handlers)
- Reduction: ~40-50%

## Notes
- The MCP client already uses pooling correctly (no spawns)
- The MCP server's one-spawn-per-client is appropriate for the protocol
- Most problematic spawns are in pool cleanup operations
- No block_on issues found (excellent!)