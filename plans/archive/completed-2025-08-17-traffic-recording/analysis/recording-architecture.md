# Recording Architecture Design

## Executive Summary

This document describes the new architecture for the traffic recording layer that properly separates transport concerns from wire-format recording concerns. The key change is removing `TransportContext::Sse` and passing raw wire data alongside `MessageEnvelope` to the recording layer, where SSE metadata can be extracted when needed for replay.

## Current Problems

1. **Semantic Boundary Violation**: `TransportContext::Sse` carries wire-format details (event_id, event_type, retry_ms) that don't belong in the transport layer
2. **Duplicate Types**: Three different SseEvent structs exist across the codebase
3. **Missing Wire Data**: Recording layer can't access raw wire format for faithful replay
4. **Type Safety Loss**: Untyped HashMap metadata in MessageContext

## Proposed Architecture

### Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                         Wire Format                              │
│        (Raw bytes from network/stdio/process output)             │
└────────────────────────┬─────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Transport Layer                              │
│                                                                  │
│  1. Receives raw bytes                                          │
│  2. Detects format (JSON/SSE/other)                            │
│  3. For SSE: Parses event, extracts JSON-RPC from data field   │
│  4. Creates MessageEnvelope with:                               │
│     - ProtocolMessage (the JSON-RPC message)                   │
│     - TransportContext::Http with ResponseMode::SseStream       │
│  5. Passes both MessageEnvelope AND raw bytes to proxy          │
└────────────────────────┬─────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Proxy Layer                                │
│                                                                  │
│  1. Processes MessageEnvelope (interceptors, routing, etc.)     │
│  2. If recording enabled, passes to recorder:                   │
│     - MessageEnvelope (for the message)                         │
│     - RawWireData (for faithful replay metadata)               │
└────────────────────────┬─────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Recording Layer                               │
│                                                                  │
│  1. Receives MessageEnvelope + RawWireData                      │
│  2. If ResponseMode::SseStream, parses SSE metadata from raw    │
│  3. Stores in FrameMetadata.transport_metadata                  │
│  4. Writes to tape with full wire format for replay             │
└───────────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

#### 1. Raw Wire Data Structure

```rust
/// Raw wire data passed alongside MessageEnvelope for recording
#[derive(Debug, Clone)]
pub struct RawWireData {
    /// The raw bytes as received from the wire
    pub bytes: Arc<Vec<u8>>,
    
    /// The detected wire format
    pub format: WireFormat,
    
    /// Direction of data flow
    pub direction: DataDirection,
}

#[derive(Debug, Clone, Copy)]
pub enum WireFormat {
    /// Plain JSON-RPC
    Json,
    
    /// Server-Sent Events with embedded JSON-RPC
    ServerSentEvent,
    
    /// Unknown/passthrough format
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum DataDirection {
    ClientToServer,
    ServerToClient,
}
```

#### 2. Updated TransportContext

```rust
pub enum TransportContext {
    Stdio { 
        process_id: Option<u32>,
        command: Option<String>,
    },
    Http {
        method: String,
        path: String,
        headers: HashMap<String, String>,
        status_code: Option<u16>,
        remote_addr: Option<String>,
        response_mode: Option<ResponseMode>, // NEW: Json/SseStream/Passthrough
    }
    // No Sse variant!
}
```

#### 3. Consolidated SseEvent

```rust
// Single canonical type in transport::sse::event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SseEvent {
    pub id: Option<String>,        // SSE event ID for reconnection
    pub event_type: String,         // Usually "message" for MCP
    pub data: String,               // Contains JSON-RPC message
    pub retry: Option<u64>,         // Reconnection delay hint in ms
}
```

#### 4. Recording Interface Changes

```rust
impl TapeRecorder {
    /// Record a frame with optional raw wire data
    pub async fn record_frame_with_raw(
        &self,
        envelope: MessageEnvelope,
        raw_data: Option<RawWireData>,
    ) -> RecorderResult<()> {
        // Extract SSE metadata if present
        let transport_metadata = if let Some(raw) = &raw_data {
            Self::extract_transport_metadata(&envelope, raw)?
        } else {
            Self::extract_transport_metadata_legacy(&envelope)?
        };
        
        // Continue with existing recording logic
        self.record_frame_internal(envelope, transport_metadata).await
    }
    
    fn extract_transport_metadata(
        envelope: &MessageEnvelope,
        raw_data: &RawWireData,
    ) -> RecorderResult<TransportMetadata> {
        let mut metadata = TransportMetadata::default();
        
        match &envelope.context.transport {
            TransportContext::Http { response_mode, .. } => {
                // If SSE stream, parse metadata from raw data
                if matches!(response_mode, Some(ResponseMode::SseStream)) {
                    if let WireFormat::ServerSentEvent = raw_data.format {
                        let sse_event = parse_sse_event(&raw_data.bytes)?;
                        metadata.sse_metadata = Some(SseMetadata {
                            event_id: sse_event.id,
                            event_type: Some(sse_event.event_type),
                            retry_ms: sse_event.retry,
                            last_event_id: None, // Could extract from headers
                        });
                    }
                }
                // Continue with HTTP metadata extraction...
            }
            TransportContext::Stdio { .. } => {
                // Extract stdio metadata...
            }
        }
        
        Ok(metadata)
    }
}
```

### Memory Management Strategy

#### Arc vs Rc Decision
- **Use Arc<Vec<u8>>** for raw data sharing
- Rationale:
  - Proxy runs in async/multi-threaded context
  - Recording may happen on different task
  - Arc allows safe sharing across threads
  - Overhead is minimal for large payloads

#### Lifecycle Management
```rust
// In transport layer
let raw_bytes: Vec<u8> = read_from_wire().await?;
let raw_data = RawWireData {
    bytes: Arc::new(raw_bytes.clone()), // Arc for sharing
    format: detect_format(&raw_bytes),
    direction: DataDirection::ServerToClient,
};

// Parse message from bytes
let message = parse_message(&raw_bytes)?;
let envelope = create_envelope(message);

// Pass both to proxy - Arc prevents copy
proxy.handle_message(envelope, Some(raw_data)).await?;

// raw_data dropped when recording completes
```

#### Memory Optimization
- Small messages (<8KB): Acceptable to have both parsed and raw
- Large messages: Consider streaming approach in future
- SSE metadata extraction is lazy - only when recording
- Arc ensures single allocation for raw data

### Transport Layer Changes

#### HTTP Transport Updates

```rust
impl HttpTransport {
    async fn receive_with_raw(&mut self) -> Result<(MessageEnvelope, Option<RawWireData>)> {
        let response = self.client.recv().await?;
        let content_type = response.headers()
            .get("content-type")
            .map(|v| v.to_str().unwrap_or(""))
            .unwrap_or("");
        
        let response_mode = ResponseMode::from_content_type(content_type);
        let raw_bytes = response.bytes().await?;
        
        // Create raw data for recording
        let raw_data = RawWireData {
            bytes: Arc::new(raw_bytes.to_vec()),
            format: match response_mode {
                ResponseMode::Json => WireFormat::Json,
                ResponseMode::SseStream => WireFormat::ServerSentEvent,
                ResponseMode::Passthrough => WireFormat::Unknown,
            },
            direction: DataDirection::ServerToClient,
        };
        
        // Parse message based on format
        let message = match response_mode {
            ResponseMode::Json => {
                serde_json::from_slice(&raw_bytes)?
            }
            ResponseMode::SseStream => {
                let sse_event = parse_sse_event(&raw_bytes)?;
                serde_json::from_str(&sse_event.data)?
            }
            ResponseMode::Passthrough => {
                // Create synthetic message for passthrough
                create_passthrough_message(&raw_bytes)
            }
        };
        
        // Create context with response mode
        let context = MessageContext {
            transport: TransportContext::Http {
                method: "POST".to_string(),
                path: self.path.clone(),
                headers: extract_headers(&response),
                status_code: Some(response.status().as_u16()),
                remote_addr: self.remote_addr.clone(),
                response_mode: Some(response_mode), // NEW field
            },
            session_id: self.session_id.clone(),
            metadata: HashMap::new(), // No SSE metadata here!
        };
        
        let envelope = MessageEnvelope::new(message, context);
        Ok((envelope, Some(raw_data)))
    }
}
```

### Proxy Layer Changes

```rust
impl ForwardProxy {
    async fn proxy_messages(
        mut client: Box<dyn Transport>,
        mut server: Box<dyn Transport>,
        tape_recorder: Option<Arc<TapeRecorder>>,
    ) -> Result<()> {
        loop {
            tokio::select! {
                // Client to server
                result = client.receive_with_raw() => {
                    let (envelope, raw_data) = result?;
                    
                    // Process with interceptors
                    let processed = interceptor.process(envelope).await?;
                    
                    // Record if enabled
                    if let Some(recorder) = &tape_recorder {
                        recorder.record_frame_with_raw(
                            processed.clone(),
                            raw_data
                        ).await?;
                    }
                    
                    // Forward to server
                    server.send(processed).await?;
                }
                
                // Server to client (similar pattern)
                result = server.receive_with_raw() => {
                    // ... same pattern
                }
            }
        }
    }
}
```

## Migration Plan Overview

### Phase 1: Add Infrastructure (Non-breaking)
1. Add `ResponseMode` field to `TransportContext::Http`
2. Create `RawWireData` structure
3. Add `record_frame_with_raw` method alongside existing
4. Implement SSE metadata extraction from raw data

### Phase 2: Update Transports (Non-breaking)
1. Add `receive_with_raw` method to Transport trait
2. Implement in each transport (stdio, http, sse)
3. Update transports to set ResponseMode

### Phase 3: Update Proxies (Non-breaking)
1. Update forward proxy to use new methods
2. Update reverse proxy to use new methods
3. Keep fallback to old methods for compatibility

### Phase 4: Remove Old Code (Breaking)
1. Remove `TransportContext::Sse` variant
2. Remove old `record_frame` method
3. Remove duplicate SseEvent structs
4. Update all tests

## Backward Compatibility

### Tape Format Compatibility

Since we haven't released shadowcat yet, we can make breaking changes to the tape format. However, for any existing test recordings:

```rust
impl TapeRecorder {
    /// Load tape with migration for old format
    pub async fn load_tape(path: &Path) -> Result<Tape> {
        let mut tape = read_tape_file(path).await?;
        
        // Migrate old TransportContext::Sse to Http with ResponseMode
        for frame in &mut tape.frames {
            if let Some(old_sse) = frame.extract_old_sse_context() {
                frame.context.transport = TransportContext::Http {
                    method: "GET".to_string(),
                    path: "/".to_string(),
                    headers: old_sse.headers,
                    status_code: Some(200),
                    remote_addr: None,
                    response_mode: Some(ResponseMode::SseStream),
                };
                
                // Move SSE metadata to transport_metadata
                frame.metadata.transport_metadata.sse_metadata = Some(SseMetadata {
                    event_id: old_sse.event_id,
                    event_type: old_sse.event_type,
                    retry_ms: old_sse.retry_ms,
                    last_event_id: None,
                });
            }
        }
        
        Ok(tape)
    }
}
```

## Testing Strategy

### Unit Tests
1. Test `ResponseMode::from_content_type` detection
2. Test SSE metadata extraction from raw bytes
3. Test Arc memory management
4. Test tape format migration

### Integration Tests
1. Record and replay SSE session
2. Verify SSE metadata preserved
3. Test with real MCP servers (everything server)
4. Performance tests with large payloads

### Conformance Tests
1. Verify MCP protocol compliance
2. Test reconnection with Last-Event-ID
3. Test retry behavior

## Performance Considerations

### Memory Impact
- **Baseline**: MessageEnvelope already in memory
- **Additional**: Arc<Vec<u8>> for raw data
- **Optimization**: Raw data dropped after recording
- **Estimate**: ~2x memory during recording phase only

### CPU Impact
- **Parsing**: SSE already parsed in transport, just preserved
- **Extraction**: Lazy - only when recording enabled
- **Arc operations**: Negligible overhead

### Benchmarks to Run
```rust
#[bench]
fn bench_record_with_raw_small_message() { /* 1KB message */ }

#[bench]
fn bench_record_with_raw_large_message() { /* 1MB message */ }

#[bench]
fn bench_arc_allocation_overhead() { /* Measure Arc vs Box */ }
```

## Implementation Checklist

### Task A.2: Design Architecture ✅
- [x] Define data flow from wire to recording
- [x] Design RawWireData structure
- [x] Plan TransportContext updates
- [x] Design recording interface changes
- [x] Define memory management strategy
- [x] Document backward compatibility approach

### Task A.3: Migration Plan (Next)
- [ ] Detailed step-by-step refactoring sequence
- [ ] Identify all affected files
- [ ] Test migration for each step
- [ ] Risk assessment for each phase

### Task B.1: Consolidate SseEvent
- [ ] Move all to transport::sse::event
- [ ] Remove duplicate in outgoing::http
- [ ] Update all references
- [ ] Ensure serialization compatibility

### Task B.2: Remove TransportContext::Sse
- [ ] Add ResponseMode to Http variant
- [ ] Migrate existing Sse usage to Http
- [ ] Update all match statements
- [ ] Fix compilation errors

### Task C.1: Pass Raw Data
- [ ] Add receive_with_raw to Transport trait
- [ ] Implement for each transport
- [ ] Update proxy to pass raw data
- [ ] Add record_frame_with_raw

### Task C.2: Extract SSE Metadata
- [ ] Implement parse_sse_event
- [ ] Extract metadata in recorder
- [ ] Store in transport_metadata
- [ ] Test extraction logic

## Risks and Mitigation

| Risk | Mitigation |
|------|------------|
| Memory usage doubles during recording | Use Arc, drop early, add memory limit config |
| Breaking existing tests | Run full test suite after each step |
| SSE parsing complexity | Reuse existing SseEvent parser |
| Performance regression | Benchmark before/after, add metrics |

## Conclusion

This architecture cleanly separates transport concerns from recording concerns while maintaining type safety and performance. The key insight is that SSE metadata belongs in the recording layer for replay, not in the transport layer for message processing.

The use of `ResponseMode` in `TransportContext::Http` properly indicates the response format without leaking wire-format details. The recording layer can then extract any needed metadata from the raw wire data when recording is enabled.

This design respects semantic boundaries, eliminates code duplication, and provides a clean path forward for future transport types.