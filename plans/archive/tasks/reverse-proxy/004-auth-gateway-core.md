# Task 004: AuthGateway Core Implementation and Middleware

**Phase:** 5 (Reverse Proxy & Authentication)  
**Week:** 1 (Core Infrastructure)  
**Day:** 4  
**Priority:** Critical  
**Estimated Time:** 6-8 hours

## Overview

Implement the unified AuthGateway that integrates OAuth 2.1 flow, JWT validation, and secure token management into a cohesive authentication system. This task creates the central authentication component that enforces MCP security requirements while providing seamless integration with the HTTP server and middleware stack.

## Success Criteria

- [x] Research validated integration architecture and performance targets
- [ ] AuthGateway unifies OAuth 2.1 and JWT validation components
- [ ] HTTP authentication middleware with Bearer token support
- [ ] Rate limiting integration for authentication requests
- [ ] Audit logging for all authentication events
- [ ] Session-based authentication context management
- [ ] Performance target: < 5ms total authentication overhead
- [ ] Security compliance: Never forward client tokens upstream
- [ ] Integration with existing Phase 4 InterceptorChain
- [ ] All tests passing (unit + integration + security)

## Technical Specifications

### AuthGateway Core Implementation
```rust
use crate::auth::{OAuth2Client, TokenValidator, SecureTokenStore};
use crate::interceptor::InterceptorChain;
use crate::session::{SessionManager, SessionId};
use tower_governor::{governor::GovernorConfig, key_extractor::KeyExtractor};

pub struct AuthGateway {
    oauth_client: Arc<OAuth2Client>,
    token_validator: Arc<TokenValidator>,
    token_store: Arc<SecureTokenStore>,
    rate_limiter: Arc<GovernorConfig<String>>,
    audit_logger: Arc<AuditLogger>,
    session_manager: Arc<SessionManager>,
    config: AuthGatewayConfig,
    metrics: Arc<AuthMetrics>,
}

impl AuthGateway {
    pub async fn new(
        config: AuthGatewayConfig,
        session_manager: Arc<SessionManager>,
    ) -> Result<Self, AuthGatewayError> {
        let oauth_client = Arc::new(OAuth2Client::new(config.oauth2.clone()).await?);
        
        let token_validator = Arc::new(
            TokenValidator::new(config.jwt_validation.clone()).await?
        );
        
        let token_store = Arc::new(
            SecureTokenStore::new(config.token_storage.clone()).await?
        );

        let rate_limiter = Arc::new(
            GovernorConfig::builder()
                .per_minute(config.rate_limits.auth_requests_per_minute)
                .burst_size(config.rate_limits.auth_burst_size)
                .key_extractor(AuthKeyExtractor)
                .build()
                .unwrap()
        );

        let audit_logger = Arc::new(AuditLogger::new(config.audit.clone()));
        let metrics = Arc::new(AuthMetrics::new());

        Ok(Self {
            oauth_client,
            token_validator,
            token_store,
            rate_limiter,
            audit_logger,
            session_manager,
            config,
            metrics,
        })
    }
}
```

### Core Authentication Flow
```rust
impl AuthGateway {
    #[instrument(
        skip(self, token),
        fields(
            session_id = %session_id,
            event_type = "authentication"
        )
    )]
    pub async fn authenticate_request(
        &self,
        token: &str,
        session_id: &SessionId,
        request_info: &RequestInfo,
    ) -> Result<AuthContext, AuthGatewayError> {
        let start_time = Instant::now();

        // 1. Rate limiting check
        self.check_rate_limit(token, request_info).await?;

        // 2. JWT validation with JWKS
        let validated_claims = self.token_validator
            .validate_token(token)
            .await
            .map_err(|e| {
                self.audit_logger.log_auth_failure(
                    session_id,
                    &AuthFailureReason::InvalidToken(e.to_string()),
                    request_info,
                );
                AuthGatewayError::TokenValidation(e)
            })?;

        // 3. Create authentication context (NEVER forward client token)
        let auth_context = AuthContext {
            session_id: session_id.clone(),
            subject: validated_claims.subject,
            issuer: validated_claims.issuer,
            audience: validated_claims.audience,
            scopes: validated_claims.scopes,
            expires_at: UNIX_EPOCH + Duration::from_secs(validated_claims.expires_at),
            custom_claims: validated_claims.custom_claims,
            authenticated_at: Instant::now(),
        };

        // 4. Store authentication context (not the token!)
        self.token_store
            .store_auth_context(session_id.clone(), &auth_context)
            .await?;

        // 5. Record successful authentication
        let auth_duration = start_time.elapsed();
        self.metrics.record_successful_auth(auth_duration);
        self.audit_logger.log_auth_success(&auth_context, request_info);

        Ok(auth_context)
    }

    async fn check_rate_limit(
        &self,
        token: &str,
        request_info: &RequestInfo,
    ) -> Result<(), AuthGatewayError> {
        // Extract rate limiting key (could be IP, subject, or combination)
        let rate_key = self.extract_rate_limit_key(token, request_info)?;
        
        // Check rate limit using tower-governor
        match self.rate_limiter.check_key(&rate_key) {
            Ok(_) => Ok(()),
            Err(_) => {
                self.metrics.record_rate_limit_exceeded();
                self.audit_logger.log_rate_limit_exceeded(&rate_key, request_info);
                Err(AuthGatewayError::RateLimitExceeded)
            }
        }
    }
}
```

### HTTP Authentication Middleware
```rust
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn jwt_auth_middleware(
    State(auth_gateway): State<Arc<AuthGateway>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthGatewayError> {
    // Extract Bearer token from Authorization header
    let auth_header = request.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthGatewayError::MissingAuthHeader)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AuthGatewayError::InvalidAuthFormat);
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix

    // Extract session ID from MCP headers
    let session_id = extract_mcp_session_id(request.headers())?;

    // Create request info for audit logging
    let request_info = RequestInfo {
        method: request.method().clone(),
        uri: request.uri().clone(),
        user_agent: extract_user_agent(request.headers()),
        client_ip: extract_client_ip(&request),
        timestamp: Instant::now(),
    };

    // Authenticate the request
    let auth_context = auth_gateway
        .authenticate_request(token, &session_id, &request_info)
        .await?;

    // Add authentication context to request extensions
    request.extensions_mut().insert(auth_context);
    request.extensions_mut().insert(session_id);

    // Continue with the request
    Ok(next.run(request).await)
}
```

### Authentication Context Management
```rust
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub session_id: SessionId,
    pub subject: String,
    pub issuer: String,
    pub audience: Vec<String>,
    pub scopes: Vec<String>,
    pub expires_at: SystemTime,
    pub custom_claims: HashMap<String, serde_json::Value>,
    pub authenticated_at: Instant,
}

impl AuthContext {
    pub fn has_scope(&self, required_scope: &str) -> bool {
        self.scopes.contains(&required_scope.to_string())
    }

    pub fn has_any_scope(&self, required_scopes: &[&str]) -> bool {
        required_scopes.iter().any(|scope| self.has_scope(scope))
    }

    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }

    pub fn get_custom_claim<T>(&self, claim_name: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.custom_claims
            .get(claim_name)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }
}
```

### Integration with Phase 4 Interceptor Chain
```rust
// Enhanced InterceptContext with authentication information
impl AuthGateway {
    pub async fn enhance_intercept_context(
        &self,
        mut context: InterceptContext,
        auth_context: Option<AuthContext>,
    ) -> InterceptContext {
        // Add authentication context to existing interceptor system
        context.auth_context = auth_context.clone();
        
        // Add HTTP-specific metadata
        if let Some(auth) = auth_context {
            context.metadata.insert("auth.subject".to_string(), auth.subject);
            context.metadata.insert("auth.issuer".to_string(), auth.issuer);
            context.metadata.insert(
                "auth.scopes".to_string(),
                auth.scopes.join(",")
            );
            
            // Add custom claims as metadata
            for (key, value) in auth.custom_claims {
                let metadata_key = format!("auth.claim.{}", key);
                context.metadata.insert(metadata_key, value.to_string());
            }
        }

        context
    }
}
```

### Audit Logging Implementation
```rust
use tracing::{info, warn, error, instrument};

pub struct AuditLogger {
    config: AuditConfig,
}

impl AuditLogger {
    #[instrument(
        skip(self),
        fields(
            event_type = "auth_success",
            session_id = %auth_context.session_id,
            subject = %auth_context.subject,
            scopes = %auth_context.scopes.join(",")
        )
    )]
    pub fn log_auth_success(
        &self,
        auth_context: &AuthContext,
        request_info: &RequestInfo,
    ) {
        info!(
            session_id = %auth_context.session_id,
            subject = %auth_context.subject,
            method = %request_info.method,
            uri = %request_info.uri,
            user_agent = %request_info.user_agent.as_deref().unwrap_or("unknown"),
            "Authentication successful"
        );
    }

    #[instrument(
        skip(self),
        fields(
            event_type = "auth_failure",
            session_id = %session_id,
            reason = %failure_reason
        )
    )]
    pub fn log_auth_failure(
        &self,
        session_id: &SessionId,
        failure_reason: &AuthFailureReason,
        request_info: &RequestInfo,
    ) {
        warn!(
            session_id = %session_id,
            reason = %failure_reason,
            method = %request_info.method,
            uri = %request_info.uri,
            user_agent = %request_info.user_agent.as_deref().unwrap_or("unknown"),
            "Authentication failed"
        );
    }

    #[instrument(
        skip(self),
        fields(
            event_type = "rate_limit_exceeded",
            rate_key = %rate_key
        )
    )]
    pub fn log_rate_limit_exceeded(
        &self,
        rate_key: &str,
        request_info: &RequestInfo,
    ) {
        warn!(
            rate_key = %rate_key,
            method = %request_info.method,
            uri = %request_info.uri,
            client_ip = %request_info.client_ip.as_deref().unwrap_or("unknown"),
            "Rate limit exceeded"
        );
    }
}
```

### Performance Metrics
```rust
pub struct AuthMetrics {
    auth_duration: Arc<RwLock<Vec<Duration>>>,
    successful_auths: AtomicU64,
    failed_auths: AtomicU64,
    rate_limit_exceeded: AtomicU64,
    token_cache_hits: AtomicU64,
    token_cache_misses: AtomicU64,
}

impl AuthMetrics {
    pub fn record_successful_auth(&self, duration: Duration) {
        self.successful_auths.fetch_add(1, Ordering::Relaxed);
        
        let mut durations = self.auth_duration.write().unwrap();
        durations.push(duration);
        
        // Keep rolling window of last 1000 measurements
        if durations.len() > 1000 {
            durations.drain(0..100);
        }
    }

    pub fn get_average_auth_time(&self) -> Duration {
        let durations = self.auth_duration.read().unwrap();
        if durations.is_empty() {
            return Duration::from_nanos(0);
        }
        
        let total: Duration = durations.iter().sum();
        total / durations.len() as u32
    }

    pub fn get_success_rate(&self) -> f64 {
        let successful = self.successful_auths.load(Ordering::Relaxed);
        let failed = self.failed_auths.load(Ordering::Relaxed);
        
        if successful + failed == 0 {
            return 1.0; // No auth attempts yet
        }
        
        successful as f64 / (successful + failed) as f64
    }
}
```

## Implementation Steps

### Step 1: Core AuthGateway Structure
- Implement unified AuthGateway struct
- Integrate OAuth2Client and TokenValidator
- Add rate limiting and audit logging
- Create performance metrics collection

### Step 2: HTTP Middleware Implementation
- JWT authentication middleware
- Error handling and HTTP status mapping
- Request information extraction
- Integration with Axum router

### Step 3: Session and Context Management
- AuthContext structure and methods
- Integration with existing SessionManager
- Context enhancement for InterceptorChain
- Secure context storage

### Step 4: Audit and Monitoring
- Structured audit logging with tracing
- Performance metrics collection
- Security event tracking
- Rate limiting monitoring

### Step 5: Integration Testing
- End-to-end authentication flow
- Middleware integration testing
- Performance benchmarking
- Security validation

## Dependencies

### Blocked By
- Task 001: Axum HTTP Server Setup
- Task 002: OAuth 2.1 Flow Implementation
- Task 003: JWT Validation with JWKS Client

### Blocks
- Task 006: Extended RuleBasedInterceptor with HTTP Conditions
- Task 007: Rate Limiting and Audit Logging Integration

### Integrates With
- Existing Phase 4 SessionManager
- Existing Phase 4 InterceptorChain
- HTTP server from Task 001

## Testing Requirements

### Unit Tests
- [ ] AuthGateway initialization and configuration
- [ ] Authentication flow with valid/invalid tokens
- [ ] Rate limiting behavior
- [ ] AuthContext creation and validation
- [ ] Audit logging accuracy
- [ ] Metrics collection correctness

### Integration Tests
- [ ] End-to-end HTTP authentication flow
- [ ] Middleware integration with Axum
- [ ] Session management integration
- [ ] InterceptorChain context enhancement
- [ ] Error handling and HTTP status mapping

### Security Tests
- [ ] Token forwarding prevention (critical)
- [ ] Rate limiting effectiveness
- [ ] Authentication bypass attempts
- [ ] Context injection attacks
- [ ] Audit log tampering prevention

### Performance Tests
- [ ] Authentication overhead (target: < 5ms)
- [ ] Concurrent authentication handling
- [ ] Memory usage per authenticated session
- [ ] Rate limiting performance impact
- [ ] Audit logging performance impact

## Configuration Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthGatewayConfig {
    pub oauth2: OAuth2Config,
    pub jwt_validation: JwtValidationConfig,
    pub token_storage: TokenStorageConfig,
    pub rate_limits: RateLimitConfig,
    pub audit: AuditConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub auth_requests_per_minute: u32,
    pub auth_burst_size: u32,
    pub rate_key_strategy: RateKeyStrategy, // IP, Subject, Combined
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub log_successful_auths: bool,
    pub log_failed_auths: bool,
    pub log_rate_limits: bool,
    pub structured_logging: bool,
}
```

## Security Requirements

### Token Security
- **NEVER forward client tokens upstream** (critical MCP requirement)
- Store only authentication context, not tokens
- Encrypt sensitive authentication data at rest
- Implement secure session management

### Rate Limiting
- Per-IP, per-subject, or combined rate limiting
- Configurable rate limits and burst sizes
- Audit logging of rate limit violations
- DDoS protection for authentication endpoints

### Audit Requirements
- Log all authentication attempts (success/failure)
- Log rate limiting violations
- Structured logging for security analysis
- Tamper-evident audit trails

## Performance Requirements

- **Total authentication overhead:** < 5ms
- **Rate limiting overhead:** < 100µs
- **Audit logging overhead:** < 500µs
- **Memory per authenticated session:** < 2KB
- **Concurrent authentication support:** 1000+ simultaneous

## Risk Assessment

**Medium Risk**: Integration complexity, security-critical authentication logic.

**Mitigation Strategies**:
- Comprehensive security testing
- Integration testing with all components
- Performance monitoring and optimization
- Regular security audits and penetration testing

## Completion Checklist

- [ ] AuthGateway successfully integrates all auth components
- [ ] HTTP middleware working with Bearer token authentication
- [ ] Rate limiting properly configured and functional
- [ ] Audit logging capturing all security events
- [ ] Authentication context properly managed and stored
- [ ] Performance targets met (< 5ms authentication overhead)
- [ ] Security requirement validation (no token forwarding)
- [ ] Integration with Phase 4 InterceptorChain working
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Security tests validating compliance
- [ ] Performance benchmarks meeting targets
- [ ] Configuration schema documented
- [ ] Code review completed

## Files Modified/Created

### New Files
- `src/auth/gateway.rs`: Core AuthGateway implementation
- `src/auth/middleware.rs`: HTTP authentication middleware
- `src/auth/context.rs`: AuthContext and session management
- `src/auth/audit.rs`: Audit logging implementation
- `src/auth/metrics.rs`: Authentication metrics
- `src/config/auth_gateway.rs`: AuthGateway configuration
- `tests/unit/auth_gateway_test.rs`: Unit tests
- `tests/integration/auth_middleware_test.rs`: Integration tests

### Modified Files
- `src/auth/mod.rs`: Export AuthGateway and related modules
- `src/proxy/reverse.rs`: Integrate authentication middleware
- `src/interceptor/mod.rs`: Add auth context to InterceptContext
- `Cargo.toml`: Add tower-governor for rate limiting
- `src/config/mod.rs`: Include AuthGateway configuration

## Next Task
Upon completion, proceed to **Task 005: Connection Pool and Circuit Breaker Implementation** which provides the upstream connection management needed for the reverse proxy functionality.