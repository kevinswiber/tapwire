# Task 005: Connection Pool and Circuit Breaker Implementation

**Phase:** 5 (Reverse Proxy & Authentication)  
**Week:** 1 (Core Infrastructure)  
**Day:** 5  
**Priority:** Critical  
**Estimated Time:** 8-10 hours

## Overview

Implement a custom load-balancing HTTP client connection pool with circuit breaker resilience for upstream MCP servers. This task addresses the critical challenge identified in research where persistent connections bypass upstream load balancers, while providing fault tolerance and automatic recovery for upstream failures.

## Success Criteria

- [x] Research validated custom connection pool requirement for load balancing
- [x] Research validated failsafe-rs for circuit breaker implementation
- [ ] Custom connection pool with periodic refresh for load balancing
- [ ] Circuit breaker with exponential backoff and automatic recovery
- [ ] Connection health monitoring and automatic cleanup
- [ ] Load balancing across multiple upstream servers
- [ ] Resource management preventing connection leaks
- [ ] Performance target: < 2ms connection overhead
- [ ] Resilience target: Automatic recovery from upstream failures
- [ ] Integration with AuthGateway for authenticated upstream requests
- [ ] All tests passing (unit + integration + resilience)

## Technical Specifications

### LoadBalancingClientPool Implementation
```rust
use reqwest::{Client, ClientBuilder};
use failsafe::{CircuitBreaker, Config, FailurePolicy, BackoffStrategy};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, instrument};

pub struct LoadBalancingClientPool {
    clients: Arc<RwLock<HashMap<UpstreamId, ClientEntry>>>,
    upstreams: Vec<UpstreamConfig>,
    pool_config: PoolConfig,
    circuit_breakers: Arc<RwLock<HashMap<UpstreamId, Arc<CircuitBreaker<UpstreamError>>>>>,
    metrics: Arc<PoolMetrics>,
    health_checker: Arc<HealthChecker>,
}

#[derive(Debug, Clone)]
struct ClientEntry {
    client: Client,
    created_at: Instant,
    request_count: u64,
    last_used: Instant,
    health_status: HealthStatus,
}

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_age: Duration,                    // 5 minutes default
    pub max_requests_per_client: u64,         // 1000 requests default
    pub max_idle_per_host: usize,            // 10 connections default
    pub idle_timeout: Duration,               // 90 seconds default
    pub request_timeout: Duration,            // 30 seconds default
    pub connection_timeout: Duration,         // 10 seconds default
    pub health_check_interval: Duration,      // 30 seconds default
}

impl LoadBalancingClientPool {
    pub async fn new(
        upstreams: Vec<UpstreamConfig>,
        pool_config: PoolConfig,
    ) -> Result<Self, PoolError> {
        let clients = Arc::new(RwLock::new(HashMap::new()));
        let circuit_breakers = Arc::new(RwLock::new(HashMap::new()));
        let metrics = Arc::new(PoolMetrics::new());
        let health_checker = Arc::new(HealthChecker::new(pool_config.health_check_interval));

        // Initialize circuit breakers for each upstream
        let mut breakers = circuit_breakers.write().await;
        for upstream in &upstreams {
            let circuit_breaker = Arc::new(
                Config::new()
                    .failure_policy(FailurePolicy::ConsecutiveFailures(3))
                    .success_threshold(2)
                    .timeout(Duration::from_secs(60))
                    .backoff_strategy(BackoffStrategy::ExponentialBackoff {
                        base: Duration::from_millis(100),
                        max: Duration::from_secs(30),
                    })
                    .build()
            );
            breakers.insert(upstream.id.clone(), circuit_breaker);
        }
        drop(breakers);

        let pool = Self {
            clients,
            upstreams,
            pool_config,
            circuit_breakers,
            metrics,
            health_checker,
        };

        // Start background tasks
        pool.start_cleanup_task().await;
        pool.start_health_check_task().await;

        Ok(pool)
    }
}
```

### Load Balancing and Client Selection
```rust
impl LoadBalancingClientPool {
    #[instrument(skip(self))]
    pub async fn get_client(&self, target_upstream: Option<UpstreamId>) -> Result<(Client, UpstreamId), PoolError> {
        // Select upstream using load balancing strategy
        let upstream_id = match target_upstream {
            Some(id) => id,
            None => self.select_upstream().await?,
        };

        // Check circuit breaker status
        let circuit_breaker = {
            let breakers = self.circuit_breakers.read().await;
            breakers.get(&upstream_id)
                .ok_or(PoolError::UpstreamNotFound(upstream_id.clone()))?
                .clone()
        };

        // Check if circuit breaker allows requests
        if !circuit_breaker.is_call_permitted() {
            self.metrics.record_circuit_breaker_open(&upstream_id);
            return Err(PoolError::CircuitBreakerOpen(upstream_id));
        }

        // Get or create client for upstream
        let client = self.get_or_create_client(&upstream_id).await?;
        
        Ok((client, upstream_id))
    }

    async fn select_upstream(&self) -> Result<UpstreamId, PoolError> {
        // Weighted round-robin selection based on health and latency
        let mut available_upstreams = Vec::new();
        
        for upstream in &self.upstreams {
            // Check circuit breaker status
            let breakers = self.circuit_breakers.read().await;
            let circuit_breaker = breakers.get(&upstream.id).unwrap();
            
            if circuit_breaker.is_call_permitted() {
                // Check health status
                let health_status = self.health_checker.get_health(&upstream.id).await;
                if health_status.is_healthy() {
                    available_upstreams.push((upstream.id.clone(), upstream.weight));
                }
            }
        }

        if available_upstreams.is_empty() {
            return Err(PoolError::NoHealthyUpstreams);
        }

        // Weighted random selection
        let total_weight: u32 = available_upstreams.iter().map(|(_, weight)| *weight).sum();
        let mut random_weight = rand::random::<u32>() % total_weight;
        
        for (upstream_id, weight) in available_upstreams {
            if random_weight < weight {
                return Ok(upstream_id);
            }
            random_weight -= weight;
        }

        // Fallback to first available upstream
        Ok(available_upstreams[0].0.clone())
    }

    async fn get_or_create_client(&self, upstream_id: &UpstreamId) -> Result<Client, PoolError> {
        let mut clients = self.clients.write().await;
        
        // Check if we have a valid existing client
        if let Some(entry) = clients.get_mut(upstream_id) {
            let now = Instant::now();
            let age = now.duration_since(entry.created_at);
            let idle_time = now.duration_since(entry.last_used);
            
            // Check if client should be refreshed
            let should_refresh = age > self.pool_config.max_age
                || entry.request_count >= self.pool_config.max_requests_per_client
                || idle_time > self.pool_config.idle_timeout;
                
            if !should_refresh {
                entry.last_used = now;
                entry.request_count += 1;
                return Ok(entry.client.clone());
            }
        }

        // Create new client
        let upstream_config = self.upstreams
            .iter()
            .find(|u| u.id == *upstream_id)
            .ok_or_else(|| PoolError::UpstreamNotFound(upstream_id.clone()))?;

        let client = self.create_http_client(upstream_config).await?;
        
        let entry = ClientEntry {
            client: client.clone(),
            created_at: Instant::now(),
            request_count: 1,
            last_used: Instant::now(),
            health_status: HealthStatus::Unknown,
        };

        clients.insert(upstream_id.clone(), entry);
        
        self.metrics.record_client_created(upstream_id);
        
        Ok(client)
    }

    async fn create_http_client(&self, upstream: &UpstreamConfig) -> Result<Client, PoolError> {
        let mut builder = ClientBuilder::new()
            .timeout(self.pool_config.request_timeout)
            .connect_timeout(self.pool_config.connection_timeout)
            .pool_idle_timeout(self.pool_config.idle_timeout)
            .pool_max_idle_per_host(self.pool_config.max_idle_per_host)
            .user_agent("Shadowcat/1.0");

        // Add TLS configuration if needed
        if let Some(tls_config) = &upstream.tls {
            builder = builder
                .danger_accept_invalid_certs(tls_config.accept_invalid_certs)
                .min_tls_version(tls_config.min_version);
        }

        // Add authentication if configured
        if let Some(auth) = &upstream.auth {
            match auth {
                UpstreamAuth::Bearer(token) => {
                    builder = builder.default_headers({
                        let mut headers = reqwest::header::HeaderMap::new();
                        headers.insert(
                            reqwest::header::AUTHORIZATION,
                            format!("Bearer {}", token).parse().unwrap()
                        );
                        headers
                    });
                }
                UpstreamAuth::Basic { username, password } => {
                    builder = builder.basic_auth(username, Some(password));
                }
            }
        }

        builder.build().map_err(|e| PoolError::ClientCreation(e.to_string()))
    }
}
```

### Circuit Breaker Integration
```rust
impl LoadBalancingClientPool {
    #[instrument(skip(self, request_fn))]
    pub async fn execute_request<T, F, Fut>(
        &self,
        upstream_id: Option<UpstreamId>,
        request_fn: F,
    ) -> Result<T, PoolError>
    where
        F: FnOnce(Client) -> Fut,
        Fut: Future<Output = Result<T, reqwest::Error>>,
    {
        let (client, selected_upstream) = self.get_client(upstream_id).await?;
        
        let circuit_breaker = {
            let breakers = self.circuit_breakers.read().await;
            breakers.get(&selected_upstream).unwrap().clone()
        };

        // Execute request through circuit breaker
        let result = circuit_breaker.call(|| async {
            request_fn(client).await.map_err(|e| {
                // Convert reqwest errors to circuit breaker failures
                match e.is_timeout() || e.is_connect() {
                    true => UpstreamError::Timeout(e.to_string()),
                    false => UpstreamError::RequestError(e.to_string()),
                }
            })
        }).await;

        match &result {
            Ok(_) => {
                self.metrics.record_successful_request(&selected_upstream);
            }
            Err(e) => {
                self.metrics.record_failed_request(&selected_upstream, e);
                
                // Log circuit breaker state changes
                if let Err(failsafe::Error::CircuitBreakerOpen) = result {
                    warn!(
                        upstream_id = %selected_upstream,
                        "Circuit breaker opened for upstream"
                    );
                }
            }
        }

        result.map_err(|e| match e {
            failsafe::Error::CircuitBreakerOpen => PoolError::CircuitBreakerOpen(selected_upstream),
            failsafe::Error::Inner(upstream_error) => PoolError::UpstreamError(upstream_error),
            _ => PoolError::InternalError(e.to_string()),
        })
    }
}
```

### Health Checking Implementation
```rust
pub struct HealthChecker {
    health_status: Arc<RwLock<HashMap<UpstreamId, HealthStatus>>>,
    check_interval: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy { reason: String, since: Instant },
    Unknown,
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }
}

impl HealthChecker {
    pub async fn new(check_interval: Duration) -> Self {
        Self {
            health_status: Arc::new(RwLock::new(HashMap::new())),
            check_interval,
        }
    }

    pub async fn start_health_checks(&self, upstreams: Vec<UpstreamConfig>) {
        for upstream in upstreams {
            let health_status = self.health_status.clone();
            let check_interval = self.check_interval;
            
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(check_interval);
                
                loop {
                    interval.tick().await;
                    
                    let status = Self::check_upstream_health(&upstream).await;
                    
                    let mut status_map = health_status.write().await;
                    status_map.insert(upstream.id.clone(), status);
                }
            });
        }
    }

    async fn check_upstream_health(upstream: &UpstreamConfig) -> HealthStatus {
        // Simple HTTP health check
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        let health_url = format!("{}/health", upstream.base_url);
        
        match client.get(&health_url).send().await {
            Ok(response) if response.status().is_success() => HealthStatus::Healthy,
            Ok(response) => HealthStatus::Unhealthy {
                reason: format!("HTTP {}", response.status()),
                since: Instant::now(),
            },
            Err(e) => HealthStatus::Unhealthy {
                reason: format!("Connection error: {}", e),
                since: Instant::now(),
            },
        }
    }

    pub async fn get_health(&self, upstream_id: &UpstreamId) -> HealthStatus {
        let status_map = self.health_status.read().await;
        status_map.get(upstream_id)
            .cloned()
            .unwrap_or(HealthStatus::Unknown)
    }
}
```

### Background Tasks and Cleanup
```rust
impl LoadBalancingClientPool {
    async fn start_cleanup_task(&self) {
        let clients = self.clients.clone();
        let pool_config = self.pool_config.clone();
        let metrics = self.metrics.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60)); // Every minute
            
            loop {
                interval.tick().await;
                
                let mut clients_guard = clients.write().await;
                let now = Instant::now();
                let mut to_remove = Vec::new();
                
                for (upstream_id, entry) in clients_guard.iter() {
                    let age = now.duration_since(entry.created_at);
                    let idle_time = now.duration_since(entry.last_used);
                    
                    if age > pool_config.max_age 
                        || entry.request_count >= pool_config.max_requests_per_client
                        || idle_time > pool_config.idle_timeout {
                        to_remove.push(upstream_id.clone());
                    }
                }
                
                for upstream_id in to_remove {
                    clients_guard.remove(&upstream_id);
                    metrics.record_client_removed(&upstream_id);
                }
            }
        });
    }

    async fn start_health_check_task(&self) {
        let health_checker = self.health_checker.clone();
        let upstreams = self.upstreams.clone();
        
        tokio::spawn(async move {
            health_checker.start_health_checks(upstreams).await;
        });
    }
}
```

### Performance Metrics
```rust
pub struct PoolMetrics {
    clients_created: AtomicU64,
    clients_removed: AtomicU64,
    requests_successful: AtomicU64,
    requests_failed: AtomicU64,
    circuit_breaker_opens: AtomicU64,
    connection_times: Arc<RwLock<Vec<Duration>>>,
}

impl PoolMetrics {
    pub fn record_successful_request(&self, upstream_id: &UpstreamId) {
        self.requests_successful.fetch_add(1, Ordering::Relaxed);
        info!(upstream_id = %upstream_id, "Request successful");
    }

    pub fn record_failed_request(&self, upstream_id: &UpstreamId, error: &UpstreamError) {
        self.requests_failed.fetch_add(1, Ordering::Relaxed);
        warn!(upstream_id = %upstream_id, error = %error, "Request failed");
    }

    pub fn record_circuit_breaker_open(&self, upstream_id: &UpstreamId) {
        self.circuit_breaker_opens.fetch_add(1, Ordering::Relaxed);
        warn!(upstream_id = %upstream_id, "Circuit breaker opened");
    }

    pub fn get_success_rate(&self) -> f64 {
        let successful = self.requests_successful.load(Ordering::Relaxed);
        let failed = self.requests_failed.load(Ordering::Relaxed);
        
        if successful + failed == 0 {
            return 1.0;
        }
        
        successful as f64 / (successful + failed) as f64
    }
}
```

## Implementation Steps

### Step 1: Core Pool Structure
- Implement LoadBalancingClientPool with connection management
- Add PoolConfig for customizable connection parameters
- Create ClientEntry structure for connection metadata
- Implement basic client creation and reuse logic

### Step 2: Circuit Breaker Integration
- Integrate failsafe-rs for circuit breaker functionality
- Configure circuit breaker policies per upstream
- Implement failure detection and recovery logic
- Add circuit breaker state monitoring

### Step 3: Load Balancing Logic
- Implement upstream selection strategies (round-robin, weighted)
- Add health-based upstream filtering
- Create fallback mechanisms for upstream failures
- Optimize selection performance

### Step 4: Health Checking System
- Implement health checker with configurable intervals
- Add HTTP-based health check endpoints
- Create health status tracking and reporting
- Integrate health status with load balancing

### Step 5: Background Tasks and Cleanup
- Implement periodic client refresh to enable load balancing
- Add connection lifecycle management
- Create resource cleanup and leak prevention
- Add performance monitoring and metrics

## Dependencies

### Blocked By
- Task 004: AuthGateway Core Implementation (for authenticated requests)

### Blocks
- Task 006: Extended RuleBasedInterceptor with HTTP Conditions
- Task 008: End-to-End Integration Testing

### Integrates With
- HTTP server from Task 001
- AuthGateway from Task 004
- Existing session management infrastructure

## Testing Requirements

### Unit Tests
- [ ] Client pool creation and configuration
- [ ] Connection selection and load balancing
- [ ] Circuit breaker behavior and state transitions
- [ ] Health checking accuracy
- [ ] Resource cleanup and lifecycle management
- [ ] Metrics collection correctness

### Integration Tests
- [ ] End-to-end request flow through pool
- [ ] Multiple upstream server handling
- [ ] Circuit breaker with simulated failures
- [ ] Health checking with mock upstream servers
- [ ] Concurrent request handling
- [ ] Connection pool under high load

### Resilience Tests
- [ ] Upstream server failures and recovery
- [ ] Network timeouts and connection errors
- [ ] Circuit breaker opening and closing
- [ ] Health check failure scenarios
- [ ] Resource exhaustion handling

### Performance Tests
- [ ] Connection overhead (target: < 2ms)
- [ ] Request throughput with connection reuse
- [ ] Memory usage under various loads
- [ ] Load balancing effectiveness
- [ ] Circuit breaker performance impact

## Configuration Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    pub id: UpstreamId,
    pub base_url: String,
    pub weight: u32,
    pub tls: Option<TlsConfig>,
    pub auth: Option<UpstreamAuth>,
    pub health_check_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpstreamAuth {
    Bearer(String),
    Basic { username: String, password: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub accept_invalid_certs: bool,
    pub min_version: tls::Version,
}

pub type UpstreamId = String;

#[derive(Debug, Clone, thiserror::Error)]
pub enum UpstreamError {
    #[error("Request timeout: {0}")]
    Timeout(String),
    #[error("Request error: {0}")]
    RequestError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
}
```

## Performance Requirements

- **Connection overhead:** < 2ms per request
- **Memory per connection:** < 1KB metadata
- **Load balancing decision:** < 100µs
- **Health check overhead:** < 50ms per upstream
- **Circuit breaker decision:** < 10µs

## Risk Assessment

**Medium Risk**: Custom connection pool implementation, complex failure scenarios.

**Mitigation Strategies**:
- Extensive testing with simulated failures
- Gradual rollout with monitoring
- Fallback mechanisms for all failure modes
- Comprehensive logging and observability

## Completion Checklist

- [ ] LoadBalancingClientPool implemented with load balancing
- [ ] Circuit breaker integration with failsafe-rs working
- [ ] Health checking system operational
- [ ] Connection lifecycle management preventing leaks
- [ ] Background tasks for cleanup and health checks running
- [ ] Performance targets met (< 2ms connection overhead)
- [ ] Load balancing across multiple upstreams working
- [ ] Resilience testing validates automatic recovery
- [ ] Integration with AuthGateway for authenticated requests
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Resilience tests validate fault tolerance
- [ ] Performance benchmarks meeting targets
- [ ] Configuration schema documented
- [ ] Code review completed

## Files Modified/Created

### New Files
- `src/proxy/connection_pool.rs`: Core connection pool implementation
- `src/proxy/circuit_breaker.rs`: Circuit breaker integration
- `src/proxy/health_checker.rs`: Health checking system
- `src/proxy/load_balancer.rs`: Load balancing logic
- `src/proxy/metrics.rs`: Pool performance metrics
- `src/config/upstream.rs`: Upstream configuration
- `tests/unit/connection_pool_test.rs`: Unit tests
- `tests/integration/pool_resilience_test.rs`: Resilience tests

### Modified Files
- `src/proxy/mod.rs`: Export connection pool modules
- `src/proxy/reverse.rs`: Integrate connection pool
- `Cargo.toml`: Add failsafe and related dependencies
- `src/config/mod.rs`: Include upstream configuration

## Next Task
Upon completion, proceed to **Task 006: Extended RuleBasedInterceptor with HTTP Conditions** which adds policy-based security enforcement for HTTP requests using the authentication context and connection pool established in previous tasks.