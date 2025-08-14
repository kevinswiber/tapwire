# Task S.3: Add SSE Reconnection Logic

## Objective
Improve SSE connection reliability with proper reconnection and recovery mechanisms.

## Duration
1 hour

## Dependencies
- S.2 (Buffering improvements)

## Key Features
1. Exponential backoff for reconnects
2. Connection health monitoring
3. Partial message recovery
4. Connection drop metrics

## Process

### 1. Reconnection Strategy (30 min)
- Implement exponential backoff with jitter
- Add configurable retry limits
- Track last event ID for resumption
- Handle connection failures gracefully

### 2. Health Monitoring (15 min)
- Add connection health checks
- Implement heartbeat/keepalive
- Track connection quality metrics
- Detect stale connections

### 3. Message Recovery (15 min)
- Implement last-event-id tracking
- Handle partial message recovery
- Ensure no message loss
- Add sequence validation

## Deliverables
1. Robust reconnection logic
2. Connection health monitoring
3. Message recovery mechanism
4. Connection metrics

## Success Criteria
- [ ] Exponential backoff implemented
- [ ] Health monitoring active
- [ ] No message loss during reconnects
- [ ] Metrics tracking connection drops
- [ ] Tests for reconnection scenarios

## Implementation Notes
```rust
// Reconnection configuration
pub struct ReconnectConfig {
    initial_delay: Duration,
    max_delay: Duration,
    max_retries: Option<usize>,
    jitter: bool,
}

// Track last event for resumption
struct SseState {
    last_event_id: Option<String>,
    reconnect_count: usize,
    last_successful: Instant,
}
```