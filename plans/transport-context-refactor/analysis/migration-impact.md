# Migration Impact Assessment

## Breaking Changes

### Unavoidable Breaks

1. **Transport Trait Signature Changes**
   - Current: `async fn send(&mut self, message: TransportMessage) -> Result<()>`
   - New: `async fn send(&mut self, envelope: MessageEnvelope) -> Result<()>`
   - Impact: All 5 transport implementations must be updated

2. **Session Manager API Changes**
   - Current: `record_frame(session_id, direction, message)`
   - New: `record_frame(envelope)` - direction now in envelope
   - Impact: 44 call sites in session/manager.rs

3. **Interceptor Interface Changes**
   - Current: `intercept(&mut self, message: TransportMessage) -> InterceptAction`
   - New: `intercept(&mut self, envelope: MessageEnvelope) -> InterceptAction`
   - Impact: All interceptor implementations

### Mitigatable Breaks

1. **Public Type Exports**
   - Can provide type alias: `type TransportMessage = McpMessage`
   - Implement `From<TransportMessage>` for `MessageEnvelope`
   - Default context for backward compatibility

2. **Pattern Matching Sites**
   - Provide helper methods for common patterns
   - Extension traits for ergonomic access
   - Gradual migration with both patterns supported

3. **Serialization Format**
   - Version field in tapes for compatibility
   - Deserializer that handles both formats
   - Automatic upgrade on read

## Compatibility Requirements

### Must Maintain Compatibility

1. **Wire Protocol Format**
   - JSON-RPC 2.0 structure unchanged
   - No additional fields in protocol messages
   - Transport headers remain separate

2. **CLI Interface**
   - All commands work identically
   - Same output formats by default
   - New context info only with flags

3. **Tape Recording Format**
   - Version 1 tapes still readable
   - Version 2 adds context metadata
   - Automatic format detection

4. **MCP Protocol Compliance**
   - Session management unchanged
   - Capability negotiation preserved
   - Message routing semantics maintained

### Can Break (Internal)

1. **Internal Module Interfaces**
   - Transport to Session Manager communication
   - Interceptor chain processing
   - Recorder internal formats

2. **Test Infrastructure**
   - Mock implementations
   - Test helpers and builders
   - Fixture formats

3. **Debug Representations**
   - Debug trait implementations
   - Logging formats
   - Error messages

## Performance Considerations

### Hot Paths

1. **Message Forwarding**
   - Current: ~10,000 msgs/sec capability
   - Added overhead: 1 allocation for envelope
   - Projected: < 2% performance impact

2. **Session Lookup**
   - Current: O(1) HashMap lookup
   - New: Same, but with richer context
   - No performance change expected

3. **Pattern Matching**
   - Current: Direct enum matching
   - New: May need nested matching
   - Mitigation: Provide direct accessors

### Memory Impact

```rust
// Current
size_of::<TransportMessage>() = ~280 bytes (with large Value fields)

// New
size_of::<MessageEnvelope>() = ~360 bytes
  - McpMessage: ~280 bytes
  - TransportContext: ~64 bytes
  - SessionContext: ~16 bytes

// Impact: ~28% increase per message
// At 1000 concurrent messages: 80KB additional memory
```

### Allocation Patterns

- **Current**: 1 allocation per message
- **New**: 1-2 allocations (message + optional context)
- **Mitigation**: Use `Arc` for shared context

## Risk Matrix

| Component | Risk Level | Impact | Likelihood | Mitigation |
|-----------|------------|--------|------------|------------|
| Session Manager | **CRITICAL** | All messages | High | Extensive testing, gradual rollout |
| Forward Proxy | **HIGH** | Request forwarding | High | Compatibility layer, A/B testing |
| Reverse Proxy | **HIGH** | Auth flows | Medium | Keep auth separate from refactor |
| Transport Layer | **HIGH** | All I/O | High | One transport at a time |
| Interceptors | **MEDIUM** | Message modification | Medium | Support both formats temporarily |
| Recorder | **MEDIUM** | Tape compatibility | Low | Versioned format |
| CLI Tools | **LOW** | Developer tools | Low | Update atomically |
| Tests | **LOW** | Development only | Low | Update as needed |

## Testing Requirements

### Unit Tests Needed

1. **Envelope Creation and Conversion**
   - From TransportMessage with default context
   - From parts (message, transport, session)
   - Round-trip serialization

2. **Context Preservation**
   - Through proxy forwarding
   - Through interceptor modification
   - Through session recording

3. **Direction Detection**
   - Client-to-server requests
   - Server-to-client responses
   - Bidirectional notifications

### Integration Tests Needed

1. **End-to-end Proxy Flow**
   - stdio client → proxy → stdio server
   - HTTP client → proxy → HTTP server
   - Mixed transport scenarios

2. **Session Continuity**
   - Context preserved across messages
   - Session recovery after disconnect
   - Multi-session handling

3. **Notification Routing**
   - Client notifications reach server
   - Server notifications reach client
   - Proper direction tracking

### Performance Tests Needed

1. **Throughput Regression**
   - Baseline current performance
   - Measure with envelopes
   - Target: < 5% degradation

2. **Memory Usage**
   - Per-message overhead
   - Session storage growth
   - Peak memory under load

3. **Latency Impact**
   - Message forwarding latency
   - Context creation overhead
   - Serialization performance

## Timeline Estimate

### Phase 0: Analysis and Design (10 hours) ✓
- Protocol analysis: 2 hours ✓
- Usage analysis: 3 hours ✓
- Design work: 3 hours (A.2)
- Migration strategy: 2 hours (A.3)

### Phase 1: Core Infrastructure (10 hours)
- Define new types: 3 hours
- Compatibility layer: 3 hours
- Conversion traits: 2 hours
- Basic tests: 2 hours

### Phase 2: Transport Layer (9 hours)
- stdio transport: 2 hours
- HTTP/MCP transport: 2 hours
- SSE transport: 2 hours
- Replay transport: 1 hour
- Transport tests: 2 hours

### Phase 3: Session & Proxy (8 hours)
- Session manager: 3 hours
- Forward proxy: 2 hours
- Reverse proxy: 2 hours
- Integration tests: 1 hour

### Phase 4: Supporting Systems (7 hours)
- Interceptors: 2 hours
- Recorder: 2 hours
- CLI tools: 1 hour
- Documentation: 2 hours

### Phase 5: Testing & Validation (6 hours)
- Performance testing: 2 hours
- Integration testing: 2 hours
- Bug fixes: 2 hours

### Buffer (10 hours)
- Unexpected issues
- Additional testing
- Documentation updates

**Total Estimate: 60 hours** (7.5 person-days)

## Migration Phases

### Phase 1: Non-Breaking Foundation
- Add new types alongside old
- Implement conversion traits
- No behavior changes

### Phase 2: Gradual Transport Migration
- Update transports to create envelopes internally
- Maintain TransportMessage API
- Test each transport independently

### Phase 3: Core System Updates
- Update session manager
- Update proxy layers
- Enable full context flow

### Phase 4: Deprecation
- Mark old types deprecated
- Update documentation
- Provide migration guide

### Phase 5: Cleanup
- Remove compatibility layers
- Optimize performance
- Final testing

## Success Metrics

1. **Functional**: All existing tests pass
2. **Performance**: < 5% latency increase
3. **Memory**: < 30% memory increase per message
4. **Compatibility**: All tapes remain playable
5. **Quality**: No increase in error rates
6. **Direction**: Notifications properly routed bidirectionally