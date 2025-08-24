# Task C.7.0: Create Connection Trait and Core Types

**Status**: 🟢 Ready to implement  
**Duration**: 2 hours  
**Dependencies**: Architecture decision (complete)  
**Priority**: CRITICAL - Foundation for new architecture  

## Objective

Define the core `Connection` trait and supporting types that will replace Sink/Stream pattern.

## Implementation

### 1. Core Connection Trait
```rust
// crates/mcp/src/connection/mod.rs

use async_trait::async_trait;
use serde_json::Value;
use std::io;

/// A connection to an MCP server or client
#[async_trait]
pub trait Connection: Send + Sync {
    /// Send request and await response
    async fn request(&mut self, msg: Value) -> io::Result<Value>;
    
    /// Send one-way notification
    async fn notify(&mut self, msg: Value) -> io::Result<()>;
    
    /// Receive server-initiated message (if supported)
    async fn receive(&mut self) -> io::Result<Option<Value>>;
    
    /// Get session ID if connection has one
    fn session_id(&self) -> Option<&str> {
        None
    }
    
    /// Check if connection supports multiplexing
    fn is_multiplexed(&self) -> bool {
        false
    }
    
    /// Check if connection is bidirectional
    fn is_bidirectional(&self) -> bool {
        false
    }
}
```

### 2. Protocol Selection
```rust
// crates/mcp/src/connection/protocol.rs

#[derive(Debug, Clone, Copy)]
pub enum Protocol {
    Stdio,
    Http,
    Http2,
    WebSocket,
    Tcp,
    Unix,
}

pub struct ProtocolStrategy {
    preferred: Vec<Protocol>,
    fallback: Protocol,
}

impl ProtocolStrategy {
    pub fn select(&self, url: &Url) -> Protocol {
        // Parse URL scheme and capabilities
        match url.scheme() {
            "http" | "https" => {
                // Check for HTTP/2 support
                if self.supports_http2(url) {
                    Protocol::Http2
                } else {
                    Protocol::Http
                }
            }
            "ws" | "wss" => Protocol::WebSocket,
            "tcp" => Protocol::Tcp,
            "unix" => Protocol::Unix,
            "stdio" | "-" => Protocol::Stdio,
            _ => self.fallback,
        }
    }
}
```

### 3. Connection Manager
```rust
// crates/mcp/src/connection/manager.rs

use std::sync::Arc;
use dashmap::DashMap;

pub struct ConnectionManager {
    /// Connections by URL origin (for HTTP/2 multiplexing)
    connections: Arc<DashMap<String, Box<dyn Connection>>>,
    /// Protocol selection strategy
    strategy: ProtocolStrategy,
    /// Connection pool config
    config: ConnectionConfig,
}

impl ConnectionManager {
    pub async fn get_connection(&self, url: &Url) -> io::Result<Box<dyn Connection>> {
        let protocol = self.strategy.select(url);
        
        match protocol {
            Protocol::Http2 if self.can_multiplex(url) => {
                // Reuse existing HTTP/2 connection
                self.get_or_create_http2(url).await
            }
            Protocol::WebSocket => {
                // Create new WebSocket (not pooled)
                self.create_websocket(url).await
            }
            Protocol::Stdio => {
                // Singleton stdio
                self.get_stdio().await
            }
            _ => {
                // Create new connection
                self.create_connection(url, protocol).await
            }
        }
    }
}
```

### 4. Adapter for Migration
```rust
// crates/mcp/src/connection/adapter.rs

/// Adapter to use old Sink+Stream transports with new Connection trait
pub struct TransportAdapter<T> {
    transport: Arc<Mutex<T>>,
}

#[async_trait]
impl<T> Connection for TransportAdapter<T>
where
    T: Sink<Value, Error = io::Error> + Stream<Item = io::Result<Value>> + Unpin + Send + 'static,
{
    async fn request(&mut self, msg: Value) -> io::Result<Value> {
        let mut transport = self.transport.lock().await;
        
        // Send request
        transport.send(msg).await?;
        
        // Wait for response
        match transport.next().await {
            Some(Ok(response)) => Ok(response),
            Some(Err(e)) => Err(e),
            None => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed")),
        }
    }
    
    async fn notify(&mut self, msg: Value) -> io::Result<()> {
        let mut transport = self.transport.lock().await;
        transport.send(msg).await
    }
    
    async fn receive(&mut self) -> io::Result<Option<Value>> {
        let mut transport = self.transport.lock().await;
        transport.next().await.transpose()
    }
}
```

## File Structure

```
crates/mcp/src/
├── connection/
│   ├── mod.rs           # Connection trait
│   ├── protocol.rs      # Protocol selection
│   ├── manager.rs       # Connection management
│   ├── adapter.rs       # Migration adapter
│   ├── http.rs         # HTTP implementation (C.7.1)
│   ├── websocket.rs    # WebSocket implementation (C.7.2)
│   └── stdio.rs        # Stdio implementation (C.7.3)
├── transport/           # OLD - to be deprecated
│   └── ...
```

## Success Criteria

- [ ] Connection trait defined with async_trait
- [ ] Protocol selection strategy implemented
- [ ] Connection manager skeleton ready
- [ ] Adapter allows using old transports
- [ ] All code compiles with no warnings
- [ ] Basic tests for adapter

## Testing

```rust
#[tokio::test]
async fn test_adapter_with_old_transport() {
    // Create old-style transport
    let old_transport = create_mock_transport();
    
    // Wrap in adapter
    let mut connection = TransportAdapter::new(old_transport);
    
    // Use as Connection
    let response = connection.request(json!({"method": "test"})).await?;
    assert_eq!(response["result"], "ok");
}
```

## Notes

- This is the foundation - keep it simple initially
- Focus on clean trait definition
- Adapter enables incremental migration
- Connection manager will evolve as we implement protocols