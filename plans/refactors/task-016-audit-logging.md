# Task 016: Add Audit Logging

## Overview
Implement comprehensive audit logging for security events, authentication attempts, and critical operations to enable security monitoring and compliance.

## Context
From the [comprehensive review](../../reviews/shadowcat-comprehensive-review-2025-08-06.md), "Missing Audit Logging" is identified as a medium risk security vulnerability with no security event tracking currently in place.

## Scope
- **Files to modify**: `src/audit/`, auth modules, proxy modules
- **Priority**: MEDIUM - Security and compliance
- **Time estimate**: 1 day

## Current Problem

### Missing Capabilities
- No security event tracking
- No authentication attempt logging
- No audit trail for configuration changes
- No compliance-ready logging
- Limited visibility into suspicious activities

## Implementation Plan

### Step 1: Define Audit Event Types

```rust
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum AuditEvent {
    /// Authentication events
    AuthenticationAttempt {
        user_id: Option<String>,
        method: AuthMethod,
        success: bool,
        ip_address: IpAddr,
        user_agent: Option<String>,
        reason: Option<String>,
    },
    
    /// Authorization events
    AuthorizationCheck {
        user_id: String,
        resource: String,
        action: String,
        granted: bool,
        policy: Option<String>,
    },
    
    /// Session events
    SessionCreated {
        session_id: String,
        user_id: Option<String>,
        ip_address: IpAddr,
        transport: String,
    },
    
    SessionTerminated {
        session_id: String,
        reason: SessionTerminationReason,
        duration_ms: u64,
    },
    
    /// Configuration changes
    ConfigurationChanged {
        changed_by: String,
        setting: String,
        old_value: Option<String>,
        new_value: String,
    },
    
    /// Security events
    SecurityViolation {
        violation_type: SecurityViolationType,
        ip_address: IpAddr,
        details: String,
        blocked: bool,
    },
    
    /// Rate limiting
    RateLimitExceeded {
        ip_address: IpAddr,
        limit_type: String,
        limit: u32,
        window_secs: u64,
    },
    
    /// Circuit breaker events
    CircuitBreakerStateChange {
        service: String,
        old_state: String,
        new_state: String,
        reason: String,
    },
    
    /// Data access
    SensitiveDataAccess {
        user_id: String,
        resource_type: String,
        resource_id: String,
        operation: DataOperation,
    },
    
    /// System events
    SystemStartup {
        version: String,
        config_hash: String,
    },
    
    SystemShutdown {
        reason: String,
        graceful: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    OAuth,
    ApiKey,
    Certificate,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionTerminationReason {
    UserLogout,
    Timeout,
    AdminAction,
    SecurityViolation,
    SystemShutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityViolationType {
    InvalidInput,
    PathTraversal,
    SqlInjection,
    XssAttempt,
    UnauthorizedAccess,
    SuspiciousPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataOperation {
    Read,
    Write,
    Delete,
    Export,
}
```

### Step 2: Create Audit Logger

```rust
use tokio::sync::mpsc;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event: AuditEvent,
    pub correlation_id: Option<String>,
    pub metadata: HashMap<String, Value>,
}

impl AuditLogEntry {
    pub fn new(event: AuditEvent) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            event,
            correlation_id: None,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_correlation_id(mut self, id: String) -> Self {
        self.correlation_id = Some(id);
        self
    }
    
    pub fn with_metadata(mut self, key: String, value: Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

pub struct AuditLogger {
    sender: mpsc::Sender<AuditLogEntry>,
    config: AuditConfig,
}

#[derive(Debug, Clone)]
pub struct AuditConfig {
    pub enabled: bool,
    pub log_file: PathBuf,
    pub max_file_size: u64,
    pub retention_days: u32,
    pub buffer_size: usize,
    pub include_sensitive: bool,
    pub async_logging: bool,
}

impl AuditLogger {
    pub fn new(config: AuditConfig) -> Self {
        let (sender, receiver) = mpsc::channel(config.buffer_size);
        
        if config.enabled {
            tokio::spawn(Self::run_writer(receiver, config.clone()));
        }
        
        Self { sender, config }
    }
    
    pub async fn log(&self, event: AuditEvent) -> Result<(), AuditError> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let entry = AuditLogEntry::new(event);
        
        if self.config.async_logging {
            // Non-blocking send
            let _ = self.sender.try_send(entry);
        } else {
            // Blocking send
            self.sender.send(entry).await
                .map_err(|_| AuditError::ChannelClosed)?;
        }
        
        Ok(())
    }
    
    pub async fn log_with_context(
        &self,
        event: AuditEvent,
        correlation_id: String,
        metadata: HashMap<String, Value>,
    ) -> Result<(), AuditError> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let mut entry = AuditLogEntry::new(event);
        entry.correlation_id = Some(correlation_id);
        entry.metadata = metadata;
        
        self.sender.send(entry).await
            .map_err(|_| AuditError::ChannelClosed)?;
        
        Ok(())
    }
    
    async fn run_writer(
        mut receiver: mpsc::Receiver<AuditLogEntry>,
        config: AuditConfig,
    ) {
        let mut file = match Self::open_log_file(&config.log_file).await {
            Ok(f) => f,
            Err(e) => {
                tracing::error!("Failed to open audit log file: {}", e);
                return;
            }
        };
        
        let mut current_size = file.metadata().await
            .map(|m| m.len())
            .unwrap_or(0);
        
        while let Some(entry) = receiver.recv().await {
            let json = match serde_json::to_string(&entry) {
                Ok(j) => j,
                Err(e) => {
                    tracing::error!("Failed to serialize audit entry: {}", e);
                    continue;
                }
            };
            
            let line = format!("{}\n", json);
            let bytes = line.as_bytes();
            
            // Rotate if needed
            if current_size + bytes.len() as u64 > config.max_file_size {
                if let Err(e) = Self::rotate_log_file(&config.log_file).await {
                    tracing::error!("Failed to rotate audit log: {}", e);
                }
                
                file = match Self::open_log_file(&config.log_file).await {
                    Ok(f) => f,
                    Err(e) => {
                        tracing::error!("Failed to open new audit log: {}", e);
                        return;
                    }
                };
                
                current_size = 0;
            }
            
            if let Err(e) = file.write_all(bytes).await {
                tracing::error!("Failed to write audit entry: {}", e);
            } else {
                current_size += bytes.len() as u64;
                
                // Flush for important events
                if Self::is_critical_event(&entry.event) {
                    let _ = file.flush().await;
                }
            }
        }
    }
    
    async fn open_log_file(path: &Path) -> Result<File, io::Error> {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await
    }
    
    async fn rotate_log_file(path: &Path) -> Result<(), io::Error> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let rotated_name = format!(
            "{}.{}",
            path.display(),
            timestamp
        );
        
        tokio::fs::rename(path, rotated_name).await
    }
    
    fn is_critical_event(event: &AuditEvent) -> bool {
        matches!(event,
            AuditEvent::SecurityViolation { .. } |
            AuditEvent::AuthenticationAttempt { success: false, .. } |
            AuditEvent::SystemShutdown { graceful: false, .. }
        )
    }
}
```

### Step 3: Integrate with Authentication

```rust
// In src/auth/oauth.rs

impl OAuthGateway {
    async fn authenticate(
        &self,
        request: &Request,
        audit_logger: &AuditLogger,
    ) -> Result<User, AuthError> {
        let ip_address = self.extract_ip(&request);
        let user_agent = self.extract_user_agent(&request);
        
        let result = self.verify_token(request).await;
        
        // Log authentication attempt
        let event = AuditEvent::AuthenticationAttempt {
            user_id: result.as_ref().ok().map(|u| u.id.clone()),
            method: AuthMethod::OAuth,
            success: result.is_ok(),
            ip_address,
            user_agent,
            reason: result.as_ref().err().map(|e| e.to_string()),
        };
        
        audit_logger.log(event).await?;
        
        result
    }
}
```

### Step 4: Add Security Violation Detection

```rust
pub struct SecurityMonitor {
    audit_logger: Arc<AuditLogger>,
    patterns: Vec<SecurityPattern>,
}

#[derive(Debug)]
struct SecurityPattern {
    name: String,
    regex: regex::Regex,
    violation_type: SecurityViolationType,
}

impl SecurityMonitor {
    pub fn new(audit_logger: Arc<AuditLogger>) -> Self {
        let patterns = vec![
            SecurityPattern {
                name: "SQL Injection".to_string(),
                regex: regex::Regex::new(r"('|(--|#|/\*|\*/))").unwrap(),
                violation_type: SecurityViolationType::SqlInjection,
            },
            SecurityPattern {
                name: "Path Traversal".to_string(),
                regex: regex::Regex::new(r"\.\.(/|\\)").unwrap(),
                violation_type: SecurityViolationType::PathTraversal,
            },
            SecurityPattern {
                name: "XSS".to_string(),
                regex: regex::Regex::new(r"<script|javascript:|onerror=").unwrap(),
                violation_type: SecurityViolationType::XssAttempt,
            },
        ];
        
        Self {
            audit_logger,
            patterns,
        }
    }
    
    pub async fn check_input(
        &self,
        input: &str,
        ip_address: IpAddr,
    ) -> Result<(), SecurityError> {
        for pattern in &self.patterns {
            if pattern.regex.is_match(input) {
                let event = AuditEvent::SecurityViolation {
                    violation_type: pattern.violation_type.clone(),
                    ip_address,
                    details: format!("Pattern '{}' detected", pattern.name),
                    blocked: true,
                };
                
                self.audit_logger.log(event).await?;
                
                return Err(SecurityError::ViolationDetected(pattern.name.clone()));
            }
        }
        
        Ok(())
    }
}
```

### Step 5: Add Compliance Features

```rust
pub struct ComplianceAuditor {
    logger: Arc<AuditLogger>,
    pii_fields: HashSet<String>,
}

impl ComplianceAuditor {
    pub fn new(logger: Arc<AuditLogger>) -> Self {
        let mut pii_fields = HashSet::new();
        pii_fields.insert("email".to_string());
        pii_fields.insert("ssn".to_string());
        pii_fields.insert("credit_card".to_string());
        pii_fields.insert("phone".to_string());
        
        Self {
            logger,
            pii_fields,
        }
    }
    
    pub async fn log_data_access(
        &self,
        user_id: String,
        resource_type: String,
        resource_id: String,
        operation: DataOperation,
        fields_accessed: Vec<String>,
    ) -> Result<(), AuditError> {
        // Check if PII was accessed
        let accessed_pii: Vec<_> = fields_accessed
            .iter()
            .filter(|f| self.pii_fields.contains(*f))
            .cloned()
            .collect();
        
        if !accessed_pii.is_empty() {
            let event = AuditEvent::SensitiveDataAccess {
                user_id: user_id.clone(),
                resource_type: resource_type.clone(),
                resource_id: resource_id.clone(),
                operation: operation.clone(),
            };
            
            let mut metadata = HashMap::new();
            metadata.insert(
                "pii_fields".to_string(),
                json!(accessed_pii),
            );
            
            self.logger.log_with_context(
                event,
                Uuid::new_v4().to_string(),
                metadata,
            ).await?;
        }
        
        Ok(())
    }
}
```

### Step 6: Add Query and Export Capabilities

```rust
pub struct AuditQueryBuilder {
    filters: Vec<AuditFilter>,
}

#[derive(Debug)]
pub enum AuditFilter {
    TimeRange {
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    },
    EventType(String),
    UserId(String),
    IpAddress(IpAddr),
    Success(bool),
}

impl AuditQueryBuilder {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }
    
    pub fn time_range(
        mut self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        self.filters.push(AuditFilter::TimeRange { start, end });
        self
    }
    
    pub fn event_type(mut self, event_type: String) -> Self {
        self.filters.push(AuditFilter::EventType(event_type));
        self
    }
    
    pub async fn execute(&self, log_file: &Path) -> Result<Vec<AuditLogEntry>, AuditError> {
        let file = tokio::fs::File::open(log_file).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut results = Vec::new();
        
        while let Some(line) = lines.next_line().await? {
            if let Ok(entry) = serde_json::from_str::<AuditLogEntry>(&line) {
                if self.matches(&entry) {
                    results.push(entry);
                }
            }
        }
        
        Ok(results)
    }
    
    fn matches(&self, entry: &AuditLogEntry) -> bool {
        for filter in &self.filters {
            match filter {
                AuditFilter::TimeRange { start, end } => {
                    if entry.timestamp < *start || entry.timestamp > *end {
                        return false;
                    }
                }
                AuditFilter::EventType(t) => {
                    // Match based on event type
                    // Implementation depends on event structure
                }
                // ... other filters
            }
        }
        true
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_audit_logging() {
        let config = AuditConfig {
            enabled: true,
            log_file: PathBuf::from("/tmp/test_audit.log"),
            max_file_size: 1024 * 1024,
            retention_days: 7,
            buffer_size: 100,
            include_sensitive: false,
            async_logging: false,
        };
        
        let logger = AuditLogger::new(config);
        
        let event = AuditEvent::AuthenticationAttempt {
            user_id: Some("user123".to_string()),
            method: AuthMethod::OAuth,
            success: true,
            ip_address: "127.0.0.1".parse().unwrap(),
            user_agent: Some("TestAgent".to_string()),
            reason: None,
        };
        
        logger.log(event).await.unwrap();
        
        // Verify log was written
        let contents = tokio::fs::read_to_string("/tmp/test_audit.log").await.unwrap();
        assert!(contents.contains("AuthenticationAttempt"));
        assert!(contents.contains("user123"));
    }
    
    #[tokio::test]
    async fn test_security_violation_detection() {
        let logger = Arc::new(AuditLogger::new(Default::default()));
        let monitor = SecurityMonitor::new(logger);
        
        let malicious_inputs = vec![
            "../../../etc/passwd",
            "'; DROP TABLE users; --",
            "<script>alert('xss')</script>",
        ];
        
        for input in malicious_inputs {
            let result = monitor.check_input(input, "127.0.0.1".parse().unwrap()).await;
            assert!(result.is_err());
        }
    }
    
    #[tokio::test]
    async fn test_log_rotation() {
        let config = AuditConfig {
            enabled: true,
            log_file: PathBuf::from("/tmp/test_rotate.log"),
            max_file_size: 100,  // Small size to trigger rotation
            ..Default::default()
        };
        
        let logger = AuditLogger::new(config);
        
        // Log many events to trigger rotation
        for i in 0..10 {
            let event = AuditEvent::SessionCreated {
                session_id: format!("session_{}", i),
                user_id: None,
                ip_address: "127.0.0.1".parse().unwrap(),
                transport: "stdio".to_string(),
            };
            logger.log(event).await.unwrap();
        }
        
        // Check for rotated files
        let dir = tokio::fs::read_dir("/tmp").await.unwrap();
        // ... verify rotation occurred
    }
}
```

## Validation

### Pre-check
```bash
# Check for audit logging
rg "audit|log.*security" --type rust | wc -l
```

### Post-check
```bash
# Audit logging implemented
rg "AuditLogger|AuditEvent" --type rust | wc -l  # Should be significant

# Run tests
cargo test audit

# Check log output
tail -f /var/log/shadowcat/audit.log
```

## Success Criteria

- [ ] All security events logged
- [ ] Authentication attempts tracked
- [ ] Configuration changes audited
- [ ] PII access logged for compliance
- [ ] Log rotation implemented
- [ ] Query capabilities available
- [ ] Performance impact < 2%
- [ ] All tests pass
- [ ] Compliance requirements met

## Configuration Example

```toml
[audit]
enabled = true
log_file = "/var/log/shadowcat/audit.log"
max_file_size_mb = 100
retention_days = 90
buffer_size = 1000
include_sensitive = false
async_logging = true

[audit.filters]
log_authentication = true
log_authorization = true
log_security_violations = true
log_data_access = true
log_configuration_changes = true
```

## Compliance Considerations

### GDPR
- PII access logging
- Data retention policies
- Right to audit access

### SOC2
- Security event tracking
- Authentication logging
- Change management audit trail

### HIPAA
- PHI access logging
- Minimum necessary principle
- Audit log retention (6 years)

## Performance Considerations

1. **Async logging** - Don't block main operations
2. **Buffered writes** - Batch log entries
3. **Selective logging** - Configure what to log
4. **Efficient serialization** - Consider binary formats for high volume

## Integration Points

- Authentication system (OAuth, API keys)
- Session management
- Configuration management
- Security validation (Task 014)
- Circuit breaker events (Task 015)

## Notes

- Essential for security monitoring
- Required for many compliance standards
- Consider integration with SIEM systems
- May want to support multiple output formats (JSON, CEF, syslog)