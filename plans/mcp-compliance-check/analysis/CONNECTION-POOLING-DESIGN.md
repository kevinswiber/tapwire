# Connection Pooling Design with Shadowcat's Generic Pool

**Date**: 2025-08-24  
**Status**: Design Document  
**Context**: How to leverage shadowcat's generic `Pool<T: PoolableResource>` for Connection pattern

## Executive Summary

Shadowcat already has a sophisticated, production-ready generic pool that:
- Uses semaphore-based capacity control
- Has SQLx-style hooks (after_create, before_acquire, after_release)
- Includes health checks and idle cleanup
- Handles graceful shutdown and maintenance

We can leverage this directly for Connection pooling with protocol-specific pooling strategies.

## The Pool Architecture

```rust
// Shadowcat's existing pool (simplified)
pub trait PoolableResource: Send + Sync {
    async fn is_healthy(&self) -> bool;
    async fn close(&mut self) -> Result<()>;
    fn resource_id(&self) -> String;
}

pub struct Pool<T: PoolableResource> {
    inner: Arc<PoolInner<T>>,
}

pub struct PoolOptions {
    pub max_connections: usize,
    pub acquire_timeout: Duration,
    pub idle_timeout: Option<Duration>,
    pub max_lifetime: Option<Duration>,
    pub health_check_interval: Duration,
}
```

## Making Connections Poolable

### 1. Connection as PoolableResource

```rust
// crates/mcp/src/connection/poolable.rs

use shadowcat::pool::{PoolableResource, Result as PoolResult};
use async_trait::async_trait;

/// Wrapper to make Connection trait compatible with PoolableResource
pub struct PoolableConnection {
    inner: Box<dyn Connection>,
    origin: String,
    protocol: Protocol,
    created_at: Instant,
    last_used: Arc<RwLock<Instant>>,
    request_count: Arc<AtomicU64>,
}

#[async_trait]
impl PoolableResource for PoolableConnection {
    async fn is_healthy(&self) -> bool {
        // Protocol-specific health checks
        match self.protocol {
            Protocol::Http2 => {
                // HTTP/2 connections are multiplexed and long-lived
                // Check if underlying TCP is still alive
                self.inner.is_healthy().await
            }
            Protocol::WebSocket => {
                // WebSocket needs ping/pong
                self.inner.ping().await.is_ok()
            }
            Protocol::Stdio => {
                // Stdio is singleton, always healthy if process alive
                true
            }
            _ => self.inner.is_healthy().await
        }
    }
    
    async fn close(&mut self) -> PoolResult<()> {
        self.inner.close().await
            .map_err(|e| shadowcat::pool::Error::CloseFailed(e.to_string()))
    }
    
    fn resource_id(&self) -> String {
        format!("{}-{}-{}", self.protocol, self.origin, self.created_at.timestamp())
    }
}
```

### 2. Protocol-Specific Pooling Strategies

```rust
// crates/mcp/src/connection/pool_strategy.rs

pub enum PoolingStrategy {
    /// Singleton - one connection for all (stdio)
    Singleton,
    
    /// PerOrigin - share connections by origin (HTTP/2)
    PerOrigin { max_per_origin: usize },
    
    /// PerSession - dedicated connection per session (WebSocket)
    PerSession,
    
    /// RoundRobin - distribute across pool (HTTP/1.1)
    RoundRobin { pool_size: usize },
}

impl PoolingStrategy {
    pub fn for_protocol(protocol: Protocol) -> Self {
        match protocol {
            Protocol::Stdio => PoolingStrategy::Singleton,
            Protocol::Http2 => PoolingStrategy::PerOrigin { max_per_origin: 100 },
            Protocol::WebSocket => PoolingStrategy::PerSession,
            Protocol::Http => PoolingStrategy::RoundRobin { pool_size: 10 },
            _ => PoolingStrategy::RoundRobin { pool_size: 5 },
        }
    }
}
```

## Connection Manager Using Shadowcat's Pool

```rust
// crates/mcp/src/connection/manager.rs

use shadowcat::pool::{Pool, PoolOptions, PoolHooks, PoolConnectionMetadata};
use dashmap::DashMap;
use std::sync::Arc;

pub struct ConnectionManager {
    /// Pools by origin (for HTTP/2 multiplexing)
    origin_pools: Arc<DashMap<String, Pool<PoolableConnection>>>,
    
    /// Session-dedicated connections (for WebSocket)
    session_connections: Arc<DashMap<String, Box<dyn Connection>>>,
    
    /// Singleton connections (for stdio)
    singletons: Arc<DashMap<Protocol, Box<dyn Connection>>>,
    
    /// Default pool options
    default_options: PoolOptions,
    
    /// Metrics
    metrics: Arc<ConnectionMetrics>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            origin_pools: Arc::new(DashMap::new()),
            session_connections: Arc::new(DashMap::new()),
            singletons: Arc::new(DashMap::new()),
            default_options: PoolOptions {
                max_connections: 100,  // Per origin for HTTP/2
                acquire_timeout: Duration::from_secs(5),
                idle_timeout: Some(Duration::from_secs(300)),
                max_lifetime: Some(Duration::from_secs(3600)),
                health_check_interval: Duration::from_secs(30),
            },
            metrics: Arc::new(ConnectionMetrics::new()),
        }
    }
    
    /// Get or create a connection based on URL and protocol
    pub async fn get_connection(
        &self,
        url: &Url,
        session_id: Option<&str>,
    ) -> Result<Box<dyn Connection>> {
        let protocol = Protocol::from_url(url);
        let strategy = PoolingStrategy::for_protocol(protocol);
        
        match strategy {
            PoolingStrategy::Singleton => {
                self.get_singleton(protocol).await
            }
            
            PoolingStrategy::PerOrigin { .. } => {
                self.get_from_origin_pool(url, protocol).await
            }
            
            PoolingStrategy::PerSession => {
                let session_id = session_id.ok_or("WebSocket requires session_id")?;
                self.get_session_connection(url, session_id).await
            }
            
            PoolingStrategy::RoundRobin { .. } => {
                self.get_from_round_robin_pool(url, protocol).await
            }
        }
    }
    
    /// Get connection from origin-based pool (HTTP/2)
    async fn get_from_origin_pool(
        &self,
        url: &Url,
        protocol: Protocol,
    ) -> Result<Box<dyn Connection>> {
        let origin = format!("{}://{}", url.scheme(), url.host_str().unwrap());
        
        // Get or create pool for this origin
        let pool = self.origin_pools
            .entry(origin.clone())
            .or_insert_with(|| {
                // Create pool with hooks for HTTP/2 specific behavior
                let hooks = PoolHooks::<PoolableConnection> {
                    after_create: Some(Arc::new(move |conn, _meta| {
                        Box::pin(async move {
                            // HTTP/2 handshake, SETTINGS frame, etc.
                            conn.inner.initialize().await?;
                            Ok(())
                        })
                    })),
                    
                    before_acquire: Some(Arc::new(|conn, meta| {
                        Box::pin(async move {
                            // Check if connection is still multiplexable
                            if meta.age > Duration::from_secs(1800) {
                                // After 30 min, prefer fresh connection
                                Ok(false)
                            } else {
                                Ok(conn.inner.can_multiplex().await)
                            }
                        })
                    })),
                    
                    after_release: Some(Arc::new(|conn, _meta| {
                        Box::pin(async move {
                            // Keep HTTP/2 connections alive for multiplexing
                            let requests = conn.request_count.load(Ordering::Relaxed);
                            if requests > 10000 {
                                // Close after many requests to prevent issues
                                Ok(false)
                            } else {
                                Ok(true)
                            }
                        })
                    })),
                };
                
                Pool::new_with_hooks(self.default_options.clone(), hooks)
            })
            .clone();
        
        // Acquire from pool
        let mut pool_conn = pool.acquire(|| {
            let url = url.clone();
            let origin = origin.clone();
            let protocol = protocol.clone();
            async move {
                // Factory creates new HTTP/2 connection
                let inner = Http2Connection::new(url).await?;
                Ok(PoolableConnection {
                    inner: Box::new(inner),
                    origin,
                    protocol,
                    created_at: Instant::now(),
                    last_used: Arc::new(RwLock::new(Instant::now())),
                    request_count: Arc::new(AtomicU64::new(0)),
                })
            }
        }).await?;
        
        // Extract the connection
        let conn = pool_conn.resource().inner.clone();
        self.metrics.connection_acquired(&origin);
        
        Ok(conn)
    }
    
    /// Get singleton connection (stdio)
    async fn get_singleton(&self, protocol: Protocol) -> Result<Box<dyn Connection>> {
        self.singletons
            .entry(protocol)
            .or_try_insert_with(|| async {
                match protocol {
                    Protocol::Stdio => Ok(Box::new(StdioConnection::new()) as Box<dyn Connection>),
                    _ => Err("Not a singleton protocol"),
                }
            })
            .await?
            .clone()
    }
    
    /// Get session-dedicated connection (WebSocket)
    async fn get_session_connection(
        &self,
        url: &Url,
        session_id: &str,
    ) -> Result<Box<dyn Connection>> {
        self.session_connections
            .entry(session_id.to_string())
            .or_try_insert_with(|| async {
                let ws = WebSocketConnection::connect(url, session_id).await?;
                Ok(Box::new(ws) as Box<dyn Connection>)
            })
            .await?
            .clone()
    }
}
```

## Integration with Shadowcat Proxy

```rust
// In shadowcat's main proxy code

pub struct Proxy {
    /// Connection manager with pooling
    connections: Arc<ConnectionManager>,
    
    /// Session manager
    sessions: Arc<SessionManager>,
}

impl Proxy {
    pub async fn handle_request(
        &self,
        session: &Session,
        request: Value,
    ) -> Result<Value> {
        // Get connection - manager handles pooling strategy
        let mut conn = self.connections
            .get_connection(&session.upstream_url, Some(&session.id))
            .await?;
        
        // Send request through connection
        let response = conn.request(request).await?;
        
        // Connection automatically returned to pool on drop
        Ok(response)
    }
}
```

## Advantages of Using Shadowcat's Pool

### 1. **Battle-Tested**
- Already in production use
- Handles edge cases (double close, maintenance, shutdown)
- Weak references prevent leaks

### 2. **Feature-Rich**
- Health checks with configurable intervals
- Idle timeout and max lifetime
- Graceful shutdown
- Metrics and observability

### 3. **Flexible Hooks**
- `after_create`: Initialize connections (HTTP/2 handshake)
- `before_acquire`: Validate before reuse (check multiplexing)
- `after_release`: Decide whether to keep alive

### 4. **Performance**
- Semaphore-based capacity control (no spinning)
- Efficient idle queue management
- Automatic cleanup of unhealthy connections

## Protocol-Specific Behaviors

### HTTP/2 Multiplexing
```rust
// Connections are long-lived and shared
PoolOptions {
    max_connections: 100,      // High - one per origin
    idle_timeout: Some(Duration::from_secs(1800)),  // 30 min
    max_lifetime: Some(Duration::from_secs(7200)),  // 2 hours
}
```

### WebSocket (Not Pooled)
```rust
// Each session gets dedicated connection
// Not using pool, just DashMap for session mapping
session_connections.insert(session_id, websocket_conn);
```

### Stdio (Singleton)
```rust
// One global connection, never expires
singletons.insert(Protocol::Stdio, stdio_conn);
```

### HTTP/1.1 (Round-Robin Pool)
```rust
// Small pool with quick recycling
PoolOptions {
    max_connections: 10,       // Small pool
    idle_timeout: Some(Duration::from_secs(60)),   // 1 min
    max_lifetime: Some(Duration::from_secs(300)),  // 5 min
}
```

## Metrics and Monitoring

```rust
pub struct ConnectionMetrics {
    connections_created: Counter,
    connections_reused: Counter,
    connections_closed: Counter,
    active_connections: Gauge,
    multiplexed_streams: Histogram,
    pool_wait_time: Histogram,
}

// Integrated with pool hooks
after_create: Some(Arc::new(move |conn, _| {
    metrics.connections_created.inc();
    Box::pin(async { Ok(()) })
}))
```

## Migration Path

### Phase 1: Wrap Existing Connections
1. Create `PoolableConnection` wrapper
2. Implement `PoolableResource` trait
3. Test with single protocol

### Phase 2: Add Pool Management
1. Create `ConnectionManager` with origin pools
2. Integrate shadowcat's `Pool<T>`
3. Add protocol-specific strategies

### Phase 3: Optimize Pooling
1. Tune pool options per protocol
2. Add comprehensive metrics
3. Implement connection warming

## Success Criteria

1. **HTTP/2**: 10K clients share ~100 connections
2. **Memory**: < 10MB overhead for connection pooling
3. **Latency**: < 100µs to acquire connection from pool
4. **Reuse Rate**: > 95% for HTTP/2 connections

## Conclusion

Shadowcat's generic pool is perfect for our Connection pattern:
- Already handles all the complex pooling logic
- Hooks provide protocol-specific customization
- Proven in production with comprehensive tests
- Zero additional dependencies

We just need to:
1. Make Connection implement PoolableResource
2. Create ConnectionManager with protocol strategies
3. Configure pools appropriately per protocol

This gives us enterprise-grade connection pooling with minimal code!