# Phase 1 - Task 1.3: SSE Reconnection and Resumability

## Task Overview
Implement automatic reconnection with exponential backoff for SSE connections, including support for resumability using Last-Event-ID headers as specified in the MCP protocol.

**Duration**: 3-4 hours
**Priority**: HIGH - Essential for robust SSE communication
**Dependencies**: Tasks 1.1 (Parser) and 1.2 (Connection Management) must be complete

## Objectives

### Primary Goals
1. Implement automatic reconnection on connection failure
2. Add exponential backoff with jitter
3. Support Last-Event-ID for stream resumption
4. Handle retry hints from server
5. Implement connection health monitoring

### Success Criteria
- [ ] Automatic reconnection on network failures
- [ ] Exponential backoff algorithm with configurable parameters
- [ ] Last-Event-ID header sent on reconnection
- [ ] Server retry hints respected
- [ ] Maximum retry limit enforcement
- [ ] Connection health checks and proactive reconnection
- [ ] Event deduplication after resumption
- [ ] Graceful degradation when reconnection fails
- [ ] Comprehensive test coverage for failure scenarios

## Technical Requirements

### SSE Resumability Specification
From the MCP Streamable HTTP specification:

1. **Event IDs**:
   - Server MAY attach `id` field to SSE events
   - IDs MUST be globally unique within session
   - Used as cursor for stream resumption

2. **Reconnection**:
   - Client SHOULD include `Last-Event-ID` header on reconnection
   - Server MAY replay messages after last event ID
   - Server MUST NOT replay messages from different streams

3. **Retry Timing**:
   - SSE `retry:` field specifies reconnection delay in milliseconds
   - Client should respect server-provided retry intervals

### Reconnection Strategy

1. **Exponential Backoff**:
   ```
   delay = min(base * (2 ^ attempt) + jitter, max_delay)
   ```
   - Base delay: 1 second
   - Maximum delay: 60 seconds
   - Jitter: ±25% randomization

2. **Health Monitoring**:
   - Periodic keepalive checks
   - Proactive reconnection on idle timeout
   - Connection quality metrics

## Implementation Plan

### Module Structure
```
src/transport/sse/
├── reconnect.rs      # Reconnection logic
├── backoff.rs        # Backoff algorithm
├── health.rs         # Connection health monitoring
└── tests/
    ├── reconnect.rs  # Reconnection tests
    └── backoff.rs    # Backoff tests
```

### Core Components

#### 1. Reconnection Manager (`reconnect.rs`)
```rust
pub struct ReconnectionManager {
    strategy: Box<dyn ReconnectionStrategy>,
    max_attempts: usize,
    health_monitor: HealthMonitor,
    event_tracker: EventTracker,
}

impl ReconnectionManager {
    pub fn new(config: ReconnectionConfig) -> Self;
    
    pub async fn manage_connection(
        &self,
        connection: SseConnection,
        url: String,
    ) -> ReconnectingStream;
    
    async fn reconnect(
        &self,
        url: &str,
        last_event_id: Option<String>,
        attempt: usize,
    ) -> Result<SseConnection, ReconnectionError>;
    
    fn should_reconnect(&self, error: &SseError, attempt: usize) -> bool;
}

pub struct ReconnectingStream {
    inner: Arc<Mutex<StreamState>>,
    manager: Arc<ReconnectionManager>,
}

enum StreamState {
    Connected(SseConnection),
    Reconnecting { attempt: usize, next_retry: Instant },
    Failed(ReconnectionError),
}

impl Stream for ReconnectingStream {
    type Item = Result<SseEvent, SseError>;
    
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>>;
}
```

#### 2. Backoff Strategy (`backoff.rs`)
```rust
pub trait ReconnectionStrategy: Send + Sync {
    fn next_delay(&self, attempt: usize) -> Duration;
    fn reset(&self);
}

pub struct ExponentialBackoff {
    base_delay: Duration,
    max_delay: Duration,
    jitter_factor: f64,
    multiplier: f64,
}

impl ExponentialBackoff {
    pub fn new() -> Self {
        Self {
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            jitter_factor: 0.25,
            multiplier: 2.0,
        }
    }
    
    fn add_jitter(&self, delay: Duration) -> Duration {
        let jitter = (rand::random::<f64>() - 0.5) * 2.0 * self.jitter_factor;
        let ms = delay.as_millis() as f64;
        let jittered = ms * (1.0 + jitter);
        Duration::from_millis(jittered.max(0.0) as u64)
    }
}

impl ReconnectionStrategy for ExponentialBackoff {
    fn next_delay(&self, attempt: usize) -> Duration {
        let exponential = self.base_delay.as_secs_f64() 
            * self.multiplier.powi(attempt as i32);
        let capped = Duration::from_secs_f64(exponential.min(self.max_delay.as_secs_f64()));
        self.add_jitter(capped)
    }
    
    fn reset(&self) {
        // Reset internal state if needed
    }
}
```

#### 3. Health Monitoring (`health.rs`)
```rust
pub struct HealthMonitor {
    check_interval: Duration,
    idle_timeout: Duration,
    last_activity: Arc<RwLock<Instant>>,
}

impl HealthMonitor {
    pub fn new(config: HealthConfig) -> Self;
    
    pub async fn start_monitoring(
        &self,
        connection: Arc<Mutex<SseConnection>>,
    ) -> JoinHandle<()>;
    
    pub fn record_activity(&self);
    
    pub fn is_healthy(&self) -> bool {
        let last = *self.last_activity.read();
        last.elapsed() < self.idle_timeout
    }
    
    async fn check_health(&self, connection: &mut SseConnection) -> HealthStatus;
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Idle { duration: Duration },
    Unhealthy(String),
}
```

#### 4. Event Tracking (`reconnect.rs`)
```rust
pub struct EventTracker {
    last_event_id: Arc<RwLock<Option<String>>>,
    seen_events: Arc<RwLock<HashSet<String>>>,
    max_tracked: usize,
}

impl EventTracker {
    pub fn new() -> Self;
    
    pub fn record_event(&self, event: &SseEvent);
    
    pub fn get_last_event_id(&self) -> Option<String>;
    
    pub fn is_duplicate(&self, event_id: &str) -> bool;
    
    pub fn clear_old_events(&self);
}
```

### Reconnection Flow

1. **Connection Loss Detection**:
   ```
   Stream error/EOF → Check if reconnectable →
   If yes: Start reconnection
   If no: Propagate error
   ```

2. **Reconnection Attempt**:
   ```
   Calculate backoff delay → Wait →
   Create new request with Last-Event-ID →
   Establish connection → Resume stream
   ```

3. **Event Resumption**:
   ```
   Receive events → Check for duplicates →
   Filter seen events → Emit new events →
   Update last_event_id
   ```

### Configuration

```rust
pub struct ReconnectionConfig {
    pub enabled: bool,
    pub max_attempts: usize,              // Default: 10
    pub base_delay: Duration,             // Default: 1s
    pub max_delay: Duration,              // Default: 60s
    pub jitter_factor: f64,               // Default: 0.25
    pub idle_timeout: Duration,           // Default: 5 minutes
    pub health_check_interval: Duration,  // Default: 30s
    pub max_tracked_events: usize,        // Default: 1000
}

impl Default for ReconnectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 10,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            jitter_factor: 0.25,
            idle_timeout: Duration::from_secs(300),
            health_check_interval: Duration::from_secs(30),
            max_tracked_events: 1000,
        }
    }
}
```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ReconnectionError {
    #[error("Maximum reconnection attempts ({max}) exceeded")]
    MaxAttemptsExceeded { max: usize },
    
    #[error("Connection permanently failed: {reason}")]
    PermanentFailure { reason: String },
    
    #[error("Reconnection disabled by configuration")]
    ReconnectionDisabled,
    
    #[error("Server rejected reconnection: {status}")]
    ServerRejection { status: u16 },
    
    #[error("Network error during reconnection: {0}")]
    Network(#[source] std::io::Error),
}
```

## Test Cases

### Unit Tests

1. **Backoff Calculation**:
   ```rust
   #[test]
   fn test_exponential_backoff() {
       let strategy = ExponentialBackoff::new();
       assert!(strategy.next_delay(0) < Duration::from_secs(2));
       assert!(strategy.next_delay(1) < Duration::from_secs(4));
       assert!(strategy.next_delay(10) == Duration::from_secs(60));
   }
   ```

2. **Event Tracking**:
   - Track last event ID
   - Detect duplicates
   - Clear old events

3. **Health Monitoring**:
   - Detect idle connections
   - Trigger proactive reconnection
   - Activity tracking

4. **Reconnection Logic**:
   - Should reconnect on transient errors
   - Should not reconnect on permanent errors
   - Respect max attempts

### Integration Tests

1. **Full Reconnection Flow**:
   ```rust
   #[tokio::test]
   async fn test_automatic_reconnection() {
       let mut server = MockSseServer::new();
       server.disconnect_after(Duration::from_secs(2));
       
       let stream = create_reconnecting_stream(server.url());
       let events: Vec<_> = stream.take(10).collect().await;
       
       assert_eq!(events.len(), 10);
       assert!(server.reconnection_count() > 0);
   }
   ```

2. **Resume with Last-Event-ID**:
   - Verify header is sent
   - Check event deduplication
   - Confirm proper resumption

3. **Respect Server Retry**:
   - Parse retry field
   - Use server-specified delay

4. **Maximum Attempts**:
   - Fail after max attempts
   - Proper error propagation

## Performance Considerations

1. **Memory Usage**: Limit tracked events to prevent unbounded growth
2. **CPU Usage**: Efficient duplicate detection with HashSet
3. **Network Usage**: Avoid aggressive reconnection
4. **Concurrent Reconnections**: Limit simultaneous reconnection attempts
5. **Backoff Jitter**: Prevent thundering herd

## Dependencies

```toml
[dependencies]
rand = "0.8"
parking_lot = "0.12"
tokio = { version = "1", features = ["time", "sync"] }
```

## Metrics to Track

- Reconnection attempts (successful/failed)
- Time to reconnect
- Events lost during disconnection
- Duplicate events filtered
- Connection uptime percentage
- Health check results

## Integration Points

1. **Connection Manager**: Wrap connections with reconnection logic
2. **SSE Parser**: Handle resumed event streams
3. **Session Manager**: Maintain session across reconnections
4. **Metrics**: Report reconnection statistics

## Next Steps

After completing this task:
1. Task 1.4: Integrate with session management
2. Task 1.5: Performance optimization and benchmarks

## Notes

- Consider circuit breaker pattern for failing endpoints
- Implement gradual backoff reset on successful connection
- Handle clock skew in event ID comparison
- Document reconnection behavior for users
- Consider persistent event storage for critical messages