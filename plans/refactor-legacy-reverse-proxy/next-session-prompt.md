# Next Session Prompt - URGENT: Critical Fixes for Reverse Proxy Refactor

## ⚠️ CRITICAL CONTEXT

**The refactoring is architecturally complete BUT has critical production-blocking issues!**

A comprehensive review (2025-08-18) found:
- **Resource leaks** that will cause production outages
- **Performance regressions** exceeding 140% at p95 latency
- **Missing core features** (SSE reconnection, admin endpoints)
- **90% throughput loss** for stdio transport

## 🔴 Your Mission: Fix Critical Issues (Day 1 - 8 hours)

### Task H.0: Fix Connection Pool Leak (2 hours) 
**File**: `src/proxy/reverse/upstream/pool.rs:56-60`
```rust
// BROKEN - Connections leak when return channel fails
impl<T> Drop for PooledConnection<T> {
    fn drop(&mut self) {
        let _ = self.pool.return_tx.send(connection); // Silent failure!
    }
}
```
**Fix**: Implement proper cleanup with spawned task to ensure return or count decrement
**Details**: `/plans/refactor-legacy-reverse-proxy/tasks/phase-h-fixes/H.0-fix-connection-pool-leak.md`

### Task H.1: Fix Stdio Subprocess Spawning (4 hours)
**File**: `src/proxy/reverse/upstream/stdio.rs:87-106`  
**Problem**: Creates NEW process per request instead of reusing connections!
**Impact**: 90% throughput reduction, 10ms overhead per request
**Fix**: Implement true connection reuse or process pool
**Details**: `/plans/refactor-legacy-reverse-proxy/tasks/phase-h-fixes/H.1-fix-stdio-subprocess-spawning.md`

### Task H.2: Add Server Drop Implementation (2 hours)
**File**: `src/proxy/reverse/server.rs`
**Problem**: No resource cleanup on shutdown - tasks keep running, pools leak
**Fix**: Implement Drop trait to abort tasks, flush recorder, close pools
**Details**: `/plans/refactor-legacy-reverse-proxy/tasks/phase-h-fixes/H.2-add-server-drop-implementation.md`

## Working Directory & Branch
```bash
cd /Users/kevin/src/tapwire/shadowcat
git checkout refactor/legacy-reverse-proxy
```

## Validation After Each Fix
```bash
# Test for leaks
cargo test pool_tests::test_connection_pool_leak_prevention

# Check performance
cargo bench --bench reverse_proxy

# Verify no clippy warnings
cargo clippy --all-targets -- -D warnings
```

## Success Criteria for Day 1
- [ ] Connection pool Drop fixed - no leaks under pressure
- [ ] Stdio reuses connections - <20ms per request
- [ ] Server Drop implemented - clean shutdown verified
- [ ] All tests passing
- [ ] Memory stable under load test

## Critical Resources
- **Full Review**: `/plans/refactor-legacy-reverse-proxy/reviews/`
- **All Tasks**: `/plans/refactor-legacy-reverse-proxy/tasks/phase-h-fixes/`
- **Tracker**: `/plans/refactor-legacy-reverse-proxy/refactor-legacy-reverse-proxy-tracker.md`

## DO NOT
- Skip tests for any fix
- Mark complete without verification
- Merge to main until ALL critical issues fixed

## Priority Order
1. H.0 first (affects all upstreams)
2. H.1 second (biggest perf impact)  
3. H.2 third (prevents resource cleanup)

Each task file has detailed implementation steps, code examples, and test cases. Follow them exactly.

**This is blocking production deployment. Focus and fix these issues!**