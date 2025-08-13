# Phase 1 - Task 1.2: SSE Connection Management

## Task Overview
Implement persistent SSE connection management for the MCP Streamable HTTP transport, handling multiple concurrent streams, connection lifecycle, and proper resource cleanup.

**Duration**: 4-5 hours
**Priority**: CRITICAL - Required for SSE-based communication
**Dependencies**: Task 1.1 (SSE Event Parser) must be complete

## Objectives

### Primary Goals
1. Create SSE connection manager for HTTP client
2. Handle POST requests that return SSE streams
3. Manage GET requests for server-initiated streams
4. Support multiple concurrent SSE connections
5. Implement proper connection lifecycle and cleanup

### Success Criteria
- [ ] Can establish SSE connections via POST and GET
- [ ] Properly parse Content-Type headers for SSE detection
- [ ] Handle both single JSON responses and SSE streams
- [ ] Support multiple simultaneous connections per session
- [ ] Clean connection shutdown without resource leaks
- [ ] Error handling for network failures
- [ ] Connection state tracking and monitoring
- [ ] Integration with existing HTTP transport layer
- [ ] Comprehensive test coverage

## Technical Requirements

### MCP Streamable HTTP Requirements
From the specification:

1. **POST Requests**:
   - Must include `Accept: application/json, text/event-stream`
   - Server returns either JSON or SSE stream
   - SSE stream should remain open until response sent
   - Handle 202 Accepted for notifications/responses

2. **GET Requests**:
   - Must include `Accept: text/event-stream`
   - Used for server-initiated communication
   - May remain open indefinitely
   - Handle 405 Method Not Allowed gracefully

3. **Multiple Connections**:
   - Client MAY maintain multiple SSE streams
   - Server MUST NOT broadcast same message to multiple streams
   - Each stream is independent

4. **Session Management**:
   - Include `Mcp-Session-Id` header if present
   - Include `MCP-Protocol-Version` header

## Implementation Plan

### Module Structure
```
src/transport/sse/
├── connection.rs     # SSE connection handling
├── manager.rs        # Connection pool management
├── client.rs         # HTTP client integration
└── tests/
    ├── connection.rs # Connection tests
    └── manager.rs    # Manager tests
```

### Core Components

#### 1. SSE Connection (`connection.rs`)
```rust
use hyper::{Body, Response};
use futures::stream::Stream;

pub struct SseConnection {
    id: Uuid,
    stream: Pin<Box<dyn Stream<Item = Result<SseEvent, SseError>> + Send>>,
    session_id: Option<String>,
    last_event_id: Option<String>,
    created_at: Instant,
    state: ConnectionState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Reconnecting,
    Closed,
    Failed(String),
}

impl SseConnection {
    pub async fn from_response(response: Response<Body>) -> Result<Self, SseError>;
    pub async fn next_event(&mut self) -> Option<Result<SseEvent, SseError>>;
    pub fn close(&mut self);
    pub fn is_alive(&self) -> bool;
}
```

#### 2. Connection Manager (`manager.rs`)
```rust
pub struct SseConnectionManager {
    connections: Arc<RwLock<HashMap<Uuid, SseConnection>>>,
    http_client: Arc<HttpClient>,
    max_connections: usize,
    session_id: Option<String>,
    protocol_version: String,
}

impl SseConnectionManager {
    pub fn new(http_client: Arc<HttpClient>) -> Self;
    
    pub async fn post_request(
        &self,
        url: &str,
        body: JsonRpcMessage,
    ) -> Result<SseResponse, SseError>;
    
    pub async fn open_stream(
        &self,
        url: &str,
        last_event_id: Option<String>,
    ) -> Result<Uuid, SseError>;
    
    pub fn get_connection(&self, id: Uuid) -> Option<SseConnection>;
    pub fn close_connection(&self, id: Uuid);
    pub fn close_all(&self);
    pub fn active_connections(&self) -> usize;
}

pub enum SseResponse {
    Json(serde_json::Value),
    Stream(Uuid),  // Connection ID for streaming response
}
```

#### 3. HTTP Client Integration (`client.rs`)
```rust
pub struct SseHttpClient {
    inner: hyper::Client<HttpsConnector<HttpConnector>>,
    manager: Arc<SseConnectionManager>,
}

impl SseHttpClient {
    pub fn new() -> Self;
    
    pub async fn send_message(
        &self,
        url: &str,
        message: JsonRpcMessage,
        headers: HeaderMap,
    ) -> Result<MessageResponse, SseError>;
    
    async fn handle_response(
        &self,
        response: Response<Body>,
    ) -> Result<MessageResponse, SseError>;
    
    fn build_headers(
        &self,
        session_id: Option<&str>,
        protocol_version: &str,
        last_event_id: Option<&str>,
    ) -> HeaderMap;
}

pub enum MessageResponse {
    Immediate(serde_json::Value),
    Streaming(SseConnectionStream),
    Accepted,  // 202 for notifications
}

pub struct SseConnectionStream {
    connection_id: Uuid,
    manager: Arc<SseConnectionManager>,
}

impl Stream for SseConnectionStream {
    type Item = Result<SseEvent, SseError>;
    // Stream implementation
}
```

### Connection Lifecycle

1. **Connection Establishment**:
   ```
   POST/GET Request → Check Content-Type → 
   If SSE: Create SseConnection → Add to Manager → Return Stream
   If JSON: Parse and return immediately
   ```

2. **Event Processing**:
   ```
   While connection active:
     Read from stream → Parse with SseParser → 
     Emit events → Update last_event_id
   ```

3. **Connection Cleanup**:
   ```
   On error/close:
     Mark connection as closed → Remove from manager →
     Clean up resources → Notify listeners
   ```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum SseConnectionError {
    #[error("Failed to establish connection: {0}")]
    ConnectionFailed(String),
    
    #[error("Invalid response format: expected SSE but got {0}")]
    InvalidContentType(String),
    
    #[error("Connection closed unexpectedly")]
    UnexpectedClose,
    
    #[error("Maximum connections exceeded: {max}")]
    TooManyConnections { max: usize },
    
    #[error("HTTP error: {0}")]
    Http(#[from] hyper::Error),
    
    #[error("Parse error: {0}")]
    Parse(#[from] SseError),
}
```

## Test Cases

### Unit Tests

1. **Connection Creation**:
   - From SSE response
   - From JSON response
   - Invalid content type handling

2. **Stream Management**:
   - Open multiple streams
   - Close individual streams
   - Close all streams
   - Connection limit enforcement

3. **Header Management**:
   - Session ID inclusion
   - Protocol version header
   - Last-Event-ID for resumption
   - Accept header formatting

4. **Response Handling**:
   - 200 with JSON
   - 200 with SSE
   - 202 Accepted
   - 405 Method Not Allowed
   - Network errors

### Integration Tests

1. **End-to-End Flow**:
   ```rust
   #[tokio::test]
   async fn test_post_with_sse_response() {
       let manager = create_manager();
       let response = manager.post_request(url, message).await?;
       
       match response {
           SseResponse::Stream(id) => {
               let conn = manager.get_connection(id);
               let event = conn.next_event().await?;
               assert_eq!(event.data, expected);
           }
           _ => panic!("Expected stream response"),
       }
   }
   ```

2. **Multiple Connections**:
   - Open 10 concurrent connections
   - Verify independent message delivery
   - Clean shutdown

3. **Connection Recovery**:
   - Simulate network failure
   - Verify cleanup
   - Attempt reconnection

## Performance Considerations

1. **Connection Pooling**: Reuse HTTP connections where possible
2. **Buffering**: Efficient buffer sizes for SSE streams
3. **Backpressure**: Handle slow consumers gracefully
4. **Resource Limits**: Cap maximum connections per session
5. **Timeout Management**: Configurable timeouts for idle connections

## Dependencies

```toml
[dependencies]
hyper = { version = "1", features = ["client", "http1", "http2"] }
hyper-tls = "0.6"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
uuid = { version = "1", features = ["v4"] }
parking_lot = "0.12"
```

## Integration Points

1. **SSE Parser**: Use Task 1.1's parser for event processing
2. **Session Manager**: Associate connections with sessions
3. **HTTP Transport**: Extend existing HTTP transport
4. **Protocol Module**: Use version management from Phase 0
5. **Metrics**: Track connection statistics

## Configuration

```rust
pub struct SseConfig {
    pub max_connections_per_session: usize,  // Default: 10
    pub connection_timeout: Duration,        // Default: 30s
    pub idle_timeout: Duration,              // Default: 5 minutes
    pub buffer_size: usize,                  // Default: 8KB
    pub max_event_size: usize,               // Default: 10MB
}
```

## Metrics to Track

- Active connections count
- Connection duration
- Events per connection
- Bytes transferred
- Connection failures
- Reconnection attempts

## Next Steps

After completing this task:
1. Task 1.3: Implement automatic reconnection
2. Task 1.4: Integrate with session management
3. Task 1.5: Performance optimization

## Notes

- Ensure thread-safe connection management
- Handle partial reads gracefully
- Implement connection health checks
- Consider connection warmup strategies
- Document connection limits and timeouts