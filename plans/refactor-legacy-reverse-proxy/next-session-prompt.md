# Next Session Prompt - Continue Critical Fixes

## Context
We've successfully fixed the most critical performance regression in the reverse proxy refactor. The connection pool now properly reuses connections, resolving the 90% throughput loss for stdio transport.

## What We've Completed (Session 9)

### ✅ COMPLETE: Connection Pool Fix with Inner Arc Pattern
**Journey**:
1. Found root cause: Drop impl shutting down on ANY clone drop
2. Initial fix: Removed Drop (worked but no cleanup)
3. GPT-5 review: Suggested inner Arc pattern for proper cleanup
4. **Final implementation**: Perfect inner Arc pattern with last-reference Drop

**Results**:
- Pool correctly reuses connections (1 subprocess for N requests)
- Drop only triggers on last ConnectionPool reference
- Automatic cleanup as safety net
- All tests pass including new last-reference tests

### Applied All GPT-5 Recommendations
1. **✅ Inner Arc Pattern** - Clean last-reference detection
2. **✅ Fixed Semaphore Leak** - OwnedSemaphorePermit
3. **✅ Fixed Receiver Pattern** - Direct ownership in maintenance task
4. **✅ Fixed Subprocess Health** - Disconnection detection
5. **✅ Fixed Lock Contention** - No await while holding locks
6. **✅ Added comprehensive tests** - Reuse and cleanup verification

## Files to Examine
```bash
cd /Users/kevin/src/tapwire/shadowcat
git checkout refactor/legacy-reverse-proxy

# Core pool implementation
src/proxy/pool.rs:280-318  # maintenance_loop - should be receiving
src/proxy/pool.rs:320-350  # process_returned_connection - should add to idle
src/proxy/pool.rs:248-274  # get_idle_connection - should find reusable

# Test showing the issue
tests/test_pool_reuse_integration.rs  # Simple test that should reuse
```

git checkout refactor/legacy-reverse-proxy
```

## Immediate Next Steps (Critical Issues)

### H.2: Add Server Drop Implementation (2h) 🔴 CRITICAL
The reverse proxy server lacks a Drop trait implementation. Without it:
- Connection pools won't call shutdown() (though inner Arc provides safety net)
- Background tasks continue after server drops
- Potential resource leaks in production

**Implementation needed**:
```rust
impl Drop for ReverseProxyServer {
    fn drop(&mut self) {
        // 1. Call pool.shutdown() for all pools
        // 2. Cancel/abort background tasks
        // 3. Close session store connections
        // 4. Wait for graceful shutdown
    }
}
```

### H.3: Investigate P95 Latency (2h) 🔴 CRITICAL
While we fixed stdio throughput, p95 latency is still 140% higher. Need to:
- Profile the request path
- Check for hidden blocking operations
- Verify no double-buffering in SSE path
- Benchmark against legacy implementation

### H.4: Deduplicate AppState Creation (1h) 🟡 HIGH
Multiple methods create AppState differently:
- Consolidate into single `AppState::new()` method
- Ensure consistent initialization across all paths

### H.5: Implement SSE Reconnection (6h) 🟡 HIGH
SSE connections don't reconnect on failure:
- Add exponential backoff retry logic
- Track connection state properly
- Implement proper error recovery

## Test Commands
```bash
# Run integration tests to verify fixes
cargo test --test integration_reverse_proxy

# Check for performance improvements
cargo bench reverse_proxy
```

## Success Criteria
- [x] Connection pool properly reuses connections ✅
- [x] No subprocess spawning overhead ✅ 
- [ ] Server properly cleans up resources on shutdown
- [ ] P95 latency within 5% of legacy implementation
- [ ] SSE connections automatically reconnect
- [ ] All tests passing

## References
- Tracker: `plans/refactor-legacy-reverse-proxy/refactor-legacy-reverse-proxy-tracker.md`
- Critical issues: `plans/refactor-legacy-reverse-proxy/tasks/phase-h-fixes/`
- Review: `/Users/kevin/src/tapwire/reviews/refactor-legacy-reverse-proxy-review.md`