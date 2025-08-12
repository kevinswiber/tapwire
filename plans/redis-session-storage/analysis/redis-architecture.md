# Redis Session Storage Architecture

## Executive Summary

This document outlines the architectural design for adding Redis as a session storage backend to Shadowcat. The design prioritizes performance, reliability, and seamless integration with the existing in-memory storage system.

## Current State Analysis

### Existing Storage Pattern

The current `SessionManager` uses an `InMemorySessionStore` with the following characteristics:

```rust
pub struct InMemorySessionStore {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    frames: Arc<RwLock<HashMap<SessionId, Vec<MessageEnvelope>>>>,
}
```

**Strengths:**
- Zero network latency
- Simple implementation
- No external dependencies
- Atomic operations via RwLock

**Limitations:**
- Sessions lost on restart
- No sharing between instances
- Memory constrained by single host
- No persistence options

### Storage Operations Profile

Based on analysis of `SessionManager`:

| Operation | Frequency | Latency Sensitivity | Data Size |
|-----------|-----------|-------------------|-----------|
| get_session | Very High | Critical | ~1KB |
| update_activity | Very High | Critical | 8 bytes |
| add_frame | High | Important | 1-10KB |
| create_session | Medium | Low | ~1KB |
| list_sessions | Low | Low | Variable |
| delete_session | Low | Low | N/A |

## Redis Architecture Design

### Data Model

#### 1. Session Metadata (Hash)
```
Key: shadowcat:session:{session_id}
Fields:
  - transport_type: "stdio" | "http" | "sse"
  - status: "active" | "completed" | "failed" | "timeout"
  - state: "initializing" | "active" | "shutting_down" | "closed"
  - created_at: timestamp (ms)
  - last_activity: timestamp (ms)
  - frame_count: integer
  - client_info: JSON string (nullable)
  - server_info: JSON string (nullable)
  - version_state: JSON object
  - tags: JSON array
```

#### 2. Message Frames (List + Hash)
```
# Frame list for ordering
Key: shadowcat:frames:{session_id}
Values: frame IDs (UUIDs)

# Individual frame data
Key: shadowcat:frame:{frame_id}
Value: Serialized MessageEnvelope (bincode)
```

#### 3. Session Indices (Sets/Sorted Sets)
```
# Active sessions
Key: shadowcat:sessions:active
Type: Set
Members: session_ids

# Session expiry tracking
Key: shadowcat:sessions:expiry
Type: Sorted Set
Score: expiry_timestamp
Member: session_id

# Sessions by transport type (optional)
Key: shadowcat:sessions:by_transport:{transport_type}
Type: Set
Members: session_ids
```

#### 4. Pending Requests (Hash)
```
Key: shadowcat:requests:pending
Field: request_id
Value: JSON { session_id, method, timestamp }
```

#### 5. Metrics (Strings/Counters)
```
shadowcat:metrics:sessions:created     -> Counter
shadowcat:metrics:sessions:completed   -> Counter
shadowcat:metrics:sessions:failed      -> Counter
shadowcat:metrics:frames:total        -> Counter
```

### Connection Architecture

```
┌─────────────────┐
│  SessionManager │
└────────┬────────┘
         │
    ┌────▼────┐
    │ Storage │ (trait)
    │  Layer  │
    └────┬────┘
         │
    ┌────▼────────────┐
    │  StorageRouter  │ (selects backend)
    └────┬──────┬─────┘
         │      │
    ┌────▼──┐ ┌─▼──────────┐
    │Memory │ │   Redis     │
    │ Store │ │   Store     │
    └───────┘ └──┬──────────┘
                 │
            ┌────▼────┐
            │  Pool   │ (bb8/deadpool)
            └────┬────┘
                 │
            ┌────▼────┐
            │  Redis  │
            │ Cluster │
            └─────────┘
```

### Connection Pool Design

Using `bb8` with `redis-rs`:

```rust
pub struct RedisConfig {
    pub url: String,
    pub pool: PoolConfig,
    pub timeouts: TimeoutConfig,
    pub retry: RetryConfig,
}

pub struct PoolConfig {
    pub max_size: u32,          // Default: 20
    pub min_idle: Option<u32>,  // Default: 5
    pub max_lifetime: Option<Duration>,
    pub idle_timeout: Option<Duration>,
    pub connection_timeout: Duration,  // Default: 5s
}

pub struct TimeoutConfig {
    pub operation: Duration,    // Default: 1s
    pub connect: Duration,      // Default: 5s
}

pub struct RetryConfig {
    pub max_attempts: u32,      // Default: 3
    pub backoff: BackoffStrategy,
}
```

## Performance Optimization Strategies

### 1. Pipelining
Batch multiple Redis commands in a single round-trip:

```rust
// Instead of multiple round-trips
redis::pipe()
    .hset("session:123", "last_activity", now)
    .hincrby("session:123", "frame_count", 1)
    .lpush("frames:123", frame_id)
    .query_async(&mut conn).await?;
```

### 2. Local Caching Layer
Hot session cache with TTL:

```rust
pub struct CachedRedisStore {
    redis: RedisStore,
    cache: Arc<RwLock<LruCache<SessionId, CachedSession>>>,
}

struct CachedSession {
    session: Session,
    cached_at: Instant,
    ttl: Duration,  // 30 seconds for active sessions
}
```

### 3. Lazy Frame Loading
Don't load all frames at once:

```rust
pub async fn get_frames_paginated(
    &self, 
    session_id: &SessionId,
    offset: usize,
    limit: usize,
) -> SessionResult<Vec<MessageEnvelope>>;
```

### 4. Lua Scripts for Atomicity
Complex operations as Lua scripts:

```lua
-- Atomic session creation with indices
local session_key = KEYS[1]
local active_set = KEYS[2]
local expiry_zset = KEYS[3]
local session_data = ARGV[1]
local expiry_score = ARGV[2]

redis.call('HSET', session_key, unpack(session_data))
redis.call('SADD', active_set, session_id)
redis.call('ZADD', expiry_zset, expiry_score, session_id)
return 1
```

## Failover & Reliability

### 1. Circuit Breaker Pattern

```rust
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
}

enum CircuitState {
    Closed,      // Normal operation
    Open(Instant),  // Failing, reject requests
    HalfOpen,    // Testing recovery
}
```

### 2. Fallback Strategy

```rust
pub enum FallbackStrategy {
    None,              // Fail immediately
    InMemory,          // Use in-memory store
    ReadOnly,          // Allow reads, queue writes
    DegradedMode,      // Limited functionality
}
```

### 3. Health Monitoring

```rust
pub struct HealthMonitor {
    check_interval: Duration,
    unhealthy_threshold: u32,
    healthy_threshold: u32,
}

impl HealthMonitor {
    async fn check_health(&self) -> HealthStatus {
        // PING command
        // Check latency
        // Verify memory usage
        // Test write/read
    }
}
```

## Migration & Compatibility

### Storage Factory Pattern

```rust
pub struct StorageFactory;

impl StorageFactory {
    pub fn create(config: &StorageConfig) -> Result<Box<dyn SessionStore>> {
        match config.backend {
            Backend::Memory => Ok(Box::new(InMemoryStore::new())),
            Backend::Redis => {
                let redis = RedisStore::new(&config.redis)?;
                if config.redis.cache_enabled {
                    Ok(Box::new(CachedRedisStore::new(redis)))
                } else {
                    Ok(Box::new(redis))
                }
            }
            Backend::Hybrid => {
                Ok(Box::new(HybridStore::new(
                    InMemoryStore::new(),
                    RedisStore::new(&config.redis)?,
                )))
            }
        }
    }
}
```

### Zero-Downtime Migration

1. **Phase 1**: Deploy with hybrid mode (write to both)
2. **Phase 2**: Migrate reads to Redis
3. **Phase 3**: Disable in-memory writes
4. **Phase 4**: Remove in-memory storage

## Security Considerations

### 1. Connection Security
- TLS/SSL for Redis connections
- Authentication via ACL or password
- Network isolation (VPC/private network)

### 2. Data Protection
- Encrypt sensitive session data before storage
- Use Redis ACLs to limit command access
- Implement key namespacing to prevent collisions

### 3. DoS Prevention
- Rate limiting at storage layer
- Maximum session limits
- Automatic cleanup of abandoned sessions

## Monitoring & Observability

### Metrics to Track

```rust
pub struct StorageMetrics {
    // Operations
    pub ops_total: Counter,
    pub ops_failed: Counter,
    pub ops_duration: Histogram,
    
    // Connection pool
    pub pool_size: Gauge,
    pub pool_idle: Gauge,
    pub pool_wait_time: Histogram,
    
    // Cache (if enabled)
    pub cache_hits: Counter,
    pub cache_misses: Counter,
    pub cache_evictions: Counter,
    
    // Health
    pub redis_latency: Histogram,
    pub redis_errors: Counter,
    pub failover_count: Counter,
}
```

### Logging Strategy

```rust
// Success path - debug level
debug!(session_id = %id, backend = "redis", "Session retrieved");

// Failures - warn/error level
warn!(session_id = %id, error = %e, "Redis operation failed, using fallback");

// Performance - info level when slow
info!(duration_ms = %d, "Slow Redis operation detected");
```

## Testing Strategy

### 1. Unit Tests
- Mock Redis client for isolated testing
- Test serialization/deserialization
- Verify retry logic
- Test circuit breaker states

### 2. Integration Tests
```rust
#[tokio::test]
async fn test_redis_session_lifecycle() {
    let container = testcontainers::Redis::default();
    let store = RedisStore::new(&config).await.unwrap();
    // Test full lifecycle
}
```

### 3. Stress Tests
- Connection pool exhaustion
- High concurrency (1000+ sessions)
- Network failures
- Redis memory pressure

### 4. Failover Tests
- Redis becomes unavailable
- Network partitions
- Slow Redis responses
- Connection timeouts

## Future Considerations

### Redis Cluster Support
- Consistent hashing for session distribution
- Read replicas for scaling reads
- Automatic failover with Redis Sentinel

### Redis Modules
- **RedisJSON**: Native JSON operations
- **RedisTimeSeries**: Metrics and analytics
- **RedisBloom**: Probabilistic data structures

### Advanced Features
- Session replication across regions
- Point-in-time session recovery
- Session analytics and insights
- Real-time session monitoring dashboard

## Conclusion

This architecture provides a robust, scalable Redis backend while maintaining full compatibility with the existing in-memory storage. The design prioritizes:

1. **Performance**: < 5ms p95 latency through caching and pipelining
2. **Reliability**: Automatic failover and circuit breakers
3. **Compatibility**: Zero breaking changes to existing code
4. **Scalability**: Support for 10,000+ concurrent sessions
5. **Observability**: Comprehensive metrics and logging

The phased implementation approach ensures each component can be thoroughly tested before proceeding, minimizing risk to the production system.