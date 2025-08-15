# Current Forward Proxy Architecture Analysis

## Executive Summary
The forward proxy currently operates in a **single-session mode**, accepting one client connection and forwarding to one upstream server. Each invocation of the proxy creates a dedicated ForwardProxy instance with a single session ID.

## Key Limitations

### 1. Single Session Design
- `ForwardProxy` struct has a single `session_id: SessionId` field
- No mechanism to accept multiple client connections
- Each proxy instance is tied to one client-server pair
- Cannot handle concurrent clients

### 2. Transport Binding
- Client transport is created once at startup (stdio or HTTP)
- For stdio: inherently single-connection (process stdin/stdout)
- For HTTP: Currently creates one HTTP listener but doesn't handle multiple connections
- Server transport is also single-instance per proxy

### 3. Lifecycle Management
- Proxy runs until client disconnects or shutdown signal
- No way to spawn new sessions while running
- No session registry or tracking of multiple active connections

## Current Architecture Flow

```
CLI Command
    ↓
ForwardCommand::execute()
    ↓
ShadowcatBuilder → Shadowcat instance
    ↓
shadowcat.forward_stdio() OR shadowcat.forward_http()
    ↓
Creates single client transport (IncomingTransport)
Creates single server transport (OutgoingTransport)
    ↓
ForwardProxy::new() with single session_id
    ↓
proxy.run_with_shutdown()
    ↓
Spawns 2 tasks:
  - client_to_server forwarding loop
  - server_to_client forwarding loop
    ↓
Runs until disconnection or shutdown
```

## Code Structure

### ForwardProxy Structure
```rust
pub struct ForwardProxy {
    session_id: SessionId,              // Single session
    session_manager: Option<Arc<SessionManager>>,
    interceptor_chain: Option<Arc<InterceptorChain>>,
    tape_recorder: Option<Arc<TapeRecorder>>,
    rate_limiter: Option<Arc<MultiTierRateLimiter>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    tasks: Vec<JoinHandle<()>>,
}
```

### Key Methods
- `ForwardProxy::new()` - Creates with single session ID
- `ForwardProxy::start()` - Accepts one client, connects to one server
- Two forwarding tasks run in parallel for bidirectional flow

## Transport Specifics

### Stdio Transport
- **Inherently single-connection**: Uses process stdin/stdout
- Cannot accept multiple connections by design
- Would need protocol-level multiplexing to support multiple logical sessions

### HTTP Transport
- **Could support multiple connections** on same port
- Currently doesn't have accept loop for multiple clients
- Each HTTP request could spawn a new session
- Connection pooling possible for upstream

### SSE Transport
- Long-lived connections
- Could support multiple SSE streams
- Each stream would be a separate session

## Session Management
- `SessionManager` exists and could track multiple sessions
- Currently only one session created per proxy instance
- No mechanism to query active sessions or spawn new ones

## Resource Usage
- Current: ~100KB memory per proxy instance
- 2 tokio tasks per proxy (client→server, server→client)
- 2 file descriptors (client + server connections)

## Path Forward for Multi-Session

### Required Changes
1. **Accept Loop**: Add connection accept loop for HTTP/SSE
2. **Session Registry**: Track multiple active sessions
3. **Per-Session Tasks**: Spawn handler tasks for each client
4. **Resource Limits**: Implement max session limits
5. **Lifecycle Management**: Handle individual session cleanup

### Transport Strategy
- **HTTP/SSE**: Full multi-session support with accept loop
- **Stdio**: Keep single-session OR implement protocol multiplexing
- **Connection Pooling**: Share upstream HTTP connections where possible

## Next Steps
1. Design session registry and accept loop architecture
2. Determine stdio strategy (single vs multiplexed)
3. Plan resource management and limits
4. Design connection pooling for HTTP upstreams