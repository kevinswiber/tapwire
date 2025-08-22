# Task A.3: Interface Design

## Objective
Design clean, testable interfaces (traits) that define contracts between modules and enable dependency injection.

## Context
Good interfaces are crucial for:
- Testing components in isolation
- Swapping implementations
- Managing dependencies
- Future extensibility

## Deliverables

### 1. Core Trait Definitions
Create `analysis/trait-definitions.rs`:
```rust
// Complete trait definitions with:
// - Documentation
// - Error types
// - Async considerations
// - Lifetime requirements
```

### 2. Dependency Injection Design
Document in `analysis/dependency-injection.md`:
- How components get their dependencies
- Builder pattern usage
- Factory patterns
- Service locator vs constructor injection

### 3. Error Handling Strategy
Create `analysis/error-strategy.md`:
- Error types per module
- Error propagation
- Error transformation
- Client-facing errors

### 4. Testing Strategy
Document in `analysis/testing-approach.md`:
- Mock implementations
- Test fixtures
- Integration test setup
- Benchmark approach

## Interface Designs

### Handler Traits
```rust
/// Core handler abstraction
#[async_trait]
pub trait RequestHandler: Send + Sync + 'static {
    /// Handle an incoming request
    async fn handle(
        &self,
        request: Request<Body>,
        context: RequestContext,
    ) -> Result<Response<Body>, HandlerError>;
    
    /// Handler name for metrics/logging
    fn name(&self) -> &str;
    
    /// Check if handler can process this request
    fn can_handle(&self, request: &Request<Body>) -> bool;
}

/// Context passed to handlers
pub struct RequestContext {
    pub session: Arc<Session>,
    pub app_state: Arc<AppState>,
    pub trace_id: Uuid,
    pub start_time: Instant,
}
```

### Upstream Management
```rust
/// Upstream server selection
#[async_trait]
pub trait UpstreamSelector: Send + Sync + 'static {
    /// Select an upstream for a request
    async fn select(
        &self,
        request: &Request<Body>,
        session: &Session,
    ) -> Result<UpstreamHandle, SelectionError>;
    
    /// Report upstream health
    async fn report_health(
        &self,
        upstream_id: &str,
        status: HealthStatus,
    );
    
    /// Get all upstreams
    async fn list_upstreams(&self) -> Vec<UpstreamInfo>;
}

/// Handle to an upstream connection
pub struct UpstreamHandle {
    pub id: String,
    pub transport: Box<dyn Transport>,
    pub config: UpstreamConfig,
}
```

### Session Operations
```rust
/// Session management operations
#[async_trait]
pub trait SessionOps: Send + Sync + 'static {
    /// Get or create session
    async fn get_or_create(
        &self,
        session_id: SessionId,
        headers: &HeaderMap,
    ) -> Result<Arc<Session>, SessionError>;
    
    /// Update session state
    async fn update(
        &self,
        session_id: &SessionId,
        update: SessionUpdate,
    ) -> Result<(), SessionError>;
    
    /// Session metrics
    async fn metrics(&self) -> SessionMetrics;
}
```

### Middleware Interface
```rust
/// Middleware that can intercept requests/responses
#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    /// Process request before handler
    async fn pre_process(
        &self,
        request: &mut Request<Body>,
        context: &mut RequestContext,
    ) -> Result<MiddlewareAction, MiddlewareError>;
    
    /// Process response after handler
    async fn post_process(
        &self,
        response: &mut Response<Body>,
        context: &RequestContext,
    ) -> Result<(), MiddlewareError>;
    
    /// Middleware name
    fn name(&self) -> &str;
    
    /// Execution priority (lower = earlier)
    fn priority(&self) -> i32;
}

pub enum MiddlewareAction {
    Continue,
    Respond(Response<Body>),
    Reject(RejectionReason),
}
```

### Builder Interfaces
```rust
/// Builder for constructing the server
pub trait ServerBuilder {
    type Server;
    type Error;
    
    /// Set configuration
    fn with_config(self, config: ReverseProxyConfig) -> Self;
    
    /// Add handler
    fn add_handler(self, handler: Box<dyn RequestHandler>) -> Self;
    
    /// Add middleware
    fn add_middleware(self, middleware: Box<dyn Middleware>) -> Self;
    
    /// Set upstream selector
    fn with_upstream_selector(self, selector: Box<dyn UpstreamSelector>) -> Self;
    
    /// Build the server
    async fn build(self) -> Result<Self::Server, Self::Error>;
}
```

## Error Types
```rust
/// Handler-specific errors
#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Upstream error: {0}")]
    UpstreamError(#[from] UpstreamError),
    
    #[error("Session error: {0}")]
    SessionError(#[from] SessionError),
    
    #[error("Internal error: {0}")]
    Internal(#[source] Box<dyn std::error::Error + Send + Sync>),
}
```

## Success Criteria
- [ ] All major interfaces defined
- [ ] Error handling strategy clear
- [ ] Async traits properly designed
- [ ] Testability considered
- [ ] Extension points identified

## Estimated Time
2 hours

## Design Considerations

### Async Trait Challenges
- Use `#[async_trait]` for now
- Consider future migration to native async traits
- Be careful with lifetimes in async traits
- Avoid unnecessary boxing

### Testability
- All traits should be mockable
- Avoid concrete types in signatures
- Use dependency injection
- Provide test implementations

### Performance
- Minimize allocations in hot paths
- Use `Arc` for shared immutable data
- Consider `&str` vs `String` carefully
- Profile trait object overhead

## Notes
- These interfaces will be in `src/proxy/reverse/traits.rs`
- Mock implementations go in `src/proxy/reverse/mocks.rs`
- Consider a separate `shadowcat-reverse-proxy` crate for the interfaces
- Builder pattern provides flexibility for construction