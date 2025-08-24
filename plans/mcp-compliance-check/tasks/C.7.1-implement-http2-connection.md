# Task C.7.1: Implement HTTP/2 Connection

**Status**: 🟡 Blocked by C.7.0  
**Duration**: 4 hours  
**Dependencies**: C.7.0 (Connection trait)  
**Priority**: HIGH - Most important for proxy performance  

## Objective

Implement HTTP/2 connection with multiplexing support for efficient proxy operations.

## Implementation

### 1. HTTP/2 Connection
```rust
// crates/mcp/src/connection/http.rs

use hyper::{Client, Body, Request, Response, Method, StatusCode};
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use tokio::sync::RwLock;
use std::sync::Arc;

pub struct Http2Connection {
    /// Shared HTTP/2 client (multiplexes internally)
    client: Arc<Client<HttpsConnector<HttpConnector>>>,
    /// Target URL
    url: Url,
    /// Session ID from headers
    session_id: RwLock<Option<String>>,
    /// Metrics
    metrics: Arc<ConnectionMetrics>,
}

impl Http2Connection {
    pub fn new(url: Url) -> io::Result<Self> {
        // Create HTTP/2 client
        let https = HttpsConnector::new();
        let client = Client::builder()
            .http2_only(true)  // Force HTTP/2
            .http2_initial_stream_window_size(1024 * 1024)  // 1MB
            .http2_initial_connection_window_size(1024 * 1024 * 10)  // 10MB
            .http2_max_concurrent_reset_streams(100)
            .build::<_, Body>(https);
        
        Ok(Self {
            client: Arc::new(client),
            url,
            session_id: RwLock::new(None),
            metrics: Arc::new(ConnectionMetrics::new()),
        })
    }
    
    async fn build_request(&self, msg: &Value) -> io::Result<Request<Body>> {
        let body = serde_json::to_vec(msg)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let mut request = Request::builder()
            .method(Method::POST)
            .uri(self.url.as_str())
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream");
        
        // Add session header if we have one
        if let Some(ref session) = *self.session_id.read().await {
            request = request.header("Mcp-Session-Id", session);
        }
        
        request.body(Body::from(body))
            .map_err(|e| io::Error::other(e))
    }
    
    async fn handle_response(&self, response: Response<Body>) -> io::Result<Value> {
        // Extract and store session ID
        if let Some(session) = response.headers().get("Mcp-Session-Id") {
            if let Ok(session_str) = session.to_str() {
                *self.session_id.write().await = Some(session_str.to_string());
            }
        }
        
        // Check status
        if !response.status().is_success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("HTTP error: {}", response.status())
            ));
        }
        
        // Parse response body
        let body = hyper::body::to_bytes(response.into_body()).await
            .map_err(|e| io::Error::other(e))?;
        
        serde_json::from_slice(&body)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

#[async_trait]
impl Connection for Http2Connection {
    async fn request(&mut self, msg: Value) -> io::Result<Value> {
        self.metrics.request_started();
        
        let request = self.build_request(&msg).await?;
        
        // HTTP/2 multiplexes this internally!
        let response = self.client.request(request).await
            .map_err(|e| io::Error::other(e))?;
        
        let result = self.handle_response(response).await;
        
        self.metrics.request_completed(&result);
        result
    }
    
    async fn notify(&mut self, msg: Value) -> io::Result<()> {
        // Notifications still need to send HTTP request
        // but we don't wait for/process response
        let request = self.build_request(&msg).await?;
        
        self.client.request(request).await
            .map_err(|e| io::Error::other(e))?;
        
        Ok(())
    }
    
    async fn receive(&mut self) -> io::Result<Option<Value>> {
        // HTTP/2 doesn't support server-initiated messages
        // (would need SSE or WebSocket upgrade)
        Ok(None)
    }
    
    fn session_id(&self) -> Option<&str> {
        // Can't return borrowed value from RwLock
        // This is a limitation we'll need to work around
        None
    }
    
    fn is_multiplexed(&self) -> bool {
        true  // HTTP/2 multiplexes
    }
}
```

### 2. SSE Support for Streaming
```rust
// crates/mcp/src/connection/http_sse.rs

pub struct Http2SseConnection {
    base: Http2Connection,
    sse_receiver: Option<SseReceiver>,
}

impl Http2SseConnection {
    async fn start_sse(&mut self, msg: Value) -> io::Result<()> {
        let request = self.base.build_request(&msg).await?;
        let response = self.base.client.request(request).await
            .map_err(|e| io::Error::other(e))?;
        
        // Check for SSE response
        let content_type = response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        
        if content_type.contains("text/event-stream") {
            self.sse_receiver = Some(SseReceiver::new(response.into_body()));
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Not an SSE response"))
        }
    }
}

#[async_trait]
impl Connection for Http2SseConnection {
    async fn receive(&mut self) -> io::Result<Option<Value>> {
        if let Some(ref mut receiver) = self.sse_receiver {
            receiver.receive().await.map(Some)
        } else {
            Ok(None)
        }
    }
    
    // Delegate other methods to base
    async fn request(&mut self, msg: Value) -> io::Result<Value> {
        self.base.request(msg).await
    }
    
    // ...
}
```

### 3. Connection Pooling
```rust
// crates/mcp/src/connection/pool.rs

use dashmap::DashMap;

pub struct Http2ConnectionPool {
    /// Connections by origin (scheme + host + port)
    connections: Arc<DashMap<String, Arc<Http2Connection>>>,
    /// Maximum connections per origin
    max_per_origin: usize,
}

impl Http2ConnectionPool {
    pub async fn get_connection(&self, url: &Url) -> io::Result<Arc<Http2Connection>> {
        let origin = format!("{}://{}", url.scheme(), url.host_str().unwrap());
        
        // Try to get existing connection
        if let Some(conn) = self.connections.get(&origin) {
            return Ok(conn.clone());
        }
        
        // Create new connection
        let conn = Arc::new(Http2Connection::new(url.clone())?);
        self.connections.insert(origin, conn.clone());
        
        Ok(conn)
    }
}
```

## Performance Considerations

### Multiplexing Benefits
```rust
// Before (Sink/Stream with workers):
// 10K requests = 10K worker tasks = 10K OS threads worth of stack

// After (HTTP/2 multiplexing):
// 10K requests = ~100 HTTP/2 connections = ~100 streams per connection
// Hyper handles multiplexing internally with zero-copy
```

### Memory Usage
```rust
// Each HTTP/2 connection:
// - 1MB stream window
// - 10MB connection window  
// - Shared across all requests to same origin

// For 100 origins: ~1.1GB total
// For 10K individual connections: ~11GB (10x worse!)
```

## Testing

```rust
#[tokio::test]
async fn test_http2_multiplexing() {
    let mut conn = Http2Connection::new(url).unwrap();
    
    // Send multiple concurrent requests
    let mut handles = vec![];
    for i in 0..100 {
        let mut conn = conn.clone();
        handles.push(tokio::spawn(async move {
            conn.request(json!({"id": i})).await
        }));
    }
    
    // All should complete without spawning workers
    for handle in handles {
        assert!(handle.await.is_ok());
    }
}

#[tokio::test]
async fn test_connection_reuse() {
    let pool = Http2ConnectionPool::new();
    
    let conn1 = pool.get_connection(&url1).await?;
    let conn2 = pool.get_connection(&url1).await?;
    
    // Should be same connection
    assert!(Arc::ptr_eq(&conn1, &conn2));
}
```

## Success Criteria

- [ ] HTTP/2 connection implements Connection trait
- [ ] Multiplexing verified with concurrent requests
- [ ] Session headers properly managed
- [ ] Connection pooling by origin
- [ ] SSE support for streaming responses
- [ ] Performance metrics collected
- [ ] No worker tasks spawned

## Notes

- Hyper handles HTTP/2 multiplexing internally
- We just need to reuse the same Client
- This is MUCH simpler than worker pattern
- Natural backpressure from async/await