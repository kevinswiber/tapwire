# Multi-Session Forward Proxy Architecture Design

## Overview
Transform the forward proxy from single-session to multi-session, allowing concurrent client connections with independent upstream servers.

## Core Architecture

### 1. MultiSessionForwardProxy Structure
```rust
pub struct MultiSessionForwardProxy {
    // Core components
    sessions: Arc<RwLock<HashMap<SessionId, SessionHandle>>>,
    session_manager: Arc<SessionManager>,
    
    // Configuration
    config: ProxyConfig,
    max_sessions: usize,
    session_timeout: Duration,
    
    // Shared resources
    interceptor_chain: Option<Arc<InterceptorChain>>,
    tape_recorder: Option<Arc<TapeRecorder>>,
    rate_limiter: Option<Arc<MultiTierRateLimiter>>,
    
    // Connection pools (future optimization)
    http_pools: Arc<RwLock<HashMap<String, ConnectionPool<HttpTransport>>>>,
    
    // Lifecycle
    shutdown_token: ShutdownToken,
    accept_handle: Option<JoinHandle<()>>,
    cleanup_handle: Option<JoinHandle<()>>,
}
```

### 2. Session Handle
```rust
struct SessionHandle {
    session_id: SessionId,
    client_addr: Option<SocketAddr>,
    created_at: Instant,
    last_activity: Arc<RwLock<Instant>>,
    
    // Transport handles
    client_task: JoinHandle<()>,
    server_task: JoinHandle<()>,
    
    // Graceful shutdown
    shutdown_tx: mpsc::Sender<()>,
}
```

## Key Components

### 1. Accept Loop (HTTP/SSE Only)

```rust
async fn accept_loop(self: Arc<Self>) {
    let listener = TcpListener::bind(self.bind_addr).await?;
    
    loop {
        select! {
            Ok((stream, addr)) = listener.accept() => {
                // Check session limit
                if self.sessions.read().await.len() >= self.max_sessions {
                    // Reject with 503 Service Unavailable
                    continue;
                }
                
                // Spawn session handler
                self.handle_new_session(stream, addr).await;
            }
            _ = self.shutdown_token.cancelled() => {
                break;
            }
        }
    }
}
```

### 2. Session Handler

```rust
async fn handle_new_session(
    self: Arc<Self>,
    stream: TcpStream,
    client_addr: SocketAddr,
) {
    let session_id = SessionId::new();
    
    // Create client transport
    let client_transport = Box::new(HttpIncomingTransport::from_stream(stream));
    
    // Create server transport (based on config)
    let server_transport = self.create_upstream_transport(&session_id).await?;
    
    // Create session handle
    let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
    
    // Spawn forwarding tasks
    let client_task = tokio::spawn(forward_client_to_server(...));
    let server_task = tokio::spawn(forward_server_to_client(...));
    
    let handle = SessionHandle {
        session_id: session_id.clone(),
        client_addr: Some(client_addr),
        created_at: Instant::now(),
        last_activity: Arc::new(RwLock::new(Instant::now())),
        client_task,
        server_task,
        shutdown_tx,
    };
    
    // Register session
    self.sessions.write().await.insert(session_id, handle);
}
```

### 3. Session Cleanup

```rust
async fn cleanup_loop(self: Arc<Self>) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    
    loop {
        select! {
            _ = interval.tick() => {
                let mut sessions = self.sessions.write().await;
                let now = Instant::now();
                
                // Find expired sessions
                let expired: Vec<SessionId> = sessions
                    .iter()
                    .filter(|(_, handle)| {
                        let last = handle.last_activity.read().await;
                        now.duration_since(*last) > self.session_timeout
                    })
                    .map(|(id, _)| id.clone())
                    .collect();
                
                // Clean up expired sessions
                for session_id in expired {
                    if let Some(handle) = sessions.remove(&session_id) {
                        let _ = handle.shutdown_tx.send(()).await;
                        // Tasks will clean up on shutdown signal
                    }
                }
            }
            _ = self.shutdown_token.cancelled() => {
                break;
            }
        }
    }
}
```

## Transport-Specific Behavior

### HTTP Transport
- **Accept Loop**: Yes - listen on TCP port
- **Multiple Sessions**: Yes - each HTTP connection is a session
- **Connection Pooling**: Future optimization for upstream
- **Session Mapping**: HTTP Connection → Session ID via headers

### Stdio Transport
- **Accept Loop**: No - single stdin/stdout
- **Multiple Sessions**: No* (see multiplexing option below)
- **Connection Pooling**: No - process-based
- **Session Mapping**: Single implicit session

### SSE Transport
- **Accept Loop**: Yes - listen on TCP port
- **Multiple Sessions**: Yes - each SSE stream is a session
- **Connection Pooling**: No for SSE, possible for HTTP POST
- **Session Mapping**: SSE Stream → Session ID via Last-Event-Id

## Migration Strategy

### Phase 1: Refactor Current ForwardProxy
1. Extract forwarding logic into standalone functions
2. Create `SessionHandle` abstraction
3. Add session registry

### Phase 2: Add Multi-Session Support
1. Implement accept loop for HTTP/SSE
2. Add session spawning logic
3. Implement cleanup loop

### Phase 3: Maintain Backward Compatibility
1. Keep single-session mode for stdio
2. Add `--multi-session` flag for HTTP/SSE
3. Default to single-session for compatibility

## Resource Management

### Limits
```rust
pub struct SessionLimits {
    max_sessions: usize,           // Default: 1000
    max_sessions_per_client: usize, // Default: 10
    session_timeout: Duration,      // Default: 5 minutes
    max_memory_per_session: usize,  // Default: 1MB
}
```

### Monitoring
```rust
pub struct SessionMetrics {
    active_sessions: AtomicU64,
    total_sessions: AtomicU64,
    rejected_sessions: AtomicU64,
    avg_session_duration: AtomicU64,
}
```

## API Changes

### CLI Interface
```bash
# Single session (backward compatible)
shadowcat forward stdio -- my-server

# Multi-session HTTP
shadowcat forward http --port 8080 --multi-session --max-sessions 100 --url http://upstream

# Multi-session with limits
shadowcat forward http --port 8080 --multi-session \
  --max-sessions 1000 \
  --session-timeout 300 \
  --max-per-client 10
```

### Configuration
```yaml
forward_proxy:
  multi_session:
    enabled: true
    max_sessions: 1000
    session_timeout_secs: 300
    max_sessions_per_client: 10
    cleanup_interval_secs: 60
```

## Testing Strategy

1. **Unit Tests**: Session registry, limits, cleanup
2. **Integration Tests**: Multiple concurrent clients
3. **Load Tests**: Max session limits, resource usage
4. **Compatibility Tests**: Single-session mode still works

## Future Enhancements

### Stdio Multiplexing (Optional)
```rust
// Protocol-level session multiplexing for stdio
enum StdioMessage {
    SessionCreate { session_id: SessionId },
    SessionData { session_id: SessionId, data: Vec<u8> },
    SessionClose { session_id: SessionId },
}
```

### Advanced Features
- Session affinity for load balancing
- Session migration between upstreams
- Session persistence across proxy restarts
- WebSocket upgrade support

## Success Criteria
- [ ] Accept multiple concurrent HTTP clients
- [ ] Each client gets independent upstream connection
- [ ] Sessions properly cleaned up on timeout
- [ ] Resource limits enforced
- [ ] Backward compatibility maintained
- [ ] Performance targets met (< 5% overhead)