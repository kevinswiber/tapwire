# Task: Implement Forward Proxy Logic

**Status:** Not Started  
**Priority:** High  
**Estimated Time:** 2 days  
**Dependencies:** Transport trait, Session manager interface

---

## Objective

Implement the core ForwardProxy struct that handles bidirectional message routing between a client and an MCP server. This is the heart of Shadowcat's proxy functionality.

---

## Design

### Core Structure
```rust
pub struct ForwardProxy {
    client_transport: Box<dyn Transport>,
    server_transport: Box<dyn Transport>,
    session_manager: Arc<SessionManager>,
    recorder: Option<Arc<TapeRecorder>>,
    interceptor_chain: InterceptorChain,
    config: ForwardProxyConfig,
}
```

### Message Flow
1. Client connects to proxy
2. Proxy connects to server
3. Bidirectional message routing:
   - Client → Interceptors → Recorder → Server
   - Server → Interceptors → Recorder → Client
4. Session tracking throughout
5. Clean shutdown on either side disconnect

---

## Implementation Steps

### 1. Create ForwardProxyConfig
```rust
pub struct ForwardProxyConfig {
    pub buffer_size: usize,
    pub timeout_ms: u64,
    pub max_message_size: usize,
    pub record_enabled: bool,
}
```

### 2. Implement ForwardProxy Constructor
```rust
impl ForwardProxy {
    pub fn new(
        client_transport: Box<dyn Transport>,
        server_transport: Box<dyn Transport>,
        session_manager: Arc<SessionManager>,
        config: ForwardProxyConfig,
    ) -> Self {
        // Initialize with empty interceptor chain
        // Optional recorder based on config
    }
}
```

### 3. Implement Core Proxy Loop
```rust
pub async fn run(&mut self) -> Result<()> {
    // Create session
    let session = self.session_manager.create_session(
        self.client_transport.transport_type()
    ).await?;
    
    // Connect transports
    self.client_transport.connect().await?;
    self.server_transport.connect().await?;
    
    // Spawn bidirectional routing tasks
    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
    
    let client_to_server = self.route_client_to_server(
        session.clone(),
        shutdown_rx.resubscribe()
    );
    
    let server_to_client = self.route_server_to_client(
        session.clone(),
        shutdown_rx.resubscribe()
    );
    
    // Wait for either to complete
    tokio::select! {
        result = client_to_server => {
            log_result("client->server", result);
        }
        result = server_to_client => {
            log_result("server->client", result);
        }
    }
    
    // Shutdown
    let _ = shutdown_tx.send(());
    self.shutdown().await?;
    
    Ok(())
}
```

### 4. Implement Routing Functions
```rust
async fn route_client_to_server(
    &mut self,
    session: Arc<Session>,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<()> {
    loop {
        tokio::select! {
            _ = shutdown.recv() => break,
            result = self.client_transport.receive() => {
                match result {
                    Ok(msg) => {
                        // Add to session
                        self.session_manager.add_frame(
                            &session.id,
                            Frame::new(Direction::ClientToServer, &msg)
                        ).await?;
                        
                        // Process through interceptors
                        let msg = self.interceptor_chain.process(
                            InterceptContext::new(&session, Direction::ClientToServer, msg)
                        ).await?;
                        
                        // Record if enabled
                        if let Some(recorder) = &self.recorder {
                            recorder.record_frame(
                                Direction::ClientToServer,
                                TransportEdge::ProxyOut,
                                &msg
                            ).await;
                        }
                        
                        // Forward to server
                        self.server_transport.send(msg).await?;
                    }
                    Err(e) => {
                        error!("Client receive error: {}", e);
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}
```

### 5. Add Connection Management
```rust
pub struct ConnectionPool {
    transports: HashMap<String, Box<dyn Transport>>,
    max_connections: usize,
}

impl ConnectionPool {
    pub async fn get_or_create(&mut self, key: &str) -> Result<&mut Box<dyn Transport>> {
        // Implement connection pooling logic
    }
}
```

### 6. Implement Graceful Shutdown
```rust
async fn shutdown(&mut self) -> Result<()> {
    info!("Shutting down forward proxy");
    
    // Close transports
    if let Err(e) = self.client_transport.close().await {
        warn!("Error closing client transport: {}", e);
    }
    
    if let Err(e) = self.server_transport.close().await {
        warn!("Error closing server transport: {}", e);
    }
    
    // Finalize recording
    if let Some(recorder) = &mut self.recorder {
        recorder.finalize().await?;
    }
    
    // Update session state
    self.session_manager.end_session(&session.id).await?;
    
    Ok(())
}
```

---

## Testing Strategy

### Unit Tests
- Mock transports for controlled message flow
- Test error handling in routing
- Test interceptor integration
- Test recorder integration

### Integration Tests
- Real stdio transport tests
- Multi-message session tests
- Error scenario tests
- Shutdown behavior tests

### Example Test
```rust
#[tokio::test]
async fn test_forward_proxy_basic_flow() {
    let client = MockTransport::new();
    let server = MockTransport::new();
    
    client.expect_receive().returning(|| {
        Ok(TransportMessage::new_request(
            "1".to_string(),
            "test".to_string(),
            json!({})
        ))
    });
    
    server.expect_send().times(1).returning(|_| Ok(()));
    server.expect_receive().returning(|| {
        Ok(TransportMessage::new_response(
            "1".to_string(),
            Some(json!({"result": "ok"})),
            None
        ))
    });
    
    let proxy = ForwardProxy::new(
        Box::new(client),
        Box::new(server),
        Arc::new(MemorySessionManager::new()),
        ForwardProxyConfig::default()
    );
    
    proxy.run().await.unwrap();
}
```

---

## Error Handling

- Transport errors should trigger shutdown
- Interceptor errors should be logged but not fatal
- Recording errors should be logged but not fatal
- Session errors should trigger shutdown
- Use structured logging for debugging

---

## Performance Considerations

- Use bounded channels to prevent memory growth
- Implement backpressure between transports
- Consider message batching for recording
- Profile memory usage with many sessions

---

## Future Enhancements

- Message transformation support
- Rate limiting
- Circuit breaker for server connections
- Metrics collection
- WebSocket transport support