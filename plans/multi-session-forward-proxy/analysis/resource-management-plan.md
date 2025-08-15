# Resource Management Plan for Multi-Session Forward Proxy

## Resource Dimensions

### 1. Memory Management

#### Per-Session Memory
- **Base overhead**: ~60-100KB per session
- **Buffer allocation**: 8-16KB for message buffering
- **Metadata**: ~1KB for session tracking
- **Total per session**: ~100KB typical, 1MB max

#### Global Memory
- **Session registry**: O(n) where n = active sessions
- **Connection pools**: ~10KB per pool + connections
- **Shared resources**: Interceptors, rate limiters (shared across sessions)

#### Memory Limits
```rust
pub struct MemoryLimits {
    max_total_memory: usize,        // Default: 100MB
    max_memory_per_session: usize,  // Default: 1MB
    buffer_pool_size: usize,        // Default: 10MB
}
```

### 2. File Descriptors

#### Per-Session FDs
- **Client connection**: 1 FD (TCP socket or pipe)
- **Server connection**: 1 FD (TCP socket or pipe)
- **Stdio subprocess**: +3 FDs (stdin, stdout, stderr)
- **Total**: 2-5 FDs per session

#### FD Limits
```rust
pub struct FdLimits {
    max_fds: usize,              // Default: 2000 (system limit / 2)
    reserve_fds: usize,          // Default: 100 (for system use)
    max_fds_per_session: usize,  // Default: 5
}
```

### 3. CPU & Concurrency

#### Task Management
- **Per session**: 2-3 tokio tasks
- **Global tasks**: Accept loop, cleanup loop, maintenance
- **Task limit**: 3 * max_sessions + overhead

#### CPU Allocation
```rust
pub struct ConcurrencyLimits {
    max_concurrent_sessions: usize,   // Default: 1000
    max_tasks_per_session: usize,     // Default: 3
    worker_threads: usize,             // Default: CPU cores
}
```

### 4. Network Resources

#### Bandwidth Management
- **Per-session rate limiting**: Already implemented
- **Global rate limiting**: Aggregate across sessions
- **Connection limits**: Max connections per client IP

```rust
pub struct NetworkLimits {
    max_bandwidth_per_session: usize,  // bytes/sec
    max_total_bandwidth: usize,        // bytes/sec
    max_connections_per_ip: usize,     // Default: 10
}
```

## Resource Monitoring

### Metrics Collection
```rust
pub struct ResourceMetrics {
    // Memory
    total_memory_used: AtomicU64,
    memory_per_session: HashMap<SessionId, u64>,
    
    // File descriptors
    active_fds: AtomicU64,
    fds_per_session: HashMap<SessionId, u32>,
    
    // CPU
    active_tasks: AtomicU64,
    cpu_usage_percent: AtomicU64,
    
    // Network
    bytes_in: AtomicU64,
    bytes_out: AtomicU64,
    active_connections: AtomicU64,
}
```

### Health Checks
```rust
async fn check_resource_health(&self) -> HealthStatus {
    let memory_usage = self.get_memory_usage();
    let fd_usage = self.get_fd_usage();
    let cpu_usage = self.get_cpu_usage();
    
    if memory_usage > 0.9 * self.limits.max_total_memory {
        return HealthStatus::Critical("Memory pressure");
    }
    
    if fd_usage > 0.9 * self.limits.max_fds {
        return HealthStatus::Critical("FD exhaustion");
    }
    
    if cpu_usage > 90.0 {
        return HealthStatus::Warning("High CPU usage");
    }
    
    HealthStatus::Healthy
}
```

## Resource Enforcement

### 1. Admission Control
```rust
async fn can_accept_session(&self) -> Result<(), ResourceError> {
    // Check session count
    if self.active_sessions() >= self.limits.max_sessions {
        return Err(ResourceError::MaxSessionsReached);
    }
    
    // Check memory
    if self.total_memory() + ESTIMATED_SESSION_MEMORY > self.limits.max_memory {
        return Err(ResourceError::InsufficientMemory);
    }
    
    // Check file descriptors
    if self.active_fds() + 5 > self.limits.max_fds {
        return Err(ResourceError::InsufficientFds);
    }
    
    Ok(())
}
```

### 2. Per-Client Limits
```rust
struct ClientLimits {
    sessions_per_client: HashMap<IpAddr, HashSet<SessionId>>,
    max_per_client: usize,
}

impl ClientLimits {
    fn can_accept_from(&self, client: IpAddr) -> bool {
        self.sessions_per_client
            .get(&client)
            .map(|sessions| sessions.len() < self.max_per_client)
            .unwrap_or(true)
    }
}
```

### 3. Backpressure Mechanisms

#### Session Throttling
- Slow down accept rate when approaching limits
- Reject new connections with 503 Service Unavailable
- Implement exponential backoff for retries

#### Memory Pressure Response
```rust
async fn handle_memory_pressure(&self) {
    // Level 1: Stop accepting new sessions
    self.pause_accept_loop().await;
    
    // Level 2: Evict idle sessions
    self.evict_idle_sessions(Duration::from_secs(60)).await;
    
    // Level 3: Force close oldest sessions
    if self.critical_memory_pressure() {
        self.force_close_oldest_sessions(10).await;
    }
}
```

## Resource Lifecycle

### Session Creation
1. Check admission control
2. Reserve resources (memory, FDs)
3. Create session with resource tracking
4. Monitor resource usage

### Session Operation
1. Track memory allocations
2. Monitor bandwidth usage
3. Update last activity time
4. Check resource limits

### Session Cleanup
1. Close connections
2. Release file descriptors
3. Free memory buffers
4. Update metrics
5. Return to pools if applicable

## Configuration

### Default Limits
```yaml
resource_limits:
  # Memory
  max_total_memory_mb: 100
  max_memory_per_session_kb: 1024
  
  # Sessions
  max_sessions: 1000
  max_sessions_per_client: 10
  session_timeout_secs: 300
  
  # File descriptors
  max_fds: 2000
  reserve_fds: 100
  
  # Network
  max_bandwidth_mbps: 100
  max_connections_per_ip: 10
  
  # Cleanup
  cleanup_interval_secs: 60
  idle_timeout_secs: 300
```

### Dynamic Adjustment
```rust
impl ResourceManager {
    async fn auto_tune(&mut self) {
        // Monitor system resources
        let available_memory = get_available_memory();
        let available_fds = get_fd_limit() - get_used_fds();
        
        // Adjust limits based on available resources
        self.limits.max_sessions = min(
            self.config.max_sessions,
            available_memory / ESTIMATED_SESSION_MEMORY
        );
        
        self.limits.max_fds = min(
            self.config.max_fds,
            available_fds - self.config.reserve_fds
        );
    }
}
```

## Monitoring & Alerting

### Key Metrics
1. **Session count**: Current vs max
2. **Memory usage**: Per session and total
3. **FD usage**: Active vs limit
4. **CPU usage**: Tasks and utilization
5. **Network**: Bandwidth and connections

### Alert Thresholds
- **Warning**: 80% of any limit
- **Critical**: 90% of any limit
- **Emergency**: 95% of any limit

### Observability
```rust
// Prometheus metrics
sessions_active{transport="http"} 42
sessions_total{transport="http"} 1337
memory_used_bytes{component="sessions"} 4194304
fds_active 84
cpu_usage_percent 23.5
bandwidth_bytes_per_sec{direction="in"} 1048576
```

## Testing Strategy

### Unit Tests
- Resource limit enforcement
- Admission control logic
- Cleanup and eviction

### Integration Tests
- Multiple sessions with limits
- Memory pressure handling
- FD exhaustion scenarios

### Load Tests
- Max sessions (1000)
- Memory usage under load
- FD management
- CPU utilization

### Stress Tests
- Beyond limits behavior
- Recovery from resource exhaustion
- Graceful degradation

## Implementation Priority

1. **Phase 1**: Basic limits (session count, timeouts)
2. **Phase 2**: Memory management and monitoring
3. **Phase 3**: FD tracking and limits
4. **Phase 4**: Advanced (per-client limits, auto-tuning)

## Success Metrics
- [ ] Stay within memory budget (< 100MB for 1000 sessions)
- [ ] No FD leaks under normal operation
- [ ] Graceful handling at resource limits
- [ ] < 5% CPU overhead for resource management
- [ ] Clear metrics and alerting