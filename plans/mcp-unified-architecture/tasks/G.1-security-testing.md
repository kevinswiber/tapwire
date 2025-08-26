# Task G.1: Security Testing and Hardening

## Objective
Implement comprehensive security testing including dependency auditing, input fuzzing, and denial-of-service protection to ensure production readiness.

## Background
From Gemini's review: "Explicitly add a task for a preliminary security review including dependency audit, input fuzzing, and denial-of-service testing."

## Key Requirements

### 1. Dependency Security Audit
```bash
# Setup cargo-audit
cargo install cargo-audit

# Run audit
cargo audit

# Generate SBOM (Software Bill of Materials)
cargo sbom > sbom.json

# Check for known vulnerabilities
cargo audit --deny warnings

# CI integration
name: Security Audit
on:
  schedule:
    - cron: '0 0 * * *'  # Daily
  push:
    paths:
      - '**/Cargo.toml'
      - '**/Cargo.lock'

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

### 2. Input Fuzzing
```rust
#![cfg(test)]
use arbitrary::{Arbitrary, Unstructured};

#[derive(Debug, Arbitrary)]
struct FuzzInput {
    json_rpc: String,
    method: String,
    params: Vec<u8>,
    id: Option<FuzzId>,
}

#[derive(Debug, Arbitrary)]
enum FuzzId {
    Number(i64),
    String(String),
    Null,
}

#[test]
fn fuzz_json_rpc_parser() {
    use libfuzzer_sys::fuzz_target;
    
    fuzz_target!(|data: &[u8]| {
        // Generate arbitrary input
        let mut u = Unstructured::new(data);
        let input = match FuzzInput::arbitrary(&mut u) {
            Ok(input) => input,
            Err(_) => return,
        };
        
        // Try to parse as JSON-RPC
        let json = format!(
            r#"{{"jsonrpc":"{}","method":"{}","params":{},"id":{}}}"#,
            input.json_rpc,
            input.method,
            String::from_utf8_lossy(&input.params),
            match input.id {
                Some(FuzzId::Number(n)) => n.to_string(),
                Some(FuzzId::String(s)) => format!(r#""{}""#, s),
                Some(FuzzId::Null) | None => "null".to_string(),
            }
        );
        
        // Should not panic
        let _ = serde_json::from_str::<JsonRpcRequest>(&json);
    });
}

// Structured fuzzing for protocol messages
#[derive(Debug, Arbitrary)]
struct FuzzMessage {
    #[arbitrary(with = fuzz_json_rpc_version)]
    jsonrpc: String,
    
    #[arbitrary(with = fuzz_method_name)]
    method: String,
    
    params: Option<serde_json::Value>,
    
    id: Option<JsonRpcId>,
}

fn fuzz_json_rpc_version(u: &mut Unstructured) -> Result<String, arbitrary::Error> {
    Ok(match u.int_in_range(0..=3)? {
        0 => "2.0".to_string(),
        1 => "1.0".to_string(),
        2 => "".to_string(),
        _ => u.arbitrary::<String>()?,
    })
}

fn fuzz_method_name(u: &mut Unstructured) -> Result<String, arbitrary::Error> {
    Ok(match u.int_in_range(0..=5)? {
        0 => "initialize".to_string(),
        1 => "shutdown".to_string(),
        2 => format!("method_{}", u.int_in_range(0..=1000)?),
        3 => "a".repeat(u.int_in_range(1..=10000)?), // Long method name
        4 => String::from_utf8_lossy(&u.bytes(100)?).to_string(), // Random bytes
        _ => u.arbitrary::<String>()?,
    })
}

// Property-based testing with proptest
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_request_response_roundtrip(
        method in "[a-zA-Z][a-zA-Z0-9_]*",
        params in prop::option::of(any::<serde_json::Value>()),
        id in prop::option::of(any::<u64>().prop_map(JsonRpcId::Number))
    ) {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id: id.clone(),
        };
        
        // Serialize and deserialize
        let json = serde_json::to_string(&request).unwrap();
        let parsed: JsonRpcRequest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(request.method, parsed.method);
        assert_eq!(request.id, parsed.id);
    }
    
    #[test]
    fn test_session_id_validation(
        input in prop::string::string_regex("[a-zA-Z0-9-_]{1,128}").unwrap()
    ) {
        let session_id = SessionId::try_from(input.as_str());
        prop_assert!(session_id.is_ok());
        
        let session_id = session_id.unwrap();
        prop_assert!(session_id.as_str().len() <= 128);
    }
}
```

### 3. Denial-of-Service Protection
```rust
pub struct DosProtection {
    connection_limiter: ConnectionLimiter,
    request_limiter: RequestLimiter,
    payload_validator: PayloadValidator,
    slowloris_detector: SlowlorisDetector,
}

impl DosProtection {
    pub fn new(config: DosProtectionConfig) -> Self {
        Self {
            connection_limiter: ConnectionLimiter::new(config.max_connections_per_ip),
            request_limiter: RequestLimiter::new(config.rate_limit),
            payload_validator: PayloadValidator::new(config.max_payload_size),
            slowloris_detector: SlowlorisDetector::new(config.header_timeout),
        }
    }
}

// Connection limiting per IP
pub struct ConnectionLimiter {
    limits: Arc<RwLock<HashMap<IpAddr, ConnectionCount>>>,
    max_per_ip: usize,
}

impl ConnectionLimiter {
    pub async fn check_and_increment(&self, addr: IpAddr) -> Result<ConnectionGuard, Error> {
        let mut limits = self.limits.write().await;
        let count = limits.entry(addr).or_default();
        
        if count.active >= self.max_per_ip {
            return Err(Error::TooManyConnections);
        }
        
        count.active += 1;
        Ok(ConnectionGuard {
            limiter: self.clone(),
            addr,
        })
    }
}

// Slowloris attack detection
pub struct SlowlorisDetector {
    header_timeout: Duration,
    connections: Arc<RwLock<HashMap<ConnectionId, Instant>>>,
}

impl SlowlorisDetector {
    pub async fn track_connection(&self, conn_id: ConnectionId) {
        let mut connections = self.connections.write().await;
        connections.insert(conn_id, Instant::now());
        
        // Start timeout task
        let detector = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(detector.header_timeout).await;
            
            let mut connections = detector.connections.write().await;
            if let Some(start) = connections.get(&conn_id) {
                if start.elapsed() >= detector.header_timeout {
                    tracing::warn!(?conn_id, "Slowloris attack detected, closing connection");
                    // Force close the connection
                }
            }
        });
    }
    
    pub async fn headers_received(&self, conn_id: ConnectionId) {
        let mut connections = self.connections.write().await;
        connections.remove(&conn_id);
    }
}

// Large message DoS protection
pub struct PayloadValidator {
    max_size: usize,
    max_array_depth: usize,
    max_object_depth: usize,
}

impl PayloadValidator {
    pub fn validate_json(&self, json: &str) -> Result<(), Error> {
        if json.len() > self.max_size {
            return Err(Error::PayloadTooLarge);
        }
        
        // Check JSON complexity
        let depth = self.calculate_depth(json)?;
        if depth > self.max_object_depth {
            return Err(Error::PayloadTooComplex);
        }
        
        Ok(())
    }
    
    fn calculate_depth(&self, json: &str) -> Result<usize, Error> {
        let mut depth = 0;
        let mut max_depth = 0;
        
        for ch in json.chars() {
            match ch {
                '{' | '[' => {
                    depth += 1;
                    max_depth = max_depth.max(depth);
                    if depth > self.max_object_depth {
                        return Err(Error::PayloadTooComplex);
                    }
                },
                '}' | ']' => depth = depth.saturating_sub(1),
                _ => {},
            }
        }
        
        Ok(max_depth)
    }
}

#[cfg(test)]
mod dos_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_connection_flooding() {
        let server = create_protected_server().await;
        let addr = server.local_addr();
        
        // Try to flood with connections
        let mut connections = vec![];
        for _ in 0..1000 {
            match TcpStream::connect(addr).await {
                Ok(conn) => connections.push(conn),
                Err(e) => {
                    // Should start rejecting after limit
                    assert!(connections.len() >= MAX_CONNECTIONS_PER_IP);
                    break;
                }
            }
        }
        
        assert!(connections.len() <= MAX_CONNECTIONS_PER_IP);
    }
    
    #[tokio::test]
    async fn test_slowloris_protection() {
        let server = create_protected_server().await;
        
        // Connect but send headers very slowly
        let mut conn = TcpStream::connect(server.local_addr()).await.unwrap();
        
        // Send partial HTTP request
        conn.write_all(b"POST /mcp HTTP/1.1\r\n").await.unwrap();
        
        // Wait longer than header timeout
        tokio::time::sleep(Duration::from_secs(35)).await;
        
        // Connection should be closed by server
        let mut buf = [0u8; 1];
        let result = conn.read(&mut buf).await;
        assert_eq!(result.unwrap(), 0); // Connection closed
    }
    
    #[tokio::test]
    async fn test_large_payload_rejection() {
        let client = create_test_client();
        
        // Create a very large payload
        let large_params = serde_json::json!({
            "data": "x".repeat(100_000_000) // 100MB
        });
        
        let result = client.request("test.method", large_params).await;
        assert!(matches!(result, Err(Error::PayloadTooLarge)));
    }
    
    #[tokio::test] 
    async fn test_deeply_nested_json() {
        let client = create_test_client();
        
        // Create deeply nested JSON
        let mut nested = serde_json::json!({});
        for _ in 0..1000 {
            nested = serde_json::json!({ "nested": nested });
        }
        
        let result = client.request("test.method", nested).await;
        assert!(matches!(result, Err(Error::PayloadTooComplex)));
    }
}
```

### 4. Security Test Suite
```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[test]
    fn test_no_sensitive_data_in_errors() {
        let errors = vec![
            Error::DatabaseConnection("password=secret123".into()),
            Error::AuthFailed("token: Bearer abc123".into()),
            Error::ConfigParse("api_key: sk-1234567890".into()),
        ];
        
        for error in errors {
            let error_string = error.to_string();
            assert!(!error_string.contains("password"));
            assert!(!error_string.contains("token"));
            assert!(!error_string.contains("api_key"));
            assert!(!error_string.contains("secret"));
        }
    }
    
    #[test]
    fn test_constant_time_token_comparison() {
        use subtle::ConstantTimeEq;
        
        let token1 = b"correct_token_value";
        let token2 = b"incorrect_token_val";
        let token3 = b"correct_token_value";
        
        // Should use constant-time comparison
        assert!(token1.ct_eq(token3).unwrap_u8() == 1);
        assert!(token1.ct_eq(token2).unwrap_u8() == 0);
    }
    
    #[tokio::test]
    async fn test_sql_injection_prevention() {
        // If using SQL for session storage
        let store = SqliteSessionStore::new(":memory:").await.unwrap();
        
        // Try SQL injection in session ID
        let malicious_id = SessionId::from("'; DROP TABLE sessions; --");
        let result = store.get(&malicious_id).await;
        
        // Should handle safely (parameterized queries)
        assert!(result.is_ok());
        
        // Verify table still exists
        let count = sqlx::query!("SELECT COUNT(*) as count FROM sessions")
            .fetch_one(&store.pool)
            .await
            .unwrap();
        assert!(count.count >= 0);
    }
    
    #[test]
    fn test_path_traversal_prevention() {
        let paths = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32",
            "recordings/../../../sensitive",
        ];
        
        for path in paths {
            let result = validate_recording_path(path);
            assert!(result.is_err());
        }
    }
}
```

### 5. Security Configuration
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    /// DoS protection settings
    pub dos_protection: DosProtectionConfig,
    
    /// Input validation settings
    pub validation: ValidationConfig,
    
    /// Authentication settings
    pub auth: AuthConfig,
    
    /// TLS configuration
    pub tls: Option<TlsConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DosProtectionConfig {
    /// Maximum connections per IP address
    pub max_connections_per_ip: usize,
    
    /// Maximum payload size in bytes
    pub max_payload_size: usize,
    
    /// Header receive timeout (Slowloris protection)
    pub header_timeout: Duration,
    
    /// Request rate limiting
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ValidationConfig {
    /// Maximum JSON nesting depth
    pub max_json_depth: usize,
    
    /// Maximum array size
    pub max_array_size: usize,
    
    /// Maximum string length
    pub max_string_length: usize,
    
    /// Allowed methods regex
    pub allowed_methods_pattern: String,
}
```

## Implementation Steps

1. **Set up security tooling** (30 min)
   - Install cargo-audit
   - Configure cargo-fuzz
   - Set up proptest

2. **Implement fuzzing tests** (2 hours)
   - JSON-RPC parser fuzzing
   - Session ID validation
   - Method parameter fuzzing

3. **Add DoS protection** (2 hours)
   - Connection limiting
   - Slowloris detection
   - Payload validation

4. **Security test suite** (1.5 hours)
   - SQL injection tests
   - Path traversal tests
   - Sensitive data leakage tests

5. **CI integration** (30 min)
   - Dependency audit workflow
   - Fuzzing in CI
   - Security scanning

6. **Security documentation** (30 min)
   - Security best practices
   - Threat model
   - Incident response

## Testing Strategy

1. **Automated Security Scanning**
   - Run cargo-audit on every commit
   - Weekly dependency updates
   - SBOM generation

2. **Continuous Fuzzing**
   - Run fuzzers for 1 hour daily
   - Store corpus for regression
   - Track coverage improvements

3. **Penetration Testing**
   - Attempt common attacks
   - Verify all mitigations work
   - Document any findings

## Success Criteria

- [ ] Zero high/critical vulnerabilities in dependencies
- [ ] Fuzzer runs for 1M iterations without crashes
- [ ] All DoS attacks mitigated
- [ ] No sensitive data in logs/errors
- [ ] Security documentation complete

## Risk Mitigation

1. **Zero-day Vulnerabilities**: Regular dependency updates, security monitoring
2. **Novel Attack Vectors**: Defense in depth, multiple layers of protection
3. **Performance Impact**: Benchmark security features, make configurable

## Dependencies
- Core MCP implementation
- Testing infrastructure

## Estimated Duration
7 hours

## Notes
- Consider integrating with security scanning services (Snyk, etc.)
- May need rate limiting at reverse proxy level too
- Document security configuration best practices for production