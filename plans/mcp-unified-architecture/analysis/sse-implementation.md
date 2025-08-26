# SSE Implementation Guide for MCP Server

## Executive Summary

GPT-5 confirms `service_fn` works perfectly for SSE - it's just a long-lived streaming response. This guide shows how to implement production-ready SSE for MCP notifications using hyper v1.

## Core Implementation

### 1. SSE Service with StreamBody

```rust
use http_body_util::{BodyExt, StreamBody};
use hyper::body::Frame;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use futures::stream::{Stream, StreamExt};

async fn handle_request(
    req: Request<Incoming>,
    sessions: Arc<RwLock<SessionMap>>,
) -> Result<Response<BoxBody>, hyper::Error> {
    // Check for SSE upgrade
    if req.uri().path() == "/events" && accepts_sse(&req) {
        return create_sse_response(req, sessions).await;
    }
    
    // Regular JSON-RPC handling
    handle_jsonrpc(req, sessions).await
}

fn accepts_sse(req: &Request<Incoming>) -> bool {
    req.headers()
        .get(ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("text/event-stream"))
        .unwrap_or(false)
}

async fn create_sse_response(
    req: Request<Incoming>,
    sessions: Arc<RwLock<SessionMap>>,
) -> Result<Response<BoxBody>, hyper::Error> {
    let session_id = extract_session_id(&req);
    
    // Create channel for SSE messages (bounded for backpressure)
    let (tx, rx) = mpsc::channel::<Result<Frame<Bytes>, std::io::Error>>(100);
    
    // Register SSE writer with session
    {
        let mut sessions = sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.sse_writer = Some(SseWriter { tx: tx.clone() });
        }
    }
    
    // Create streaming body from receiver
    let stream = ReceiverStream::new(rx);
    let body = StreamBody::new(stream).boxed();
    
    // Build response with proper SSE headers
    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, "text/event-stream")
        .header(CACHE_CONTROL, "no-cache")
        .header(CONNECTION, "keep-alive")
        .header("X-Accel-Buffering", "no") // Disable nginx buffering
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*") // Configure CORS as needed
        .body(body)?;
    
    Ok(response)
}
```

### 2. SSE Writer for Notifications

```rust
#[derive(Clone)]
pub struct SseWriter {
    tx: mpsc::Sender<Result<Frame<Bytes>, std::io::Error>>,
}

impl SseWriter {
    /// Send MCP notification as SSE event
    pub async fn send_notification(&self, method: &str, params: Value) -> Result<()> {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        
        // Format as SSE (data: line, followed by blank line)
        let event = format!(
            "data: {}\n\n",
            serde_json::to_string(&notification)?
        );
        
        // Send as frame (Hyper handles flushing)
        let frame = Frame::data(Bytes::from(event));
        self.tx.send(Ok(frame)).await
            .map_err(|_| Error::SseDisconnected)?;
        
        Ok(())
    }
    
    /// Send SSE heartbeat/keepalive
    pub async fn send_heartbeat(&self) -> Result<()> {
        // SSE comment for keepalive
        let heartbeat = ": heartbeat\n\n";
        let frame = Frame::data(Bytes::from(heartbeat));
        
        self.tx.send(Ok(frame)).await
            .map_err(|_| Error::SseDisconnected)?;
        
        Ok(())
    }
    
    /// Send SSE retry hint
    pub async fn send_retry(&self, milliseconds: u64) -> Result<()> {
        let retry = format!("retry: {}\n\n", milliseconds);
        let frame = Frame::data(Bytes::from(retry));
        
        self.tx.send(Ok(frame)).await
            .map_err(|_| Error::SseDisconnected)?;
        
        Ok(())
    }
}
```

### 3. Server Configuration for SSE

```rust
use hyper::server::conn::http1;
use std::time::Duration;

pub async fn serve_with_sse(listener: TcpListener) -> Result<()> {
    let sessions = Arc::new(RwLock::new(HashMap::new()));
    
    loop {
        let (stream, addr) = listener.accept().await?;
        
        // Enable TCP_NODELAY for low-latency SSE events
        stream.set_nodelay(true)?;
        
        let io = TokioIo::new(stream);
        let sessions = sessions.clone();
        
        tokio::spawn(async move {
            let service = service_fn(move |req| {
                handle_request(req, sessions.clone())
            });
            
            // Configure HTTP/1.1 for SSE
            let result = http1::Builder::new()
                // Allow half-close for SSE (client closes write, we keep sending)
                .half_close(true)
                // Reasonable limits for long-lived connections
                .keep_alive(true)
                .header_read_timeout(Duration::from_secs(30))
                // No body timeout for SSE streams
                .timer(TokioTimer::new())
                // Pipeline disabled for SSE compatibility
                .pipeline_flush(false)
                .serve_connection(io, service)
                .await;
            
            if let Err(e) = result {
                // IncompleteMessage is normal when SSE client disconnects
                match e.downcast_ref::<hyper::Error>() {
                    Some(he) if he.is_incomplete_message() => {
                        debug!("SSE client disconnected normally");
                    }
                    _ => {
                        error!("Connection error: {}", e);
                    }
                }
            }
        });
    }
}
```

### 4. Heartbeat Management

```rust
/// Spawn heartbeat task for SSE connection
pub fn spawn_sse_heartbeat(
    session_id: String,
    sessions: Arc<RwLock<SessionMap>>,
    shutdown: CancellationToken,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            tokio::select! {
                _ = shutdown.cancelled() => break,
                _ = interval.tick() => {
                    let writer = {
                        let sessions = sessions.read().await;
                        sessions.get(&session_id)
                            .and_then(|s| s.sse_writer.clone())
                    };
                    
                    if let Some(writer) = writer {
                        if writer.send_heartbeat().await.is_err() {
                            // Client disconnected, clean up
                            sessions.write().await.remove(&session_id);
                            break;
                        }
                    } else {
                        // No SSE writer, exit
                        break;
                    }
                }
            }
        }
    });
}
```

### 5. HTTP/2 Support (Recommended)

```rust
use hyper::server::conn::http2;

pub async fn serve_with_http2_sse(listener: TcpListener) -> Result<()> {
    // For production, use rustls or native-tls for TLS
    let tls_acceptor = create_tls_acceptor()?;
    let sessions = Arc::new(RwLock::new(HashMap::new()));
    
    loop {
        let (tcp_stream, addr) = listener.accept().await?;
        tcp_stream.set_nodelay(true)?;
        
        // TLS handshake
        let tls_stream = tls_acceptor.accept(tcp_stream).await?;
        let io = TokioIo::new(tls_stream);
        
        let sessions = sessions.clone();
        
        tokio::spawn(async move {
            let service = service_fn(move |req| {
                handle_request(req, sessions.clone())
            });
            
            // HTTP/2 configuration for SSE
            let result = http2::Builder::new(TokioExecutor::new())
                // Allow larger windows for streaming
                .initial_stream_window_size(1024 * 1024)
                .initial_connection_window_size(2 * 1024 * 1024)
                // Reasonable concurrent stream limit
                .max_concurrent_streams(100)
                // Keep alive for long connections
                .keep_alive_interval(Some(Duration::from_secs(30)))
                .keep_alive_timeout(Duration::from_secs(60))
                .serve_connection(io, service)
                .await;
            
            if let Err(e) = result {
                debug!("HTTP/2 connection closed: {}", e);
            }
        });
    }
}
```

## WebSocket Integration

```rust
use hyper_tungstenite::{tungstenite, HyperWebsocket};

async fn handle_request_with_ws(
    mut req: Request<Incoming>,
    sessions: Arc<RwLock<SessionMap>>,
) -> Result<Response<BoxBody>, hyper::Error> {
    // Check for WebSocket upgrade
    if hyper_tungstenite::is_upgrade_request(&req) {
        let (response, websocket) = hyper_tungstenite::upgrade(&mut req, None)?;
        
        // Spawn WebSocket handler
        tokio::spawn(async move {
            if let Ok(ws) = websocket.await {
                handle_websocket(ws, sessions).await;
            }
        });
        
        return Ok(response.map(|b| b.boxed()));
    }
    
    // Check for SSE
    if req.uri().path() == "/events" && accepts_sse(&req) {
        return create_sse_response(req, sessions).await;
    }
    
    // Regular JSON-RPC
    handle_jsonrpc(req, sessions).await
}
```

## Key Implementation Points

### 1. Backpressure
- Use bounded channels (capacity 100-1000)
- Drop old events if buffer fills
- Monitor send failures

### 2. Connection Limits
```rust
// Per-client SSE limit (browsers limit ~6 HTTP/1.1 connections)
const MAX_SSE_PER_CLIENT: usize = 2;

// Global SSE connection limit
static GLOBAL_SSE_COUNT: AtomicUsize = AtomicUsize::new(0);
const MAX_GLOBAL_SSE: usize = 10000;
```

### 3. Error Handling
```rust
// Detect disconnects cleanly
match writer.send_notification(method, params).await {
    Ok(_) => {},
    Err(Error::SseDisconnected) => {
        // Normal disconnect, cleanup
        sessions.write().await.remove(&session_id);
    }
    Err(e) => {
        error!("Failed to send SSE: {}", e);
    }
}
```

### 4. Testing SSE

```rust
#[tokio::test]
async fn test_sse_notifications() {
    let server = spawn_test_server().await;
    
    // Connect SSE client
    let client = reqwest::Client::new();
    let mut event_stream = client
        .get(format!("{}/events", server.url))
        .header("Accept", "text/event-stream")
        .header("MCP-Session-Id", "test-session")
        .send()
        .await?
        .bytes_stream();
    
    // Send notification from another connection
    let notification = json!({
        "jsonrpc": "2.0",
        "method": "resources/updated",
        "params": {"uri": "file:///test.txt"}
    });
    
    server.notify_client("test-session", notification).await?;
    
    // Receive SSE event
    let event = event_stream.next().await.unwrap()?;
    assert!(event.starts_with(b"data: "));
    
    let json_part = &event[6..]; // Skip "data: "
    let received: Value = serde_json::from_slice(json_part)?;
    assert_eq!(received["method"], "resources/updated");
}
```

## Production Checklist

- [ ] **TLS Required**: Use HTTP/2 with TLS in production
- [ ] **CORS Headers**: Configure appropriately for your clients
- [ ] **Rate Limiting**: Implement per-client SSE connection limits
- [ ] **Monitoring**: Track SSE connection count, event rates
- [ ] **Heartbeats**: Send every 30-60 seconds to detect stale connections
- [ ] **Reconnection**: Include retry hints for client reconnection
- [ ] **Load Balancing**: Use sticky sessions for SSE connections
- [ ] **Proxies**: Set `X-Accel-Buffering: no` for nginx
- [ ] **Timeouts**: Disable body timeouts for SSE endpoints
- [ ] **Graceful Shutdown**: Close SSE streams cleanly on shutdown

## Browser Considerations

### HTTP/1.1 Limits
- Chrome/Firefox: ~6 connections per domain
- Solution: Use HTTP/2 or domain sharding

### Event ID Tracking
```rust
// Include event IDs for reconnection
let event = format!(
    "id: {}\ndata: {}\n\n",
    event_id,
    serde_json::to_string(&notification)?
);
```

### Last-Event-ID Header
```rust
// Resume from last event on reconnection
if let Some(last_id) = req.headers().get("Last-Event-ID") {
    // Resume from this point
}
```

## Performance Tips

1. **TCP_NODELAY**: Always enable for real-time events
2. **Buffer Sizes**: Tune based on event frequency
3. **Compression**: Consider gzip for large events (careful with real-time)
4. **Batch Events**: Group multiple notifications when possible
5. **Connection Pooling**: Reuse SSE connections across sessions

## Conclusion

SSE with hyper v1 is production-ready and efficient. The key is proper configuration:
- Use StreamBody for backpressure
- Handle disconnects gracefully
- Enable half-close and TCP_NODELAY
- Prefer HTTP/2 to avoid browser limits
- Monitor connection health with heartbeats

This approach gives us efficient server->client notifications without the complexity of full WebSocket when we only need unidirectional push.