# Task E.0: Fix Critical block_on Deadlock Issue

## Objective
Remove the `block_on` call in `hyper_sse_intercepted.rs` that will cause thread starvation and deadlocks under load.

## Problem
The file `shadowcat/src/proxy/reverse/hyper_sse_intercepted.rs` contains a critical `block_on` call at line 200 within an async Stream implementation. This will cause deadlocks when the system is under load because it blocks the tokio runtime thread.

```rust
// Line 199-200 - CRITICAL ISSUE
let runtime = tokio::runtime::Handle::current();
let processed = runtime.block_on(self.process_event(event));
```

## Why This Is Critical
- **Thread Starvation**: Blocks tokio runtime threads, preventing other tasks from running
- **Deadlocks at Scale**: At 100+ connections, will cause complete system freeze
- **Performance Degradation**: Even at low load, adds unnecessary blocking overhead
- **Violates Async Contract**: Stream's poll_next should never block

## Solution Approach

### Option 1: State Machine Pattern (Recommended)
Transform the interceptor processing into a state machine that can be polled:

```rust
enum StreamState {
    Ready,
    ProcessingEvent(Pin<Box<dyn Future<Output = Option<SseEvent>> + Send>>),
    SendingEvent(SseEvent),
}
```

### Option 2: Spawn Task Pattern
Spawn interceptor processing as separate tasks and use channels:

```rust
let (tx, rx) = mpsc::channel(100);
tokio::spawn(async move {
    let result = process_event(event).await;
    tx.send(result).await;
});
```

### Option 3: Pre-process Events
Process events in the hyper handler before creating the stream, avoiding async in Stream entirely.

## Implementation Steps

1. **Analyze Current Flow**
   - Map out how events flow through the stream
   - Identify what state needs to be maintained
   - Check if Session needs to be Arc<RwLock<Session>>

2. **Implement State Machine**
   - Add StreamState enum to track async operations
   - Store pending futures in the struct
   - Poll futures in poll_next without blocking

3. **Update Event Processing**
   - Make process_event return a Future
   - Store the future in StreamState::ProcessingEvent
   - Poll it on subsequent poll_next calls

4. **Test Under Load**
   - Unit test with multiple concurrent streams
   - Integration test with 100+ connections
   - Verify no thread blocking with tokio-console

## Files to Modify
- `shadowcat/src/proxy/reverse/hyper_sse_intercepted.rs` - Main fix
- `shadowcat/tests/integration_reverse_proxy.rs` - Add concurrency tests

## Success Criteria
- [ ] No `block_on` calls in async contexts
- [ ] Stream implementation properly polls futures
- [ ] Tests pass with 100+ concurrent SSE streams
- [ ] No thread blocking visible in tokio-console
- [ ] Performance: <1ms overhead per event

## Estimated Time
2-3 hours for implementation and testing

## Risk Assessment
- **High Risk**: Current code will deadlock in production
- **Medium Complexity**: State machine pattern requires careful implementation
- **Low Impact**: Changes isolated to SSE streaming module