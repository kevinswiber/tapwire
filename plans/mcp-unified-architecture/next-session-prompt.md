# Next Session Prompt - Sprint 1 Task 1.4 Session Manager Core

## Session Goal
Continue Sprint 1 - Implement Session Manager Core for managing MCP sessions with proper lifecycle and tracking.

## Context
- ✅ Task 1.0 Complete: Async patterns already optimal (2h instead of 8h)
- ✅ Task 1.1 Complete: OpenTelemetry observability with Prometheus implemented
- ✅ Task 1.2 Complete: Basic Hyper HTTP server with HTTP/1.1 and HTTP/2 support
- ✅ Task 1.3 Complete: Basic Hyper HTTP client with connection pooling
- 🎯 Task 1.4: Session Manager Core (8h)
- Following v2 tracker in `mcp-tracker-v2-critical-path.md`

## Current Status

### ✅ Completed (Sprint 1)
1. **Task 1.0 - Async Patterns**
   - Bounded executor preventing spawn explosion
   - One-spawn-per-client pattern validated

2. **Task 1.1 - Observability** 
   - OpenTelemetry + Prometheus metrics
   - Server, client, and pool metrics
   - Export via `export_metrics()` method

3. **Task 1.2 - Basic Hyper Server**
   - Hyper 1.x server implementation
   - Support for HTTP/1.1 and HTTP/2
   - One spawn per connection pattern
   - Health, metrics, and MCP endpoints
   - Demo example and integration tests

4. **Task 1.3 - Basic Hyper Client**
   - HTTP client using Hyper 1.x patterns
   - Connection pooling integration
   - Support for both HTTP/1.1 and HTTP/2
   - Client metrics integration
   - Created transport/http/client.rs module
   - Integration tests and examples

## Sprint 1 Task 1.4: Session Manager Core (8h) ⭐ CRITICAL

### Goal
Implement core session management functionality for tracking MCP sessions across connections.

### Key Requirements
1. **Session lifecycle management** - Create, track, expire sessions
2. **Thread-safe session storage** - Concurrent access support
3. **Session metadata tracking** - Creation time, last activity, protocol version
4. **Integration with existing pool** - Work with connection pooling
5. **Metrics integration** - Track session metrics

### Implementation Plan

1. **Review Existing Session Code** (1 hour)
   - Check existing session implementations in codebase
   - Understand current session patterns
   - Review how sessions integrate with connections

2. **Design Session Manager** (2 hours)
   - Define session lifecycle states
   - Design thread-safe storage
   - Plan session expiry mechanism
   - Define session metadata structure

3. **Implement Core Session Manager** (3 hours)
   - Create SessionManager struct
   - Implement session creation/deletion
   - Add session lookup and validation
   - Integrate with existing metrics

4. **Add Session Lifecycle** (1.5 hours)
   - Session expiry and cleanup
   - Activity tracking
   - Graceful shutdown handling

5. **Testing & Integration** (30 min)
   - Unit tests for session manager
   - Integration with HTTP client/server
   - Verify thread safety
   - Test session expiry

### Success Criteria
- [ ] Sessions can be created and tracked
- [ ] Thread-safe concurrent access
- [ ] Sessions expire after idle timeout
- [ ] Metrics track session lifecycle
- [ ] Integration with HTTP client/server works
- [ ] All tests pass

## Files to Review

1. Session-related code:
   - Check for existing session implementations
   - Review connection lifecycle management
   - Look at current metadata tracking

2. Integration points:
   - `/crates/mcp/src/pool/` - Connection pooling
   - `/crates/mcp/src/metrics/` - Metrics system
   - `/crates/mcp/src/transport/http/` - HTTP client/server

## Commands to Run

```bash
# Navigate to MCP crate
cd ~/src/tapwire/shadowcat-mcp-compliance
cd crates/mcp

# Search for existing session code
rg "session" --type rust -i
rg "SessionManager" --type rust

# Run tests
cargo test --lib

# Check metrics
cargo test --lib metrics::
```

## Next Steps After 1.4

- Task 1.5: Memory Session Store (4h)
- Then Sprint 2: Persistence & SSE

Sprint 1 will deliver a working HTTP client-server foundation with proper async patterns, observability, session management, and Hyper 1.x integration.

## Notes
- We're ahead of schedule (saved ~8h so far from Tasks 1.0-1.3)
- Session manager is critical for proxy functionality
- Should integrate cleanly with existing pool and metrics
- Memory store (Task 1.5) will be simple implementation for testing