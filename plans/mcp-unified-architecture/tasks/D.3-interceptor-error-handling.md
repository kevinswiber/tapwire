# Task D.3: Interceptor Error Handling and Safety

## Objective
Define comprehensive error handling for interceptors, including typed errors, recovery strategies, and dependency management to prevent runtime failures.

## Background
From Gemini's review: The plan shows interceptors returning `Result<InterceptAction>`, but doesn't specify what happens when an interceptor returns an `Err`. This is a critical detail for production stability.

## Key Requirements

### 1. InterceptorError Enum
Define a comprehensive error type that guides chain behavior:

```rust
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum InterceptorError {
    /// Fatal error - stops the chain and returns error to caller
    #[error("Fatal interceptor error: {0}")]
    Fatal(Box<dyn std::error::Error + Send + Sync>),
    
    /// Recoverable error - logs and continues with next interceptor
    #[error("Recoverable interceptor error: {0}")]
    Recoverable(Box<dyn std::error::Error + Send + Sync>),
    
    /// Retry the interceptor after delay
    #[error("Interceptor requested retry after {0:?}")]
    Retry(Duration),
    
    /// Skip this interceptor for this message
    #[error("Interceptor skipped: {0}")]
    Skip(String),
    
    /// Configuration error - interceptor is misconfigured
    #[error("Interceptor configuration error: {0}")]
    Configuration(String),
}

impl InterceptorError {
    pub fn fatal<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
        Self::Fatal(Box::new(err))
    }
    
    pub fn recoverable<E: std::error::Error + Send + Sync + 'static>(err: E) -> Self {
        Self::Recoverable(Box::new(err))
    }
}
```

### 2. Interceptor Trait Update
```rust
#[async_trait]
pub trait Interceptor: Send + Sync {
    /// Process a request through the interceptor
    async fn intercept_request(
        &self,
        request: &mut JsonRpcRequest,
        context: &InterceptorContext,
    ) -> Result<InterceptAction, InterceptorError>;
    
    /// Process a response through the interceptor
    async fn intercept_response(
        &self,
        response: &mut JsonRpcResponse,
        context: &InterceptorContext,
    ) -> Result<InterceptAction, InterceptorError>;
    
    /// Supported protocol versions (for version-aware filtering)
    fn supported_versions(&self) -> &[ProtocolVersion] {
        &[ProtocolVersion::V2025_11_05] // Default to latest
    }
    
    /// Dependencies this interceptor requires in context
    fn required_context_keys(&self) -> &[&str] {
        &[]
    }
}
```

### 3. InterceptorChain Error Handling
```rust
impl InterceptorChain {
    pub async fn process_request(
        &self,
        mut request: JsonRpcRequest,
        mut context: InterceptorContext,
    ) -> Result<(JsonRpcRequest, InterceptorContext), Error> {
        let version = context.protocol_version();
        
        for interceptor in &self.interceptors {
            // Version filtering
            if !interceptor.supported_versions().contains(&version) {
                tracing::trace!(
                    interceptor = %interceptor.name(),
                    version = %version,
                    "Skipping interceptor for version"
                );
                continue;
            }
            
            // Check dependencies
            for required_key in interceptor.required_context_keys() {
                if !context.has_key(required_key) {
                    tracing::error!(
                        interceptor = %interceptor.name(),
                        missing_key = required_key,
                        "Interceptor missing required context"
                    );
                    return Err(Error::InterceptorDependency {
                        interceptor: interceptor.name(),
                        missing: required_key.to_string(),
                    });
                }
            }
            
            // Process with error handling
            match interceptor.intercept_request(&mut request, &context).await {
                Ok(action) => {
                    match action {
                        InterceptAction::Continue => {},
                        InterceptAction::Block(reason) => {
                            return Err(Error::Blocked(reason));
                        },
                        InterceptAction::Modify(new_request) => {
                            request = new_request;
                        },
                        // ... handle other actions
                    }
                },
                Err(InterceptorError::Fatal(e)) => {
                    tracing::error!(
                        interceptor = %interceptor.name(),
                        error = %e,
                        "Fatal interceptor error"
                    );
                    metrics::counter!("mcp.interceptor.errors", "type" => "fatal").increment(1);
                    return Err(Error::InterceptorFatal {
                        interceptor: interceptor.name(),
                        source: e,
                    });
                },
                Err(InterceptorError::Recoverable(e)) => {
                    tracing::warn!(
                        interceptor = %interceptor.name(),
                        error = %e,
                        "Recoverable interceptor error, continuing chain"
                    );
                    metrics::counter!("mcp.interceptor.errors", "type" => "recoverable").increment(1);
                    // Continue to next interceptor
                },
                Err(InterceptorError::Retry(delay)) => {
                    tracing::info!(
                        interceptor = %interceptor.name(),
                        delay = ?delay,
                        "Interceptor requested retry"
                    );
                    tokio::time::sleep(delay).await;
                    // Retry this interceptor
                    match interceptor.intercept_request(&mut request, &context).await {
                        Ok(action) => { /* process action */ },
                        Err(_) => {
                            // Don't retry again, treat as recoverable
                            tracing::warn!("Retry failed, continuing chain");
                        }
                    }
                },
                Err(InterceptorError::Skip(reason)) => {
                    tracing::debug!(
                        interceptor = %interceptor.name(),
                        reason = %reason,
                        "Interceptor skipped"
                    );
                    metrics::counter!("mcp.interceptor.skipped").increment(1);
                    // Continue to next interceptor
                },
                Err(InterceptorError::Configuration(msg)) => {
                    tracing::error!(
                        interceptor = %interceptor.name(),
                        error = %msg,
                        "Interceptor configuration error"
                    );
                    // Configuration errors are fatal
                    return Err(Error::InterceptorConfig {
                        interceptor: interceptor.name(),
                        message: msg,
                    });
                }
            }
        }
        
        Ok((request, context))
    }
}
```

### 4. InterceptorContext with Dependencies
```rust
#[derive(Debug, Clone)]
pub struct InterceptorContext {
    inner: Arc<RwLock<InterceptorContextInner>>,
}

#[derive(Debug)]
struct InterceptorContextInner {
    session_id: SessionId,
    protocol_version: ProtocolVersion,
    auth: Option<AuthInfo>,
    rate_limit: Option<RateLimitState>,
    metadata: BTreeMap<String, serde_json::Value>,
}

impl InterceptorContext {
    pub fn builder(session_id: SessionId) -> InterceptorContextBuilder {
        InterceptorContextBuilder::new(session_id)
    }
    
    pub fn has_key(&self, key: &str) -> bool {
        let inner = self.inner.read();
        match key {
            "auth" => inner.auth.is_some(),
            "rate_limit" => inner.rate_limit.is_some(),
            _ => inner.metadata.contains_key(key),
        }
    }
    
    pub fn get_auth(&self) -> Option<AuthInfo> {
        self.inner.read().auth.clone()
    }
    
    pub fn set_auth(&self, auth: AuthInfo) {
        self.inner.write().auth = Some(auth);
    }
    
    // Type-safe accessors for common fields
    pub fn get_rate_limit(&self) -> Option<RateLimitState> {
        self.inner.read().rate_limit.clone()
    }
    
    pub fn set_rate_limit(&self, state: RateLimitState) {
        self.inner.write().rate_limit = Some(state);
    }
    
    // Generic metadata for extensibility
    pub fn get_metadata<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.inner.read()
            .metadata
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
    
    pub fn set_metadata<T: Serialize>(&self, key: String, value: T) -> Result<(), Error> {
        let json_value = serde_json::to_value(value)?;
        self.inner.write().metadata.insert(key, json_value);
        Ok(())
    }
}
```

### 5. Graceful Degradation for SessionStore
```rust
impl SessionManager {
    pub async fn new(config: SessionConfig) -> Result<Self, Error> {
        let store = match config.storage_backend {
            StorageBackend::Redis(redis_config) => {
                match RedisSessionStore::new(redis_config).await {
                    Ok(store) => Arc::new(store) as Arc<dyn SessionStore>,
                    Err(e) => {
                        tracing::error!(
                            error = %e,
                            "Failed to initialize Redis session store, falling back to memory"
                        );
                        metrics::counter!("mcp.session.store.fallback").increment(1);
                        
                        if config.require_persistent_storage {
                            // Panic if persistence is required
                            panic!("Required persistent session storage unavailable: {}", e);
                        }
                        
                        // Fall back to memory store
                        Arc::new(InMemorySessionStore::new())
                    }
                }
            },
            StorageBackend::Memory => Arc::new(InMemorySessionStore::new()),
        };
        
        Ok(Self {
            store,
            config,
            // ...
        })
    }
    
    pub async fn get_session(&self, id: &SessionId) -> Option<Session> {
        match self.store.get(id).await {
            Ok(session) => session,
            Err(e) => {
                tracing::error!(
                    session_id = %id,
                    error = %e,
                    "Failed to retrieve session, operating stateless"
                );
                metrics::counter!("mcp.session.store.errors").increment(1);
                
                // Return None to operate stateless for this request
                None
            }
        }
    }
}
```

## Implementation Steps

1. **Define error types** (1 hour)
   - Create `InterceptorError` enum
   - Add to main Error enum
   - Document error semantics

2. **Update Interceptor trait** (30 min)
   - Change return type to use `InterceptorError`
   - Add dependency declaration methods

3. **Implement error handling in chain** (2 hours)
   - Handle each error variant appropriately
   - Add retry logic with backoff
   - Implement dependency checking

4. **Create InterceptorContext** (1.5 hours)
   - Type-safe fields for common data
   - Generic metadata storage
   - Builder pattern for construction

5. **Add graceful degradation** (1 hour)
   - SessionStore fallback logic
   - Stateless operation on errors
   - Configuration for required vs optional persistence

6. **Add metrics and logging** (30 min)
   - Error counters by type
   - Latency histograms
   - Structured logging with context

## Testing Strategy

1. **Unit Tests**
   - Each error type behavior
   - Retry logic with delays
   - Dependency checking

2. **Integration Tests**
   - Chain with failing interceptors
   - SessionStore failure and fallback
   - Stateless operation under store errors

3. **Fault Injection**
   - Random interceptor failures
   - Store unavailability
   - Network errors

## Success Criteria

- [ ] All error types have defined behavior
- [ ] No interceptor error can crash the server
- [ ] Graceful degradation to stateless operation
- [ ] Dependencies enforced at runtime
- [ ] Comprehensive error metrics

## Risk Mitigation

1. **Cascading Failures**: Circuit breakers for external dependencies
2. **Retry Storms**: Exponential backoff with jitter
3. **Memory Leaks**: Bounded retry attempts

## Dependencies
- Task D.0 (Base interceptor implementation)
- Task D.1 (Interceptor chain)

## Estimated Duration
6.5 hours

## Notes
- Consider using `tower::retry` for sophisticated retry logic
- Error types should be serializable for debugging
- Metrics are critical for monitoring interceptor health in production