# SSE Integration Updates After Transport Context Refactor

## Overview

The Transport Context Refactor has significantly improved the foundation for SSE integration. The new `MessageEnvelope` system provides exactly what we need for proper SSE support without workarounds.

## Key Changes from Refactor

### 1. Type Name Changes
- `TransportMessage` → `ProtocolMessage`
- `Direction` → `MessageDirection`
- `Frame` → `MessageEnvelope` (complete replacement)

### 2. Transport Trait Signature
```rust
// OLD (from plans)
async fn send(&mut self, message: TransportMessage) -> TransportResult<()>;
async fn receive(&mut self) -> TransportResult<TransportMessage>;

// NEW (after refactor)
async fn send(&mut self, envelope: MessageEnvelope) -> TransportResult<()>;
async fn receive(&mut self) -> TransportResult<MessageEnvelope>;
```

### 3. SSE Context Support
The `TransportContext::Sse` variant already includes all needed fields:
```rust
Sse {
    event_type: Option<String>,    // SSE event type
    event_id: Option<String>,      // SSE event ID for resumption
    retry_ms: Option<u64>,         // Retry timing
    headers: HashMap<String, String>, // Including Last-Event-ID
}
```

## Updated Implementation Strategy

### Task F.3: Batch Handler (Current Task)
Update the implementation to use new types:

```rust
use serde_json::Value;
use crate::mcp::protocol::ProtocolVersion;
use crate::transport::{ProtocolMessage, MessageEnvelope, MessageContext};

pub struct BatchHandler {
    version: ProtocolVersion,
}

impl BatchHandler {
    pub fn split_if_batch(&self, value: Value) -> Vec<Value> {
        match value {
            Value::Array(arr) if self.version.supports_batching() => arr,
            single => vec![single],
        }
    }
    
    // When combining, we're working with ProtocolMessages
    pub fn combine_messages(&self, messages: Vec<ProtocolMessage>) -> Vec<ProtocolMessage> {
        if self.version.supports_batching() && messages.len() > 1 {
            // Return as-is for batching at JSON level
            messages
        } else {
            messages
        }
    }
}
```

### Task 1.2: SSE Transport Wrapper
Update to use MessageEnvelope:

```rust
use crate::transport::{Transport, MessageEnvelope, MessageContext, TransportContext};
use crate::transport::{ProtocolMessage, SessionId, MessageDirection};

pub struct SseTransport {
    session_id: SessionId,
    // ... other fields
}

#[async_trait]
impl Transport for SseTransport {
    async fn send(&mut self, envelope: MessageEnvelope) -> TransportResult<()> {
        // Extract SSE context if present
        if let TransportContext::Sse { event_id, event_type, .. } = &envelope.context.transport {
            // Use SSE-specific metadata
        }
        
        // Send the protocol message
        let json = protocol_message_to_json(&envelope.message)?;
        self.send_sse_event(json).await
    }
    
    async fn receive(&mut self) -> TransportResult<MessageEnvelope> {
        let event = self.receive_sse_event().await?;
        
        // Parse the protocol message
        let protocol_message = parse_json_to_protocol_message(&event.data)?;
        
        // Build proper context
        let mut sse_context = TransportContext::sse();
        if let TransportContext::Sse { event_id, event_type, retry_ms, .. } = &mut sse_context {
            *event_id = Some(event.id.clone());
            *event_type = event.event_type.clone();
            *retry_ms = event.retry;
        }
        
        let context = MessageContext::new(
            self.session_id.clone(),
            MessageDirection::ServerToClient,
            sse_context,
        );
        
        Ok(MessageEnvelope::new(protocol_message, context))
    }
}
```

## Benefits of the Refactor for SSE

1. **No More Workarounds**: The context is built-in, no need for separate tracking
2. **Clean Abstractions**: Transport metadata properly separated from protocol messages
3. **Session Integration**: Session ID is part of MessageContext, no manual tracking
4. **Direction Clarity**: MessageDirection in context, no ambiguity about notification direction
5. **SSE Metadata**: All SSE-specific fields available in TransportContext::Sse

## Migration Guide for Existing SSE Code

If any SSE code was written during the previous attempts:

1. Replace `TransportMessage` with `ProtocolMessage`
2. Wrap messages in `MessageEnvelope` before sending through transports
3. Extract messages from envelopes when receiving
4. Use `TransportContext::Sse` for SSE-specific metadata
5. Use `MessageContext` for session and direction information

## Next Steps

1. **Update F.3 (Batch Handler)** to use `ProtocolMessage`
2. **Continue with F.4 and F.5** foundation tasks
3. **Begin Phase 1 SSE Integration** with the new envelope system
4. **Leverage TransportContext::Sse** throughout the implementation

## Timeline Impact

The refactor actually **accelerates** SSE integration:
- No need to build context tracking (saved ~4 hours)
- No need for direction workarounds (saved ~2 hours)
- Cleaner integration with existing transports (saved ~3 hours)

**Estimated time savings: 9 hours**

The SSE integration should now be cleaner and faster to implement!