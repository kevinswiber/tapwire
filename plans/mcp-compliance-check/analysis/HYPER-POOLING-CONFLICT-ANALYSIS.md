# Hyper Connection Pooling vs Our Pool - Conflict Analysis

**Date**: 2025-08-24  
**Status**: Critical Design Issue  
**Question**: Are we creating double pooling with performance overhead?

## TL;DR

**YES, we would have double pooling!** But we can avoid it by:
1. **Option A**: Disable Hyper's pool and use only ours (recommended)
2. **Option B**: Use Hyper's pool directly without our wrapper
3. **Option C**: Use lower-level `hyper::client::conn` APIs

## Hyper's Built-in Connection Pooling

### What Hyper Does

```rust
// Hyper Client has built-in pooling!
let client = Client::builder()
    .pool_idle_timeout(Duration::from_secs(90))  // Default: 90s
    .pool_max_idle_per_host(100)                 // Default: usize::MAX
    .http2_only(true)                            // Forces HTTP/2
    .http2_initial_stream_window_size(1024 * 1024)
    .http2_initial_connection_window_size(1024 * 1024 * 10)
    .build::<_, Body>(connector);
```

### Hyper's Pool Features
- **Per-origin pooling** (exactly what we want!)
- **HTTP/2 multiplexing** (one connection per origin)
- **Automatic connection reuse**
- **Health checks via TCP keepalive**
- **Idle timeout management**

## The Double Pooling Problem

### Current Design Stack
```
Our Pool (shadowcat::pool)
    ↓
PoolableConnection wrapper
    ↓
Http2Connection
    ↓
hyper::Client (with its own pool!)  ← PROBLEM!
    ↓
TCP connections
```

### Performance Impact
1. **Double overhead**: Two pools managing same resources
2. **Fighting pools**: Our pool might close what Hyper wants open
3. **Memory waste**: Two sets of metadata per connection
4. **Complexity**: Debugging which pool is doing what

## Solution Options

### Option A: Disable Hyper Pool, Use Ours (RECOMMENDED)

```rust
// crates/mcp/src/connection/http.rs

pub struct Http2Connection {
    // Create new client per "connection" with pooling disabled
    client: Client<HttpsConnector<HttpConnector>>,
    url: Url,
    session_id: Option<String>,
}

impl Http2Connection {
    pub fn new(url: Url) -> Self {
        let https = HttpsConnector::new();
        let client = Client::builder()
            .pool_idle_timeout(None)  // ← DISABLE Hyper's pool!
            .pool_max_idle_per_host(1) // ← Only this connection
            .http2_only(true)
            .build::<_, Body>(https);
        
        Self { client, url, session_id: None }
    }
}

// Our pool manages these Http2Connection instances
```

**Pros:**
- Full control over pooling strategy
- Consistent with other protocols (WebSocket, stdio)
- Can implement MCP-specific session affinity
- Shadowcat's pool handles all lifecycle

**Cons:**
- Lose Hyper's optimizations
- Need to manage HTTP/2 PING frames ourselves

### Option B: Use Hyper's Pool Directly (NO WRAPPER)

```rust
// Don't wrap hyper::Client in a Connection at all!

pub struct HttpConnectionManager {
    // Share one hyper::Client across everything
    client: Arc<Client<HttpsConnector<HttpConnector>>>,
}

impl HttpConnectionManager {
    pub async fn request(&self, url: &Url, msg: Value) -> Result<Value> {
        // Hyper handles all pooling internally
        let request = self.build_request(url, msg)?;
        let response = self.client.request(request).await?;
        self.parse_response(response).await
    }
}
```

**Pros:**
- Leverage Hyper's battle-tested pooling
- Minimal code
- Best performance for HTTP

**Cons:**
- Inconsistent with WebSocket/stdio pattern
- Can't use shadowcat::pool features
- Less control over session affinity

### Option C: Use Lower-Level APIs (MOST CONTROL)

```rust
// Use hyper::client::conn for manual connection management

use hyper::client::conn::{self, SendRequest};
use tokio::net::TcpStream;

pub struct Http2Connection {
    sender: SendRequest<Body>,
    connection: conn::Connection<TcpStream, Body>,
}

impl Http2Connection {
    pub async fn connect(url: &Url) -> Result<Self> {
        let stream = TcpStream::connect(url.socket_addrs()?).await?;
        let (sender, connection) = conn::Builder::new()
            .http2_only(true)
            .handshake(stream)
            .await?;
        
        // Must drive the connection
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });
        
        Ok(Self { sender })
    }
}
```

**Pros:**
- Complete control
- No double pooling
- Can implement custom multiplexing

**Cons:**
- Much more complex
- Need to handle all HTTP/2 details
- More code to maintain

## Performance Comparison

### Double Pooling (Current Path)
```
Memory: 2x metadata (~1KB per connection)
CPU: 2x health checks, 2x idle cleanup
Latency: +~100ns for double acquire/release
Complexity: HIGH (two systems fighting)
```

### Single Pool (Option A - Our Pool Only)
```
Memory: 1x metadata (~500B per connection)
CPU: 1x health checks, our control
Latency: Baseline
Complexity: MEDIUM (we manage everything)
```

### Hyper Pool Only (Option B)
```
Memory: Minimal (Hyper is optimized)
CPU: Hyper's optimized internals
Latency: Baseline (possibly better)
Complexity: LOW (trust Hyper)
```

## Recommendation: Option A with Optimizations

Use our pool but disable Hyper's:

```rust
pub struct Http2Connection {
    // Each "connection" is a dedicated hyper::Client
    client: Client<HttpsConnector<HttpConnector>>,
    metrics: Arc<ConnectionMetrics>,
}

impl Http2Connection {
    pub fn new(url: Url) -> Self {
        let client = Client::builder()
            .pool_idle_timeout(None)     // Disable Hyper pool
            .pool_max_idle_per_host(1)    // Just this connection
            .http2_only(true)
            .http2_keep_alive_interval(Some(Duration::from_secs(30)))
            .http2_keep_alive_timeout(Duration::from_secs(10))
            .build::<_, Body>(HttpsConnector::new());
        
        Self { client, metrics: Arc::new(ConnectionMetrics::new()) }
    }
}

// Shadowcat's pool manages Http2Connection instances
impl PoolableResource for Http2Connection {
    async fn is_healthy(&self) -> bool {
        // We control health checks
        self.last_request.elapsed() < Duration::from_secs(60)
    }
}
```

### Why This Works Best

1. **Consistent Architecture**: All protocols use shadowcat::pool
2. **MCP-Specific Features**: Session affinity, custom health checks
3. **No Double Pooling**: Clear single owner of connections
4. **Proxy Optimized**: Designed for our scale needs

### Migration Path

1. **Phase 1**: Implement with Hyper pooling disabled
2. **Phase 2**: Benchmark against Hyper-only approach
3. **Phase 3**: Optimize based on production metrics

## Alternative: Hybrid Approach

For HTTP specifically, we could skip our Connection trait:

```rust
enum TransportBackend {
    // Use our Connection trait for these
    Stdio(Box<dyn Connection>),
    WebSocket(Box<dyn Connection>),
    
    // Use Hyper directly for HTTP
    Http(Arc<Client<HttpsConnector<HttpConnector>>>),
}
```

This gives us:
- Best performance for HTTP (Hyper's optimized pool)
- Consistent interface for other protocols
- Complexity isolated to HTTP path

## Conclusion

**We must avoid double pooling.** The performance hit isn't huge (~100ns) but the complexity is unnecessary.

**Recommended approach:**
1. Disable Hyper's pool, use shadowcat::pool exclusively
2. Each `Http2Connection` owns a non-pooling `hyper::Client`
3. Shadowcat's pool manages `Http2Connection` lifecycle

This gives us full control while avoiding double pooling overhead.

**Fallback option:**
If benchmarks show Hyper's pool is significantly better, we can use the hybrid approach where HTTP bypasses our Connection trait entirely.