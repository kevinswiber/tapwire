# Task A.3: Create Migration Strategy

**Duration**: 2 hours  
**Dependencies**: A.2 (Design MessageEnvelope Structure)  
**Status**: ⬜ Not Started  

## Objective

Create a comprehensive migration strategy for transitioning from `TransportMessage` to `MessageEnvelope` across 90 files with 658 occurrences, ensuring minimal disruption and maintaining backward compatibility.

## Migration Constraints

### Hard Requirements
- **Zero downtime** - System must remain operational during migration
- **Backward compatibility** - Existing code must continue to work
- **Incremental rollout** - Must be able to migrate component by component
- **Rollback capability** - Must be able to revert if issues arise

### Soft Requirements
- Minimize code duplication during transition
- Clear migration path for each component
- Performance parity or improvement
- Maintain test coverage throughout

## Migration Phases Overview

```
Phase 0: Analysis & Design (Current)
   ↓
Phase 1: Parallel Infrastructure
   - New types alongside old
   - Compatibility layer active
   - No breaking changes
   ↓
Phase 2: Core Components
   - Transport implementations
   - Session management
   - Critical paths migrated
   ↓
Phase 3: Proxy & Interceptors  
   - Forward/reverse proxy
   - Interceptor chain
   - Context propagation
   ↓
Phase 4: Peripheral Components
   - Recorder/replay
   - Audit/metrics
   - Rate limiting
   ↓
Phase 5: Cleanup
   - Remove compatibility layer
   - Delete old types
   - Update documentation
```

## Detailed Migration Strategy

### Phase 1: Parallel Infrastructure (Week 1)

#### Goals
- Introduce new types without breaking existing code
- Establish compatibility layer
- Enable gradual adoption

#### Implementation Steps

1. **Add New Types** (No breaking changes)
```rust
// src/transport/envelope.rs - NEW FILE
pub struct MessageEnvelope { ... }
pub struct MessageContext { ... }

// src/transport/mod.rs - ADD alongside existing
pub use envelope::{MessageEnvelope, MessageContext};
// Keep existing: pub use TransportMessage;
```

2. **Create Compatibility Layer**
```rust
// src/transport/compatibility.rs - NEW FILE
impl From<TransportMessage> for MessageEnvelope { ... }
impl From<MessageEnvelope> for TransportMessage { ... }

// Temporary trait for dual support
pub trait DualTransport: Transport {
    fn receive_any(&mut self) -> Result<Either<TransportMessage, MessageEnvelope>>;
}
```

3. **Add Feature Flag** (Optional rollback)
```toml
[features]
message-envelope = []  # New system
legacy-transport = []   # Force old behavior
```

#### Verification
- [ ] Existing tests still pass
- [ ] New types compile
- [ ] Conversion round-trips work

### Phase 2: Core Components Migration (Week 1-2)

#### Priority Order (Based on A.1 Analysis)
1. **Transport Implementations** (Source of context)
   - `transport/stdio.rs` - Simplest, migrate first
   - `transport/http.rs` - Add header context
   - `transport/http_mcp.rs` - MCP-specific handling

2. **Session Management** (Context consumer)
   - `session/manager.rs` - Track context per session
   - `session/store.rs` - Store context with session

3. **Base Proxy Logic** (Context propagator)
   - `proxy/mod.rs` - Add context-aware trait
   - `proxy/forward.rs` - Preserve context
   - `proxy/reverse.rs` - Extract/inject context

#### Migration Pattern for Each Component

```rust
// BEFORE: transport/stdio.rs
impl Transport for StdioTransport {
    async fn receive(&mut self) -> Result<TransportMessage> {
        // existing implementation
    }
}

// AFTER: transport/stdio.rs
impl Transport for StdioTransport {
    async fn receive(&mut self) -> Result<TransportMessage> {
        // Compatibility: still works
        self.receive_envelope().await
            .map(|env| env.into_transport_message())
    }
}

impl TransportWithContext for StdioTransport {
    async fn receive_envelope(&mut self) -> Result<MessageEnvelope> {
        // New implementation with context
        let message = self.read_message().await?;
        Ok(MessageEnvelope::new(message)
            .with_transport(self.create_context()))
    }
}
```

#### Component Migration Checklist

For each component:
- [ ] Add context support alongside existing
- [ ] Update tests to verify both paths
- [ ] Performance benchmark before/after
- [ ] Document context handling
- [ ] Mark component as migrated

### Phase 3: Proxy & Interceptor Migration (Week 2)

#### Components
- `proxy/forward.rs` - Context preservation
- `proxy/reverse.rs` - Context transformation
- `interceptor/engine.rs` - Context-aware rules
- `interceptor/rules.rs` - Match on context

#### Critical Behaviors to Preserve

1. **Context Forwarding**
```rust
// Forward proxy must preserve context
async fn forward_with_context(
    upstream: &mut Transport,
    envelope: MessageEnvelope,
) -> Result<MessageEnvelope> {
    // Preserve original context
    let mut response = upstream.send_envelope(envelope).await?;
    response.context.merge_from(envelope.context);
    Ok(response)
}
```

2. **Context Injection (Reverse Proxy)**
```rust
// Reverse proxy extracts HTTP context
async fn handle_http_request(
    req: Request<Body>,
    upstream: &mut Transport,
) -> Result<Response<Body>> {
    let context = extract_http_context(&req);
    let envelope = parse_body(req.body())
        .with_context(context);
    // ...
}
```

### Phase 4: Peripheral Components (Week 2-3)

#### Lower Priority Components
- `recorder/` - Add context to recordings
- `audit/` - Log context information
- `metrics/` - Track context metrics
- `rate_limiting/` - Context-aware limits

#### Migration Approach
- Can use compatibility layer longer
- Migrate opportunistically
- Focus on not breaking existing behavior

### Phase 5: Cleanup (Week 3)

#### Steps
1. **Deprecate Old Types**
```rust
#[deprecated(since = "0.2.0", note = "Use MessageEnvelope instead")]
pub type TransportMessage = ProtocolMessage;
```

2. **Remove Compatibility Layer** (After grace period)
3. **Update all imports**
4. **Remove feature flags**
5. **Final documentation update**

## Rollback Strategy

### Level 1: Feature Flag Rollback
```rust
#[cfg(not(feature = "message-envelope"))]
pub type MessageEnvelope = TransportMessage;  // Alias to old type
```

### Level 2: Git Revert
- Each phase in separate PR
- Can revert individual phases
- Keep changes isolated

### Level 3: Compatibility Mode
```rust
// Runtime fallback
if std::env::var("USE_LEGACY_TRANSPORT").is_ok() {
    return self.receive_legacy().await;
}
```

## Testing Strategy

### Test Categories

1. **Compatibility Tests**
```rust
#[test]
fn test_round_trip_conversion() {
    let old = create_transport_message();
    let env: MessageEnvelope = old.clone().into();
    let back: TransportMessage = env.into();
    assert_eq!(old, back);
}
```

2. **Parallel Tests**
```rust
#[test]
async fn test_both_paths_work() {
    let mut transport = create_transport();
    
    // Old path
    let msg1 = transport.receive().await.unwrap();
    
    // New path  
    let env2 = transport.receive_envelope().await.unwrap();
    
    // Should be equivalent
    assert_eq!(msg1, env2.into_transport_message());
}
```

3. **Performance Tests**
```rust
#[bench]
fn bench_message_processing_old(b: &mut Bencher) {
    b.iter(|| process_transport_message(create_message()));
}

#[bench]
fn bench_message_processing_new(b: &mut Bencher) {
    b.iter(|| process_message_envelope(create_envelope()));
}
```

### Test Migration Order
1. Add tests for new functionality
2. Keep existing tests passing
3. Add compatibility tests
4. Add performance benchmarks
5. Remove old tests only in Phase 5

## Communication Plan

### For Each Phase

1. **Before Starting**
   - Update this strategy document
   - Notify team of phase beginning
   - Create phase-specific PR

2. **During Implementation**
   - Daily status in tracker
   - Flag any blockers immediately
   - Update migration checklist

3. **After Completion**
   - Run full test suite
   - Performance benchmarks
   - Update progress metrics
   - Document lessons learned

### Progress Metrics

Track in `migration-progress.md`:
```markdown
## Migration Progress

### Phase 1: Infrastructure ✅
- New types: 100%
- Compatibility layer: 100%
- Tests: 100%

### Phase 2: Core Components 🔄
- Transports: 2/3 (66%)
- Session: 1/2 (50%)
- Proxy: 0/2 (0%)

### Overall Progress
- Files migrated: 15/90 (17%)
- Occurrences updated: 89/658 (14%)
- Tests passing: 100%
- Performance delta: +0.3%
```

## Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking production code | Low | High | Feature flags, gradual rollout |
| Performance regression | Medium | Medium | Benchmark each phase |
| Incomplete migration | Low | Medium | Clear checklist, tracking |
| Context loss in forwarding | Medium | High | Comprehensive tests |
| Memory leaks from Arc cycles | Low | Medium | Memory profiling |

## Deliverables

### 1. Migration Guide
**Location**: `plans/transport-context-refactor/migration-guide.md`
- Step-by-step instructions
- Code examples
- Common pitfalls
- FAQ

### 2. Progress Tracker
**Location**: `plans/transport-context-refactor/migration-progress.md`
- Real-time status
- Blockers and issues
- Performance metrics
- Test results

### 3. Compatibility Matrix
**Location**: `plans/transport-context-refactor/compatibility-matrix.md`
- Component status
- Version compatibility
- Feature flag settings
- Known issues

## Success Criteria

- [ ] All phases documented
- [ ] Rollback strategy tested
- [ ] Performance benchmarks established
- [ ] Communication plan clear
- [ ] Risk matrix complete
- [ ] Timeline realistic
- [ ] Dependencies identified

## Notes

- Consider using `cargo-edit` to update dependencies
- Use `cargo-outdated` to check compatibility
- Consider `cargo-breaking` for API changes
- Monitor CI/CD closely during migration
- Keep PRs small and focused

## Related Tasks

- **Depends on**: A.2 (Design)
- **Next**: A.4 - Document Breaking Changes
- **Enables**: Phase 1 implementation

---

**Task Owner**: _Unassigned_  
**Created**: 2025-08-08  
**Last Updated**: 2025-08-08