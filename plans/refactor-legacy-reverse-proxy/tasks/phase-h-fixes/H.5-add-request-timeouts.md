# H.5: Add Request Timeouts

**Priority**: 🟡 HIGH  
**Duration**: 3 hours  
**Status**: ⏳ Pending  

## Problem

No timeout configuration exists for upstream requests, which can lead to:
- Requests hanging forever
- Resource exhaustion
- Poor user experience
- Cascading failures

## Solution

### Step 1: Add Timeout Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Maximum time to wait for connection establishment
    pub connect_timeout: Duration,
    /// Maximum time for complete request/response cycle
    pub request_timeout: Duration,
    /// Maximum idle time between data packets
    pub idle_timeout: Duration,
    /// Timeout for initial response headers
    pub first_byte_timeout: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(60),
            idle_timeout: Duration::from_secs(30),
            first_byte_timeout: Duration::from_secs(15),
        }
    }
}

// Add to ReverseUpstreamConfig
pub struct ReverseUpstreamConfig {
    // ... existing fields ...
    pub timeouts: TimeoutConfig,
}
```

### Step 2: Implement for HTTP Upstream

```rust
impl HttpUpstream {
    async fn send_request_with_timeout(
        &self,
        request: Request<Body>,
    ) -> Result<Response<Body>> {
        let timeout = self.config.timeouts.request_timeout;
        
        tokio::time::timeout(timeout, async {
            let client = self.create_client_with_timeouts()?;
            client.request(request).await
        })
        .await
        .map_err(|_| {
            ReverseProxyError::Timeout(format!(
                "Request timeout after {:?}",
                timeout
            ))
        })?
    }
    
    fn create_client_with_timeouts(&self) -> Result<Client<HttpConnector>> {
        let mut connector = HttpConnector::new();
        connector.set_connect_timeout(Some(self.config.timeouts.connect_timeout));
        
        let client = Client::builder()
            .pool_idle_timeout(self.config.timeouts.idle_timeout)
            .http2_initial_stream_window_size(65536)
            .build(connector);
        
        Ok(client)
    }
}
```

### Step 3: Implement for Stdio Upstream

```rust
impl StdioUpstream {
    async fn send_request_with_timeout(
        &self,
        request: Value,
    ) -> Result<Value> {
        let timeout = self.config.timeouts.request_timeout;
        
        tokio::time::timeout(timeout, async {
            let connection = self.pool.acquire().await?;
            connection.send_request(request).await
        })
        .await
        .map_err(|_| {
            ReverseProxyError::Timeout(format!(
                "Stdio request timeout after {:?}",
                timeout
            ))
        })?
    }
}
```

### Step 4: Add SSE-Specific Timeouts

```rust
impl SseStreamInitiator {
    async fn connect_with_timeout(&self) -> Result<SseStream> {
        // Initial connection timeout
        let connect_future = self.connect_to_upstream();
        
        tokio::time::timeout(
            self.config.timeouts.connect_timeout,
            connect_future
        )
        .await
        .map_err(|_| anyhow!("SSE connection timeout"))?
    }
    
    async fn read_event_with_timeout(
        &mut self,
    ) -> Result<Option<SseEvent>> {
        // Idle timeout between events
        tokio::time::timeout(
            self.config.timeouts.idle_timeout,
            self.stream.next()
        )
        .await
        .map_err(|_| anyhow!("SSE idle timeout"))?
    }
}
```

### Step 5: Add Circuit Breaker for Timeout Failures

```rust
pub struct TimeoutCircuitBreaker {
    consecutive_timeouts: AtomicU32,
    open_until: RwLock<Option<Instant>>,
    config: CircuitBreakerConfig,
}

impl TimeoutCircuitBreaker {
    pub async fn call<F, T>(&self, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        // Check if circuit is open
        if let Some(open_until) = *self.open_until.read().await {
            if Instant::now() < open_until {
                return Err(anyhow!("Circuit breaker open"));
            }
        }
        
        // Execute with timeout
        match f.await {
            Ok(result) => {
                // Reset on success
                self.consecutive_timeouts.store(0, Ordering::Relaxed);
                Ok(result)
            }
            Err(e) if is_timeout_error(&e) => {
                let count = self.consecutive_timeouts.fetch_add(1, Ordering::Relaxed) + 1;
                
                if count >= self.config.timeout_threshold {
                    // Open circuit
                    let open_duration = Duration::from_secs(30);
                    *self.open_until.write().await = Some(Instant::now() + open_duration);
                    
                    warn!("Circuit breaker opened after {} timeouts", count);
                }
                
                Err(e)
            }
            Err(e) => Err(e),
        }
    }
}
```

## Testing

### Unit Tests
```rust
#[tokio::test]
async fn test_request_timeout() {
    let upstream = create_slow_upstream(response_delay: Duration::from_secs(5));
    let config = TimeoutConfig {
        request_timeout: Duration::from_secs(1),
        ..Default::default()
    };
    
    let result = upstream.send_request_with_timeout(test_request()).await;
    
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ReverseProxyError::Timeout(_)
    ));
}

#[tokio::test]
async fn test_connect_timeout() {
    let upstream = HttpUpstream::new(
        "http://192.0.2.1:8080", // Non-routable IP
        TimeoutConfig {
            connect_timeout: Duration::from_millis(100),
            ..Default::default()
        }
    );
    
    let result = upstream.connect().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_circuit_breaker_opens() {
    let breaker = TimeoutCircuitBreaker::new(threshold: 3);
    
    // Simulate 3 timeouts
    for _ in 0..3 {
        let _ = breaker.call(timeout_future()).await;
    }
    
    // Circuit should be open
    let result = breaker.call(normal_future()).await;
    assert_eq!(result.unwrap_err().to_string(), "Circuit breaker open");
}
```

## Success Criteria

- [ ] All upstream types have timeout configuration
- [ ] Timeouts are configurable per upstream
- [ ] Circuit breaker prevents cascade failures
- [ ] Timeout errors are properly propagated
- [ ] No hanging requests in production
- [ ] Tests cover all timeout scenarios

## Files to Modify

1. `src/proxy/reverse/config.rs` - Add TimeoutConfig
2. `src/proxy/reverse/upstream/http/client.rs` - HTTP timeouts
3. `src/proxy/reverse/upstream/stdio.rs` - Stdio timeouts
4. `src/proxy/reverse/upstream/http/streaming/` - SSE timeouts
5. `src/error.rs` - Add Timeout error variant

## Configuration Example

```yaml
reverse_proxy:
  upstream_configs:
    - id: "primary"
      http_url: "http://localhost:8080"
      timeouts:
        connect_timeout: 5s
        request_timeout: 30s
        idle_timeout: 15s
        first_byte_timeout: 10s
```

## Monitoring

Add metrics for timeout tracking:
```rust
metrics.timeout_count.increment();
metrics.timeout_duration.record(elapsed);
metrics.circuit_breaker_state.set(state);
```