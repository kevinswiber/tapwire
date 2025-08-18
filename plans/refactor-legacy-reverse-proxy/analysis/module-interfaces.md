# Module Interface Definitions

## Core Trait Definitions

### Request Handler Trait
```rust
// handlers/mod.rs
use axum::extract::Request;
use axum::response::Response;

/// Core trait for all request handlers
#[async_trait]
pub trait RequestHandler: Send + Sync + 'static {
    /// Handle an incoming request
    async fn handle(&self, ctx: RequestContext) -> Result<Response>;
    
    /// Check if handler can process this request
    fn can_handle(&self, req: &Request) -> bool;
    
    /// Handler priority for routing decisions
    fn priority(&self) -> u32 {
        100
    }
}

/// Context passed to handlers
pub struct RequestContext {
    pub request: Request,
    pub session: Arc<Session>,
    pub state: Arc<AppState>,
    pub span: tracing::Span,
}
```

### Transport Processor Trait
```rust
// transport/mod.rs
use serde_json::Value;

/// Abstraction for different transport mechanisms
#[async_trait]
pub trait TransportProcessor: Send + Sync + 'static {
    /// Process a message through this transport
    async fn process(
        &self,
        message: Value,
        session: &Session,
    ) -> Result<Value>;
    
    /// Check if transport is healthy
    async fn health_check(&self) -> Result<()>;
    
    /// Get transport metrics
    fn metrics(&self) -> TransportMetrics;
}

/// Transport selection strategy
#[async_trait]
pub trait TransportSelector: Send + Sync + 'static {
    /// Select appropriate transport for request
    async fn select(
        &self,
        session: &Session,
        upstream: &UpstreamConfig,
    ) -> Result<Box<dyn TransportProcessor>>;
}
```

### Session Management Traits
```rust
// session/mod.rs
use crate::mcp::SessionId;

/// Session lifecycle management
#[async_trait]
pub trait SessionManager: Send + Sync + 'static {
    /// Get or create a session
    async fn get_or_create(
        &self,
        id: SessionId,
        init_data: Option<SessionInitData>,
    ) -> Result<Arc<Session>>;
    
    /// Get existing session
    async fn get(&self, id: &SessionId) -> Result<Option<Arc<Session>>>;
    
    /// Update session state
    async fn update(&self, session: Arc<Session>) -> Result<()>;
    
    /// Remove session
    async fn remove(&self, id: &SessionId) -> Result<()>;
    
    /// List active sessions
    async fn list_active(&self) -> Result<Vec<SessionInfo>>;
}

/// Session initialization data
pub struct SessionInitData {
    pub transport_type: TransportType,
    pub upstream_id: String,
    pub client_info: Option<ClientInfo>,
}
```

### Middleware Traits
```rust
// middleware/mod.rs
use tower::Service;

/// Middleware that can intercept and modify requests/responses
#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    /// Process request before handler
    async fn pre_process(&self, req: &mut Request) -> Result<MiddlewareAction>;
    
    /// Process response after handler
    async fn post_process(&self, resp: &mut Response) -> Result<()>;
}

/// Action to take after middleware processing
pub enum MiddlewareAction {
    Continue,
    Respond(Response),
    Reject(ReverseProxyError),
}

/// Message interceptor for MCP protocol
#[async_trait]
pub trait MessageInterceptor: Send + Sync + 'static {
    /// Intercept outgoing message
    async fn intercept_request(
        &self,
        msg: &mut Value,
        context: &InterceptContext,
    ) -> Result<InterceptAction>;
    
    /// Intercept incoming response
    async fn intercept_response(
        &self,
        msg: &mut Value,
        context: &InterceptContext,
    ) -> Result<InterceptAction>;
}
```

### Upstream Management Traits
```rust
// config/upstream.rs
/// Upstream server selection
#[async_trait]
pub trait UpstreamSelector: Send + Sync + 'static {
    /// Select upstream based on strategy
    async fn select(
        &self,
        session: &Session,
        strategy: &LoadBalancingStrategy,
    ) -> Result<UpstreamConfig>;
    
    /// Mark upstream as healthy/unhealthy
    async fn mark_health(&self, upstream_id: &str, healthy: bool) -> Result<()>;
    
    /// Get all upstream statuses
    async fn get_statuses(&self) -> Result<Vec<UpstreamStatus>>;
}

/// Health check for upstream servers
#[async_trait]
pub trait HealthChecker: Send + Sync + 'static {
    /// Perform health check
    async fn check(&self, upstream: &UpstreamConfig) -> Result<HealthStatus>;
    
    /// Start periodic health checks
    fn start_monitoring(&self, upstream: UpstreamConfig) -> JoinHandle<()>;
}
```

### Metrics Collection Traits
```rust
// metrics.rs
/// Metrics collector interface
pub trait MetricsCollector: Send + Sync + 'static {
    /// Record request metrics
    fn record_request(&self, duration: Duration, success: bool);
    
    /// Record response size
    fn record_response_size(&self, bytes: usize);
    
    /// Record error
    fn record_error(&self, error_type: &str);
    
    /// Get current metrics snapshot
    fn snapshot(&self) -> MetricsSnapshot;
    
    /// Reset metrics
    fn reset(&self);
}

/// Metrics snapshot
#[derive(Clone, Debug, Serialize)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub total_bytes_transferred: u64,
}
```

### Recording Traits
```rust
// recorder/mod.rs
/// Session recording interface
#[async_trait]
pub trait SessionRecorder: Send + Sync + 'static {
    /// Record request
    async fn record_request(
        &self,
        session_id: SessionId,
        request: &Value,
    ) -> Result<()>;
    
    /// Record response
    async fn record_response(
        &self,
        session_id: SessionId,
        response: &Value,
    ) -> Result<()>;
    
    /// Start new recording session
    async fn start_session(&self, session_id: SessionId) -> Result<()>;
    
    /// End recording session
    async fn end_session(&self, session_id: SessionId) -> Result<()>;
}
```

### Rate Limiting Traits
```rust
// middleware/rate_limit.rs
/// Rate limiter interface
#[async_trait]
pub trait RateLimiter: Send + Sync + 'static {
    /// Check if request is allowed
    async fn check_rate_limit(
        &self,
        key: &str,
        tier: RateLimitTier,
    ) -> Result<RateLimitDecision>;
    
    /// Record request for rate limiting
    async fn record_request(&self, key: &str) -> Result<()>;
}

pub enum RateLimitDecision {
    Allow,
    Deny { retry_after_seconds: u64 },
}

pub enum RateLimitTier {
    Anonymous,
    Basic,
    Premium,
    Unlimited,
}
```

### Authentication Traits
```rust
// middleware/auth.rs
/// Authentication provider
#[async_trait]
pub trait AuthProvider: Send + Sync + 'static {
    /// Authenticate request
    async fn authenticate(&self, req: &Request) -> Result<AuthContext>;
    
    /// Validate token
    async fn validate_token(&self, token: &str) -> Result<TokenClaims>;
    
    /// Refresh token if needed
    async fn refresh_token(&self, token: &str) -> Result<String>;
}

/// Authorization provider
#[async_trait]
pub trait AuthorizationProvider: Send + Sync + 'static {
    /// Check if user is authorized for resource
    async fn authorize(
        &self,
        context: &AuthContext,
        resource: &str,
        action: &str,
    ) -> Result<bool>;
}
```

### Connection Pool Traits
```rust
// transport/pool.rs
/// Connection pool for reusable transports
#[async_trait]
pub trait ConnectionPool<T: PoolableTransport>: Send + Sync + 'static {
    /// Acquire connection from pool
    async fn acquire(&self) -> Result<PooledConnection<T>>;
    
    /// Return connection to pool
    async fn release(&self, conn: PooledConnection<T>);
    
    /// Get pool statistics
    fn stats(&self) -> PoolStats;
    
    /// Clear all connections
    async fn clear(&self);
}

/// Transport that can be pooled
pub trait PoolableTransport: Send + Sync + 'static {
    /// Check if transport is still valid
    async fn is_valid(&self) -> bool;
    
    /// Reset transport for reuse
    async fn reset(&self) -> Result<()>;
}
```

## Interface Usage Examples

### Implementing a Custom Handler
```rust
pub struct CustomHandler {
    config: HandlerConfig,
}

#[async_trait]
impl RequestHandler for CustomHandler {
    async fn handle(&self, ctx: RequestContext) -> Result<Response> {
        // Custom handling logic
        let session = ctx.session;
        let request = ctx.request;
        
        // Process request
        let response = process_custom_logic(request).await?;
        
        Ok(response)
    }
    
    fn can_handle(&self, req: &Request) -> bool {
        req.uri().path().starts_with("/custom")
    }
}
```

### Implementing a Custom Transport
```rust
pub struct WebSocketTransport {
    client: WsClient,
}

#[async_trait]
impl TransportProcessor for WebSocketTransport {
    async fn process(
        &self,
        message: Value,
        session: &Session,
    ) -> Result<Value> {
        let ws_message = self.client.send(message).await?;
        Ok(ws_message)
    }
    
    async fn health_check(&self) -> Result<()> {
        self.client.ping().await
    }
    
    fn metrics(&self) -> TransportMetrics {
        TransportMetrics {
            messages_sent: self.client.messages_sent(),
            messages_received: self.client.messages_received(),
            errors: self.client.errors(),
        }
    }
}
```

### Implementing Custom Middleware
```rust
pub struct LoggingMiddleware {
    logger: Logger,
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn pre_process(&self, req: &mut Request) -> Result<MiddlewareAction> {
        self.logger.log_request(req);
        Ok(MiddlewareAction::Continue)
    }
    
    async fn post_process(&self, resp: &mut Response) -> Result<()> {
        self.logger.log_response(resp);
        Ok(())
    }
}
```

## Dependency Injection Setup

### Service Registry
```rust
pub struct ServiceRegistry {
    handlers: HashMap<String, Arc<dyn RequestHandler>>,
    transports: HashMap<String, Arc<dyn TransportProcessor>>,
    middleware: Vec<Arc<dyn Middleware>>,
    session_manager: Arc<dyn SessionManager>,
    metrics: Arc<dyn MetricsCollector>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn register_handler(
        &mut self,
        name: &str,
        handler: Arc<dyn RequestHandler>,
    ) {
        self.handlers.insert(name.to_string(), handler);
    }
    
    pub fn register_transport(
        &mut self,
        name: &str,
        transport: Arc<dyn TransportProcessor>,
    ) {
        self.transports.insert(name.to_string(), transport);
    }
}
```

## Testing Support

### Mock Implementations
```rust
#[cfg(test)]
pub mod mocks {
    use super::*;
    
    pub struct MockSessionManager {
        sessions: Arc<Mutex<HashMap<SessionId, Arc<Session>>>>,
    }
    
    #[async_trait]
    impl SessionManager for MockSessionManager {
        async fn get_or_create(
            &self,
            id: SessionId,
            _init_data: Option<SessionInitData>,
        ) -> Result<Arc<Session>> {
            let mut sessions = self.sessions.lock().await;
            Ok(sessions.entry(id)
                .or_insert_with(|| Arc::new(Session::new(id)))
                .clone())
        }
        
        // Other methods...
    }
}
```

## Interface Versioning

### Version Compatibility
```rust
/// Versioned interface for backwards compatibility
pub trait RequestHandlerV1: Send + Sync {
    async fn handle_v1(&self, req: Request) -> Result<Response>;
}

pub trait RequestHandlerV2: RequestHandlerV1 {
    async fn handle_v2(&self, ctx: RequestContext) -> Result<Response>;
    
    // Default implementation for backwards compatibility
    async fn handle_v1(&self, req: Request) -> Result<Response> {
        let ctx = RequestContext::from_request(req);
        self.handle_v2(ctx).await
    }
}
```