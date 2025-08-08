# Task 1.3 Implementation Notes: Async Stream Patterns

## Critical Lesson: Avoiding `block_on()` in Poll Context

### The Problem
The initial implementation of `ReconnectingStream` used `tokio::runtime::Handle::current().block_on()` inside the `Stream::poll_next` method. This is a **critical anti-pattern** that can cause:
- Deadlocks in the async runtime
- Performance degradation
- Unpredictable behavior in production

### The Solution: AsyncOperation State Machine

Instead of blocking, we implemented a proper state machine using an `AsyncOperation` enum to track pending async operations:

```rust
enum AsyncOperation {
    None,
    CheckingDuplicate { event: SseEvent, future: BoxFuture<'static, bool> },
    RecordingEvent { event: SseEvent, future: BoxFuture<'static, ()> },
    RecordingActivity { future: BoxFuture<'static, ()> },
    WaitingForReconnect { delay: Pin<Box<Sleep>> },
    Reconnecting { attempt: usize, future: CreateConnectionFuture },
    GettingLastEventId { future: BoxFuture<'static, Option<String>> },
}
```

### Key Implementation Pattern

1. **Store futures as state**: Instead of awaiting directly, store futures in the state machine
2. **Poll futures in poll_next**: Use `future.poll_unpin(cx)` to poll without blocking
3. **State transitions**: Use `mem::replace` to transition between states cleanly
4. **Re-poll when needed**: Return `self.poll_next(cx)` to continue processing

### Example Pattern

```rust
fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    let this = self.as_mut().get_mut();
    
    match &mut this.async_op {
        AsyncOperation::CheckingDuplicate { event: _, future } => {
            match future.poll_unpin(cx) {
                Poll::Ready(result) => {
                    // Extract event using mem::replace
                    let event = match mem::replace(&mut this.async_op, AsyncOperation::None) {
                        AsyncOperation::CheckingDuplicate { event, .. } => event,
                        _ => unreachable!(),
                    };
                    // Process result and transition to next state
                    // ...
                    return self.poll_next(cx); // Re-poll for next operation
                }
                Poll::Pending => return Poll::Pending,
            }
        }
        // ... other states
    }
}
```

## Important Patterns for Future Tasks

### 1. Async Operations in Stream Context

When implementing Stream trait with async operations:
- **Never use `block_on()`** inside poll methods
- **Store futures as state** and poll them incrementally
- **Use state machines** to track complex async workflows
- **Re-poll appropriately** to continue processing

### 2. Thread-Safe State Management

For shared state across async boundaries:
- Use `Arc<RwLock<T>>` for read-heavy workloads
- Use `Arc<Mutex<T>>` for write-heavy or simpler cases
- Clone Arc before moving into async blocks
- Keep lock scopes minimal

### 3. Event Deduplication Pattern

The circular buffer approach using `VecDeque`:
- Efficient O(1) push/pop operations
- Bounded memory usage with capacity limits
- Fast contains() checks for recent events
- Automatic old event eviction

### 4. Exponential Backoff with Jitter

Standard formula implemented:
```rust
delay = min(base_delay * (2 ^ attempt), max_delay)
jitter = random(-jitter_factor, +jitter_factor) * delay
final_delay = delay + jitter
```

Default values that work well:
- Base delay: 1 second
- Max delay: 60 seconds  
- Jitter factor: 0.25 (25%)
- Max attempts: 10

### 5. Error Categorization for Retry Logic

Distinguish between:
- **Retryable**: 5xx, 429, network errors, timeouts
- **Non-retryable**: 4xx (except 429), parse errors, invalid content
- **Special handling**: 429 with Retry-After header

## Implications for Task 1.4 (Session Integration)

Based on the patterns established in Task 1.3:

1. **Reuse AsyncOperation pattern**: The state machine approach will be valuable for session lifecycle management
2. **Session-scoped event tracking**: Extend EventTracker to support session namespacing
3. **Health monitoring per session**: Each session should have its own health monitor instance
4. **Reconnection context**: Ensure session ID and state survive reconnections

## Testing Considerations

Critical test scenarios identified:
1. **Rapid disconnect/reconnect cycles**: Ensure state machine handles transitions correctly
2. **Concurrent operations**: Multiple streams with shared managers
3. **Memory leaks**: Verify proper cleanup in Drop implementations
4. **Event ordering**: Ensure deduplication doesn't break event sequences

## Performance Notes

Optimizations applied:
- Type aliases for complex types reduce compilation time
- Polling without allocation on each poll
- Efficient buffer management with pre-allocated capacity
- Minimal lock contention with fine-grained locking

## Code Review Checklist for Async Streams

Before committing any Stream implementation:
- [ ] No `block_on()` calls in poll methods
- [ ] Futures stored as state, not awaited directly
- [ ] Proper state machine for async operations
- [ ] Clean state transitions with mem::replace
- [ ] Re-polling logic is correct
- [ ] Drop implementation handles cleanup
- [ ] No potential for infinite poll loops
- [ ] Thread-safe for concurrent use

## References

- Task 1.3 implementation: `/Users/kevin/src/tapwire/shadowcat/src/transport/sse/reconnect.rs`
- Fixed in commit: `feat: complete MCP Task 1.3 - SSE Reconnection Logic`
- Related issue: Critical async anti-pattern in Stream implementation