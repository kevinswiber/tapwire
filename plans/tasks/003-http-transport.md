# Task: Implement HTTP Transport

**Status:** Not Started  
**Priority:** Medium  
**Estimated Time:** 2 days  
**Dependencies:** Transport trait, axum/hyper setup

---

## Objective

Implement HTTP transport that supports the MCP Streamable HTTP specification, including session management via headers, POST for requests, GET for streaming responses, and SSE fallback.

---

## MCP HTTP Specification Requirements

1. **Session Management**: Via `Mcp-Session-Id` header
2. **Protocol Version**: Via `MCP-Protocol-Version` header  
3. **Request Method**: POST for sending messages
4. **Response Streaming**: GET with server-sent events
5. **Content Type**: `application/json` for requests
6. **SSE Support**: For legacy compatibility

---

## Design

### Core Structure
```rust
pub struct HttpTransport {
    client: Client,
    base_url: Url,
    session_id: Option<String>,
    sse_client: Option<SseClient>,
    config: HttpTransportConfig,
    connected: bool,
}

pub struct HttpTransportConfig {
    pub timeout: Duration,
    pub max_redirects: usize,
    pub user_agent: String,
    pub enable_compression: bool,
}
```

### Message Flow
1. **Connect**: Establish HTTP client, no actual connection
2. **Send**: POST to base URL with JSON body
3. **Receive**: GET with SSE for streaming responses
4. **Close**: Close SSE connection if active

---

## Implementation Steps

### 1. Create HTTP Client Configuration
```rust
impl HttpTransport {
    pub fn new(base_url: Url) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(90))
            .http2_prior_knowledge()
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            base_url,
            session_id: None,
            sse_client: None,
            config: HttpTransportConfig::default(),
            connected: false,
        }
    }
}
```

### 2. Implement Transport Trait
```rust
#[async_trait]
impl Transport for HttpTransport {
    async fn connect(&mut self) -> TransportResult<()> {
        if self.connected {
            return Ok(());
        }
        
        // Generate session ID if not provided
        if self.session_id.is_none() {
            self.session_id = Some(Uuid::new_v4().to_string());
        }
        
        // Initialize SSE client for receiving
        let sse_url = self.base_url.clone();
        self.sse_client = Some(SseClient::new(sse_url, self.session_id.clone()));
        
        self.connected = true;
        info!("HTTP transport connected with session: {:?}", self.session_id);
        
        Ok(())
    }
}
```

### 3. Implement Send Method
```rust
async fn send(&mut self, msg: TransportMessage) -> TransportResult<()> {
    if !self.connected {
        return Err(TransportError::SendFailed("Not connected".to_string()));
    }
    
    let json_body = self.serialize_message(&msg)?;
    
    let response = self.client
        .post(self.base_url.as_str())
        .header("Content-Type", "application/json")
        .header("Mcp-Session-Id", self.session_id.as_ref().unwrap())
        .header("MCP-Protocol-Version", MCP_PROTOCOL_VERSION)
        .header("User-Agent", &self.config.user_agent)
        .body(json_body)
        .send()
        .await
        .map_err(|e| TransportError::SendFailed(e.to_string()))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(TransportError::SendFailed(
            format!("HTTP {}: {}", status, body)
        ));
    }
    
    debug!("Message sent successfully");
    Ok(())
}
```

### 4. Implement SSE Client
```rust
pub struct SseClient {
    url: Url,
    session_id: String,
    event_stream: Option<EventStream>,
    receiver: Option<mpsc::Receiver<TransportMessage>>,
}

impl SseClient {
    pub fn new(url: Url, session_id: String) -> Self {
        Self {
            url,
            session_id,
            event_stream: None,
            receiver: None,
        }
    }
    
    pub async fn connect(&mut self) -> TransportResult<()> {
        let client = EventsourceClient::new(self.url.as_str())
            .header("Mcp-Session-Id", &self.session_id)
            .header("MCP-Protocol-Version", MCP_PROTOCOL_VERSION)
            .build()
            .map_err(|e| TransportError::ConnectionFailed(e.to_string()))?;
        
        let (tx, rx) = mpsc::channel(100);
        self.receiver = Some(rx);
        
        // Spawn SSE reading task
        let mut stream = client.stream();
        tokio::spawn(async move {
            while let Some(event) = stream.next().await {
                match event {
                    Ok(Event::Message(msg)) => {
                        if let Ok(transport_msg) = parse_sse_message(&msg) {
                            let _ = tx.send(transport_msg).await;
                        }
                    }
                    Ok(Event::Error(e)) => {
                        error!("SSE error: {}", e);
                        break;
                    }
                    Err(e) => {
                        error!("Stream error: {}", e);
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
}
```

### 5. Implement Receive Method
```rust
async fn receive(&mut self) -> TransportResult<TransportMessage> {
    if !self.connected {
        return Err(TransportError::ReceiveFailed("Not connected".to_string()));
    }
    
    let sse_client = self.sse_client.as_mut()
        .ok_or_else(|| TransportError::ReceiveFailed("SSE client not initialized".to_string()))?;
    
    // Connect SSE if not already connected
    if sse_client.receiver.is_none() {
        sse_client.connect().await?;
    }
    
    let receiver = sse_client.receiver.as_mut()
        .ok_or_else(|| TransportError::ReceiveFailed("SSE receiver not available".to_string()))?;
    
    let timeout_duration = Duration::from_millis(self.config.timeout.as_millis() as u64);
    
    match timeout(timeout_duration, receiver.recv()).await {
        Ok(Some(msg)) => Ok(msg),
        Ok(None) => Err(TransportError::Closed),
        Err(_) => Err(TransportError::Timeout("Receive timeout".to_string())),
    }
}
```

### 6. Handle Legacy SSE Format
```rust
fn parse_sse_message(event: &MessageEvent) -> TransportResult<TransportMessage> {
    // Handle both standard JSON and legacy SSE formats
    let data = event.data();
    
    // Try parsing as JSON first
    if let Ok(json_value) = serde_json::from_str::<Value>(&data) {
        return parse_json_to_transport_message(json_value);
    }
    
    // Handle legacy SSE format
    if data.starts_with("data: ") {
        let json_str = data.trim_start_matches("data: ");
        if let Ok(json_value) = serde_json::from_str::<Value>(json_str) {
            return parse_json_to_transport_message(json_value);
        }
    }
    
    Err(TransportError::ProtocolError(
        format!("Invalid SSE message format: {}", data)
    ))
}
```

### 7. Implement Connection Pooling
```rust
pub struct HttpConnectionPool {
    clients: HashMap<String, Client>,
    max_connections_per_host: usize,
}

impl HttpConnectionPool {
    pub fn get_or_create(&mut self, url: &Url) -> &Client {
        let host = url.host_str().unwrap_or("unknown");
        
        self.clients.entry(host.to_string()).or_insert_with(|| {
            Client::builder()
                .pool_max_idle_per_host(self.max_connections_per_host)
                .build()
                .expect("Failed to create client")
        })
    }
}
```

---

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_http_transport_creation() {
    let transport = HttpTransport::new(Url::parse("http://localhost:3000").unwrap());
    assert_eq!(transport.transport_type(), TransportType::Http);
    assert!(!transport.is_connected());
}

#[tokio::test]
async fn test_http_message_serialization() {
    let transport = HttpTransport::new(Url::parse("http://localhost:3000").unwrap());
    
    let msg = TransportMessage::Request {
        id: "1".to_string(),
        method: "test".to_string(),
        params: json!({"key": "value"}),
    };
    
    let serialized = transport.serialize_message(&msg).unwrap();
    let parsed: Value = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(parsed["jsonrpc"], "2.0");
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_http_transport_with_mock_server() {
    // Use mockito or similar for HTTP mocking
    let mock = mockito::mock("POST", "/")
        .match_header("Mcp-Session-Id", mockito::Matcher::Any)
        .match_header("Content-Type", "application/json")
        .with_status(200)
        .create();
    
    let mut transport = HttpTransport::new(
        Url::parse(&mockito::server_url()).unwrap()
    );
    
    transport.connect().await.unwrap();
    
    let msg = TransportMessage::new_request("1", "test", json!({}));
    transport.send(msg).await.unwrap();
    
    mock.assert();
}
```

---

## Error Handling

- Network errors → TransportError::ConnectionFailed
- HTTP errors → TransportError::SendFailed with status
- SSE errors → TransportError::ReceiveFailed
- Timeout errors → TransportError::Timeout
- Invalid responses → TransportError::ProtocolError

---

## Performance Considerations

- Use connection pooling for multiple requests
- Enable HTTP/2 for multiplexing
- Consider compression for large payloads
- Implement retry logic with exponential backoff
- Monitor connection pool metrics

---

## Security Considerations

- Validate URLs before connecting
- Support TLS with certificate validation
- Never log sensitive headers or body content
- Implement request signing if needed
- Add rate limiting support

---

## Future Enhancements

- WebSocket transport as alternative
- HTTP/3 support
- Custom authentication schemes
- Request/response interceptors
- Bandwidth throttling
- Proxy support