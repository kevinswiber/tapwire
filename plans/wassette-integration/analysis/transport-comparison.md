# Wassette vs Shadowcat Transport Analysis

## Wassette Stdio Transport

### Implementation
- **Library**: `rmcp::transport::stdio`
- **Process Model**: Single process, stdin/stdout communication
- **Message Format**: Line-delimited JSON-RPC 2.0
- **Logging**: Uses stderr to avoid protocol interference
- **Lifecycle**: Lives for duration of process

### Message Handling
```rust
// Wassette uses rmcp's stdio transport directly
let transport = stdio_transport();
let running_service = serve_server(server, transport).await?;
```

### Protocol Flow
1. Client spawns `wassette serve --stdio`
2. Sends JSON-RPC messages to stdin
3. Receives responses on stdout
4. Process terminates on client disconnect

## Shadowcat Stdio Transport

### Implementation
- **Module**: `src/transport/stdio.rs`
- **Process Model**: Spawns child process for MCP server
- **Message Format**: Line-delimited JSON-RPC 2.0
- **Buffering**: Uses buffer pool for efficiency
- **Lifecycle**: Manages child process lifecycle

### Message Handling
```rust
// Shadowcat spawns and manages stdio process
let mut transport = StdioTransport::new(cmd);
transport.connect().await?;
transport.send(message).await?;
let response = transport.receive().await?;
```

### Protocol Flow
1. Shadowcat spawns MCP server process
2. Bidirectional pipe for stdin/stdout
3. Message framing and buffering
4. Process management and cleanup

## Compatibility Analysis

### Protocol Level
- ✅ **JSON-RPC 2.0**: Both use same format
- ✅ **Line Delimited**: Same framing mechanism
- ✅ **Message Types**: Request/Response/Notification
- ✅ **Error Format**: Standard JSON-RPC errors

### Transport Level
- ✅ **Stdio Streams**: Compatible stdin/stdout usage
- ✅ **Process Model**: Shadowcat can spawn Wassette
- ⚠️ **Logging**: Need to handle stderr properly
- ✅ **Lifecycle**: Clean spawn/terminate semantics

### Session Management
- **Wassette**: Sessions handled by rmcp internally
- **Shadowcat**: Explicit session tracking
- **Integration**: Can correlate via message IDs

## Required Modifications

### Shadowcat Changes

#### 1. Wassette-Aware Spawning
```rust
// Add Wassette-specific command builder
impl StdioTransport {
    pub fn spawn_wassette(plugin_dir: Option<PathBuf>) -> Result<Self> {
        let mut cmd = Command::new("wassette");
        cmd.args(["serve", "--stdio"]);
        if let Some(dir) = plugin_dir {
            cmd.arg("--plugin-dir").arg(dir);
        }
        cmd.stderr(Stdio::piped()); // Capture logs
        Self::new(cmd)
    }
}
```

#### 2. stderr Log Handling
```rust
// Separate stderr handling for Wassette logs
async fn handle_stderr(stderr: ChildStderr) {
    let reader = BufReader::new(stderr);
    while let Ok(line) = reader.read_line().await {
        tracing::debug!(target: "wassette", "{}", line);
    }
}
```

#### 3. Message Routing
```rust
// Route tool calls to Wassette
match message {
    ProtocolMessage::Request { method, .. } if method.starts_with("tools/") => {
        // Forward to Wassette transport
        wassette_transport.send(message).await?
    }
    _ => // Handle normally
}
```

### Wassette Adaptations

No changes needed to Wassette itself - it already provides:
- Standard stdio interface
- JSON-RPC message handling
- Proper stderr separation

## Proxy Pattern Design

### Architecture
```
Client -> Shadowcat Proxy -> Wassette Process
       ↑                  ↑
       |                  |
   Recording          Component
   Interception       Execution
```

### Implementation Approach

#### Option 1: Direct Process Proxy (Recommended)
```rust
// Shadowcat spawns Wassette as child process
let wassette = StdioTransport::spawn_wassette(plugin_dir)?;
let proxy = ForwardProxy::new(client_transport, wassette);
proxy.run().await?;
```

**Pros**:
- Simple integration
- Full control over lifecycle
- Easy recording/interception

**Cons**:
- Single client per Wassette instance
- Process overhead per client

#### Option 2: Shared Wassette Instance
```rust
// Single Wassette HTTP instance, multiple clients
let wassette_http = HttpTransport::connect("http://localhost:9001")?;
let proxy = ForwardProxy::new_shared(wassette_http);
// Handle multiple clients
```

**Pros**:
- Resource efficient
- Multiple concurrent clients
- Component sharing

**Cons**:
- Requires HTTP transport
- More complex session management

## Performance Considerations

### Latency Impact
- **Process Spawn**: ~10-50ms for Wassette startup
- **Message Overhead**: < 1ms for proxy forwarding
- **Component Load**: ~5-10ms first invocation
- **Steady State**: < 5ms round trip

### Optimization Strategies
1. **Process Pooling**: Pre-spawn Wassette instances
2. **Connection Reuse**: Keep processes alive
3. **Batch Loading**: Load components on startup
4. **Caching**: Cache component metadata

### Throughput
- **Stdio Limit**: ~10k msg/sec per process
- **Proxy Overhead**: < 5% throughput reduction
- **Bottleneck**: Component execution, not transport

## Buffer Management

### Message Size Limits
- **Wassette**: No hard limit (streaming)
- **Shadowcat**: Configurable buffer pool
- **Recommendation**: 10MB default, 100MB max

### Streaming Strategy
```rust
// For large messages
impl StreamingProxy {
    async fn forward_chunked(&mut self, message: &[u8]) {
        for chunk in message.chunks(CHUNK_SIZE) {
            self.upstream.write(chunk).await?;
        }
    }
}
```

## Error Handling

### Transport Errors
- **Process Crash**: Detect and restart
- **Pipe Broken**: Clean shutdown
- **Timeout**: Configurable per operation

### Protocol Errors
- **Invalid JSON**: Return error response
- **Unknown Method**: Forward to Wassette
- **Component Failure**: Propagate error

## Implementation Plan

### Phase 1: Basic Proxy
1. Extend StdioTransport for Wassette
2. Implement stderr handling
3. Basic forward proxy

### Phase 2: Enhanced Features
1. Add recording capability
2. Implement interception
3. Session correlation

### Phase 3: Optimization
1. Process pooling
2. Connection reuse
3. Performance tuning

## Testing Strategy

### Unit Tests
- Mock Wassette process
- Test message forwarding
- Verify error handling

### Integration Tests
```rust
#[tokio::test]
async fn test_wassette_proxy() {
    let wassette = spawn_test_wassette().await;
    let proxy = create_proxy(wassette).await;
    
    // Test tool invocation
    let response = proxy.call_tool("fetch", params).await?;
    assert!(response.is_ok());
}
```

### Performance Tests
- Measure latency overhead
- Verify < 5% impact
- Load testing with multiple clients