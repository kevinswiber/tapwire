# Next Session Prompt - Continue Critical Fixes

## Context
We’ve fixed the most critical performance regression in the reverse proxy refactor. The connection pool now properly reuses connections, resolving the 90% throughput loss for stdio transport.

## What We've Completed (Session 9)

### ✅ COMPLETE: Connection Pool Fix (sqlx-style)
**Journey**:
1. Found root cause: Drop impl shutting down on ANY clone drop
2. Initial fix: Removed Drop (worked but no cleanup)
3. GPT-5 review: Suggested inner Arc pattern for proper cleanup
4. Intermediate: Inner Arc pattern with last-reference Drop
5. **Final implementation**: Inner-Arc + weak-backed maintenance loop, plus last-ref async cleanup backstop

**Results**:
- Pool correctly reuses connections (1 subprocess for N sequential requests for persistent stdio servers)
- Maintenance loop owns the return channel and uses Weak<..> - no circular reference
- Drop correctly detects last user reference and triggers async cleanup (best-effort)
- Backpressure-safe return path: on return-channel error, close connection with timeout, then decrement active
- Follows industry best practices from sqlx

### Applied All GPT-5 and SQLx Best Practices
1. **✅ Inner-Arc Pattern** – Clean last-reference detection
2. **✅ Weak-backed Maintenance** – No circular dependencies
3. **✅ OwnedSemaphorePermit** – Prevents semaphore leaks
4. **✅ Receiver Ownership** – Direct ownership in maintenance task (no Arc<Mutex<Receiver>>)
5. **⚠️ Subprocess Health Follow-up** – Still needed: mark disconnected on stdout EOF; optional child.try_wait()
6. **✅ No Await-in-Lock** – Idle cleanup drains, processes off-lock
7. **✅ Async Cleanup Backstop** – Best-effort shutdown on last Drop
8. **✅ Backpressure-safe Return** – Close with timeout if return channel full/closed
9. **Docs** – Clarify Drop vs. shutdown() semantics; document stdio single-shot CLI limitation

## Files to Examine (if needed)
```bash
cd /Users/kevin/src/tapwire/shadowcat
git checkout refactor/legacy-reverse-proxy

# Core pool implementation
src/proxy/pool.rs           # Latest pool implementation (fixed)
src/transport/outgoing/subprocess.rs  # Update health semantics (see below)

# Test showing the issue
tests/test_pool_reuse_integration.rs  # Simple test that should reuse
```

git checkout refactor/legacy-reverse-proxy
```

## Immediate Next Steps (Critical Issues)

### H.1: Fix Stdio Subprocess Spawning – Health Semantics (2h) 🔴 CRITICAL
Persistent stdio servers can now be reused via the pool; ensure subprocess health is accurate so dead connections aren’t returned to idle.

Actions:
- In `shadowcat/src/transport/outgoing/subprocess.rs`:
  - In `receive_response()`, set `self.connected = false` when `rx.recv().await` returns `None` (stdout closed).
  - Optionally check `child.try_wait()` in `is_connected()`; if the process has exited, return `false`.
- Tests: Persistent server reuse with `max_connections=1`; single-shot CLI returns should not be added to idle and should not leak.

### H.2: Add Server Drop/Shutdown Implementation (2h) 🔴 CRITICAL
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
SSE connections don't reconnect on failure.

Actions:
- Integrate existing module `shadowcat/src/transport/sse/reconnect.rs` instead of re-implementing reconnection in reverse-stream layer.
- Define policy: exponential backoff, full jitter, max backoff cap, reset on success.
- Tests: Use deterministic timers; ensure no thundering herd and no duplicate subscriptions.

### H.6: Add Request Timeouts (3h) 🟡 HIGH
Separate connect/request/response timeouts with sensible defaults and overrides. Ensure retries respect overall deadlines.

### H.9: Performance Benchmarks (3h) 🟡 HIGH
Add a lightweight benchmark harness (wrk/k6 or a Rust microbench) to measure p50/p95 latency and throughput for stdio and HTTP paths with/without pool reuse. Store artifacts for diffs.

## Test Commands
```bash
# Run integration tests to verify fixes
cargo test --test integration_reverse_proxy

# Check for performance improvements
cargo bench reverse_proxy
```

## Success Criteria
- [x] Connection pool properly reuses connections ✅
- [x] No subprocess spawning overhead for persistent stdio ✅
- [ ] Subprocess health semantics updated; single-shot CLIs not reused
- [ ] Server properly cleans up resources on shutdown
- [ ] P95 latency within 5% of legacy implementation
- [ ] SSE connections automatically reconnect (policy + tests)
- [ ] All tests passing

## References
- Tracker: `plans/refactor-legacy-reverse-proxy/refactor-legacy-reverse-proxy-tracker.md`
- Critical issues: `plans/refactor-legacy-reverse-proxy/tasks/phase-h-fixes/`
- GPT findings: `plans/refactor-legacy-reverse-proxy/gpt-findings/`
