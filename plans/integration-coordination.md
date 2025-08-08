# Integration Coordination: SSE Proxy ↔ MCP Message Handling

## Overview

This document identifies synergies and dependencies between the SSE Proxy Integration and MCP Message Handling initiatives, ensuring both efforts complement each other.

## Key Integration Points

### 1. Message Parser Placement

**Opportunity**: The MCP message parser (Phase 1 of MCP Message Handling) should be built into the SSE transport wrapper from the start.

**In SSE Proxy Integration (Task 1.2)**:
```rust
// Instead of just converting to TransportMessage
impl SseTransport {
    // Add MCP parser integration
    mcp_parser: McpParser,  // From MCP message handling
    
    async fn process_sse_event(&mut self, event: SseEvent) -> Result<ProcessedMessage> {
        // Parse as MCP message first
        let mcp_message = self.mcp_parser.parse(&event.data)?;
        
        // Store for correlation and interception
        self.message_store.record(mcp_message.clone());
        
        // Convert to TransportMessage for compatibility
        let transport_msg = self.convert_to_transport(mcp_message);
        
        Ok(ProcessedMessage {
            transport: transport_msg,
            mcp: Some(mcp_message),
            correlation_hint: extract_id(&mcp_message),
        })
    }
}
```

**Benefit**: SSE transport immediately becomes MCP-aware, enabling correlation and intelligent processing.

### 2. Batch Handling Coordination

**Opportunity**: The batch handling logic for MCP 2025-03-26 should be shared between both initiatives.

**Shared Component** (`src/mcp/batch_handler.rs`):
```rust
pub struct BatchHandler {
    version: ProtocolVersion,
}

impl BatchHandler {
    // Used by both SSE transport and MCP message handler
    pub fn split_batch(&self, message: McpMessage) -> Vec<JsonRpcMessage> {
        match (self.version, message) {
            (ProtocolVersion::V2025_03_26, McpMessage::Batch(msgs)) => msgs,
            (_, McpMessage::Single(msg)) => vec![msg],
            _ => vec![],
        }
    }
    
    pub fn combine_responses(&self, responses: Vec<JsonRpcMessage>) -> McpMessage {
        match self.version {
            ProtocolVersion::V2025_03_26 if responses.len() > 1 => {
                McpMessage::Batch(responses)
            }
            _ => McpMessage::Single(responses.into_iter().next().unwrap())
        }
    }
}
```

### 3. Session Context Sharing

**Opportunity**: The SSE session integration (Task 1.4) should provide hooks for MCP message handling.

**In SSE Session Integration**:
```rust
pub struct SseSessionState {
    // Existing fields...
    
    // Add MCP message handling hooks
    pub message_correlator: Option<Arc<MessageCorrelator>>,
    pub message_interceptor: Option<Arc<McpInterceptor>>,
    pub message_recorder: Option<Arc<McpRecorder>>,
}

impl SseSessionState {
    pub async fn process_message(&mut self, message: McpMessage) -> Result<McpMessage> {
        // Correlate if correlator present
        if let Some(correlator) = &self.message_correlator {
            correlator.track(&message, self.session_id.clone()).await;
        }
        
        // Intercept if interceptor present
        let processed = if let Some(interceptor) = &self.message_interceptor {
            interceptor.process(message).await?
        } else {
            message
        };
        
        // Record if recorder present
        if let Some(recorder) = &self.message_recorder {
            recorder.record(processed.clone(), MessageContext {
                session_id: self.session_id.clone(),
                transport: TransportType::Sse,
                ..Default::default()
            }).await?;
        }
        
        Ok(processed)
    }
}
```

### 4. Streamable HTTP Endpoint Enhancement

**Opportunity**: The `/mcp` endpoint (Task 2.1) should prepare for MCP-aware processing.

**In Dual-Method Endpoint**:
```rust
async fn handle_post(
    headers: HeaderMap,
    state: McpEndpointState,
    body: Option<Json<Value>>,
) -> Result<Response, McpError> {
    let protocol_version = extract_protocol_version(&headers)?;
    
    // Parse as MCP message early
    let mcp_message = parse_mcp_message(&body.0, protocol_version)?;
    
    // Store for future MCP-aware features
    state.message_processor.prepare(mcp_message.clone()).await;
    
    // Continue with existing logic but with MCP context
    match determine_message_types(&mcp_message) {
        // ... existing logic
    }
}
```

### 5. SSE Event ID Strategy

**Opportunity**: Align SSE event IDs with MCP correlation IDs for better tracking.

**Shared Event ID Generator**:
```rust
pub struct UnifiedEventIdGenerator {
    node_id: String,
    counter: AtomicU64,
}

impl UnifiedEventIdGenerator {
    // Generate ID that works for both SSE and MCP correlation
    pub fn generate(&self, session_id: &str, message_id: Option<&JsonRpcId>) -> String {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        
        match message_id {
            Some(id) => {
                // Include JSON-RPC ID for correlation
                format!("{}-{}-{}-{}", session_id, self.node_id, id, count)
            }
            None => {
                // No JSON-RPC ID (notification)
                format!("{}-{}-notif-{}", session_id, self.node_id, count)
            }
        }
    }
    
    pub fn extract_correlation(&self, event_id: &str) -> Option<String> {
        // Extract the JSON-RPC ID portion if present
        let parts: Vec<&str> = event_id.split('-').collect();
        if parts.len() >= 3 && parts[2] != "notif" {
            Some(parts[2].to_string())
        } else {
            None
        }
    }
}
```

## Implementation Phases Coordination

### Phase A: Foundation (Weeks 1-2)
**SSE Proxy Integration**:
- Task 1.1: CLI options ✓
- Task 1.2: SSE Transport wrapper (enhance with MCP parser hooks)

**MCP Message Handling**:
- Phase 1: MCP Parser (build concurrently)
- Share batch handling logic

### Phase B: Integration (Weeks 3-4)
**SSE Proxy Integration**:
- Task 2.1: Dual-method endpoint (add MCP awareness)
- Task 2.2: SSE Response handler (use shared event ID generator)

**MCP Message Handling**:
- Phase 2: Correlation engine (integrate with SSE sessions)
- Share session context

### Phase C: Enhancement (Weeks 5-6)
**Both**:
- Integrate MCP interceptor with SSE streams
- Add recording hooks to SSE transport
- Test end-to-end MCP over SSE

## Shared Components to Build

### 1. Protocol Version Manager
```rust
// src/mcp/protocol.rs
pub struct ProtocolVersionManager {
    pub version: ProtocolVersion,
}

impl ProtocolVersionManager {
    pub fn supports_batching(&self) -> bool {
        matches!(self.version, ProtocolVersion::V2025_03_26)
    }
    
    pub fn default_version() -> ProtocolVersion {
        ProtocolVersion::V2025_03_26  // For backwards compatibility
    }
    
    pub fn from_header(header: Option<&str>) -> ProtocolVersion {
        match header {
            Some("2025-06-18") => ProtocolVersion::V2025_06_18,
            Some("2025-03-26") => ProtocolVersion::V2025_03_26,
            _ => Self::default_version(),
        }
    }
}
```

### 2. Message Context
```rust
// src/mcp/context.rs
#[derive(Debug, Clone)]
pub struct McpMessageContext {
    pub session_id: SessionId,
    pub mcp_session_id: Option<String>,
    pub protocol_version: ProtocolVersion,
    pub transport: TransportType,
    pub direction: MessageDirection,
    pub correlation_id: Option<CorrelationId>,
    pub timestamp: Instant,
    pub intercepted: bool,
    pub recorded: bool,
}

impl McpMessageContext {
    pub fn from_sse(session: &SseSessionState) -> Self {
        Self {
            session_id: session.session_id.clone(),
            mcp_session_id: session.mcp_session_id.clone(),
            protocol_version: ProtocolVersion::from_str(&session.protocol_version),
            transport: TransportType::Sse,
            direction: MessageDirection::Inbound,
            correlation_id: None,
            timestamp: Instant::now(),
            intercepted: false,
            recorded: false,
        }
    }
}
```

### 3. Early MCP Parser
```rust
// src/mcp/early_parser.rs
// Minimal parser that can be used immediately by SSE integration
pub struct EarlyMcpParser;

impl EarlyMcpParser {
    pub fn parse_minimal(&self, data: &str) -> Result<MinimalMcpInfo> {
        let value: Value = serde_json::from_str(data)?;
        
        Ok(MinimalMcpInfo {
            is_batch: value.is_array(),
            message_count: if value.is_array() { 
                value.as_array().map(|a| a.len()).unwrap_or(0)
            } else { 
                1 
            },
            has_id: Self::extract_any_id(&value).is_some(),
            method: Self::extract_method(&value),
        })
    }
}

pub struct MinimalMcpInfo {
    pub is_batch: bool,
    pub message_count: usize,
    pub has_id: bool,
    pub method: Option<String>,
}
```

## Benefits of Coordination

### For SSE Proxy Integration:
1. **Immediate MCP awareness** - Parse and understand messages from day one
2. **Better debugging** - Correlation IDs help track request/response pairs
3. **Future-proof** - Hooks ready for interceptor/recorder integration
4. **Shared testing** - Reuse MCP message test fixtures

### For MCP Message Handling:
1. **Real transport testing** - SSE provides immediate real-world transport
2. **Session context** - Leverage SSE session management
3. **Early validation** - Test parser with actual SSE traffic
4. **Performance baseline** - Measure overhead with real SSE streams

## Recommended Changes to Task Files

### Task 1.2 (SSE Transport Wrapper)
Add to implementation:
- Import minimal MCP parser
- Add correlation ID extraction
- Prepare message context for future MCP handling
- Use shared batch handler

### Task 2.1 (Dual-Method Endpoint)
Add to implementation:
- Early MCP message parsing
- Store parsed message in request context
- Prepare for MCP-aware interceptors

### Task 2.2 (SSE Response Handler)
Add to implementation:
- Use unified event ID generator
- Include correlation hints in SSE events
- Prepare for response validation

## Testing Strategy

### Shared Test Fixtures
```rust
// tests/fixtures/mcp_messages.rs
pub mod mcp_test_messages {
    pub fn request_2025_06_18() -> Value { /* ... */ }
    pub fn request_2025_03_26() -> Value { /* ... */ }
    pub fn batch_request_2025_03_26() -> Value { /* ... */ }
    pub fn response_with_error() -> Value { /* ... */ }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_sse_with_mcp_parsing() {
    // Test that SSE transport correctly parses MCP messages
    let transport = SseTransport::builder()
        .url("http://test".parse().unwrap())
        .enable_mcp_parsing(true)
        .build()
        .await
        .unwrap();
    
    // Send MCP message via SSE
    let event = SseEvent::new(mcp_test_messages::request_2025_06_18());
    let processed = transport.process_event(event).await.unwrap();
    
    assert!(processed.mcp.is_some());
    assert_eq!(processed.mcp.unwrap().method(), Some("test"));
}
```

## Conclusion

By coordinating these two initiatives:
1. **Reduce duplicate work** - Share parsers, batch handlers, and ID generators
2. **Accelerate delivery** - SSE gets MCP awareness, MCP gets real transport
3. **Improve quality** - Integrated testing from the start
4. **Future-proof architecture** - Clean interfaces between layers

The SSE proxy integration should include MCP parsing hooks from the beginning, while the MCP message handling should leverage the SSE implementation as its first real-world transport integration.