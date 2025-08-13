# Breaking Changes Documentation [NOT RELEVANT - NO USERS]

> ⚠️ **NOTE: This document is NOT RELEVANT**
> 
> **We have NO external users - Shadowcat hasn't been released!**
> 
> This document was written assuming we needed to maintain backward compatibility.
> Since we have no users, we can:
> - Break any API we want
> - Delete old code immediately
> - Skip all compatibility layers
> - Not worry about migration paths
> 
> **See:** [migration-strategy-simplified.md](migration-strategy-simplified.md) for the approach we're actually using.

---

## [NOT RELEVANT] Executive Summary

~~The Transport Context Refactor introduces the `MessageEnvelope` system to properly separate protocol messages from transport metadata. While designed for maximum backward compatibility, some breaking changes are unavoidable. This document catalogs all breaking changes, their severity, timeline, and migration paths.~~

## Breaking Change Categories

### Severity Levels
- **CRITICAL**: Breaks compilation or runtime behavior immediately
- **HIGH**: Major API changes affecting most users
- **MEDIUM**: API changes affecting some users
- **LOW**: Minor changes, deprecations, or optional improvements

### Timeline Categories
- **Immediate**: Cannot be avoided, breaks in Phase 1
- **Delayed**: Can be postponed with compatibility layer
- **Optional**: Can be permanently avoided if needed

## Detailed Breaking Changes

### 1. Transport Trait Evolution

**Severity**: HIGH  
**Timeline**: Delayed (Phase 2-4)  
**Affected**: All transport implementations and custom transports

#### The Change
```rust
// OLD - Current Transport trait
#[async_trait]
pub trait Transport: Send + Sync {
    async fn receive(&mut self) -> TransportResult<TransportMessage>;
    async fn send(&mut self, message: TransportMessage) -> TransportResult<()>;
}

// NEW - Extended Transport trait
#[async_trait]
pub trait Transport: Send + Sync {
    // Old methods (deprecated in Phase 3)
    async fn receive(&mut self) -> TransportResult<TransportMessage>;
    async fn send(&mut self, message: TransportMessage) -> TransportResult<()>;
    
    // New methods (required in Phase 4)
    async fn receive_envelope(&mut self) -> TransportResult<MessageEnvelope>;
    async fn send_envelope(&mut self, envelope: MessageEnvelope) -> TransportResult<()>;
    fn current_context(&self) -> TransportContext;
}
```

#### Migration Path
```rust
// Phase 1-2: Both methods work
let msg = transport.receive().await?;  // Still works
let env = transport.receive_envelope().await?;  // New way

// Phase 3: Deprecation warnings
#[deprecated(note = "Use receive_envelope() instead")]
async fn receive(&mut self) -> TransportResult<TransportMessage>

// Phase 4: Old methods removed (BREAKING)
```

#### Who This Affects
- Custom transport implementations
- Direct transport users (not using proxy)
- Test code using mock transports

### 2. TransportMessage to ProtocolMessage Rename

**Severity**: MEDIUM  
**Timeline**: Delayed (Phase 5)  
**Affected**: All code using TransportMessage (34 files, 330 occurrences)

#### The Change
```rust
// OLD
pub enum TransportMessage {
    Request { id: String, method: String, params: Value },
    Response { id: String, result: Option<Value>, error: Option<Value> },
    Notification { method: String, params: Value },
}

// INTERMEDIATE (Phase 1-4)
pub type TransportMessage = ProtocolMessage;  // Type alias for compatibility

// FINAL (Phase 5)
pub enum ProtocolMessage {
    Request { id: MessageId, method: String, params: Value },
    Response { id: MessageId, result: Option<Value>, error: Option<ErrorObject> },
    Notification { method: String, params: Value },
}
```

#### Migration Path
```rust
// Phase 1-4: Both names work
use transport::TransportMessage;  // Works via type alias
use transport::ProtocolMessage;   // Direct use of new name

// Phase 5: Must use new name
use transport::ProtocolMessage;   // TransportMessage removed
```

### 3. Direction Requirement for Notifications

**Severity**: HIGH  
**Timeline**: Immediate for new features, Delayed for existing code  
**Affected**: Notification routing, SSE integration

#### The Problem
Currently, notification direction is implicit and often wrong:
```rust
// OLD - No direction information
TransportMessage::Notification { method: "tools/list", params }
// Is this from client or server? No way to know!
```

#### The Solution
```rust
// NEW - Explicit direction in context
MessageEnvelope {
    message: ProtocolMessage::Notification { method: "tools/list", params },
    context: MessageContext {
        direction: MessageDirection::ServerToClient,  // Now explicit!
        ...
    }
}
```

#### Migration Impact
- Existing code using compatibility layer gets `MessageDirection::Unknown`
- New code MUST specify direction for correct routing
- SSE integration REQUIRES proper direction

### 4. Frame Structure Changes

**Severity**: MEDIUM  
**Timeline**: Delayed (Phase 3)  
**Affected**: Session storage, recording, replay

#### The Change
```rust
// OLD
pub struct Frame {
    pub session_id: SessionId,
    pub direction: Direction,
    pub message: TransportMessage,
    pub timestamp: SystemTime,
}

// NEW
pub struct Frame {
    pub envelope: MessageEnvelope,  // Contains all previous fields
    pub timestamp: SystemTime,
}
```

#### Migration Path
```rust
// Compatibility conversion
impl From<OldFrame> for Frame {
    fn from(old: OldFrame) -> Self {
        Frame {
            envelope: MessageEnvelope::new(old.message)
                .with_direction(old.direction.into())
                .with_session(SessionContext::with_id(old.session_id)),
            timestamp: old.timestamp,
        }
    }
}
```

### 5. Proxy Method Signatures

**Severity**: HIGH  
**Timeline**: Delayed (Phase 4)  
**Affected**: Forward and reverse proxy users

#### The Changes
```rust
// OLD - Forward Proxy
pub async fn forward(
    &mut self,
    message: TransportMessage,
    upstream: &mut Transport,
) -> TransportResult<TransportMessage>

// NEW - Forward Proxy
pub async fn forward_envelope(
    &mut self,
    envelope: MessageEnvelope,
    upstream: &mut dyn TransportWithContext,
) -> TransportResult<MessageEnvelope>

// OLD - Reverse Proxy
pub async fn handle_request(
    &self,
    req: Request<Body>,
) -> Result<Response<Body>>

// NEW - Reverse Proxy (internal change, external API same)
// Internally uses MessageEnvelope for context preservation
```

#### Migration Path
```rust
// Temporary adapter methods
impl ForwardProxy {
    #[deprecated(note = "Use forward_envelope()")]
    pub async fn forward(...) -> TransportResult<TransportMessage> {
        let envelope = message.into();
        let result = self.forward_envelope(envelope, upstream).await?;
        Ok(result.into())
    }
}
```

### 6. Interceptor Rule Interface

**Severity**: LOW  
**Timeline**: Optional  
**Affected**: Custom interceptor implementations

#### The Enhancement
```rust
// OLD
pub trait InterceptorRule {
    fn matches(&self, message: &TransportMessage) -> bool;
    fn apply(&self, message: TransportMessage) -> TransportMessage;
}

// NEW (with default implementations)
pub trait InterceptorRule {
    // Old methods still work
    fn matches(&self, message: &TransportMessage) -> bool;
    fn apply(&self, message: TransportMessage) -> TransportMessage;
    
    // New context-aware methods (optional)
    fn matches_envelope(&self, envelope: &MessageEnvelope) -> bool {
        self.matches(&envelope.message)
    }
    fn apply_envelope(&self, envelope: MessageEnvelope) -> MessageEnvelope {
        envelope.with_message(self.apply(envelope.message.clone()))
    }
}
```

### 7. Session Storage Schema

**Severity**: MEDIUM  
**Timeline**: Delayed (Phase 3)  
**Affected**: Persistent session storage

#### The Change
```rust
// OLD - Session table schema
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    transport_type TEXT,
    created_at INTEGER
);

// NEW - Extended schema
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    transport_type TEXT,
    protocol_version TEXT,  -- New
    session_state TEXT,     -- New
    context_json TEXT,      -- New: Full context
    created_at INTEGER
);
```

#### Migration Strategy
- Add new columns with defaults
- Migrate existing sessions on read
- Write both formats during transition

### 8. Tape Format Version

**Severity**: LOW  
**Timeline**: Delayed (Phase 5)  
**Affected**: Recording and replay

#### The Change
```json
// OLD - Tape format v1
{
  "version": 1,
  "frames": [
    {
      "session_id": "abc",
      "direction": "ClientToServer",
      "message": { /* TransportMessage */ },
      "timestamp": 1234567890
    }
  ]
}

// NEW - Tape format v2
{
  "version": 2,
  "frames": [
    {
      "envelope": { /* MessageEnvelope with full context */ },
      "timestamp": 1234567890
    }
  ]
}
```

#### Compatibility
- Reader supports both formats
- Writer uses v2 by default
- Flag to force v1 output

## Breaking Change Timeline

### Phase 1 (Week 1, Day 1-2) - No Breaking Changes
- ✅ All changes additive
- ✅ Full backward compatibility
- ✅ Type aliases maintain compatibility

### Phase 2 (Week 1, Day 3-4) - Deprecation Warnings
- ⚠️ Deprecation warnings on old methods
- ✅ Both old and new APIs work
- ✅ No runtime breaks

### Phase 3 (Week 1, Day 5) - Soft Breaking
- ⚠️ Strong deprecation warnings
- ⚠️ Some internal APIs change
- ✅ Public API still compatible

### Phase 4 (Week 2, Day 1-2) - API Breaking
- ❌ Transport trait requires new methods
- ❌ Proxy methods use envelopes
- ⚠️ TransportMessage type alias deprecated

### Phase 5 (Week 2, Day 3) - Full Breaking
- ❌ TransportMessage type removed
- ❌ Old Transport methods removed
- ❌ Compatibility layer removed

## Migration Checklist

### For Library Users

- [ ] **Phase 1**: No action required
- [ ] **Phase 2**: Update imports to use new types (optional)
- [ ] **Phase 3**: Start migrating to envelope methods
- [ ] **Phase 4**: Complete migration to new APIs
- [ ] **Phase 5**: Remove all references to old types

### For Transport Implementers

- [ ] **Phase 1**: Review new trait methods
- [ ] **Phase 2**: Implement TransportWithContext trait
- [ ] **Phase 3**: Add context extraction logic
- [ ] **Phase 4**: Remove old method implementations
- [ ] **Phase 5**: Clean up compatibility code

### For Proxy Users

- [ ] **Phase 1**: No changes needed
- [ ] **Phase 2**: Review new forward_envelope method
- [ ] **Phase 3**: Start using envelope methods
- [ ] **Phase 4**: Migrate all proxy calls
- [ ] **Phase 5**: Remove old method calls

## Risk Assessment

| Component | Risk Level | Mitigation | Notes |
|-----------|------------|------------|-------|
| Transport implementations | HIGH | Default implementations, gradual migration | 3 core + unknown custom |
| Session management | MEDIUM | Compatibility layer, data migration | 44 direct uses |
| Proxy operations | HIGH | Adapter methods, phased rollout | Critical path |
| Interceptors | LOW | Optional new interface | Backward compatible |
| Recording/Replay | LOW | Version detection, dual format support | Non-critical |
| External integrations | MEDIUM | Long deprecation period | Unknown count |

## Communication Plan

### Pre-Release (2 weeks before v0.2.0)
- Blog post: "Preparing for Transport Context Refactor"
- Discord announcement
- GitHub discussion opened

### Release v0.2.0 (Phase 1-2)
- Release notes with migration guide
- No breaking changes yet
- Deprecation timeline announced

### Release v0.3.0 (Phase 3-4)
- Deprecation warnings active
- Migration guide updated
- Direct email to known users

### Release v0.4.0 (Phase 5)
- Breaking changes implemented
- Final migration guide
- Support for questions

## Support Strategy

### Documentation
- Migration guide with examples
- API documentation updated
- Video walkthrough available

### Tooling
- Migration lint rules
- Automated conversion script (partial)
- Compatibility checker

### Support Channels
- GitHub issues for bugs
- Discord for questions
- Office hours for major users

## Version Support Matrix

| Version | TransportMessage | MessageEnvelope | Direction Support | Support Until |
|---------|------------------|-----------------|-------------------|---------------|
| 0.1.x | ✅ Native | ❌ | ❌ | 2025-09-01 |
| 0.2.x | ✅ Via alias | ✅ Native | ⚠️ Optional | 2025-12-01 |
| 0.3.x | ⚠️ Deprecated | ✅ Native | ✅ Required | 2026-03-01 |
| 0.4.x | ❌ | ✅ Native | ✅ Required | Active |

## Conclusion

The Transport Context Refactor introduces necessary breaking changes to solve fundamental architectural issues, particularly around notification routing and transport metadata preservation. The phased approach with compatibility layers minimizes disruption, allowing users to migrate gradually over several releases.

Key points:
- **No immediate breaking changes** in Phase 1
- **6-month migration window** from first deprecation to removal
- **Clear migration path** for every breaking change
- **Compatibility layer** for gradual adoption
- **Comprehensive support** during transition

The benefits (proper notification routing, SSE support, transport metadata preservation) far outweigh the migration costs, and the careful rollout plan ensures minimal disruption to users.