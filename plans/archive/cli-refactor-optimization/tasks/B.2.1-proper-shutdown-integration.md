# Task B.2.1: Proper Shutdown Integration

## Objective
Fix the shortcuts taken in B.2 by properly integrating the shutdown system with actual proxy implementations, not placeholder code. The shutdown infrastructure is solid, but the integration is incomplete and uses test-only implementations.

## Background
During B.2, we took several shortcuts that compromise the design:
- CLI commands use placeholder implementations instead of real proxy loops
- No stdin/stdout client transport for forward proxy
- HTTP transport shutdown not implemented
- Bypassed the builder patterns we created in B.1
- Mixed abstraction levels (high-level shutdown, low-level transport manipulation)

This technical debt will block B.3 (High-Level API) and make B.6 (Integration Tests) fail immediately.

## Key Problems to Fix

### 1. Forward Proxy Doesn't Actually Proxy
Current implementation just sends one test message:
```rust
// This is wrong - not a real proxy
let test_msg = ProtocolMessage::new_request(...);
server_transport.send(envelope).await?;
```

### 2. Missing Client Transport
We need a proper stdin/stdout reader for the client side of forward proxy.

### 3. Inconsistent Use of Builders
We created builders in B.1 but aren't using them in the CLI shutdown integration.

### 4. HTTP Transport Ignored
Only stdio transports have shutdown support.

## Step-by-Step Process

### 1. Create Proper Stdin/Stdout Client Transport
```rust
// src/transport/stdio_client.rs
pub struct StdioClientTransport {
    stdin: tokio::io::Stdin,
    stdout: tokio::io::Stdout,
    // ... proper implementation
}

impl Transport for StdioClientTransport {
    // Read from stdin, write to stdout
}
```

### 2. Fix Forward Command Integration
```rust
// src/cli/forward.rs
async fn run_stdio_forward_with_shutdown(
    command_args: Vec<String>,
    config: ProxyConfig,
    shutdown: ShutdownToken,
) -> Result<()> {
    // Create session manager with shutdown support
    let session_manager = Arc::new(SessionManager::with_config(config.to_session_config()));
    
    // Start cleanup task with shutdown
    let cleanup_handle = tokio::spawn({
        let manager = session_manager.clone();
        let shutdown = shutdown.child();
        async move {
            manager.run_with_shutdown(shutdown).await
        }
    });

    // Create transports
    let client = StdioClientTransport::new();
    let server = StdioTransport::new(build_command(command_args));
    
    // Use the builder pattern properly!
    let proxy = ForwardProxyBuilder::new()
        .with_session_manager(session_manager)
        .with_shutdown_token(shutdown.child())
        .build();
    
    // Run the actual proxy with shutdown
    let result = proxy.run_with_shutdown(client, server, shutdown.child()).await;
    
    // Cleanup
    cleanup_handle.abort();
    result
}
```

### 3. Implement Real Proxy Loop with Shutdown
```rust
// src/proxy/forward.rs
impl ForwardProxy {
    pub async fn run_with_shutdown<C, S>(
        mut self,
        client: C,
        server: S,
        mut shutdown: ShutdownToken,
    ) -> Result<()> 
    where
        C: Transport + 'static,
        S: Transport + 'static,
    {
        // Connect transports
        client.connect().await?;
        server.connect().await?;
        
        // Create bidirectional channels
        let (client_to_server_tx, client_to_server_rx) = mpsc::channel(100);
        let (server_to_client_tx, server_to_client_rx) = mpsc::channel(100);
        
        // Spawn reader tasks with shutdown support
        let client_reader = tokio::spawn({
            let shutdown = shutdown.child();
            async move {
                loop {
                    tokio::select! {
                        msg = client.receive() => {
                            match msg {
                                Ok(m) => client_to_server_tx.send(m).await?,
                                Err(e) => break,
                            }
                        }
                        _ = shutdown.wait() => {
                            info!("Client reader shutting down");
                            break;
                        }
                    }
                }
            }
        });
        
        // Similar for server_reader, client_writer, server_writer
        
        // Wait for shutdown or completion
        tokio::select! {
            _ = client_reader => {},
            _ = server_reader => {},
            _ = shutdown.wait() => {
                info!("Proxy received shutdown signal");
            }
        }
        
        // Graceful cleanup
        client.close().await?;
        server.close().await?;
        
        Ok(())
    }
}
```

### 4. Fix Record Command Similarly
Update the record command to use proper proxy with recording, not placeholder implementation.

### 5. Add HTTP Transport Shutdown
```rust
// src/cli/forward.rs
ForwardTransport::StreamableHttp { ... } => {
    run_streamable_http_forward_with_shutdown(url, ..., shutdown).await
}
```

### 6. Remove Unused Macro
Delete the `select_with_shutdown!` macro that we're not using.

### 7. Improve Error Handling
Replace all `let _ = result` patterns with proper error handling or logging.

## Expected Deliverables

### Modified Files
- `src/transport/mod.rs` - Add StdioClientTransport
- `src/transport/stdio_client.rs` - New client implementation
- `src/cli/forward.rs` - Proper proxy integration
- `src/cli/record.rs` - Proper recording integration  
- `src/cli/reverse.rs` - Ensure consistency
- `src/proxy/forward.rs` - Real run_with_shutdown implementation
- `src/shutdown.rs` - Remove unused macro

### New Tests
- `tests/integration/shutdown_proxy.rs` - Test actual proxy with shutdown
- Test that proxy properly forwards messages during shutdown
- Test graceful connection draining

### Verification Commands
```bash
# Test real proxy with shutdown
echo '{"jsonrpc":"2.0","method":"initialize","id":1}' | \
  cargo run -- forward stdio -- your-mcp-server

# Press Ctrl+C and verify graceful shutdown with proper cleanup

# Run integration tests
cargo test integration::shutdown_proxy
```

## Success Criteria Checklist
- [ ] Forward proxy actually forwards messages (not just test message)
- [ ] Stdin/stdout properly handled for client side
- [ ] Builders used consistently throughout CLI
- [ ] HTTP transport has shutdown support
- [ ] No placeholder implementations remain
- [ ] Error handling improved (no `let _ =` patterns)
- [ ] Integration tests pass with real proxy scenarios
- [ ] Abstraction levels consistent (CLI thin, library rich)

## Risk Assessment
- **Risk**: Breaking existing functionality
  - **Mitigation**: Comprehensive testing before/after
  
- **Risk**: Complexity in bidirectional proxy with shutdown
  - **Mitigation**: Clear separation of concerns, good logging

## Duration Estimate
**4 hours**
- 1 hour: Create StdioClientTransport
- 1.5 hours: Fix forward/record command integration
- 1 hour: Implement proper proxy loops with shutdown
- 30 min: Testing and verification

## Dependencies
- B.1: Builder patterns (using them properly)
- B.2: Shutdown system (extending it correctly)

## Notes
- This is critical technical debt that must be addressed
- Without this, B.3 (High-Level API) will be building on sand
- B.6 (Integration Tests) will immediately expose these issues
- This maintains our design principles and aesthetic