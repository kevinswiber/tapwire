# Task 2.2: SSE Response Handler

## Overview
Implement the SSE response handler for the reverse proxy to detect when to return SSE streams vs JSON, convert TransportMessages to SSE events, and manage response streaming.

**Duration**: 4 hours  
**Priority**: HIGH  
**Prerequisites**: Task 2.1 (Dual-method endpoint), SSE infrastructure  
**Compatibility Note**: See [MCP 2025-03-26 Compatibility Guide](../compatibility-2025-03-26.md) for batch response formatting

## Current State

Existing components:
- `/mcp` endpoint routing (from Task 2.1)
- SSE event types and parser
- Session management
- TransportMessage types

Missing:
- Logic to detect SSE vs JSON response preference
- SSE stream creation from TransportMessages
- Chunked transfer encoding support
- Event ID generation and tracking
- Stream lifecycle management

## Requirements

### Functional Requirements
1. Detect when to return SSE stream vs JSON response
2. Create SSE response streams from transport messages
3. Convert TransportMessage to properly formatted SSE events
4. Support chunked transfer encoding
5. Manage stream lifecycle (creation, data flow, termination)

### Non-Functional Requirements
- Zero-copy streaming where possible
- Backpressure handling for slow clients
- Proper cleanup on disconnect
- < 10ms latency for event conversion

## Implementation Plan

### Step 1: SSE Response Decision Logic
**File**: `src/proxy/reverse/sse_handler.rs` (new)

```rust
use crate::transport::{TransportMessage, SessionId};
use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Determines whether to return SSE or JSON based on request/response characteristics
pub struct ResponseDecider {
    config: ResponseConfig,
}

#[derive(Debug, Clone)]
pub struct ResponseConfig {
    /// Size threshold for automatic streaming (bytes)
    pub stream_threshold: usize,
    /// Patterns indicating streaming preference
    pub stream_indicators: Vec<String>,
    /// Maximum response time before forcing stream
    pub max_response_time: Duration,
    /// Enable automatic streaming for subscriptions
    pub auto_stream_subscriptions: bool,
}

impl Default for ResponseConfig {
    fn default() -> Self {
        Self {
            stream_threshold: 10_000, // 10KB
            stream_indicators: vec![
                "subscribe".to_string(),
                "watch".to_string(),
                "stream".to_string(),
                "tail".to_string(),
            ],
            max_response_time: Duration::from_secs(5),
            auto_stream_subscriptions: true,
        }
    }
}

impl ResponseDecider {
    pub fn new(config: ResponseConfig) -> Self {
        Self { config }
    }
    
    /// Decide whether to stream based on request and initial response
    pub fn should_stream(
        &self,
        request: &TransportMessage,
        response_hint: Option<&Value>,
        client_accepts_sse: bool,
    ) -> StreamDecision {
        // Client must accept SSE
        if !client_accepts_sse {
            return StreamDecision::Json;
        }
        
        // Check request method for streaming indicators
        if let TransportMessage::Request { method, .. } = request {
            for indicator in &self.config.stream_indicators {
                if method.contains(indicator) {
                    return StreamDecision::Stream(StreamReason::MethodIndicator);
                }
            }
        }
        
        // Check response hints
        if let Some(hint) = response_hint {
            // Check for explicit streaming flag
            if hint.get("stream").and_then(|v| v.as_bool()).unwrap_or(false) {
                return StreamDecision::Stream(StreamReason::ExplicitFlag);
            }
            
            // Check response size
            let size = estimate_json_size(hint);
            if size > self.config.stream_threshold {
                return StreamDecision::Stream(StreamReason::LargeResponse(size));
            }
            
            // Check for subscription/event patterns
            if self.config.auto_stream_subscriptions {
                if hint.get("subscription_id").is_some() ||
                   hint.get("event_stream").is_some() {
                    return StreamDecision::Stream(StreamReason::Subscription);
                }
            }
        }
        
        StreamDecision::Json
    }
}

#[derive(Debug, Clone)]
pub enum StreamDecision {
    Json,
    Stream(StreamReason),
}

#[derive(Debug, Clone)]
pub enum StreamReason {
    MethodIndicator,
    ExplicitFlag,
    LargeResponse(usize),
    Subscription,
    Timeout,
}

fn estimate_json_size(value: &Value) -> usize {
    // Quick estimation without full serialization
    match value {
        Value::Null => 4,
        Value::Bool(_) => 5,
        Value::Number(_) => 10,
        Value::String(s) => s.len() + 2,
        Value::Array(arr) => arr.iter().map(estimate_json_size).sum::<usize>() + arr.len() * 2,
        Value::Object(obj) => obj.iter()
            .map(|(k, v)| k.len() + estimate_json_size(v) + 4)
            .sum::<usize>() + 2,
    }
}
```

### Step 2: SSE Stream Creator
**File**: `src/proxy/reverse/sse_handler.rs`

```rust
/// Creates SSE streams from transport messages
pub struct SseStreamCreator {
    event_id_generator: Arc<EventIdGenerator>,
    session_manager: Arc<SessionManager>,
    config: StreamConfig,
}

#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub heartbeat_interval: Duration,
    pub max_event_size: usize,
    pub buffer_size: usize,
    pub compression: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(30),
            max_event_size: 1024 * 1024, // 1MB
            buffer_size: 100,
            compression: false,
        }
    }
}

impl SseStreamCreator {
    pub fn new(
        session_manager: Arc<SessionManager>,
        config: StreamConfig,
    ) -> Self {
        Self {
            event_id_generator: Arc::new(EventIdGenerator::new()),
            session_manager,
            config,
        }
    }
    
    /// Create an SSE stream from a channel of transport messages
    pub async fn create_stream(
        &self,
        session_id: SessionId,
        mut receiver: mpsc::Receiver<TransportMessage>,
    ) -> impl Stream<Item = Result<Event, axum::Error>> {
        let event_gen = self.event_id_generator.clone();
        let heartbeat_interval = self.config.heartbeat_interval;
        let max_event_size = self.config.max_event_size;
        
        async_stream::stream! {
            let mut heartbeat = tokio::time::interval(heartbeat_interval);
            heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            
            loop {
                tokio::select! {
                    // Receive transport messages
                    msg = receiver.recv() => {
                        match msg {
                            Some(transport_msg) => {
                                match convert_to_sse_event(transport_msg, &event_gen, max_event_size).await {
                                    Ok(event) => yield Ok(event),
                                    Err(e) => {
                                        error!("Failed to convert message: {}", e);
                                        // Send error event
                                        let error_event = Event::default()
                                            .event("error")
                                            .data(format!("Conversion error: {}", e));
                                        yield Ok(error_event);
                                    }
                                }
                            }
                            None => {
                                // Channel closed, end stream
                                debug!("Message channel closed for session {}", session_id);
                                break;
                            }
                        }
                    }
                    
                    // Send heartbeat
                    _ = heartbeat.tick() => {
                        let heartbeat_event = Event::default()
                            .event("heartbeat")
                            .data(format!("{}", chrono::Utc::now().timestamp()));
                        yield Ok(heartbeat_event);
                    }
                }
            }
            
            // Send stream end event
            let end_event = Event::default()
                .event("stream-end")
                .id(event_gen.generate(&session_id))
                .data("Stream terminated");
            yield Ok(end_event);
        }
    }
    
    /// Create a stream for a single response
    pub async fn create_single_response_stream(
        &self,
        session_id: SessionId,
        response: TransportMessage,
    ) -> impl Stream<Item = Result<Event, axum::Error>> {
        let event_gen = self.event_id_generator.clone();
        let max_event_size = self.config.max_event_size;
        
        async_stream::stream! {
            // Send the response as an SSE event
            match convert_to_sse_event(response, &event_gen, max_event_size).await {
                Ok(event) => yield Ok(event),
                Err(e) => {
                    error!("Failed to convert response: {}", e);
                    let error_event = Event::default()
                        .event("error")
                        .data(format!("Response conversion error: {}", e));
                    yield Ok(error_event);
                }
            }
            
            // Send stream end to indicate completion
            let end_event = Event::default()
                .event("stream-end")
                .id(event_gen.generate(&session_id))
                .data("Response complete");
            yield Ok(end_event);
        }
    }
}
```

### Step 3: Message to SSE Event Conversion
**File**: `src/proxy/reverse/sse_handler.rs`

```rust
/// Convert TransportMessage to SSE Event
async fn convert_to_sse_event(
    msg: TransportMessage,
    event_gen: &EventIdGenerator,
    max_size: usize,
) -> Result<Event, SseConversionError> {
    // Convert to JSON-RPC format
    let json = match msg {
        TransportMessage::Request { id, method, params } => {
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": method,
                "params": params
            })
        }
        TransportMessage::Response { id, result, error } => {
            let mut resp = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id
            });
            if let Some(result) = result {
                resp["result"] = result;
            }
            if let Some(error) = error {
                resp["error"] = error;
            }
            resp
        }
        TransportMessage::Notification { method, params } => {
            serde_json::json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params
            })
        }
    };
    
    // Serialize to string
    let data = serde_json::to_string(&json)
        .map_err(|e| SseConversionError::Serialization(e))?;
    
    // Check size limit
    if data.len() > max_size {
        return Err(SseConversionError::TooLarge(data.len(), max_size));
    }
    
    // Determine event type
    let event_type = match msg {
        TransportMessage::Request { .. } => "request",
        TransportMessage::Response { .. } => "response",
        TransportMessage::Notification { .. } => "notification",
    };
    
    // Create SSE event
    let event = Event::default()
        .id(event_gen.generate_simple())
        .event(event_type)
        .data(data);
    
    Ok(event)
}

/// Event ID generator with session scoping
pub struct EventIdGenerator {
    counter: Arc<AtomicU64>,
    node_id: String,
}

impl EventIdGenerator {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(AtomicU64::new(0)),
            node_id: format!("{:x}", uuid::Uuid::new_v4().as_u128() & 0xFFFF),
        }
    }
    
    pub fn generate(&self, session_id: &SessionId) -> String {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        format!("{}-{}-{}", session_id, self.node_id, count)
    }
    
    pub fn generate_simple(&self) -> String {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        format!("{}-{}", self.node_id, count)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SseConversionError {
    #[error("Serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Event too large: {0} bytes (max: {1})")]
    TooLarge(usize, usize),
    
    #[error("Invalid message format")]
    InvalidFormat,
}
```

### Step 4: Response Handler Integration
**File**: `src/proxy/reverse/sse_handler.rs`

```rust
/// Main SSE response handler
pub struct SseResponseHandler {
    decider: ResponseDecider,
    creator: SseStreamCreator,
    session_manager: Arc<SessionManager>,
}

impl SseResponseHandler {
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        Self {
            decider: ResponseDecider::new(ResponseConfig::default()),
            creator: SseStreamCreator::new(
                session_manager.clone(),
                StreamConfig::default(),
            ),
            session_manager,
        }
    }
    
    /// Handle a request and determine response type
    pub async fn handle_request(
        &self,
        request: TransportMessage,
        session_id: SessionId,
        client_accepts_sse: bool,
        upstream_client: &HttpClient,
    ) -> Result<ResponseType, HandlerError> {
        // Forward request to upstream
        let upstream_response = upstream_client
            .send_request(request.clone())
            .await?;
        
        // Peek at response to determine format
        let decision = self.decider.should_stream(
            &request,
            upstream_response.peek(),
            client_accepts_sse,
        );
        
        match decision {
            StreamDecision::Json => {
                // Return immediate JSON response
                let json = upstream_response.into_json().await?;
                Ok(ResponseType::Json(json))
            }
            StreamDecision::Stream(reason) => {
                info!("Streaming response for session {} (reason: {:?})", session_id, reason);
                
                // Create response channel
                let (tx, rx) = mpsc::channel(100);
                
                // Send initial response
                if let Some(initial) = upstream_response.initial_message() {
                    tx.send(initial).await?;
                }
                
                // Set up ongoing stream forwarding
                self.setup_stream_forwarding(
                    session_id.clone(),
                    upstream_response,
                    tx,
                ).await?;
                
                // Create SSE stream
                let stream = self.creator.create_stream(session_id, rx).await;
                Ok(ResponseType::Stream(Box::pin(stream)))
            }
        }
    }
    
    /// Set up forwarding from upstream to SSE stream
    async fn setup_stream_forwarding(
        &self,
        session_id: SessionId,
        mut upstream: UpstreamResponse,
        tx: mpsc::Sender<TransportMessage>,
    ) -> Result<(), HandlerError> {
        // Spawn task to forward messages
        tokio::spawn(async move {
            while let Some(msg) = upstream.next_message().await {
                if tx.send(msg).await.is_err() {
                    // Receiver dropped, stop forwarding
                    break;
                }
            }
            debug!("Stream forwarding ended for session {}", session_id);
        });
        
        Ok(())
    }
}

pub enum ResponseType {
    Json(Value),
    Stream(Pin<Box<dyn Stream<Item = Result<Event, axum::Error>> + Send>>),
}

impl IntoResponse for ResponseType {
    fn into_response(self) -> Response {
        match self {
            ResponseType::Json(json) => {
                Json(json).into_response()
            }
            ResponseType::Stream(stream) => {
                Sse::new(stream)
                    .keep_alive(axum::response::sse::KeepAlive::default())
                    .into_response()
            }
        }
    }
}
```

### Step 5: Chunked Transfer Support
**File**: `src/proxy/reverse/chunked.rs` (new)

```rust
use hyper::body::HttpBody;
use tokio::io::{AsyncWrite, AsyncWriteExt};

/// Support for chunked transfer encoding
pub struct ChunkedEncoder<W> {
    writer: W,
    buffer: Vec<u8>,
    buffer_size: usize,
}

impl<W: AsyncWrite + Unpin> ChunkedEncoder<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            buffer: Vec::with_capacity(8192),
            buffer_size: 8192,
        }
    }
    
    pub async fn write_chunk(&mut self, data: &[u8]) -> io::Result<()> {
        // Write chunk size in hex
        let size_str = format!("{:X}\r\n", data.len());
        self.writer.write_all(size_str.as_bytes()).await?;
        
        // Write chunk data
        self.writer.write_all(data).await?;
        
        // Write chunk trailer
        self.writer.write_all(b"\r\n").await?;
        
        // Flush if buffer is full
        if self.buffer.len() >= self.buffer_size {
            self.flush().await?;
        }
        
        Ok(())
    }
    
    pub async fn finish(mut self) -> io::Result<()> {
        // Write final chunk
        self.writer.write_all(b"0\r\n\r\n").await?;
        self.flush().await
    }
    
    async fn flush(&mut self) -> io::Result<()> {
        self.writer.flush().await
    }
}
```

## Testing Plan

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_response_decision_json() {
        let decider = ResponseDecider::new(ResponseConfig::default());
        
        let request = TransportMessage::Request {
            id: serde_json::json!(1),
            method: "simple_query".to_string(),
            params: Value::Null,
        };
        
        let decision = decider.should_stream(&request, None, false);
        assert!(matches!(decision, StreamDecision::Json));
    }
    
    #[test]
    fn test_response_decision_stream() {
        let decider = ResponseDecider::new(ResponseConfig::default());
        
        let request = TransportMessage::Request {
            id: serde_json::json!(1),
            method: "subscribe_events".to_string(),
            params: Value::Null,
        };
        
        let decision = decider.should_stream(&request, None, true);
        assert!(matches!(decision, StreamDecision::Stream(_)));
    }
    
    #[tokio::test]
    async fn test_message_conversion() {
        let event_gen = EventIdGenerator::new();
        
        let msg = TransportMessage::Response {
            id: serde_json::json!(1),
            result: Some(serde_json::json!({"value": 42})),
            error: None,
        };
        
        let event = convert_to_sse_event(msg, &event_gen, 10000).await.unwrap();
        
        // Verify event has required fields
        assert!(event.id.is_some());
        assert_eq!(event.event, Some("response".to_string()));
    }
    
    #[test]
    fn test_event_id_generation() {
        let generator = EventIdGenerator::new();
        let session_id = SessionId::from("test-session");
        
        let id1 = generator.generate(&session_id);
        let id2 = generator.generate(&session_id);
        
        assert_ne!(id1, id2);
        assert!(id1.starts_with("test-session"));
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_sse_stream_creation() {
    let session_manager = Arc::new(SessionManager::new());
    let creator = SseStreamCreator::new(session_manager, StreamConfig::default());
    
    let (tx, rx) = mpsc::channel(10);
    let session_id = SessionId::from("test");
    
    // Send test messages
    tx.send(TransportMessage::Notification {
        method: "test".to_string(),
        params: Value::Null,
    }).await.unwrap();
    
    drop(tx); // Close channel
    
    let mut stream = Box::pin(creator.create_stream(session_id, rx).await);
    
    // Collect events
    let mut events = Vec::new();
    while let Some(result) = stream.next().await {
        events.push(result.unwrap());
    }
    
    // Should have notification and stream-end
    assert!(events.len() >= 2);
}

#[tokio::test]
async fn test_chunked_encoding() {
    let mut buffer = Vec::new();
    let mut encoder = ChunkedEncoder::new(&mut buffer);
    
    encoder.write_chunk(b"Hello").await.unwrap();
    encoder.write_chunk(b"World").await.unwrap();
    encoder.finish().await.unwrap();
    
    let output = String::from_utf8(buffer).unwrap();
    assert!(output.contains("5\r\nHello\r\n"));
    assert!(output.contains("5\r\nWorld\r\n"));
    assert!(output.ends_with("0\r\n\r\n"));
}
```

## Success Criteria

- [ ] Automatic detection of SSE vs JSON responses
- [ ] Proper SSE event formatting with IDs
- [ ] Stream lifecycle management working
- [ ] Heartbeat/keepalive implemented
- [ ] Chunked transfer encoding supported
- [ ] All tests passing
- [ ] Performance: < 10ms conversion latency

## Dependencies

- Task 2.1 (Dual-method endpoint)
- SSE event types and parser
- Session management system
- Upstream HTTP client

## Notes

- Consider implementing event buffering for burst scenarios
- May need to add compression support (gzip/deflate)
- Should monitor memory usage with many concurrent streams
- Coordinate with Task 3.1 for interceptor integration
- Event IDs must be globally unique within session for resumability