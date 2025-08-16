# Task D.2: Client SSE Resilience

**Duration**: 3 hours  
**Dependencies**: D.1 complete (upstream resilience working)  
**Status**: ⬜ Not Started

## Objective
Support client SSE reconnections by properly handling Last-Event-Id headers, maintaining event history, and resuming streams without duplicates.

## Key Questions to Answer
1. How much event history should we store per session?
2. How do we handle clients with stale Last-Event-Id?
3. Should we support multiple concurrent client connections per session?
4. How do we coordinate client and upstream Last-Event-Id?

## Process

### 1. Parse Client Last-Event-Id (1 hour)
```bash
# Check current header handling
rg "Last-Event-Id" src/proxy/reverse/

# Review SSE spec for Last-Event-Id
```

Implementation:
- Extract Last-Event-Id from request headers
- Store in session state
- Use for:
  - Initial upstream request
  - Client reconnection resumption
  - Deduplication starting point

### 2. Event History Management (1 hour)
```bash
# Review EventTracker capacity management
grep -n "max_tracked_events" src/transport/sse/reconnect.rs
```

Design decisions:
- Store last N event IDs (default: 1000)
- Optional: Store full events for replay (memory concern)
- LRU eviction when capacity reached
- Per-session or global cache?

### 3. Client Reconnection Flow (1 hour)
```bash
# Review current SSE endpoint
grep -n "handle_sse_get" src/proxy/reverse/legacy.rs
```

Reconnection handling:
1. Client connects with Last-Event-Id: "42"
2. Check if we have events after ID 42
3. If yes: Resume from that point
4. If no: Forward to upstream with Last-Event-Id
5. Deduplicate any repeated events

## Deliverables

### Code Changes
1. **Modified**: `src/proxy/reverse/legacy.rs`
   - Parse Last-Event-Id header in SSE endpoint
   - Pass to session for tracking

2. **Modified**: `src/session/mod.rs`
   - Add event history buffer
   - Methods to query event history
   - Cleanup old events

3. **Modified**: `src/proxy/reverse/hyper_sse_intercepted.rs`
   - Check for client's Last-Event-Id
   - Skip events already seen by client
   - Update session's last event tracking

### Tests
- Client reconnection with Last-Event-Id
- Multiple reconnections with different IDs
- Stale Last-Event-Id handling
- Memory usage with large event history

## Success Criteria
- [ ] Client Last-Event-Id parsed and stored
- [ ] Events deduplicated based on client's last ID
- [ ] Client can reconnect and resume stream
- [ ] No duplicate events sent to client
- [ ] Memory usage bounded and predictable
- [ ] Works with both upstream and client reconnections

## Commands to Run
```bash
# Test client reconnection
curl -H "Last-Event-Id: 42" http://localhost:8080/sse

# Test with inspector
# Start proxy, disconnect client, reconnect with Last-Event-Id

# Memory profiling
cargo test --release test_event_history_memory
```

## Test Scenarios

### 1. Simple Reconnection
```bash
# First connection
curl http://localhost:8080/sse
# Receives events 1, 2, 3
# Disconnect

# Reconnection
curl -H "Last-Event-Id: 3" http://localhost:8080/sse
# Should receive events 4, 5, 6...
```

### 2. Stale Last-Event-Id
```bash
# Client reconnects with old ID we don't have
curl -H "Last-Event-Id: ancient-id" http://localhost:8080/sse
# Should handle gracefully, possibly request full upstream replay
```

### 3. Concurrent Clients
```bash
# Multiple clients with different Last-Event-Id values
# Each should receive appropriate events
```

## Configuration Options
```yaml
sse:
  client_resilience:
    max_event_history: 1000      # Per session
    event_ttl: 300               # Seconds to keep events
    support_replay: false        # Store full events vs just IDs
    dedupe_window: 100          # How many events to check for duplicates
```

## Notes
- Consider privacy implications of storing events
- Document retention policy
- Add metrics for cache hit/miss rates
- Consider compression for stored events
- Plan for session migration if distributed