# Integration Glue Tasks

These tasks connect SSE proxy integration with MCP message handling components.

## S.4: Add MCP Parser Hooks to Transport

**Duration**: 2 hours  
**Dependencies**: S.2 (SSE Transport), F.2 (Minimal Parser)  
**File**: Enhancement to `src/transport/sse_transport.rs`

### Implementation

```rust
// Add to SseTransport struct
pub struct SseTransport {
    // ... existing fields ...
    
    // MCP integration
    mcp_parser: Option<MinimalMcpParser>,
    event_generator: Arc<UnifiedEventIdGenerator>,
    message_context: Arc<RwLock<HashMap<String, McpMessageContext>>>,
}

impl SseTransport {
    // Enhanced builder
    pub fn builder() -> SseTransportBuilder {
        SseTransportBuilder {
            enable_mcp_parsing: true,  // Default to enabled
            ..Default::default()
        }
    }
    
    // Process SSE event with MCP awareness
    async fn process_sse_event(&mut self, event: SseEvent) -> Result<ProcessedMessage> {
        let mut context = McpMessageContext::new(
            self.session_id.clone(),
            TransportType::Sse,
        );
        context.start_processing();
        
        // Try to parse as MCP if parser available
        let mcp_info = if let Some(parser) = &self.mcp_parser {
            match parser.parse(&event.data) {
                Ok(info) => Some(info),
                Err(e) => {
                    debug!("Not an MCP message or parse failed: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        // Generate event ID with correlation
        let event_id = if let Some(info) = &mcp_info {
            if let Some(first_msg) = info.messages.first() {
                self.event_generator.generate(
                    &self.session_id,
                    first_msg.id.as_ref(),
                )
            } else {
                self.event_generator.generate_simple()
            }
        } else {
            self.event_generator.generate_simple()
        };
        
        // Store context for correlation
        if let Some(info) = &mcp_info {
            for msg in &info.messages {
                if let Some(id) = &msg.id {
                    let correlation_id = format!("{:?}", id);
                    context.correlation_id = Some(correlation_id.clone());
                    self.message_context.write().await.insert(
                        correlation_id,
                        context.clone(),
                    );
                }
            }
        }
        
        context.end_processing();
        
        // Convert to transport message
        let transport_msg = self.convert_to_transport(event.data)?;
        
        Ok(ProcessedMessage {
            transport: transport_msg,
            mcp_info,
            event_id,
            context,
        })
    }
}

#[derive(Debug)]
pub struct ProcessedMessage {
    pub transport: TransportMessage,
    pub mcp_info: Option<ParsedInfo>,
    pub event_id: String,
    pub context: McpMessageContext,
}
```

## R.4: Add Early Message Correlation

**Duration**: 2 hours  
**Dependencies**: R.1 (Dual-Method Endpoint), F.2 (Minimal Parser)  
**File**: Enhancement to `src/proxy/reverse/mcp_endpoint.rs`

### Implementation

```rust
// Add to McpEndpointState
pub struct McpEndpointState {
    // ... existing fields ...
    
    // Early correlation tracking
    pub pending_requests: Arc<RwLock<HashMap<String, PendingRequest>>>,
    pub mcp_parser: MinimalMcpParser,
    pub correlation_timeout: Duration,
}

#[derive(Debug, Clone)]
struct PendingRequest {
    request_id: String,
    method: String,
    received_at: Instant,
    session_id: String,
    mcp_session_id: Option<String>,
}

// Enhanced POST handler
async fn handle_post(
    headers: HeaderMap,
    state: McpEndpointState,
    body: Option<Json<Value>>,
) -> Result<Response, McpError> {
    let protocol_version = extract_protocol_version(&headers)?;
    let session_id = extract_session_id(&headers)?;
    
    // Parse with MCP parser for early understanding
    let parsed = state.mcp_parser.parse(&serde_json::to_string(&body.0)?)?;
    
    // Track requests for correlation
    for msg in &parsed.messages {
        if msg.message_type == MessageType::Request {
            if let Some(id) = &msg.id {
                let pending = PendingRequest {
                    request_id: format!("{:?}", id),
                    method: msg.method.clone().unwrap_or_default(),
                    received_at: Instant::now(),
                    session_id: session_id.clone().unwrap_or_default(),
                    mcp_session_id: session_id.clone(),
                };
                
                state.pending_requests.write().await.insert(
                    pending.request_id.clone(),
                    pending,
                );
                
                // Schedule cleanup after timeout
                let pending_reqs = state.pending_requests.clone();
                let req_id = pending.request_id.clone();
                let timeout = state.correlation_timeout;
                tokio::spawn(async move {
                    tokio::time::sleep(timeout).await;
                    pending_reqs.write().await.remove(&req_id);
                });
            }
        }
    }
    
    // Continue with existing logic, but now with parsed MCP context
    // ...
}

// Add correlation lookup for responses
async fn correlate_response(
    state: &McpEndpointState,
    response_id: &str,
) -> Option<PendingRequest> {
    state.pending_requests.write().await.remove(response_id)
}
```

## M.5: Wire Correlation to SSE Transport

**Duration**: 2 hours  
**Dependencies**: M.4 (Correlation Engine), S.4 (Parser Hooks)  
**File**: Integration code in `src/transport/sse_transport.rs`

### Implementation

```rust
use crate::mcp::correlation::{MessageCorrelator, CorrelationId};

// Enhanced SseTransport with correlation
pub struct SseTransport {
    // ... existing fields ...
    
    // Full correlation engine (when available)
    correlator: Option<Arc<MessageCorrelator>>,
}

impl SseTransport {
    // Upgrade to use full correlation engine
    pub fn with_correlator(mut self, correlator: Arc<MessageCorrelator>) -> Self {
        self.correlator = Some(correlator);
        self
    }
    
    async fn process_with_correlation(&mut self, message: ProcessedMessage) -> Result<()> {
        if let Some(correlator) = &self.correlator {
            if let Some(mcp_info) = &message.mcp_info {
                for msg in &mcp_info.messages {
                    match msg.message_type {
                        MessageType::Request => {
                            if let Some(id) = &msg.id {
                                correlator.track_request(
                                    CorrelationId::from(id),
                                    msg.clone(),
                                    message.context.clone(),
                                ).await?;
                            }
                        }
                        MessageType::Response => {
                            if let Some(id) = &msg.id {
                                if let Some(request) = correlator.match_response(
                                    CorrelationId::from(id),
                                    msg.clone(),
                                ).await? {
                                    // Calculate response time
                                    let response_time = message.context.timestamp
                                        .duration_since(request.timestamp);
                                    
                                    // Store metrics
                                    self.metrics.record_response_time(
                                        request.method.as_deref().unwrap_or("unknown"),
                                        response_time,
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

## I.4: SSE Stream Interception

**Duration**: 3 hours  
**Dependencies**: I.3 (Interceptor Chain), S.4 (Parser Hooks)  
**File**: `src/interceptor/sse_stream.rs`

### Implementation

```rust
use crate::interceptor::{McpInterceptor, InterceptDecision};
use crate::transport::sse::ProcessedMessage;

pub struct SseStreamInterceptor {
    chain: Arc<InterceptorChain>,
    buffer: Vec<ProcessedMessage>,
    stream_context: StreamContext,
}

#[derive(Debug, Clone)]
pub struct StreamContext {
    pub session_id: String,
    pub stream_id: Uuid,
    pub started_at: Instant,
    pub message_count: usize,
    pub last_method: Option<String>,
    pub in_sequence: bool,
}

impl SseStreamInterceptor {
    pub async fn intercept_stream_message(
        &mut self,
        message: ProcessedMessage,
    ) -> Result<Vec<ProcessedMessage>> {
        // Update stream context
        self.stream_context.message_count += 1;
        if let Some(mcp_info) = &message.mcp_info {
            if let Some(first_msg) = mcp_info.messages.first() {
                self.stream_context.last_method = first_msg.method.clone();
            }
        }
        
        // Apply interceptors if MCP message
        if let Some(mcp_info) = &message.mcp_info {
            let mut results = Vec::new();
            
            for mcp_msg in &mcp_info.messages {
                let decision = self.chain.intercept(
                    mcp_msg,
                    &message.context,
                ).await?;
                
                match decision {
                    InterceptDecision::Allow => {
                        results.push(message.clone());
                    }
                    InterceptDecision::Block(reason) => {
                        warn!("Blocked SSE message: {}", reason);
                        // Could inject error event here
                    }
                    InterceptDecision::Modify(modified) => {
                        let mut modified_msg = message.clone();
                        // Update with modified content
                        results.push(modified_msg);
                    }
                    InterceptDecision::Delay(duration) => {
                        tokio::time::sleep(duration).await;
                        results.push(message.clone());
                    }
                    InterceptDecision::Fork(copies) => {
                        for _ in 0..copies {
                            results.push(message.clone());
                        }
                    }
                }
            }
            
            Ok(results)
        } else {
            // Not MCP, pass through
            Ok(vec![message])
        }
    }
    
    pub async fn intercept_stream_sequence(
        &mut self,
        messages: Vec<ProcessedMessage>,
    ) -> Result<Vec<ProcessedMessage>> {
        self.stream_context.in_sequence = true;
        
        // Could apply sequence-based rules here
        let mut results = Vec::new();
        for msg in messages {
            let intercepted = self.intercept_stream_message(msg).await?;
            results.extend(intercepted);
        }
        
        self.stream_context.in_sequence = false;
        Ok(results)
    }
}
```

## I.5: Reverse Proxy Interception

**Duration**: 2 hours  
**Dependencies**: I.3 (Interceptor Chain), R.4 (Early Correlation)  
**File**: Enhancement to `src/proxy/reverse.rs`

### Implementation

```rust
// Add to ReverseProxy
pub struct ReverseProxy {
    // ... existing fields ...
    
    interceptor_chain: Option<Arc<InterceptorChain>>,
}

impl ReverseProxy {
    async fn handle_mcp_message(
        &mut self,
        message: MinimalMessage,
        context: McpMessageContext,
    ) -> Result<MinimalMessage> {
        if let Some(chain) = &self.interceptor_chain {
            // Apply server-side interception
            let decision = chain.intercept(&message, &context).await?;
            
            match decision {
                InterceptDecision::Allow => Ok(message),
                InterceptDecision::Block(reason) => {
                    // Return error response
                    Err(McpError::Blocked(reason))
                }
                InterceptDecision::Modify(modified) => {
                    Ok(modified)
                }
                _ => Ok(message), // Other decisions not applicable here
            }
        } else {
            Ok(message)
        }
    }
}
```

## C.4: SSE Recording Integration

**Duration**: 2 hours  
**Dependencies**: C.2 (Session Recorder), S.4 (Parser Hooks)  
**File**: Integration in `src/transport/sse_transport.rs`

### Implementation

```rust
// Add to SseTransport
pub struct SseTransport {
    // ... existing fields ...
    
    recorder: Option<Arc<McpRecorder>>,
}

impl SseTransport {
    pub fn with_recorder(mut self, recorder: Arc<McpRecorder>) -> Self {
        self.recorder = Some(recorder);
        self
    }
    
    async fn record_if_enabled(&self, message: &ProcessedMessage) -> Result<()> {
        if let Some(recorder) = &self.recorder {
            if let Some(mcp_info) = &message.mcp_info {
                for mcp_msg in &mcp_info.messages {
                    recorder.record_message(
                        mcp_msg.raw.clone(),
                        message.context.clone(),
                    ).await?;
                }
            }
        }
        Ok(())
    }
}
```

## C.5: Reverse Proxy Recording

**Duration**: 2 hours  
**Dependencies**: C.2 (Session Recorder), R.4 (Early Correlation)  
**File**: Enhancement to `src/proxy/reverse.rs`

### Implementation

```rust
// Add to reverse proxy endpoint handler
async fn handle_and_record(
    state: &McpEndpointState,
    message: MinimalMessage,
    context: McpMessageContext,
) -> Result<()> {
    if let Some(recorder) = &state.recorder {
        // Record with HTTP-specific metadata
        let mut enriched_context = context.clone();
        enriched_context.metadata.insert(
            "transport".to_string(),
            json!("streamable-http"),
        );
        enriched_context.metadata.insert(
            "endpoint".to_string(),
            json!("/mcp"),
        );
        
        recorder.record_message(
            message.raw.clone(),
            enriched_context,
        ).await?;
    }
    Ok(())
}
```

## P.4: SSE Replay Support

**Duration**: 3 hours  
**Dependencies**: P.1 (Replay Engine), S.2 (SSE Transport)  
**File**: `src/replay/sse_replay.rs`

### Implementation

```rust
use crate::replay::{ReplayEngine, ReplayTarget};
use crate::transport::sse::SseTransport;

pub struct SseReplayAdapter {
    engine: Arc<ReplayEngine>,
    transport: SseTransport,
}

impl SseReplayAdapter {
    pub async fn replay_via_sse(
        &mut self,
        tape_id: TapeId,
        target_url: String,
    ) -> Result<ReplayReport> {
        // Load tape
        let tape = self.engine.load_tape(tape_id).await?;
        
        // Configure SSE transport as replay target
        let mut transport = SseTransport::builder()
            .url(target_url.parse()?)
            .enable_mcp_parsing(true)
            .build()
            .await?;
        
        // Connect transport
        transport.connect().await?;
        
        // Replay each message
        for entry in tape.entries {
            // Transform if needed
            let transformed = self.engine.transform_message(
                entry.message.raw.clone(),
            ).await?;
            
            // Send via SSE
            transport.send(transformed).await?;
            
            // Handle timing
            if let Some(delay) = self.calculate_delay(&entry) {
                tokio::time::sleep(delay).await;
            }
            
            // Check for responses if request
            if entry.message.parsed.message_type == MessageType::Request {
                if let Some(response) = transport.receive().await? {
                    // Validate if configured
                    if self.engine.config.validate_responses {
                        self.validate_response(response, &entry).await?;
                    }
                }
            }
        }
        
        // Generate report
        self.engine.generate_report().await
    }
}

impl ReplayTarget for SseTransport {
    async fn send(&mut self, message: McpMessage) -> Result<()> {
        // Convert MCP message to transport format
        let transport_msg = self.convert_from_mcp(message)?;
        Transport::send(self, transport_msg).await
    }
    
    async fn receive(&mut self) -> Result<Option<McpMessage>> {
        if let Some(msg) = Transport::receive(self).await? {
            // Convert transport message to MCP format
            self.convert_to_mcp(msg)
        } else {
            Ok(None)
        }
    }
}
```

## Testing Strategy

Each glue task should have integration tests verifying the connection:

```rust
#[tokio::test]
async fn test_sse_with_mcp_parser() {
    let transport = SseTransport::builder()
        .enable_mcp_parsing(true)
        .build()
        .await
        .unwrap();
    
    let event = SseEvent::new(r#"{"jsonrpc":"2.0","method":"test","id":1}"#);
    let processed = transport.process_event(event).await.unwrap();
    
    assert!(processed.mcp_info.is_some());
    assert_eq!(
        processed.mcp_info.unwrap().messages[0].method,
        Some("test".to_string())
    );
}

#[tokio::test]
async fn test_correlation_tracking() {
    let correlator = Arc::new(MessageCorrelator::new());
    let transport = SseTransport::builder()
        .with_correlator(correlator.clone())
        .build()
        .await
        .unwrap();
    
    // Send request
    let request_event = SseEvent::new(r#"{"jsonrpc":"2.0","method":"test","id":1}"#);
    transport.process_event(request_event).await.unwrap();
    
    // Send response
    let response_event = SseEvent::new(r#"{"jsonrpc":"2.0","result":{},"id":1}"#);
    transport.process_event(response_event).await.unwrap();
    
    // Verify correlation
    let stats = correlator.get_stats().await;
    assert_eq!(stats.matched_pairs, 1);
}
```