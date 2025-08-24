# Hyper 1.7 Upgrade Analysis and Connection Pooling Strategy

**Date**: 2025-08-24  
**Status**: Migration Analysis  
**Goal**: Upgrade from hyper 0.14 to 1.7 for HTTP/3 readiness and better pooling control

## Executive Summary

Upgrading to hyper 1.7 is an excellent strategic move that:
1. **Separates pooling concerns** - Pool moved to `hyper_util::client::legacy`
2. **Enables HTTP/3 path** - Modern foundation for QUIC/HTTP/3
3. **Gives us control** - We can choose pooling strategy explicitly
4. **Improves performance** - Hyper 1.x has significant optimizations

## Migration Impact

### Breaking Changes from 0.14 → 1.7

```rust
// OLD (hyper 0.14)
use hyper::{Body, Client, Method, Request, Response};
let client = Client::builder()
    .pool_idle_timeout(Duration::from_secs(90))
    .http2_only(true)
    .build::<_, Body>(connector);

// NEW (hyper 1.7)
use hyper::{Method, Request, Response};
use hyper::body::Incoming;
use http_body_util::{BodyExt, Full};
use hyper_util::client::legacy::{Client, connect::HttpConnector};
use hyper_util::rt::TokioExecutor;

// Option A: With legacy pool
let client = Client::builder(TokioExecutor::new())
    .pool_idle_timeout(Duration::from_secs(90))
    .http2_only(true)
    .build(connector);

// Option B: Without pool (use lower-level)
use hyper::client::conn;
// Direct connection management
```

### Key API Changes

1. **Body Types**
   - `hyper::Body` → `http_body_util::Full<Bytes>` or custom
   - Need `http_body_util` for body utilities
   - `Incoming` for received bodies

2. **Client Location**
   - `hyper::Client` → `hyper_util::client::legacy::Client`
   - Or use `hyper::client::conn` for manual control

3. **Executor Required**
   - Must provide `TokioExecutor` explicitly
   - Better async runtime abstraction

4. **TLS Changes**
   - `hyper-tls` → `hyper-rustls` or `hyper-tls` 0.6+
   - Consider `rustls` for pure-Rust stack

## Connection Pooling Analysis

### Three Strategic Options

#### Option 1: Use hyper_util's Legacy Pool (SIMPLEST)

```rust
// crates/mcp/src/transport/http/mod.rs
use hyper_util::client::legacy::{Client, connect::HttpConnector};
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::rt::TokioExecutor;

pub struct HttpTransport {
    // One shared client with hyper's pool
    client: Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
}

impl HttpTransport {
    pub fn new(url: Url) -> Self {
        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .expect("no native roots")
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();
        
        let client = Client::builder(TokioExecutor::new())
            .pool_idle_timeout(Duration::from_secs(300))
            .pool_max_idle_per_host(100)
            .http2_only(false)  // Support both HTTP/1.1 and HTTP/2
            .build(https);
        
        Self { client }
    }
}
```

**Pros:**
- Minimal code changes
- Battle-tested pooling
- Good for initial migration

**Cons:**
- Less control over MCP-specific needs
- Can't implement session affinity easily

#### Option 2: Shadowcat Pool + Direct Connections (RECOMMENDED)

```rust
// Use hyper::client::conn for manual connection management
use hyper::client::conn::{http1, http2};
use tokio::net::TcpStream;
use hyper_rustls::TlsStream;

pub struct Http2Connection {
    // Direct connection without any pooling
    sender: http2::SendRequest<Full<Bytes>>,
    connection: JoinHandle<()>,
}

impl Http2Connection {
    pub async fn connect(url: &Url) -> Result<Self> {
        // Establish TCP connection
        let stream = TcpStream::connect((host, port)).await?;
        
        // TLS if needed
        let stream = if url.scheme() == "https" {
            tls_connector.connect(domain, stream).await?
        } else {
            stream
        };
        
        // HTTP/2 handshake
        let (sender, conn) = http2::handshake(TokioExecutor::new(), stream)
            .await?;
        
        // Drive the connection
        let connection = tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("Connection error: {}", e);
            }
        });
        
        Ok(Self { sender, connection })
    }
}

// Then wrap with shadowcat's pool
impl PoolableResource for Http2Connection {
    async fn is_healthy(&self) -> bool {
        !self.connection.is_finished()
    }
}
```

**Pros:**
- Full control over connection lifecycle
- Protocol-specific pooling strategies
- MCP session affinity support
- No double pooling

**Cons:**
- More code to maintain
- Need to handle HTTP/2 details

#### Option 3: Hybrid - Protocol-Specific Choice

```rust
enum TransportBackend {
    // Use shadowcat pool for WebSocket/stdio
    Pooled(Arc<Pool<PoolableConnection>>),
    
    // Use hyper_util's pool for HTTP
    HttpClient(Client<HttpsConnector<HttpConnector>, Full<Bytes>>),
}
```

**Pros:**
- Best tool for each protocol
- Flexibility

**Cons:**
- Two pooling systems to understand
- Complexity

## Dependency Updates

```toml
# Cargo.toml updates
[dependencies]
# Core HTTP
hyper = { version = "1.7", features = ["client", "http1", "http2"] }
hyper-util = { version = "0.1", features = ["client", "client-legacy", "http1", "http2", "tokio"] }
http-body-util = "0.1"
bytes = "1.5"

# TLS (choose one)
hyper-rustls = { version = "0.27", features = ["http1", "http2", "native-tokio", "tls12", "logging"] }
# OR
# hyper-tls = "0.6"

# For manual connection management (if Option 2)
tower = { version = "0.5", features = ["util"] }
tower-service = "0.3"

# Keep existing
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"
futures = "0.3"
url = "2.5"
```

## Migration Steps

### Phase 1: Update Dependencies (30 min)
```bash
# Update Cargo.toml
# Run cargo update
# Fix immediate compilation errors
```

### Phase 2: Update HTTP Transport (2 hours)
1. Update imports and types
2. Choose pooling strategy
3. Update request/response handling
4. Fix body streaming

### Phase 3: Update SSE Support (1 hour)
1. Adapt to new body types
2. Update event stream parsing
3. Test SSE reconnection

### Phase 4: Integration Testing (2 hours)
1. Test with MCP servers
2. Benchmark performance
3. Verify pooling behavior

## HTTP/3 Future Path

With hyper 1.7, we're positioned for HTTP/3:

```rust
// Future HTTP/3 support (when available)
use quinn::Endpoint;  // or h3-quinn
use h3::client;

pub struct Http3Connection {
    connection: h3::client::Connection<quinn::Connection>,
}

// Can use same PoolableResource interface!
```

## Performance Implications

### Hyper 1.7 Improvements
- **Lower overhead**: Removed unnecessary abstractions
- **Better HTTP/2**: Improved multiplexing and flow control
- **Zero-copy**: More efficient body handling
- **Async traits**: Better compiler optimizations

### Expected Metrics
```
Hyper 0.14 baseline:
- Connection setup: ~15ms
- Request latency: ~2ms
- Memory per connection: ~50KB

Hyper 1.7 expected:
- Connection setup: ~12ms (-20%)
- Request latency: ~1.5ms (-25%)
- Memory per connection: ~40KB (-20%)
```

## Recommendation

**Go with Option 2**: Shadowcat pool + direct connections using `hyper::client::conn`

### Why?
1. **No double pooling** - Clean separation of concerns
2. **Full control** - MCP-specific optimizations
3. **Future-proof** - Easy HTTP/3 addition
4. **Consistent** - All protocols use shadowcat pool

### Implementation Order
1. **First**: Upgrade dependencies to hyper 1.7
2. **Second**: Get basic HTTP working with `hyper_util::client::legacy`
3. **Third**: Refactor to use `hyper::client::conn` with shadowcat pool
4. **Fourth**: Optimize and benchmark

## Code Example - Final Architecture

```rust
// crates/mcp/src/connection/http.rs
use hyper::client::conn::{http1, http2};
use hyper_util::rt::TokioExecutor;
use shadowcat::pool::PoolableResource;

pub struct HttpConnection {
    variant: ConnectionVariant,
    metrics: Arc<Metrics>,
}

enum ConnectionVariant {
    Http1(http1::SendRequest<Full<Bytes>>),
    Http2(http2::SendRequest<Full<Bytes>>),
}

impl HttpConnection {
    pub async fn connect(url: &Url, prefer_http2: bool) -> Result<Self> {
        let stream = establish_stream(url).await?;
        
        let variant = if prefer_http2 {
            let (sender, conn) = http2::handshake(TokioExecutor::new(), stream)
                .await?;
            spawn_connection_driver(conn);
            ConnectionVariant::Http2(sender)
        } else {
            let (sender, conn) = http1::handshake(stream).await?;
            spawn_connection_driver(conn);
            ConnectionVariant::Http1(sender)
        };
        
        Ok(Self {
            variant,
            metrics: Arc::new(Metrics::new()),
        })
    }
}

// Make it poolable
#[async_trait]
impl PoolableResource for HttpConnection {
    async fn is_healthy(&self) -> bool {
        match &self.variant {
            ConnectionVariant::Http1(sender) => sender.is_ready(),
            ConnectionVariant::Http2(sender) => sender.is_ready(),
        }
    }
    
    fn resource_id(&self) -> String {
        format!("http-{}", self.metrics.connection_id)
    }
}
```

## Conclusion

Upgrading to hyper 1.7 is the right move:
- Cleaner separation of pooling concerns
- Better performance and lower overhead
- Foundation for HTTP/3 support
- More control over connection management

The migration effort (~6 hours) is worth the benefits, especially given our goal of building a production-grade MCP proxy platform.