# Next Session: Critical Reverse Proxy Issues

## ⚠️ CRITICAL: block_on Deadlock Bug

**THIS MUST BE FIXED BEFORE PRODUCTION**

Location: `shadowcat/src/proxy/reverse/hyper_sse_intercepted.rs:200`
```rust
// THIS WILL DEADLOCK AT SCALE!
let runtime = tokio::runtime::Handle::current();
let processed = runtime.block_on(self.process_event(event));
```

**Impact**: System freeze at 100+ concurrent connections
**Risk**: Production outage under load

## Status Update (2025-08-18)

### What's Actually Complete ✅
- **SSE Resilience**: Implemented via EventTracker (different approach than planned)
- **Session Store**: Architecture complete with SessionStore trait
- **Event Tracking**: PersistenceWorker handling all persistence efficiently
- **Tests**: 775 unit tests passing

### What's NOT Complete ❌
1. **block_on deadlock** - Critical production blocker
2. **Hyper migration** - legacy.rs still 3,465 lines (unchanged)
3. **Integration testing** - Needed after fixes

## Priority Tasks

### Task 1: Fix block_on Deadlock 🔥 (2-3 hours)
**File**: `tasks/E.0-fix-block-on-deadlock.md`

**Approach Options**:
1. State machine pattern with pending futures
2. Spawn task with channel communication
3. Pre-process events before streaming

**Quick Check**:
```bash
rg "block_on" shadowcat/src/proxy/reverse/hyper_sse_intercepted.rs
```

### Task 2: Complete Hyper Migration (13-18 hours)
**File**: `tasks/E.1-complete-hyper-migration.md`

**Current State**:
- legacy.rs: 3,465 lines (should be 0)
- Hyper modules: Created but minimally used
- Target: Delete legacy.rs entirely

**Module Breakdown Needed**:
- Server setup (~500 lines) → server.rs, config.rs
- Request routing (~800 lines) → handlers/
- Middleware (~600 lines) → middleware/
- Admin UI (~876 lines) → admin/

### Task 3: Integration Testing (3 hours)
**File**: `tasks/E.2-integration-testing.md`

**After block_on fix**:
- Test 100+ concurrent SSE streams
- Verify no deadlocks with tokio-console
- Load test with mixed traffic
- Memory profiling

## Other Options (After Critical Fixes)

### Multi-Session Forward Proxy
- Forward proxy only supports single client
- Needs concurrent client support
- Plan in `plans/multi-session-forward-proxy/`

### Redis Session Store
- Implement RedisSessionStore trait
- Enable distributed deployments
- ~4 hours implementation

## Immediate Actions

```bash
# 1. See the problem
cd shadowcat
rg "block_on" src/

# 2. Check tests still pass
cargo test --lib

# 3. Start fixing (see task E.0)
code src/proxy/reverse/hyper_sse_intercepted.rs

# 4. Test concurrency after fix
RUST_LOG=debug cargo test test_concurrent_sse_streams -- --nocapture
```

## Success Metrics
- ✅ No block_on in async contexts
- ✅ 100+ connections without deadlock
- ✅ All tests passing
- ✅ Memory <100KB per session
- ✅ legacy.rs deleted

## Why This Matters
The block_on issue is a **ticking time bomb**. It will cause production outages when traffic increases. This is not theoretical - it WILL happen at scale.

Fix the deadlock first, then continue with the refactor.