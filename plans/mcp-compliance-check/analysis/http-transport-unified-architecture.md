# HTTP Transport Unified Architecture (JSON + SSE + WebSocket)

## Executive Summary

The HTTP transport in MCP is actually **ONE transport with THREE response modes**:
1. **JSON** - `200 OK` with `application/json` (single response)
2. **SSE** - `200 OK` with `text/event-stream` (streaming responses)
3. **WebSocket** - `101 Switching Protocols` (bidirectional streaming)

All three start with an HTTP request. The server chooses the response mode based on the operation and capabilities.

## The Unified Protocol Flow

```
Client                              Server
  |                                   |
  |------------ HTTP POST ----------->|
  |   Accept: application/json,       |
  |           text/event-stream       |
  |   Connection: Upgrade             |
  |   Upgrade: websocket              |
  |   Body: {"jsonrpc":"2.0"...}      |
  |                                   |
  |<----------- Response --------------|
  |                                   |
  | Server chooses ONE of:            |
  |                                   |
  | (1) 200 OK                        |
  |     Content-Type: application/json|
  |     Body: {"jsonrpc":"2.0"...}    |
  |                                   |
  | (2) 200 OK                        |
  |     Content-Type: text/event-stream
  |     data: {"jsonrpc":"2.0"...}    |
  |     data: {"jsonrpc":"2.0"...}    |
  |                                   |
  | (3) 101 Switching Protocols       |
  |     Upgrade: websocket            |
  |     [WebSocket frames follow]     |
  |                                   |
```

## Implementation Architecture

### Single HttpTransport Handling All Three Modes

```rust
pub struct HttpTransport {
    client: hyper::Client<HttpsConnector>,
    url: Url,
    
    // Request queue
    pending_requests: VecDeque<JsonRpcMessage>,
    
    // Response handling for all three modes
    single_responses: VecDeque<JsonRpcMessage>,      // JSON responses
    sse_streams: HashMap<RequestId, SseStream>,      // SSE streams
    ws_connection: Option<WebSocketStream>,          // WebSocket (if upgraded)
    
    // State
    transport_mode: TransportMode,
}

enum TransportMode {
    RequestResponse,    // Default: each request gets a response
    Upgraded(UpgradeType),  // After 101 or SSE response
}

enum UpgradeType {
    Sse,       // Streaming via SSE
    WebSocket, // Full duplex via WebSocket
}

impl HttpTransport {
    async fn send_request(&mut self, msg: JsonRpcMessage) -> Result<()> {
        // If already upgraded to WebSocket, send via WebSocket
        if let Some(ref mut ws) = self.ws_connection {
            ws.send(Message::text(serde_json::to_string(&msg)?)).await?;
            return Ok(());
        }
        
        // Otherwise, send via HTTP POST
        let request = Request::post(&self.url)
            .header("Accept", "application/json, text/event-stream")
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Key", generate_key())
            .header("Sec-WebSocket-Version", "13")
            .body(Body::from(serde_json::to_vec(&msg)?))?;
        
        let response = self.client.request(request).await?;
        
        // Handle response based on status and headers
        match response.status() {
            StatusCode::OK => {
                match response.headers().get("content-type").map(|v| v.to_str()) {
                    Some(Ok("application/json")) => {
                        // Single JSON response
                        let body = hyper::body::to_bytes(response.into_body()).await?;
                        let msg: JsonRpcMessage = serde_json::from_slice(&body)?;
                        self.single_responses.push_back(msg);
                    }
                    Some(Ok("text/event-stream")) => {
                        // SSE stream
                        let stream = SseStream::new(response.into_body());
                        self.sse_streams.insert(msg.id, stream);
                        self.transport_mode = TransportMode::Upgraded(UpgradeType::Sse);
                    }
                    _ => return Err(TransportError::InvalidContentType),
                }
            }
            StatusCode::SWITCHING_PROTOCOLS => {
                // WebSocket upgrade
                let upgraded = hyper::upgrade::on(response).await?;
                let ws_stream = WebSocketStream::from_raw_socket(
                    upgraded,
                    Role::Client,
                    None
                ).await;
                self.ws_connection = Some(ws_stream);
                self.transport_mode = TransportMode::Upgraded(UpgradeType::WebSocket);
            }
            _ => return Err(TransportError::UnexpectedStatus),
        }
        Ok(())
    }
}

impl Stream for HttpTransport {
    type Item = Result<JsonRpcMessage, TransportError>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Priority 1: Single JSON responses
        if let Some(msg) = self.single_responses.pop_front() {
            return Poll::Ready(Some(Ok(msg)));
        }
        
        // Priority 2: WebSocket messages (if upgraded)
        if let Some(ref mut ws) = self.ws_connection {
            if let Poll::Ready(Some(msg)) = ws.poll_next_unpin(cx) {
                match msg {
                    Ok(Message::Text(text)) => {
                        let json_msg: JsonRpcMessage = serde_json::from_str(&text)?;
                        return Poll::Ready(Some(Ok(json_msg)));
                    }
                    Ok(Message::Close(_)) => {
                        self.ws_connection = None;
                        self.transport_mode = TransportMode::RequestResponse;
                    }
                    _ => {}
                }
            }
        }
        
        // Priority 3: SSE streams
        for stream in self.sse_streams.values_mut() {
            if let Poll::Ready(Some(event)) = stream.poll_next_unpin(cx) {
                match parse_sse_event(event) {
                    Ok(msg) => return Poll::Ready(Some(Ok(msg))),
                    Err(e) => return Poll::Ready(Some(Err(e))),
                }
            }
        }
        
        Poll::Pending
    }
}

impl Sink<JsonRpcMessage> for HttpTransport {
    type Error = TransportError;
    
    fn start_send(mut self: Pin<&mut Self>, msg: JsonRpcMessage) -> Result<(), Self::Error> {
        // If WebSocket is active, send directly
        if let Some(ref mut ws) = self.ws_connection {
            let text = serde_json::to_string(&msg)?;
            Pin::new(ws).start_send(Message::Text(text))?;
        } else {
            // Otherwise queue for HTTP POST
            self.pending_requests.push_back(msg);
        }
        Ok(())
    }
    
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Flush WebSocket if active
        if let Some(ref mut ws) = self.ws_connection {
            return Pin::new(ws).poll_flush(cx).map_err(Into::into);
        }
        
        // Send pending HTTP requests
        while let Some(msg) = self.pending_requests.pop_front() {
            // This would need proper async handling
            self.get_mut().send_request(msg);
        }
        Poll::Ready(Ok(()))
    }
}
```

## Module Organization (Revised)

```rust
mcp::transports::{
    stdio,          // Line-delimited JSON over stdin/stdout
    subprocess,     // Line-delimited JSON over child process stdio
    http,           // HTTP transport with three modes: JSON, SSE, WebSocket
}

// Internal implementation modules:
mcp::transports::http::{
    transport,      // Main HttpTransport implementation
    sse,            // SSE event parsing
    websocket,      // WebSocket upgrade handling
    response,       // Response mode detection and handling
}
```

## Key Insights

### Why This Makes Sense

1. **Protocol Continuity**
   - All three modes start with HTTP POST
   - Server decides response mode based on operation
   - Client doesn't choose - it adapts to server's choice

2. **Progressive Enhancement**
   - Simple operations → JSON response
   - Streaming operations → SSE response  
   - Full duplex needed → WebSocket upgrade

3. **Single Transport Interface**
   - User just specifies "http" transport
   - Implementation handles all three modes transparently
   - Same `Sink + Stream` interface regardless of mode

### State Machine

```
                  ┌─────────────┐
                  │   Initial   │
                  │   (HTTP)    │
                  └──────┬──────┘
                         │
                    HTTP Request
                         │
           ┌─────────────┼─────────────┐
           │             │             │
      200 + JSON    200 + SSE    101 WebSocket
           │             │             │
           ▼             ▼             ▼
    ┌────────────┐ ┌──────────┐ ┌────────────┐
    │   Stay     │ │   SSE    │ │ WebSocket  │
    │   HTTP     │ │  Active  │ │   Active   │
    └────────────┘ └──────────┘ └────────────┘
                                       │
                              All future messages
                                 via WebSocket
```

## Comparison with Previous Understanding

### What We Thought
- HTTP, SSE, and WebSocket were separate transports
- User would choose which transport to use
- WebSocket was completely independent

### What's Actually True
- HTTP is ONE transport with three response modes
- Server chooses the mode based on the operation
- WebSocket is just another upgrade path like SSE

## Implementation Considerations

### Connection Management
```rust
impl HttpTransport {
    pub fn new(url: Url) -> Self {
        Self {
            client: hyper::Client::new(),
            url,
            transport_mode: TransportMode::RequestResponse,
            // ... other fields
        }
    }
    
    pub fn is_upgraded(&self) -> bool {
        matches!(self.transport_mode, TransportMode::Upgraded(_))
    }
    
    pub fn upgrade_type(&self) -> Option<&UpgradeType> {
        if let TransportMode::Upgraded(ref upgrade) = self.transport_mode {
            Some(upgrade)
        } else {
            None
        }
    }
}
```

### Error Handling
- JSON mode: Request errors are immediate
- SSE mode: Connection errors may occur later
- WebSocket mode: Must handle disconnections gracefully

### Testing
```rust
#[tokio::test]
async fn test_http_transport_modes() {
    // Mock server that responds differently based on method
    let server = MockServer::new();
    
    // Simple method → JSON response
    server.expect_request("simple_method")
        .return_json(json!({"result": "ok"}));
    
    // Streaming method → SSE response
    server.expect_request("streaming_method")
        .return_sse(vec![
            json!({"progress": 0.5}),
            json!({"progress": 1.0}),
        ]);
    
    // Interactive method → WebSocket upgrade
    server.expect_request("interactive_method")
        .upgrade_to_websocket();
    
    let mut transport = HttpTransport::new(server.url());
    // Test all three modes with same transport...
}
```

## Benefits of Unified Approach

1. **Simpler API**
   - One transport type to configure
   - Automatic adaptation to server capabilities
   - No need to choose transport mode upfront

2. **Better Abstraction**
   - HTTP/SSE/WebSocket details hidden
   - Just send and receive JsonRpcMessages
   - Transport handles protocol negotiation

3. **Future Proof**
   - Easy to add new HTTP-based modes
   - Could support HTTP/2 server push
   - Could support HTTP/3 when available

## Conclusion

The HTTP transport is actually a **sophisticated adaptive transport** that starts with HTTP and can upgrade to either SSE or WebSocket based on the server's response. This is not three transports but one transport with three modes of operation.

This design makes sense because:
- All three use HTTP for initial negotiation
- Server controls the communication mode
- Client transparently handles all modes
- Single configuration point for users

---

*Document created: 2025-08-24*  
*Key insight: HTTP, SSE, and WebSocket are three modes of one transport, not three transports*