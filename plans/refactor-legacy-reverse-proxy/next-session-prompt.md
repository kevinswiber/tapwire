# Next Session Prompt - Continue Critical Fixes

## Context
We've been fixing critical issues in the reverse proxy refactor that were causing severe performance regressions (140% p95 latency, 90% throughput loss).

## What We've Completed (Sessions 8-9)

### ✅ SOLVED: Connection Pool Not Reusing Connections
**Root Cause Found**: The Drop implementation was triggering shutdown on ANY clone drop, not just the last reference. This caused the maintenance loop to shut down immediately, preventing connection reuse.

**Fix Applied**: Removed the problematic Drop implementation. The pool now correctly reuses connections.

**Verified**: Tests confirm only 1 subprocess is created for N requests (was N subprocesses before).

### Applied GPT-5's Architectural Fixes
1. **✅ Fixed Semaphore Leak** - Now uses `OwnedSemaphorePermit` tied to connection lifetime
2. **✅ Fixed Receiver Pattern** - Moved from `Arc<Mutex<Receiver>>` to direct ownership in maintenance task
3. **✅ Fixed Subprocess Health** - Marks disconnected when stdout closes or send fails
4. **✅ Fixed Lock Contention** - No more await while holding locks
5. **✅ Fixed Pool Capacity** - Was rejecting at wrong threshold
6. **✅ Fixed Drop Implementation** - Removed premature shutdown trigger

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

### H.2: Add Server Drop Implementation (2h)
The reverse proxy server lacks a Drop trait implementation for proper resource cleanup. This causes:
- Tasks continue running after shutdown
- Connection pools not properly closed
- Potential resource leaks in production

**Implementation needed**:
```rust
impl Drop for ReverseProxyServer {
    fn drop(&mut self) {
        // Shutdown all pools
        // Cancel background tasks
        // Close database connections
    }
}
```

### H.3: Deduplicate AppState Creation (1h)
Multiple methods create AppState differently, causing inconsistency.
- Consolidate into single `AppState::new()` method
- Ensure all components use same initialization

### H.4: Implement SSE Reconnection (6h)
SSE connections don't reconnect on failure. Need:
- Exponential backoff retry logic
- Connection state tracking
- Proper error recovery

## Test Commands
```bash
# Run integration tests to verify fixes
cargo test --test integration_reverse_proxy

# Check for performance improvements
cargo bench reverse_proxy
```

## Success Criteria
- [ ] Server properly cleans up resources on shutdown
- [ ] Performance within 5% of legacy implementation
- [ ] SSE connections automatically reconnect
- [ ] All tests passing

## References
- Tracker: `plans/refactor-legacy-reverse-proxy/refactor-legacy-reverse-proxy-tracker.md`
- Critical issues: `plans/refactor-legacy-reverse-proxy/tasks/phase-h-fixes/`
- Review: `/Users/kevin/src/tapwire/reviews/refactor-legacy-reverse-proxy-review.md`