# Task 015: Implement Circuit Breaker

## Overview
Implement a circuit breaker pattern to prevent cascading failures and improve system resilience when upstream services fail.

## Context
From the [comprehensive review](../../reviews/shadowcat-comprehensive-review-2025-08-06.md), "No Circuit Breaker" is listed as a medium risk issue. There's a TODO at `src/proxy/reverse.rs:301` indicating this critical resilience feature is missing.

## Scope
- **Files to modify**: `src/proxy/reverse.rs`, possibly new `src/circuit_breaker/mod.rs`
- **Priority**: MEDIUM - Reliability concern
- **Time estimate**: 1 day

## Current Problem

### Missing Implementation
**Location**: `src/proxy/reverse.rs:301`
```rust
// TODO: Implement circuit breaker
```

### Risks Without Circuit Breaker
- Cascading failures when upstream is down
- Resource exhaustion from retrying failed requests
- Poor user experience with long timeouts
- No automatic recovery detection

## Implementation Plan

### Step 1: Define Circuit Breaker States

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Circuit is functioning normally
    Closed,
    /// Circuit is failing, rejecting requests
    Open { 
        opened_at: Instant,
        retry_after: Instant,
    },
    /// Testing if service has recovered
    HalfOpen {
        started_at: Instant,
        successes: u32,
    },
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Success rate below this opens circuit (0.0 - 1.0)
    pub failure_rate_threshold: f32,
    /// Time window for counting failures
    pub window_duration: Duration,
    /// How long to wait before trying half-open
    pub timeout_duration: Duration,
    /// Successes needed in half-open to close circuit
    pub success_threshold: u32,
    /// Minimum requests in window before evaluating
    pub minimum_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            failure_rate_threshold: 0.5,
            window_duration: Duration::from_secs(60),
            timeout_duration: Duration::from_secs(30),
            success_threshold: 3,
            minimum_requests: 10,
        }
    }
}
```

### Step 2: Implement Circuit Breaker Core

```rust
pub struct CircuitBreaker {
    name: String,
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    
    // Metrics for current window
    total_requests: AtomicU32,
    failed_requests: AtomicU32,
    window_start: Arc<RwLock<Instant>>,
    
    // Consecutive failures for fast-fail
    consecutive_failures: AtomicU32,
    
    // Event callbacks
    on_state_change: Option<Arc<dyn Fn(CircuitState) + Send + Sync>>,
}

impl CircuitBreaker {
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        Self {
            name,
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            total_requests: AtomicU32::new(0),
            failed_requests: AtomicU32::new(0),
            window_start: Arc::new(RwLock::new(Instant::now())),
            consecutive_failures: AtomicU32::new(0),
            on_state_change: None,
        }
    }
    
    /// Check if request should be allowed
    pub async fn should_allow(&self) -> Result<(), CircuitBreakerError> {
        self.maybe_reset_window().await;
        
        let state = self.state.read().await.clone();
        
        match state {
            CircuitState::Closed => {
                self.total_requests.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            CircuitState::Open { retry_after, .. } => {
                if Instant::now() >= retry_after {
                    // Transition to half-open
                    self.transition_to_half_open().await;
                    self.total_requests.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                } else {
                    Err(CircuitBreakerError::Open {
                        retry_after: retry_after.duration_since(Instant::now()),
                    })
                }
            }
            CircuitState::HalfOpen { .. } => {
                // Allow limited requests in half-open
                self.total_requests.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
        }
    }
    
    /// Record successful request
    pub async fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
        
        let state = self.state.read().await.clone();
        
        if let CircuitState::HalfOpen { successes, started_at } = state {
            let new_successes = successes + 1;
            
            if new_successes >= self.config.success_threshold {
                // Close the circuit
                self.transition_to_closed().await;
            } else {
                // Update success count
                *self.state.write().await = CircuitState::HalfOpen {
                    started_at,
                    successes: new_successes,
                };
            }
        }
    }
    
    /// Record failed request
    pub async fn record_failure(&self) {
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
        let consecutive = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        
        let state = self.state.read().await.clone();
        
        match state {
            CircuitState::Closed => {
                // Check if we should open
                if self.should_open(consecutive).await {
                    self.transition_to_open().await;
                }
            }
            CircuitState::HalfOpen { .. } => {
                // Single failure in half-open reopens circuit
                self.transition_to_open().await;
            }
            CircuitState::Open { .. } => {
                // Already open, nothing to do
            }
        }
    }
    
    async fn should_open(&self, consecutive_failures: u32) -> bool {
        // Fast-fail on consecutive failures
        if consecutive_failures >= self.config.failure_threshold {
            return true;
        }
        
        // Check failure rate
        let total = self.total_requests.load(Ordering::Relaxed);
        if total < self.config.minimum_requests {
            return false;
        }
        
        let failed = self.failed_requests.load(Ordering::Relaxed);
        let failure_rate = failed as f32 / total as f32;
        
        failure_rate >= self.config.failure_rate_threshold
    }
    
    async fn transition_to_open(&self) {
        let now = Instant::now();
        let new_state = CircuitState::Open {
            opened_at: now,
            retry_after: now + self.config.timeout_duration,
        };
        
        *self.state.write().await = new_state.clone();
        
        tracing::warn!(
            circuit = %self.name,
            "Circuit breaker opened, will retry after {:?}",
            self.config.timeout_duration
        );
        
        if let Some(callback) = &self.on_state_change {
            callback(new_state);
        }
    }
    
    async fn transition_to_half_open(&self) {
        let new_state = CircuitState::HalfOpen {
            started_at: Instant::now(),
            successes: 0,
        };
        
        *self.state.write().await = new_state.clone();
        
        tracing::info!(
            circuit = %self.name,
            "Circuit breaker half-open, testing recovery"
        );
        
        if let Some(callback) = &self.on_state_change {
            callback(new_state);
        }
    }
    
    async fn transition_to_closed(&self) {
        *self.state.write().await = CircuitState::Closed;
        
        // Reset metrics
        self.total_requests.store(0, Ordering::Relaxed);
        self.failed_requests.store(0, Ordering::Relaxed);
        self.consecutive_failures.store(0, Ordering::Relaxed);
        *self.window_start.write().await = Instant::now();
        
        tracing::info!(
            circuit = %self.name,
            "Circuit breaker closed, service recovered"
        );
        
        if let Some(callback) = &self.on_state_change {
            callback(CircuitState::Closed);
        }
    }
    
    async fn maybe_reset_window(&self) {
        let now = Instant::now();
        let window_start = *self.window_start.read().await;
        
        if now.duration_since(window_start) > self.config.window_duration {
            // Reset window
            *self.window_start.write().await = now;
            self.total_requests.store(0, Ordering::Relaxed);
            self.failed_requests.store(0, Ordering::Relaxed);
        }
    }
}
```

### Step 3: Create Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CircuitBreakerError {
    #[error("Circuit breaker is open, retry after {retry_after:?}")]
    Open { retry_after: Duration },
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Request timeout")]
    Timeout,
}
```

### Step 4: Integrate with Reverse Proxy

```rust
// In src/proxy/reverse.rs

use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError};

pub struct ReverseProxy {
    // ... existing fields ...
    circuit_breakers: Arc<DashMap<String, Arc<CircuitBreaker>>>,
}

impl ReverseProxy {
    pub fn new(config: ReverseProxyConfig) -> Self {
        let circuit_breakers = Arc::new(DashMap::new());
        
        // Create circuit breakers for each upstream
        for upstream in &config.upstreams {
            let cb_config = CircuitBreakerConfig {
                failure_threshold: config.circuit_breaker_threshold.unwrap_or(5),
                timeout_duration: config.circuit_breaker_timeout.unwrap_or(Duration::from_secs(30)),
                ..Default::default()
            };
            
            let cb = Arc::new(CircuitBreaker::new(
                upstream.url.to_string(),
                cb_config,
            ));
            
            circuit_breakers.insert(upstream.url.to_string(), cb);
        }
        
        Self {
            // ... existing fields ...
            circuit_breakers,
        }
    }
    
    pub async fn forward_request(
        &self,
        upstream_url: &str,
        request: Request,
    ) -> Result<Response, ProxyError> {
        // Get or create circuit breaker for this upstream
        let circuit_breaker = self.get_or_create_circuit_breaker(upstream_url);
        
        // Check if request is allowed
        circuit_breaker.should_allow().await
            .map_err(|e| match e {
                CircuitBreakerError::Open { retry_after } => {
                    ProxyError::CircuitOpen {
                        upstream: upstream_url.to_string(),
                        retry_after,
                    }
                }
                _ => ProxyError::from(e),
            })?;
        
        // Attempt the request with timeout
        let result = tokio::time::timeout(
            self.config.request_timeout,
            self.send_request(upstream_url, request)
        ).await;
        
        match result {
            Ok(Ok(response)) => {
                // Record success if response is successful
                if response.status().is_success() {
                    circuit_breaker.record_success().await;
                } else if response.status().is_server_error() {
                    // 5xx errors count as failures
                    circuit_breaker.record_failure().await;
                }
                Ok(response)
            }
            Ok(Err(e)) => {
                // Request failed
                circuit_breaker.record_failure().await;
                Err(e)
            }
            Err(_) => {
                // Timeout
                circuit_breaker.record_failure().await;
                Err(ProxyError::Timeout)
            }
        }
    }
    
    fn get_or_create_circuit_breaker(&self, upstream: &str) -> Arc<CircuitBreaker> {
        self.circuit_breakers
            .entry(upstream.to_string())
            .or_insert_with(|| {
                Arc::new(CircuitBreaker::new(
                    upstream.to_string(),
                    self.config.circuit_breaker_config.clone(),
                ))
            })
            .clone()
    }
}
```

### Step 5: Add Monitoring and Metrics

```rust
#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    pub state: CircuitState,
    pub total_requests: u32,
    pub failed_requests: u32,
    pub success_rate: f32,
    pub consecutive_failures: u32,
}

impl CircuitBreaker {
    pub async fn get_metrics(&self) -> CircuitBreakerMetrics {
        let total = self.total_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);
        
        CircuitBreakerMetrics {
            state: self.state.read().await.clone(),
            total_requests: total,
            failed_requests: failed,
            success_rate: if total > 0 {
                (total - failed) as f32 / total as f32
            } else {
                1.0
            },
            consecutive_failures: self.consecutive_failures.load(Ordering::Relaxed),
        }
    }
}

// Health check endpoint
pub async fn health_check(State(proxy): State<Arc<ReverseProxy>>) -> impl IntoResponse {
    let mut health = json!({
        "status": "healthy",
        "circuit_breakers": {}
    });
    
    for entry in proxy.circuit_breakers.iter() {
        let metrics = entry.value().get_metrics().await;
        health["circuit_breakers"][entry.key()] = json!({
            "state": format!("{:?}", metrics.state),
            "success_rate": metrics.success_rate,
            "total_requests": metrics.total_requests,
        });
    }
    
    Json(health)
}
```

### Step 6: Add Fallback Mechanism

```rust
pub trait FallbackHandler: Send + Sync {
    async fn handle_fallback(&self, request: &Request) -> Option<Response>;
}

pub struct DefaultFallback;

impl FallbackHandler for DefaultFallback {
    async fn handle_fallback(&self, _request: &Request) -> Option<Response> {
        Some(Response::builder()
            .status(503)
            .header("Content-Type", "application/json")
            .body(json!({
                "error": "Service temporarily unavailable",
                "retry_after": 30
            }).to_string())
            .unwrap())
    }
}

impl ReverseProxy {
    pub async fn forward_with_fallback(
        &self,
        upstream: &str,
        request: Request,
    ) -> Result<Response, ProxyError> {
        match self.forward_request(upstream, request.clone()).await {
            Ok(response) => Ok(response),
            Err(ProxyError::CircuitOpen { .. }) => {
                // Try fallback
                if let Some(fallback) = &self.fallback_handler {
                    if let Some(response) = fallback.handle_fallback(&request).await {
                        return Ok(response);
                    }
                }
                Err(ProxyError::CircuitOpen { .. })
            }
            Err(e) => Err(e),
        }
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
    async fn test_circuit_opens_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        // Should be closed initially
        assert!(cb.should_allow().await.is_ok());
        
        // Record failures
        for _ in 0..3 {
            cb.record_failure().await;
        }
        
        // Should be open now
        assert!(matches!(
            cb.should_allow().await,
            Err(CircuitBreakerError::Open { .. })
        ));
    }
    
    #[tokio::test]
    async fn test_circuit_closes_after_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout_duration: Duration::from_millis(100),
            ..Default::default()
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        // Open the circuit
        cb.record_failure().await;
        cb.record_failure().await;
        assert!(cb.should_allow().await.is_err());
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Should be half-open
        assert!(cb.should_allow().await.is_ok());
        
        // Record successes
        cb.record_success().await;
        cb.record_success().await;
        
        // Should be closed
        assert!(cb.should_allow().await.is_ok());
        assert_eq!(*cb.state.read().await, CircuitState::Closed);
    }
    
    #[tokio::test]
    async fn test_half_open_reopens_on_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            timeout_duration: Duration::from_millis(50),
            ..Default::default()
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        // Open circuit
        cb.record_failure().await;
        
        // Wait for half-open
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(cb.should_allow().await.is_ok());
        
        // Fail in half-open
        cb.record_failure().await;
        
        // Should be open again
        assert!(cb.should_allow().await.is_err());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_proxy_with_circuit_breaker() {
    // Start mock failing upstream
    let failing_server = MockServer::start().await;
    failing_server.mock(|when, then| {
        when.any_request();
        then.status(500);
    });
    
    let config = ReverseProxyConfig {
        upstreams: vec![failing_server.uri()],
        circuit_breaker_threshold: Some(2),
        ..Default::default()
    };
    
    let proxy = ReverseProxy::new(config);
    
    // First failures should go through
    let _ = proxy.forward_request(&failing_server.uri(), test_request()).await;
    let _ = proxy.forward_request(&failing_server.uri(), test_request()).await;
    
    // Circuit should be open now
    let result = proxy.forward_request(&failing_server.uri(), test_request()).await;
    assert!(matches!(result, Err(ProxyError::CircuitOpen { .. })));
}
```

## Validation

### Pre-check
```bash
# Find TODO
rg "TODO.*circuit breaker" --type rust

# Check current error rates
# (Would need monitoring in place)
```

### Post-check
```bash
# TODO removed
rg "TODO.*circuit breaker" --type rust | wc -l  # Should be 0

# Run tests
cargo test circuit_breaker

# Load test with failing upstream
./test-circuit-breaker.sh
```

## Success Criteria

- [ ] Circuit breaker implementation complete
- [ ] Opens on consecutive failures or high failure rate
- [ ] Transitions through Closed -> Open -> Half-Open -> Closed
- [ ] Prevents cascade failures
- [ ] Automatic recovery detection
- [ ] Metrics and monitoring available
- [ ] Fallback mechanism works
- [ ] TODO comment removed
- [ ] All tests pass
- [ ] No performance degradation for successful requests

## Configuration Example

```toml
[proxy.circuit_breaker]
failure_threshold = 5
failure_rate_threshold = 0.5
window_duration_secs = 60
timeout_duration_secs = 30
success_threshold = 3
minimum_requests = 10
```

## Performance Considerations

1. **Lock-free design** - Use atomics where possible
2. **Async-friendly** - No blocking operations
3. **Memory efficient** - Bounded metrics storage
4. **Fast decisions** - O(1) allow/deny checks

## Integration Points

- Coordinate with metrics collection (Task 018)
- Works with audit logging (Task 016)
- May affect rate limiting behavior
- Consider health check endpoints

## Notes

- Essential for production resilience
- Consider per-endpoint circuit breakers
- May want different configs for different upstreams
- Monitor and tune thresholds based on real traffic