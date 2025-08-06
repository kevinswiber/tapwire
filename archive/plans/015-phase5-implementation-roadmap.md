# Phase 5: Reverse Proxy & Authentication Implementation Roadmap

**Project:** Shadowcat Phase 5 - Reverse Proxy with OAuth 2.1 Authentication  
**Roadmap Date:** August 4, 2025  
**Implementation Period:** 2 weeks (10 working days)  
**Prerequisites:** Phase 4 Complete (127 tests passing)

---

## Overview

This roadmap details the step-by-step implementation of the **Reverse Proxy** component with integrated OAuth 2.1 authentication. Unlike the forward proxy (development tool), this implements the production-ready API gateway where clients connect TO Shadowcat for authenticated MCP access.

**Architecture Reference:** `plans/014-phase5-security-auth-architecture.md`

---

## Week 1: Reverse Proxy Infrastructure + Authentication

### Day 1: Project Setup & OAuth 2.1 Foundation

**Goals:** Set up authentication module structure and implement basic OAuth 2.1 components

**Tasks:**
1. **Create Auth Module Structure**
   ```bash
   mkdir -p src/auth
   touch src/auth/mod.rs
   touch src/auth/oauth.rs
   touch src/auth/token.rs
   touch src/auth/gateway.rs
   touch src/auth/policy.rs
   touch src/auth/audit.rs
   ```

2. **Add Dependencies to Cargo.toml**
   ```toml
   [dependencies]
   # OAuth 2.1 and JWT
   oauth2 = "4.4"
   jsonwebtoken = "9.3"
   jwks-client = "0.4"
   
   # HTTP client for OAuth flows
   reqwest = { version = "0.12", features = ["json"] }
   
   # Cryptographic operations
   ring = "0.17"
   
   # Configuration and serialization
   config = "0.14"
   base64 = "0.22"
   sha2 = "0.10"
   ```

3. **Implement OAuth 2.1 Core Types**
   - `OAuth2Config` struct with all required fields
   - `PKCEChallenge` generation and validation
   - Basic error types in `src/auth/error.rs`

4. **PKCE Implementation**
   - Code verifier generation (43-128 chars, A-Z, a-z, 0-9, -._~)
   - Code challenge creation using SHA256
   - Unit tests for PKCE challenge/verifier pairs

**Success Criteria:**
- [ ] Auth module structure created
- [ ] Dependencies added and building successfully
- [ ] PKCE challenge generation working
- [ ] Basic unit tests passing (5+ tests)

**Files Created:**
- `src/auth/mod.rs`
- `src/auth/oauth.rs` 
- `src/auth/error.rs`
- `src/auth/pkce.rs`

### Day 2: OAuth 2.1 Flow Implementation

**Goals:** Complete OAuth authorization code flow with PKCE

**Tasks:**
1. **OAuth2AuthFlow Implementation**
   ```rust
   impl OAuth2AuthFlow {
       pub fn generate_auth_url(&self, scopes: &[String]) -> Result<(String, PKCEChallenge), AuthError>;
       pub async fn exchange_code_for_tokens(&self, auth_code: &str, pkce_verifier: &PKCEChallenge) -> Result<TokenResponse, AuthError>;
       pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse, AuthError>;
   }
   ```

2. **HTTP Client Integration**
   - Implement token endpoint communication
   - Add proper error handling for OAuth errors
   - Handle HTTP client timeouts and retries

3. **Token Response Handling**
   - Parse OAuth token responses
   - Validate required fields (access_token, token_type, expires_in)
   - Handle optional fields (refresh_token, scope)

4. **Unit Tests**
   - Mock OAuth server responses
   - Test authorization URL generation
   - Test token exchange flows
   - Test error handling

**Success Criteria:**
- [ ] Complete OAuth flow implementation
- [ ] Token exchange working with mock server
- [ ] Error handling for invalid responses
- [ ] 10+ unit tests passing

**Files Modified:**
- `src/auth/oauth.rs` (complete implementation)
- `tests/unit/auth/oauth_test.rs` (new)

### Day 3: JWT Token Validation

**Goals:** Implement secure JWT token validation with JWKS

**Tasks:**
1. **TokenValidator Implementation**
   ```rust
   impl TokenValidator {
       pub async fn validate_token(&self, token: &str) -> Result<TokenClaims, AuthError>;
       pub async fn ensure_token_valid(&self, token: &str) -> Result<String, AuthError>;
       pub fn validate_audience(&self, claims: &TokenClaims, expected: &str) -> Result<(), AuthError>;
   }
   ```

2. **JWKS Client Integration**
   - Implement public key retrieval from JWKS endpoint
   - Add key caching with TTL
   - Handle key rotation gracefully

3. **JWT Claims Validation**
   - Validate signature using RS256/ES256
   - Check expiration (`exp` claim)
   - Validate issuer (`iss` claim)
   - Validate audience (`aud` claim) - critical for MCP security

4. **Token Claims Structure**
   ```rust
   pub struct TokenClaims {
       pub sub: String,        // Subject (user ID)
       pub aud: Vec<String>,   // Audience (critical for MCP)
       pub iss: String,        // Issuer
       pub exp: i64,           // Expiration
       pub iat: i64,           // Issued at
       pub scope: String,      // OAuth scopes
       pub mcp_permissions: Vec<String>, // MCP-specific permissions
   }
   ```

**Success Criteria:**
- [ ] JWT validation working with test tokens
- [ ] JWKS key retrieval and caching
- [ ] All required claim validations
- [ ] 8+ unit tests passing

**Files Created:**
- `src/auth/token.rs` (complete implementation)
- `src/auth/jwks.rs` (JWKS client)
- `tests/unit/auth/token_test.rs` (new)

### Day 4: AuthGateway Core Implementation

**Goals:** Implement central authentication gateway

**Tasks:**
1. **AuthGateway Structure**
   ```rust
   pub struct AuthGateway {
       oauth_config: OAuth2Config,
       token_validator: Arc<TokenValidator>,
       policy_engine: Arc<PolicyEngine>,
       audit_logger: Arc<AuditLogger>,
       token_cache: Arc<TokenCache>,
       metrics: Arc<AuthMetrics>,
   }
   ```

2. **Request Authentication**
   - Extract tokens from requests (Authorization header)
   - Validate tokens using TokenValidator
   - Create AuthContext with user information
   - Handle missing/invalid tokens appropriately

3. **Token Cache Implementation**
   - In-memory cache for validated tokens
   - TTL-based expiration
   - Thread-safe concurrent access

4. **Auth Context Creation**
   ```rust
   pub struct AuthContext {
       pub user_id: String,
       pub scopes: Vec<String>,
       pub permissions: Vec<String>,
       pub token_claims: TokenClaims,
       pub authenticated: bool,
   }
   ```

**Success Criteria:**
- [ ] AuthGateway structure implemented
- [ ] Token extraction and validation
- [ ] AuthContext creation working
- [ ] Token caching functional
- [ ] 6+ unit tests passing

**Files Created:**
- `src/auth/gateway.rs` (core implementation)
- `src/auth/cache.rs` (token cache)
- `tests/unit/auth/gateway_test.rs` (new)

### Day 5: Reverse Proxy Implementation

**Goals:** Implement the core reverse proxy HTTP server with authentication integration

**Tasks:**
1. **ReverseProxy Structure**
   ```rust
   pub struct ReverseProxy {
       auth_gateway: Arc<AuthGateway>,
       session_manager: Arc<SessionManager>,
       interceptor_chain: Arc<InterceptorChain>,
       tape_recorder: Option<Arc<TapeRecorder>>,
       upstream_clients: Arc<UpstreamClientPool>,
       config: ReverseProxyConfig,
   }
   ```

2. **HTTP Server with Axum**
   - Implement HTTP server listening on configured bind address
   - Handle incoming HTTP requests with proper MCP headers
   - Convert HTTP requests to TransportMessage format
   - Route responses back to HTTP clients

3. **Request Processing Pipeline**
   ```rust
   async fn handle_request(&self, request: HttpRequest) -> Result<HttpResponse, ProxyError> {
       // 1. Extract and validate authentication token
       let auth_context = self.auth_gateway.authenticate_request(&request).await?;
       
       // 2. Convert HTTP request to TransportMessage
       let transport_message = self.http_to_transport_message(request).await?;
       
       // 3. Create session and intercept context with auth
       let intercept_context = InterceptContext {
           session_id: SessionId::new(),
           message: transport_message.clone(),
           transport_type: TransportType::Http,
           auth_context: Some(auth_context),
       };
       
       // 4. Process through interceptor chain
       let action = self.interceptor_chain.intercept(&intercept_context).await?;
       
       // 5. Route to upstream based on action
       self.handle_intercept_action(action, auth_context, transport_message).await
   }
   ```

4. **Upstream Client Pool**
   - HTTP client pool for upstream MCP servers
   - Connection pooling and reuse
   - Server token management (never forward client tokens)

**Success Criteria:**
- [ ] HTTP server listening and accepting connections
- [ ] Authentication integrated in request pipeline
- [ ] HTTP to TransportMessage conversion working
- [ ] Upstream routing functional
- [ ] 8+ integration tests passing

**Files Created:**
- `src/proxy/reverse.rs` (complete implementation)
- `src/proxy/upstream.rs` (client pool)
- `tests/integration/reverse_proxy_test.rs` (new)

---

## Week 2: Policy Engine & Security Features

### Day 6: Policy Engine Foundation

**Goals:** Implement rule-based authorization policies

**Tasks:**
1. **Policy Engine Structure**
   ```rust
   pub struct PolicyEngine {
       policies: Arc<RwLock<Vec<SecurityPolicy>>>,
       policy_watcher: Option<FileWatcher>,
       metrics: Arc<PolicyMetrics>,
   }
   ```

2. **Policy Definition Types**
   ```rust
   pub struct SecurityPolicy {
       pub id: String,
       pub name: String,
       pub enabled: bool,
       pub conditions: PolicyCondition,
       pub actions: Vec<PolicyAction>,
       pub priority: u32,
   }
   ```

3. **Policy Condition Matching**
   - User-based conditions
   - Scope-based conditions  
   - Method-based conditions
   - Transport-based conditions
   - Logical operators (AND, OR, NOT)

4. **Policy Actions**
   ```rust
   pub enum PolicyAction {
       Allow,
       Deny(String),
       RequireScope(String),
       RateLimit { requests: u32, window: Duration },
       AuditLog { level: AuditLevel },
   }
   ```

**Success Criteria:**
- [ ] Policy engine structure implemented
- [ ] Policy condition evaluation working
- [ ] Policy action execution
- [ ] JSON policy loading
- [ ] 8+ unit tests passing

**Files Created:**
- `src/auth/policy.rs` (complete implementation)
- `src/auth/policy_types.rs` (policy definitions)
- `tests/unit/auth/policy_test.rs` (new)

### Day 7: Policy Integration & Hot-Reloading

**Goals:** Integrate policies with AuthGateway and add hot-reloading

**Tasks:**
1. **AuthGateway Policy Integration**
   ```rust
   impl AuthGateway {
       pub async fn authorize_action(&self, auth_context: &AuthContext, action: &PolicyAction) -> Result<AuthzDecision, AuthError>;
   }
   ```

2. **Policy File Hot-Reloading**
   - Use existing `notify` crate pattern from interceptor
   - Monitor policy files for changes
   - Atomic policy reloading with validation
   - Rollback on invalid policies

3. **Authorization Decision Types**
   ```rust
   pub enum AuthzDecision {
       Allow,
       Deny(String),
       Conditional(Vec<PolicyAction>),
   }
   ```

4. **Policy File Format**
   ```json
   {
     "version": "1.0",
     "policies": [
       {
         "id": "admin-access",
         "name": "Admin method access control",
         "enabled": true,
         "priority": 100,
         "conditions": {
           "operator": "and",
           "method": {"prefix": "admin/"},
           "auth_context": {"scope": {"contains": "admin"}}
         },
         "actions": ["allow"]
       }
     ]
   }
   ```

**Success Criteria:**
- [ ] Policy integration with AuthGateway
- [ ] Hot-reloading working
- [ ] Policy file validation
- [ ] Authorization decisions implemented
- [ ] 6+ integration tests passing

**Files Modified:**
- `src/auth/gateway.rs` (policy integration)
- `src/auth/policy.rs` (hot-reloading)
- `tests/integration/policy_test.rs` (new)

### Day 8: Audit Logging System

**Goals:** Implement comprehensive security audit logging

**Tasks:**
1. **AuditLogger Implementation**
   ```rust
   pub struct AuditLogger {
       storage: Arc<AuditStorage>,
       formatter: AuditFormatter,
       config: AuditConfig,
   }
   ```

2. **Audit Event Types**
   ```rust
   pub struct AuditEvent {
       pub timestamp: DateTime<Utc>,
       pub event_type: AuditEventType,
       pub session_id: SessionId,
       pub user_id: Option<String>,
       pub source_ip: Option<String>,
       pub method: Option<String>,
       pub result: AuditResult,
       pub details: Value,
   }
   ```

3. **Storage Implementation**
   - SQLite storage for audit events
   - Structured JSON details field
   - Indexed queries by timestamp, user, event type
   - Retention policy implementation

4. **Audit Event Integration**
   - Log authentication attempts
   - Log authorization decisions
   - Log policy violations
   - Log token operations
   - Log security events

**Success Criteria:**
- [ ] AuditLogger fully implemented
- [ ] SQLite storage working
- [ ] All security events logged
- [ ] Query interface for audit events
- [ ] 7+ unit tests passing

**Files Created:**
- `src/auth/audit.rs` (complete implementation)
- `src/auth/audit_storage.rs` (SQLite storage)
- `tests/unit/auth/audit_test.rs` (new)

### Day 9: Rate Limiting & Security Features

**Goals:** Implement rate limiting and additional security features

**Tasks:**
1. **Rate Limiting Implementation**
   ```rust
   pub struct RateLimiter {
       windows: Arc<RwLock<HashMap<String, WindowState>>>,
       config: RateLimitConfig,
   }
   ```

2. **Token Exchange Security**
   - Implement secure server token exchange
   - NEVER forward client tokens (critical requirement)
   - Server-specific token generation
   - Token expiration and refresh

3. **Security Metrics**
   ```rust
   pub struct AuthMetrics {
       auth_attempts: Counter,
       auth_failures: Counter,
       policy_violations: Counter,
       rate_limit_hits: Counter,
       token_validations: Histogram,
   }
   ```

4. **Security Violation Detection**
   - Detect suspicious patterns
   - Multiple failed authentication attempts
   - Policy violation patterns
   - Rate limiting violations

**Success Criteria:**
- [ ] Rate limiting working
- [ ] Token exchange security implemented
- [ ] Security metrics collection
- [ ] Violation detection
- [ ] 8+ unit tests passing

**Files Created:**
- `src/auth/rate_limit.rs` (rate limiting)
- `src/auth/metrics.rs` (security metrics)
- `src/auth/security.rs` (violation detection)

### Day 10: Reverse Proxy CLI & Final Integration

**Goals:** Complete reverse proxy CLI interface and final integration testing

**Tasks:**
1. **Reverse Proxy CLI Commands**
   ```bash
   shadowcat reverse --bind 0.0.0.0:8080 --upstream https://mcp-server.example.com
   shadowcat reverse status --show-sessions
   shadowcat auth login --provider default
   shadowcat auth policy list
   shadowcat auth audit --since 1h
   ```

2. **Enhanced Main CLI Integration**
   - Update `src/main.rs` to implement reverse proxy command
   - Remove "not yet implemented" placeholder
   - Add authentication and policy options
   - Configuration file support

3. **CLI Implementation**
   ```rust
   // Update Commands enum
   Commands::Reverse { bind, upstream, auth_config, policy_file } => {
       let config = load_reverse_proxy_config(auth_config, policy_file).await?;
       let reverse_proxy = ReverseProxy::new(config);
       reverse_proxy.start_server(&bind).await?;
   }
   ```

4. **End-to-End Integration Testing**
   - Complete reverse proxy flow: HTTP client → Shadowcat → upstream MCP server
   - OAuth 2.1 authentication flow testing
   - Policy enforcement in reverse proxy context
   - Session management with HTTP clients
   - Performance testing with concurrent requests
   - Audit logging verification

5. **Production Readiness**
   - Graceful shutdown handling
   - Health check endpoints
   - Metrics exposure
   - Configuration validation

**Success Criteria:**
- [ ] `shadowcat reverse` command fully implemented
- [ ] Complete authentication + reverse proxy integration
- [ ] All integration tests passing (20+ tests)
- [ ] Performance benchmarks met (< 5ms auth overhead)
- [ ] Production deployment ready

**Files Modified:**
- `src/main.rs` (reverse proxy implementation)
- `src/cli/mod.rs` (CLI structure updates)
- `tests/integration/e2e_reverse_proxy_test.rs` (end-to-end)

---

## Quality Assurance & Testing

### Test Coverage Targets

- **Unit Tests:** 95% coverage for all auth modules
- **Integration Tests:** Complete flow testing
- **Security Tests:** Penetration testing scenarios
- **Performance Tests:** Latency and throughput validation

### Test Categories

**Unit Tests (Target: 80+ tests)**
- OAuth 2.1 flow components (15 tests)
- PKCE generation and validation (8 tests)
- JWT token validation (12 tests)
- Policy engine evaluation (15 tests)
- Audit logging (10 tests)
- Rate limiting (8 tests)
- Configuration (5 tests)
- Error handling (7 tests)

**Integration Tests (Target: 15+ tests)**
- End-to-end OAuth flow (3 tests)
- Proxy authentication integration (4 tests)
- Policy enforcement (3 tests)
- Audit logging (2 tests)
- Performance benchmarks (3 tests)

**Security Tests (Target: 10+ tests)**
- Token manipulation attempts
- Policy bypass attempts
- Rate limiting effectiveness
- PKCE security validation
- Audit log integrity

### Performance Benchmarks

- **Authentication Overhead:** < 5ms per request
- **Token Validation:** < 2ms (cached)
- **Policy Evaluation:** < 1ms per rule
- **Memory Usage:** < 10MB additional
- **Startup Time:** < 100ms additional

---

## Dependencies & Configuration

### New Dependencies

```toml
[dependencies]
# OAuth 2.1 and JWT
oauth2 = "4.4"
jsonwebtoken = "9.3"
jwks-client = "0.4"

# HTTP client
reqwest = { version = "0.12", features = ["json"] }

# Cryptography
ring = "0.17"
sha2 = "0.10"
base64 = "0.22"

# Time and serialization  
chrono = { version = "0.4", features = ["serde"] }

# Configuration
config = "0.14"

# Rate limiting
governor = "0.6"

# Audit storage
sqlx = { version = "0.8", features = ["sqlite", "chrono"] }
```

### Configuration Files

**`config/auth.yaml`**
```yaml
oauth:
  providers:
    - name: "default"
      client_id: "${OAUTH_CLIENT_ID}"
      authorization_endpoint: "https://auth.example.com/oauth/authorize"
      token_endpoint: "https://auth.example.com/oauth/token"
      jwks_uri: "https://auth.example.com/.well-known/jwks.json"

policy:
  enabled: true
  policy_file: "config/security-policies.json"
  hot_reload: true

audit:
  enabled: true
  storage: "sqlite"
  connection_string: "audit.db"
  retention_days: 90
```

**`config/security-policies.json`**
```json
{
  "version": "1.0", 
  "policies": [
    {
      "id": "admin-access-control",
      "name": "Admin method access control",
      "enabled": true,
      "priority": 100,
      "conditions": {
        "operator": "and",
        "method": {"prefix": "admin/"},
        "auth_context": {"scope": {"contains": "admin"}}
      },
      "actions": ["allow"]
    }
  ]
}
```

---

## Risk Mitigation

### Technical Risks

**Risk: OAuth 2.1 Complexity**
- *Mitigation:* Start with well-tested libraries, comprehensive unit tests
- *Fallback:* Implement basic OAuth 2.0 with PKCE if needed

**Risk: JWT Validation Performance**
- *Mitigation:* Implement aggressive caching, async validation
- *Fallback:* Simpler token validation if performance issues

**Risk: Policy Engine Complexity**
- *Mitigation:* Start with simple conditions, iterate to complex
- *Fallback:* Basic allow/deny policies if complex evaluation fails

### Security Risks

**Risk: Token Leakage**
- *Mitigation:* Never log tokens, secure storage, proper cleanup
- *Monitoring:* Audit all token operations

**Risk: Policy Bypass**
- *Mitigation:* Fail-secure evaluation, comprehensive testing
- *Monitoring:* Log all authorization decisions

**Risk: Rate Limiting Bypass**
- *Mitigation:* Multiple rate limiting layers, distributed tracking
- *Monitoring:* Alert on unusual patterns

---

## Success Criteria

### Functional Requirements ✅

- [ ] **OAuth 2.1 Compliance**: Full implementation with mandatory PKCE
- [ ] **MCP Authentication**: Meet all MCP security requirements
- [ ] **Token Security**: Never forward client tokens upstream
- [ ] **Policy Engine**: Rule-based access control working
- [ ] **Audit Logging**: Comprehensive security event logging
- [ ] **Rate Limiting**: Protection against abuse
- [ ] **CLI Interface**: Complete management interface
- [ ] **Configuration**: Flexible YAML/JSON configuration
- [ ] **Hot-Reloading**: Live policy updates without restart

### Performance Requirements ✅

- [ ] **Authentication Overhead**: < 5ms per request
- [ ] **Memory Usage**: < 10MB additional memory
- [ ] **Startup Time**: < 100ms additional startup time
- [ ] **Token Validation**: < 2ms with caching
- [ ] **Policy Evaluation**: < 1ms per rule

### Quality Requirements ✅

- [ ] **Test Coverage**: 95% unit test coverage
- [ ] **Integration Tests**: Complete flow testing
- [ ] **Security Tests**: Penetration testing validation
- [ ] **Documentation**: Complete API and usage docs
- [ ] **Backwards Compatibility**: Existing configs work unchanged

### Compliance Requirements ✅

- [ ] **OAuth 2.1 Compliance**: All required features implemented
- [ ] **MCP Security**: All MCP auth requirements met
- [ ] **PKCE Mandatory**: Required for all OAuth flows
- [ ] **Audit Compliance**: Comprehensive security logging
- [ ] **Enterprise Security**: Policy engine and access control

---

## Deliverables

### Code Deliverables

1. **Reverse Proxy** (`src/proxy/reverse.rs`)
   - Complete HTTP server implementation with Axum
   - Request processing pipeline with authentication
   - Upstream client pool and connection management
   - Session management for HTTP clients
   - Production-ready deployment features

2. **Auth Module** (`src/auth/`)
   - Complete OAuth 2.1 implementation with PKCE
   - JWT token validation with JWKS
   - Policy engine with hot-reloading
   - Audit logging system
   - Rate limiting and security features

3. **CLI Integration** (`src/main.rs`)
   - Complete `shadowcat reverse` command implementation
   - Authentication configuration management
   - Policy and audit management commands
   - Production deployment options

### Documentation Deliverables

1. **Architecture Documentation**
   - Security architecture overview
   - OAuth 2.1 implementation details
   - Policy engine documentation

2. **User Documentation**
   - Authentication setup guide
   - Policy configuration guide
   - CLI command reference

3. **Security Documentation**
   - Threat model and mitigations
   - Security best practices
   - Audit logging guide

### Testing Deliverables

1. **Test Suite**
   - 80+ unit tests
   - 15+ integration tests
   - 10+ security tests
   - Performance benchmarks

2. **Test Documentation**
   - Test strategy document
   - Security test results
   - Performance test results

---

This roadmap provides a comprehensive plan for implementing OAuth 2.1 compliant authentication and authorization in Shadowcat while maintaining the high quality standards established in previous phases.