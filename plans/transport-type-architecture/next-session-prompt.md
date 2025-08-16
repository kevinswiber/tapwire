# Next Session: Implement Phase C - Extract Shared Transport Logic

## Current Status
✅ **Phase A Complete**: Comprehensive analysis and design work done (10 hours)
✅ **Phase B Complete**: ResponseMode enum and ClientCapabilities implemented (4 hours)
🎯 **Phase C Ready**: Extract shared transport logic (8 hours estimated)

All 873 tests passing. Working branch: `refactor/transport-type-architecture` (in shadowcat repo).

## Your Mission: Implement Phase C

You need to implement **Phase 2 from the implementation roadmap** (Extract Shared Transport Logic). This is pure refactoring - no new functionality, just cleaning up the architecture.

### Task Sequence

Start with the task files in order:

1. **[C.0: Create Raw Transport Primitives](tasks/C.0-create-raw-transport-primitives.md)** (2 hours)
   - Extract low-level I/O operations into `src/transport/raw/` module
   - Create StdioCore, HttpCore, SseCore - just bytes, no MCP knowledge
   - Reference: `analysis/implementation-roadmap.md` lines 207-251

2. **[C.1: Refactor Directional Transports](tasks/C.1-refactor-directional-transports.md)** (2 hours)
   - Make IncomingTransport/OutgoingTransport use the raw primitives
   - Eliminate duplicate I/O code through delegation
   - Reference: `analysis/implementation-roadmap.md` lines 253-282

3. **[C.2: Create Unified Factory](tasks/C.2-create-unified-factory.md)** (2 hours)
   - Implement TransportFactory for consistent transport creation
   - Centralize buffer pools and connection pooling
   - Reference: `analysis/implementation-roadmap.md` lines 284-332

4. **[C.3: Integration Testing](tasks/C.3-integration-testing.md)** (2 hours)
   - Validate the refactoring worked
   - Performance benchmarks must show <5% overhead
   - Code duplication should be reduced by >50%

## Critical Guidelines

### DO:
- ✅ **Follow the existing design** - All design work is complete in `analysis/` directory
- ✅ **Move code, don't rewrite** - Extract existing logic into shared modules
- ✅ **Preserve optimizations** - Keep buffer pooling, keep performance characteristics
- ✅ **Test continuously** - Run `cargo test` after each step
- ✅ **Reference the roadmap** - `analysis/implementation-roadmap.md` has exact code snippets

### DON'T:
- ❌ **Don't reference old code in comments** - No mentions of `is_sse_session` or removed code
- ❌ **Don't worry about backward compatibility** - Shadowcat is unreleased, we can break things
- ❌ **Don't redesign** - The design is done, just implement it
- ❌ **Don't add new features** - This is pure refactoring
- ❌ **Don't create "transport adapters"** - That was a mistake, stick to the roadmap

### If You Hit Design Issues:
1. **PAUSE** - Don't try to solve on the fly
2. **Document the issue** clearly
3. **Check the analysis documents** - The answer might already be there
4. **Ask for clarification** if needed

## Key Design Decisions Already Made

From `analysis/design-decisions.md`:
- **ResponseMode is separate from TransportType** (Decision 1)
- **Use DirectionalTransports everywhere** (Decision 2) 
- **Generic connection pooling** (Decision 3)
- **Shared transport implementations in raw module** (Decision 4)
- **ProxyCore, not UnifiedProxy** (Decision 5)
- **Keep TransportType name, rename Sse to StreamableHttp** (Decision 8)

## Success Criteria

After Phase C:
- [ ] All 873+ tests still passing
- [ ] Raw transport primitives extracted
- [ ] Directional transports use shared logic
- [ ] Transport factory working
- [ ] Code duplication reduced >50%
- [ ] Performance within 5% of baseline
- [ ] No clippy warnings

## Code Locations

### You'll be working in:
- `shadowcat/src/transport/raw/` (new module)
- `shadowcat/src/transport/directional/` (refactor existing)
- `shadowcat/src/transport/factory/` (new module)
- `shadowcat/src/cli/forward.rs` (update to use factory)
- `shadowcat/src/cli/reverse.rs` (update to use factory)

### Reference implementations in:
- `shadowcat/src/transport/stdio.rs` (extract logic from here)
- `shadowcat/src/transport/http.rs` (extract logic from here)
- `shadowcat/src/transport/sse/` (extract logic from here)

## Testing Commands

```bash
# After each step
cargo test transport::
cargo clippy --all-targets -- -D warnings

# Performance check
cargo bench --bench transport_performance

# Final validation
cargo test  # All 873+ tests must pass
```

## Example Code Pattern

The refactoring follows this pattern:

```rust
// BEFORE (in directional transport)
impl StdioIncoming {
    async fn receive_request(&mut self) -> Result<Message> {
        // Direct I/O implementation here
        let mut buffer = vec![0; 8192];
        self.stdin.read(&mut buffer).await?;
        // Parse protocol...
    }
}

// AFTER (delegating to raw primitive)
impl StdioIncoming {
    async fn receive_request(&mut self) -> Result<Message> {
        // Delegate I/O to core
        let bytes = self.core.receive_bytes().await?;
        // Handle protocol at this layer
        protocol::deserialize(&bytes)
    }
}
```

## When Complete

After finishing all Phase C tasks:
1. Run full test suite: `cargo test`
2. Check performance: `cargo bench`
3. Verify no clippy warnings: `cargo clippy --all-targets -- -D warnings`
4. Update tracker with actual times
5. Commit with message: `feat: extract shared transport logic (Phase C complete)`

## Remember

This is **Phase 2 from the implementation roadmap**, not Phase 3. We're extracting shared transport logic, not unifying proxy architecture yet. That comes in Phase D.

The design work is DONE. Your job is to implement what's already been designed. The `analysis/` directory has all the answers - use it!

---

**Start with**: [C.0-create-raw-transport-primitives.md](tasks/C.0-create-raw-transport-primitives.md)
**Reference**: [implementation-roadmap.md](analysis/implementation-roadmap.md) for exact implementation details
**Tracker**: [transport-type-architecture-tracker.md](transport-type-architecture-tracker.md) for progress