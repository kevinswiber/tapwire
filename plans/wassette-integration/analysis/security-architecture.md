# Wassette-Shadowcat Security Architecture

## Overview

This document defines the comprehensive security architecture for the Wassette-Shadowcat integration, ensuring defense-in-depth while maintaining the security guarantees of both systems.

## Security Principles

1. **Zero Trust**: Never trust, always verify at every layer
2. **Least Privilege**: Grant minimal permissions required
3. **Defense in Depth**: Multiple security layers
4. **Token Isolation**: Strict boundary enforcement
5. **Audit Everything**: Comprehensive logging
6. **Fail Secure**: Deny by default on errors

## Token Flow Architecture

### Token Boundaries

```
┌──────────────────────────────────────────────────────────┐
│                     Token Zone 1                          │
│                  (Client Authentication)                  │
│  ┌────────┐                                              │
│  │ Client │ ─────OAuth 2.1 Token────►                    │
│  └────────┘                                              │
└──────────────────────────────────┬───────────────────────┘
                                   │
                    ╔══════════════▼══════════════╗
                    ║   Token Stripping Boundary   ║
                    ╚══════════════▼══════════════╝
                                   │
┌──────────────────────────────────┴───────────────────────┐
│                     Token Zone 2                          │
│                   (Internal Trust)                        │
│  ┌───────────┐         No Tokens          ┌──────────┐  │
│  │ Shadowcat │ ◄──────────────────────────►│ Wassette │  │
│  └───────────┘      Process Boundary       └──────────┘  │
└──────────────────────────────────┬───────────────────────┘
                                   │
                    ╔══════════════▼══════════════╗
                    ║   Capability Boundary        ║
                    ╚══════════════▼══════════════╝
                                   │
┌──────────────────────────────────┴───────────────────────┐
│                     Token Zone 3                          │
│                 (Component Capabilities)                  │
│  ┌──────────┐      WASI Capabilities      ┌──────────┐  │
│  │Component │ ◄───────────────────────────►│Resources │  │
│  └──────────┘                              └──────────┘  │
└───────────────────────────────────────────────────────────┘
```

### Token Stripping Implementation

```rust
pub struct TokenStripper {
    sensitive_headers: HashSet<String>,
    sensitive_params: HashSet<String>,
}

impl TokenStripper {
    pub fn new() -> Self {
        let mut sensitive_headers = HashSet::new();
        sensitive_headers.insert("Authorization".to_string());
        sensitive_headers.insert("Cookie".to_string());
        sensitive_headers.insert("X-API-Key".to_string());
        sensitive_headers.insert("X-Auth-Token".to_string());
        
        let mut sensitive_params = HashSet::new();
        sensitive_params.insert("api_key".to_string());
        sensitive_params.insert("access_token".to_string());
        sensitive_params.insert("refresh_token".to_string());
        
        Self {
            sensitive_headers,
            sensitive_params,
        }
    }
    
    pub fn strip_message(&self, mut message: ProtocolMessage) -> ProtocolMessage {
        match &mut message {
            ProtocolMessage::Request { params, .. } => {
                self.strip_params(params);
            }
            _ => {}
        }
        message
    }
    
    fn strip_params(&self, params: &mut Value) {
        if let Value::Object(map) = params {
            for sensitive_key in &self.sensitive_params {
                if map.contains_key(sensitive_key) {
                    map.insert(sensitive_key.clone(), Value::String("[REDACTED]".to_string()));
                }
            }
        }
    }
}
```

### Authentication Gateway

```rust
pub struct AuthenticationGateway {
    jwt_validator: JwtValidator,
    oauth_client: OAuth2Client,
    session_store: SessionStore,
}

impl AuthenticationGateway {
    pub async fn authenticate(&self, request: &Request) -> Result<AuthContext> {
        // Extract token from request
        let token = self.extract_token(request)?;
        
        // Validate JWT
        let claims = self.jwt_validator.validate(&token).await?;
        
        // Check token expiry
        if claims.exp < Utc::now().timestamp() {
            return Err(AuthError::TokenExpired);
        }
        
        // Verify audience
        if !claims.aud.contains(&self.expected_audience()) {
            return Err(AuthError::InvalidAudience);
        }
        
        // Create auth context (no token included!)
        Ok(AuthContext {
            user_id: claims.sub,
            permissions: claims.permissions,
            session_id: SessionId::new(),
            expires_at: claims.exp,
        })
    }
    
    fn extract_token(&self, request: &Request) -> Result<String> {
        // Try Bearer token first
        if let Some(auth_header) = request.headers.get("Authorization") {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                return Ok(token.to_string());
            }
        }
        
        // Try cookie
        if let Some(cookie) = request.headers.get("Cookie") {
            if let Some(token) = self.parse_cookie_token(cookie) {
                return Ok(token);
            }
        }
        
        Err(AuthError::NoToken)
    }
}
```

## Policy Integration

### Unified Policy Model

```rust
pub struct UnifiedPolicyEngine {
    shadowcat_policies: PolicyStore,
    wassette_policies: HashMap<ComponentId, WassettePolicy>,
    policy_resolver: PolicyResolver,
}

impl UnifiedPolicyEngine {
    pub async fn evaluate(&self, request: &PolicyRequest) -> PolicyDecision {
        // Layer 1: Shadowcat policies (authentication, rate limiting)
        let shadowcat_decision = self.shadowcat_policies.evaluate(request).await;
        if shadowcat_decision == PolicyDecision::Deny {
            return PolicyDecision::Deny;
        }
        
        // Layer 2: Wassette policies (component capabilities)
        if let Some(component_id) = &request.component_id {
            if let Some(wassette_policy) = self.wassette_policies.get(component_id) {
                let wassette_decision = wassette_policy.evaluate(request).await;
                if wassette_decision == PolicyDecision::Deny {
                    return PolicyDecision::Deny;
                }
            }
        }
        
        // Layer 3: Combined policies (additional restrictions)
        self.policy_resolver.resolve(shadowcat_decision, request).await
    }
}
```

### Policy Configuration

```yaml
# Unified policy configuration
policies:
  shadowcat:
    authentication:
      required: true
      methods:
        - jwt
        - oauth2
    
    rate_limiting:
      default:
        requests_per_minute: 100
        burst: 150
      
      per_user:
        premium:
          requests_per_minute: 1000
          burst: 1500
    
    access_control:
      default_role: user
      roles:
        admin:
          - "*"
        user:
          - "tools/call"
          - "tools/list"
  
  wassette:
    default_policy: deny-all
    component_policies:
      fetch-rs:
        permissions:
          network:
            allow:
              - host: "api.example.com"
              - host: "*.trusted-domain.com"
          storage:
            - uri: "fs://tmp/**"
              access: [read, write]
  
  combined:
    require_signed_components: true
    max_component_size: 10MB
    allowed_registries:
      - "registry.example.com"
      - "docker.io/trusted-org/*"
```

## Audit System

### Audit Event Schema

```rust
#[derive(Serialize, Deserialize)]
pub struct AuditEvent {
    // Core fields
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub severity: AuditSeverity,
    
    // Context
    pub session_id: Option<SessionId>,
    pub user_id: Option<String>,
    pub component_id: Option<String>,
    
    // Request details
    pub request_method: Option<String>,
    pub request_params: Option<Value>,
    pub request_source: IpAddr,
    
    // Response details
    pub response_status: Option<ResponseStatus>,
    pub response_error: Option<String>,
    
    // Security
    pub security_context: SecurityContext,
    pub policy_decisions: Vec<PolicyDecision>,
    
    // Performance
    pub duration_ms: Option<u64>,
    pub bytes_transferred: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub enum AuditEventType {
    // Authentication
    AuthenticationAttempt,
    AuthenticationSuccess,
    AuthenticationFailure,
    TokenRefresh,
    
    // Authorization
    AccessGranted,
    AccessDenied,
    PolicyViolation,
    
    // Component operations
    ComponentLoad,
    ComponentUnload,
    ComponentInvocation,
    ComponentError,
    
    // Security events
    SuspiciousActivity,
    RateLimitExceeded,
    SignatureVerificationFailure,
    
    // System events
    SystemStartup,
    SystemShutdown,
    ConfigurationChange,
}

#[derive(Serialize, Deserialize)]
pub enum AuditSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}
```

### Audit Storage

```rust
pub struct AuditStorage {
    database: SqlitePool,
    encryption_key: Key,
    retention_policy: RetentionPolicy,
}

impl AuditStorage {
    pub async fn store_event(&self, event: AuditEvent) -> Result<()> {
        // Encrypt sensitive fields
        let encrypted_event = self.encrypt_sensitive_fields(event)?;
        
        // Store in database
        sqlx::query!(
            r#"
            INSERT INTO audit_events (
                id, timestamp, event_type, severity,
                session_id, user_id, component_id,
                request_method, request_params, request_source,
                response_status, response_error,
                security_context, policy_decisions,
                duration_ms, bytes_transferred
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            encrypted_event.id,
            encrypted_event.timestamp,
            encrypted_event.event_type,
            encrypted_event.severity,
            // ... other fields
        )
        .execute(&self.database)
        .await?;
        
        // Trigger alerts if needed
        self.check_alert_conditions(&event).await?;
        
        Ok(())
    }
    
    fn encrypt_sensitive_fields(&self, mut event: AuditEvent) -> Result<AuditEvent> {
        // Encrypt request params that might contain sensitive data
        if let Some(params) = event.request_params {
            event.request_params = Some(self.encrypt_value(params)?);
        }
        Ok(event)
    }
}
```

### Audit Queries

```rust
pub struct AuditQuery {
    filters: AuditFilters,
    database: SqlitePool,
}

impl AuditQuery {
    pub async fn security_incidents(&self, time_range: TimeRange) -> Result<Vec<AuditEvent>> {
        sqlx::query_as!(
            AuditEvent,
            r#"
            SELECT * FROM audit_events
            WHERE severity IN ('Error', 'Critical')
            AND event_type IN (
                'AuthenticationFailure',
                'PolicyViolation',
                'SuspiciousActivity',
                'SignatureVerificationFailure'
            )
            AND timestamp BETWEEN ? AND ?
            ORDER BY timestamp DESC
            "#,
            time_range.start,
            time_range.end
        )
        .fetch_all(&self.database)
        .await
    }
    
    pub async fn user_activity(&self, user_id: &str, time_range: TimeRange) -> Result<Vec<AuditEvent>> {
        sqlx::query_as!(
            AuditEvent,
            r#"
            SELECT * FROM audit_events
            WHERE user_id = ?
            AND timestamp BETWEEN ? AND ?
            ORDER BY timestamp DESC
            "#,
            user_id,
            time_range.start,
            time_range.end
        )
        .fetch_all(&self.database)
        .await
    }
}
```

## Component Security

### Component Verification

```rust
pub struct ComponentVerifier {
    trusted_keys: HashMap<String, PublicKey>,
    allowed_registries: Vec<String>,
    signature_validator: SignatureValidator,
}

impl ComponentVerifier {
    pub async fn verify_component(&self, component: &ComponentData) -> Result<VerificationResult> {
        // Check source registry
        if !self.is_allowed_registry(&component.source) {
            return Ok(VerificationResult::UntrustedSource);
        }
        
        // Verify signature
        if let Some(signature) = &component.signature {
            let key = self.get_signing_key(&component.source)?;
            if !self.signature_validator.verify(component.data, signature, &key)? {
                return Ok(VerificationResult::InvalidSignature);
            }
        } else if self.require_signatures() {
            return Ok(VerificationResult::MissingSignature);
        }
        
        // Scan for malicious patterns
        if self.detect_malicious_patterns(&component.data)? {
            return Ok(VerificationResult::MaliciousContent);
        }
        
        Ok(VerificationResult::Verified)
    }
    
    fn detect_malicious_patterns(&self, data: &[u8]) -> Result<bool> {
        // Check for known malicious bytecode patterns
        // This would integrate with security scanning tools
        Ok(false)
    }
}
```

### Runtime Security Monitoring

```rust
pub struct SecurityMonitor {
    anomaly_detector: AnomalyDetector,
    threat_intelligence: ThreatIntelligence,
    incident_responder: IncidentResponder,
}

impl SecurityMonitor {
    pub async fn monitor_invocation(&self, invocation: &ComponentInvocation) -> Result<()> {
        // Check for anomalous behavior
        if self.anomaly_detector.is_anomalous(invocation).await? {
            self.handle_anomaly(invocation).await?;
        }
        
        // Check against threat intelligence
        if let Some(threat) = self.threat_intelligence.check(invocation).await? {
            self.handle_threat(threat, invocation).await?;
        }
        
        // Update behavior baseline
        self.anomaly_detector.update_baseline(invocation).await?;
        
        Ok(())
    }
    
    async fn handle_anomaly(&self, invocation: &ComponentInvocation) -> Result<()> {
        // Log security event
        self.log_security_event(SecurityEvent::AnomalousB behavior {
            component_id: invocation.component_id.clone(),
            details: self.anomaly_detector.get_details(invocation),
        }).await?;
        
        // Potentially block or rate limit
        if self.should_block_anomaly(invocation) {
            return Err(SecurityError::AnomalyDetected);
        }
        
        Ok(())
    }
}
```

## Secure Configuration

### Configuration Encryption

```rust
pub struct SecureConfig {
    encrypted_values: HashMap<String, EncryptedValue>,
    key_manager: KeyManager,
}

impl SecureConfig {
    pub fn get_secret(&self, key: &str) -> Result<String> {
        let encrypted = self.encrypted_values.get(key)
            .ok_or_else(|| anyhow!("Secret not found"))?;
        
        let decryption_key = self.key_manager.get_key(&encrypted.key_id)?;
        let decrypted = decrypt(encrypted.ciphertext, &decryption_key)?;
        
        Ok(String::from_utf8(decrypted)?)
    }
    
    pub fn set_secret(&mut self, key: &str, value: &str) -> Result<()> {
        let encryption_key = self.key_manager.get_current_key()?;
        let encrypted = encrypt(value.as_bytes(), &encryption_key)?;
        
        self.encrypted_values.insert(key.to_string(), EncryptedValue {
            key_id: encryption_key.id.clone(),
            ciphertext: encrypted,
            created_at: Utc::now(),
        });
        
        Ok(())
    }
}
```

### Secure Defaults

```yaml
# Default security configuration
security:
  defaults:
    # Authentication
    require_authentication: true
    session_timeout: 3600
    max_session_duration: 86400
    
    # Token management
    strip_tokens: true
    token_validation: strict
    
    # Component security
    require_signed_components: false  # Start permissive, tighten later
    verify_component_hashes: true
    max_component_size: 10MB
    
    # Network security
    tls_required: true
    min_tls_version: "1.3"
    allowed_ciphers:
      - TLS_AES_256_GCM_SHA384
      - TLS_CHACHA20_POLY1305_SHA256
    
    # Rate limiting
    rate_limit_enabled: true
    default_rate_limit: 100
    burst_multiplier: 1.5
    
    # Audit
    audit_enabled: true
    audit_level: info
    audit_retention_days: 90
    
    # Monitoring
    security_monitoring: true
    anomaly_detection: true
    threat_intelligence_feeds:
      - https://threat-intel.example.com/feed
```

## Incident Response

### Incident Detection

```rust
pub struct IncidentDetector {
    rules: Vec<DetectionRule>,
    ml_model: Option<AnomalyModel>,
}

impl IncidentDetector {
    pub async fn analyze_event(&self, event: &AuditEvent) -> Option<Incident> {
        // Rule-based detection
        for rule in &self.rules {
            if rule.matches(event) {
                return Some(self.create_incident(event, rule));
            }
        }
        
        // ML-based detection
        if let Some(model) = &self.ml_model {
            if model.is_anomalous(event) {
                return Some(self.create_ml_incident(event));
            }
        }
        
        None
    }
}
```

### Incident Response

```rust
pub struct IncidentResponder {
    response_plans: HashMap<IncidentType, ResponsePlan>,
    notification_service: NotificationService,
}

impl IncidentResponder {
    pub async fn respond(&self, incident: Incident) -> Result<()> {
        let plan = self.response_plans.get(&incident.incident_type)
            .ok_or_else(|| anyhow!("No response plan for incident type"))?;
        
        // Execute immediate actions
        for action in &plan.immediate_actions {
            self.execute_action(action, &incident).await?;
        }
        
        // Send notifications
        self.notification_service.notify_incident(&incident).await?;
        
        // Create incident ticket
        self.create_incident_ticket(&incident).await?;
        
        // Start remediation
        if plan.auto_remediate {
            self.start_remediation(&incident, plan).await?;
        }
        
        Ok(())
    }
    
    async fn execute_action(&self, action: &ResponseAction, incident: &Incident) -> Result<()> {
        match action {
            ResponseAction::BlockUser(user_id) => {
                // Block user access
            }
            ResponseAction::DisableComponent(component_id) => {
                // Disable component
            }
            ResponseAction::IncreaseLogging => {
                // Increase logging level
            }
            ResponseAction::TriggerBackup => {
                // Trigger backup
            }
        }
        Ok(())
    }
}
```

## Security Testing

### Security Test Framework

```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_token_stripping() {
        let stripper = TokenStripper::new();
        let message = create_message_with_token();
        
        let stripped = stripper.strip_message(message);
        
        assert!(!contains_token(&stripped));
    }
    
    #[tokio::test]
    async fn test_policy_enforcement() {
        let engine = UnifiedPolicyEngine::new();
        let request = PolicyRequest {
            user_id: "test_user".to_string(),
            component_id: Some("restricted_component".to_string()),
            action: "execute".to_string(),
        };
        
        let decision = engine.evaluate(&request).await;
        
        assert_eq!(decision, PolicyDecision::Deny);
    }
    
    #[tokio::test]
    async fn test_component_verification() {
        let verifier = ComponentVerifier::new();
        let malicious_component = create_malicious_component();
        
        let result = verifier.verify_component(&malicious_component).await.unwrap();
        
        assert_eq!(result, VerificationResult::MaliciousContent);
    }
}
```

### Penetration Testing

```rust
pub struct PenTestFramework {
    test_suites: Vec<Box<dyn PenTestSuite>>,
}

impl PenTestFramework {
    pub async fn run_all_tests(&self) -> PenTestReport {
        let mut report = PenTestReport::new();
        
        for suite in &self.test_suites {
            let suite_results = suite.run().await;
            report.add_suite_results(suite_results);
        }
        
        report
    }
}

#[async_trait]
trait PenTestSuite {
    async fn run(&self) -> SuiteResults;
}

struct AuthenticationPenTest;

#[async_trait]
impl PenTestSuite for AuthenticationPenTest {
    async fn run(&self) -> SuiteResults {
        let mut results = SuiteResults::new("Authentication");
        
        // Test token injection
        results.add(self.test_token_injection().await);
        
        // Test token replay
        results.add(self.test_token_replay().await);
        
        // Test privilege escalation
        results.add(self.test_privilege_escalation().await);
        
        results
    }
}
```

## Compliance

### Compliance Monitoring

```rust
pub struct ComplianceMonitor {
    requirements: Vec<ComplianceRequirement>,
    audit_store: AuditStorage,
}

impl ComplianceMonitor {
    pub async fn generate_compliance_report(&self) -> ComplianceReport {
        let mut report = ComplianceReport::new();
        
        for requirement in &self.requirements {
            let status = self.check_requirement(requirement).await;
            report.add_requirement_status(requirement, status);
        }
        
        report
    }
    
    async fn check_requirement(&self, requirement: &ComplianceRequirement) -> ComplianceStatus {
        match requirement {
            ComplianceRequirement::AuditLogging { retention_days } => {
                self.check_audit_retention(*retention_days).await
            }
            ComplianceRequirement::Encryption { algorithm } => {
                self.check_encryption_algorithm(algorithm).await
            }
            ComplianceRequirement::AccessControl { rbac_required } => {
                self.check_access_control(*rbac_required).await
            }
        }
    }
}
```

## Security Recommendations

### Critical Security Controls

1. **Always enable token stripping**
2. **Enforce component signature verification in production**
3. **Implement rate limiting at proxy layer**
4. **Enable comprehensive audit logging**
5. **Regular security updates for both systems**

### Security Checklist

- [ ] Token stripping configured and tested
- [ ] Authentication gateway enabled
- [ ] Component verification active
- [ ] Audit logging to persistent storage
- [ ] Security monitoring dashboards configured
- [ ] Incident response plan documented
- [ ] Regular security testing scheduled
- [ ] Compliance requirements mapped
- [ ] Encryption keys properly managed
- [ ] Network security (TLS) configured

### Security Best Practices

1. **Development Environment**
   - Use test certificates
   - Enable verbose logging
   - Relaxed rate limits
   - Component signature warnings (not errors)

2. **Production Environment**
   - Production certificates with proper rotation
   - Structured logging to SIEM
   - Strict rate limits
   - Mandatory component signatures
   - Regular security audits

3. **Key Management**
   - Use hardware security modules (HSM) in production
   - Regular key rotation (quarterly)
   - Separate keys for different environments
   - Secure key backup and recovery

4. **Monitoring and Alerting**
   - Real-time security dashboards
   - Automated incident detection
   - Integration with incident management
   - Regular security reports

This security architecture provides comprehensive protection while maintaining usability and performance. The layered approach ensures that even if one security control fails, others provide continued protection.