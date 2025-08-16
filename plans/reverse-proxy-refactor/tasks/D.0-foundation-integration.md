# Task D.0: SSE Reconnection Foundation Integration

**Duration**: 4 hours  
**Dependencies**: Phase C complete (SSE streaming working)  
**Status**: ⬜ Not Started

## Objective
Integrate the existing SSE reconnection infrastructure from `src/transport/sse/` into the reverse proxy, creating a foundation for resilient SSE connections.

## Key Questions to Answer
1. How do we wrap ReconnectionManager for reverse proxy use?
2. Where should Last-Event-Id be stored in sessions?
3. How do we integrate EventTracker without breaking existing flow?
4. What health monitoring thresholds make sense for proxy SSE?

## Process

### 1. Create ReverseProxySseManager (1.5 hours)
```bash
# Create new module
touch src/proxy/reverse/sse_resilience.rs

# Examine existing components
grep -n "ReconnectionManager" src/transport/sse/reconnect.rs
grep -n "EventTracker" src/transport/sse/reconnect.rs
```

Design wrapper that:
- Uses Arc<ReconnectionManager> for shared state
- Manages per-session EventTrackers
- Configures health monitoring for proxy use case
- Provides factory methods for creating resilient streams

### 2. Extend Session for SSE State (1 hour)
```bash
# Review session structure
rg "pub struct Session" src/session/
```

Add to Session:
- `last_event_id: Option<String>` - Track client's last seen event
- `upstream_event_tracker: Option<Arc<EventTracker>>` - For deduplication
- Methods to get/set Last-Event-Id safely

### 3. Integrate EventTracker (1 hour)
```bash
# Review EventTracker API
grep -A 10 "impl EventTracker" src/transport/sse/reconnect.rs
```

Integration points:
- Create EventTracker when SSE session starts
- Check for duplicates before forwarding events
- Record event IDs after successful transmission
- Clean up on session end

### 4. Configure Health Monitoring (0.5 hours)
```bash
# Review health monitor configuration
grep -n "HealthMonitor" src/transport/sse/reconnect.rs
```

Configure for reverse proxy:
- Idle timeout: 30 seconds (configurable)
- Health check interval: 10 seconds
- Connection timeout: 60 seconds
- Max tracked events: 1000 per session

## Deliverables

### Code Changes
1. **New file**: `src/proxy/reverse/sse_resilience.rs`
   - ReverseProxySseManager struct
   - Factory methods for resilient streams
   - Configuration helpers

2. **Modified**: `src/session/mod.rs`
   - Add SSE-specific fields
   - Last-Event-Id tracking methods

3. **Modified**: `src/proxy/reverse/hyper_sse_intercepted.rs`
   - Integrate EventTracker for deduplication
   - Add health monitoring hooks

### Tests
- Unit tests for ReverseProxySseManager
- Session SSE state management tests
- EventTracker integration tests

## Success Criteria
- [ ] ReverseProxySseManager created and compiles
- [ ] Session tracks Last-Event-Id
- [ ] EventTracker integrated without breaking existing SSE
- [ ] Health monitoring configured and testable
- [ ] All existing tests still pass

## Commands to Run
```bash
# Build and test
cargo build --release
cargo test --lib session::
cargo test --lib proxy::reverse::

# Check for compilation issues
cargo check

# Run specific SSE tests
cargo test sse_resilience
```

## Notes
- Keep changes backward compatible
- Use feature flags if needed for gradual rollout
- Document configuration options
- Consider memory implications of event tracking