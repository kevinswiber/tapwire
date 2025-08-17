# Next Session: Implement Revised Phase C - Extract Shared Utilities

## Current Status
✅ **Phase A Complete**: Comprehensive analysis and design work done (10 hours)
✅ **Phase B Complete**: ResponseMode enum and ClientCapabilities implemented (4 hours)
🔄 **Phase C Revised**: New approach based on implementation learnings (5 hours estimated)

All 874 tests passing. Working branch: `refactor/transport-type-architecture` (in shadowcat repo).

## Important Context: Design Change

**We discovered the original unified cores approach was flawed**. Creating StdioCore, HttpCore, and SseCore with mode flags would violate Single Responsibility Principle. The revised approach extracts shared utilities while keeping transport types separate. See [Phase C Revised Approach](analysis/phase-c-revised-approach.md) for full details.

## Your Mission: Implement Revised Phase C

You need to implement the **revised Phase C approach** that extracts shared utilities without creating unified cores.

### Task Sequence

Start with the revised task files in order:

1. **[C.0: Create Shared Utilities](tasks/C.0-create-shared-utilities.md)** (1 hour)
   - Create `src/transport/raw/common/` module
   - Extract connection, buffer, validation, and timeout utilities
   - No unified cores - just shared helper functions

2. **[C.1: Refactor Transports to Use Utilities](tasks/C.1-refactor-transports-use-utilities.md)** (2 hours)
   - Update StdioRawIncoming/Outgoing to use utilities
   - Update HttpRawClient/Server to use utilities
   - Maintain exact same API and behavior

3. **[C.2: Optimize and Validate](tasks/C.2-optimize-and-validate.md)** (1 hour)
   - Optimize buffer pool usage
   - Add metrics and benchmarks
   - Validate code reduction achieved

4. **[C.3: Final Integration Testing](tasks/C.3-final-integration-testing.md)** (1 hour)
   - Run comprehensive test suite
   - Verify no regressions
   - Document results

## Clean Up First

Before starting, remove the abandoned unified core files:
```bash
rm src/transport/raw/stdio_core.rs
rm src/transport/raw/http_core.rs  
rm src/transport/raw/sse_core.rs

# Remove references from mod.rs
# Remove lines:
# pub mod stdio_core;
# pub mod http_core;
# pub mod sse_core;
# pub use stdio_core::StdioCore;
# pub use http_core::HttpCore;
# pub use sse_core::{SseCore, SseEvent};
```

## Key Design Principles

### DO:
- ✅ **Extract truly common patterns** - Connection checks, buffer management, etc.
- ✅ **Keep transport types separate** - StdioRawIncoming vs StdioRawOutgoing
- ✅ **Use composition** - Utilities and helper structs, not inheritance
- ✅ **Maintain exact behavior** - This is pure refactoring
- ✅ **Test continuously** - Run tests after each step

### DON'T:
- ❌ **Don't create unified cores** - No StdioCore with mode flags
- ❌ **Don't use conditionals for modes** - No "if client_mode then X"
- ❌ **Don't change public APIs** - Keep same interfaces
- ❌ **Don't add new features** - Pure refactoring only
- ❌ **Don't over-abstract** - Only extract truly duplicated code

## Expected Outcomes

After Phase C:
- [ ] ~500 lines of duplicate code eliminated
- [ ] All 874+ tests still passing
- [ ] No unified cores with mode flags
- [ ] Clean separation of concerns maintained
- [ ] Performance within 5% of current
- [ ] Ready for Phase D

## Code Patterns to Extract

### Connection Validation
```rust
// Duplicated in every transport
if !self.connected {
    return Err(TransportError::NotConnected);
}

// Extract to utility
connection::ensure_connected(self.connected)?;
```

### Buffer Management
```rust
// Duplicated pattern
let mut buffer = pool.acquire();
buffer.extend_from_slice(data);
let result = buffer.to_vec();
pool.release(buffer);

// Extract to utilities
let buffer = buffer::acquire_and_fill(&pool, data);
let result = buffer::to_vec_and_release(&pool, buffer);
```

### Message Size Validation
```rust
// Duplicated check
if data.len() > self.config.max_message_size {
    return Err(TransportError::MessageTooLarge { ... });
}

// Extract to utility
validation::validate_message_size(data, self.config.max_message_size)?;
```

## Testing Commands

```bash
# After each task
cargo test transport::raw::

# Check for regressions
cargo test

# Verify no clippy warnings
cargo clippy --all-targets -- -D warnings

# Final validation
cargo test --release
```

## Success Criteria

Phase C is complete when:
1. Shared utilities module exists and is used
2. Code duplication reduced by >50% (~500 lines)
3. All tests passing (874+)
4. No performance regression
5. Clean architecture maintained

## Remember

This revised approach is **better** than the original plan. We're achieving the same goal (reduce duplication) with a cleaner architecture that maintains type safety and Single Responsibility Principle.

---

**Start with**: [C.0-create-shared-utilities.md](tasks/C.0-create-shared-utilities.md)
**Reference**: [phase-c-revised-approach.md](analysis/phase-c-revised-approach.md) for rationale
**Tracker**: [transport-type-architecture-tracker.md](transport-type-architecture-tracker.md) for progress