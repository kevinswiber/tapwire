# Reverse Proxy Refactor - Implementation Tracker

## Project Status
**Status**: 🚧 SSE Reconnection Foundation Complete - Phase D.0 Done!  
**Started**: 2025-01-15  
**Last Updated**: 2025-08-16  
**Estimated Duration**: 32-35 hours (added Phase D for SSE reconnection)
**Progress**: Phases A, B, C Complete + D.0 Foundation (~26 hours)

## Executive Summary

### Critical Discoveries from Analysis
1. **SSE Bug**: Proxy makes duplicate requests for SSE streams (wasteful, causes timeouts) - ✅ FIXED
2. **No SessionStore Trait**: Direct coupling to InMemorySessionStore blocks distributed sessions - ✅ FIXED
3. **Module Size**: 3,482 lines in single file (admin handler alone is 876 lines) - ⚠️ IMPROVED (removed 1,772 lines)
4. **SSE Duplication**: Had 9 different SSE modules with 3 approaches - ✅ CONSOLIDATED to 1

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

### JSON-RPC ID Type Preservation Issue (FIXED 2025-08-15)
**Problem**: Proxy was converting all JSON-RPC IDs to strings
- Request: `{"id": 0, ...}` → Response: `{"id": "0", ...}` 
- MCP Inspector SDK stores mappings by original ID type (numeric 0)
- Response lookup fails because SDK receives string "0" back
- Error: "No connection established for request ID: 0"

**Solution**: Changed `ProtocolMessage` to preserve original JSON type
- Changed ID fields from `String` to `serde_json::Value`
- All parsing/serialization now preserves original type
- Numeric IDs stay numeric, string IDs stay strings

**✅ COMPLETE (2025-08-15)**: JsonRpcId type refactor implemented
- Created `JsonRpcId` enum with `String` and `Number` variants
- Updated entire codebase to use `JsonRpcId` instead of `Value`
- Full type safety achieved throughout the proxy
- MCP Inspector now works correctly with numeric IDs!

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

## Phase B: SessionStore Abstraction (4-5 hours) - ✅ COMPLETE

**Successfully abstracted storage to enable distributed sessions**

### B.0: Design SessionStore Trait (1 hour)
**Goal**: Create storage abstraction with distributed systems in mind  
**Status**: ✅ **COMPLETE**

**Key Architectural Decision**: Discovered that frames/MessageEnvelopes were conflated
- Frames belong in recording/tape domain
- MessageEnvelopes are live transport messages
- Removed frame storage from SessionStore entirely
- Created `analysis/frame-vs-envelope-decision.md` documenting this

**Completed Tasks**:
- ✅ Defined SessionStore trait with core session methods only
- ✅ Added SSE-specific methods (Last-Event-Id only, NO frame storage)
- ✅ Designed for async/await and connection pooling
- ✅ Documented decision to separate frames from session management

### B.1: Refactor InMemoryStore (2 hours)
**Goal**: Implement trait for existing store  
**Status**: ✅ **COMPLETE**

**Completed Tasks**:
- ✅ Moved InMemorySessionStore to `src/session/memory.rs`
- ✅ Implemented SessionStore trait for InMemoryStore
- ✅ Updated SessionManager to use `Arc<dyn SessionStore>`
- ✅ All 57 session tests passing

### B.2: Fix Compilation & Tests (2 hours)
**Goal**: Update all affected code  
**Status**: ✅ **COMPLETE**

**Completed Tasks**:
- ✅ Removed frame recording from reverse proxy
- ✅ Updated CLI to remove frame display
- ✅ Fixed all compilation errors
- ✅ Updated tests to not use frame methods

### B.3: SSE Module Consolidation (4 hours)
**Goal**: Clean up duplicate SSE implementations
**Status**: ✅ **COMPLETE**

**Completed Tasks**:
- ✅ Added interceptor support to SSE streaming
- ✅ Removed eventsource-client fallback approach  
- ✅ Deleted 7 unused SSE modules (66% code reduction)
- ✅ Consolidated to single hyper-based implementation
- ✅ Moved thresholds to transport::constants
- ✅ Fixed JsonRpcId test failures
- ✅ All 856 tests passing

### B.4: Message Recording Abstraction (TODO)
**Goal**: Design proper recording abstraction  
**Status**: 📝 **TODO** - Deferred to Phase D

**Future Work**:
- [ ] Design MessageEventReceiver trait for recording
- [ ] Proxy shouldn't know about tape format or "frames"
- [ ] Enable injection of recording implementation
- [ ] Keep recording concerns separate from proxying

## Phase C: Fix SSE Bug Properly (4-6 hours) - ✅ COMPLETE

**Successfully eliminated duplicate HTTP requests for SSE streams!**

### C.0: Implement UpstreamResponse Wrapper (2 hours)
**Goal**: Fix the duplicate request bug with proper architecture  
**Status**: ✅ **COMPLETE**

**Completed Tasks**:
- ✅ Created `UpstreamResponse` struct with Response + metadata
- ✅ Modified `process_via_http_new()` to return UpstreamResponse
- ✅ Updated callers to branch based on content-type
- ✅ Removed `SseStreamingRequired` error hack completely

### C.1: SSE Stream Processing (2 hours)
**Goal**: Stream SSE without buffering  
**Status**: ✅ **COMPLETE**

**Completed Tasks**:
- ✅ Integrated existing `SseStream` from transport layer
- ✅ Implemented streaming path for SSE without buffering
- ✅ Process events through interceptors incrementally
- ✅ Stream to client without accumulation using bounded channels

### C.2: Session Integration & Testing (2 hours)
**Goal**: Proper session tracking and validation  
**Status**: ✅ **COMPLETE**

**Completed Tasks**:
- ✅ Added Last-Event-Id support in SSE streaming
- ✅ Integrated with handle_mcp_request routing
- ✅ Fixed 202 Accepted handling (passthrough without buffering)
- ✅ Tested with MCP Inspector - found upstream issue

**Key Discovery**: The SSE connection closes after single event because **reqwest does not support long-lived SSE connections**. The `bytes_stream()` method completes when initial data is consumed rather than keeping the connection open for future events. This is a known limitation (see [reqwest#2677](https://github.com/seanmonstar/reqwest/issues/2677)).

## Phase C.5: Fix SSE Client for Long-lived Connections (4-6 hours) - ✅ COMPLETE

### Problem Statement
Reqwest's `bytes_stream()` is not designed for SSE. It treats the end of currently available data as stream completion, rather than waiting for more events. This causes SSE connections to close after receiving the first event.

**Solution Implemented**: Switched to hyper-based implementation that properly handles long-lived SSE streams.

### Critical Discovery (2025-08-15)
After implementing eventsource-client integration, we discovered:
1. **eventsource-client is incompatible with MCP's SSE pattern**:
   - eventsource-client makes its own GET requests to SSE endpoints
   - MCP returns SSE in response to POST requests with JSON-RPC bodies
   - We already have a response stream - we don't need a new connection
   
2. **The upstream server behavior**:
   - MCP example server closes connections after sending responses
   - Even with SSE format, it sends one event then closes
   - This may be expected behavior for non-subscription methods

3. **The real issue**:
   - We need to properly handle the existing response body stream
   - reqwest's `bytes_stream()` might be mishandling chunked transfer encoding
   - Solution requires working with the raw hyper body stream

### Evaluated Options
1. ~~**LaunchDarkly's eventsource-client**~~ ❌ (Incompatible)
   - Makes its own GET requests, can't use existing POST response
   - Designed for different SSE pattern than MCP uses
   
2. **Custom hyper implementation** ✅ (Now pursuing)
   - Work directly with hyper's body stream
   - Handle chunked transfer encoding properly
   - Keep connection alive as long as upstream sends data
   - Can reuse existing response without new connection

3. ~~**reqwest-eventsource**~~ ❌
   - Same fundamental issue as eventsource-client
   - Also makes its own connections

### C.5.0: ~~Implement eventsource-client Solution~~ (3 hours)
**Goal**: ~~Replace reqwest for SSE upstream connections~~
**Status**: ❌ ABANDONED - Wrong approach

**Completed**:
- [x] Add eventsource-client dependency to Cargo.toml
- [x] Create SSE client module using eventsource-client
- [x] Implement streaming with eventsource-client
- [x] Test and discover incompatibility

**Lessons Learned**:
- eventsource-client can't work with existing response streams
- Need to handle the body stream we already have, not make new connections

### C.5.1: Replace Reqwest with Hyper for Direct Control (6-8 hours)
**Goal**: Replace reqwest with hyper for HTTP upstream connections
**Status**: ✅ COMPLETE - Hyper implementation working

**Implementation Complete**:
- [x] Created hyper_client.rs for direct HTTP control
- [x] Implemented HyperBodyStream for polling hyper::body::Incoming
- [x] Added hyper_streaming.rs for SSE parsing
- [x] Created hyper_raw_streaming.rs to forward raw bytes (avoid double-encoding)
- [x] Integrated into handle_mcp_request for SSE-accepting clients

**Current Status (2025-08-15 RESOLVED)**:
- ✅ SSE streaming WORKS with standard HTTP clients
- ✅ Integration test passes - proxy correctly forwards SSE data
- ✅ 1731 bytes successfully forwarded from upstream to client
- ✅ Headers properly set (no duplicates, correct SSE headers)
- ❌ MCP Inspector specifically fails due to its proxy layer

**Root Cause Identified**:
- Our proxy works correctly (proven by integration test)
- MCP Inspector error: "No connection established for request ID: 0"
- Inspector's StreamableHTTPServerTransport can't correlate response
- This is an Inspector-specific issue, not a general proxy bug

**Solutions Implemented**:
1. ✅ Fixed double-encoding issue (raw forwarding instead of re-parsing)
2. ✅ Added proper SSE headers (cache-control, x-accel-buffering)
3. ✅ Removed duplicate headers
4. ✅ Created integration test to verify functionality
5. ✅ Using hyper for direct body streaming control

**Completed**:
- ✅ Hyper client implementation complete
- ✅ SSE streaming works with standard HTTP clients
- ✅ Integration test passes
- ✅ Headers properly set
- ✅ MCP Inspector connects successfully

**Critical Issue Confirmed**:
- Client cannot establish successful connection through proxy
- SSE streaming is not working properly
- Reqwest's abstraction layer is preventing proper SSE handling

**Root Cause Analysis**:
1. **Reqwest limitations**:
   - Cannot extract raw hyper::Body from reqwest::Response
   - `bytes_stream()` may be completing prematurely
   - No control over how chunks are handled
   
2. **What we need**:
   - Direct access to hyper::Body
   - Full control over streaming behavior
   - Ability to poll body continuously until connection closes
   - Proper handling of chunked transfer encoding

**Solution: Use Hyper Directly**:
- [ ] Replace reqwest::Client with hyper::Client in process_via_http_new
- [ ] Build HTTP requests using hyper directly
- [ ] Handle response body streaming with full control
- [ ] Implement proper chunked transfer encoding handling
- [ ] Test with MCP Inspector to verify connections work

### C.5.1: Integrate with SSE Streaming (2 hours)
**Goal**: Connect eventsource-client to our SSE streaming infrastructure
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Update sse_streaming.rs to consume eventsource-client stream
- [ ] Convert eventsource-client events to our SseEvent format
- [ ] Maintain interceptor support
- [ ] Handle reconnection and Last-Event-Id

### C.5.2: Test Long-lived Connections (1 hour)
**Goal**: Verify SSE connections stay open
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Test with MCP Inspector
- [ ] Verify multiple events are received
- [ ] Test reconnection on failure
- [ ] Measure performance impact

## Phase C.6: SSE Module Consolidation (3-4 hours) - 📋 ANALYZED

### Problem Statement
After implementing hyper-based SSE that works with MCP Inspector, we have 9 SSE-related modules (~75KB) with multiple abandoned approaches (reqwest, eventsource-client, hyper).

### Analysis Complete (2025-08-16)
See `analysis/sse-module-consolidation.md` for detailed analysis.

**Key Findings**:
- **Keep**: 3 modules (hyper_client, hyper_raw_streaming, hyper_sse_intercepted)
- **Remove**: 5-6 modules from abandoned approaches
- **Potential**: 66% code reduction, cleaner architecture

### C.6.0: Remove eventsource-client modules (1 hour)
**Goal**: Remove abandoned eventsource-client approach
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Remove call to `stream_sse_with_eventsource` at legacy.rs:1356
- [ ] Delete `sse_streaming_v2.rs`
- [ ] Delete `sse_client.rs`
- [ ] Delete `process_via_http_sse_aware.rs`
- [ ] Remove eventsource-client from Cargo.toml

### C.6.1: Remove reqwest SSE modules (1 hour)
**Goal**: Remove obsolete reqwest-based approaches
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Delete `sse_streaming.rs`
- [ ] Delete `process_via_http_hyper.rs`
- [ ] Review and likely delete `hyper_streaming.rs`
- [ ] Extract any needed utilities from `http_processing.rs`

### C.6.2: Consolidate and organize (1-2 hours)
**Goal**: Clean module structure
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Merge HTTP processing functions in legacy.rs
- [ ] Consider moving to `src/proxy/reverse/sse/` subdirectory
- [ ] Update all imports and exports
- [ ] Test with MCP Inspector

## Phase D: SSE Reconnection Integration (12 hours)

### D.0: Foundation Integration (4 hours)
**Goal**: Integrate existing reconnection infrastructure  
**Status**: ✅ **COMPLETE** (2025-08-16)

**Completed Tasks**:
- ✅ Created `ReverseProxySseManager` wrapping ReconnectionManager
- ✅ Added Last-Event-Id tracking to Session struct
- ✅ Integrated EventTracker for deduplication per session
- ✅ Added HealthMonitor for connection health
- ✅ Created SessionSseExt trait for clean API
- ✅ All 4 tests passing

### D.1: Upstream Resilience (3 hours)
**Goal**: Auto-reconnect to upstream SSE servers  
**Status**: ⏸️ **PAUSED** - Blocked by transport architecture refactor

**Completed**:
- ✅ Analyzed reconnection architecture challenge
- ✅ Added disconnection detection and logging
- ✅ Documented where reconnection would occur
- ✅ Created detailed implementation plan

**Blocked on Transport Architecture Refactor**:
- 🚧 See [Transport Type Architecture Plan](../transport-type-architecture/transport-type-architecture-tracker.md)
- 🚧 Issue: `is_sse_session` code smell indicates deeper architectural problems
- 🚧 Solution: Clean up transport type modeling first
- ⏸️ Full ReconnectingStream integration
- ⏸️ Exponential backoff implementation
- ⏸️ Last-Event-Id resumption

### D.2: Client Resilience (3 hours)
**Goal**: Support client SSE reconnections  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Parse client's Last-Event-Id header
- [ ] Store event IDs in session storage
- [ ] Resume streams from client's last ID
- [ ] Handle deduplication for client reconnects

### D.3: Testing & Polish (2 hours)
**Goal**: Validate reconnection behavior  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Integration tests with connection drops
- [ ] Performance testing under reconnection scenarios
- [ ] Add metrics for reconnection attempts
- [ ] Update documentation

## Phase E: Modularization (8-10 hours)

### E.0: Create Module Structure (2 hours)
**Goal**: Set up modular organization  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Create `src/proxy/reverse/` directory structure
- [ ] Extract configuration to `config.rs`
- [ ] Extract metrics to `metrics.rs`
- [ ] Update imports

### E.1: Extract Handlers (3 hours)
**Goal**: Separate request handling logic  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Create `handlers/json.rs` for JSON processing
- [ ] Create `handlers/sse.rs` for SSE streaming
- [ ] Extract routing logic
- [ ] Add handler tests

### E.2: Extract Upstream Management (2 hours)
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

### D.4: Message Recording Abstraction (2 hours)
**Goal**: Implement proper recording abstraction  
**Status**: ⬜ Not Started

**Tasks**:
- [ ] Design MessageEventReceiver trait for recording
- [ ] Implement for TapeRecorder
- [ ] Update proxy to use abstraction (not know about tapes/frames)
- [ ] Enable dependency injection of recorder

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

### 🚧 DETOUR: Transport Architecture Refactor Required

During Phase D.0 implementation, we discovered that the `is_sse_session` boolean is a code smell indicating deeper architectural issues with how we model transport types. Before continuing with SSE reconnection (Phase D.1-D.3), we need to:

1. **Complete Transport Type Architecture Refactor**
   - See [Transport Type Architecture Plan](../transport-type-architecture/transport-type-architecture-tracker.md)
   - Eliminate `is_sse_session` boolean
   - Properly model bidirectional transports
   - Add explicit ResponseMode tracking
   - Unify forward and reverse proxy transport handling

2. **Then Resume Phase D.1-D.3**
   - With clean transport architecture in place
   - Proper session tracking for SSE
   - Clear response mode handling

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
- `analysis/sse-module-consolidation.md` - SSE module cleanup and consolidation plan (2025-08-16)

## Notes
- Current SSE implementation makes duplicate requests (temporary workaround)
- `ReverseProxyError::SseStreamingRequired` should be removed after Phase C
- Focus on streaming-first architecture for SSE
- Redis implementation deferred but design must support it
- Consider using existing `SseParser` and `SseStream` from transport layer