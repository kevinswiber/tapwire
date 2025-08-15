# Implementation Requirements & Open Questions

## Critical Questions for Phase B (SessionStore)

### 1. SessionStore Trait Design
**Current Situation**: `SessionManager` directly uses `Arc<InMemorySessionStore>` in 13+ files

**Decisions Made**:
1. **Async Methods**: Use `async-trait` ✅
   - Already in dependencies, stable and proven

2. **Error Handling**: Extend `SessionError` ✅
   - Add storage-specific variants as needed

3. **Core Methods Needed** (confirmed):
   ```rust
   #[async_trait]
   pub trait SessionStore: Send + Sync {
       // Core session operations
       async fn create_session(&self, session: Session) -> SessionResult<()>;
       async fn get_session(&self, id: &SessionId) -> SessionResult<Session>;
       async fn update_session(&self, session: Session) -> SessionResult<()>;
       async fn delete_session(&self, id: &SessionId) -> SessionResult<()>;
       async fn count_sessions(&self) -> SessionResult<usize>;
       async fn list_sessions(&self) -> SessionResult<Vec<Session>>;
       
       // Frame operations
       async fn add_frame(&self, frame: MessageEnvelope) -> SessionResult<()>;
       async fn get_frames(&self, session_id: &SessionId) -> SessionResult<Vec<MessageEnvelope>>;
       
       // SSE-specific (new, for Phase C)
       async fn store_last_event_id(&self, session_id: &SessionId, event_id: String) -> SessionResult<()>;
       async fn get_last_event_id(&self, session_id: &SessionId) -> SessionResult<Option<String>>;
       
       // Batch operations (for Redis efficiency)
       async fn get_sessions_batch(&self, ids: &[SessionId]) -> SessionResult<Vec<Session>>;
       async fn update_sessions_batch(&self, sessions: Vec<Session>) -> SessionResult<()>;
   }
   ```

4. **Connection Pooling**: Implementation-specific ✅
   - Not in trait - not all stores need it (e.g., InMemoryStore)
   - Each implementation handles its own pooling needs

### 2. Migration Strategy

**Blast Radius Analysis**:
- 13 files reference SessionManager or InMemorySessionStore
- Key touchpoints:
  - `src/proxy/reverse.rs` - Creates and uses SessionManager
  - `src/proxy/forward.rs` - Creates and uses SessionManager  
  - `src/main.rs` - Initializes SessionManager
  - `src/session/builder.rs` - Builds SessionManager with store

**Migration Approach**:
1. **Type Aliases**: Not needed ✅
   - No backwards compatibility requirements (Shadowcat unreleased)
   - Direct refactoring is cleaner

2. **Store Ownership**: Reference via Arc ✅
   - SessionManager references: `store: Arc<dyn SessionStore>`
   - Enables library consumers to provide custom implementations
   - Store can be injected via Shadowcat API
   - Shared ownership via Arc for multi-threaded access

## Critical Questions for Phase C (SSE Fix)

### 1. UpstreamResponse Design

**Required Fields**:
```rust
struct UpstreamResponse {
    response: reqwest::Response,     // The unconsumed response
    content_type: Option<mime::Mime>, // Parsed MIME type
    content_length: Option<usize>,    // For buffering decisions
    is_chunked: bool,                 // Transfer-Encoding: chunked
    session_id: Option<String>,       // From Mcp-Session-Id header
}
```

**Decisions Made**:
1. **MIME Parsing**: Parse eagerly ✅
   - We almost always need it for routing decisions

2. **Non-SSE/JSON Content**: Stream pass-through ✅
   - Pass through with minimal buffering
   - Apply client backpressure to upstream
   - Let client decide on unknown content types
   - MCP doesn't specify binary data but extensions might use it

3. **Chunked Transfer Encoding**: Transparent handling ✅
   - Let reqwest handle chunked encoding automatically
   - No special logic needed

### 2. Testing Strategy

**Decisions Made**:
1. **Mock Servers**: Both approaches ✅
   - Use `mockito` or `wiremock` for unit tests
   - Use actual MCP servers for integration tests
   - Provides both speed and real-world validation

2. **SSE Testing**: Fixtures + async streams ✅
   - Create SSE test fixtures
   - Use `tokio::test` for async testing
   - Test reconnection and Last-Event-Id scenarios

3. **Session Store Testing**: Common test suite ✅
   - Create trait test suite both implementations must pass
   - Use test macros for implementation-agnostic tests

## Parallelization Opportunities

### What MUST Be Serial:
1. **Phase B** → **Phase C**: SSE fix depends on SessionStore trait
2. **Phase E**: Final integration must be last

### What CAN Be Parallel:

#### Phase D (Modularization) - Can Split:
**Track D1**: Admin Extraction (3 hours) - **INDEPENDENT**
- Extract 876 lines of admin handlers
- Create `reverse/admin/` module
- Self-contained HTML generation
- Minimal dependencies on other code
- **Can start immediately after Phase B**

**Track D2**: Core Modularization (5 hours) - **DEPENDS on Phase C**
- Extract config, metrics, handlers
- Requires UpstreamResponse changes
- Touches SSE handling code

#### Parallel Work Plan:
```
Phase B (SessionStore) - 4-5 hours
    ↓
Phase C (SSE Fix) - 4-6 hours ──┐
    ↓                            │
Track D2: Core Modules           │ Track D1: Admin Extract
(5 hours)                        │ (3 hours)
    ↓                            ↓
    └────────────────────────────┘
                ↓
        Phase E: Integration
```

This saves ~3 hours if we have two developers.

## Required Test Fixtures

### For Phase B:
- Sample Session objects with various states
- Test MessageEnvelope frames
- Mock Redis responses (for future)

### For Phase C:
- SSE stream fixtures:
  ```
  data: {"jsonrpc":"2.0","method":"test","id":1}\n\n
  event: notification\ndata: {"status":"ok"}\n\n
  id: 123\ndata: test with id\n\n
  ```
- Large JSON responses for buffering tests
- Chunked response fixtures

## Configuration Decisions

### SessionStore Configuration:
```toml
[session]
storage = "memory"  # or "redis" later

[session.memory]
max_sessions = 10000
cleanup_interval = "5m"

[session.redis]  # Future
url = "redis://localhost:6379"
pool_size = 10
ttl = "24h"
```

### Should we add this config now or wait?
- Recommendation: Add config structure now, only implement memory section

## Documentation Needs

1. **Migration Guide**: How to update code using SessionManager
2. **Trait Implementation Guide**: For future storage backends
3. **SSE Streaming Architecture**: Document the new flow
4. **Testing Guide**: How to test with new abstractions

## Decision Points Summary

**All Decisions Made**:
1. ✅ Use `async-trait` for SessionStore
2. ✅ Extend SessionError (not new error type)
3. ✅ SessionManager references store via Arc<dyn SessionStore>
4. ✅ Parse MIME eagerly for routing
5. ✅ Stream non-JSON/SSE content with backpressure
6. ✅ Connection pooling is implementation-specific
7. ✅ No type aliases needed (no backwards compatibility)
8. ✅ Store can be injected for library consumers

**Deferred to Future**:
1. Redis implementation details
2. Batch operation optimization
3. Session expiry/TTL handling
4. Distributed lock mechanisms

## Next Actions

### If Solo Developer:
1. Phase B: SessionStore trait (4-5 hours)
2. Phase C: SSE fix (4-6 hours)
3. Phase D: All modularization (8 hours)
4. Phase E: Integration (4 hours)
Total: ~20-23 hours

### If Two Developers:
**Developer 1**:
1. Phase B: SessionStore trait (4-5 hours)
2. Phase C: SSE fix (4-6 hours)
3. Track D2: Core modularization (5 hours)

**Developer 2**:
1. Wait for Phase B completion
2. Track D1: Admin extraction (3 hours) - start after B
3. Help with Phase E integration

Total: ~17-20 hours (saves 3 hours)

## File Creation Order

### Phase B Files:
1. `src/session/store.rs` - Trait definition
2. `src/session/memory.rs` - Refactored InMemoryStore
3. `src/session/errors.rs` - Extended error types (if needed)

### Phase C Files:
1. `src/proxy/reverse/upstream_response.rs` - New wrapper type
2. `src/proxy/reverse/sse_handler.rs` - SSE streaming logic
3. `tests/sse_streaming.rs` - Integration tests

### Phase D Files:
1. `src/proxy/reverse/mod.rs` - Module root
2. `src/proxy/reverse/config.rs` - Configuration
3. `src/proxy/reverse/admin/mod.rs` - Admin module
4. `src/proxy/reverse/handlers/mod.rs` - Handler traits
5. `src/proxy/reverse/handlers/json.rs` - JSON handling
6. `src/proxy/reverse/handlers/sse.rs` - SSE handling