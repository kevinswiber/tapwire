# MessageEnvelope Design Document

## Executive Summary

This document presents the concrete design for the `MessageEnvelope` structure that will properly separate protocol concerns from transport metadata in Shadowcat. This design addresses the fundamental issue where `TransportMessage` lacks direction information and transport context, which is critical for SSE proxy integration and proper bidirectional notification routing.

## Design Goals

1. **Separation of Concerns**: Clear distinction between MCP protocol messages and transport metadata
2. **Zero-Cost Migration**: Existing code can continue working during gradual migration
3. **Performance**: < 1% overhead with careful memory management
4. **Extensibility**: Support for future transports (WebSocket, gRPC)
5. **Type Safety**: Compile-time guarantees for context requirements

## Core Type Definitions

### MessageEnvelope - The Complete Message

```rust
// src/transport/envelope.rs

use std::sync::Arc;
use std::borrow::Cow;
use serde::{Serialize, Deserialize};
use serde_json::Value;

/// The complete message with all context needed for routing and processing
#[derive(Debug, Clone)]
pub struct MessageEnvelope {
    /// The protocol-level message (MCP/JSON-RPC)
    pub message: ProtocolMessage,
    
    /// Complete context including transport, session, and direction
    pub context: MessageContext,
}

impl MessageEnvelope {
    /// Create envelope with minimal context (for compatibility)
    pub fn new(message: ProtocolMessage) -> Self {
        Self {
            message,
            context: MessageContext::minimal(),
        }
    }
    
    /// Builder pattern for adding context
    pub fn with_direction(mut self, direction: MessageDirection) -> Self {
        self.context.direction = direction;
        self
    }
    
    pub fn with_session(mut self, session: SessionContext) -> Self {
        self.context.session = session;
        self
    }
    
    pub fn with_transport(mut self, transport: TransportContext) -> Self {
        self.context.transport = transport;
        self
    }
}
```

### ProtocolMessage - The MCP Layer

```rust
/// Protocol-level message representing MCP semantics
/// This is essentially the current TransportMessage, renamed for clarity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "jsonrpc", rename = "2.0")]
pub enum ProtocolMessage {
    #[serde(rename = "request")]
    Request {
        id: MessageId,
        method: String,
        params: Value,
    },
    
    #[serde(rename = "response")]
    Response {
        id: MessageId,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<ErrorObject>,
    },
    
    #[serde(rename = "notification")]
    Notification {
        method: String,
        params: Value,
    },
}

/// Message ID can be string or number per JSON-RPC spec
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum MessageId {
    String(String),
    Number(i64),
}

/// JSON-RPC error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorObject {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}
```

### MessageContext - The Metadata

```rust
/// Complete context for message handling and routing
#[derive(Debug, Clone)]
pub struct MessageContext {
    /// CRITICAL: Message direction for proper routing
    /// This solves the core notification routing problem
    pub direction: MessageDirection,
    
    /// Session information for correlation
    pub session: SessionContext,
    
    /// Transport-specific metadata (headers, SSE events, etc.)
    pub transport: TransportContext,
    
    /// Timing information for performance tracking
    pub timing: TimingContext,
}

impl MessageContext {
    /// Create minimal context for compatibility during migration
    pub fn minimal() -> Self {
        Self {
            direction: MessageDirection::Unknown,
            session: SessionContext::anonymous(),
            transport: TransportContext::unknown(),
            timing: TimingContext::now(),
        }
    }
    
    /// Check if context has sufficient information for routing
    pub fn is_complete(&self) -> bool {
        self.direction != MessageDirection::Unknown &&
        self.session.session_id.is_some()
    }
}
```

### MessageDirection - Solving the Core Problem

```rust
/// Message flow direction - CRITICAL for notification routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageDirection {
    /// Client initiated the message (request or client notification)
    ClientToServer,
    
    /// Server initiated the message (response or server notification)
    ServerToClient,
    
    /// Internal routing (e.g., interceptor-generated)
    Internal,
    
    /// Unknown direction (compatibility mode only)
    #[serde(other)]
    Unknown,
}

impl MessageDirection {
    /// Determine if this message should be forwarded to server
    pub fn should_forward_to_server(&self) -> bool {
        matches!(self, Self::ClientToServer)
    }
    
    /// Determine if this message should be sent to client
    pub fn should_send_to_client(&self) -> bool {
        matches!(self, Self::ServerToClient)
    }
    
    /// Invert direction for response routing
    pub fn reverse(&self) -> Self {
        match self {
            Self::ClientToServer => Self::ServerToClient,
            Self::ServerToClient => Self::ClientToServer,
            other => *other,
        }
    }
}
```

### SessionContext - MCP Session Information

```rust
/// Session-level context for MCP protocol
#[derive(Debug, Clone)]
pub struct SessionContext {
    /// Unique session identifier (None for pre-initialization)
    pub session_id: Option<SessionId>,
    
    /// MCP protocol version
    pub protocol_version: ProtocolVersion,
    
    /// Correlation ID for request/response matching
    pub correlation_id: Option<String>,
    
    /// Current session state
    pub state: SessionState,
}

impl SessionContext {
    /// Create anonymous session for pre-initialization messages
    pub fn anonymous() -> Self {
        Self {
            session_id: None,
            protocol_version: ProtocolVersion::default(),
            correlation_id: None,
            state: SessionState::Uninitialized,
        }
    }
    
    /// Create session context with ID
    pub fn with_id(session_id: SessionId) -> Self {
        Self {
            session_id: Some(session_id),
            protocol_version: ProtocolVersion::default(),
            correlation_id: None,
            state: SessionState::Active,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Uninitialized,
    Initializing,
    Active,
    Closing,
    Closed,
}

/// MCP protocol version
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolVersion(pub Cow<'static, str>);

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self(Cow::Borrowed("2025-11-05"))
    }
}

/// Unique session identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(pub String);
```

### TransportContext - Transport Layer Metadata

```rust
/// Transport-specific context and metadata
#[derive(Debug, Clone)]
pub struct TransportContext {
    /// Which transport type this message came from/goes to
    pub transport_type: TransportType,
    
    /// Transport-specific metadata
    pub metadata: TransportMetadata,
    
    /// Connection information
    pub connection: Option<ConnectionInfo>,
}

impl TransportContext {
    /// Create unknown context for compatibility
    pub fn unknown() -> Self {
        Self {
            transport_type: TransportType::Unknown,
            metadata: TransportMetadata::None,
            connection: None,
        }
    }
    
    /// Create stdio transport context
    pub fn stdio() -> Self {
        Self {
            transport_type: TransportType::Stdio,
            metadata: TransportMetadata::Stdio {
                process_id: std::process::id(),
            },
            connection: None,
        }
    }
    
    /// Create HTTP transport context
    pub fn http(headers: Arc<HeaderMap>, method: Method, uri: Uri) -> Self {
        Self {
            transport_type: TransportType::Http,
            metadata: TransportMetadata::Http {
                headers,
                method,
                uri,
                status: None,
            },
            connection: None,
        }
    }
}

/// Transport type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    Stdio,
    Http,
    Sse,
    WebSocket,
    Unknown,
}

/// Transport-specific metadata variants
#[derive(Debug, Clone)]
pub enum TransportMetadata {
    /// No metadata (compatibility mode)
    None,
    
    /// Standard I/O transport
    Stdio {
        process_id: u32,
    },
    
    /// HTTP transport (including MCP-over-HTTP)
    Http {
        /// HTTP headers (shared reference for efficiency)
        headers: Arc<HeaderMap>,
        /// HTTP method
        method: Method,
        /// Request URI
        uri: Uri,
        /// Response status (for responses)
        status: Option<StatusCode>,
    },
    
    /// Server-Sent Events
    Sse {
        /// SSE event ID for resumption
        event_id: Option<String>,
        /// SSE event type
        event_type: Option<String>,
        /// Retry-After hint from server
        retry_after: Option<Duration>,
        /// Last received event ID (for reconnection)
        last_event_id: Option<String>,
        /// Stream identifier
        stream_id: StreamId,
    },
    
    /// Future: WebSocket
    WebSocket {
        /// Frame type (text/binary/close/ping/pong)
        frame_type: WsFrameType,
        /// Is this the final fragment
        is_final: bool,
        /// Compression enabled
        compression: bool,
    },
}

/// Connection information
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Remote address
    pub remote_addr: Option<SocketAddr>,
    /// Local address
    pub local_addr: Option<SocketAddr>,
    /// Connection ID for correlation
    pub connection_id: String,
    /// TLS information if applicable
    pub tls_info: Option<TlsInfo>,
}
```

### TimingContext - Performance Tracking

```rust
use std::time::{Instant, SystemTime};

/// Timing information for performance analysis
#[derive(Debug, Clone, Copy)]
pub struct TimingContext {
    /// When the message was created/received
    pub created_at: Instant,
    
    /// System time for absolute timestamps
    pub system_time: SystemTime,
    
    /// When processing started
    pub processing_started: Option<Instant>,
    
    /// When processing completed
    pub processing_completed: Option<Instant>,
    
    /// When message was sent
    pub sent_at: Option<Instant>,
}

impl TimingContext {
    /// Create new timing context
    pub fn now() -> Self {
        Self {
            created_at: Instant::now(),
            system_time: SystemTime::now(),
            processing_started: None,
            processing_completed: None,
            sent_at: None,
        }
    }
    
    /// Mark processing as started
    pub fn start_processing(&mut self) {
        self.processing_started = Some(Instant::now());
    }
    
    /// Mark processing as completed
    pub fn complete_processing(&mut self) {
        self.processing_completed = Some(Instant::now());
    }
    
    /// Get total processing duration
    pub fn processing_duration(&self) -> Option<Duration> {
        match (self.processing_started, self.processing_completed) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}
```

## Compatibility Layer

Critical for zero-downtime migration:

```rust
// src/transport/compatibility.rs

/// Type alias for gradual migration
/// This allows existing code to continue compiling
pub type TransportMessage = ProtocolMessage;

/// Extension trait for compatibility
pub trait TransportMessageExt {
    /// Convert to new envelope format
    fn into_envelope(self) -> MessageEnvelope;
    
    /// Create with assumed direction (for migration)
    fn into_envelope_with_direction(self, direction: MessageDirection) -> MessageEnvelope;
}

impl TransportMessageExt for TransportMessage {
    fn into_envelope(self) -> MessageEnvelope {
        MessageEnvelope::new(self)
    }
    
    fn into_envelope_with_direction(self, direction: MessageDirection) -> MessageEnvelope {
        MessageEnvelope::new(self).with_direction(direction)
    }
}

/// Compatibility conversion from Frame
impl From<Frame> for MessageEnvelope {
    fn from(frame: Frame) -> Self {
        MessageEnvelope::new(frame.message)
            .with_direction(frame.direction.into())
            .with_session(SessionContext::with_id(frame.session_id))
    }
}

/// Convert old Direction to new MessageDirection
impl From<Direction> for MessageDirection {
    fn from(old: Direction) -> Self {
        match old {
            Direction::ClientToServer => MessageDirection::ClientToServer,
            Direction::ServerToClient => MessageDirection::ServerToClient,
        }
    }
}
```

## Transport Trait Evolution

```rust
// src/transport/traits.rs

/// Extended transport trait with context awareness
#[async_trait]
pub trait TransportWithContext: Transport {
    /// Receive message with full context
    async fn receive_envelope(&mut self) -> TransportResult<MessageEnvelope>;
    
    /// Send message with preserved context
    async fn send_envelope(&mut self, envelope: MessageEnvelope) -> TransportResult<()>;
    
    /// Get current transport context
    fn current_context(&self) -> TransportContext;
}

/// Blanket implementation for existing transports
#[async_trait]
impl<T: Transport> TransportWithContext for T {
    default async fn receive_envelope(&mut self) -> TransportResult<MessageEnvelope> {
        let message = self.receive().await?;
        Ok(MessageEnvelope::new(message)
            .with_transport(self.current_context()))
    }
    
    default async fn send_envelope(&mut self, envelope: MessageEnvelope) -> TransportResult<()> {
        // During migration, just send the protocol message
        self.send(envelope.message).await
    }
    
    default fn current_context(&self) -> TransportContext {
        TransportContext::unknown()
    }
}
```

## Memory and Performance Optimizations

### Arc Usage for Shared Data
- HTTP headers are wrapped in `Arc<HeaderMap>` to avoid cloning
- Large metadata structures use `Arc` for cheap clones
- Extensions use `Arc<HashMap>` internally

### Cow for String Data
- Static strings use `Cow::Borrowed`
- Dynamic strings use `Cow::Owned` only when necessary
- Protocol version uses `Cow<'static, str>` for common versions

### Option for Optional Fields
- Many fields are `Option<T>` to avoid allocation when not needed
- Zero overhead for minimal contexts

### Copy Types Where Possible
- `MessageDirection`, `TransportType`, `SessionState` are `Copy`
- `TimingContext` uses `Instant` and `SystemTime` which are small

## Migration Strategy

### Phase 1: Parallel Introduction (Week 1)
1. Add new types without removing old ones
2. Implement compatibility layer
3. Add conversion traits
4. Update transport trait with default implementations

### Phase 2: Transport Updates (Week 1-2)
1. Update StdioTransport to use envelopes internally
2. Update HttpTransport to extract headers into context
3. Update HttpMcpTransport to preserve MCP headers
4. Add SSE context extraction when implemented

### Phase 3: Core Component Migration (Week 2)
1. Update SessionManager to use envelopes
2. Migrate Frame to use MessageEnvelope
3. Update proxy to preserve context
4. Convert interceptors to context-aware

### Phase 4: Cleanup (Week 3)
1. Remove workarounds identified in analysis
2. Deprecate old types
3. Update all tests
4. Performance validation

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_direction_routing() {
        let envelope = MessageEnvelope::new(create_notification())
            .with_direction(MessageDirection::ServerToClient);
        
        assert!(!envelope.context.direction.should_forward_to_server());
        assert!(envelope.context.direction.should_send_to_client());
    }
    
    #[test]
    fn test_context_preservation() {
        let original = create_test_envelope();
        let cloned = original.clone();
        
        assert_eq!(
            original.context.session.session_id,
            cloned.context.session.session_id
        );
    }
    
    #[test]
    fn test_compatibility_conversion() {
        let old_msg = create_legacy_message();
        let envelope = old_msg.clone().into_envelope();
        
        assert_eq!(envelope.message, old_msg);
        assert_eq!(envelope.context.direction, MessageDirection::Unknown);
    }
}
```

### Integration Tests
- End-to-end message flow with context preservation
- Proxy forwarding with context
- Session correlation with envelopes
- Performance benchmarks

### Benchmark Suite
```rust
#[bench]
fn bench_envelope_creation(b: &mut Bencher) {
    let msg = create_test_message();
    b.iter(|| {
        MessageEnvelope::new(msg.clone())
            .with_direction(MessageDirection::ClientToServer)
            .with_session(SessionContext::with_id(SessionId("test".into())))
    });
}

#[bench]
fn bench_context_clone(b: &mut Bencher) {
    let envelope = create_full_envelope();
    b.iter(|| {
        envelope.clone()
    });
}
```

## Benefits Over Current Architecture

### Solves Core Problems
1. **Notification Direction**: Explicitly tracked in every message
2. **Transport Metadata**: Preserved through entire pipeline
3. **Session Context**: Travels with message, no lookups needed
4. **SSE Support**: Event IDs and retry information preserved

### Eliminates Workarounds
- No more Frame wrapping for direction
- No more session extraction heuristics
- No more method-based routing guesses
- No more separate header handling
- No more context reconstruction

### Performance Improvements
- Fewer hash map lookups (context travels with message)
- Reduced locking (less shared state)
- Better cache locality (related data together)
- Predictable memory usage

### Developer Experience
- Type-safe context requirements
- Clear separation of concerns
- Easier testing with explicit context
- Better debugging with full context visibility

## Open Design Questions Resolved

1. **Q: Should we rename TransportMessage?**
   **A:** Yes, to `ProtocolMessage` for clarity, with type alias for compatibility

2. **Q: How to handle direction for requests/responses?**
   **A:** Requests always have explicit direction, responses inherit reversed direction

3. **Q: Should context be optional?**
   **A:** Yes, for compatibility, but `is_complete()` method indicates readiness

4. **Q: How to minimize allocations?**
   **A:** Use `Arc`, `Cow`, and `Option` strategically as shown

5. **Q: Transport-specific metadata as enum or trait?**
   **A:** Enum for now, can add extension mechanism later if needed

## Implementation Priority

1. **Critical Path** (Must have for SSE):
   - MessageEnvelope type
   - MessageDirection enum
   - TransportContext with SSE variant
   - Compatibility layer

2. **Important** (Improves system):
   - SessionContext
   - TimingContext
   - Connection info

3. **Nice to Have** (Future-proofing):
   - WebSocket metadata
   - TLS information
   - Extensions system

## Conclusion

This design provides a clean, performant solution to the context problem while maintaining backward compatibility. It eliminates 17 identified workarounds, enables proper SSE integration, and provides a foundation for future transport types. The gradual migration path ensures zero downtime and allows for incremental adoption.

The key innovation is making message direction and transport context explicit first-class citizens of the message envelope, solving the fundamental routing problems that have plagued the codebase.