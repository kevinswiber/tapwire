# Task A.2: Design MessageEnvelope Structure

**Duration**: 2 hours  
**Dependencies**: A.0 (MCP Protocol Analysis), A.1 (Usage Analysis)  
**Status**: ⬜ Not Started  

## Objective

Design the new `MessageEnvelope` structure and related types that properly separate protocol concerns from transport metadata, based on findings from protocol analysis and usage patterns.

## Design Requirements

### From Protocol Analysis (A.0)
- Clear separation of Transport, MCP, and JSON-RPC layers
- Support for bidirectional notifications with explicit direction
- Transport-agnostic protocol messages
- Extensible metadata system

### From Usage Analysis (A.1)
- Backward compatibility with existing TransportMessage
- Minimal performance overhead
- Support for existing workarounds
- Gradual migration path

### From SSE Integration Needs
- SSE event IDs and types
- Retry-After headers
- Stream correlation
- Event history tracking

## Design Decisions to Make

### 1. Type Structure
- Should we rename TransportMessage to McpMessage?
- How to handle the transition period?
- Should context be optional or required?
- How to minimize allocations?

### 2. Metadata Ownership
- Who owns the context (Transport, Session, Proxy)?
- How is context propagated through layers?
- When can context be modified?
- How to handle context merging?

### 3. Direction Semantics
- Explicit direction field vs implicit from context?
- How to handle direction for requests/responses?
- Should direction be transport-specific?

### 4. Extensibility
- How to add new transport types?
- How to handle transport-specific metadata?
- Forward compatibility considerations?

## Proposed Design

### Core Types

```rust
// src/transport/envelope.rs

use std::sync::Arc;
use std::borrow::Cow;

/// The complete message with all context
#[derive(Debug, Clone)]
pub struct MessageEnvelope {
    /// The protocol-level message
    pub message: ProtocolMessage,
    /// Full context including transport and session
    pub context: MessageContext,
}

/// Protocol-level message (renamed from TransportMessage)
/// This represents MCP semantic messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolMessage {
    Request {
        id: MessageId,
        method: String,
        params: Value,
    },
    Response {
        id: MessageId,
        result: Option<Value>,
        error: Option<ErrorObject>,
    },
    Notification {
        method: String,
        params: Value,
    },
}

/// Complete context for message handling
#[derive(Debug, Clone)]
pub struct MessageContext {
    /// Message direction for routing
    pub direction: MessageDirection,
    /// Session information
    pub session: SessionContext,
    /// Transport-specific metadata
    pub transport: TransportContext,
    /// Timing information
    pub timing: TimingContext,
    /// Extensible metadata
    pub extensions: Extensions,
}

/// Message flow direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDirection {
    /// Client initiated the message
    ClientToServer,
    /// Server initiated the message
    ServerToClient,
    /// Internal routing (e.g., interceptor-generated)
    Internal,
}

/// Session-level context
#[derive(Debug, Clone)]
pub struct SessionContext {
    /// Unique session identifier
    pub session_id: SessionId,
    /// MCP protocol version
    pub protocol_version: ProtocolVersion,
    /// Optional correlation for request/response
    pub correlation_id: Option<String>,
    /// Session state
    pub state: SessionState,
}

/// Transport-specific context
#[derive(Debug, Clone)]
pub struct TransportContext {
    /// Which transport type
    pub transport_type: TransportType,
    /// Transport-specific metadata
    pub metadata: TransportMetadata,
    /// Connection information
    pub connection: ConnectionInfo,
}

/// Transport-specific metadata variants
#[derive(Debug, Clone)]
pub enum TransportMetadata {
    /// Standard I/O transport
    Stdio {
        process_id: Option<u32>,
        pipe_name: Option<String>,
    },
    
    /// HTTP transport
    Http {
        headers: Arc<HeaderMap>,
        method: http::Method,
        uri: http::Uri,
        status: Option<http::StatusCode>,
    },
    
    /// Server-Sent Events
    Sse {
        event_id: Option<String>,
        event_type: Option<String>,
        retry_after: Option<Duration>,
        last_event_id: Option<String>,
        stream_id: StreamId,
    },
    
    /// Future: WebSocket
    WebSocket {
        frame_type: FrameType,
        is_final: bool,
        compression: bool,
    },
}

/// Timing information for performance tracking
#[derive(Debug, Clone, Copy)]
pub struct TimingContext {
    /// When the message was received/created
    pub created_at: Instant,
    /// When the message was sent (if applicable)
    pub sent_at: Option<Instant>,
    /// Processing start time
    pub processing_started: Option<Instant>,
    /// Processing completion time
    pub processing_completed: Option<Instant>,
}

/// Extensible metadata for future needs
#[derive(Debug, Clone, Default)]
pub struct Extensions {
    data: Arc<HashMap<String, Value>>,
}
```

### Compatibility Layer

```rust
// src/transport/compatibility.rs

/// Compatibility wrapper for gradual migration
impl From<TransportMessage> for MessageEnvelope {
    fn from(msg: TransportMessage) -> Self {
        MessageEnvelope {
            message: msg.into_protocol_message(),
            context: MessageContext::default_for_compatibility(),
        }
    }
}

impl MessageEnvelope {
    /// Extract the protocol message for legacy code
    pub fn into_transport_message(self) -> TransportMessage {
        self.message.into_legacy_format()
    }
    
    /// Get a reference that acts like TransportMessage
    pub fn as_transport_message(&self) -> TransportMessageRef<'_> {
        TransportMessageRef::from(&self.message)
    }
}

/// Temporary type alias during migration
#[deprecated(note = "Use ProtocolMessage or MessageEnvelope")]
pub type TransportMessage = ProtocolMessage;
```

### Builder Pattern for Context

```rust
// src/transport/envelope/builder.rs

impl MessageEnvelope {
    /// Create a new envelope with minimal context
    pub fn new(message: ProtocolMessage) -> Self {
        Self {
            message,
            context: MessageContext::minimal(),
        }
    }
    
    /// Builder for adding context
    pub fn with_session(mut self, session: SessionContext) -> Self {
        self.context.session = session;
        self
    }
    
    pub fn with_transport(mut self, transport: TransportContext) -> Self {
        self.context.transport = transport;
        self
    }
    
    pub fn with_direction(mut self, direction: MessageDirection) -> Self {
        self.context.direction = direction;
        self
    }
}

impl MessageContext {
    /// Create minimal context for compatibility
    pub fn minimal() -> Self {
        Self {
            direction: MessageDirection::ClientToServer,
            session: SessionContext::anonymous(),
            transport: TransportContext::unknown(),
            timing: TimingContext::now(),
            extensions: Extensions::default(),
        }
    }
}
```

### Transport Trait Extension

```rust
// src/transport/traits.rs

/// Extended transport trait with context support
#[async_trait]
pub trait TransportWithContext: Transport {
    /// Receive message with full context
    async fn receive_envelope(&mut self) -> TransportResult<MessageEnvelope>;
    
    /// Send message with specific context
    async fn send_envelope(&mut self, envelope: MessageEnvelope) -> TransportResult<()>;
    
    /// Get the current transport context
    fn current_context(&self) -> TransportContext;
}

/// Automatic implementation for existing transports
impl<T: Transport> TransportWithContext for T {
    default async fn receive_envelope(&mut self) -> TransportResult<MessageEnvelope> {
        let message = self.receive().await?;
        Ok(MessageEnvelope::new(message)
            .with_transport(self.current_context()))
    }
    
    default async fn send_envelope(&mut self, envelope: MessageEnvelope) -> TransportResult<()> {
        self.send(envelope.message).await
    }
    
    default fn current_context(&self) -> TransportContext {
        TransportContext::legacy(self.transport_type())
    }
}
```

## Performance Considerations

### Memory Optimization
- Use `Arc` for shared immutable data (headers)
- Use `Cow` for strings that are often static
- Lazy initialization of optional fields
- Small enum discriminants

### Zero-Cost Abstractions
- Inline small functions
- Use const generics where applicable
- Avoid unnecessary allocations
- Profile critical paths

### Benchmarks Needed
```rust
#[bench]
fn bench_envelope_creation(b: &mut Bencher) {
    b.iter(|| {
        MessageEnvelope::new(test_message())
            .with_session(test_session())
            .with_transport(test_transport())
    });
}

#[bench]
fn bench_legacy_conversion(b: &mut Bencher) {
    b.iter(|| {
        let envelope: MessageEnvelope = legacy_message.into();
        let _back: TransportMessage = envelope.into_transport_message();
    });
}
```

## Migration Strategy

### Phase 1: Parallel Types
1. Introduce new types without removing old ones
2. Add conversion methods
3. Update transport trait with default implementations

### Phase 2: Core Migration
1. Update transport implementations to use envelopes
2. Add context extraction in receive paths
3. Add context injection in send paths

### Phase 3: Proxy Migration
1. Update proxy to preserve context
2. Add context-aware routing
3. Implement context merging for forwarding

### Phase 4: Full Migration
1. Update all consumers to use envelopes
2. Deprecate TransportMessage
3. Remove compatibility layer

## Testing Plan

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_envelope_creation() {
        let envelope = MessageEnvelope::new(test_request())
            .with_direction(MessageDirection::ClientToServer);
        assert_eq!(envelope.context.direction, MessageDirection::ClientToServer);
    }
    
    #[test]
    fn test_legacy_compatibility() {
        let old_msg = create_legacy_message();
        let envelope: MessageEnvelope = old_msg.clone().into();
        let back = envelope.into_transport_message();
        assert_eq!(old_msg, back);
    }
    
    #[test]
    fn test_context_preservation() {
        let mut envelope = create_test_envelope();
        envelope = forward_through_proxy(envelope);
        assert_eq!(envelope.context.session.session_id, original_session_id);
    }
}
```

### Integration Tests
- End-to-end message flow with context
- Context preservation through proxy
- Performance regression tests
- Memory usage tests

## Deliverables

### 1. Type Definitions
**Location**: `shadowcat/src/transport/envelope.rs`
- Complete type definitions
- Comprehensive documentation
- Derive implementations

### 2. Compatibility Layer
**Location**: `shadowcat/src/transport/compatibility.rs`
- Conversion implementations
- Type aliases
- Migration helpers

### 3. Design Document
**Location**: `plans/transport-context-refactor/design/message-envelope-design.md`
- Design rationale
- Alternative approaches considered
- Performance analysis
- Migration timeline

## Success Criteria

- [ ] Types compile without warnings
- [ ] Zero-cost conversion for legacy code
- [ ] < 1% performance overhead
- [ ] All metadata requirements met
- [ ] Extensible for future transports
- [ ] Clear migration path
- [ ] Comprehensive documentation

## Open Questions

1. Should we use a type parameter for transport metadata instead of an enum?
2. How to handle partial context (e.g., no session yet)?
3. Should timing be part of core context or an extension?
4. How to efficiently merge contexts when forwarding?
5. Should we support context inheritance?

## Notes

- Consider using `pin-project` for async context propagation
- Look at `tower`'s approach to request extensions
- Review `hyper`'s handling of HTTP extensions
- Consider `tracing`'s span context as inspiration

## Related Tasks

- **Depends on**: A.0, A.1  
- **Next**: A.3 - Create Migration Strategy
- **Enables**: C.1 - Create MessageEnvelope Types (implementation)

---

**Task Owner**: _Unassigned_  
**Created**: 2025-08-08  
**Last Updated**: 2025-08-08