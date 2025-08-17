# Task D.0: Create HttpOutgoing Transport

**Duration**: 2 hours  
**Dependencies**: Phase C complete  
**Priority**: HIGH

## Objective

Create an HttpOutgoing implementation of the OutgoingTransport trait to enable consistent transport abstraction for HTTP upstreams in the reverse proxy.

## Key Questions

1. How should we handle connection pooling in HttpOutgoing?
2. Should HttpOutgoing support both HTTP and HTTPS?
3. How do we handle SSE streaming through the trait interface?
4. Should we reuse existing HyperHttpClient or create new implementation?

## Process

### Step 1: Create HttpOutgoing Structure (30 min)

1. Create new file: `src/transport/directional/outgoing/http.rs`
2. Define HttpOutgoing struct with:
   - HyperHttpClient instance (reuse existing)
   - Target URL configuration
   - Session ID tracking
   - Response mode detection

### Step 2: Implement OutgoingTransport Trait (45 min)

```rust
impl OutgoingTransport for HttpOutgoing {
    async fn connect(&mut self) -> TransportResult<()>
    async fn send_request(&mut self, envelope: MessageEnvelope) -> TransportResult<()>
    async fn receive_response(&mut self) -> TransportResult<MessageEnvelope>
    async fn close(&mut self) -> TransportResult<()>
    fn is_connected(&self) -> bool
    fn transport_type(&self) -> TransportType
    // ... other trait methods
}
```

### Step 3: Handle SSE Streaming (30 min)

1. Detect SSE responses via Content-Type header
2. Return appropriate ResponseMode in envelope
3. Stream SSE events through receive_response
4. Maintain compatibility with existing SSE handling

### Step 4: Integration and Testing (15 min)

1. Update `src/transport/directional/outgoing/mod.rs` to export HttpOutgoing
2. Create unit tests for:
   - Basic request/response cycle
   - SSE streaming detection
   - Error handling
   - Connection lifecycle

### Step 5: Update Reverse Proxy Usage (10 min)

1. Modify reverse proxy to use HttpOutgoing for HTTP upstreams
2. Enable connection pooling through PoolableOutgoingTransport wrapper
3. Verify existing tests still pass

## Commands to Run

```bash
# Create the new file
touch src/transport/directional/outgoing/http.rs

# Run tests after implementation
cargo test transport::directional::outgoing::http
cargo test reverse_proxy

# Check for any regressions
cargo test --lib
```

## Deliverables

1. **File Created**: `src/transport/directional/outgoing/http.rs`
   - Complete HttpOutgoing implementation
   - Full OutgoingTransport trait implementation
   - Proper SSE handling

2. **Files Modified**:
   - `src/transport/directional/outgoing/mod.rs` - Export HttpOutgoing
   - `src/proxy/reverse/legacy.rs` - Use HttpOutgoing for HTTP upstreams

3. **Tests Added**:
   - Unit tests in http.rs
   - Integration test coverage

## Success Criteria

- [ ] HttpOutgoing implements OutgoingTransport trait completely
- [ ] Reverse proxy can use HttpOutgoing for HTTP upstreams
- [ ] Connection pooling works with PoolableOutgoingTransport wrapper
- [ ] SSE responses are properly detected and handled
- [ ] All existing tests pass
- [ ] No performance regression

## Risk Mitigation

- **Risk**: Breaking existing HTTP upstream handling
  - **Mitigation**: Keep old code path available until new one is proven
  
- **Risk**: SSE streaming incompatibility
  - **Mitigation**: Test thoroughly with existing SSE test cases

- **Risk**: Connection pooling issues
  - **Mitigation**: Verify pooling behavior with integration tests

## Notes

- This is a targeted improvement, not a full architectural change
- Focus on compatibility with existing reverse proxy code
- Avoid over-engineering - keep it simple and focused
- This enables better testing and consistency without major refactoring