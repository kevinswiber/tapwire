# SSE Reconnection Integration Analysis

## Discovery
We have a complete, production-ready SSE reconnection system in `src/transport/sse/` that's not being used by the reverse proxy. This is a significant missed opportunity for resilience.

## Existing Components Available for Reuse

### 1. Core Reconnection Infrastructure (`transport/sse/reconnect.rs`)
- **ReconnectionManager**: Full lifecycle management with exponential backoff
- **ReconnectingStream**: Async state machine that auto-reconnects without blocking
- **EventTracker**: Thread-safe Last-Event-Id and deduplication tracking
- **HealthMonitor**: Detects idle/stalled connections

### 2. SSE Client Components (`transport/sse/client.rs`)
- **SseHttpClient**: Manages SSE connections with reconnection support
- **MessageResponse**: Handles both SSE streams and regular responses
- **Connection pooling**: Via SseConnectionManager

### 3. Parser and Events (`transport/sse/parser.rs`, `event.rs`)
- **SseParser**: Already used in hyper_sse_intercepted.rs
- **SseEvent**: Standardized event structure with ID tracking

## Current Reverse Proxy SSE Gaps

### What We Have (`hyper_sse_intercepted.rs`)
- ✅ Basic SSE streaming from upstream to client
- ✅ Interceptor support for modifying events
- ✅ Event parsing on-the-fly
- ✅ Hyper-based implementation (no blocking)

### What We're Missing
- ❌ No reconnection when upstream drops
- ❌ No Last-Event-Id tracking or forwarding
- ❌ No event deduplication after reconnects
- ❌ No health monitoring for stalled streams
- ❌ No client reconnection support (Last-Event-Id header)
- ❌ No connection pooling for upstream SSE

## Integration Architecture

### Design Principles
1. **Leverage existing code**: Don't reinvent what we already have
2. **Maintain streaming**: Never buffer entire SSE streams
3. **Preserve interceptors**: Keep the interceptor chain working
4. **Bidirectional resilience**: Handle both client and upstream reconnections

### Proposed Architecture

```
Client                  Reverse Proxy                    Upstream
  │                           │                              │
  ├─SSE Request──────────────►│                              │
  │ (Last-Event-Id: 42)       │                              │
  │                           ├─Create ReconnectionManager──►│
  │                           ├─Track Last-Event-Id         │
  │                           ├─SSE Request─────────────────►│
  │                           │ (Last-Event-Id: 42)         │
  │                           │                              │
  │◄──SSE Stream──────────────┤◄─────SSE Stream─────────────┤
  │   (with interceptors)     │  (auto-reconnect on drop)   │
  │                           │                              │
  │   [Connection Drop]       │                              │
  │                           │                              │
  ├─SSE Request──────────────►│                              │
  │ (Last-Event-Id: 99)       │                              │
  │                           ├─Resume from ID 99           │
  │                           ├─Deduplicate events          │
  │◄──SSE Stream──────────────┤                              │
  │   (no duplicates)         │                              │
  │                           │   [Upstream Drop]            │
  │                           ├─Auto-reconnect──────────────►│
  │                           │ (Last-Event-Id: 150)        │
  │                           ├─Resume streaming             │
  │◄──Continuous Stream───────┤◄─────Resumed Stream─────────┤
```

## Implementation Phases

### Phase D.1: Foundation Integration (4 hours)
1. Create `ReverseProxySseManager` that wraps ReconnectionManager
2. Add Last-Event-Id tracking per session
3. Integrate EventTracker for deduplication
4. Add connection health monitoring

### Phase D.2: Upstream Resilience (3 hours)
1. Replace direct hyper client with ReconnectingStream
2. Handle upstream disconnections gracefully
3. Resume from last known event ID
4. Implement exponential backoff

### Phase D.3: Client Resilience (3 hours)
1. Parse client's Last-Event-Id header
2. Store event IDs in session
3. Resume streams from client's last ID
4. Handle deduplication for client reconnects

### Phase D.4: Testing & Polish (2 hours)
1. Integration tests with connection drops
2. Performance testing under reconnection scenarios
3. Metrics for reconnection attempts
4. Documentation updates

## Benefits

### Immediate Gains
- **Resilience**: Auto-recovery from network issues
- **Efficiency**: No duplicate events after reconnects
- **Compatibility**: Full SSE spec compliance with Last-Event-Id
- **Monitoring**: Health checks for stalled connections

### Future Possibilities
- Connection pooling for multiple SSE upstreams
- Advanced retry strategies per upstream
- Circuit breaker integration
- SSE connection metrics and observability

## Risk Assessment

### Low Risk
- Reusing battle-tested code from transport layer
- Non-breaking changes (enhances existing functionality)
- Can be feature-flagged if needed

### Considerations
- Memory usage for event tracking (configurable limit)
- Complexity of state management (mitigated by reusing existing state machine)
- Testing overhead (offset by improved reliability)

## Recommendation

**STRONG RECOMMEND**: Implement this integration in Phase D. The code already exists, is well-tested, and would significantly improve the reverse proxy's reliability. This is a clear win for code reuse and system resilience.

## File References

### Existing Code to Reuse
- `src/transport/sse/reconnect.rs` - Core reconnection logic
- `src/transport/sse/client.rs` - SSE client with reconnection
- `src/transport/sse/event.rs` - Event structure with IDs
- `src/transport/sse/parser.rs` - Already using this

### Files to Modify
- `src/proxy/reverse/hyper_sse_intercepted.rs` - Add reconnection
- `src/proxy/reverse/legacy.rs` - Update SSE endpoint handling
- `src/session/mod.rs` - Add Last-Event-Id tracking

### New Files to Create
- `src/proxy/reverse/sse_resilience.rs` - Integration layer
- `tests/integration/reverse_proxy_sse_reconnect.rs` - Tests