# Unified HTTP Transport Implementation Strategy

**Created**: 2025-08-17  
**Purpose**: Detailed strategy for implementing unified HTTP transport with hyper

## Overview

This document provides the complete implementation strategy for consolidating all HTTP-based transports into a single, hyper-based implementation that handles JSON, SSE, and passthrough transparently.

## Core Design Principles

1. **Content Negotiation Over Modes**: Response type determined by Content-Type header
2. **Proxy Transparency**: Pass through what we don't understand
3. **Streaming First**: Design for streaming, buffer only when necessary
4. **Single Implementation**: One HTTP transport for all use cases

## Architecture

### Layer Structure
```
┌─────────────────────────────────────┐
│   Directional Layer (Traits)        │
│   HttpOutgoing implements           │
│   OutgoingTransport                 │
└─────────────────────────────────────┘
                ↓
┌─────────────────────────────────────┐
│   Protocol Layer                    │
│   Serialization/Deserialization     │
│   MCP message handling              │
└─────────────────────────────────────┘
                ↓
┌─────────────────────────────────────┐
│   Raw Transport Layer               │
│   HyperHttpTransport (unified)      │
│   Content-type detection            │
│   Streaming/buffering logic         │
└─────────────────────────────────────┘
                ↓
┌─────────────────────────────────────┐
│   Hyper Library                     │
│   HTTP/1.1, HTTP/2                  │
│   Connection pooling                │
└─────────────────────────────────────┘
```

## Implementation Details

### 1. Raw Transport Layer (`transport/raw/http.rs`)

```rust
use hyper::{Body, Client, Request, Response};
use hyper::body::Incoming;
use hyper_util::client::legacy::Client as LegacyClient;
use hyper_util::rt::TokioExecutor;

pub struct HyperHttpTransport {
    client: LegacyClient<HttpConnector, Body>,
    pending_response: Option<Response<Incoming>>,
    sse_buffer: Option<SseEventStream>,
}

impl HyperHttpTransport {
    pub fn new() -> Self {
        let client = LegacyClient::builder(TokioExecutor::new())
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(32)
            .http2_prior_knowledge()
            .build_http();
            
        Self {
            client,
            pending_response: None,
            sse_buffer: None,
        }
    }
    
    pub async fn send_request(&mut self, url: &str, body: Vec<u8>) -> Result<()> {
        let request = Request::post(url)
            .header("Accept", "application/json, text/event-stream, */*")
            .header("Content-Type", "application/json")
            .body(Body::from(body))?;
            
        let response = self.client.request(request).await?;
        self.pending_response = Some(response);
        Ok(())
    }
    
    pub async fn receive_response(&mut self) -> Result<TransportResponse> {
        let response = self.pending_response.take()
            .ok_or(Error::NoResponse)?;
            
        let content_type = response.headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream");
            
        // Smart content handling
        if content_type.contains("application/json") {
            self.handle_json_response(response).await
        } else if content_type.contains("text/event-stream") {
            self.handle_sse_response(response).await
        } else {
            self.handle_passthrough_response(response).await
        }
    }
}
```

### 2. Response Handling Strategies

#### JSON Response (Buffer and Parse)
```rust
async fn handle_json_response(&mut self, response: Response<Incoming>) -> Result<TransportResponse> {
    // Must buffer for JSON parsing
    let body = response.into_body()
        .collect()
        .await?
        .to_bytes();
        
    Ok(TransportResponse::Json(body.to_vec()))
}
```

#### SSE Response (Stream with Event Parsing)
```rust
async fn handle_sse_response(&mut self, response: Response<Incoming>) -> Result<TransportResponse> {
    // Don't buffer, create streaming wrapper
    let stream = SseEventStream::new(response.into_body());
    self.sse_buffer = Some(stream);
    
    // Return first event or streaming indicator
    if let Some(event) = self.sse_buffer.as_mut().unwrap().next_event().await? {
        Ok(TransportResponse::SseEvent(event))
    } else {
        Ok(TransportResponse::SseStarted)
    }
}
```

#### Passthrough Response (Forward As-Is)
```rust
async fn handle_passthrough_response(&mut self, response: Response<Incoming>) -> Result<TransportResponse> {
    // Don't interpret, just forward
    Ok(TransportResponse::Passthrough {
        content_type: content_type.to_string(),
        body: response.into_body(),
    })
}
```

### 3. Directional Layer (`transport/directional/outgoing/http.rs`)

```rust
pub struct HttpOutgoing {
    transport: HyperHttpTransport,
    protocol: Arc<dyn ProtocolHandler>,
    session_id: SessionId,
    url: String,
}

impl OutgoingTransport for HttpOutgoing {
    async fn send_request(&mut self, envelope: MessageEnvelope) -> Result<()> {
        let body = self.protocol.serialize(&envelope.message)?;
        self.transport.send_request(&self.url, body).await
    }
    
    async fn receive_response(&mut self) -> Result<MessageEnvelope> {
        match self.transport.receive_response().await? {
            TransportResponse::Json(data) => {
                let message = self.protocol.deserialize(&data)?;
                Ok(MessageEnvelope::new(message, ...))
            }
            TransportResponse::SseEvent(event) => {
                // Parse SSE event as MCP message
                let message = self.protocol.deserialize(&event.data)?;
                Ok(MessageEnvelope::new(message, ...))
            }
            TransportResponse::Passthrough { .. } => {
                // For MCP, this is an error
                // For general proxy, this would forward
                Err(Error::UnsupportedContentType)
            }
        }
    }
}
```

### 4. SSE Event Buffering Strategy

The key challenge: OutgoingTransport trait expects discrete messages, but SSE is a continuous stream.

**Solution: Internal Event Queue**

```rust
pub struct SseEventStream {
    body: Incoming,
    parser: SseParser,
    event_queue: VecDeque<SseEvent>,
}

impl SseEventStream {
    pub async fn next_event(&mut self) -> Result<Option<SseEvent>> {
        // If we have buffered events, return one
        if let Some(event) = self.event_queue.pop_front() {
            return Ok(Some(event));
        }
        
        // Otherwise, read more from stream
        while let Some(chunk) = self.body.frame().await {
            let bytes = chunk?.into_data()?;
            self.parser.push(&bytes);
            
            // Parse all complete events
            while let Some(event) = self.parser.next_event() {
                self.event_queue.push_back(event);
            }
            
            // Return first if any
            if let Some(event) = self.event_queue.pop_front() {
                return Ok(Some(event));
            }
        }
        
        Ok(None) // Stream ended
    }
}
```

## Connection Pooling

Hyper provides connection pooling, but we need to configure it properly:

```rust
pub struct ConnectionPoolConfig {
    pub max_idle_per_host: usize,      // Default: 32
    pub idle_timeout: Duration,         // Default: 90s
    pub max_http2_streams: usize,       // Default: 100
}

impl HyperHttpTransport {
    pub fn with_pool_config(config: ConnectionPoolConfig) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(config.max_idle_per_host)
            .pool_idle_timeout(config.idle_timeout)
            .http2_max_concurrent_reset_streams(config.max_http2_streams)
            .build();
        // ...
    }
}
```

## Error Handling

Hyper errors need careful handling:

```rust
fn convert_hyper_error(err: hyper::Error) -> TransportError {
    if err.is_connect() {
        TransportError::ConnectionFailed(err.to_string())
    } else if err.is_timeout() {
        TransportError::Timeout
    } else if err.is_parse() {
        TransportError::ProtocolError(err.to_string())
    } else {
        TransportError::Other(err.to_string())
    }
}
```

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_json_response_handling() {
    // Mock hyper response with JSON content-type
    // Verify correct parsing
}

#[tokio::test]
async fn test_sse_response_handling() {
    // Mock hyper response with SSE content-type
    // Verify streaming works
}

#[tokio::test]
async fn test_passthrough_response_handling() {
    // Mock hyper response with unknown content-type
    // Verify passthrough behavior
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_real_json_server() {
    // Start test HTTP server
    // Send request, verify response
}

#[tokio::test]
async fn test_real_sse_server() {
    // Start SSE test server
    // Verify streaming events
}
```

## Migration Checklist

- [ ] Back up existing implementations
- [ ] Implement HyperHttpTransport in raw layer
- [ ] Implement HttpOutgoing in directional layer
- [ ] Update factory to create unified transport
- [ ] Update reverse proxy to use new transport
- [ ] Update forward proxy to use new transport
- [ ] Delete old implementations
- [ ] Update all tests
- [ ] Update documentation

## Performance Considerations

1. **Buffer Reuse**: Use BytesMut for parsing
2. **Connection Pooling**: Tune for workload
3. **HTTP/2**: Enable when possible
4. **Streaming**: Never buffer SSE unnecessarily

## Security Considerations

1. **Header Validation**: Sanitize headers
2. **Body Size Limits**: Prevent memory exhaustion
3. **Timeout Configuration**: Prevent hanging connections
4. **TLS**: Verify certificates properly

## Future Enhancements

1. **WebSocket Support**: When MCP adds it
2. **Compression**: gzip/br support
3. **Metrics**: Connection pool stats
4. **Circuit Breaking**: For resilience

## Success Metrics

- [ ] Single HTTP implementation
- [ ] SSE streaming works correctly
- [ ] Unknown content types pass through
- [ ] Connection pooling active
- [ ] All tests passing
- [ ] ~500 lines code reduction

## Conclusion

This unified HTTP transport will:
- Eliminate code duplication
- Fix SSE issues permanently
- Provide true proxy transparency
- Simplify maintenance
- Improve performance

The key is using hyper's streaming capabilities properly and letting Content-Type drive behavior, not predetermined modes.