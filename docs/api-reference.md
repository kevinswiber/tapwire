# Shadowcat API Reference

## Library API

Shadowcat can be used as a library in your Rust applications. This reference covers the main public APIs.

## Core Types

### Transport Types

```rust
use shadowcat::transport::{Transport, TransportType, MessageEnvelope};

pub enum TransportType {
    Stdio,
    Http,
    Sse,
    StreamableHttp,
}

pub struct MessageEnvelope {
    pub content: Bytes,
    pub metadata: EnvelopeMetadata,
    pub session_id: Option<String>,
}

pub struct EnvelopeMetadata {
    pub timestamp: SystemTime,
    pub direction: MessageDirection,
    pub transport_type: TransportType,
    pub session_id: Option<String>,
    pub headers: HashMap<String, String>,
}
```

### Proxy Types

```rust
use shadowcat::proxy::{ForwardProxy, ReverseProxy, GatewayProxy};

// Forward Proxy - Client controls both ends
pub struct ForwardProxy {
    // Use builder pattern for construction
}

// Reverse Proxy - Production deployment with auth
pub struct ReverseProxy {
    // Use builder pattern for construction
}

// Gateway Proxy - Multiple upstreams
pub struct GatewayProxy {
    // Use builder pattern for construction
}
```

## Builder APIs

### ForwardProxy Builder

```rust
use shadowcat::proxy::ForwardProxy;
use std::time::Duration;

let proxy = ForwardProxy::builder()
    // Transport configuration
    .transport("stdio")
    .command("npx")
    .args(vec!["@modelcontextprotocol/server-everything"])
    
    // Session configuration
    .session_timeout(Duration::from_secs(300))
    .max_sessions(100)
    
    // Interceptors
    .interceptor(my_interceptor)
    
    // Recording
    .recorder(tape_recorder)
    
    // Build and start
    .build()
    .await?;

// Run the proxy
proxy.run().await?;
```

### ReverseProxy Builder

```rust
use shadowcat::proxy::ReverseProxy;
use shadowcat::auth::{AuthProvider, OAuthConfig};

let proxy = ReverseProxy::builder()
    // Network binding
    .bind("127.0.0.1:8080")
    
    // Upstream configuration
    .upstream("http://mcp-server:3000")
    .upstream_timeout(Duration::from_secs(30))
    
    // Authentication
    .auth_provider(AuthProvider::OAuth2(oauth_config))
    .require_auth(true)
    
    // Rate limiting
    .rate_limit(100, Duration::from_secs(60))
    
    // Policy engine
    .policy_file("policies.yaml")
    
    // TLS configuration
    .tls_cert("cert.pem")
    .tls_key("key.pem")
    
    .build()
    .await?;

proxy.serve().await?;
```

### GatewayProxy Builder

```rust
use shadowcat::proxy::{GatewayProxy, LoadBalanceStrategy};

let proxy = GatewayProxy::builder()
    .bind("127.0.0.1:8080")
    
    // Multiple upstreams
    .upstream("primary", "http://server1:3000")
    .upstream("secondary", "http://server2:3000")
    
    // Load balancing
    .load_balance_strategy(LoadBalanceStrategy::RoundRobin)
    
    // Health checks
    .health_check_interval(Duration::from_secs(10))
    .health_check_timeout(Duration::from_secs(5))
    
    // Circuit breaker
    .circuit_breaker_threshold(5)
    .circuit_breaker_timeout(Duration::from_secs(60))
    
    .build()
    .await?;
```

## Interceptor API

### Creating Custom Interceptors

```rust
use shadowcat::interceptor::{Interceptor, InterceptorAction};
use async_trait::async_trait;

pub struct MyInterceptor {
    // Your configuration
}

#[async_trait]
impl Interceptor for MyInterceptor {
    async fn process(
        &self,
        envelope: MessageEnvelope,
    ) -> Result<InterceptorAction> {
        // Inspect the message
        let content = std::str::from_utf8(&envelope.content)?;
        
        // Decide action
        if content.contains("secret") {
            return Ok(InterceptorAction::Block {
                reason: "Contains sensitive data".into(),
            });
        }
        
        if content.contains("transform") {
            let modified = modify_content(envelope)?;
            return Ok(InterceptorAction::Modify {
                envelope: modified,
            });
        }
        
        // Continue normally
        Ok(InterceptorAction::Continue(envelope))
    }
}
```

### Interceptor Actions

```rust
pub enum InterceptorAction {
    // Continue processing
    Continue(MessageEnvelope),
    
    // Pause processing (for user interaction)
    Pause {
        resume_token: String,
    },
    
    // Modify the message
    Modify {
        envelope: MessageEnvelope,
    },
    
    // Block the message
    Block {
        reason: String,
    },
}
```

## Session Management API

### SessionManager

```rust
use shadowcat::session::{SessionManager, Session, SessionState};

// Create manager with store
let manager = SessionManager::new(store).await?;

// Create a new session
let session = manager.create_session(
    "client-id",
    TransportType::Http,
).await?;

// Get session by ID
let session = manager.get_session(&session_id).await?;

// Update session state
manager.update_state(&session_id, SessionState::Active).await?;

// Update session metadata
manager.update_session(&session_id, |session| {
    session.metadata.insert(
        "user".into(),
        serde_json::json!("alice"),
    );
}).await?;

// List active sessions
let sessions = manager.list_active_sessions().await?;

// Clean up expired sessions
manager.cleanup_expired().await?;
```

### Session Store Trait

```rust
use shadowcat::session::{SessionStore, Session};

#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create(&self, session: Session) -> Result<()>;
    async fn get(&self, id: &str) -> Result<Option<Session>>;
    async fn update(&self, session: Session) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn list_active(&self) -> Result<Vec<Session>>;
    async fn cleanup_expired(&self) -> Result<usize>;
}
```

## Recording & Replay API

### Recorder

```rust
use shadowcat::recorder::{Recorder, TapeFormat};

let recorder = Recorder::builder()
    .output_path("session.tape")
    .format(TapeFormat::JsonLines)
    .include_metadata(true)
    .compression(true)
    .build()?;

// Record a message
recorder.record(envelope).await?;

// Finalize recording
recorder.finalize().await?;
```

### Replay Engine

```rust
use shadowcat::replay::{ReplayEngine, ReplayMode};

let engine = ReplayEngine::builder()
    .tape_path("session.tape")
    .mode(ReplayMode::Realtime)
    .speed_multiplier(2.0)
    .transformer(my_transformer)
    .build()?;

// Start replay
engine.replay().await?;

// Pause/resume
engine.pause().await?;
engine.resume().await?;

// Seek to position
engine.seek(Duration::from_secs(30)).await?;
```

## Authentication API

### OAuth 2.1 Configuration

```rust
use shadowcat::auth::{OAuthConfig, TokenValidator};

let oauth_config = OAuthConfig::builder()
    .client_id("your-client-id")
    .client_secret("your-secret")
    .auth_url("https://auth.example.com/oauth2/authorize")
    .token_url("https://auth.example.com/oauth2/token")
    .jwks_url("https://auth.example.com/.well-known/jwks.json")
    .redirect_uri("http://localhost:8080/callback")
    .scopes(vec!["read", "write"])
    .pkce_required(true)
    .build()?;
```

### Token Validation

```rust
use shadowcat::auth::{TokenValidator, Claims};

let validator = TokenValidator::new(jwks_url).await?;

// Validate JWT
let claims: Claims = validator.validate(token).await?;

// Check specific claims
if claims.scope.contains("admin") {
    // Allow admin operations
}
```

## Rate Limiting API

### Rate Limiter Configuration

```rust
use shadowcat::rate_limiting::{RateLimiter, RateLimitConfig};

let config = RateLimitConfig::builder()
    .global_limit(1000, Duration::from_secs(60))
    .per_user_limit(100, Duration::from_secs(60))
    .per_endpoint_limits(vec![
        ("/api/expensive", 10, Duration::from_secs(60)),
        ("/api/normal", 100, Duration::from_secs(60)),
    ])
    .burst_size(20)
    .build()?;

let limiter = RateLimiter::new(config);

// Check if request is allowed
let key = format!("user:{}", user_id);
if limiter.check_limit(&key).await? {
    // Process request
} else {
    // Return 429 Too Many Requests
}
```

## Policy Engine API

### Policy Definition

```rust
use shadowcat::auth::policy::{Policy, PolicyEngine, Rule};

let policy = Policy::builder()
    .name("api-access")
    .rules(vec![
        Rule::allow()
            .resource("/api/public/*")
            .methods(vec!["GET"]),
        
        Rule::deny()
            .resource("/api/admin/*")
            .unless_scope("admin"),
        
        Rule::allow()
            .resource("/api/user/*")
            .if_scope("user")
            .methods(vec!["GET", "POST"]),
    ])
    .default_action(PolicyAction::Deny)
    .build()?;

let engine = PolicyEngine::new(vec![policy]);

// Evaluate request
let allowed = engine.evaluate(
    &request_path,
    &request_method,
    &user_claims,
).await?;
```

## Transport API

### Creating Custom Transports

```rust
use shadowcat::transport::{Transport, MessageEnvelope};
use async_trait::async_trait;

pub struct CustomTransport {
    // Your transport state
}

#[async_trait]
impl Transport for CustomTransport {
    async fn connect(&mut self) -> Result<()> {
        // Establish connection
        Ok(())
    }
    
    async fn send(&mut self, envelope: MessageEnvelope) -> Result<()> {
        // Send message
        Ok(())
    }
    
    async fn receive(&mut self) -> Result<MessageEnvelope> {
        // Receive message
        Ok(envelope)
    }
    
    async fn close(&mut self) -> Result<()> {
        // Clean shutdown
        Ok(())
    }
}
```

## Error Handling

### Error Types

```rust
use shadowcat::error::{ShadowcatError, Result};

pub enum ShadowcatError {
    Transport(String),
    Protocol(String),
    Session(String),
    Auth(String),
    Policy(String),
    Io(std::io::Error),
    Config(String),
    Validation(String),
}

// Using errors
fn process() -> Result<()> {
    operation()
        .map_err(|e| ShadowcatError::Transport(e.to_string()))?;
    Ok(())
}
```

## Telemetry API

### OpenTelemetry Integration

```rust
use shadowcat::telemetry::{TelemetryConfig, init_telemetry};

let config = TelemetryConfig::builder()
    .service_name("shadowcat")
    .otlp_endpoint("http://localhost:4317")
    .sampling_ratio(0.1)
    .build()?;

init_telemetry(config)?;

// Use tracing macros
tracing::info!("Starting proxy");
tracing::debug!(session_id = %id, "Session created");
```

## Configuration API

### Loading Configuration

```rust
use shadowcat::config::{Config, load_config};

// From file
let config = load_config("shadowcat.toml")?;

// From environment
let config = Config::from_env()?;

// Programmatic
let config = Config::builder()
    .proxy_mode(ProxyMode::Forward)
    .transport_type(TransportType::Stdio)
    .session_timeout(300)
    .build()?;
```

## Utilities

### Buffer Pooling

```rust
use shadowcat::transport::buffer_pool::{global_pools, BytesPool};

// Get buffer from pool
let mut buf = global_pools::STDIO_POOL.acquire();

// Use buffer
buf.extend_from_slice(data);

// Return to pool (automatic on drop)
global_pools::STDIO_POOL.release(buf);
```

### Circuit Breaker

```rust
use shadowcat::proxy::CircuitBreaker;

let breaker = CircuitBreaker::new(
    5,     // failure threshold
    60,    // timeout seconds
    0.5,   // failure ratio
);

// Check if circuit is open
if breaker.is_open() {
    return Err("Circuit open");
}

// Record result
match operation().await {
    Ok(result) => {
        breaker.record_success();
        Ok(result)
    }
    Err(e) => {
        breaker.record_failure();
        Err(e)
    }
}
```

## Examples

See the `examples/` directory for complete examples:

- `examples/forward_proxy.rs` - Basic forward proxy
- `examples/reverse_proxy.rs` - Reverse proxy with auth
- `examples/custom_interceptor.rs` - Custom interceptor
- `examples/recording.rs` - Recording sessions
- `examples/replay.rs` - Replaying sessions