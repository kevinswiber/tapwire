# Next Session: SSE Reconnection Integration

## Context
We've completed the SSE module consolidation (Phase C.6), achieving a 66% code reduction by removing duplicate implementations. The reverse proxy now uses a single hyper-based approach for SSE streaming with interceptor support.

## Discovery
We found comprehensive SSE reconnection infrastructure already exists in `src/transport/sse/` but isn't being used by the reverse proxy. This includes:
- ReconnectionManager with exponential backoff
- EventTracker for deduplication
- ReconnectingStream async state machine
- HealthMonitor for idle detection

## Next Phase: D - SSE Reconnection Integration (12 hours)

### Primary Goals
1. **Integrate existing reconnection code** - Don't reinvent the wheel
2. **Add upstream resilience** - Auto-reconnect when upstream drops
3. **Support client reconnections** - Handle Last-Event-Id properly
4. **Maintain streaming** - Never buffer entire SSE streams

### Task Sequence
Start with **D.0: Foundation Integration** (4 hours):
- Review `plans/reverse-proxy-refactor/tasks/D.0-foundation-integration.md`
- Create ReverseProxySseManager wrapping existing components
- Add Last-Event-Id tracking to sessions
- Integrate EventTracker for deduplication

Then continue with tasks D.1, D.2, and D.3 as documented.

### Key Files to Review
```bash
# Existing reconnection code to reuse
src/transport/sse/reconnect.rs      # Core reconnection logic
src/transport/sse/client.rs         # SSE client with reconnection
src/transport/sse/event.rs          # Event structure with IDs

# Current SSE implementation to enhance
src/proxy/reverse/hyper_sse_intercepted.rs  # Add reconnection here
src/proxy/reverse/legacy.rs                 # Update SSE endpoint

# Session updates needed
src/session/mod.rs                  # Add Last-Event-Id tracking
```

### Commands to Start
```bash
# Review the plan and task files
cat plans/reverse-proxy-refactor/analysis/sse-reconnection-integration.md
cat plans/reverse-proxy-refactor/tasks/D.0-foundation-integration.md

# Check existing reconnection code
grep -n "ReconnectionManager" src/transport/sse/reconnect.rs
grep -n "EventTracker" src/transport/sse/reconnect.rs

# Start implementation
code src/proxy/reverse/sse_resilience.rs  # Create new integration module
```

### Success Metrics
- Upstream disconnections auto-reconnect
- Client Last-Event-Id headers honored
- No duplicate events after reconnections
- All existing tests still pass
- Memory usage bounded

### Architecture Reminder
We're building on proven code:
- The transport layer has battle-tested reconnection logic
- We're adapting it for reverse proxy use
- Focus on integration, not reimplementation

### Testing Approach
Use MCP Inspector for real-world testing:
1. Start reverse proxy
2. Connect Inspector as client
3. Simulate upstream failures
4. Verify reconnection and deduplication

Remember: The reconnection code already exists and works. Our job is to integrate it properly with the reverse proxy's SSE handling.