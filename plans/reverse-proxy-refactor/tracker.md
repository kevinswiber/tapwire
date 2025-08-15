# Reverse Proxy Refactor - Implementation Tracker

## Project Status
**Status**: ✅ Analysis Complete - Ready for Implementation  
**Started**: 2025-01-15  
**Last Updated**: 2025-01-15  
**Estimated Duration**: 20-23 hours (with optional parallelization saving 3 hours)
**Progress**: Phase A Complete (~5 hours)

## Executive Summary

### Critical Discoveries from Analysis
1. **SSE Bug**: Proxy makes duplicate requests for SSE streams (wasteful, causes timeouts)
2. **No SessionStore Trait**: Direct coupling to InMemorySessionStore blocks distributed sessions
3. **Module Size**: 3,482 lines in single file (admin handler alone is 876 lines)

### Execution Strategy - Foundation First

**Build the Right Architecture from the Start:**

1. **Phase B (First)**: SessionStore Abstraction (4-5 hours)
   - Extract trait with distributed sessions in mind
   - Refactor InMemoryStore to implement trait
   - Design for Redis but don't implement yet
   - This unblocks everything else

2. **Phase C (Second)**: Fix SSE Bug Properly (4-6 hours)
   - Implement full UpstreamResponse wrapper solution
   - No quick hacks - align with target architecture
   - Integrate with new SessionStore trait

3. **Phase D**: Modularization (8-10 hours)
   - Break up the 3,482-line monolith
   - Extract admin interface (876 lines)
   - Create clean module boundaries

4. **Phase E**: Integration & Testing (4-6 hours)

### Why This Approach?
- **Avoids technical debt**: No quick patches that need rework
- **Foundation first**: SessionStore enables everything else
- **Aligned implementation**: SSE fix uses proper abstractions
- **Redis-ready**: Can add Redis backend later without refactoring

## Context
The reverse proxy in `src/proxy/reverse.rs` has grown to 3,482 lines and has architectural issues with SSE streaming. This refactor will modularize the code, fix SSE handling, and implement proper session mapping.

## Related Plans (Critical Dependencies)
- **[Redis Session Storage](../redis-session-storage/)**: Session storage abstraction for distributed deployments
- **[Reverse Proxy Session Mapping](../reverse-proxy-session-mapping/)**: Dual session ID tracking for proxy scenarios

These plans are CRITICAL because the proxy must:
1. Support distributed session storage (not just in-memory)
2. Maintain separate proxy and upstream session IDs
3. Route requests to correct persistent sessions
4. Handle SSE reconnection with event replay
5. Support connection pooling and upstream failover

## Key Findings from Analysis

### Critical Bug Identified
**SSE Buffering Issue** (Lines 2312-2454, 1289-1311):
- Proxy attempts to buffer infinite SSE streams causing timeouts
- Makes duplicate requests as workaround (wasteful)
- Root cause: Function signatures expect `ProtocolMessage`, incompatible with streaming

### Immediate Fix Required
- Detect SSE early via Accept header BEFORE making upstream request
- Branch to separate streaming path that doesn't attempt buffering
- Eliminate duplicate request anti-pattern

### Module Size Issues
- `handle_admin_request()`: 876 lines (needs major refactor)
- `handle_mcp_request()`: 567 lines (should be split)
- Total file: 3,482 lines (target: ~500 lines per module)

## Phase A: Analysis & Architecture (6-8 hours)

### A.0: Code Analysis (2 hours)
**Goal**: Complete understanding of current implementation  
**Status**: ✅ **COMPLETE** (2025-01-15)

**Deliverables**:
- ✅ `analysis/current-architecture.md` - Complete code map with 3,482 line analysis
- ✅ `analysis/dependencies.md` - External dependencies and interfaces documented
- ✅ `analysis/state-management.md` - Shared state and concurrency patterns analyzed
- ✅ `analysis/sse-comparison.md` - SSE implementation comparison with references
- ✅ `analysis/findings-summary.md` - Executive summary with recommendations

### A.1: SSE Infrastructure Review (1.5 hours)
**Goal**: Understand existing SSE modules for distributed session support  
**Status**: ✅ **COMPLETE** (2025-01-15)

**Key Findings**:
- ✅ SseParser and SseStream are directly reusable
- ❌ No SessionStore trait exists - direct coupling to InMemorySessionStore
- ❌ Session management not abstracted for distributed backends
- ⚠️ SSE reconnection logic needs distributed Last-Event-Id support

**Deliverables**:
- ✅ `analysis/sse-infrastructure.md` - Critical gaps identified for distributed sessions
- ✅ `analysis/corrected-sse-solution.md` - Proper SSE handling with UpstreamResponse wrapper

### A.2: Reconciled Architecture Plan (2-3 hours)
**Goal**: Create unified plan addressing all discovered issues  
**Status**: ✅ **COMPLETE** (2025-01-15)

**Critical Decisions Made**:
1. **SessionStore Priority**: Prerequisite - must be done first
2. **Fix Approach**: Full UpstreamResponse implementation
3. **Distributed Sessions**: Design for it, implement later

**Deliverables**:
- ✅ `analysis/unified-plan.md` - Single source of truth for implementation
- ✅ `analysis/implementation-requirements.md` - Detailed requirements and questions
- ✅ Updated tracker with clear sequential phases

## Phase B: SessionStore Abstraction (4-5 hours) - PREREQUISITE

**Must be completed first to unblock everything else**

### B.0: Design SessionStore Trait (1 hour)
**Goal**: Create storage abstraction with distributed systems in mind  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Define SessionStore trait with core session methods
- [ ] Add SSE-specific methods (Last-Event-Id, event buffering)
- [ ] Design for async/await and connection pooling
- [ ] Document trait contract and Redis considerations

**Key Design Decisions**:
- Methods must be async for network backends
- Include batch operations for efficiency
- Support TTL/expiry for session cleanup
- Consider pagination for large result sets

### B.1: Refactor InMemoryStore (2 hours)
**Goal**: Implement trait for existing store  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Extract interface from InMemorySessionStore
- [ ] Implement SessionStore trait for InMemoryStore
- [ ] Update SessionManager to use trait instead of concrete type
- [ ] Ensure all existing tests pass

### B.2: Session Mapping Design (1-2 hours)
**Goal**: Design dual session ID architecture  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Design ProxySessionMapping structure
- [ ] Add methods for proxy↔upstream mapping
- [ ] Plan session lifecycle with distributed store
- [ ] Document reconnection scenarios

**Note**: Redis implementation deferred to later phase

## Phase C: Fix SSE Bug Properly (4-6 hours)

**With SessionStore abstraction in place, we can fix the bug properly**

### C.0: Implement UpstreamResponse Wrapper (2 hours)
**Goal**: Fix the duplicate request bug with proper architecture  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Create `UpstreamResponse` struct with Response + metadata
- [ ] Modify `process_via_http()` to return UpstreamResponse
- [ ] Update callers to branch based on content-type
- [ ] Remove `SseStreamingRequired` error hack

### C.1: SSE Stream Processing (2 hours)
**Goal**: Stream SSE without buffering  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Integrate SseParser with bytes_stream()
- [ ] Implement streaming path for SSE
- [ ] Process events through interceptors incrementally
- [ ] Stream to client without accumulation

### C.2: Session Integration for SSE (2 hours)
**Goal**: Proper session tracking using SessionStore  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Track Last-Event-Id via SessionStore trait
- [ ] Handle SSE reconnection with proper abstractions
- [ ] Map proxy session to upstream session
- [ ] Test with MCP Inspector

## Phase D: Modularization (8-10 hours)

### D.0: Create Module Structure (2 hours)
**Goal**: Set up modular organization  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Create `src/proxy/reverse/` directory structure
- [ ] Extract configuration to `config.rs`
- [ ] Extract metrics to `metrics.rs`
- [ ] Update imports

### D.1: Extract Handlers (3 hours)
**Goal**: Separate request handling logic  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Create `handlers/json.rs` for JSON processing
- [ ] Create `handlers/sse.rs` for SSE streaming
- [ ] Extract routing logic
- [ ] Add handler tests

### D.2: Extract Upstream Management (2 hours)
**Goal**: Centralize upstream logic  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Create `upstream.rs` module
- [ ] Implement connection pooling
- [ ] Add load balancing support
- [ ] Create upstream tests

### D.3: Admin Interface Separation (3 hours)
**Goal**: Move admin UI to separate module  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Create `admin/` subdirectory
- [ ] Extract admin handlers (876 lines!)
- [ ] Separate HTML templates
- [ ] Add admin tests

## Phase E: Integration & Testing (6-8 hours)

### E.0: Integration Tests (3 hours)
**Goal**: Comprehensive integration testing  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Test fixed SSE streaming without duplicate requests
- [ ] Test distributed session management with Redis
- [ ] Test session mapping and Last-Event-Id
- [ ] Test with MCP Inspector

### E.1: Performance Testing (2 hours)
**Goal**: Verify performance targets  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Benchmark SSE streaming latency
- [ ] Test session storage performance
- [ ] Measure memory usage with streaming
- [ ] Verify < 5% p95 overhead maintained

### E.2: Full Integration (3 hours)
**Goal**: Merge parallel tracks  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Integrate SessionStore with SSE streaming
- [ ] Complete session mapping implementation
- [ ] Test failover scenarios
- [ ] Update documentation

## Risk Assessment

### High Risk
- **Breaking existing functionality**: Mitigate with comprehensive tests
- **Performance regression**: Mitigate with benchmarks before/after
- **SSE streaming complexity**: Mitigate with incremental implementation

### Medium Risk
- **Module boundary violations**: Mitigate with clear interfaces
- **Session mapping edge cases**: Mitigate with extensive testing
- **Interceptor integration issues**: Mitigate with phased rollout

### Low Risk
- **Code organization issues**: Mitigate with team review
- **Documentation gaps**: Mitigate with continuous updates

## Success Metrics

### Phase B (SessionStore Foundation)
- [ ] SessionStore trait defined with async methods
- [ ] InMemoryStore implements trait
- [ ] SessionManager uses trait (not concrete type)
- [ ] All existing tests pass
- [ ] Redis-ready design (but not implemented)

### Phase C (SSE Fix)
- [ ] No duplicate requests for SSE streams
- [ ] SSE streaming without buffering/timeouts
- [ ] UpstreamResponse wrapper implemented
- [ ] Integrates with SessionStore trait
- [ ] Tested with MCP Inspector

### Phase D (Modularization)
- [ ] `reverse.rs` < 500 lines per module
- [ ] Admin interface extracted (876 lines → separate module)
- [ ] Clear module boundaries
- [ ] No circular dependencies

### Phase E (Integration)
- [ ] All refactored code integrated
- [ ] Performance maintained (< 5% p95 overhead)
- [ ] 90%+ code coverage for new code
- [ ] Documentation complete

### Future (Post-Refactor)
- [ ] Redis backend implementation
- [ ] Distributed session support
- [ ] Connection pooling
- [ ] Upstream failover

## Dependencies
- Existing SSE transport infrastructure (`src/transport/sse/`)
- Session manager (`src/session/`)
- Interceptor framework (`src/interceptor/`)
- MCP protocol implementation (`rmcp` crate)

## Next Steps

### Ready to Implement - All Decisions Made ✅

**Start Phase B immediately with these files:**
1. Create `src/session/store.rs` with SessionStore trait
2. Move InMemorySessionStore to `src/session/memory.rs`
3. Update SessionManager to reference store via Arc<dyn SessionStore>
4. Enable store injection through Shadowcat API

### Key Implementation Decisions
- SessionManager **references** store (enables injection)
- Connection pooling is **implementation-specific** 
- Parse MIME types **eagerly**
- Stream non-JSON/SSE with **backpressure**
- **No backwards compatibility** needed (refactor freely)

### Complete Documentation Available
- `analysis/unified-plan.md` - Step-by-step implementation guide
- `analysis/implementation-requirements.md` - All questions answered
- `analysis/final-decisions.md` - Summary of all decisions

## Notes
- Current SSE implementation makes duplicate requests (temporary workaround)
- `ReverseProxyError::SseStreamingRequired` should be removed after Phase C
- Focus on streaming-first architecture for SSE
- Redis implementation deferred but design must support it
- Consider using existing `SseParser` and `SseStream` from transport layer