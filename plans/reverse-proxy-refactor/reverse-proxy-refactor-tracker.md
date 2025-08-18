# Reverse Proxy Refactor Tracker

## Overview
Refactoring the 3,465-line reverse proxy implementation into clean, modular components.

## Current Status - UPDATED 2025-08-18
- **Phase**: Partially Complete
- **Lines in legacy.rs**: Still 3,465 (unchanged)
- **Critical Issues**: 1 (block_on deadlock)

## What's Actually Complete ✅

### Event Tracking & SSE Resilience (via different approach)
- Implemented through `refactor-event-tracking` plan
- SessionManager creates EventTracker with persistence channel
- Single PersistenceWorker handles all persistence
- SessionStore trait enables custom backends (Redis future)
- **No ReverseProxySseManager needed** - cleaner architecture

### What Works Now
- EventTracker integrated in SessionManager
- SSE events recorded with deduplication
- Last-Event-Id support for reconnection
- Channel-based backpressure
- All 775 unit tests passing

## What's NOT Complete ❌

### Critical Issues

#### 1. block_on Deadlock Bug 🔥
**Location**: `shadowcat/src/proxy/reverse/hyper_sse_intercepted.rs:200`
```rust
let runtime = tokio::runtime::Handle::current();
let processed = runtime.block_on(self.process_event(event));  // WILL DEADLOCK!
```
**Impact**: System freeze at 100+ connections
**Fix**: Need state machine pattern or spawn tasks

#### 2. Hyper Migration Incomplete
**Status**: Created modules but still using legacy.rs
- `legacy.rs`: Still 3,465 lines (was supposed to be deleted)
- Hyper modules: Created but minimally integrated
- Main logic: Still in legacy.rs

### Module Status

| Module | Created | Integrated | Lines |
|--------|---------|------------|-------|
| hyper_client.rs | ✅ | ✅ | ~100 |
| hyper_raw_streaming.rs | ✅ | ⚠️ Partial | ~150 |
| hyper_sse_intercepted.rs | ✅ | ⚠️ Has block_on bug | ~300 |
| json_processing.rs | ✅ | ⚠️ Partial | ~200 |
| upstream_response.rs | ✅ | ✅ | ~100 |
| **legacy.rs** | N/A | Still main impl | **3,465** |

### What's Still in Legacy
1. **Server Setup** (~500 lines)
   - ReverseProxyServer struct
   - Configuration types
   - Builder pattern

2. **Request Routing** (~800 lines)
   - handle_mcp_request
   - handle_mcp_sse_request
   - Router creation

3. **Middleware Stack** (~600 lines)
   - Authentication integration
   - Rate limiting
   - CORS/tracing

4. **Admin Interface** (~876 lines)
   - Dashboard HTML
   - Admin routes
   - Metrics endpoints

5. **Session/Interceptor Integration** (~400 lines)
   - Session management hookup
   - Interceptor chain application

## Remaining Tasks

### Task E.0: Fix block_on Deadlock 🔥 (2-3 hours)
- Remove blocking call in Stream implementation
- Implement state machine or spawn pattern
- Test with 100+ concurrent connections
- **File**: `tasks/E.0-fix-block-on-deadlock.md`

### Task E.1: Complete Hyper Migration (13-18 hours)
- Extract modules from legacy.rs
- Target: Delete legacy.rs completely
- Keep modules under 500 lines each
- **File**: `tasks/E.1-complete-hyper-migration.md`

### Task E.2: Integration Testing (2-3 hours)
- Test SSE resilience with new EventTracker
- Verify hyper modules work together
- Load test after block_on fix
- **File**: To be created

## Success Criteria
- [ ] No block_on in async contexts
- [ ] legacy.rs deleted
- [ ] All modules < 500 lines
- [ ] 100+ concurrent connections stable
- [ ] All tests passing

## Key Decisions Made

### Abandoned Original SSE Approach
- **Planned**: ReverseProxySseManager, complex integration
- **Actual**: EventTracker via SessionManager, cleaner design
- **Result**: Better architecture, already working

### Hyper Migration Strategy
- Start with working code (even if in legacy)
- Extract incrementally
- Keep tests passing at each step
- Delete legacy.rs only when empty

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| block_on deadlock | CRITICAL | 100% at scale | Fix immediately |
| Migration breaks functionality | HIGH | Medium | Incremental approach |
| Performance regression | Medium | Low | Benchmark each change |

## Next Session Priority
1. **Fix block_on deadlock** - Production blocker
2. **Start hyper migration** - Technical debt
3. **Integration tests** - Verify resilience

## Documentation Trail
- Original plan: `plans/reverse-proxy-refactor/`
- Event tracking: `plans/refactor-event-tracking/` (COMPLETE)
- Code review: `plans/reverse-proxy-refactor/reviews/`