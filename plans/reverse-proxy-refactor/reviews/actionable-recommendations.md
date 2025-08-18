# Actionable Recommendations - Resource Management & Blindspots

## Immediate Actions (Day 1)

### 1. Fix Critical block_on in SSE Streaming
**File**: `shadowcat/src/proxy/reverse/hyper_sse_intercepted.rs:199-200`
**Action**: Replace synchronous blocking with async state machine

```rust
// REMOVE THIS:
let runtime = tokio::runtime::Handle::current();
let processed = runtime.block_on(self.process_event(event));

// REPLACE WITH:
enum StreamState {
    Reading,
    Processing(Pin<Box<dyn Future<Output = Option<SseEvent>> + Send>>),
    Ready(Option<SseEvent>),
}

// Use state machine in poll_next implementation
```

### 2. Add Bounded Buffers to Prevent OOM
**Files**: Multiple locations
**Action**: Implement backpressure mechanisms

```rust
// Add to constants.rs
pub const MAX_PENDING_EVENTS: usize = 100;
pub const MAX_BUFFER_SIZE: usize = 1_048_576; // 1MB
pub const MAX_EVENT_TRACKER_EVENTS: usize = 100; // Not 1000!

// Use VecDeque with size checks
if self.pending_events.len() >= MAX_PENDING_EVENTS {
    return Err(std::io::Error::other("Buffer full - applying backpressure"));
}
```

### 3. Fix Task Handle Storage in Multi-Session
**File**: `shadowcat/src/proxy/forward/multi_session.rs:327`
**Action**: Store abort handles instead of JoinHandles

```rust
pub struct SessionHandle {
    // Change from Option<JoinHandle<()>> to:
    abort_handles: Vec<AbortHandle>,
}

// When spawning tasks:
let (abort_handle, abort_registration) = AbortHandle::new_pair();
let task = tokio::spawn(Abortable::new(async move { /* ... */ }, abort_registration));
session_handle.abort_handles.push(abort_handle);
```

## Resource Management Framework

### Global Resource Manager
Create a new module `src/resource_manager.rs`:

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;
use prometheus::IntGauge;

pub struct ResourceManager {
    // Task management
    task_semaphore: Arc<Semaphore>,
    active_tasks: IntGauge,
    
    // Memory management
    memory_limit: usize,
    current_memory: Arc<AtomicUsize>,
    
    // Connection management
    connection_pool: ConnectionPool,
    max_connections_per_type: HashMap<ProxyType, usize>,
    
    // Buffer pool quotas
    buffer_quotas: HashMap<String, BufferQuota>,
}

impl ResourceManager {
    pub async fn acquire_task_permit(&self) -> Result<SemaphorePermit> {
        match self.task_semaphore.try_acquire() {
            Ok(permit) => {
                self.active_tasks.inc();
                Ok(permit)
            }
            Err(_) => {
                // Log warning about task limit
                warn!("Task limit reached, waiting for permit");
                Ok(self.task_semaphore.acquire().await?)
            }
        }
    }
    
    pub fn check_memory_limit(&self, requested: usize) -> bool {
        let current = self.current_memory.load(Ordering::Relaxed);
        current + requested <= self.memory_limit
    }
}
```

### Per-Session Resource Tracking
Add to each session:

```rust
pub struct SessionResources {
    memory_usage: AtomicUsize,
    task_count: AtomicU32,
    buffer_count: AtomicU32,
    created_at: Instant,
    last_activity: RwLock<Instant>,
}

impl SessionResources {
    pub fn track_allocation(&self, bytes: usize) {
        self.memory_usage.fetch_add(bytes, Ordering::Relaxed);
    }
    
    pub fn track_deallocation(&self, bytes: usize) {
        self.memory_usage.fetch_sub(bytes, Ordering::Relaxed);
    }
    
    pub fn memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }
}
```

## Traffic Load Handling Strategies

### 1. Low Traffic (1-10 connections)
- **Current**: Works fine
- **Optimization**: Pre-warm buffer pools
```rust
// On startup
for _ in 0..MIN_POOL_SIZE {
    pool.prewarm().await;
}
```

### 2. Medium Traffic (100-500 connections)
- **Current**: Memory growth, lock contention
- **Solution**: Implement sharded session storage
```rust
// Replace RwLock<HashMap> with DashMap
use dashmap::DashMap;
sessions: Arc<DashMap<SessionId, SessionHandle>>,
```

### 3. High Traffic (1000-5000 connections)
- **Current**: System failure
- **Solution**: Implement admission control
```rust
pub struct AdmissionController {
    max_connections: usize,
    current_connections: AtomicUsize,
    rejection_rate: RateLimiter,
}

impl AdmissionController {
    pub async fn admit(&self) -> Result<AdmissionTicket> {
        let current = self.current_connections.load(Ordering::Relaxed);
        if current >= self.max_connections {
            self.rejection_rate.check()?;
            return Err(ProxyError::CapacityExceeded);
        }
        // Admit connection
        Ok(AdmissionTicket::new())
    }
}
```

### 4. Burst Traffic (sudden spikes)
- **Solution**: Implement gradual degradation
```rust
pub enum ServiceLevel {
    Full,        // All features enabled
    Degraded,    // Disable interceptors, reduce buffer sizes
    Essential,   // Minimal proxy only, no recording
    Protective,  // Reject new connections, maintain existing
}

pub struct LoadManager {
    current_level: RwLock<ServiceLevel>,
    
    pub async fn evaluate_load(&self) -> ServiceLevel {
        let metrics = self.collect_metrics().await;
        match (metrics.cpu_usage, metrics.memory_usage, metrics.connection_count) {
            (cpu, mem, conn) if cpu < 50 && mem < 50 && conn < 100 => ServiceLevel::Full,
            (cpu, mem, conn) if cpu < 70 && mem < 70 && conn < 500 => ServiceLevel::Degraded,
            (cpu, mem, conn) if cpu < 90 && mem < 90 && conn < 1000 => ServiceLevel::Essential,
            _ => ServiceLevel::Protective,
        }
    }
}
```

## Memory Management Best Practices

### 1. Event Tracker Improvements
```rust
// Add time-based eviction
impl EventTracker {
    pub async fn cleanup_old_events(&self, max_age: Duration) {
        let now = Instant::now();
        let mut seen = self.seen_events.write().await;
        let mut order = self.recent_order.write().await;
        
        // Remove events older than max_age
        order.retain(|event| {
            if let Some(timestamp) = event.timestamp {
                now.duration_since(timestamp) < max_age
            } else {
                false
            }
        });
        
        // Rebuild seen set
        seen.clear();
        for event in order.iter() {
            seen.insert(event.id.clone());
        }
    }
}
```

### 2. Buffer Pool Enhancements
```rust
// Add per-proxy-type pools
pub mod proxy_pools {
    lazy_static! {
        pub static ref FORWARD_POOL: BytesPool = BytesPool::new(
            "forward",
            256,  // max_pooled
            8192, // initial_capacity
        );
        
        pub static ref REVERSE_POOL: BytesPool = BytesPool::new(
            "reverse",
            256,
            16384,
        );
    }
}
```

### 3. Session Eviction Policy
```rust
pub struct EvictionPolicy {
    max_sessions: usize,
    max_memory: usize,
    max_age: Duration,
    
    pub async fn evaluate(&self, sessions: &DashMap<SessionId, SessionHandle>) -> Vec<SessionId> {
        let mut candidates: Vec<(SessionId, EvictionScore)> = vec![];
        
        for entry in sessions.iter() {
            let score = self.calculate_score(&entry);
            candidates.push((entry.key().clone(), score));
        }
        
        // Sort by score (lower = more likely to evict)
        candidates.sort_by_key(|(_, score)| *score);
        
        // Return sessions to evict
        let to_evict = self.determine_eviction_count(&candidates);
        candidates.into_iter()
            .take(to_evict)
            .map(|(id, _)| id)
            .collect()
    }
}
```

## Testing & Validation

### Load Test Suite
Create `tests/load_testing/scenarios.rs`:

```rust
#[tokio::test]
async fn test_gradual_load_increase() {
    let mut proxy = setup_proxy().await;
    
    for connections in [10, 50, 100, 500, 1000] {
        println!("Testing with {} connections", connections);
        
        let handles = spawn_connections(connections).await;
        
        // Monitor metrics
        let metrics = proxy.collect_metrics().await;
        assert!(metrics.memory_usage < connections * 1_000_000); // 1MB per connection max
        assert!(metrics.task_count < connections * 5); // 5 tasks per connection max
        
        cleanup_connections(handles).await;
    }
}

#[tokio::test]
async fn test_burst_traffic() {
    let proxy = setup_proxy().await;
    
    // Start with baseline
    let baseline = spawn_connections(10).await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // Sudden burst
    let burst = spawn_connections(990).await;
    
    // Should handle gracefully
    let metrics = proxy.collect_metrics().await;
    assert!(metrics.rejected_connections < 100); // Less than 10% rejected
    
    cleanup_all().await;
}

#[tokio::test]
async fn test_memory_bounds() {
    let proxy = setup_proxy_with_limits(ResourceLimits {
        max_memory: 100_000_000, // 100MB
        max_connections: 1000,
        max_tasks: 3000,
    }).await;
    
    // Try to exhaust memory
    let connections = spawn_sse_connections(500).await;
    
    // Send large events
    for conn in &connections {
        conn.send_large_event(1_000_000).await; // 1MB event
    }
    
    // Should apply backpressure, not OOM
    let metrics = proxy.collect_metrics().await;
    assert!(metrics.memory_usage < 100_000_000);
    assert!(metrics.backpressure_events > 0);
}
```

## Monitoring & Observability

### Key Metrics to Track
```rust
// Add to metrics module
pub struct ProxyMetrics {
    // Resource metrics
    pub active_connections: IntGauge,
    pub active_tasks: IntGauge,
    pub memory_usage: IntGauge,
    pub buffer_pool_usage: IntGauge,
    
    // Performance metrics
    pub request_duration: Histogram,
    pub task_spawn_duration: Histogram,
    pub lock_wait_duration: Histogram,
    
    // Error metrics
    pub task_panics: IntCounter,
    pub oom_errors: IntCounter,
    pub backpressure_events: IntCounter,
    
    // Session metrics
    pub session_duration: Histogram,
    pub events_per_session: Histogram,
    pub bytes_per_session: Histogram,
}
```

### Health Check Endpoint
```rust
pub async fn health_check(State(proxy): State<Arc<Proxy>>) -> impl IntoResponse {
    let metrics = proxy.collect_health_metrics().await;
    
    let status = match metrics.health_score() {
        score if score > 80 => StatusCode::OK,
        score if score > 50 => StatusCode::OK, // Degraded but operational
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };
    
    (status, Json(metrics))
}
```

## Rollout Plan

### Week 1: Critical Fixes
- [ ] Day 1: Fix block_on issues
- [ ] Day 2: Add bounded buffers
- [ ] Day 3: Fix task handle storage
- [ ] Day 4: Implement basic resource manager
- [ ] Day 5: Add admission control

### Week 2: Stability
- [ ] Implement sharded session storage
- [ ] Add memory eviction policies
- [ ] Create load test suite
- [ ] Add monitoring metrics
- [ ] Document resource limits

### Week 3: Production Readiness
- [ ] Performance testing at scale
- [ ] Tune resource limits
- [ ] Add operational runbooks
- [ ] Deploy to staging
- [ ] Monitor and iterate

## Configuration Recommendations

```toml
# shadowcat.toml
[resources]
max_connections = 1000
max_memory_mb = 512
max_tasks = 3000
max_tasks_per_session = 3

[buffers]
pool_size = 256
max_buffer_size = 1048576  # 1MB
prewarm_count = 16

[sessions]
max_sessions = 1000
max_session_memory = 1048576  # 1MB
session_timeout_secs = 300
cleanup_interval_secs = 30

[sse]
max_events_per_session = 100
event_ttl_secs = 60
reconnect_timeout_secs = 30

[backpressure]
enable = true
threshold = 0.8  # 80% of limits
degradation_levels = ["full", "degraded", "essential", "protective"]
```

## Success Metrics
- Zero OOM kills under 1000 concurrent connections
- < 100ms p99 latency under normal load
- < 500MB memory usage for 1000 sessions
- Zero task leaks after 24 hours of operation
- Graceful degradation under overload

This roadmap addresses all critical resource management issues while maintaining system stability and performance targets.