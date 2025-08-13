# Phase 5: Reverse Proxy & Authentication Architecture Design

**Project:** Shadowcat Phase 5 - Reverse Proxy with OAuth 2.1 Authentication  
**Design Date:** August 4, 2025  
**Implementation Target:** Phase 5 (Weeks 9-10)  
**Dependencies:** Phase 4 Complete (Interception & Rule Engine)

---

## Executive Summary

Phase 5 implements the **Reverse Proxy** component with integrated OAuth 2.1 authentication, making Shadowcat production-ready for secure MCP gateway deployments. Unlike the forward proxy (development tool), the reverse proxy is where clients connect TO Shadowcat as an authenticated API gateway.

**Key Goals:**
- **Reverse Proxy Implementation**: HTTP server accepting client connections
- **OAuth 2.1 Authentication Gateway**: Clients authenticate to Shadowcat 
- **MCP Security Compliance**: All MCP authentication requirements met
- **Zero Client Token Passthrough**: Critical security requirement
- **Policy-Based Authorization**: Fine-grained access control
- **Production-Ready Deployment**: Enterprise security features

---

## Architecture Overview

### Reverse Proxy Architecture

```
┌─────────────────┐    ┌─────────────────────────────────────┐    ┌─────────────────┐
│   MCP Client    │    │          Shadowcat Reverse Proxy    │    │   MCP Server    │
│                 │    │                                     │    │                 │
│                 │    │ ┌─────────────┐ ┌─────────────────┐ │    │                 │
│ HTTP Request    │───▶│ │ HTTP Server │ │ AuthGateway     │ │───▶│ Upstream MCP    │
│ + Bearer Token  │    │ │ (axum)      │ │ OAuth 2.1+PKCE │ │    │ Server          │
│                 │    │ └─────────────┘ └─────────────────┘ │    │                 │
│                 │    │         │               │           │    │                 │
│                 │    │ ┌─────────────┐ ┌─────────────────┐ │    │                 │
│ Response        │◀───│ │ Response    │ │ Policy Engine   │ │◀───│ Server Response │
│                 │    │ │ Handler     │ │ + Audit Log     │ │    │                 │
└─────────────────┘    │ └─────────────┘ └─────────────────┘ │    └─────────────────┘
                       └─────────────────────────────────────┘
```

### Reverse Proxy Message Flow

```
HTTP Request → AuthGateway → SessionManager → InterceptorChain → ReverseProxy → Upstream Server
      ↓              ↓              ↓               ↓                   ↓
Token Validation  Session Context   Frame Recording  Rule Evaluation   HTTP Client
Policy Check      Session Headers   Audit Logging    Security Actions  Request
Audit Logging
```

---

## Core Components

### 1. ReverseProxy - HTTP Server & Request Router

**Location:** `src/proxy/reverse.rs`

```rust
pub struct ReverseProxy {
    auth_gateway: Arc<AuthGateway>,
    session_manager: Arc<SessionManager>,
    interceptor_chain: Arc<InterceptorChain>,
    tape_recorder: Option<Arc<TapeRecorder>>,
    upstream_clients: Arc<UpstreamClientPool>,
    config: ReverseProxyConfig,
}

impl ReverseProxy {
    /// Start HTTP server listening for client connections
    pub async fn start_server(&self, bind_address: &str) -> Result<(), ProxyError>;
    
    /// Handle incoming HTTP request with authentication
    pub async fn handle_request(
        &self,
        request: HttpRequest
    ) -> Result<HttpResponse, ProxyError>;
    
    /// Route authenticated request to upstream MCP server
    pub async fn route_to_upstream(
        &self,
        auth_context: AuthContext,
        request: TransportMessage
    ) -> Result<TransportMessage, ProxyError>;
}
```

### 2. AuthGateway - Central Authentication Hub

**Location:** `src/auth/gateway.rs`

```rust
pub struct AuthGateway {
    oauth_config: OAuth2Config,
    token_validator: Arc<TokenValidator>,
    policy_engine: Arc<PolicyEngine>,
    audit_logger: Arc<AuditLogger>,
    token_cache: Arc<TokenCache>,
    metrics: Arc<AuthMetrics>,
}

impl AuthGateway {
    /// Validates incoming request and extracts auth context
    pub async fn authenticate_request(
        &self, 
        request: &TransportMessage,
        transport_type: TransportType,
        session_id: &SessionId
    ) -> Result<AuthContext, AuthError>;
    
    /// Exchange client token for server token (NEVER forward client tokens)
    pub async fn exchange_for_server_token(
        &self,
        client_token: &str,
        target_server: &str
    ) -> Result<String, AuthError>;
    
    /// Check policy authorization for specific action
    pub async fn authorize_action(
        &self,
        auth_context: &AuthContext,
        action: &PolicyAction
    ) -> Result<AuthzDecision, AuthError>;
}
```

### 2. OAuth 2.1 Implementation with PKCE

**Location:** `src/auth/oauth.rs`

```rust
pub struct OAuth2Config {
    pub client_id: String,
    pub client_secret: Option<String>, // None for public clients
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
    pub pkce_required: bool, // Always true in OAuth 2.1
    pub supported_grant_types: Vec<GrantType>,
}

#[derive(Debug, Clone)]
pub struct PKCEChallenge {
    pub code_verifier: String,  // 43-128 char random string
    pub code_challenge: String, // SHA256(code_verifier) base64url
    pub code_challenge_method: String, // "S256" (required)
}

pub struct OAuth2AuthFlow {
    config: OAuth2Config,
    pkce_generator: PKCEGenerator,
    http_client: reqwest::Client,
}

impl OAuth2AuthFlow {
    /// Generate authorization URL with PKCE challenge
    pub fn generate_auth_url(&self, scopes: &[String]) -> Result<(String, PKCEChallenge), AuthError>;
    
    /// Exchange authorization code for tokens using PKCE verifier
    pub async fn exchange_code_for_tokens(
        &self,
        auth_code: &str,
        pkce_verifier: &PKCEChallenge
    ) -> Result<TokenResponse, AuthError>;
    
    /// Refresh access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse, AuthError>;
}
```

### 3. Token Validation & JWT Handling

**Location:** `src/auth/token.rs`

```rust
pub struct TokenValidator {
    jwks_client: JwksClient,
    issuer_validator: IssuerValidator,
    audience_validator: AudienceValidator,
    time_validator: TimeValidator,
}

impl TokenValidator {
    /// Validate JWT token and extract claims
    pub async fn validate_token(&self, token: &str) -> Result<TokenClaims, AuthError>;
    
    /// Check token expiration and refresh if needed
    pub async fn ensure_token_valid(&self, token: &str) -> Result<String, AuthError>;
    
    /// Validate audience claim (critical for MCP security)
    pub fn validate_audience(&self, claims: &TokenClaims, expected: &str) -> Result<(), AuthError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

### 4. Policy Engine

**Location:** `src/auth/policy.rs`

```rust
pub struct PolicyEngine {
    policies: Arc<RwLock<Vec<SecurityPolicy>>>,
    policy_watcher: Option<FileWatcher>,
    metrics: Arc<PolicyMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub conditions: PolicyCondition,
    pub actions: Vec<PolicyAction>,
    pub priority: u32,
}

#[derive(Debug, Clone)]
pub enum PolicyCondition {
    User(String),
    Scope(String),
    Method(String),
    Transport(TransportType),
    And(Box<PolicyCondition>, Box<PolicyCondition>),
    Or(Box<PolicyCondition>, Box<PolicyCondition>),
    Not(Box<PolicyCondition>),
}

#[derive(Debug, Clone)]
pub enum PolicyAction {
    Allow,
    Deny(String),
    RequireScope(String),
    RateLimit { requests: u32, window: Duration },
    AuditLog { level: AuditLevel },
}
```

### 5. Security Audit Logging

**Location:** `src/auth/audit.rs`

```rust
pub struct AuditLogger {
    storage: Arc<AuditStorage>,
    formatter: AuditFormatter,
    config: AuditConfig,
}

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub enum AuditEventType {
    Authentication,
    Authorization,
    TokenExchange,
    PolicyViolation,
    SecurityEvent,
}

impl AuditLogger {
    pub async fn log_auth_event(&self, event: AuditEvent) -> Result<(), AuditError>;
    pub async fn log_policy_decision(&self, decision: &AuthzDecision) -> Result<(), AuditError>;
    pub async fn log_security_violation(&self, violation: &SecurityViolation) -> Result<(), AuditError>;
}
```

---

## Integration with Existing Architecture

### 1. ReverseProxy Implementation

**New Component:** `src/proxy/reverse.rs`

```rust
impl ReverseProxy {
    /// Handle incoming HTTP request with full authentication flow
    pub async fn handle_request(&self, request: HttpRequest) -> Result<HttpResponse, ProxyError> {
        // 1. Extract and validate authentication token
        let auth_context = self.auth_gateway
            .authenticate_request(&request)
            .await?;
        
        // 2. Convert HTTP request to TransportMessage
        let transport_message = self.http_to_transport_message(request).await?;
        
        // 3. Create session and intercept context
        let session_id = SessionId::new();
        let intercept_context = InterceptContext {
            session_id: session_id.clone(),
            message: transport_message.clone(),
            transport_type: TransportType::Http,
            auth_context: Some(auth_context), // Authentication context included
        };
        
        // 4. Process through interceptor chain with auth-aware rules
        let action = self.interceptor_chain.intercept(&intercept_context).await?;
        
        // 5. Handle intercept action and route to upstream
        match action {
            InterceptAction::Continue => {
                self.route_to_upstream(auth_context, transport_message).await
            },
            InterceptAction::Block(reason) => {
                Ok(HttpResponse::forbidden(reason))
            },
            // ... handle other actions
        }
    }
    
    /// Route authenticated request to upstream MCP server
    async fn route_to_upstream(
        &self,
        auth_context: AuthContext,
        message: TransportMessage
    ) -> Result<TransportMessage, ProxyError> {
        // 1. Select upstream server based on auth context/policy
        let upstream = self.select_upstream(&auth_context).await?;
        
        // 2. Exchange client token for server token (NEVER forward client token)
        let server_token = self.auth_gateway
            .exchange_for_server_token(&auth_context.token, &upstream.url)
            .await?;
        
        // 3. Forward request with server token
        let client = self.upstream_clients.get_client(&upstream).await?;
        client.send_with_token(message, server_token).await
    }
}
```

### 2. InterceptContext Enhancement

**Enhancement:** `src/interceptor/mod.rs`

```rust
#[derive(Debug, Clone)]
pub struct InterceptContext {
    pub session_id: SessionId,
    pub message: TransportMessage,
    pub transport_type: TransportType,
    pub auth_context: Option<AuthContext>, // NEW: Authentication info
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub scopes: Vec<String>,
    pub permissions: Vec<String>,
    pub token_claims: TokenClaims,
    pub authenticated: bool,
}
```

### 3. Rule-Based Security Policies

**Enhancement:** `src/interceptor/rules.rs`

Rules can now match on authentication context:

```json
{
  "version": "1.0",
  "rules": [
    {
      "id": "require-admin-scope",
      "name": "Require admin scope for admin methods",
      "enabled": true,
      "priority": 1000,
      "match_conditions": {
        "operator": "and",
        "method": {
          "match_type": "prefix",
          "value": "admin/"
        },
        "auth_context": {
          "scope": {
            "match_type": "contains",
            "value": "admin"
          }
        }
      },
      "actions": [
        {
          "action_type": "block",
          "parameters": {
            "reason": "Admin scope required for admin methods"
          }
        }
      ]
    }
  ]
}
```

---

## Security Requirements Compliance

### OAuth 2.1 Compliance Checklist

- ✅ **PKCE Mandatory**: All authorization code flows use PKCE
- ✅ **Deprecated Grants Removed**: No implicit or password grants
- ✅ **Bearer Token Security**: No bearer tokens in query strings
- ✅ **Refresh Token Security**: One-time use for public clients
- ✅ **Redirect URI Validation**: Exact string matching
- ✅ **Client Authentication**: Prefer asymmetric key authentication

### MCP Security Requirements Compliance

- ✅ **OAuth 2.1 Implementation**: Full OAuth 2.1 with PKCE
- ✅ **Authorization Server Metadata**: RFC8414 support
- ✅ **Dynamic Client Registration**: RFC7591 support (optional)
- ✅ **Token Protection**: NEVER forward client tokens upstream
- ✅ **Access Control**: Fine-grained permissions for tools and resources
- ✅ **Audit Logging**: All authentication/authorization events logged

### Enterprise Security Features

- ✅ **Policy Engine**: Rule-based access control
- ✅ **Audit Trail**: Comprehensive security event logging
- ✅ **Token Management**: Secure token storage and rotation
- ✅ **Rate Limiting**: Protection against abuse
- ✅ **Multi-tenancy**: Support for multiple OAuth providers

---

## Implementation Plan

### Week 1: Core Authentication (Days 1-5)

**Day 1-2: OAuth 2.1 Foundation**
- Implement `OAuth2Config` and `OAuth2AuthFlow`
- Add PKCE challenge generation and validation
- Create basic token exchange functionality
- Unit tests for OAuth flow components

**Day 3-4: Token Validation**
- Implement `TokenValidator` with JWT validation
- Add JWKS client for public key retrieval
- Implement audience and issuer validation
- Add token caching and refresh logic

**Day 5: AuthGateway Integration**
- Implement core `AuthGateway` struct
- Integrate with `ForwardProxy` message routing
- Add authentication to `InterceptContext`
- Basic integration tests

### Week 2: Policy & Security (Days 6-10)

**Day 6-7: Policy Engine**
- Implement `PolicyEngine` with rule evaluation
- Add policy file loading and hot-reloading
- Create policy condition matching logic
- Policy-based authorization decisions

**Day 8-9: Audit & Security**
- Implement `AuditLogger` with structured events
- Add security violation detection
- Implement rate limiting and abuse protection
- Security metrics and monitoring

**Day 10: CLI & Integration**
- Add `shadowcat auth` CLI commands
- Implement configuration management
- Complete integration testing
- Performance validation

---

## Configuration

### OAuth 2.1 Configuration

```yaml
# config/auth.yaml
oauth:
  providers:
    - name: "default"
      client_id: "${OAUTH_CLIENT_ID}"
      client_secret: "${OAUTH_CLIENT_SECRET}"
      authorization_endpoint: "https://auth.example.com/oauth/authorize"
      token_endpoint: "https://auth.example.com/oauth/token"
      jwks_uri: "https://auth.example.com/.well-known/jwks.json"
      pkce_required: true
      supported_scopes: ["read", "write", "admin"]

policy:
  enabled: true
  policy_file: "config/security-policies.json"
  hot_reload: true
  default_action: "deny"

audit:
  enabled: true
  storage: "sqlite"
  connection_string: "audit.db"
  log_level: "info"
  retention_days: 90
```

### Security Policy Example

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
        "auth_context": {
          "scope": {"contains": "admin"}
        }
      },
      "actions": ["allow"]
    },
    {
      "id": "rate-limit-public",
      "name": "Rate limit for public users",
      "enabled": true,
      "priority": 200,
      "conditions": {
        "auth_context": {
          "scope": {"not_contains": "premium"}
        }
      },
      "actions": [
        {
          "type": "rate_limit",
          "requests": 100,
          "window": "60s"
        }
      ]
    }
  ]
}
```

---

## CLI Interface

### Auth Commands

```bash
# OAuth flow management
shadowcat auth login --provider default
shadowcat auth logout --all
shadowcat auth token --validate
shadowcat auth refresh

# Policy management
shadowcat auth policy list
shadowcat auth policy add ./security-policy.json
shadowcat auth policy validate ./policy.json
shadowcat auth policy remove policy-id

# Audit and monitoring
shadowcat auth audit --since 1h
shadowcat auth audit --user user123 --export csv
shadowcat auth metrics --security
```

### Running Reverse Proxy with Authentication

```bash
# Start reverse proxy with OAuth 2.1 authentication
shadowcat reverse --bind 0.0.0.0:8080 --upstream https://mcp-server.example.com \
  --auth-config ./auth.yaml

# Start with specific OAuth provider and policies
shadowcat reverse --bind 127.0.0.1:3000 --upstream http://localhost:8000 \
  --auth-provider github --policy-file ./security-policies.json

# Start with comprehensive security features
shadowcat reverse --bind 0.0.0.0:443 --upstream https://internal-mcp.company.com \
  --auth-config ./production-auth.yaml \
  --policy-file ./enterprise-policies.json \
  --audit-enabled --rate-limit-enabled

# Development mode with mock auth
shadowcat reverse --bind 127.0.0.1:8080 --upstream http://localhost:3000 \
  --auth-disabled --allow-anonymous
```

---

## Performance & Security Metrics

### Performance Targets

- **Authentication Overhead**: < 5ms per request
- **Token Validation**: < 2ms (cached)
- **Policy Evaluation**: < 1ms per rule
- **Memory Usage**: < 10MB additional for auth components
- **Startup Time**: < 100ms additional

### Security Metrics

- **Authentication Success Rate**: > 99%
- **Policy Violation Detection**: Real-time
- **Audit Event Processing**: < 10ms
- **Token Refresh Success**: > 99%
- **Rate Limiting Accuracy**: > 99%

### Monitoring Points

- Authentication attempts and failures
- Token validation errors
- Policy violations and decisions
- Rate limiting triggers
- Security event frequencies
- Performance degradation alerts

---

## Testing Strategy

### Unit Tests (Target: 95% Coverage)

```rust
// OAuth 2.1 flow testing
#[tokio::test]
async fn test_pkce_challenge_generation() {
    let flow = OAuth2AuthFlow::new(config);
    let (auth_url, challenge) = flow.generate_auth_url(&["read"]).unwrap();
    assert!(challenge.code_verifier.len() >= 43);
    assert!(auth_url.contains("code_challenge="));
}

// Token validation testing
#[tokio::test]
async fn test_jwt_validation() {
    let validator = TokenValidator::new(jwks_client);
    let claims = validator.validate_token(valid_jwt).await.unwrap();
    assert_eq!(claims.sub, "user123");
}

// Policy engine testing
#[tokio::test]
async fn test_policy_evaluation() {
    let engine = PolicyEngine::new();
    let decision = engine.evaluate_policy(&auth_context, &policy).await.unwrap();
    assert_eq!(decision, AuthzDecision::Allow);
}
```

### Integration Tests

- End-to-end OAuth 2.1 flow with real auth server
- Policy enforcement in proxy message routing
- Audit logging verification
- Performance testing under load
- Security violation detection

### Security Tests

- Token manipulation attempts
- Policy bypass attempts  
- Rate limiting effectiveness
- Audit log integrity
- PKCE security validation

---

## Dependencies

### New Dependencies (to be added)

```toml
[dependencies]
# OAuth 2.1 and JWT
oauth2 = "4.4"
jsonwebtoken = "9.3"
jwks-client = "0.4"

# HTTP client for OAuth flows
reqwest = { version = "0.12", features = ["json"] }

# Policy and security
rego = "0.3"  # Optional: For complex policy evaluation
ring = "0.17"  # Cryptographic operations

# Audit and logging
sqlx = { version = "0.8", features = ["sqlite", "chrono"] }
chrono = { version = "0.4", features = ["serde"] }

# Configuration
config = "0.14"
```

---

## Migration & Backwards Compatibility

### Opt-in Authentication

Authentication is completely optional and can be enabled/disabled:

```rust
// Without authentication (current behavior)
let proxy = ForwardProxy::new()
    .with_session_manager(session_manager)
    .with_interceptor_chain(interceptor_chain);

// With authentication (new functionality) 
let proxy = ForwardProxy::new()
    .with_session_manager(session_manager)
    .with_interceptor_chain(interceptor_chain)
    .with_auth_gateway(auth_gateway);  // NEW
```

### Configuration Migration

- Current configurations continue to work unchanged
- New auth configuration is optional
- Clear upgrade path for existing deployments

---

## Security Considerations

### Critical Security Requirements

1. **NEVER Forward Client Tokens**: Absolute requirement for MCP compliance
2. **Token Storage Security**: Encrypted storage of refresh tokens
3. **Audit Trail Integrity**: Tamper-evident audit logging
4. **Policy Enforcement**: Fail-secure policy evaluation
5. **Rate Limiting**: Protection against abuse and DoS

### Threat Mitigation

- **Token Interception**: PKCE prevents authorization code attacks
- **Token Replay**: Short-lived tokens with proper validation
- **Policy Bypass**: Fail-secure evaluation with comprehensive logging
- **Resource Exhaustion**: Rate limiting and resource quotas
- **Data Exfiltration**: Fine-grained permission controls

---

## Conclusion

Phase 5 delivers enterprise-grade security for Shadowcat while maintaining the clean architecture and performance characteristics established in previous phases. The OAuth 2.1 implementation with mandatory PKCE provides modern security standards, while the policy engine enables fine-grained access control suitable for production MCP deployments.

**Success Metrics:**
- ✅ Full OAuth 2.1 compliance with PKCE
- ✅ MCP authentication requirements met
- ✅ Zero client token passthrough (critical requirement)
- ✅ Policy-based access control
- ✅ Comprehensive audit logging
- ✅ < 5ms authentication overhead
- ✅ Seamless integration with existing architecture
- ✅ Backwards compatibility maintained

The implementation provides a solid foundation for secure MCP proxy deployments while maintaining the flexibility and performance that makes Shadowcat suitable for development and production use cases.