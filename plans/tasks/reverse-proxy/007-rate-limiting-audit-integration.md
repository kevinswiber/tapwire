# Task 007: Rate Limiting and Audit Logging Integration

**Phase:** 5 (Reverse Proxy & Authentication)  
**Week:** 2 (Security & Integration)  
**Day:** 7  
**Priority:** High  
**Estimated Time:** 6-8 hours

## Overview

Implement comprehensive multi-tier rate limiting using tower-governor with GCRA algorithm and unified audit logging across all system components. This task creates a cohesive security monitoring and protection layer that spans authentication, policy enforcement, and upstream communication.

## Success Criteria

- [x] Research validated tower-governor with GCRA for optimal performance
- [x] Research validated tracing framework for structured audit logging
- [ ] Multi-tier rate limiting (global, per-user, per-IP, per-endpoint)
- [ ] Integration with AuthGateway, HTTP middleware, and rules engine
- [ ] Unified audit logging with structured security events
- [ ] Performance target: < 100µs rate limiting overhead
- [ ] Security target: DDoS protection and attack detection
- [ ] Compliance logging for security audits and forensics
- [ ] Real-time monitoring and alerting capabilities
- [ ] Integration with existing metrics and observability
- [ ] All tests passing (unit + integration + security)

## Technical Specifications

### Multi-Tier Rate Limiting Architecture
```rust
use tower_governor::{
    governor::{GovernorConfig, GovernorConfigBuilder},
    key_extractor::{KeyExtractor, SmartIpKeyExtractor},
    GovernorLayer,
};
use std::net::IpAddr;

pub struct MultiTierRateLimiter {
    // Global rate limiting across all requests
    global_limiter: Arc<GovernorConfig<GlobalKey>>,
    
    // Per-user rate limiting based on authenticated subject
    user_limiter: Arc<GovernorConfig<String>>,
    
    // Per-IP rate limiting for unauthenticated requests
    ip_limiter: Arc<GovernorConfig<IpAddr>>,
    
    // Per-endpoint rate limiting for specific API paths
    endpoint_limiter: Arc<GovernorConfig<EndpointKey>>,
    
    // Per-session rate limiting for MCP sessions
    session_limiter: Arc<GovernorConfig<SessionId>>,
    
    metrics: Arc<RateLimitMetrics>,
    config: RateLimitConfig,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GlobalKey;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct EndpointKey {
    pub method: String,
    pub path_pattern: String,
}

impl MultiTierRateLimiter {
    pub async fn new(config: RateLimitConfig) -> Result<Self, RateLimitError> {
        let global_limiter = Arc::new(
            GovernorConfigBuilder::default()
                .per_minute(config.global.requests_per_minute)
                .burst_size(config.global.burst_size)
                .build()
                .unwrap()
        );

        let user_limiter = Arc::new(
            GovernorConfigBuilder::default()
                .per_minute(config.per_user.requests_per_minute)
                .burst_size(config.per_user.burst_size)
                .key_extractor(UserKeyExtractor)
                .build()
                .unwrap()
        );

        let ip_limiter = Arc::new(
            GovernorConfigBuilder::default()
                .per_minute(config.per_ip.requests_per_minute)
                .burst_size(config.per_ip.burst_size)
                .key_extractor(SmartIpKeyExtractor)
                .build()
                .unwrap()
        );

        let endpoint_limiter = Arc::new(
            GovernorConfigBuilder::default()
                .per_minute(config.per_endpoint.requests_per_minute)
                .burst_size(config.per_endpoint.burst_size)
                .key_extractor(EndpointKeyExtractor)
                .build()
                .unwrap()
        );

        let session_limiter = Arc::new(
            GovernorConfigBuilder::default()
                .per_minute(config.per_session.requests_per_minute)
                .burst_size(config.per_session.burst_size)
                .key_extractor(SessionKeyExtractor)
                .build()
                .unwrap()
        );

        Ok(Self {
            global_limiter,
            user_limiter,
            ip_limiter,
            endpoint_limiter,
            session_limiter,
            metrics: Arc::new(RateLimitMetrics::new()),
            config,
        })
    }
}
```

### Rate Limiting Enforcement
```rust
impl MultiTierRateLimiter {
    #[instrument(
        skip(self),
        fields(
            session_id = %request_context.session_id,
            client_ip = %request_context.client_ip.as_deref().unwrap_or("unknown"),
            endpoint = %request_context.endpoint_key()
        )
    )]
    pub async fn check_rate_limits(
        &self,
        request_context: &RequestContext,
    ) -> Result<(), RateLimitError> {
        let start_time = Instant::now();

        // 1. Global rate limiting (always applied)
        self.global_limiter.check_key(&GlobalKey)
            .map_err(|_| {
                self.metrics.record_global_limit_exceeded();
                RateLimitError::GlobalLimitExceeded
            })?;

        // 2. Per-session rate limiting (for MCP sessions)
        self.session_limiter.check_key(&request_context.session_id)
            .map_err(|_| {
                self.metrics.record_session_limit_exceeded(&request_context.session_id);
                RateLimitError::SessionLimitExceeded(request_context.session_id.clone())
            })?;

        // 3. Per-user or per-IP rate limiting
        if let Some(auth_context) = &request_context.auth_context {
            // Authenticated request: use per-user limiting
            self.user_limiter.check_key(&auth_context.subject)
                .map_err(|_| {
                    self.metrics.record_user_limit_exceeded(&auth_context.subject);
                    RateLimitError::UserLimitExceeded(auth_context.subject.clone())
                })?;
        } else if let Some(client_ip) = &request_context.client_ip {
            // Unauthenticated request: use per-IP limiting
            let ip_addr: IpAddr = client_ip.parse()
                .map_err(|_| RateLimitError::InvalidClientIp(client_ip.clone()))?;
                
            self.ip_limiter.check_key(&ip_addr)
                .map_err(|_| {
                    self.metrics.record_ip_limit_exceeded(&ip_addr);
                    RateLimitError::IpLimitExceeded(ip_addr)
                })?;
        }

        // 4. Per-endpoint rate limiting
        let endpoint_key = request_context.endpoint_key();
        self.endpoint_limiter.check_key(&endpoint_key)
            .map_err(|_| {
                self.metrics.record_endpoint_limit_exceeded(&endpoint_key);
                RateLimitError::EndpointLimitExceeded(endpoint_key)
            })?;

        // Record successful rate limit check
        let check_duration = start_time.elapsed();
        self.metrics.record_successful_check(check_duration);

        Ok(())
    }

    // Get current rate limit status for monitoring
    pub async fn get_rate_limit_status(
        &self,
        request_context: &RequestContext,
    ) -> RateLimitStatus {
        RateLimitStatus {
            global_remaining: self.global_limiter.check_key(&GlobalKey).is_ok(),
            session_remaining: self.session_limiter.check_key(&request_context.session_id).is_ok(),
            user_or_ip_remaining: match &request_context.auth_context {
                Some(auth) => self.user_limiter.check_key(&auth.subject).is_ok(),
                None => {
                    if let Some(ip_str) = &request_context.client_ip {
                        if let Ok(ip) = ip_str.parse::<IpAddr>() {
                            self.ip_limiter.check_key(&ip).is_ok()
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
            },
            endpoint_remaining: self.endpoint_limiter.check_key(&request_context.endpoint_key()).is_ok(),
        }
    }
}
```

### Request Context for Rate Limiting
```rust
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub session_id: SessionId,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub method: String,
    pub path: String,
    pub auth_context: Option<AuthContext>,
    pub timestamp: Instant,
    pub request_id: String,
}

impl RequestContext {
    pub fn endpoint_key(&self) -> EndpointKey {
        EndpointKey {
            method: self.method.clone(),
            path_pattern: self.normalize_path_pattern(&self.path),
        }
    }

    fn normalize_path_pattern(&self, path: &str) -> String {
        // Convert specific paths to patterns for rate limiting
        // Example: /users/123/profile -> /users/{id}/profile
        let path_segments: Vec<&str> = path.split('/').collect();
        let normalized_segments: Vec<String> = path_segments
            .iter()
            .map(|segment| {
                if segment.chars().all(|c| c.is_ascii_digit()) {
                    "{id}".to_string()
                } else if segment.len() > 20 || segment.contains('-') && segment.len() > 10 {
                    "{uuid}".to_string()
                } else {
                    segment.to_string()
                }
            })
            .collect();
        
        normalized_segments.join("/")
    }
}
```

### Key Extractors for Rate Limiting
```rust
// Custom key extractors for different rate limiting tiers
pub struct UserKeyExtractor;

impl KeyExtractor<String> for UserKeyExtractor {
    fn extract<B>(&self, req: &Request<B>) -> Result<String, GovernorError> {
        req.extensions()
            .get::<AuthContext>()
            .map(|auth| auth.subject.clone())
            .ok_or(GovernorError::UnableToExtractKey)
    }
}

pub struct EndpointKeyExtractor;

impl KeyExtractor<EndpointKey> for EndpointKeyExtractor {
    fn extract<B>(&self, req: &Request<B>) -> Result<EndpointKey, GovernorError> {
        let method = req.method().to_string();
        let path = req.uri().path();
        
        Ok(EndpointKey {
            method,
            path_pattern: normalize_path_for_rate_limiting(path),
        })
    }
}

pub struct SessionKeyExtractor;

impl KeyExtractor<SessionId> for SessionKeyExtractor {
    fn extract<B>(&self, req: &Request<B>) -> Result<SessionId, GovernorError> {
        extract_mcp_session_id(req.headers())
            .map_err(|_| GovernorError::UnableToExtractKey)
    }
}
```

### Unified Audit Logging System
```rust
use tracing::{info, warn, error, debug, instrument, Span};
use serde_json::json;

#[derive(Clone)]
pub struct UnifiedAuditLogger {
    config: AuditConfig,
    rate_limit_logger: RateLimitAuditLogger,
    auth_logger: AuthAuditLogger,
    policy_logger: PolicyAuditLogger,
    request_logger: RequestAuditLogger,
}

impl UnifiedAuditLogger {
    pub fn new(config: AuditConfig) -> Self {
        Self {
            rate_limit_logger: RateLimitAuditLogger::new(config.clone()),
            auth_logger: AuthAuditLogger::new(config.clone()),
            policy_logger: PolicyAuditLogger::new(config.clone()),
            request_logger: RequestAuditLogger::new(config.clone()),
            config,
        }
    }

    #[instrument(
        skip(self),
        fields(
            event_type = "rate_limit_exceeded",
            limit_type = %limit_type,
            client_ip = %client_ip.as_deref().unwrap_or("unknown"),
            user_id = %user_id.as_deref().unwrap_or("anonymous")
        )
    )]
    pub fn log_rate_limit_exceeded(
        &self,
        limit_type: RateLimitType,
        client_ip: Option<&str>,
        user_id: Option<&str>,
        session_id: &SessionId,
        request_context: &RequestContext,
    ) {
        let event = json!({
            "event_type": "rate_limit_exceeded",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "session_id": session_id.to_string(),
            "limit_type": limit_type.to_string(),
            "client_ip": client_ip,
            "user_id": user_id,
            "method": request_context.method,
            "path": request_context.path,
            "user_agent": request_context.user_agent,
            "request_id": request_context.request_id,
        });

        match limit_type {
            RateLimitType::Global => {
                error!(
                    event = %event,
                    "Global rate limit exceeded - potential DDoS attack"
                );
            }
            RateLimitType::User | RateLimitType::Ip => {
                warn!(
                    event = %event,
                    "User/IP rate limit exceeded"
                );
            }
            RateLimitType::Endpoint => {
                info!(
                    event = %event,
                    "Endpoint rate limit exceeded"
                );
            }
            RateLimitType::Session => {
                warn!(
                    event = %event,
                    "Session rate limit exceeded"
                );
            }
        }

        // Store in audit trail for compliance
        if self.config.store_rate_limit_violations {
            self.store_audit_event(&event);
        }
    }

    #[instrument(
        skip(self),
        fields(
            event_type = "policy_enforcement",
            policy_id = %policy_id,
            action = %action,
            session_id = %session_id
        )
    )]
    pub fn log_policy_enforcement(
        &self,
        policy_id: &str,
        action: &str,
        session_id: &SessionId,
        request_context: &RequestContext,
        auth_context: Option<&AuthContext>,
    ) {
        let event = json!({
            "event_type": "policy_enforcement",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "session_id": session_id.to_string(),
            "policy_id": policy_id,
            "action": action,
            "method": request_context.method,
            "path": request_context.path,
            "client_ip": request_context.client_ip,
            "user_agent": request_context.user_agent,
            "request_id": request_context.request_id,
            "user_context": auth_context.map(|auth| json!({
                "subject": auth.subject,
                "scopes": auth.scopes,
                "issuer": auth.issuer
            }))
        });

        match action {
            "block" | "deny" => {
                warn!(
                    event = %event,
                    "Policy blocked request"
                );
            }
            "allow" => {
                if self.config.log_successful_policy_decisions {
                    info!(
                        event = %event,
                        "Policy allowed request"
                    );
                }
            }
            _ => {
                debug!(
                    event = %event,
                    "Policy action executed"
                );
            }
        }

        if self.config.store_policy_decisions {
            self.store_audit_event(&event);
        }
    }

    #[instrument(
        skip(self),
        fields(
            event_type = "authentication",
            session_id = %session_id,
            success = %success
        )
    )]
    pub fn log_authentication_event(
        &self,
        success: bool,
        session_id: &SessionId,
        request_context: &RequestContext,
        auth_context: Option<&AuthContext>,
        failure_reason: Option<&str>,
    ) {
        let event = json!({
            "event_type": "authentication",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "session_id": session_id.to_string(),
            "success": success,
            "method": request_context.method,
            "path": request_context.path,
            "client_ip": request_context.client_ip,
            "user_agent": request_context.user_agent,
            "request_id": request_context.request_id,
            "user_context": auth_context.map(|auth| json!({
                "subject": auth.subject,
                "scopes": auth.scopes,
                "issuer": auth.issuer
            })),
            "failure_reason": failure_reason
        });

        if success {
            info!(
                event = %event,
                "Authentication successful"
            );
        } else {
            warn!(
                event = %event,
                failure_reason = %failure_reason.unwrap_or("unknown"),
                "Authentication failed"
            );
        }

        if (success && self.config.log_successful_auth) || (!success && self.config.log_failed_auth) {
            self.store_audit_event(&event);
        }
    }

    fn store_audit_event(&self, event: &serde_json::Value) {
        // Store audit events for compliance and forensics
        // Implementation could use database, file system, or external service
        if let Some(audit_store) = &self.config.audit_store {
            tokio::spawn({
                let event = event.clone();
                let store = audit_store.clone();
                async move {
                    if let Err(e) = store.store_event(event).await {
                        error!("Failed to store audit event: {}", e);
                    }
                }
            });
        }
    }
}
```

### Integration with HTTP Middleware
```rust
// Rate limiting middleware for Axum
pub async fn rate_limiting_middleware(
    State(rate_limiter): State<Arc<MultiTierRateLimiter>>,
    State(audit_logger): State<Arc<UnifiedAuditLogger>>,
    mut request: Request,
    next: Next,
) -> Result<Response, RateLimitError> {
    // Extract request context
    let request_context = RequestContext {
        session_id: extract_mcp_session_id(request.headers())?,
        client_ip: extract_client_ip(&request),
        user_agent: extract_user_agent(request.headers()),
        method: request.method().to_string(),
        path: request.uri().path().to_string(),
        auth_context: request.extensions().get::<AuthContext>().cloned(),
        timestamp: Instant::now(),
        request_id: generate_request_id(),
    };

    // Check rate limits
    match rate_limiter.check_rate_limits(&request_context).await {
        Ok(_) => {
            // Rate limits passed, continue with request
            request.extensions_mut().insert(request_context);
            Ok(next.run(request).await)
        }
        Err(rate_limit_error) => {
            // Rate limit exceeded, log and return error response
            let limit_type = match &rate_limit_error {
                RateLimitError::GlobalLimitExceeded => RateLimitType::Global,
                RateLimitError::UserLimitExceeded(_) => RateLimitType::User,
                RateLimitError::IpLimitExceeded(_) => RateLimitType::Ip,
                RateLimitError::EndpointLimitExceeded(_) => RateLimitType::Endpoint,
                RateLimitError::SessionLimitExceeded(_) => RateLimitType::Session,
                _ => RateLimitType::Global,
            };

            audit_logger.log_rate_limit_exceeded(
                limit_type,
                request_context.client_ip.as_deref(),
                request_context.auth_context.as_ref().map(|auth| auth.subject.as_str()),
                &request_context.session_id,
                &request_context,
            );

            // Return HTTP 429 Too Many Requests
            let response = Response::builder()
                .status(StatusCode::TOO_MANY_REQUESTS)
                .header("Retry-After", "60") // Suggest retry in 60 seconds
                .body(Body::from(format!("Rate limit exceeded: {}", rate_limit_error)))
                .unwrap();

            Ok(response)
        }
    }
}
```

### Performance Metrics and Monitoring
```rust
pub struct RateLimitMetrics {
    checks_performed: AtomicU64,
    checks_passed: AtomicU64,
    global_limits_exceeded: AtomicU64,
    user_limits_exceeded: AtomicU64,
    ip_limits_exceeded: AtomicU64,
    endpoint_limits_exceeded: AtomicU64,
    session_limits_exceeded: AtomicU64,
    check_durations: Arc<RwLock<Vec<Duration>>>,
}

impl RateLimitMetrics {
    pub fn record_successful_check(&self, duration: Duration) {
        self.checks_performed.fetch_add(1, Ordering::Relaxed);
        self.checks_passed.fetch_add(1, Ordering::Relaxed);
        
        let mut durations = self.check_durations.write().unwrap();
        durations.push(duration);
        
        // Keep rolling window
        if durations.len() > 1000 {
            durations.drain(0..100);
        }
    }

    pub fn get_success_rate(&self) -> f64 {
        let performed = self.checks_performed.load(Ordering::Relaxed);
        let passed = self.checks_passed.load(Ordering::Relaxed);
        
        if performed == 0 {
            return 1.0;
        }
        
        passed as f64 / performed as f64
    }

    pub fn get_average_check_time(&self) -> Duration {
        let durations = self.check_durations.read().unwrap();
        if durations.is_empty() {
            return Duration::from_nanos(0);
        }
        
        let total: Duration = durations.iter().sum();
        total / durations.len() as u32
    }
}
```

## Implementation Steps

### Step 1: Multi-Tier Rate Limiter
- Implement MultiTierRateLimiter with GCRA algorithm
- Create custom key extractors for different rate limiting tiers
- Add rate limit checking logic with proper error handling
- Integrate with tower-governor middleware

### Step 2: Unified Audit Logger
- Implement structured audit logging with tracing
- Create audit event schemas for different event types
- Add configurable audit storage backends
- Integrate with existing logging infrastructure

### Step 3: HTTP Middleware Integration
- Create rate limiting middleware for Axum
- Integrate audit logging with middleware
- Add proper error responses for rate limit violations
- Test middleware integration with existing auth flow

### Step 4: Performance Monitoring
- Implement rate limiting metrics collection
- Add performance monitoring for audit logging
- Create dashboard-ready metrics exports
- Add alerting for rate limit threshold breaches

### Step 5: Configuration and Testing
- Create comprehensive configuration schema
- Add CLI commands for rate limit management
- Implement security testing scenarios
- Add performance benchmarks

## Dependencies

### Blocked By
- Task 004: AuthGateway Core Implementation (authentication context)
- Task 006: Extended RuleBasedInterceptor (policy integration)

### Blocks
- Task 008: End-to-End Integration Testing
- Task 009: Performance Testing and Optimization

### Integrates With
- All previous tasks (authentication, policy, connection pool)
- Existing Phase 4 session management
- HTTP server middleware stack

## Testing Requirements

### Unit Tests
- [ ] Rate limiting logic for all tiers
- [ ] Key extraction accuracy
- [ ] Audit logging event generation
- [ ] Metrics collection correctness
- [ ] Configuration validation

### Integration Tests
- [ ] End-to-end rate limiting with HTTP requests
- [ ] Audit logging across all system components
- [ ] Rate limit integration with authentication
- [ ] Policy enforcement with audit trails
- [ ] Performance under various load patterns

### Security Tests
- [ ] DDoS attack simulation and mitigation
- [ ] Rate limit bypass attempts
- [ ] Audit log tampering prevention
- [ ] Performance degradation under attack
- [ ] Compliance audit trail validation

### Performance Tests
- [ ] Rate limiting overhead (target: < 100µs)
- [ ] Audit logging performance impact
- [ ] Memory usage under high load
- [ ] Concurrent rate limit checking
- [ ] GCRA algorithm performance validation

## Configuration Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub global: TierConfig,
    pub per_user: TierConfig,
    pub per_ip: TierConfig,
    pub per_endpoint: TierConfig,
    pub per_session: TierConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub log_successful_auth: bool,
    pub log_failed_auth: bool,
    pub log_successful_policy_decisions: bool,
    pub store_rate_limit_violations: bool,
    pub store_policy_decisions: bool,
    pub audit_store: Option<AuditStoreConfig>,
}
```

## Performance Requirements

- **Rate limiting overhead:** < 100µs per request
- **Audit logging overhead:** < 200µs per event
- **Memory per rate limit key:** < 64 bytes
- **Concurrent rate limit checks:** 10,000+ per second
- **Audit event throughput:** 1000+ events per second

## Risk Assessment

**Low Risk**: Using proven tower-governor and tracing libraries, well-defined integration points.

**Mitigation Strategies**:
- Comprehensive performance testing
- Gradual rollout with monitoring
- Fallback mechanisms for rate limiting failures
- Audit log reliability and durability testing

## Completion Checklist

- [ ] Multi-tier rate limiting implemented with GCRA
- [ ] Custom key extractors working for all tiers
- [ ] Unified audit logging across all components
- [ ] HTTP middleware integration functional
- [ ] Performance targets met (< 100µs rate limiting overhead)
- [ ] Security testing validates DDoS protection
- [ ] Audit compliance requirements satisfied
- [ ] Real-time monitoring and alerting working
- [ ] Integration with existing metrics complete
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Security tests validate protection mechanisms
- [ ] Performance benchmarks meeting targets
- [ ] Configuration schema documented
- [ ] Code review completed

## Files Modified/Created

### New Files
- `src/rate_limiting/multi_tier.rs`: Multi-tier rate limiter implementation
- `src/rate_limiting/key_extractors.rs`: Custom key extractors
- `src/audit/unified_logger.rs`: Unified audit logging system
- `src/audit/event_schemas.rs`: Audit event structures
- `src/middleware/rate_limiting.rs`: Rate limiting middleware
- `src/config/rate_limiting.rs`: Rate limiting configuration
- `tests/unit/rate_limiting_test.rs`: Unit tests
- `tests/integration/rate_limit_audit_test.rs`: Integration tests
- `tests/security/ddos_protection_test.rs`: Security tests

### Modified Files
- `src/proxy/reverse.rs`: Add rate limiting middleware
- `src/auth/gateway.rs`: Integrate with audit logging
- `src/interceptor/engine.rs`: Add audit logging to policy decisions
- `Cargo.toml`: Add tower-governor and related dependencies
- `src/lib.rs`: Export rate limiting and audit modules

## Next Task
Upon completion, proceed to **Task 008: End-to-End Integration Testing and Debugging** which validates the complete reverse proxy system with comprehensive integration testing across all components.