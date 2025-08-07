# Task 014: Add Security Validation

## Overview
Implement comprehensive input validation and security measures to prevent attacks and ensure safe operation.

## Context
From the [comprehensive review](../../reviews/shadowcat-comprehensive-review-2025-08-06.md), several HIGH/MEDIUM risk security vulnerabilities were identified:
- No request size limits (can cause OOM)
- Unvalidated user input (JSON injection risks)
- Missing security headers
- No input sanitization

## Scope
- **Files to modify**: Transport layer, interceptors, proxy modules
- **Priority**: HIGH - Security vulnerabilities
- **Time estimate**: 1.5 days

## Current Security Issues

### High Risk Issues
1. **No Request Size Limits** - OOM attacks possible
2. **Unvalidated User Input** - JSON injection, path traversal
3. **Missing Rate Limiting** - DOS vulnerability (covered in Task 007)

### Medium Risk Issues
4. **No Circuit Breaker** - Cascading failures (covered in Task 015)
5. **Missing Audit Logging** - No security event tracking (covered in Task 016)
6. **Insufficient Input Validation** - Malformed data can crash handlers

## Implementation Plan

### Step 1: Add Input Validation Layer

```rust
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Payload too large: {0} bytes exceeds maximum {1}")]
    PayloadTooLarge(usize, usize),
    
    #[error("Invalid JSON structure: {0}")]
    InvalidJson(String),
    
    #[error("Forbidden characters in input: {0}")]
    ForbiddenCharacters(String),
    
    #[error("Path traversal attempt detected: {0}")]
    PathTraversal(String),
    
    #[error("Invalid method name: {0}")]
    InvalidMethod(String),
    
    #[error("Nested depth exceeds maximum: {0}")]
    ExcessiveNesting(usize),
}

pub struct InputValidator {
    max_payload_size: usize,
    max_string_length: usize,
    max_array_length: usize,
    max_nesting_depth: usize,
    forbidden_patterns: Vec<regex::Regex>,
}

impl Default for InputValidator {
    fn default() -> Self {
        Self {
            max_payload_size: 10 * 1024 * 1024,  // 10MB
            max_string_length: 1024 * 1024,      // 1MB per string
            max_array_length: 10000,             // Max array items
            max_nesting_depth: 100,              // Max JSON depth
            forbidden_patterns: vec![
                regex::Regex::new(r"\.\./").unwrap(),  // Path traversal
                regex::Regex::new(r"<script").unwrap(), // XSS attempt
                regex::Regex::new(r"';--").unwrap(),    // SQL injection
            ],
        }
    }
}

impl InputValidator {
    pub fn validate_json(&self, value: &Value, depth: usize) -> Result<(), ValidationError> {
        if depth > self.max_nesting_depth {
            return Err(ValidationError::ExcessiveNesting(depth));
        }
        
        match value {
            Value::String(s) => self.validate_string(s)?,
            Value::Array(arr) => {
                if arr.len() > self.max_array_length {
                    return Err(ValidationError::InvalidJson(
                        format!("Array too large: {} items", arr.len())
                    ));
                }
                for item in arr {
                    self.validate_json(item, depth + 1)?;
                }
            }
            Value::Object(obj) => {
                for (key, val) in obj {
                    self.validate_string(key)?;
                    self.validate_json(val, depth + 1)?;
                }
            }
            _ => {}  // Numbers, bools, null are safe
        }
        
        Ok(())
    }
    
    fn validate_string(&self, s: &str) -> Result<(), ValidationError> {
        // Check length
        if s.len() > self.max_string_length {
            return Err(ValidationError::InvalidJson(
                format!("String too long: {} bytes", s.len())
            ));
        }
        
        // Check for forbidden patterns
        for pattern in &self.forbidden_patterns {
            if pattern.is_match(s) {
                return Err(ValidationError::ForbiddenCharacters(
                    pattern.as_str().to_string()
                ));
            }
        }
        
        // Check for path traversal
        if s.contains("../") || s.contains("..\\") {
            return Err(ValidationError::PathTraversal(s.to_string()));
        }
        
        Ok(())
    }
}
```

### Step 2: Validate MCP Messages

```rust
pub struct McpValidator {
    input_validator: InputValidator,
    allowed_methods: HashSet<String>,
}

impl McpValidator {
    pub fn new() -> Self {
        let mut allowed_methods = HashSet::new();
        // Add all valid MCP methods
        allowed_methods.insert("initialize".to_string());
        allowed_methods.insert("initialized".to_string());
        allowed_methods.insert("ping".to_string());
        allowed_methods.insert("pong".to_string());
        allowed_methods.insert("tools/list".to_string());
        allowed_methods.insert("tools/call".to_string());
        allowed_methods.insert("resources/list".to_string());
        allowed_methods.insert("resources/read".to_string());
        allowed_methods.insert("prompts/list".to_string());
        allowed_methods.insert("prompts/get".to_string());
        allowed_methods.insert("shutdown".to_string());
        
        Self {
            input_validator: InputValidator::default(),
            allowed_methods,
        }
    }
    
    pub fn validate_request(&self, msg: &Value) -> Result<(), ValidationError> {
        // Basic structure validation
        self.input_validator.validate_json(msg, 0)?;
        
        // Validate JSONRPC version
        if msg.get("jsonrpc") != Some(&Value::String("2.0".to_string())) {
            return Err(ValidationError::InvalidJson(
                "Invalid or missing jsonrpc version".to_string()
            ));
        }
        
        // Validate method if present
        if let Some(method) = msg.get("method").and_then(|m| m.as_str()) {
            if !self.allowed_methods.contains(method) {
                // Check if it matches pattern (e.g., tools/*)
                let is_valid = method.starts_with("tools/") 
                    || method.starts_with("resources/")
                    || method.starts_with("prompts/");
                    
                if !is_valid {
                    return Err(ValidationError::InvalidMethod(method.to_string()));
                }
            }
        }
        
        // Validate ID if present
        if let Some(id) = msg.get("id") {
            match id {
                Value::Number(_) | Value::String(_) => {},
                _ => return Err(ValidationError::InvalidJson(
                    "Invalid id type".to_string()
                )),
            }
        }
        
        Ok(())
    }
}
```

### Step 3: Add Security Headers

```rust
use axum::http::HeaderMap;

pub struct SecurityHeaders;

impl SecurityHeaders {
    pub fn apply(headers: &mut HeaderMap) {
        headers.insert(
            "X-Content-Type-Options",
            "nosniff".parse().unwrap()
        );
        headers.insert(
            "X-Frame-Options",
            "DENY".parse().unwrap()
        );
        headers.insert(
            "X-XSS-Protection",
            "1; mode=block".parse().unwrap()
        );
        headers.insert(
            "Content-Security-Policy",
            "default-src 'none'".parse().unwrap()
        );
        headers.insert(
            "Strict-Transport-Security",
            "max-age=31536000; includeSubDomains".parse().unwrap()
        );
        headers.insert(
            "Referrer-Policy",
            "no-referrer".parse().unwrap()
        );
    }
}

// Apply in HTTP transport
impl HttpTransport {
    async fn handle_request(&self, req: Request<Body>) -> Response<Body> {
        let mut response = self.process_request(req).await;
        SecurityHeaders::apply(response.headers_mut());
        response
    }
}
```

### Step 4: Sanitize Error Messages

```rust
pub struct ErrorSanitizer;

impl ErrorSanitizer {
    /// Remove sensitive information from error messages
    pub fn sanitize(error: &str) -> String {
        let mut sanitized = error.to_string();
        
        // Remove file paths
        let path_regex = regex::Regex::new(r"(/[^/\s]+)+/[\w\.-]+").unwrap();
        sanitized = path_regex.replace_all(&sanitized, "[PATH]").to_string();
        
        // Remove IP addresses
        let ip_regex = regex::Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap();
        sanitized = ip_regex.replace_all(&sanitized, "[IP]").to_string();
        
        // Remove potential secrets (common patterns)
        let secret_patterns = [
            (r"token[=:]\s*[\w-]+", "[TOKEN]"),
            (r"api[_-]?key[=:]\s*[\w-]+", "[API_KEY]"),
            (r"password[=:]\s*[\w-]+", "[PASSWORD]"),
            (r"secret[=:]\s*[\w-]+", "[SECRET]"),
        ];
        
        for (pattern, replacement) in &secret_patterns {
            let regex = regex::Regex::new(pattern).unwrap();
            sanitized = regex.replace_all(&sanitized, *replacement).to_string();
        }
        
        sanitized
    }
}

// Use in error responses
impl From<Error> for Response {
    fn from(error: Error) -> Self {
        let sanitized = ErrorSanitizer::sanitize(&error.to_string());
        
        // Log full error internally
        tracing::error!("Error occurred: {}", error);
        
        // Return sanitized error to client
        Response::error(sanitized)
    }
}
```

### Step 5: Add Resource Limits

```rust
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_connections: usize,
    pub max_requests_per_second: u32,
    pub max_memory_mb: usize,
    pub max_cpu_percent: f32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            max_requests_per_second: 100,
            max_memory_mb: 500,
            max_cpu_percent: 80.0,
        }
    }
}

pub struct ResourceMonitor {
    limits: ResourceLimits,
    current_connections: AtomicUsize,
}

impl ResourceMonitor {
    pub fn check_limits(&self) -> Result<(), ValidationError> {
        // Check connection limit
        let connections = self.current_connections.load(Ordering::Relaxed);
        if connections >= self.limits.max_connections {
            return Err(ValidationError::InvalidJson(
                format!("Too many connections: {}", connections)
            ));
        }
        
        // Check memory usage
        if let Ok(memory) = self.get_memory_usage() {
            if memory > self.limits.max_memory_mb {
                return Err(ValidationError::InvalidJson(
                    format!("Memory limit exceeded: {}MB", memory)
                ));
            }
        }
        
        Ok(())
    }
    
    fn get_memory_usage(&self) -> Result<usize, Error> {
        // Platform-specific memory check
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            let status = fs::read_to_string("/proc/self/status")?;
            // Parse VmRSS from status
            // ...
        }
        
        Ok(0)  // Placeholder
    }
}
```

### Step 6: Implement Content Security

```rust
pub struct ContentValidator {
    max_file_size: usize,
    allowed_mime_types: HashSet<String>,
}

impl ContentValidator {
    pub fn validate_file_upload(&self, 
        data: &[u8], 
        mime_type: &str
    ) -> Result<(), ValidationError> {
        // Check size
        if data.len() > self.max_file_size {
            return Err(ValidationError::PayloadTooLarge(
                data.len(), 
                self.max_file_size
            ));
        }
        
        // Check MIME type
        if !self.allowed_mime_types.contains(mime_type) {
            return Err(ValidationError::InvalidJson(
                format!("Forbidden MIME type: {}", mime_type)
            ));
        }
        
        // Verify actual content matches MIME type
        let detected = tree_magic::from_u8(data);
        if detected != mime_type {
            return Err(ValidationError::InvalidJson(
                format!("MIME type mismatch: claimed {}, detected {}", 
                    mime_type, detected)
            ));
        }
        
        Ok(())
    }
}
```

## Testing Strategy

### Security Tests

```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[test]
    fn test_path_traversal_detection() {
        let validator = InputValidator::default();
        
        let malicious = vec![
            "../../../etc/passwd",
            "..\\..\\windows\\system32",
            "file:///../etc/passwd",
        ];
        
        for path in malicious {
            assert!(validator.validate_string(path).is_err());
        }
    }
    
    #[test]
    fn test_xss_detection() {
        let validator = InputValidator::default();
        
        let xss_attempts = vec![
            "<script>alert('xss')</script>",
            "<img src=x onerror=alert(1)>",
            "javascript:alert(1)",
        ];
        
        for attempt in xss_attempts {
            assert!(validator.validate_string(attempt).is_err());
        }
    }
    
    #[test]
    fn test_excessive_nesting() {
        let validator = InputValidator::default();
        
        // Create deeply nested JSON
        let mut json = json!({});
        let mut current = &mut json;
        for _ in 0..200 {
            *current = json!({"nested": {}});
            current = current.get_mut("nested").unwrap();
        }
        
        assert!(validator.validate_json(&json, 0).is_err());
    }
    
    #[test]
    fn test_error_sanitization() {
        let error = "Failed to connect to /home/user/secret/database.db with token=abc123";
        let sanitized = ErrorSanitizer::sanitize(error);
        
        assert!(!sanitized.contains("/home/user"));
        assert!(!sanitized.contains("abc123"));
        assert!(sanitized.contains("[PATH]"));
        assert!(sanitized.contains("[TOKEN]"));
    }
}
```

### Fuzzing Tests

```rust
#[cfg(test)]
mod fuzz_tests {
    use quickcheck::quickcheck;
    
    quickcheck! {
        fn prop_validator_doesnt_panic(input: String) -> bool {
            let validator = InputValidator::default();
            let _ = validator.validate_string(&input);
            true  // Didn't panic
        }
        
        fn prop_json_validator_doesnt_panic(input: Vec<u8>) -> bool {
            if let Ok(json) = serde_json::from_slice::<Value>(&input) {
                let validator = InputValidator::default();
                let _ = validator.validate_json(&json, 0);
            }
            true  // Didn't panic
        }
    }
}
```

## Validation

### Pre-check
```bash
# Security scan
cargo audit

# Check for validation
rg "validate|sanitize" --type rust | wc -l
```

### Post-check
```bash
# No security vulnerabilities
cargo audit

# Validation everywhere
rg "validate_" --type rust | wc -l  # Should be significantly higher

# Test with malicious inputs
./test-security.sh  # Custom security test script
```

## Success Criteria

- [ ] All user input validated before processing
- [ ] Request size limits enforced
- [ ] Security headers added to HTTP responses
- [ ] Error messages sanitized
- [ ] Resource limits implemented
- [ ] Path traversal attempts blocked
- [ ] XSS/injection attempts detected
- [ ] All security tests pass
- [ ] No new vulnerabilities in cargo audit

## Security Checklist

### Input Validation
- [x] JSON structure validation
- [x] String length limits
- [x] Array size limits
- [x] Nesting depth limits
- [x] Character filtering
- [x] Path traversal prevention

### Headers & Transport
- [x] Security headers (CSP, HSTS, etc.)
- [x] CORS properly configured
- [x] TLS enforcement (production)

### Error Handling
- [x] Error message sanitization
- [x] No stack traces in production
- [x] No sensitive data in logs

### Resource Protection
- [x] Request size limits
- [x] Connection limits
- [x] Memory limits
- [x] CPU limits
- [x] Rate limiting (Task 007)

## Integration Points

- Coordinate with rate limiting (Task 007)
- Works with audit logging (Task 016)
- May affect performance (monitor overhead)
- Update documentation for security features

## Notes

- Balance security with usability
- Don't break legitimate use cases
- Consider adding security configuration options
- Document all security measures for users
- Consider OWASP guidelines for web security