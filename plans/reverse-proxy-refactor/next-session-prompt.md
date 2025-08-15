# Next Session: Implement Phase B - SessionStore Abstraction

## Context
We've completed Phase A (Analysis) of the reverse proxy refactor. The proxy has a critical SSE bug (makes duplicate requests) and lacks storage abstraction (directly coupled to InMemorySessionStore). We need to fix both issues through a careful refactoring.

## Current Status
- **Phase A**: ✅ COMPLETE (5 hours) - All analysis done, decisions made
- **Phase B**: ⬜ Ready to start - SessionStore abstraction (4-5 hours)
- **Phase C**: ⬜ Blocked on B - Fix SSE bug properly (5-6 hours)
- **Phase D**: ⬜ Blocked on C - Modularization (8 hours)
- **Phase E**: ⬜ Final - Integration & testing (4 hours)

## Your Task: Implement Phase B - SessionStore Abstraction

### Key Documents to Read First
1. **Main Tracker**: `plans/reverse-proxy-refactor/tracker.md`
2. **Implementation Guide**: `plans/reverse-proxy-refactor/analysis/unified-plan.md`
3. **All Decisions**: `plans/reverse-proxy-refactor/analysis/final-decisions.md`

### Critical Decisions Already Made
- SessionManager **references** store via `Arc<dyn SessionStore>` (not owns)
- Store can be **injected** via Shadowcat API for library consumers
- **No backwards compatibility** needed - Shadowcat unreleased
- Connection pooling is **implementation-specific**
- Use **async-trait** for the trait definition

### Phase B Implementation Steps

#### B.1: Create SessionStore Trait (1 hour)
**File**: `src/session/store.rs`

```rust
use async_trait::async_trait;

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
    async fn delete_frames(&self, session_id: &SessionId) -> SessionResult<()>;
    
    // SSE-specific (for Phase C)
    async fn store_last_event_id(&self, session_id: &SessionId, event_id: String) -> SessionResult<()>;
    async fn get_last_event_id(&self, session_id: &SessionId) -> SessionResult<Option<String>>;
    
    // Batch operations (for future Redis)
    async fn get_sessions_batch(&self, ids: &[SessionId]) -> SessionResult<Vec<Session>>;
    async fn update_sessions_batch(&self, sessions: Vec<Session>) -> SessionResult<()>;
}
```

#### B.2: Refactor InMemoryStore (2 hours)
1. Move current `InMemorySessionStore` from `src/session/store.rs` to `src/session/memory.rs`
2. Add new field for SSE support:
   ```rust
   last_event_ids: Arc<RwLock<HashMap<SessionId, String>>>,
   ```
3. Implement all trait methods

#### B.3: Update SessionManager (1-2 hours)
**File**: `src/session/manager.rs`
- Change `store: Arc<InMemorySessionStore>` to `store: Arc<dyn SessionStore>`
- Update builder to accept trait
- Enable library consumer injection

#### B.4: Fix Compilation (1 hour)
Update these 13 files that reference SessionManager or InMemorySessionStore:
- `src/proxy/reverse.rs`
- `src/proxy/forward.rs`
- `src/main.rs`
- `src/session/builder.rs`
- (and 9 others - use `grep -r "InMemorySessionStore"`)

### Success Criteria for Phase B
- [ ] SessionStore trait defined with all methods
- [ ] InMemoryStore implements the trait
- [ ] SessionManager uses trait reference
- [ ] All existing tests pass
- [ ] Can swap implementations at runtime

### Testing Commands
```bash
# After creating trait
cargo check

# After implementing for InMemoryStore
cargo test --lib session::

# After updating SessionManager
cargo test

# Full validation
cargo clippy --all-targets -- -D warnings
```

### What NOT to Do in Phase B
- Don't implement Redis backend yet (future phase)
- Don't fix the SSE bug yet (Phase C)
- Don't modularize reverse.rs yet (Phase D)
- Focus ONLY on the storage abstraction

### Blockers/Issues to Watch For
1. **Trait object safety** - All methods must be object-safe
2. **Lifetime issues** - Use Arc for shared ownership
3. **Breaking changes** - Update all 13 files that use the store

### Next Phase Preview (Phase C)
After Phase B is complete, Phase C will:
- Implement UpstreamResponse wrapper
- Fix SSE duplicate request bug
- Add backpressure for streaming
- Use the new SessionStore trait for Last-Event-Id

### Time Estimate
- Phase B: 4-5 hours
- Remaining work: 17-19 hours
- Total project: 22-24 hours

## Important Context
The reverse proxy currently makes duplicate HTTP requests when it detects SSE streams. This is because the function signature can't return the Response object for streaming, so it uses an error as control flow, causing the Response to be dropped. We'll fix this in Phase C with the UpstreamResponse wrapper, but first we need the SessionStore abstraction to properly track Last-Event-Id for SSE reconnections.

## Questions?
All design decisions have been made and documented in `analysis/final-decisions.md`. The implementation path is clear. Begin with B.1: Create SessionStore trait.