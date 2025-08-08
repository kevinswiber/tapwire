# Task A.4: Document Breaking Changes

**Duration**: 1 hour  
**Dependencies**: A.3 (Create Migration Strategy)  
**Status**: ⬜ Not Started  

## Objective

Identify and document all breaking changes that will occur during the transport context refactor, categorize them by severity and scope, and provide migration guidance for each.

## Breaking Change Categories

### Category 1: Unavoidable Breaks
Changes that cannot be avoided even with compatibility layers.

### Category 2: Delayed Breaks  
Changes that can be postponed using compatibility layers but will eventually break.

### Category 3: Optional Breaks
Changes that improve the API but can be permanently avoided if needed.

## Identified Breaking Changes

### 1. Transport Trait Signature Changes

**Category**: Delayed Break  
**Severity**: High  
**Scope**: All transport implementations (3 files) and custom transports  

#### Current Signature
```rust
#[async_trait]
pub trait Transport: Send + Sync {
    async fn receive(&mut self) -> TransportResult<TransportMessage>;
    async fn send(&mut self, message: TransportMessage) -> TransportResult<()>;
}
```

#### New Signature
```rust
#[async_trait]
pub trait Transport: Send + Sync {
    async fn receive(&mut self) -> TransportResult<TransportMessage>;
    async fn send(&mut self, message: TransportMessage) -> TransportResult<()>;
    
    // New methods with default implementations
    async fn receive_envelope(&mut self) -> TransportResult<MessageEnvelope> {
        self.receive().await.map(Into::into)
    }
    async fn send_envelope(&mut self, envelope: MessageEnvelope) -> TransportResult<()> {
        self.send(envelope.into_transport_message()).await
    }
}
```

#### Migration Path
```rust
// Phase 1: Add new methods with defaults (non-breaking)
// Phase 2: Encourage migration to new methods  
// Phase 3: Deprecate old methods
// Phase 4: Remove old methods (breaking)
```

### 2. TransportMessage Enum Structure

**Category**: Delayed Break  
**Severity**: High  
**Scope**: 90 files with pattern matching  

#### Current Structure
```rust
pub enum TransportMessage {
    Request { id: String, method: String, params: Value },
    Response { id: String, result: Option<Value>, error: Option<Value> },
    Notification { method: String, params: Value },
}
```

#### Issue
- No direction information for notifications
- No transport metadata
- No session context

#### Migration Path
```rust
// Phase 1: Type alias (non-breaking)
pub type TransportMessage = ProtocolMessage;

// Phase 2: Add direction field with default (semi-breaking)
pub enum ProtocolMessage {
    // ...
    Notification {
        method: String,
        params: Value,
        #[serde(default)]
        direction: MessageDirection,  // Defaults to ClientToServer
    },
}

// Phase 3: Require direction (breaking)
```

### 3. Proxy Forward/Send Method Signatures

**Category**: Unavoidable Break  
**Severity**: Medium  
**Scope**: Proxy implementations and users  

#### Current
```rust
impl ForwardProxy {
    pub async fn forward(
        &mut self,
        message: TransportMessage,
        upstream: &mut Transport,
    ) -> TransportResult<TransportMessage>
}
```

#### New
```rust
impl ForwardProxy {
    pub async fn forward(
        &mut self,
        envelope: MessageEnvelope,  // Changed parameter type
        upstream: &mut Transport,
    ) -> TransportResult<MessageEnvelope>  // Changed return type
}
```

#### Migration Guidance
```rust
// Temporary adapter
impl ForwardProxy {
    pub async fn forward_legacy(
        &mut self,
        message: TransportMessage,
        upstream: &mut Transport,
    ) -> TransportResult<TransportMessage> {
        let envelope = MessageEnvelope::from(message);
        let result = self.forward(envelope, upstream).await?;
        Ok(result.into_transport_message())
    }
}
```

### 4. Session Manager Integration

**Category**: Delayed Break  
**Severity**: Medium  
**Scope**: Session management and tracking  

#### Current
```rust
pub struct Session {
    pub id: SessionId,
    pub transport_type: TransportType,
    pub created_at: Instant,
}
```

#### New
```rust
pub struct Session {
    pub id: SessionId,
    pub context: SessionContext,  // Rich context
    pub transport_type: TransportType,
    pub created_at: Instant,
}
```

#### Migration Impact
- Session serialization format changes
- Database schema updates needed
- Existing sessions need migration

### 5. Interceptor Rules API

**Category**: Optional Break  
**Severity**: Low  
**Scope**: Interceptor rule definitions  

#### Current
```rust
pub trait InterceptorRule {
    fn matches(&self, message: &TransportMessage) -> bool;
    fn apply(&self, message: TransportMessage) -> TransportMessage;
}
```

#### Enhanced (Optional)
```rust
pub trait InterceptorRule {
    fn matches(&self, message: &TransportMessage) -> bool;
    fn matches_envelope(&self, envelope: &MessageEnvelope) -> bool {
        self.matches(&envelope.message)
    }
    fn apply(&self, message: TransportMessage) -> TransportMessage;
    fn apply_envelope(&self, envelope: MessageEnvelope) -> MessageEnvelope {
        let message = self.apply(envelope.message);
        envelope.with_message(message)
    }
}
```

### 6. Serialization Format Changes

**Category**: Unavoidable Break (for new features)  
**Severity**: Medium  
**Scope**: Wire protocol, stored recordings  

#### Issue
- Adding direction to notifications changes JSON structure
- Context metadata not representable in current format

#### Wire Format Change Example
```json
// OLD: Notification without direction
{
  "jsonrpc": "2.0",
  "method": "tools/list",
  "params": {}
}

// NEW: Notification with direction (breaking for deserializers)
{
  "jsonrpc": "2.0",
  "method": "tools/list",
  "params": {},
  "_direction": "client_to_server"  // Or embed in different way
}
```

#### Migration Approach
- Use versioned serialization
- Support reading old format
- Write new format with flag

### 7. Public API Module Exports

**Category**: Delayed Break  
**Severity**: Low  
**Scope**: External crate users  

#### Current
```rust
pub mod transport {
    pub use self::message::TransportMessage;
    pub use self::traits::Transport;
}
```

#### New
```rust
pub mod transport {
    // Old (deprecated)
    #[deprecated]
    pub use self::message::TransportMessage;
    
    // New
    pub use self::envelope::{MessageEnvelope, MessageContext};
    pub use self::protocol::ProtocolMessage;
    pub use self::traits::{Transport, TransportWithContext};
}
```

## Breaking Change Timeline

### Immediate (Phase 1)
- No breaking changes
- All changes additive
- Full backward compatibility

### Short-term (Phase 2-3) 
- Transport trait gets new methods (non-breaking with defaults)
- New types available alongside old
- Deprecation warnings added

### Medium-term (Phase 4)
- Direction field required for notifications
- Proxy methods use envelopes
- Session context required

### Long-term (Phase 5)
- Remove TransportMessage type alias
- Remove compatibility layer
- Remove deprecated methods

## Migration Guidance Document

### For Library Users

```markdown
# Migration Guide: Transport Context Refactor

## Overview
Shadowcat is migrating from TransportMessage to MessageEnvelope to better separate protocol and transport concerns.

## Timeline
- v0.2.0: New types available, full compatibility
- v0.3.0: Deprecation warnings, migration recommended  
- v0.4.0: Old types removed, migration required

## Step-by-Step Migration

### Step 1: Update Imports
```rust
// Old
use shadowcat::transport::TransportMessage;

// New  
use shadowcat::transport::{MessageEnvelope, ProtocolMessage};
```

### Step 2: Update Pattern Matching
```rust
// Old
match message {
    TransportMessage::Notification { method, params } => ...
}

// New
match envelope.message {
    ProtocolMessage::Notification { method, params, direction } => ...
}
```

### Step 3: Update Transport Usage
```rust
// Old
let message = transport.receive().await?;

// New
let envelope = transport.receive_envelope().await?;
let message = envelope.message;  // If you need just the message
```

## Common Patterns

### Preserving Context
```rust
let envelope = transport.receive_envelope().await?;
// Process message but keep context
let processed = process(envelope.message);
let response = envelope.with_message(processed);
transport.send_envelope(response).await?;
```

### Adding Context
```rust
let envelope = MessageEnvelope::new(message)
    .with_direction(MessageDirection::ServerToClient)
    .with_session(session_context);
```
```

## Mitigation Strategies

### 1. Feature Flags
```toml
[dependencies]
shadowcat = { version = "0.2", features = ["legacy-compat"] }
```

### 2. Gradual Migration
- Migrate one component at a time
- Use compatibility layer during transition
- Test thoroughly at each step

### 3. Version Pinning
```toml
[dependencies]
shadowcat = "=0.1.5"  # Pin to pre-refactor version
```

## Communication Plan

### Announcement Timeline
1. **Pre-announcement** (2 weeks before): Blog post explaining why
2. **Release announcement** (v0.2.0): Compatibility release
3. **Migration reminder** (v0.3.0): Deprecation warnings active
4. **Final warning** (1 month before v0.4.0): Breaking change coming
5. **Breaking release** (v0.4.0): Migration required

### Channels
- GitHub release notes
- Project blog
- Discord announcement
- Email to major users

## Support Matrix

| Version | TransportMessage | MessageEnvelope | Support Until |
|---------|-----------------|-----------------|---------------|
| 0.1.x   | ✅ | ❌ | 2025-09-01 |
| 0.2.x   | ✅ (deprecated) | ✅ | 2025-12-01 |
| 0.3.x   | ⚠️ (warnings) | ✅ | 2026-03-01 |
| 0.4.x   | ❌ | ✅ | Active |

## Deliverables

### 1. Breaking Changes Document
**Location**: `plans/transport-context-refactor/breaking-changes.md`
- Complete list of breaks
- Severity assessment
- Migration paths
- Timeline

### 2. Migration Guide
**Location**: `docs/migration/transport-context.md`
- User-facing guide
- Code examples
- Common patterns
- Troubleshooting

### 3. Compatibility Checklist
**Location**: `plans/transport-context-refactor/compatibility-checklist.md`
- Component-by-component status
- Test coverage
- Known issues
- Workarounds

## Success Criteria

- [ ] All breaking changes identified
- [ ] Severity assessed for each change
- [ ] Migration path defined for each change
- [ ] Timeline realistic and communicated
- [ ] Support matrix clear
- [ ] Migration guide complete
- [ ] Compatibility strategy tested

## Notes

- Consider SemVer implications carefully
- Provide generous deprecation periods
- Over-communicate changes
- Provide tooling to assist migration where possible
- Consider automated migration tools

## Related Tasks

- **Depends on**: A.3 (Migration Strategy)
- **Next**: Phase 1 implementation begins
- **Informs**: All implementation phases

---

**Task Owner**: _Unassigned_  
**Created**: 2025-08-08  
**Last Updated**: 2025-08-08