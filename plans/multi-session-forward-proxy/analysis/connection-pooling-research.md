# Connection Pooling Research for Multi-Session Forward Proxy

## Existing Infrastructure

### ConnectionPool<T> Implementation
The codebase already has a sophisticated connection pool (`src/proxy/pool.rs`) with:
- Generic pool for any `PoolableConnection` type
- Health checking and maintenance loops
- Connection lifecycle management (max idle time, max lifetime)
- Semaphore-based concurrency control
- Automatic connection return via Drop trait

### Current Usage
- **Reverse Proxy**: Uses `ConnectionPool<PoolableOutgoingTransport>` for stdio connections
- **Forward Proxy**: No pooling - creates dedicated connections per session

## Pooling Strategies by Transport Type

### 1. HTTP Transport Pooling

#### Benefits
- **Connection Reuse**: HTTP/1.1 keep-alive allows multiple requests on same TCP connection
- **Reduced Latency**: No TCP handshake for subsequent requests
- **Resource Efficiency**: Fewer sockets and file descriptors

#### Implementation Approach
```rust
// HTTP connections can be pooled per upstream host
HashMap<String, ConnectionPool<HttpOutgoingTransport>>
```

#### Considerations
- `reqwest` library has built-in connection pooling
- May not need custom pooling for HTTP if using reqwest
- Pool key should be (scheme, host, port) tuple

### 2. Stdio Transport Pooling

#### Challenges
- **Process-based**: Each stdio connection spawns a subprocess
- **Stateful**: MCP servers maintain session state
- **Single-stream**: stdin/stdout are single bidirectional streams

#### Strategy
- **NO POOLING**: Stdio connections should NOT be pooled
- Each client needs dedicated subprocess
- Process lifecycle tied to session lifecycle

### 3. SSE Transport Pooling

#### Characteristics
- **Long-lived**: SSE connections stay open for server-sent events
- **Unidirectional**: Server→Client for events, needs separate channel for Client→Server
- **Stateful**: Server maintains event stream state

#### Strategy
- **NO POOLING**: SSE connections are session-specific
- Each client needs dedicated SSE stream
- Could pool the HTTP POST connections for requests (separate from SSE)

## Recommended Architecture

### Multi-Session with Selective Pooling

```rust
pub struct MultiSessionForwardProxy {
    // Session registry
    sessions: Arc<RwLock<HashMap<SessionId, SessionState>>>,
    
    // HTTP connection pools (per upstream host)
    http_pools: Arc<RwLock<HashMap<String, ConnectionPool<HttpTransport>>>>,
    
    // No pooling for stdio - each session gets dedicated process
    // No pooling for SSE - each session gets dedicated stream
}
```

### Session State
```rust
struct SessionState {
    session_id: SessionId,
    client_transport: Arc<RwLock<Box<dyn IncomingTransport>>>,
    server_transport: ServerTransportHandle,
    created_at: Instant,
    last_activity: Instant,
}

enum ServerTransportHandle {
    Pooled(PooledConnection<HttpTransport>),  // From pool
    Dedicated(Box<dyn OutgoingTransport>),     // Stdio or SSE
}
```

## Implementation Plan

### Phase 1: Multi-Session Without Pooling
- Focus on accept loop and session management
- Each session gets dedicated upstream connection
- Simpler to implement and test

### Phase 2: Add HTTP Pooling
- Implement pooling only for HTTP upstreams
- Reuse existing `ConnectionPool<T>` infrastructure
- Pool connections per upstream host

### Phase 3: Optimization
- Monitor pool metrics
- Tune pool configuration
- Consider pooling for other transports if beneficial

## Resource Implications

### Without Pooling
- **Connections**: 2N (N clients × 2 connections each)
- **Memory**: ~100KB per session
- **File Descriptors**: 2N

### With HTTP Pooling
- **Connections**: N + P (N client connections + P pooled upstream)
- **Memory**: ~100KB per session + pool overhead
- **File Descriptors**: N + P (reduced from 2N)
- **Typical P**: 10-20 connections per upstream host

## Key Decisions

1. **Start without pooling** - Get multi-session working first
2. **Pool only HTTP** - Most benefit, least complexity
3. **No stdio pooling** - Process lifecycle issues
4. **No SSE pooling** - Long-lived stateful connections
5. **Use existing pool** - Leverage `ConnectionPool<T>` infrastructure

## Next Steps
1. Design session registry and accept loop
2. Implement multi-session without pooling
3. Add HTTP pooling as optimization
4. Monitor and tune pool parameters