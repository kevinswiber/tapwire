# Upstream SSE Reconnection Challenge

## Problem Statement
We need to add automatic reconnection when upstream SSE connections drop, but the current architecture makes this challenging because we're already mid-stream when we detect the disconnection.

## Current Flow
```
1. Client → Proxy: HTTP request with Accept: text/event-stream
2. Proxy → Upstream: Forward request
3. Upstream → Proxy: Returns SSE response (hyper::body::Incoming)
4. Proxy: Wraps in InterceptedSseStream, starts streaming to client
5. [Upstream drops connection]
6. Proxy: Detects EOF on body stream
7. Proxy: Currently just ends the stream to client
```

## The Challenge

### 1. ReconnectingStream Design Mismatch
`ReconnectingStream` is designed to:
- Own the entire connection lifecycle
- Create connections via a factory function
- Manage retry logic from the start

But in the reverse proxy:
- We already have a connection (hyper::body::Incoming)
- We're in the middle of streaming when disconnection happens
- We need to reconnect while maintaining client connection

### 2. State Management Complexity
When upstream disconnects, we need to:
- Keep client connection alive
- Store the last event ID we sent
- Create a new upstream connection with Last-Event-Id header
- Resume streaming without duplicates
- Handle the case where upstream doesn't support resumption

### 3. Architectural Mismatch
The current architecture has these issues:
- `InterceptedSseStream` wraps an existing body stream
- No way to "restart" with a new body stream
- Client expects continuous stream, not reconnection

## Potential Solutions

### Solution 1: Wrapper Stream with Reconnection
Create a new stream type that can handle reconnection:

```rust
pub struct ResilientSseStream {
    client_tx: mpsc::Sender<SseEvent>,
    reconnect_handle: JoinHandle<()>,
}

impl ResilientSseStream {
    async fn reconnect_loop(
        url: String,
        manager: Arc<ReverseProxySseManager>,
        session: Session,
        client_tx: mpsc::Sender<SseEvent>,
    ) {
        loop {
            // Create connection
            let body = connect_to_upstream(&url, last_event_id).await;
            
            // Stream until disconnection
            stream_until_disconnect(body, &client_tx).await;
            
            // Exponential backoff
            sleep(backoff_delay).await;
            
            // Get last event ID for resumption
            let last_event_id = manager.get_last_event_id(&session.id).await;
        }
    }
}
```

### Solution 2: Modify InterceptedSseStream
Add reconnection capability to existing stream:

```rust
impl InterceptedSseStream {
    fn poll_next(...) -> Poll<Option<Result<Frame<Bytes>>>> {
        // ... existing code ...
        
        None => {
            // Instead of ending, trigger reconnection
            if self.should_reconnect() {
                self.state = StreamState::Reconnecting;
                self.schedule_reconnect();
                Poll::Pending
            } else {
                Poll::Ready(None)
            }
        }
    }
}
```

### Solution 3: Use Existing ReconnectingStream
Refactor to use `ReconnectingStream` from the start:

```rust
// Instead of forwarding hyper response directly,
// wrap in ReconnectingStream immediately
let reconnecting = ReconnectingStream::new_from_response(
    response,
    url,
    manager,
    event_tracker,
);
```

## Recommended Approach

### Phase 1: Minimal Reconnection (What we can do now)
1. Detect upstream disconnection in `InterceptedSseStream`
2. Log reconnection would happen (but don't implement)
3. Add metrics for disconnection events
4. Document the limitation

### Phase 2: Full Implementation (After transport type fix)
1. Fix transport type architecture (separate issue)
2. Implement Solution 1 (Wrapper Stream)
3. Use channels to decouple upstream from client
4. Add full reconnection with backoff

## Why We Should Defer

1. **Transport Type Issue**: The `is_sse_session` issue indicates deeper architectural problems that should be fixed first
2. **Complexity**: Full reconnection requires significant refactoring
3. **Testing**: Need comprehensive test infrastructure for connection drops
4. **Client Compatibility**: Need to ensure clients handle the reconnection gracefully

## Interim Solution

For now, we can:
1. Use the `ReverseProxySseManager` we created for tracking
2. Log when disconnections happen
3. Add TODO comments where reconnection would occur
4. Document the limitation in the README

## Code Locations

Files that need modification for full implementation:
- `src/proxy/reverse/hyper_sse_intercepted.rs` - Add reconnection logic
- `src/proxy/reverse/legacy.rs` - Create resilient connections
- `src/proxy/reverse/sse_resilience.rs` - Add reconnection orchestration

## Next Steps

1. Document this limitation
2. Add metrics for SSE disconnections
3. Fix transport type architecture first
4. Return to implement full reconnection after architecture is clean

## Commands to Test Current Behavior

```bash
# Start a flaky SSE server that disconnects
python3 -m http.server 8001 &
# In another terminal
echo 'data: test\n\n' | nc -l 8002
# Kill the nc process to simulate disconnection

# Run proxy
cargo run -- reverse --bind 127.0.0.1:8080 --upstream http://localhost:8002

# Connect client
curl -N http://localhost:8080/sse
# Watch it end when upstream disconnects
```