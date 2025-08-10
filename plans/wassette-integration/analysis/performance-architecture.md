# Wassette-Shadowcat Performance Architecture

## Overview

This document defines the performance architecture for the Wassette-Shadowcat integration, ensuring the system meets latency, throughput, and resource utilization targets while maintaining reliability under load.

## Performance Requirements

### Target Metrics

| Metric | Target | Stretch Goal | Current Baseline |
|--------|--------|--------------|------------------|
| **Latency (p50)** | < 10ms | < 5ms | 7ms (Wassette alone) |
| **Latency (p95)** | < 50ms | < 25ms | 35ms (Wassette alone) |
| **Latency (p99)** | < 100ms | < 75ms | 80ms (Wassette alone) |
| **Throughput** | > 1000 req/s | > 5000 req/s | 1500 req/s |
| **Concurrent Sessions** | > 100 | > 500 | N/A |
| **Memory per Session** | < 100MB | < 50MB | 40MB (Wassette) |
| **CPU Overhead** | < 10% | < 5% | N/A |
| **Startup Time** | < 100ms | < 50ms | 70ms (Wassette) |

## Latency Budget Analysis

### End-to-End Latency Breakdown

```
Total Budget: 50ms (p95)
├── Network/IPC ────────────── 2ms
├── Shadowcat Processing ───── 3ms
│   ├── Auth Check ────────── 0.5ms
│   ├── Session Lookup ────── 0.2ms
│   ├── Interception ──────── 1ms
│   ├── Recording ─────────── 0.8ms
│   └── Message Transform ─── 0.5ms
├── Transport to Wassette ──── 2ms
├── Wassette Processing ────── 35ms
│   ├── Message Parse ──────── 1ms
│   ├── Component Lookup ──── 0.5ms
│   ├── Policy Check ───────── 2ms
│   ├── WASI Setup ─────────── 5ms
│   ├── Component Execute ─── 25ms
│   └── Result Serialization ─ 1.5ms
├── Return Path ────────────── 3ms
└── Buffer/Reserve ─────────── 5ms
```

### Critical Path Optimization

```rust
pub struct PerformanceOptimizer {
    hot_path_cache: Arc<DashMap<CacheKey, CachedResult>>,
    component_pool: Arc<ComponentPool>,
    message_batcher: Arc<MessageBatcher>,
}

impl PerformanceOptimizer {
    pub async fn optimize_request(&self, request: Request) -> Result<Response> {
        // Fast path: Check cache
        if let Some(cached) = self.hot_path_cache.get(&request.cache_key()) {
            if !cached.is_expired() {
                metrics::increment_counter!("cache_hits");
                return Ok(cached.response.clone());
            }
        }
        
        // Slow path: Process request
        let response = self.process_request(request).await?;
        
        // Update cache
        if request.is_cacheable() {
            self.hot_path_cache.insert(
                request.cache_key(),
                CachedResult::new(response.clone())
            );
        }
        
        Ok(response)
    }
}
```

## Throughput Optimization

### Connection Pooling

```rust
pub struct ConnectionPool {
    // Pre-warmed connections
    idle_connections: Arc<SegQueue<WassetteConnection>>,
    active_connections: Arc<DashMap<SessionId, WassetteConnection>>,
    
    // Pool configuration
    min_idle: usize,
    max_idle: usize,
    max_active: usize,
    
    // Metrics
    metrics: PoolMetrics,
}

impl ConnectionPool {
    pub async fn acquire(&self, session_id: SessionId) -> Result<PooledConnection> {
        let start = Instant::now();
        
        // Try to get from idle pool
        if let Some(conn) = self.idle_connections.pop() {
            self.metrics.record_acquisition(start.elapsed());
            
            // Spawn background task to maintain pool size
            if self.idle_connections.len() < self.min_idle {
                tokio::spawn(self.spawn_idle_connections(1));
            }
            
            return Ok(PooledConnection::new(conn, self.clone()));
        }
        
        // Check if we can create new connection
        if self.active_connections.len() < self.max_active {
            let conn = self.create_connection().await?;
            self.metrics.record_acquisition(start.elapsed());
            return Ok(PooledConnection::new(conn, self.clone()));
        }
        
        // Wait for available connection
        self.wait_for_connection(session_id).await
    }
    
    async fn spawn_idle_connections(&self, count: usize) {
        for _ in 0..count {
            if let Ok(conn) = self.create_connection().await {
                self.idle_connections.push(conn);
            }
        }
    }
}
```

### Message Batching

```rust
pub struct MessageBatcher {
    pending: Arc<Mutex<Vec<PendingMessage>>>,
    batch_size: usize,
    batch_timeout: Duration,
    sender: mpsc::Sender<MessageBatch>,
}

struct PendingMessage {
    message: ProtocolMessage,
    response_tx: oneshot::Sender<ProtocolMessage>,
    queued_at: Instant,
}

impl MessageBatcher {
    pub async fn send(&self, message: ProtocolMessage) -> Result<ProtocolMessage> {
        let (response_tx, response_rx) = oneshot::channel();
        
        {
            let mut pending = self.pending.lock().await;
            pending.push(PendingMessage {
                message,
                response_tx,
                queued_at: Instant::now(),
            });
            
            // Check if batch is ready
            if pending.len() >= self.batch_size {
                self.flush_batch(&mut pending).await?;
            }
        }
        
        // Wait for response
        Ok(response_rx.await?)
    }
    
    async fn flush_batch(&self, pending: &mut Vec<PendingMessage>) -> Result<()> {
        if pending.is_empty() {
            return Ok(());
        }
        
        let batch = std::mem::take(pending);
        let batch_message = MessageBatch {
            messages: batch.iter().map(|p| p.message.clone()).collect(),
            correlation_ids: batch.iter().enumerate().map(|(i, _)| i as u64).collect(),
        };
        
        // Send batch for processing
        self.sender.send(batch_message).await?;
        
        Ok(())
    }
}
```

## Memory Optimization

### Memory Pool Management

```rust
pub struct MemoryPool {
    small_buffers: Arc<SegQueue<BytesMut>>,  // 4KB buffers
    medium_buffers: Arc<SegQueue<BytesMut>>, // 64KB buffers
    large_buffers: Arc<SegQueue<BytesMut>>,  // 1MB buffers
    
    allocator_stats: Arc<AllocatorStats>,
}

impl MemoryPool {
    pub fn acquire_buffer(&self, size: usize) -> PooledBuffer {
        let buffer = match size {
            0..=4096 => {
                self.small_buffers.pop()
                    .unwrap_or_else(|| BytesMut::with_capacity(4096))
            }
            4097..=65536 => {
                self.medium_buffers.pop()
                    .unwrap_or_else(|| BytesMut::with_capacity(65536))
            }
            _ => {
                self.large_buffers.pop()
                    .unwrap_or_else(|| BytesMut::with_capacity(size))
            }
        };
        
        self.allocator_stats.record_allocation(buffer.capacity());
        PooledBuffer::new(buffer, self.clone())
    }
    
    fn return_buffer(&self, mut buffer: BytesMut) {
        buffer.clear();
        
        match buffer.capacity() {
            0..=4096 => self.small_buffers.push(buffer),
            4097..=65536 => self.medium_buffers.push(buffer),
            _ => self.large_buffers.push(buffer),
        }
    }
}
```

### Component Caching

```rust
pub struct ComponentCache {
    // LRU cache for component metadata
    metadata_cache: Arc<Mutex<LruCache<ComponentId, ComponentMetadata>>>,
    
    // Pre-instantiated components
    instance_cache: Arc<DashMap<ComponentId, Arc<ComponentInstance>>>,
    
    // Memory pressure monitoring
    memory_monitor: MemoryMonitor,
}

impl ComponentCache {
    pub async fn get_component(&self, id: &ComponentId) -> Result<Arc<ComponentInstance>> {
        // Check instance cache
        if let Some(instance) = self.instance_cache.get(id) {
            return Ok(instance.clone());
        }
        
        // Load and cache component
        let instance = self.load_component(id).await?;
        let instance = Arc::new(instance);
        
        // Check memory pressure before caching
        if self.memory_monitor.can_cache(instance.size_estimate()) {
            self.instance_cache.insert(id.clone(), instance.clone());
        }
        
        Ok(instance)
    }
    
    pub async fn evict_under_pressure(&self) {
        let pressure = self.memory_monitor.get_pressure();
        
        if pressure > 0.8 {
            // Evict least recently used components
            let to_evict = self.calculate_eviction_targets();
            for id in to_evict {
                self.instance_cache.remove(&id);
            }
        }
    }
}
```

## Concurrency Model

### Async Task Management

```rust
pub struct TaskManager {
    runtime: Arc<Runtime>,
    semaphore: Arc<Semaphore>,
    task_queue: Arc<SegQueue<Task>>,
    worker_threads: usize,
}

impl TaskManager {
    pub async fn spawn_task<F, T>(&self, task: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        // Acquire permit to limit concurrency
        let permit = self.semaphore.acquire_owned().await.unwrap();
        
        self.runtime.spawn(async move {
            let result = task.await;
            drop(permit); // Release permit
            result
        })
    }
    
    pub fn configure_for_workload(&mut self, workload: WorkloadProfile) {
        match workload {
            WorkloadProfile::HighThroughput => {
                self.worker_threads = num_cpus::get() * 2;
                self.semaphore = Arc::new(Semaphore::new(1000));
            }
            WorkloadProfile::LowLatency => {
                self.worker_threads = num_cpus::get();
                self.semaphore = Arc::new(Semaphore::new(100));
            }
            WorkloadProfile::Balanced => {
                self.worker_threads = num_cpus::get();
                self.semaphore = Arc::new(Semaphore::new(500));
            }
        }
    }
}
```

### Lock-Free Data Structures

```rust
pub struct LockFreeSessionStore {
    sessions: Arc<DashMap<SessionId, Session>>,
    session_index: Arc<SkipList<Instant, SessionId>>,
    metrics: Arc<AtomicMetrics>,
}

impl LockFreeSessionStore {
    pub fn get_session(&self, id: &SessionId) -> Option<Session> {
        self.metrics.increment_reads();
        self.sessions.get(id).map(|entry| entry.clone())
    }
    
    pub fn update_session(&self, id: SessionId, updater: impl FnOnce(&mut Session)) {
        self.metrics.increment_writes();
        
        if let Some(mut entry) = self.sessions.get_mut(&id) {
            updater(&mut entry);
            
            // Update index
            self.session_index.insert(Instant::now(), id);
        }
    }
    
    pub async fn cleanup_expired(&self) {
        let cutoff = Instant::now() - Duration::from_secs(3600);
        
        // Use index for efficient range query
        let expired: Vec<SessionId> = self.session_index
            .range(..cutoff)
            .map(|entry| entry.value().clone())
            .collect();
        
        for id in expired {
            self.sessions.remove(&id);
            self.metrics.increment_evictions();
        }
    }
}
```

## Scaling Architecture

### Horizontal Scaling

```rust
pub struct LoadBalancer {
    backends: Vec<Backend>,
    strategy: LoadBalancingStrategy,
    health_checker: HealthChecker,
}

pub enum LoadBalancingStrategy {
    RoundRobin(AtomicUsize),
    LeastConnections,
    WeightedRandom(Vec<f64>),
    ConsistentHash(HashRing),
}

impl LoadBalancer {
    pub async fn route_request(&self, request: Request) -> Result<Response> {
        let backend = self.select_backend(&request)?;
        
        // Circuit breaker check
        if backend.circuit_breaker.is_open() {
            return self.fallback_routing(request).await;
        }
        
        // Route request
        let start = Instant::now();
        let result = backend.send_request(request).await;
        
        // Update metrics
        backend.record_latency(start.elapsed());
        
        // Update circuit breaker
        match &result {
            Ok(_) => backend.circuit_breaker.record_success(),
            Err(_) => backend.circuit_breaker.record_failure(),
        }
        
        result
    }
    
    fn select_backend(&self, request: &Request) -> Result<&Backend> {
        match &self.strategy {
            LoadBalancingStrategy::RoundRobin(counter) => {
                let index = counter.fetch_add(1, Ordering::Relaxed) % self.backends.len();
                Ok(&self.backends[index])
            }
            LoadBalancingStrategy::LeastConnections => {
                self.backends
                    .iter()
                    .filter(|b| b.is_healthy())
                    .min_by_key(|b| b.active_connections())
                    .ok_or_else(|| anyhow!("No healthy backends"))
            }
            LoadBalancingStrategy::ConsistentHash(ring) => {
                let key = request.session_id.to_string();
                let backend_id = ring.get_node(&key);
                self.backends
                    .iter()
                    .find(|b| b.id == backend_id)
                    .ok_or_else(|| anyhow!("Backend not found"))
            }
            _ => unimplemented!()
        }
    }
}
```

### Backpressure Management

```rust
pub struct BackpressureController {
    current_load: Arc<AtomicU64>,
    max_load: u64,
    rejection_threshold: f64,
    shed_strategy: LoadSheddingStrategy,
}

impl BackpressureController {
    pub async fn admit_request(&self, request: &Request) -> Result<AdmissionDecision> {
        let current = self.current_load.load(Ordering::Relaxed);
        let load_ratio = current as f64 / self.max_load as f64;
        
        if load_ratio > self.rejection_threshold {
            return Ok(self.shed_load(request, load_ratio));
        }
        
        // Adaptive concurrency limiting
        if load_ratio > 0.7 {
            // Start delaying non-critical requests
            if !request.is_critical() {
                tokio::time::sleep(Duration::from_millis(
                    ((load_ratio - 0.7) * 100.0) as u64
                )).await;
            }
        }
        
        self.current_load.fetch_add(1, Ordering::Relaxed);
        Ok(AdmissionDecision::Admit)
    }
    
    fn shed_load(&self, request: &Request, load_ratio: f64) -> AdmissionDecision {
        match self.shed_strategy {
            LoadSheddingStrategy::Random => {
                if rand::random::<f64>() < (load_ratio - self.rejection_threshold) {
                    AdmissionDecision::Reject("System overloaded".to_string())
                } else {
                    AdmissionDecision::Admit
                }
            }
            LoadSheddingStrategy::Priority => {
                if request.priority() < Priority::High {
                    AdmissionDecision::Reject("Only high priority requests accepted".to_string())
                } else {
                    AdmissionDecision::Admit
                }
            }
            LoadSheddingStrategy::Adaptive => {
                // Use request characteristics to decide
                let score = self.calculate_request_score(request);
                if score < self.calculate_threshold(load_ratio) {
                    AdmissionDecision::Reject("Request does not meet threshold".to_string())
                } else {
                    AdmissionDecision::Admit
                }
            }
        }
    }
}
```

## Monitoring and Metrics

### Performance Metrics Collection

```rust
pub struct MetricsCollector {
    registry: Arc<Registry>,
    latency_histogram: Histogram,
    throughput_counter: Counter,
    error_counter: Counter,
    active_connections: Gauge,
}

impl MetricsCollector {
    pub fn record_request(&self, request: &Request, response: &Response, duration: Duration) {
        // Record latency
        self.latency_histogram
            .with_label_values(&[&request.method, &response.status.to_string()])
            .observe(duration.as_secs_f64());
        
        // Increment throughput
        self.throughput_counter
            .with_label_values(&[&request.method])
            .inc();
        
        // Track errors
        if !response.is_success() {
            self.error_counter
                .with_label_values(&[&request.method, &response.error_type()])
                .inc();
        }
    }
    
    pub fn export_metrics(&self) -> String {
        let mut buffer = String::new();
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        buffer
    }
}
```

### Real-time Performance Dashboard

```rust
pub struct PerformanceDashboard {
    metrics_store: Arc<MetricsStore>,
    websocket_clients: Arc<DashMap<ClientId, mpsc::Sender<DashboardUpdate>>>,
}

impl PerformanceDashboard {
    pub async fn stream_metrics(&self, client_id: ClientId) -> mpsc::Receiver<DashboardUpdate> {
        let (tx, rx) = mpsc::channel(100);
        self.websocket_clients.insert(client_id, tx.clone());
        
        // Send initial snapshot
        let snapshot = self.get_current_snapshot().await;
        let _ = tx.send(DashboardUpdate::Snapshot(snapshot)).await;
        
        rx
    }
    
    async fn broadcast_update(&self, update: DashboardUpdate) {
        let clients = self.websocket_clients.clone();
        
        for entry in clients.iter() {
            let _ = entry.value().send(update.clone()).await;
        }
    }
    
    async fn get_current_snapshot(&self) -> PerformanceSnapshot {
        PerformanceSnapshot {
            timestamp: Utc::now(),
            latency_p50: self.metrics_store.get_latency_percentile(0.5).await,
            latency_p95: self.metrics_store.get_latency_percentile(0.95).await,
            latency_p99: self.metrics_store.get_latency_percentile(0.99).await,
            throughput: self.metrics_store.get_throughput_rate().await,
            error_rate: self.metrics_store.get_error_rate().await,
            active_sessions: self.metrics_store.get_active_sessions().await,
            memory_usage: self.get_memory_usage(),
            cpu_usage: self.get_cpu_usage(),
        }
    }
}
```

## Performance Testing

### Load Testing Framework

```rust
pub struct LoadTest {
    config: LoadTestConfig,
    client_pool: Vec<TestClient>,
    metrics_collector: MetricsCollector,
}

impl LoadTest {
    pub async fn run(&self) -> LoadTestResults {
        let start = Instant::now();
        let mut handles = Vec::new();
        
        // Spawn concurrent clients
        for (i, client) in self.client_pool.iter().enumerate() {
            let client = client.clone();
            let config = self.config.clone();
            
            let handle = tokio::spawn(async move {
                Self::run_client_workload(i, client, config).await
            });
            
            handles.push(handle);
        }
        
        // Wait for completion
        let results: Vec<ClientResults> = futures::future::join_all(handles)
            .await
            .into_iter()
            .filter_map(Result::ok)
            .collect();
        
        LoadTestResults {
            duration: start.elapsed(),
            total_requests: results.iter().map(|r| r.request_count).sum(),
            successful_requests: results.iter().map(|r| r.success_count).sum(),
            failed_requests: results.iter().map(|r| r.failure_count).sum(),
            latency_distribution: self.calculate_latency_distribution(&results),
            throughput: self.calculate_throughput(&results, start.elapsed()),
        }
    }
    
    async fn run_client_workload(
        client_id: usize,
        client: TestClient,
        config: LoadTestConfig,
    ) -> ClientResults {
        let mut results = ClientResults::default();
        let workload = config.generate_workload(client_id);
        
        for request in workload {
            let start = Instant::now();
            
            match client.send_request(request).await {
                Ok(_) => {
                    results.success_count += 1;
                    results.latencies.push(start.elapsed());
                }
                Err(_) => {
                    results.failure_count += 1;
                }
            }
            
            results.request_count += 1;
            
            // Apply rate limiting
            if let Some(delay) = config.rate_limit_delay() {
                tokio::time::sleep(delay).await;
            }
        }
        
        results
    }
}
```

### Performance Regression Detection

```rust
pub struct PerformanceRegression {
    baseline: PerformanceBaseline,
    threshold: RegressionThreshold,
}

impl PerformanceRegression {
    pub fn check_regression(&self, current: &PerformanceMetrics) -> Vec<Regression> {
        let mut regressions = Vec::new();
        
        // Check latency regression
        if current.latency_p95 > self.baseline.latency_p95 * (1.0 + self.threshold.latency) {
            regressions.push(Regression::Latency {
                baseline: self.baseline.latency_p95,
                current: current.latency_p95,
                increase_pct: ((current.latency_p95 / self.baseline.latency_p95) - 1.0) * 100.0,
            });
        }
        
        // Check throughput regression
        if current.throughput < self.baseline.throughput * (1.0 - self.threshold.throughput) {
            regressions.push(Regression::Throughput {
                baseline: self.baseline.throughput,
                current: current.throughput,
                decrease_pct: (1.0 - (current.throughput / self.baseline.throughput)) * 100.0,
            });
        }
        
        // Check memory regression
        if current.memory_usage > self.baseline.memory_usage * (1.0 + self.threshold.memory) {
            regressions.push(Regression::Memory {
                baseline: self.baseline.memory_usage,
                current: current.memory_usage,
                increase_pct: ((current.memory_usage / self.baseline.memory_usage) - 1.0) * 100.0,
            });
        }
        
        regressions
    }
}
```

## Performance Tuning Guide

### System Configuration

```yaml
# Performance-optimized configuration
performance:
  # Connection pooling
  connection_pool:
    min_idle: 10
    max_idle: 50
    max_active: 200
    idle_timeout: 300s
    
  # Caching
  cache:
    metadata_cache_size: 1000
    component_cache_size: 100
    ttl: 60s
    
  # Batching
  batching:
    enabled: true
    batch_size: 10
    batch_timeout: 10ms
    
  # Concurrency
  concurrency:
    worker_threads: 0  # 0 = auto-detect
    max_concurrent_requests: 1000
    queue_size: 10000
    
  # Memory management
  memory:
    buffer_pool_size: 100MB
    max_message_size: 10MB
    gc_interval: 60s
    
  # Network
  network:
    tcp_nodelay: true
    keep_alive: true
    keep_alive_interval: 30s
```

### JIT Warmup

```rust
pub struct JitWarmup {
    warmup_iterations: usize,
    warmup_requests: Vec<Request>,
}

impl JitWarmup {
    pub async fn warmup(&self, system: &System) -> Result<()> {
        info!("Starting JIT warmup...");
        
        for i in 0..self.warmup_iterations {
            for request in &self.warmup_requests {
                let _ = system.process_request(request.clone()).await;
            }
            
            // Gradually increase load
            if i < self.warmup_iterations / 2 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        
        info!("JIT warmup complete");
        Ok(())
    }
}
```

## Performance Troubleshooting

### Profiling Tools Integration

```rust
pub struct Profiler {
    flamegraph_enabled: bool,
    tracy_enabled: bool,
    perf_enabled: bool,
}

impl Profiler {
    pub fn profile_request<F, T>(&self, name: &str, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        #[cfg(feature = "tracy")]
        let _span = if self.tracy_enabled {
            Some(tracy_client::span!(name))
        } else {
            None
        };
        
        #[cfg(feature = "flamegraph")]
        let _guard = if self.flamegraph_enabled {
            Some(flamegraph::start_guard(name))
        } else {
            None
        };
        
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        
        if duration > Duration::from_millis(100) {
            warn!("Slow operation '{}' took {:?}", name, duration);
        }
        
        result
    }
}
```

## Performance Recommendations

### Critical Optimizations
1. **Enable connection pooling** - Reuse Wassette processes
2. **Implement caching** - Cache component metadata
3. **Use batching** - Batch small requests
4. **Optimize serialization** - Use efficient formats
5. **Profile regularly** - Identify bottlenecks early

### Performance Checklist
- [ ] Connection pool configured
- [ ] Caching enabled for metadata
- [ ] Message batching active
- [ ] Monitoring dashboards set up
- [ ] Load testing completed
- [ ] Performance baselines established
- [ ] Profiling tools integrated
- [ ] JIT warmup implemented
- [ ] Resource limits configured
- [ ] Backpressure handling tested

This performance architecture ensures the Wassette-Shadowcat integration meets demanding performance requirements while maintaining reliability and scalability.