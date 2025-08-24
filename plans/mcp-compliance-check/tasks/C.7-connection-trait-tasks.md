# C.7 - Connection Pattern Implementation Tasks

**Created**: 2025-08-24  
**Status**: IN PROGRESS  
**Prerequisites**: Hyper 1.7 upgrade (✅ COMPLETE)

## Overview

Implement the Connection trait pattern to replace Sink/Stream, providing zero-overhead async communication with natural protocol support for multiplexing and pooling.

## Task Breakdown

### C.7.0 - Create Connection Trait (2 hours) 🚧 IN PROGRESS

**Objective**: Define the core Connection trait and migration adapters

**Implementation**:
```rust
// crates/mcp/src/connection/mod.rs
#[async_trait]
pub trait Connection: Send + Sync {
    /// Send a message through the connection
    async fn send(&mut self, message: Value) -> Result<()>;
    
    /// Receive a message from the connection
    async fn receive(&mut self) -> Result<Value>;
    
    /// Close the connection gracefully
    async fn close(&mut self) -> Result<()>;
    
    /// Check if connection is healthy (for pooling)
    fn is_healthy(&self) -> bool {
        true
    }
    
    /// Get protocol type (for routing)
    fn protocol(&self) -> Protocol {
        Protocol::Unknown
    }
}

// Migration adapter for existing Sink/Stream transports
pub struct SinkStreamAdapter<T: Sink<Value> + Stream<Item = Result<Value>>> {
    inner: T,
}
```

**Success Criteria**:
- [ ] Connection trait defined with async methods
- [ ] SinkStreamAdapter for gradual migration
- [ ] Protocol enum for routing decisions
- [ ] Tests for adapter functionality

---

### C.7.1 - Implement HTTP/2 Connection (4 hours) 🔴 CRITICAL

**Objective**: Create HTTP/2 connection using hyper 1.7's direct connection management

**Implementation**:
```rust
// crates/mcp/src/connection/http.rs
pub struct Http2Connection {
    sender: http2::SendRequest<Full<Bytes>>,
    url: Url,
    session_id: Option<String>,
    metrics: Arc<ConnectionMetrics>,
}

impl Http2Connection {
    pub async fn connect(url: Url) -> Result<Self> {
        // Use hyper 1.7 direct connection
        // TLS via rustls
        // HTTP/2 handshake
        // Drive connection task
    }
}

#[async_trait]
impl Connection for Http2Connection {
    async fn send(&mut self, message: Value) -> Result<()> {
        // Build HTTP request
        // Send via http2::SendRequest
        // Handle response
    }
    
    async fn receive(&mut self) -> Result<Value> {
        // For SSE: stream events
        // For JSON: single response
    }
}
```

**Key Features**:
- HTTP/2 multiplexing support
- SSE streaming for server events
- Session ID management
- Ready for shadowcat pooling

**Success Criteria**:
- [ ] HTTP/2 connection establishment
- [ ] Request/response handling
- [ ] SSE streaming support
- [ ] Session affinity via headers
- [ ] Integration tests

---

### C.7.2 - Implement WebSocket Connection (3 hours) 🟡 HIGH

**Objective**: Create WebSocket connection for bidirectional communication

**Implementation**:
```rust
// crates/mcp/src/connection/websocket.rs
pub struct WebSocketConnection {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    session_id: String,
}

#[async_trait]
impl Connection for WebSocketConnection {
    async fn send(&mut self, message: Value) -> Result<()> {
        let text = serde_json::to_string(&message)?;
        self.ws.send(Message::Text(text)).await?;
        Ok(())
    }
    
    async fn receive(&mut self) -> Result<Value> {
        match self.ws.next().await {
            Some(Ok(Message::Text(text))) => {
                Ok(serde_json::from_str(&text)?)
            }
            // Handle other message types
        }
    }
}
```

**Success Criteria**:
- [ ] WebSocket connection establishment
- [ ] Bidirectional messaging
- [ ] Ping/pong handling
- [ ] Clean shutdown
- [ ] Error recovery

---

### C.7.3 - Implement Stdio Connection (2 hours) 🟢 NORMAL

**Objective**: Simple stdio connection wrapper

**Implementation**:
```rust
// crates/mcp/src/connection/stdio.rs
pub struct StdioConnection {
    stdin: FramedWrite<tokio::io::Stdout, JsonCodec>,
    stdout: FramedRead<tokio::io::Stdin, JsonCodec>,
}

#[async_trait]
impl Connection for StdioConnection {
    async fn send(&mut self, message: Value) -> Result<()> {
        self.stdin.send(message).await
    }
    
    async fn receive(&mut self) -> Result<Value> {
        self.stdout.next().await
            .ok_or(Error::Closed)?
    }
}
```

**Success Criteria**:
- [ ] Stdio read/write
- [ ] JSON line protocol
- [ ] Process lifecycle management
- [ ] Tests with mock process

---

### C.7.4 - Migrate Client/Server (3 hours) 🟡 HIGH

**Objective**: Update Client and Server to use Connection trait

**Implementation**:
```rust
// crates/mcp/src/client.rs
pub struct Client<C: Connection, H: Handler> {
    connection: C,
    handler: H,
    pending: HashMap<String, oneshot::Sender<Value>>,
}

impl<C: Connection, H: Handler> Client<C, H> {
    pub async fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = Uuid::new_v4().to_string();
        let request = /* build request */;
        
        self.connection.send(request).await?;
        
        // No worker needed - direct receive
        loop {
            let response = self.connection.receive().await?;
            // Route response
        }
    }
}
```

**Success Criteria**:
- [ ] Client uses Connection trait
- [ ] Server uses Connection trait
- [ ] No worker tasks
- [ ] Request/response correlation
- [ ] All tests pass

---

### C.7.5 - Integrate Shadowcat Pool (2 hours) 🟡 HIGH

**Objective**: Make connections poolable using shadowcat's generic pool

**Implementation**:
```rust
// crates/mcp/src/connection/poolable.rs
use shadowcat::pool::PoolableResource;

pub struct PoolableConnection {
    inner: Box<dyn Connection>,
    created_at: Instant,
    request_count: Arc<AtomicU64>,
}

#[async_trait]
impl PoolableResource for PoolableConnection {
    async fn is_healthy(&self) -> bool {
        self.inner.is_healthy()
    }
    
    fn resource_id(&self) -> String {
        format!("{}-{}", self.inner.protocol(), self.created_at)
    }
}

// Protocol-specific pooling strategies
pub enum PoolStrategy {
    PerOrigin { max: usize },      // HTTP/2
    PerSession,                     // WebSocket
    Singleton,                      // Stdio
}
```

**Success Criteria**:
- [ ] PoolableResource implementation
- [ ] Protocol-specific strategies
- [ ] Pool configuration
- [ ] Health checks
- [ ] Metrics integration

---

## Testing Strategy

1. **Unit Tests**: Each connection type individually
2. **Integration Tests**: Client/Server with each connection
3. **Pool Tests**: Connection reuse and health checks
4. **Performance Tests**: Verify zero overhead vs Sink/Stream
5. **Compliance Tests**: MCP validator with new transports

## Migration Path

1. **Phase 1**: Connection trait + adapters (keep Sink/Stream working)
2. **Phase 2**: Native Connection implementations
3. **Phase 3**: Migrate Client/Server
4. **Phase 4**: Remove Sink/Stream code
5. **Phase 5**: Shadowcat pool integration

## Performance Targets

- Connection overhead: < 1µs (vs 20µs with workers)
- HTTP/2 multiplexing: 100+ streams per connection
- Pool acquire: < 100µs
- Memory per connection: < 50KB

## Files to Create/Modify

**New Files**:
- `crates/mcp/src/connection/mod.rs` - Trait definition
- `crates/mcp/src/connection/http.rs` - HTTP/2 implementation
- `crates/mcp/src/connection/websocket.rs` - WebSocket implementation
- `crates/mcp/src/connection/stdio.rs` - Stdio implementation
- `crates/mcp/src/connection/poolable.rs` - Pool integration

**Modified Files**:
- `crates/mcp/src/client.rs` - Use Connection trait
- `crates/mcp/src/server.rs` - Use Connection trait
- `crates/mcp/src/transport/` - Mark as deprecated

## Dependencies

```toml
[dependencies]
async-trait = "0.1"
hyper = { version = "1.7", features = ["client", "http1", "http2"] }
hyper-util = "0.1"
tokio-tungstenite = "0.20"
# shadowcat pool already available
```

## Success Metrics

1. ✅ All existing tests pass with new implementation
2. ✅ Zero worker tasks in production
3. ✅ < 1µs overhead per message
4. ✅ HTTP/2 multiplexing verified
5. ✅ Connection pooling working
6. ✅ MCP validator compliance