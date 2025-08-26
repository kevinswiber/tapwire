# Task E.0: Public API Design with Builder Pattern

## Objective
Design the public API for the MCP crate using the builder pattern for maximum flexibility and discoverability, allowing component sharing between instances.

## Background
From Gemini's review: "A simple `Server::new(config)` can be inflexible. Adopt a typed builder pattern for Server and Client to provide a fluent, discoverable, and type-safe way to configure components."

## Key Requirements

### 1. Server Builder API
```rust
use mcp::{Server, ServerBuilder, Handler, SessionManager, InterceptorChain};

// Basic usage
let server = Server::builder()
    .bind("127.0.0.1:8080")
    .handler(my_handler)
    .build()
    .await?;

// Advanced usage with component sharing
let shared_session_manager = Arc::new(SessionManager::new(session_config).await?);
let shared_interceptors = Arc::new(InterceptorChain::new());

let server1 = Server::builder()
    .bind("127.0.0.1:8080")
    .session_manager(shared_session_manager.clone())
    .interceptor_chain(shared_interceptors.clone())
    .handler(handler1)
    .build()
    .await?;

let server2 = Server::builder()
    .bind("127.0.0.1:8081")
    .session_manager(shared_session_manager.clone())
    .interceptor_chain(shared_interceptors.clone())
    .handler(handler2)
    .build()
    .await?;

// Full configuration
let server = Server::builder()
    .bind("127.0.0.1:8080")
    .max_connections(1000)
    .shutdown_timeout(Duration::from_secs(30))
    .enable_sse(true)
    .enable_websocket(false)
    .session_manager(Arc::new(SessionManager::new(session_config).await?))
    .interceptor(AuthInterceptor::new())
    .interceptor(RateLimitInterceptor::new())
    .interceptor(MetricsInterceptor::new())
    .handler(my_handler)
    .with_metrics(metrics_config)
    .build()
    .await?;
```

### 2. ServerBuilder Implementation
```rust
pub struct ServerBuilder {
    bind_address: Option<String>,
    max_connections: Option<usize>,
    shutdown_timeout: Option<Duration>,
    enable_sse: Option<bool>,
    enable_websocket: Option<bool>,
    session_manager: Option<Arc<SessionManager>>,
    interceptor_chain: Option<Arc<InterceptorChain>>,
    interceptors: Vec<Box<dyn Interceptor>>,
    handler: Option<Box<dyn Handler>>,
    metrics_config: Option<MetricsConfig>,
}

impl ServerBuilder {
    pub fn new() -> Self {
        Self {
            bind_address: None,
            max_connections: None,
            shutdown_timeout: None,
            enable_sse: None,
            enable_websocket: None,
            session_manager: None,
            interceptor_chain: None,
            interceptors: Vec::new(),
            handler: None,
            metrics_config: None,
        }
    }
    
    pub fn bind(mut self, address: impl Into<String>) -> Self {
        self.bind_address = Some(address.into());
        self
    }
    
    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = Some(max);
        self
    }
    
    pub fn shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = Some(timeout);
        self
    }
    
    pub fn enable_sse(mut self, enable: bool) -> Self {
        self.enable_sse = Some(enable);
        self
    }
    
    pub fn enable_websocket(mut self, enable: bool) -> Self {
        self.enable_websocket = Some(enable);
        self
    }
    
    pub fn session_manager(mut self, manager: Arc<SessionManager>) -> Self {
        self.session_manager = Some(manager);
        self
    }
    
    pub fn interceptor_chain(mut self, chain: Arc<InterceptorChain>) -> Self {
        self.interceptor_chain = Some(chain);
        self
    }
    
    pub fn interceptor(mut self, interceptor: impl Interceptor + 'static) -> Self {
        self.interceptors.push(Box::new(interceptor));
        self
    }
    
    pub fn handler(mut self, handler: impl Handler + 'static) -> Self {
        self.handler = Some(Box::new(handler));
        self
    }
    
    pub fn with_metrics(mut self, config: MetricsConfig) -> Self {
        self.metrics_config = Some(config);
        self
    }
    
    pub async fn build(self) -> Result<Server, Error> {
        // Validation
        let bind_address = self.bind_address
            .ok_or_else(|| Error::Config("bind address required".into()))?;
        
        let handler = self.handler
            .ok_or_else(|| Error::Config("handler required".into()))?;
        
        // Create defaults or use provided components
        let session_manager = match self.session_manager {
            Some(manager) => manager,
            None => {
                let config = SessionConfig::default();
                Arc::new(SessionManager::new(config).await?)
            }
        };
        
        let interceptor_chain = match self.interceptor_chain {
            Some(chain) => chain,
            None if self.interceptors.is_empty() => Arc::new(InterceptorChain::new()),
            None => {
                let mut chain = InterceptorChain::new();
                for interceptor in self.interceptors {
                    chain.register(interceptor);
                }
                Arc::new(chain)
            }
        };
        
        // Initialize metrics if configured
        if let Some(metrics_config) = self.metrics_config {
            init_metrics(&metrics_config)?;
        }
        
        // Build server config
        let config = ServerConfig {
            bind_address,
            max_connections: self.max_connections.unwrap_or(1000),
            shutdown_timeout: self.shutdown_timeout.unwrap_or(Duration::from_secs(30)),
            enable_sse: self.enable_sse.unwrap_or(true),
            enable_websocket: self.enable_websocket.unwrap_or(false),
        };
        
        Ok(Server {
            config,
            session_manager,
            interceptor_chain,
            handler,
            // ... other fields
        })
    }
}
```

### 3. Client Builder API
```rust
use mcp::{Client, ClientBuilder, Transport};

// Simple usage
let client = Client::builder()
    .transport(StdioTransport::new())
    .build()
    .await?;

// With connection pooling
let client = Client::builder()
    .transport(HttpTransport::new("http://localhost:8080"))
    .pool_config(PoolConfig {
        max_connections: 50,
        idle_timeout: Duration::from_secs(300),
        ..Default::default()
    })
    .build()
    .await?;

// Full configuration with retry and interceptors
let client = Client::builder()
    .transport(HttpTransport::new("http://localhost:8080"))
    .pool_config(pool_config)
    .retry_config(RetryConfig {
        max_retries: 3,
        initial_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(60),
        ..Default::default()
    })
    .interceptor(LoggingInterceptor::new())
    .interceptor(AuthInterceptor::new(token))
    .request_timeout(Duration::from_secs(30))
    .enable_reconnection(true)
    .with_metrics(metrics_config)
    .build()
    .await?;
```

### 4. ClientBuilder Implementation
```rust
pub struct ClientBuilder {
    transport: Option<Box<dyn Transport>>,
    pool_config: Option<PoolConfig>,
    retry_config: Option<RetryConfig>,
    interceptors: Vec<Box<dyn Interceptor>>,
    request_timeout: Option<Duration>,
    enable_reconnection: Option<bool>,
    metrics_config: Option<MetricsConfig>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            transport: None,
            pool_config: None,
            retry_config: None,
            interceptors: Vec::new(),
            request_timeout: None,
            enable_reconnection: None,
            metrics_config: None,
        }
    }
    
    pub fn transport(mut self, transport: impl Transport + 'static) -> Self {
        self.transport = Some(Box::new(transport));
        self
    }
    
    pub fn pool_config(mut self, config: PoolConfig) -> Self {
        self.pool_config = Some(config);
        self
    }
    
    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = Some(config);
        self
    }
    
    pub fn interceptor(mut self, interceptor: impl Interceptor + 'static) -> Self {
        self.interceptors.push(Box::new(interceptor));
        self
    }
    
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = Some(timeout);
        self
    }
    
    pub fn enable_reconnection(mut self, enable: bool) -> Self {
        self.enable_reconnection = Some(enable);
        self
    }
    
    pub fn with_metrics(mut self, config: MetricsConfig) -> Self {
        self.metrics_config = Some(config);
        self
    }
    
    pub async fn build(self) -> Result<Client, Error> {
        let transport = self.transport
            .ok_or_else(|| Error::Config("transport required".into()))?;
        
        // Create connection pool
        let pool = if let Some(config) = self.pool_config {
            Some(Pool::new(config.into())?)
        } else {
            None
        };
        
        // Create interceptor chain
        let interceptor_chain = if self.interceptors.is_empty() {
            None
        } else {
            let mut chain = InterceptorChain::new();
            for interceptor in self.interceptors {
                chain.register(interceptor);
            }
            Some(Arc::new(chain))
        };
        
        // Initialize metrics if configured
        if let Some(metrics_config) = self.metrics_config {
            init_metrics(&metrics_config)?;
        }
        
        let config = ClientConfig {
            request_timeout: self.request_timeout.unwrap_or(Duration::from_secs(30)),
            enable_reconnection: self.enable_reconnection.unwrap_or(true),
            retry_config: self.retry_config.unwrap_or_default(),
        };
        
        Ok(Client {
            transport,
            pool,
            interceptor_chain,
            config,
            // ... other fields
        })
    }
}
```

### 5. Component Sharing Patterns

#### Shared SessionManager
```rust
// Create a shared session manager for multiple servers
let session_manager = Arc::new(
    SessionManager::builder()
        .store(RedisSessionStore::new(redis_config).await?)
        .max_sessions(10000)
        .cleanup_interval(Duration::from_secs(60))
        .enable_heartbeat(true)
        .build()
        .await?
);

// Use in multiple servers
let api_server = Server::builder()
    .bind("0.0.0.0:8080")
    .session_manager(session_manager.clone())
    .handler(api_handler)
    .build()
    .await?;

let admin_server = Server::builder()
    .bind("127.0.0.1:9090")
    .session_manager(session_manager.clone())
    .handler(admin_handler)
    .build()
    .await?;
```

#### Shared InterceptorChain
```rust
// Create a shared interceptor chain
let interceptors = InterceptorChain::builder()
    .register(AuthInterceptor::new())
    .register(RateLimitInterceptor::new())
    .register(MetricsInterceptor::new())
    .build();

let interceptors = Arc::new(interceptors);

// Use in both server and client
let server = Server::builder()
    .bind("0.0.0.0:8080")
    .interceptor_chain(interceptors.clone())
    .handler(handler)
    .build()
    .await?;

let client = Client::builder()
    .transport(transport)
    .interceptor_chain(interceptors.clone())
    .build()
    .await?;
```

### 6. Convenience Methods
```rust
impl Server {
    /// Create a new server with minimal configuration
    pub async fn new(
        bind: impl Into<String>,
        handler: impl Handler + 'static,
    ) -> Result<Self, Error> {
        Self::builder()
            .bind(bind)
            .handler(handler)
            .build()
            .await
    }
}

impl Client {
    /// Create a new client with minimal configuration
    pub async fn new(transport: impl Transport + 'static) -> Result<Self, Error> {
        Self::builder()
            .transport(transport)
            .build()
            .await
    }
}
```

## Implementation Steps

1. **Design builder structs** (1 hour)
   - ServerBuilder with all options
   - ClientBuilder with all options
   - Type-safe state tracking

2. **Implement ServerBuilder** (2 hours)
   - Fluent API methods
   - Validation in build()
   - Component creation/sharing logic

3. **Implement ClientBuilder** (1.5 hours)
   - Transport configuration
   - Pool setup
   - Retry configuration

4. **Add component builders** (1 hour)
   - SessionManagerBuilder
   - InterceptorChainBuilder
   - For complex component creation

5. **Create convenience methods** (30 min)
   - Simple constructors for common cases
   - Default configurations

6. **Write documentation** (1 hour)
   - Builder pattern examples
   - Component sharing patterns
   - Migration guide from config-based API

## Testing Strategy

1. **Unit Tests**
   - Builder state transitions
   - Validation logic
   - Default value handling

2. **Integration Tests**
   - Component sharing between instances
   - Builder with all options
   - Error cases (missing required fields)

3. **Example Programs**
   - Simple server/client
   - Multi-server with shared sessions
   - Client with connection pooling

## Success Criteria

- [ ] Fluent, discoverable API
- [ ] Type-safe builder pattern
- [ ] Components shareable between instances
- [ ] Backwards-compatible convenience methods
- [ ] Comprehensive documentation

## Risk Mitigation

1. **API Complexity**: Provide simple defaults and convenience methods
2. **Breaking Changes**: Keep old constructors with deprecation notices
3. **Type Inference Issues**: Use explicit type parameters where needed

## Dependencies
- Core components must be implemented
- Configuration types defined

## Estimated Duration
6 hours

## Notes
- Consider using the `typed-builder` crate for boilerplate reduction
- Builder pattern allows future extension without breaking changes
- Component sharing is critical for microservices architectures