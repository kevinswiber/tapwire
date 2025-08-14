# Next Session: Phase 6B - ReverseProxy Transport Migration (15h)

## 📋 Current Status (2025-08-14 - Session 5 Complete)

### Phase 6 Partial Success ✅
- ✅ **All Test MockTransports Fixed**: integration_api_mock.rs and version_negotiation_test.rs now use directional traits
- ✅ **Tests Compile and Pass**: 860 unit tests passing, all integration tests compile
- ✅ **ForwardProxy Fully Migrated**: Clean use of Box<dyn IncomingTransport> and Box<dyn OutgoingTransport>
- ⚠️ **Dual System Still Active**: Old Transport trait cannot be removed yet

### Why We're Stuck
The old `Transport` trait is still required by:
1. **ReverseProxy Connection Pool** - `PoolableStdioTransport` wraps `StdioTransport`
2. **ReplayTransport** - Implements old `Transport` trait
3. **All Old Implementations** - StdioTransport, HttpTransport, SseTransport, etc.

### Architecture Challenge
ReverseProxy uses a fundamentally different pattern:
- **Axum HTTP Server**: Handles incoming HTTP requests
- **Connection Pool**: Manages multiple `StdioTransport` instances
- **Different Flow**: HTTP request → Pool → StdioTransport → subprocess

This is NOT a simple transport swap like ForwardProxy was.

## 🎯 Phase 6B: Complete ReverseProxy Migration

### Priority Tasks

#### Task 1: Create PoolableOutgoingTransport (4h)
**Goal**: Replace PoolableStdioTransport with directional version

**Implementation Strategy**:
```rust
// In src/proxy/pool.rs
pub struct PoolableOutgoingTransport {
    transport: Box<dyn OutgoingTransport>,
    id: String,
    created_at: Instant,
}

impl PoolableConnection for PoolableOutgoingTransport {
    async fn is_healthy(&self) -> bool {
        self.transport.is_connected()
    }
    // ... other methods
}
```

**Key Changes**:
1. Replace `StdioTransport` with `SubprocessOutgoing`
2. Update pool factory to create `SubprocessOutgoing`
3. Ensure pool lifecycle management works with new transport

#### Task 2: Update ReverseProxy Message Flow (4h)
**Current Flow**:
```
HTTP Request → Extract JSON-RPC → Pool.acquire() → StdioTransport → Send/Receive → HTTP Response
```

**New Flow**:
```
HTTP Request → Extract JSON-RPC → Pool.acquire() → OutgoingTransport → Send/Receive → HTTP Response
```

**Changes Needed**:
- Update `handle_http_request` to work with `OutgoingTransport`
- Convert HTTP body to `MessageEnvelope` for `send_request()`
- Convert `receive_response()` back to HTTP response

#### Task 3: Migrate ReplayTransport (3h)
**Options**:
1. Implement both `IncomingTransport` and `OutgoingTransport` for `ReplayTransport`
2. Create separate `ReplayIncoming` and `ReplayOutgoing` types
3. Consider if replay even needs to be a Transport

**Recommendation**: Option 1 - Add directional traits to existing ReplayTransport

#### Task 4: Remove Old Transport System (4h)
**Once Tasks 1-3 complete**:
1. Delete `trait Transport` from `src/transport/mod.rs`
2. Remove all old implementations:
   - `src/transport/stdio.rs` (StdioTransport)
   - `src/transport/stdio_client.rs` (StdioClientTransport)
   - `src/transport/http.rs` (HttpTransport)
   - `src/transport/http_mcp.rs` (HttpMcpTransport)
   - `src/transport/sse_transport.rs` (SseTransport)
   - `src/transport/sse_interceptor.rs` (InterceptedSseTransport)
3. Update all imports
4. Clean up transport/mod.rs exports

## ✅ Success Criteria

### Must Complete
- [ ] ReverseProxy works with directional transports
- [ ] Connection pool uses `PoolableOutgoingTransport`
- [ ] ReplayTransport implements directional traits
- [ ] Old Transport trait completely removed
- [ ] All 869+ tests pass
- [ ] Zero clippy warnings

### Quality Metrics
- [ ] Single transport system (directional only)
- [ ] No confusion between old and new
- [ ] Pool performance unchanged
- [ ] ReverseProxy latency unchanged

## 🚀 Commands to Run

```bash
# Start with pool changes
rg "PoolableStdioTransport" src/

# Check ReverseProxy usage
rg "StdioTransport" src/proxy/

# Find all Transport trait implementations
rg "impl Transport for" src/

# Test after each change
cargo test --lib
cargo test --test integration_api_mock

# Final validation
cargo test
cargo clippy --all-targets -- -D warnings
```

## 📊 Implementation Order

### Step 1: Pool Migration (Morning)
1. Create `PoolableOutgoingTransport`
2. Update pool factory
3. Test pool operations
4. Verify health checks work

### Step 2: ReverseProxy Update (Afternoon)
1. Update HTTP request handler
2. Convert message formats
3. Test with curl/httpie
4. Verify SSE still works

### Step 3: Cleanup (Evening)
1. Add directional traits to ReplayTransport
2. Delete old Transport trait
3. Remove all old implementations
4. Update documentation

## ⚠️ Risk Areas

### High Risk
- **Pool Lifecycle**: Ensure connections are properly managed
- **Message Conversion**: HTTP ↔ MessageEnvelope conversion must be correct
- **SSE Streaming**: Must still work after migration

### Mitigation
- Test each component in isolation first
- Keep old code until new code is verified
- Use feature flags if needed for gradual rollout

## 🔍 Key Files to Modify

### Must Update
- `src/proxy/pool.rs` - Create PoolableOutgoingTransport
- `src/proxy/reverse.rs` - Update to use new pool
- `src/transport/replay.rs` - Add directional traits
- `src/transport/mod.rs` - Remove Transport trait

### Must Delete (After Migration)
- `src/transport/stdio.rs`
- `src/transport/stdio_client.rs`
- `src/transport/http.rs`
- `src/transport/http_mcp.rs`
- `src/transport/sse_transport.rs`
- `src/transport/sse_interceptor.rs`

## 📝 Architecture Notes

### Why This Is Hard
ReverseProxy is fundamentally different from ForwardProxy:
- **ForwardProxy**: Client transport → Proxy → Server transport (both sides are transports)
- **ReverseProxy**: HTTP server → Proxy → Subprocess pool (HTTP on one side, transports on other)

### Key Insight
The pool doesn't care about transport direction - it just needs:
1. Ability to send messages
2. Ability to receive responses
3. Health checking
4. Connection lifecycle

This maps well to `OutgoingTransport`.

## 🏁 Definition of Done

When Phase 6B is complete:
- No `trait Transport` in codebase
- No old transport implementations
- ReverseProxy fully functional with new system
- All tests passing
- Documentation updated
- Single, clear transport architecture

---

**Session Time**: Estimated 15 hours
**Complexity**: High - Architectural changes required
**Priority**: CRITICAL - Technical debt growing daily

## Resources

### Key Documentation
- **Main Tracker**: `transport-refactor-tracker.md`
- **Session 5 Work**: MockTransport fixes in tests/
- **Pool Pattern**: `src/proxy/pool.rs`
- **ReverseProxy**: `src/proxy/reverse.rs`

### What Session 5 Accomplished
- Fixed MockTransport in integration_api_mock.rs (added directional traits)
- Fixed MockTransport in version_negotiation_test.rs (added directional traits)
- Updated all test proxy.start() calls to use Box::new()
- Verified 860 unit tests pass
- Identified blockers for full migration

### Technical Context
The transport refactor introduced `IncomingTransport` and `OutgoingTransport` to replace the old `Transport` trait. ForwardProxy was successfully migrated, but ReverseProxy's connection pool architecture prevents simple migration. The pool wraps StdioTransport in PoolableStdioTransport, which is tightly coupled to the old system.

**IMPORTANT**: This migration cannot be done piecemeal. Once we start modifying the pool, we must complete the entire ReverseProxy migration to maintain a working system.