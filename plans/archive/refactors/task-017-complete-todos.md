# Task 017: Complete All TODOs

## Overview
Systematically address and complete all 18 TODO comments identified in the codebase, with 6 being critical blockers for production deployment.

## Context
From the [comprehensive review](../../reviews/shadowcat-comprehensive-review-2025-08-06.md), 18 TODO comments indicate incomplete implementations throughout the codebase. These represent technical debt and potential production issues.

## Scope
- **Files to modify**: Multiple files across the codebase
- **Priority**: HIGH - Multiple critical TODOs blocking production
- **Time estimate**: 2 days

## TODO Inventory

### Critical TODOs (Production Blockers)

#### 1. Rate Limiting Implementation
**Location**: `src/interceptor/rules.rs:174`
```rust
// TODO: Implement actual rate limiting
RateLimitAction::Allow
```
**Status**: Covered in Task 007
**Action**: Defer to Task 007

#### 2. Session Matching Logic
**Location**: `src/session/manager.rs:108`
```rust
// TODO: Implement proper session matching logic
```
**Status**: Covered in Task 008
**Action**: Defer to Task 008

#### 3. Error Tracking
**Location**: `src/proxy/reverse.rs:222`
```rust
// TODO: Track errors for circuit breaker
```
**Implementation**:
```rust
// Replace TODO with:
self.error_tracker.record_error(upstream_url, error.clone());

// Add error tracker:
pub struct ErrorTracker {
    errors: Arc<DashMap<String, Vec<ErrorRecord>>>,
    window: Duration,
}

impl ErrorTracker {
    pub fn record_error(&self, upstream: &str, error: ProxyError) {
        let record = ErrorRecord {
            timestamp: Instant::now(),
            error_type: error.to_string(),
        };
        
        self.errors.entry(upstream.to_string())
            .or_insert_with(Vec::new)
            .push(record);
        
        // Clean old errors
        self.cleanup_old_errors(upstream);
    }
    
    pub fn get_error_rate(&self, upstream: &str) -> f32 {
        // Calculate error rate for circuit breaker
    }
}
```

#### 4. Token Refresh Logic
**Location**: `src/auth/oauth.rs:156`
```rust
// TODO: Implement token refresh
```
**Implementation**:
```rust
pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse, AuthError> {
    let client = &self.oauth_client;
    
    let token_response = client
        .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
        .request_async(async_http_client)
        .await
        .map_err(|e| AuthError::TokenRefreshFailed(e.to_string()))?;
    
    // Cache new tokens
    self.token_cache.insert(
        token_response.access_token().secret().to_string(),
        CachedToken {
            token: token_response.clone(),
            expires_at: Instant::now() + Duration::from_secs(
                token_response.expires_in()
                    .map(|d| d.as_secs())
                    .unwrap_or(3600)
            ),
        }
    );
    
    Ok(token_response)
}
```

#### 5. Cleanup Tasks
**Location**: `src/session/manager.rs:195`
```rust
// TODO: Implement cleanup tasks
```
**Status**: Covered in Task 009
**Action**: Defer to Task 009

#### 6. Retry Logic
**Location**: `src/transport/http.rs:234`
```rust
// TODO: Add retry logic with exponential backoff
```
**Implementation**:
```rust
use backoff::{ExponentialBackoff, future::retry};

pub async fn send_with_retry(&self, request: Request) -> Result<Response, TransportError> {
    let backoff = ExponentialBackoff {
        initial_interval: Duration::from_millis(100),
        max_interval: Duration::from_secs(10),
        max_elapsed_time: Some(Duration::from_secs(30)),
        ..Default::default()
    };
    
    retry(backoff, || async {
        match self.send_internal(request.clone()).await {
            Ok(response) => Ok(response),
            Err(e) if e.is_retryable() => {
                tracing::warn!("Request failed, retrying: {}", e);
                Err(backoff::Error::Transient(e))
            }
            Err(e) => Err(backoff::Error::Permanent(e)),
        }
    })
    .await
    .map_err(|e| match e {
        backoff::Error::Permanent(e) => e,
        backoff::Error::Transient(e) => e,
    })
}
```

### Medium Priority TODOs

#### 7. Metrics Aggregation
**Location**: `src/metrics/mod.rs:89`
```rust
// TODO: Implement metrics aggregation
```
**Status**: Covered in Task 018
**Action**: Defer to Task 018

#### 8. Connection Pooling
**Location**: `src/transport/http.rs:45`
```rust
// TODO: Implement connection pooling
```
**Implementation**:
```rust
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;

pub struct ConnectionPool {
    client: Client<HttpsConnector<HttpConnector>>,
    max_idle_per_host: usize,
}

impl ConnectionPool {
    pub fn new(max_idle_per_host: usize) -> Self {
        let https = HttpsConnector::new();
        let client = Client::builder()
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(max_idle_per_host)
            .build::<_, hyper::Body>(https);
        
        Self {
            client,
            max_idle_per_host,
        }
    }
    
    pub async fn request(&self, req: Request<Body>) -> Result<Response<Body>, Error> {
        self.client.request(req).await
    }
}
```

#### 9. Circuit Breaker
**Location**: `src/proxy/reverse.rs:301`
```rust
// TODO: Implement circuit breaker
```
**Status**: Covered in Task 015
**Action**: Defer to Task 015

#### 10. Request Validation
**Location**: `src/interceptor/validator.rs:67`
```rust
// TODO: Add request validation
```
**Status**: Covered in Task 014
**Action**: Defer to Task 014

### Low Priority TODOs

#### 11. Performance Optimization
**Location**: `src/transport/stdio.rs:189`
```rust
// TODO: Optimize buffer allocation
```
**Implementation**:
```rust
// Use buffer pool
thread_local! {
    static BUFFER_POOL: RefCell<Vec<BytesMut>> = RefCell::new(Vec::new());
}

fn get_buffer() -> BytesMut {
    BUFFER_POOL.with(|pool| {
        pool.borrow_mut().pop()
            .unwrap_or_else(|| BytesMut::with_capacity(8192))
    })
}

fn return_buffer(mut buf: BytesMut) {
    buf.clear();
    BUFFER_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        if pool.len() < 10 {  // Keep max 10 buffers
            pool.push(buf);
        }
    })
}
```

#### 12. Documentation
**Location**: `src/recorder/tape.rs:45`
```rust
// TODO: Add documentation for tape format
```
**Implementation**:
```rust
/// Tape format specification for MCP session recording
/// 
/// # Format
/// 
/// The tape format is a JSON structure containing:
/// - `version`: Format version (currently "1.0")
/// - `metadata`: Session metadata including timestamps, transport type
/// - `frames`: Array of recorded frames with timing information
/// - `summary`: Statistical summary of the session
/// 
/// # Example
/// 
/// ```json
/// {
///   "version": "1.0",
///   "metadata": {
///     "session_id": "uuid",
///     "started_at": "2025-01-01T00:00:00Z",
///     "transport": "stdio"
///   },
///   "frames": [
///     {
///       "timestamp": 0,
///       "direction": "client_to_server",
///       "data": {...}
///     }
///   ]
/// }
/// ```
pub struct TapeFormat {
    // ...
}
```

#### 13. Test Coverage
**Location**: `src/auth/jwt.rs:123`
```rust
// TODO: Add more test cases
```
**Implementation**: Add comprehensive test suite

#### 14. Error Handling Improvement
**Location**: `src/cli/mod.rs:234`
```rust
// TODO: Improve error messages
```
**Implementation**:
```rust
// Replace generic errors with context
fn format_error(error: &Error) -> String {
    match error {
        Error::ConfigNotFound(path) => {
            format!("Configuration file not found: {}\nCreate one with: shadowcat init", path)
        }
        Error::InvalidConfig(msg) => {
            format!("Invalid configuration: {}\nCheck the documentation for valid options", msg)
        }
        Error::ConnectionFailed(upstream) => {
            format!("Failed to connect to upstream: {}\nVerify the server is running and accessible", upstream)
        }
        _ => format!("An error occurred: {}", error),
    }
}
```

#### 15. Feature Flag
**Location**: `src/config.rs:89`
```rust
// TODO: Add feature flag for experimental features
```
**Implementation**:
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct FeatureFlags {
    #[serde(default)]
    pub enable_experimental: bool,
    
    #[serde(default)]
    pub enable_debug_endpoints: bool,
    
    #[serde(default)]
    pub enable_metrics_export: bool,
}

impl Config {
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "experimental" => self.features.enable_experimental,
            "debug" => self.features.enable_debug_endpoints,
            "metrics" => self.features.enable_metrics_export,
            _ => false,
        }
    }
}
```

#### 16. Logging Enhancement
**Location**: `src/proxy/forward.rs:456`
```rust
// TODO: Add structured logging
```
**Implementation**:
```rust
use tracing::{info, span, Level};

// Replace println! with structured logging
let span = span!(Level::INFO, "forward_request",
    session_id = %session_id,
    upstream = %upstream_url,
);

let _enter = span.enter();

info!(
    method = %request.method(),
    path = %request.uri().path(),
    "Forwarding request"
);
```

#### 17. Configuration Validation
**Location**: `src/config.rs:234`
```rust
// TODO: Validate configuration on load
```
**Implementation**:
```rust
impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Check required fields
        if self.upstreams.is_empty() {
            return Err(ConfigError::MissingUpstreams);
        }
        
        // Validate URLs
        for upstream in &self.upstreams {
            url::Url::parse(&upstream.url)
                .map_err(|_| ConfigError::InvalidUrl(upstream.url.clone()))?;
        }
        
        // Validate port ranges
        if let Some(port) = self.port {
            if port == 0 || port > 65535 {
                return Err(ConfigError::InvalidPort(port));
            }
        }
        
        // Validate timeouts
        if self.timeout_secs == 0 {
            return Err(ConfigError::InvalidTimeout);
        }
        
        Ok(())
    }
}
```

#### 18. Migration Support
**Location**: `src/recorder/storage.rs:78`
```rust
// TODO: Add migration support for tape format changes
```
**Implementation**:
```rust
pub trait TapeMigration {
    fn version(&self) -> &str;
    fn migrate(&self, tape: &mut TapeData) -> Result<(), MigrationError>;
}

pub struct MigrationManager {
    migrations: Vec<Box<dyn TapeMigration>>,
}

impl MigrationManager {
    pub fn migrate_tape(&self, tape: &mut TapeData) -> Result<(), MigrationError> {
        let current_version = tape.version.clone();
        
        for migration in &self.migrations {
            if migration.version() > current_version.as_str() {
                migration.migrate(tape)?;
                tape.version = migration.version().to_string();
            }
        }
        
        Ok(())
    }
}
```

## Testing Strategy

### Verification Script
```bash
#!/bin/bash
# verify-todos.sh

echo "Checking for remaining TODOs..."
TODO_COUNT=$(rg "TODO" --type rust | wc -l)

if [ "$TODO_COUNT" -eq 0 ]; then
    echo "✅ All TODOs completed!"
else
    echo "❌ Found $TODO_COUNT remaining TODOs:"
    rg "TODO" --type rust -n
    exit 1
fi

# Run tests for modified areas
cargo test auth::oauth::tests::test_token_refresh
cargo test transport::http::tests::test_retry_logic
cargo test proxy::reverse::tests::test_error_tracking
```

## Validation

### Pre-check
```bash
# Count TODOs
rg "TODO" --type rust | wc -l  # Should be 18

# List all TODOs with context
rg "TODO" --type rust -B 2 -A 2
```

### Post-check
```bash
# Should be 0
rg "TODO" --type rust | wc -l

# Run all tests
cargo test

# Check coverage
cargo tarpaulin --out Html
```

## Success Criteria

- [ ] All 18 TODO comments removed
- [ ] All critical TODOs implemented or deferred to specific tasks
- [ ] Token refresh logic working
- [ ] Error tracking implemented
- [ ] Retry logic with exponential backoff working
- [ ] Connection pooling implemented
- [ ] Documentation TODOs completed
- [ ] Configuration validation added
- [ ] All tests pass
- [ ] No new TODOs introduced

## Implementation Order

### Day 1 - Critical TODOs
1. Error tracking (30 min)
2. Token refresh (1 hour)
3. Retry logic (1 hour)
4. Defer others to respective tasks

### Day 2 - Remaining TODOs
1. Connection pooling (1 hour)
2. Buffer optimization (30 min)
3. Documentation (1 hour)
4. Error messages (30 min)
5. Feature flags (30 min)
6. Structured logging (30 min)
7. Config validation (30 min)
8. Migration support (1 hour)

## Dependencies

- Some TODOs are covered by other tasks:
  - Task 007: Rate limiting
  - Task 008: Session matching
  - Task 009: Session cleanup
  - Task 014: Request validation
  - Task 015: Circuit breaker
  - Task 018: Metrics aggregation

## Notes

- Focus on completing TODOs not covered by other tasks
- Add tests for each implemented TODO
- Update documentation as TODOs are resolved
- Consider adding a pre-commit hook to prevent new TODOs