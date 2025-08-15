# Final Implementation Decisions

## Summary of All Decisions Made

This document captures all the implementation decisions for the reverse proxy refactor, incorporating user feedback received on 2025-01-15.

## Architecture Decisions

### SessionStore Design
1. **SessionManager references the store** (not owns)
   - Enables library consumers to inject custom implementations
   - Store passed in via Shadowcat API
   - Uses `Arc<dyn SessionStore>` for shared ownership

2. **Connection pooling is implementation-specific**
   - Not defined in the trait
   - InMemoryStore doesn't need pooling
   - Redis implementation can add its own pooling

3. **No type aliases needed**
   - Shadowcat is unreleased - no backwards compatibility required
   - Direct refactoring is cleaner

### SSE Handling
1. **Parse MIME types eagerly**
   - Almost always needed for routing decisions
   - Parse once in UpstreamResponse constructor

2. **Non-SSE/JSON content handling**
   - Stream pass-through with minimal buffering
   - Apply client backpressure to upstream
   - Let client decide on unknown content types
   - Support potential MCP extensions with binary data

3. **Backpressure implementation**
   - Use bounded channels
   - Slow clients pause upstream reads
   - Fast clients get data as quickly as upstream provides

## Technical Decisions

### Phase B: SessionStore Abstraction
- Use `async-trait` (already in dependencies)
- Extend `SessionError` for storage errors
- Design trait methods with Redis in mind
- Defer actual Redis implementation

### Phase C: SSE Fix
- Full UpstreamResponse wrapper implementation
- No quick patches or hacks
- Integrate properly with SessionStore trait
- Remove `SseStreamingRequired` error

### Phase D: Modularization
- Admin extraction can be parallelized
- Core modules must wait for SSE fix
- No circular dependencies allowed

## API Design

### Library Consumer Interface
```rust
// Library consumers can provide custom session stores
let custom_store = MyCustomStore::new();
let shadowcat = Shadowcat::builder()
    .with_session_store(Arc::new(custom_store))
    .build();
```

This enables:
- Custom in-memory implementations
- Database-backed stores
- Distributed stores (Redis, Cassandra, etc.)
- Testing with mock stores

## Testing Strategy

1. **Mock servers for unit tests** (mockito/wiremock)
2. **Real MCP servers for integration tests**
3. **Common test suite for all SessionStore implementations**
4. **Test fixtures for SSE streams**
5. **Backpressure testing with slow consumers**

## Implementation Order

### Sequential Requirements
1. Phase B (SessionStore) must complete first
2. Phase C (SSE fix) depends on Phase B
3. Phase D2 (core modules) depends on Phase C
4. Phase E (integration) must be last

### Parallelization Opportunity
- Phase D1 (admin extraction) can start after Phase B
- Saves ~3 hours if two developers available

## What We're NOT Doing Now

1. **Redis implementation** - Deferred to future phase
2. **Connection pooling** - Deferred to future phase
3. **Distributed locks** - Not needed yet
4. **Session migration tools** - Can add later
5. **Batch operation optimization** - Can optimize later

## Success Criteria

### Must Have
- ✅ SessionStore trait enables custom implementations
- ✅ No duplicate requests for SSE streams
- ✅ Backpressure properly implemented
- ✅ Library consumers can inject stores
- ✅ All existing tests pass

### Should Have
- ✅ Clean module boundaries
- ✅ Admin interface extracted
- ✅ Performance maintained (<5% overhead)

### Nice to Have (Future)
- Redis backend implementation
- Connection pooling
- Distributed session support
- Upstream failover

## Key Insight

By making SessionManager reference the store rather than own it, we enable a powerful extension point for library consumers. This decision transforms Shadowcat from a monolithic proxy into a flexible library that can adapt to different deployment scenarios.

## Next Steps

1. Start Phase B immediately - create `src/session/store.rs`
2. Follow the implementation order strictly
3. Use the unified-plan.md as the implementation guide
4. Test each phase before proceeding to the next

Total estimated time: 21-24 hours (or 18-21 with parallelization)