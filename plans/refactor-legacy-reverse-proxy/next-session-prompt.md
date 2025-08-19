# Next Session Prompt - Continue Critical Fixes

## Context
We've made significant progress on the critical issues from the reverse proxy refactor review. The connection pool now properly reuses connections, subprocess health detection works correctly, and the server has proper resource cleanup.

## What We've Completed (Session 10)

### ✅ COMPLETE: H.1 - Fix Stdio Subprocess Health Semantics (2h)
**Implementation**:
1. Wrapped child process in `Arc<Mutex<Child>>` for thread-safe status checking
2. Updated `is_connected()` to use `try_wait()` to detect exited processes
3. Single-shot CLI commands (like `echo`) correctly marked as unhealthy and not reused
4. Persistent servers (like `cat`) properly remain in pool for reuse

**Results**:
- Tests verify correct behavior for both single-shot and persistent processes
- No more leaked subprocesses from single-shot commands
- Pool correctly distinguishes between reusable and non-reusable connections

### ✅ COMPLETE: H.2 - Add Server Drop Implementation (2h)
**Implementation**:
1. Added `Drop` trait to `ReverseProxyServer`
2. Properly shuts down connection pools on drop (spawns async task)
3. Aborts server task handle if running
4. Changed router field to `Option<Router>` to allow moving out

**Results**:
- Resources properly cleaned up when server is dropped
- No more leaked background tasks
- All integration tests still pass

## Files to Examine (if needed)
```bash
cd /Users/kevin/src/tapwire/shadowcat
git checkout refactor/legacy-reverse-proxy

# Core implementations fixed
src/transport/outgoing/subprocess.rs  # Health detection logic
src/proxy/reverse/server.rs          # Drop implementation
tests/test_subprocess_health.rs      # New tests for health detection
```

## Immediate Next Steps (Critical Issues Remaining)

### QUESTION: What should we focus on next?

We have several critical and high-priority items remaining. Here are the top candidates:

#### Option 1: H.3 - Investigate P95 Latency (2h) 🔴 CRITICAL
While we fixed stdio throughput, p95 latency is still 140% higher than legacy. This needs investigation:
- Profile the request path to find bottlenecks
- Check for hidden blocking operations
- Verify no double-buffering in SSE path
- Benchmark against legacy implementation

#### Option 2: H.4 - Deduplicate AppState Creation (1h) 🔴 CRITICAL
Multiple methods create AppState differently, leading to inconsistency:
- Consolidate into single `AppState::new()` method
- Ensure consistent initialization across all paths
- Simplify server initialization code

#### Option 3: H.5 - Implement SSE Reconnection (6h) 🔴 CRITICAL
SSE connections don't reconnect on failure, breaking resilience:
- Integrate existing `shadowcat/src/transport/sse/reconnect.rs`
- Define policy: exponential backoff, full jitter, max backoff cap
- Tests: deterministic timers, no thundering herd, no duplicate subscriptions

#### Option 4: H.6 - Add Request Timeouts (3h) 🟡 HIGH
Separate connect/request/response timeouts with sensible defaults:
- Add timeout configuration to all upstream implementations
- Ensure retries respect overall deadlines
- Test timeout behavior under various conditions

#### Option 5: H.9 - Performance Benchmarks (3h) 🟡 HIGH
Add benchmark harness to validate our fixes:
- Measure p50/p95 latency and throughput for stdio and HTTP paths
- Compare with/without pool reuse
- Store artifacts for regression detection

## Test Commands
```bash
# Run integration tests to verify fixes
cargo test --test integration_reverse_proxy_http

# Check subprocess health tests
cargo test --test test_subprocess_health

# Check for performance improvements  
cargo bench reverse_proxy
```

## Success Criteria Progress
- [x] Connection pool properly reuses connections ✅
- [x] No subprocess spawning overhead for persistent stdio ✅
- [x] Subprocess health semantics updated ✅
- [x] Server properly cleans up resources on shutdown ✅
- [ ] P95 latency within 5% of legacy implementation
- [ ] SSE connections automatically reconnect
- [ ] All tests passing with full coverage
- [ ] Breaking changes documented

## References
- Tracker: `plans/refactor-legacy-reverse-proxy/refactor-legacy-reverse-proxy-tracker.md`
- Critical issues: `plans/refactor-legacy-reverse-proxy/tasks/phase-h-fixes/`
- GPT findings: `plans/refactor-legacy-reverse-proxy/gpt-findings/`
- Review docs: `plans/refactor-legacy-reverse-proxy/reviews/`

## Recommendation
I recommend focusing on **Option 1: H.3 - Investigate P95 Latency** next, as performance is a critical production concern and we need to understand where the remaining overhead is coming from. Once we identify the bottleneck, we can likely fix it quickly.

After that, **Option 3: H.5 - SSE Reconnection** would be the next priority as it's a critical resilience feature that's currently missing.

What would you like to work on next?