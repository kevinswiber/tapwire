# Task F.3: Design Incoming/Outgoing Transport Traits

## Objective

Design the core `IncomingTransport` and `OutgoingTransport` traits that will replace the current unified `Transport` trait, providing clearer abstractions for proxy architecture.

## Background

Current `Transport` trait conflates:
- Connection direction (who initiates)
- Data flow direction (bidirectional for all)
- Transport role (accepting vs connecting)

This leads to confusing types like `StdioTransport` vs `StdioClientTransport`.

## Design Requirements

### IncomingTransport (Proxy Accepts Connections)

```rust
/// Transport that accepts connections/input from clients
#[async_trait]
pub trait IncomingTransport: Send + Sync {
    /// Start accepting connections or input
    async fn listen(&mut self) -> TransportResult<()>;
    
    /// Receive a request from a client
    async fn receive_request(&mut self) -> TransportResult<MessageEnvelope>;
    
    /// Send a response/notification back to the client
    async fn send_response(&mut self, envelope: MessageEnvelope) -> TransportResult<()>;
    
    /// Check if transport is ready to accept requests
    fn is_listening(&self) -> bool;
    
    /// Get binding information
    fn binding_info(&self) -> BindingInfo;
    
    /// Get session ID for this connection
    fn session_id(&self) -> &SessionId;
    
    /// Gracefully shutdown the transport
    async fn shutdown(&mut self) -> TransportResult<()>;
}
```

### OutgoingTransport (Proxy Connects to Upstream)

```rust
/// Transport that connects to upstream MCP servers
#[async_trait]
pub trait OutgoingTransport: Send + Sync {
    /// Connect to the upstream target
    async fn connect(&mut self) -> TransportResult<()>;
    
    /// Send a request to the upstream server
    async fn send_request(&mut self, envelope: MessageEnvelope) -> TransportResult<()>;
    
    /// Receive a response/notification from upstream
    async fn receive_response(&mut self) -> TransportResult<MessageEnvelope>;
    
    /// Check if connected to upstream
    fn is_connected(&self) -> bool;
    
    /// Get target information
    fn target_info(&self) -> TargetInfo;
    
    /// Get session ID for this connection
    fn session_id(&self) -> &SessionId;
    
    /// Disconnect from upstream
    async fn disconnect(&mut self) -> TransportResult<()>;
}
```

### Supporting Types

```rust
/// Information about where an IncomingTransport is listening
#[derive(Debug, Clone)]
pub enum BindingInfo {
    /// Reading from current process stdin
    Stdio,
    
    /// HTTP server listening on address
    HttpServer { 
        addr: SocketAddr,
        path: String,  // e.g., "/mcp"
    },
    
    /// Memory channel (for testing)
    Memory { 
        channel_id: String 
    },
}

/// Information about where an OutgoingTransport connects
#[derive(Debug, Clone)]
pub enum TargetInfo {
    /// Subprocess with command
    Subprocess { 
        command: String,
        args: Vec<String>,
    },
    
    /// HTTP endpoint
    HttpEndpoint { 
        url: Url,
        headers: HeaderMap,
    },
    
    /// Streamable HTTP (POST + SSE)
    StreamableHttp { 
        post_url: Url,
        sse_url: Option<Url>,  // None if same as post_url
    },
    
    /// Memory channel (for testing)
    Memory { 
        channel_id: String 
    },
}
```

## Implementation Mapping

### IncomingTransport Implementations

| Current Type | New Type | Notes |
|--------------|----------|-------|
| `StdioClientTransport` | `StdioIncoming` | Reads from stdin |
| `HttpMcpTransport` (partial) | `HttpServerIncoming` | HTTP server |
| N/A | `StreamableHttpIncoming` | HTTP server + SSE |

### OutgoingTransport Implementations

| Current Type | New Type | Notes |
|--------------|----------|-------|
| `StdioTransport` | `SubprocessOutgoing` | Spawns subprocess |
| `HttpTransport` | `HttpClientOutgoing` | HTTP client |
| `SseTransport` | (removed) | Part of StreamableHttp |
| N/A | `StreamableHttpOutgoing` | HTTP + SSE client |

## Proxy Integration

```rust
pub struct ForwardProxy {
    incoming: Box<dyn IncomingTransport>,
    outgoing: Box<dyn OutgoingTransport>,
    interceptor_chain: InterceptorChain,
    recorder: Option<SessionRecorder>,
}

impl ForwardProxy {
    pub async fn run(&mut self) -> Result<()> {
        // Start listening for incoming
        self.incoming.listen().await?;
        
        // Connect to outgoing
        self.outgoing.connect().await?;
        
        // Main proxy loop
        loop {
            tokio::select! {
                // Client -> Server
                request = self.incoming.receive_request() => {
                    let mut envelope = request?;
                    
                    // Process through interceptors
                    envelope = self.interceptor_chain.process_request(envelope).await?;
                    
                    // Record if enabled
                    if let Some(recorder) = &mut self.recorder {
                        recorder.record_request(&envelope).await?;
                    }
                    
                    // Forward to upstream
                    self.outgoing.send_request(envelope).await?;
                }
                
                // Server -> Client
                response = self.outgoing.receive_response() => {
                    let mut envelope = response?;
                    
                    // Process through interceptors
                    envelope = self.interceptor_chain.process_response(envelope).await?;
                    
                    // Record if enabled
                    if let Some(recorder) = &mut self.recorder {
                        recorder.record_response(&envelope).await?;
                    }
                    
                    // Send back to client
                    self.incoming.send_response(envelope).await?;
                }
            }
        }
    }
}
```

## Benefits

1. **Clear Semantics**: No confusion about transport direction
2. **Type Safety**: Can't accidentally use incoming as outgoing
3. **Better Testing**: Mock incoming/outgoing separately
4. **Cleaner Proxy**: Proxy code clearly shows data flow

## Migration Strategy

1. Create new traits alongside existing `Transport` trait
2. Implement adapters from old to new
3. Gradually migrate each transport type
4. Update proxies to use new traits
5. Remove old `Transport` trait

## Success Criteria

- [ ] Traits compile with no warnings
- [ ] All transport types can be mapped to new model
- [ ] Proxy integration is cleaner than current
- [ ] No loss of functionality
- [ ] Improved testability demonstrated

## Deliverables

1. `src/transport/incoming.rs` - IncomingTransport trait and types
2. `src/transport/outgoing.rs` - OutgoingTransport trait and types
3. `src/transport/common.rs` - Shared types (BindingInfo, TargetInfo)
4. Updated design document with trait specifications
5. Migration plan for existing transports