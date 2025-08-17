# Batch Handler Integration Requirements

## Overview

The BatchHandler (F.3) has been successfully implemented but needs integration with the transport layer to be functional. This document outlines the required integration work discovered during code review.

## Critical Integration Gap

**Issue**: The `BatchHandler` is not used anywhere in the codebase except tests. The transport implementations (stdio, HTTP, SSE) don't handle batch messages, which means MCP 2025-03-26 batch support is currently broken.

### Current State
- `StdioTransport::parse_message()` only handles single messages
- Batch messages (JSON arrays) would fail with "Invalid JSON-RPC" error
- No transport currently uses the BatchHandler

### Required Integration Points

## 1. Transport Layer Integration (Priority: CRITICAL)

### StdioTransport Updates

```rust
// src/transport/stdio.rs

use crate::mcp::batch::{BatchHandler, BatchError};
use crate::mcp::protocol::ProtocolVersion;

impl StdioTransport {
    fn parse_message(
        &self,
        line: &str,
        direction: MessageDirection,
    ) -> TransportResult<Vec<MessageEnvelope>> {  // Changed to return Vec
        let json_value: Value = serde_json::from_str(line)
            .map_err(|e| TransportError::ProtocolError(format!("Invalid JSON: {e}")))?;
        
        // Use BatchHandler to split if needed
        let batch_handler = BatchHandler::new(self.protocol_version);
        let messages = batch_handler.split_if_batch(json_value);
        
        let mut envelopes = Vec::new();
        for msg_value in messages {
            // Parse each message individually
            let envelope = self.parse_single_message(msg_value, direction)?;
            envelopes.push(envelope);
        }
        
        Ok(envelopes)
    }
}
```

### HTTP/SSE Transport Updates

Similar changes needed for:
- `src/transport/http_mcp.rs`
- `src/transport/sse/mod.rs` (when implemented)

## 2. MessageEnvelope Integration

### Add Adapter Methods to BatchHandler

```rust
// src/mcp/batch.rs

impl BatchHandler {
    /// Convert MessageEnvelopes to batch format for transmission
    pub fn batch_envelopes(&self, envelopes: Vec<MessageEnvelope>) -> Result<Value, BatchError> {
        let messages: Vec<Value> = envelopes
            .into_iter()
            .map(|e| e.message.to_value())
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(self.combine_if_needed(messages))
    }
    
    /// Split received batch into MessageEnvelopes with shared context
    pub fn unbatch_to_envelopes(
        &self, 
        value: Value,
        base_context: MessageContext,
    ) -> Result<Vec<MessageEnvelope>, BatchError> {
        let values = self.split_if_batch(value);
        let mut envelopes = Vec::new();
        
        for (idx, val) in values.into_iter().enumerate() {
            let msg = ProtocolMessage::from_value(val)?;
            let mut context = base_context.clone();
            context.set_batch_index(Some(idx));
            envelopes.push(MessageEnvelope::new(msg, context));
        }
        
        Ok(envelopes)
    }
}
```

## 3. Performance Limits

### Add Configuration

```rust
// src/mcp/batch.rs

pub struct BatchConfig {
    pub max_batch_size: usize,     // Default: 100
    pub max_batch_bytes: usize,    // Default: 1MB
    pub batch_timeout_ms: u64,     // Default: 100ms
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 100,
            max_batch_bytes: 1_048_576,  // 1MB
            batch_timeout_ms: 100,
        }
    }
}

pub struct BatchHandler {
    version: ProtocolVersion,
    config: BatchConfig,
}
```

## 4. SSE-Specific Enhancements (Phase 1)

```rust
// src/mcp/batch.rs

impl BatchHandler {
    /// Prepare messages for SSE transmission
    pub fn prepare_for_sse(&self, messages: Vec<MinimalMessage>) -> Vec<SseEvent> {
        // Group messages to respect SSE event boundaries
        // Add event IDs for resumption support
        // Split large batches to avoid SSE timeouts
    }
    
    /// Handle SSE reconnection with Last-Event-ID
    pub fn filter_after_event_id(
        &self, 
        event_id: &str, 
        messages: Vec<MinimalMessage>
    ) -> Vec<MinimalMessage> {
        // Return only messages after the given event ID
    }
}
```

## 5. Metrics Collection

```rust
// src/mcp/batch.rs

use std::sync::Arc;
use parking_lot::RwLock;

pub struct BatchStats {
    pub total_batches: u64,
    pub total_messages: u64,
    pub avg_batch_size: f64,
    pub max_batch_size: usize,
    pub batch_errors: u64,
}

pub struct BatchHandler {
    version: ProtocolVersion,
    config: BatchConfig,
    stats: Arc<RwLock<BatchStats>>,
}
```

## Implementation Priority

1. **Immediate (Before F.4)**:
   - [ ] Fix StdioTransport to handle batch messages
   - [ ] Add basic MessageEnvelope integration
   - [ ] Set default batch size limits

2. **Phase 1 (During SSE Transport)**:
   - [ ] Add SSE-specific methods
   - [ ] Implement event ID tracking

3. **Phase 2 (During Reverse Proxy)**:
   - [ ] Add batch routing logic
   - [ ] Implement load distribution

4. **Future**:
   - [ ] Add comprehensive metrics
   - [ ] Implement adaptive batching

## Testing Requirements

### Integration Tests Needed

```rust
#[tokio::test]
async fn test_stdio_batch_handling() {
    // Test that StdioTransport correctly handles batch messages
}

#[tokio::test]
async fn test_batch_size_limits() {
    // Test that oversized batches are rejected
}

#[tokio::test]
async fn test_batch_envelope_conversion() {
    // Test MessageEnvelope <-> batch conversion
}
```

## Risk Assessment

**Current Risk**: HIGH
- Batch messages will fail in production
- MCP 2025-03-26 protocol is not fully supported
- This blocks Phase 1 SSE implementation

**Mitigation**: 
- Integrate BatchHandler into transports immediately
- Add integration tests before Phase 1
- Consider making this part of F.4 task scope

## Recommendation

Before proceeding to F.4 (Event ID Generator), we should:
1. Create a quick integration task (F.3.1) to wire up BatchHandler
2. Add at least basic transport integration
3. Write integration tests to verify batch handling works end-to-end

This ensures our foundation is solid before building more components on top.