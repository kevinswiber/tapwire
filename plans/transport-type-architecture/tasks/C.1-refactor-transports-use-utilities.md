# Task C.1: Refactor Raw Transports to Use Shared Utilities (REVISED)

## Status: Ready
## Estimated Duration: 2 hours  
## Actual Duration: TBD

## Context

**REVISED APPROACH (2025-08-16)**: Following the creation of shared utilities in C.0, this task refactors existing raw transports to use those utilities, eliminating code duplication while maintaining the separate transport types. See [Phase C Revised Approach](../analysis/phase-c-revised-approach.md) for rationale.

## Objective

Refactor existing raw transport implementations to use the shared utilities created in C.0, reducing code duplication by >50% while maintaining exact same behavior and API.

## Prerequisites

- [x] C.0 completed - Shared utilities module exists
- [ ] All tests passing before refactoring

## Implementation Steps

### Step 1: Refactor StdioRawIncoming (30 min)

Update `src/transport/raw/stdio.rs`:

```rust
use super::common::{
    ensure_connected, 
    validate_message_size,
    acquire_and_fill,
    to_vec_and_release,
    with_timeout,
    ConnectionState,
};

pub struct StdioRawIncoming {
    connection: ConnectionState,  // Replace connected: bool
    // ... other fields remain
}

impl RawTransport for StdioRawIncoming {
    async fn send_bytes(&mut self, data: &[u8]) -> TransportResult<()> {
        // Before: Complex connection and size checking
        // After: Use utilities
        self.connection.ensure_connected()?;
        validate_message_size(data, self.config.max_message_size)?;
        
        // Use buffer utilities
        let buffer = acquire_and_fill(&self.buffer_pool, data);
        
        // ... specific sending logic remains
    }
    
    async fn receive_bytes(&mut self) -> TransportResult<Vec<u8>> {
        self.connection.ensure_connected()?;
        
        // Use timeout utility
        with_timeout(
            self.config.read_timeout,
            self.receive_internal(),
            "Read timeout on stdin"
        ).await
    }
}
```

### Step 2: Refactor StdioRawOutgoing (30 min)

Similar refactoring for outgoing:

```rust
pub struct StdioRawOutgoing {
    connection: ConnectionState,
    // ... other fields
}

impl RawTransport for StdioRawOutgoing {
    async fn send_bytes(&mut self, data: &[u8]) -> TransportResult<()> {
        self.connection.ensure_connected()?;
        validate_message_size(data, self.config.max_message_size)?;
        
        // Reuse same validation and buffer logic
        // ... specific subprocess writing remains
    }
}
```

### Step 3: Refactor HttpRawClient (20 min)

Update `src/transport/raw/http.rs`:

```rust
pub struct HttpRawClient {
    connection: ConnectionState,
    // ... other fields
}

impl RawTransport for HttpRawClient {
    async fn send_bytes(&mut self, data: &[u8]) -> TransportResult<()> {
        self.connection.ensure_connected()?;
        validate_message_size(data, self.config.max_message_size)?;
        
        let buffer = acquire_and_fill(&global_pools::HTTP_POOL, data);
        // ... HTTP-specific sending
    }
}
```

### Step 4: Refactor HttpRawServer (20 min)

Similar pattern for server:

```rust
pub struct HttpRawServer {
    connection: ConnectionState,
    // ... other fields
}

// Similar refactoring pattern
```

### Step 5: Update SSE Transports (20 min)

Apply same pattern to `src/transport/raw/sse.rs`:
- `SseRawClient`
- `SseRawServer`

### Step 6: Run Tests and Fix Issues (10 min)

```bash
# Run all transport tests
cargo test transport::raw::

# Fix any compilation or test failures
```

## Code Reduction Analysis

### Before Refactoring
- StdioRawIncoming: ~650 lines
- StdioRawOutgoing: ~700 lines  
- HttpRawClient: ~250 lines
- HttpRawServer: ~500 lines
- Total: ~2100 lines

### After Refactoring (Estimated)
- Common utilities: ~200 lines (new)
- StdioRawIncoming: ~450 lines (-200)
- StdioRawOutgoing: ~500 lines (-200)
- HttpRawClient: ~200 lines (-50)
- HttpRawServer: ~400 lines (-100)
- Total: ~1750 lines

**Net reduction: ~350 lines (>15% overall, >50% of duplicated code)**

## Validation Steps

1. **All tests pass**: `cargo test transport::`
2. **No behavior changes**: Same API, same results
3. **Performance unchanged**: Run benchmarks if available
4. **Clippy clean**: `cargo clippy -- -D warnings`

## Success Criteria

- [ ] StdioRawIncoming refactored to use utilities
- [ ] StdioRawOutgoing refactored to use utilities
- [ ] HttpRawClient refactored to use utilities
- [ ] HttpRawServer refactored to use utilities
- [ ] SSE transports refactored
- [ ] All tests passing
- [ ] Code duplication reduced by >50%
- [ ] No performance regression

## Risk Mitigation

- **Risk**: Subtle behavior changes
- **Mitigation**: Extensive test coverage, careful review of each change

- **Risk**: Performance regression
- **Mitigation**: Benchmark critical paths before/after

- **Risk**: Breaking API compatibility
- **Mitigation**: Public API remains exactly the same

## Notes

- This is pure refactoring - no new features
- Preserve all existing optimizations (buffer pooling, etc.)
- Keep transport-specific logic in the transport types
- Only extract truly common patterns to utilities

---

**Task Status**: Ready (depends on C.0)
**Dependencies**: C.0 must be complete
**Next Task**: C.2 - Optimize and validate