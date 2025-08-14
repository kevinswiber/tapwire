# Task C.1: Complete Transport Migration to Directional Traits

## Problem Statement

We currently have two transport systems running in parallel:
1. **Old System**: `trait Transport` with implementations like `StdioTransport`, `HttpTransport`
2. **New System**: `IncomingTransport` and `OutgoingTransport` directional traits

This dual system creates confusion, maintenance burden, and architectural debt. ForwardProxy has been migrated to the new system, but many components still use the old system.

## Objective

Complete the migration of ALL transport usage to the directional trait system and remove the old Transport trait entirely.

## Current State Analysis

### Components Using New Directional Transports ✅
- `ForwardProxy` - Fully migrated to Box<dyn IncomingTransport> and Box<dyn OutgoingTransport>
- `DirectionalTransportFactory` - Creates directional transports
- `API layer` - Uses factory to create transports

### Components Still Using Old Transport ❌
- `ReverseProxy` - Uses `StdioTransport` for upstream connections
- `MockTransport` in tests - Implements old `Transport` trait
- Examples - Use old transports with ForwardProxy (broken)
- Recording/replay - May use old transport patterns
- Various tests - Expect old Transport implementations

## Implementation Plan

### Step 1: Create Adapter Traits (2h)
Create temporary adapters to help with migration:

```rust
// src/transport/migration_helpers.rs

/// Adapter to make old Transport work as OutgoingTransport
pub struct TransportToOutgoing<T: Transport> {
    inner: T,
}

impl<T: Transport> OutgoingTransport for TransportToOutgoing<T> {
    async fn connect(&mut self) -> Result<()> {
        self.inner.connect().await
    }
    
    async fn send_request(&mut self, request: MessageEnvelope) -> Result<()> {
        self.inner.send(request).await
    }
    
    async fn receive_response(&mut self) -> Result<MessageEnvelope> {
        self.inner.receive().await
    }
    
    // ... other methods
}
```

### Step 2: Update MockTransport (3h)

Location: `tests/common/` or embedded in test files

```rust
impl IncomingTransport for MockTransport {
    async fn receive_request(&mut self) -> Result<MessageEnvelope> {
        // Reuse existing receive() logic
        self.receive().await
    }
    
    async fn send_response(&mut self, response: MessageEnvelope) -> Result<()> {
        // Reuse existing send() logic
        self.send(response).await
    }
    
    async fn bind_address(&self) -> Result<String> {
        Ok("mock://test".to_string())
    }
    
    async fn close(&mut self) -> Result<()> {
        self.close().await
    }
}

impl OutgoingTransport for MockTransport {
    async fn connect(&mut self) -> Result<()> {
        self.connect().await
    }
    
    async fn send_request(&mut self, request: MessageEnvelope) -> Result<()> {
        self.send(request).await
    }
    
    async fn receive_response(&mut self) -> Result<MessageEnvelope> {
        self.receive().await
    }
    
    // ... other required methods
}
```

### Step 3: Update ReverseProxy (4h)

Replace StdioTransport usage:

```rust
// Old
let mut transport = StdioTransport::new(cmd);
transport.connect().await?;
transport.send(envelope).await?;
let response = transport.receive().await?;

// New
let mut transport = SubprocessOutgoing::new(command_string);
transport.connect().await?;
transport.send_request(envelope).await?;
let response = transport.receive_response().await?;
```

Consider creating a pool adapter:
```rust
pub struct PoolableSubprocessTransport {
    inner: SubprocessOutgoing,
    // pool-specific fields
}
```

### Step 4: Update Examples (2h)

Fix compilation in examples:

```rust
// Old
proxy.start(client_transport, server_transport).await

// New
proxy.start(
    Box::new(StdioIncoming::new()),
    Box::new(SubprocessOutgoing::new(command))
).await
```

### Step 5: Update All Tests (3h)

1. Find all test files using old transports
2. Update to use directional transports or MockTransport with new traits
3. Box transports when passing to ForwardProxy

### Step 6: Remove Old System (4h)

1. **Delete trait definition**: `src/transport/mod.rs` - Remove `trait Transport`
2. **Delete implementations**:
   - `src/transport/stdio.rs` - StdioTransport
   - `src/transport/stdio_client.rs` - StdioClientTransport  
   - `src/transport/http.rs` - HttpTransport
   - `src/transport/http_mcp.rs` - HttpMcpTransport
   - `src/transport/sse_transport.rs` - SseTransport
3. **Delete old factory**: `src/transport/factory.rs`
4. **Update imports**: Remove all references to deleted types

### Step 7: Documentation Update (2h)

1. Update `docs/architecture.md`
2. Update module documentation
3. Create migration guide for external users

## Testing Strategy

### Phase 1: Get Everything Compiling
```bash
cargo check
cargo check --tests
cargo check --examples
```

### Phase 2: Fix Unit Tests
```bash
cargo test --lib
```

### Phase 3: Fix Integration Tests
```bash
cargo test --test integration_api_mock
cargo test --test transport_regression_suite
```

### Phase 4: Validate Examples
```bash
cargo build --examples
cargo run --example advanced_module_usage
```

### Phase 5: Final Validation
```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

## Risk Mitigation

### Risks
1. **Breaking changes** - External users may depend on old Transport trait
2. **Hidden dependencies** - Some code may implicitly rely on old behavior
3. **Performance regression** - New abstractions might have different performance
4. **Pool compatibility** - Connection pooling may need significant rework

### Mitigation Strategies
1. **Gradual migration** - Use adapters first, then remove
2. **Extensive testing** - Run full test suite after each change
3. **Performance benchmarks** - Compare before/after performance
4. **Version tagging** - Tag version before migration for rollback

## Success Criteria

- [ ] All code compiles without errors
- [ ] All 869+ tests pass
- [ ] Zero clippy warnings
- [ ] Examples run successfully
- [ ] No old Transport trait in codebase
- [ ] Documentation updated
- [ ] Performance within 5% of baseline

## Rollback Plan

If migration causes critical issues:
1. Git revert to tagged version
2. Re-apply only ForwardProxy changes
3. Document issues for future attempt
4. Consider keeping adapter layer longer-term

## Dependencies

- DirectionalTransport implementation (complete)
- ForwardProxy migration (complete)
- Understanding of all Transport usage sites (in progress)

## Estimated Time

Total: 20 hours
- Adapter creation: 2h
- MockTransport: 3h
- ReverseProxy: 4h
- Examples: 2h
- Tests: 3h
- Removal: 4h
- Documentation: 2h

## Notes

This is a critical refactor that will significantly improve code maintainability. The current dual-system state is confusing and will only get worse over time. Complete this migration as soon as possible.

### Key Insight
The main challenge is that old Transport combines both directions (send/receive) while new system separates them. Most migrations will involve:
1. Choosing the appropriate directional trait
2. Renaming methods (send → send_request/send_response)
3. Boxing for trait objects

### Alternative Approach
If full migration proves too risky, consider:
1. Keeping Transport trait but making it a supertrait of both directional traits
2. Gradually migrating implementations
3. Eventually removing Transport trait when no longer needed

However, the clean break approach is preferred for clarity.