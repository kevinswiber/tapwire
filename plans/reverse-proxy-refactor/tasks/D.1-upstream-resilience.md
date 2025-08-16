# Task D.1: Upstream SSE Resilience

**Duration**: 3 hours  
**Dependencies**: D.0 complete (foundation in place)  
**Status**: ⬜ Not Started

## Objective
Replace the direct hyper client connection with ReconnectingStream to enable automatic reconnection to upstream SSE servers with exponential backoff and event resumption.

## Key Questions to Answer
1. How do we adapt ReconnectingStream for reverse proxy use?
2. Where do we inject Last-Event-Id for upstream reconnects?
3. How do we handle reconnection transparently to clients?
4. What retry strategies make sense for different upstreams?

## Process

### 1. Replace Hyper Client with ReconnectingStream (1.5 hours)
```bash
# Review current hyper implementation
grep -n "HyperHttpClient" src/proxy/reverse/hyper_client.rs

# Study ReconnectingStream
grep -n "ReconnectingStream" src/transport/sse/reconnect.rs
```

Changes needed:
- Modify `process_via_http_hyper` to use ReconnectingStream
- Create connection factory that returns resilient streams
- Preserve interceptor chain integration
- Maintain streaming without buffering

### 2. Implement Reconnection Logic (1 hour)
```bash
# Review reconnection configuration
grep -n "ReconnectionConfig" src/transport/sse/reconnect.rs
```

Configuration per upstream:
- Base delay: 1 second
- Max delay: 30 seconds
- Max retries: 10 (configurable per upstream)
- Jitter: 0.1 (10% randomization)
- Support retry-after header hints

### 3. Handle Last-Event-Id on Reconnect (0.5 hours)
```bash
# Check how Last-Event-Id is currently handled
rg "Last-Event-Id" src/transport/sse/
```

Implementation:
- Get last event ID from EventTracker
- Add to reconnection request headers
- Server resumes from that ID
- Deduplicate any repeated events

## Deliverables

### Code Changes
1. **Modified**: `src/proxy/reverse/legacy.rs`
   - Update `process_via_http_hyper` to use resilient connection
   - Add reconnection configuration

2. **Modified**: `src/proxy/reverse/hyper_sse_intercepted.rs`
   - Wrap stream in ReconnectingStream
   - Handle reconnection events
   - Update metrics on reconnects

3. **New**: Configuration for reconnection
   - Per-upstream retry settings
   - Global defaults
   - Environment variable overrides

### Tests
- Simulate upstream disconnection
- Verify automatic reconnection
- Test Last-Event-Id resumption
- Measure reconnection timing

## Success Criteria
- [ ] Upstream disconnections trigger automatic reconnect
- [ ] Exponential backoff working correctly
- [ ] Last-Event-Id sent on reconnection
- [ ] No duplicate events after reconnect
- [ ] Client connection maintained during upstream reconnect
- [ ] Metrics track reconnection attempts

## Commands to Run
```bash
# Test with simulated failures
cargo test test_upstream_reconnection

# Manual testing with proxy
cargo run -- reverse --bind 127.0.0.1:8080 --upstream http://localhost:3000

# In another terminal, simulate upstream failure
# Start/stop an SSE server to trigger reconnections
```

## Test Scenarios

### 1. Clean Disconnect
- Upstream closes connection gracefully
- Expect: Immediate reconnection attempt

### 2. Network Failure
- Upstream becomes unreachable
- Expect: Exponential backoff

### 3. Partial Event Delivery
- Connection drops mid-event
- Expect: Resume from last complete event

### 4. Server Restart
- Upstream restarts and loses state
- Expect: Handle gracefully, possibly replay

## Notes
- Log reconnection attempts at INFO level
- Include upstream URL in reconnection logs
- Consider circuit breaker integration
- Document reconnection behavior for ops team