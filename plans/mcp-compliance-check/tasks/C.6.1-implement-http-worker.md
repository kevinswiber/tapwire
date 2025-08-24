# Task C.6.1: Implement HTTP Worker Pattern

**Status**: 🔴 CRITICAL - HTTP transport doesn't work  
**Duration**: 3 hours  
**Dependencies**: C.5.4 (Sink/Stream implementation)  

## Problem

Current `HttpTransport` doesn't actually send HTTP requests:
```rust
// Current broken implementation:
fn poll_flush(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    while let Some(msg) = self.pending_requests.pop_front() {
        self.single_responses.push_back(msg); // Just moves queues!
    }
    Poll::Ready(Ok(()))
}
```

## Solution

Implement worker pattern where:
1. Worker task owns HTTP client and manages connections
2. Public Sink enqueues to bounded channel
3. Worker processes requests, sends HTTP, handles responses
4. Worker manages SSE streams and reconnection
5. Public Stream reads from response channel

## Implementation Steps

### 1. New HttpTransport Structure
```rust
pub struct HttpTransport {
    // Channels for communication with worker
    request_tx: mpsc::Sender<Value>,
    response_rx: mpsc::Receiver<io::Result<Value>>,
    
    // Worker task handle
    worker_handle: JoinHandle<()>,
    
    // Shutdown signal
    shutdown_tx: Option<oneshot::Sender<()>>,
}
```

### 2. Worker Task Implementation
```rust
struct HttpWorker {
    url: Url,
    client: Client<HttpsConnector<HttpConnector>>,
    request_rx: mpsc::Receiver<Value>,
    response_tx: mpsc::Sender<io::Result<Value>>,
    sse_streams: HashMap<String, SseStream>,
    session_id: Option<String>,
}

impl HttpWorker {
    async fn run(mut self, mut shutdown_rx: oneshot::Receiver<()>) {
        loop {
            tokio::select! {
                // Process requests
                Some(request) = self.request_rx.recv() => {
                    self.handle_request(request).await;
                }
                
                // Check SSE streams for events
                Some((id, event)) = self.poll_sse_streams() => {
                    self.handle_sse_event(id, event).await;
                }
                
                // Shutdown signal
                _ = &mut shutdown_rx => {
                    break;
                }
            }
        }
        
        // Cleanup SSE streams
        self.cleanup().await;
    }
    
    async fn handle_request(&mut self, msg: Value) {
        // Build HTTP request
        let body = serde_json::to_vec(&msg).unwrap();
        
        let mut request = Request::builder()
            .method(Method::POST)
            .uri(self.url.as_str())
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream");
            
        // Add session header if we have one
        if let Some(ref session) = self.session_id {
            request = request.header("Mcp-Session-Id", session);
        }
        
        let request = request.body(Body::from(body)).unwrap();
        
        // Send HTTP request
        match self.client.request(request).await {
            Ok(response) => {
                self.handle_response(response, msg).await;
            }
            Err(e) => {
                let _ = self.response_tx.send(Err(io::Error::other(e))).await;
            }
        }
    }
    
    async fn handle_response(&mut self, response: Response<Body>, original_msg: Value) {
        // Extract session ID from headers
        if let Some(session) = response.headers().get("Mcp-Session-Id") {
            self.session_id = Some(session.to_str().unwrap().to_string());
        }
        
        // Check content type
        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
            
        if content_type.contains("application/json") {
            // Single JSON response
            let body = to_bytes(response.into_body()).await.unwrap();
            let value: Value = serde_json::from_slice(&body).unwrap();
            let _ = self.response_tx.send(Ok(value)).await;
            
        } else if content_type.contains("text/event-stream") {
            // SSE stream - store for continuous polling
            let stream_id = original_msg.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("default")
                .to_string();
                
            let sse_stream = SseStream::new(response.into_body());
            self.sse_streams.insert(stream_id, sse_stream);
        }
    }
}
```

### 3. HttpTransport Constructor
```rust
impl HttpTransport {
    pub fn new(url: Url) -> Self {
        let (request_tx, request_rx) = mpsc::channel(100);
        let (response_tx, response_rx) = mpsc::channel(100);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        
        // Create HTTP client
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, Body>(https);
        
        // Create and spawn worker
        let worker = HttpWorker {
            url: url.clone(),
            client,
            request_rx,
            response_tx,
            sse_streams: HashMap::new(),
            session_id: None,
        };
        
        let worker_handle = tokio::spawn(async move {
            worker.run(shutdown_rx).await;
        });
        
        HttpTransport {
            request_tx,
            response_rx,
            worker_handle,
            shutdown_tx: Some(shutdown_tx),
        }
    }
}
```

### 4. Implement Sink for HttpTransport
```rust
impl Sink<Value> for HttpTransport {
    type Error = io::Error;
    
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Check if channel has capacity
        self.request_tx.poll_ready(cx)
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Worker disconnected"))
    }
    
    fn start_send(mut self: Pin<&mut Self>, item: Value) -> Result<(), Self::Error> {
        self.request_tx.try_send(item)
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Worker disconnected"))
    }
    
    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Channel handles flushing
        Poll::Ready(Ok(()))
    }
    
    fn poll_close(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Signal shutdown
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        Poll::Ready(Ok(()))
    }
}
```

### 5. Implement Stream for HttpTransport
```rust
impl Stream for HttpTransport {
    type Item = io::Result<Value>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.response_rx.poll_recv(cx)
    }
}
```

## SSE Integration

Integrate existing `transport/http/streaming/sse.rs`:
```rust
impl HttpWorker {
    fn poll_sse_streams(&mut self) -> Option<(String, SseEvent)> {
        for (id, stream) in &mut self.sse_streams {
            if let Some(event) = stream.poll_next() {
                return Some((id.clone(), event));
            }
        }
        None
    }
    
    async fn handle_sse_event(&mut self, stream_id: String, event: SseEvent) {
        match event {
            SseEvent::Message(data) => {
                if let Ok(value) = serde_json::from_str::<Value>(&data) {
                    let _ = self.response_tx.send(Ok(value)).await;
                }
            }
            SseEvent::Error(e) => {
                // Handle reconnection
                self.handle_sse_reconnect(stream_id).await;
            }
        }
    }
}
```

## Test Cases

### Test 1: Actually Sends HTTP
```rust
#[tokio::test]
async fn test_http_sends_requests() {
    let mock_server = MockServer::start().await;
    let mut transport = HttpTransport::new(mock_server.url());
    
    transport.send(json!({"method": "test"})).await.unwrap();
    
    // Verify HTTP request was made
    assert_eq!(mock_server.received_requests().len(), 1);
}
```

### Test 2: Handles SSE Streams
```rust
#[tokio::test]
async fn test_http_sse_streaming() {
    let mock_server = MockServer::start_sse().await;
    let mut transport = HttpTransport::new(mock_server.url());
    
    transport.send(json!({"method": "stream"})).await.unwrap();
    
    // Should receive multiple SSE events
    let mut events = vec![];
    for _ in 0..3 {
        if let Some(Ok(event)) = transport.next().await {
            events.push(event);
        }
    }
    
    assert_eq!(events.len(), 3);
}
```

## Success Criteria

- [ ] HTTP requests are actually sent
- [ ] JSON responses are received
- [ ] SSE streams work
- [ ] Session headers are handled
- [ ] Worker task manages lifecycle
- [ ] Clean shutdown

## Files to Modify

- `crates/mcp/src/transport/http/mod.rs` - Main implementation
- `crates/mcp/src/transport/http/streaming/sse.rs` - Integrate existing SSE
- `crates/mcp/tests/http_integration.rs` - Add tests

## References

- GPT-5 findings: Lines 29-35, 58-62
- Existing SSE code: `transport/http/streaming/sse.rs`
- Session handling: `src/session/`