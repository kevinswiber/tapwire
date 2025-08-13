# Task 1.2: Create SSE Transport Wrapper

## Overview
Implement the Transport trait for SSE, creating a wrapper that bridges the existing SSE implementation with Shadowcat's transport abstraction layer.

**Duration**: 3-4 hours  
**Priority**: HIGH  
**Prerequisites**: Task 1.1 (CLI options), Completed SSE implementation  
**Compatibility Note**: See [MCP 2025-03-26 Compatibility Guide](../compatibility-2025-03-26.md) for batch handling requirements

## Current State

Existing components:
- `Transport` trait defining common transport interface
- `SseHttpClient` for HTTP/SSE communication
- `SessionAwareSseManager` for session management
- `ReconnectingStream` for resilient connections

Missing:
- Transport trait implementation for SSE
- Bi-directional message mapping
- Stream lifecycle management
- Error translation layer

## Requirements

### Functional Requirements
1. Implement `Transport` trait for SSE
2. Map `TransportMessage` to/from SSE events
3. Handle connection lifecycle (connect/disconnect)
4. Support both client-initiated and server-initiated messages
5. Integrate session management

### Non-Functional Requirements
- Zero-copy message passing where possible
- Graceful handling of connection drops
- < 5% latency overhead
- Thread-safe for concurrent access

## Implementation Plan

### Step 1: Define SSE Transport Structure
**File**: `src/transport/sse_transport.rs` (new)

```rust
use crate::transport::{Transport, TransportMessage, TransportError, SessionId};
use crate::transport::sse::{
    SseHttpClient, SessionAwareSseManager, MessageResponse,
    ReconnectingStream, SseEvent,
};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use futures::{Stream, StreamExt};
use serde_json::Value;

pub struct SseTransport {
    /// HTTP client for SSE communication
    client: Arc<SseHttpClient>,
    
    /// Session-aware manager
    manager: Arc<SessionAwareSseManager>,
    
    /// Configuration
    config: SseTransportConfig,
    
    /// Incoming message channel
    incoming_rx: mpsc::Receiver<TransportMessage>,
    incoming_tx: mpsc::Sender<TransportMessage>,
    
    /// Active streams (connection_id -> stream)
    active_streams: Arc<RwLock<HashMap<Uuid, ReconnectingStream>>>,
    
    /// Session context
    session_id: Option<SessionId>,
    mcp_session_id: Option<String>,
    
    /// Shutdown signal
    shutdown: Arc<tokio::sync::Notify>,
}

#[derive(Debug, Clone)]
pub struct SseTransportConfig {
    pub url: url::Url,
    pub session_id: Option<String>,
    pub protocol_version: String,
    pub last_event_id: Option<String>,
    pub max_connections: usize,
    pub reconnect: bool,
    pub buffer_size: usize,
}
```

### Step 2: Implement Transport Trait
**File**: `src/transport/sse_transport.rs`

```rust
#[async_trait::async_trait]
impl Transport for SseTransport {
    async fn connect(&mut self) -> Result<(), TransportError> {
        info!("Connecting SSE transport to {}", self.config.url);
        
        // Initialize session if needed
        if let Some(ref sid) = self.config.session_id {
            self.mcp_session_id = Some(sid.clone());
            
            // Create session in manager
            self.manager.create_session(
                sid.clone(),
                self.config.protocol_version.clone()
            ).await?;
        }
        
        // Open initial GET stream for server-initiated messages
        if self.config.reconnect {
            let stream = self.client
                .open_reconnecting_stream(self.config.url.as_str())
                .await;
            
            let stream_id = stream.id();
            self.active_streams.write().await.insert(stream_id, stream.clone());
            
            // Spawn task to handle incoming events
            self.spawn_stream_handler(stream);
        }
        
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<(), TransportError> {
        info!("Disconnecting SSE transport");
        
        // Signal shutdown
        self.shutdown.notify_waiters();
        
        // Close all active streams
        let mut streams = self.active_streams.write().await;
        for (_, stream) in streams.drain() {
            stream.close().await;
        }
        
        // Clean up session
        if let Some(ref sid) = self.mcp_session_id {
            self.manager.close_session(sid).await?;
        }
        
        Ok(())
    }
    
    async fn send(&mut self, message: TransportMessage) -> Result<(), TransportError> {
        debug!("Sending message via SSE: {:?}", message);
        
        // Build headers with session context
        let mut headers = HeaderMap::new();
        if let Some(ref sid) = self.mcp_session_id {
            headers.insert("Mcp-Session-Id", HeaderValue::from_str(sid)?);
        }
        headers.insert(
            "MCP-Protocol-Version",
            HeaderValue::from_str(&self.config.protocol_version)?
        );
        
        // Send via HTTP POST
        let response = self.client
            .send_message(
                self.config.url.as_str(),
                message,
                headers
            )
            .await
            .map_err(|e| TransportError::Send(e.to_string()))?;
        
        // Handle response based on type
        match response {
            MessageResponse::Immediate(json) => {
                // Convert JSON response to TransportMessage
                if let Some(msg) = Self::json_to_transport_message(json) {
                    self.incoming_tx.send(msg).await
                        .map_err(|e| TransportError::Send(e.to_string()))?;
                }
            }
            MessageResponse::Streaming(stream) => {
                // Add to active streams and spawn handler
                let stream_id = stream.connection_id();
                self.active_streams.write().await.insert(stream_id, stream.clone());
                self.spawn_stream_handler(stream);
            }
            MessageResponse::Accepted => {
                // 202 Accepted - no response expected
                debug!("Message accepted (202)");
            }
        }
        
        Ok(())
    }
    
    async fn receive(&mut self) -> Result<Option<TransportMessage>, TransportError> {
        // Receive from the incoming channel
        match self.incoming_rx.recv().await {
            Some(msg) => Ok(Some(msg)),
            None => Ok(None),
        }
    }
    
    fn session_id(&self) -> Option<&SessionId> {
        self.session_id.as_ref()
    }
    
    async fn is_connected(&self) -> bool {
        !self.active_streams.read().await.is_empty()
    }
}
```

### Step 3: Message Conversion
**File**: `src/transport/sse_transport.rs`

```rust
impl SseTransport {
    /// Convert SSE event to TransportMessage
    fn sse_event_to_transport_message(event: &SseEvent) -> Option<TransportMessage> {
        // Parse event data as JSON
        let json: Value = serde_json::from_str(&event.data).ok()?;
        
        // Check JSON-RPC structure
        if !json.get("jsonrpc").map_or(false, |v| v == "2.0") {
            return None;
        }
        
        // Determine message type
        if let Some(method) = json.get("method").and_then(|v| v.as_str()) {
            if let Some(id) = json.get("id") {
                // Request
                Some(TransportMessage::Request {
                    id: id.clone(),
                    method: method.to_string(),
                    params: json.get("params").cloned().unwrap_or(Value::Null),
                })
            } else {
                // Notification
                Some(TransportMessage::Notification {
                    method: method.to_string(),
                    params: json.get("params").cloned().unwrap_or(Value::Null),
                })
            }
        } else if json.get("id").is_some() {
            // Response
            Some(TransportMessage::Response {
                id: json.get("id").cloned().unwrap(),
                result: json.get("result").cloned(),
                error: json.get("error").cloned(),
            })
        } else {
            None
        }
    }
    
    /// Convert JSON to TransportMessage
    fn json_to_transport_message(json: Value) -> Option<TransportMessage> {
        // Similar to sse_event_to_transport_message but for direct JSON
        if !json.get("jsonrpc").map_or(false, |v| v == "2.0") {
            return None;
        }
        
        if let Some(method) = json.get("method").and_then(|v| v.as_str()) {
            if let Some(id) = json.get("id") {
                Some(TransportMessage::Request {
                    id: id.clone(),
                    method: method.to_string(),
                    params: json.get("params").cloned().unwrap_or(Value::Null),
                })
            } else {
                Some(TransportMessage::Notification {
                    method: method.to_string(),
                    params: json.get("params").cloned().unwrap_or(Value::Null),
                })
            }
        } else if json.get("id").is_some() {
            Some(TransportMessage::Response {
                id: json.get("id").cloned().unwrap(),
                result: json.get("result").cloned(),
                error: json.get("error").cloned(),
            })
        } else {
            None
        }
    }
}
```

### Step 4: Stream Handler
**File**: `src/transport/sse_transport.rs`

```rust
impl SseTransport {
    /// Spawn a task to handle incoming SSE events
    fn spawn_stream_handler<S>(&self, mut stream: S)
    where
        S: Stream<Item = Result<SseEvent, SseError>> + Send + 'static + Unpin,
    {
        let tx = self.incoming_tx.clone();
        let shutdown = self.shutdown.clone();
        let streams = self.active_streams.clone();
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown.notified() => {
                        debug!("Stream handler shutting down");
                        break;
                    }
                    event = stream.next() => {
                        match event {
                            Some(Ok(sse_event)) => {
                                // Convert and forward
                                if let Some(msg) = Self::sse_event_to_transport_message(&sse_event) {
                                    if tx.send(msg).await.is_err() {
                                        error!("Failed to forward message, receiver dropped");
                                        break;
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                error!("SSE stream error: {}", e);
                                // Stream will handle reconnection if configured
                            }
                            None => {
                                info!("SSE stream ended");
                                break;
                            }
                        }
                    }
                }
            }
        });
    }
}
```

### Step 5: Builder Pattern
**File**: `src/transport/sse_transport.rs`

```rust
impl SseTransport {
    pub fn builder() -> SseTransportBuilder {
        SseTransportBuilder::default()
    }
}

#[derive(Default)]
pub struct SseTransportBuilder {
    url: Option<url::Url>,
    session_id: Option<String>,
    protocol_version: String,
    last_event_id: Option<String>,
    max_connections: usize,
    reconnect: bool,
    buffer_size: usize,
}

impl SseTransportBuilder {
    pub fn new() -> Self {
        Self {
            protocol_version: "2025-06-18".to_string(),
            max_connections: 10,
            reconnect: true,
            buffer_size: 1024,
            ..Default::default()
        }
    }
    
    pub fn url(mut self, url: url::Url) -> Self {
        self.url = Some(url);
        self
    }
    
    pub fn session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into());
        self
    }
    
    pub fn protocol_version(mut self, version: impl Into<String>) -> Self {
        self.protocol_version = version.into();
        self
    }
    
    pub fn reconnect(mut self, enabled: bool) -> Self {
        self.reconnect = enabled;
        self
    }
    
    pub async fn build(self) -> Result<SseTransport, TransportError> {
        let url = self.url.ok_or_else(|| {
            TransportError::Configuration("URL is required".to_string())
        })?;
        
        let config = SseTransportConfig {
            url,
            session_id: self.session_id.clone(),
            protocol_version: self.protocol_version,
            last_event_id: self.last_event_id,
            max_connections: self.max_connections,
            reconnect: self.reconnect,
            buffer_size: self.buffer_size,
        };
        
        let (tx, rx) = mpsc::channel(self.buffer_size);
        
        // Create SSE client with session-aware manager
        let manager = Arc::new(SessionAwareSseManager::new(
            config.max_connections,
            config.protocol_version.clone(),
        ));
        
        let client = Arc::new(SseHttpClient::with_manager(manager.clone()));
        
        Ok(SseTransport {
            client,
            manager,
            config,
            incoming_rx: rx,
            incoming_tx: tx,
            active_streams: Arc::new(RwLock::new(HashMap::new())),
            session_id: self.session_id.map(SessionId::from),
            mcp_session_id: self.session_id,
            shutdown: Arc::new(tokio::sync::Notify::new()),
        })
    }
}
```

## Testing Plan

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_transport_creation() {
        let transport = SseTransport::builder()
            .url("https://example.com/sse".parse().unwrap())
            .session_id("test-123")
            .build()
            .await
            .unwrap();
        
        assert_eq!(transport.session_id(), Some(&SessionId::from("test-123")));
    }
    
    #[test]
    fn test_sse_event_conversion() {
        let event = SseEvent::new(r#"{"jsonrpc":"2.0","method":"test","id":1}"#);
        let msg = SseTransport::sse_event_to_transport_message(&event);
        
        assert!(matches!(msg, Some(TransportMessage::Request { .. })));
    }
    
    #[test]
    fn test_json_conversion() {
        let json = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"value": 42},
            "id": 1
        });
        
        let msg = SseTransport::json_to_transport_message(json);
        assert!(matches!(msg, Some(TransportMessage::Response { .. })));
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_connect_disconnect() {
    let mut transport = create_test_transport().await;
    
    assert!(transport.connect().await.is_ok());
    assert!(transport.is_connected().await);
    
    assert!(transport.disconnect().await.is_ok());
    assert!(!transport.is_connected().await);
}

#[tokio::test]
async fn test_send_receive() {
    let mut transport = create_test_transport().await;
    transport.connect().await.unwrap();
    
    let msg = TransportMessage::Request {
        id: serde_json::json!(1),
        method: "test".to_string(),
        params: serde_json::json!({}),
    };
    
    transport.send(msg).await.unwrap();
    
    // Would need mock server to test receive
}
```

## Success Criteria

- [ ] Transport trait fully implemented for SSE
- [ ] Bi-directional message flow working
- [ ] Session management integrated
- [ ] Reconnection logic functioning
- [ ] All tests passing
- [ ] < 5% latency overhead measured

## Dependencies

- Transport trait definition
- Existing SSE implementation
- Session management system
- Forward proxy integration (Task 1.3)

## Notes

- Consider implementing backpressure for high message rates
- May need rate limiting for outgoing messages
- Should coordinate with Task 2.1 for server-side compatibility
- Error handling should distinguish between recoverable and fatal errors