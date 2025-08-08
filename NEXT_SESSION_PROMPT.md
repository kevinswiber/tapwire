# Phase 1, Task 1.2: SSE Connection Management

## Context

You are working on the Shadowcat MCP proxy implementation in the Tapwire project. Phase 0 (Critical Version Bug Fixes) has been COMPLETED with all 5 tasks finished successfully. Phase 1 Task 1.1 (SSE Event Parser) has also been completed. The project is now at 20.7% overall completion (6 of 29 tasks).

### Phase 0 Achievements
- ✅ All critical version bugs fixed
- ✅ Dual-channel validation fully enforced
- ✅ Version downgrade prevention implemented
- ✅ Both proxy modes have version state parity
- ✅ Performance optimized after thorough code review
- ✅ 419+ tests passing, no clippy warnings

### Phase 1 Task 1.1 Achievements
- ✅ Comprehensive SSE Event Parser implemented
- ✅ 48 SSE-specific tests passing
- ✅ Full SSE specification compliance
- ✅ Support for both MCP versions (2025-03-26 and 2025-06-18)
- ✅ Async Stream trait implementation with tokio
- ✅ Edge case handling (BOM, CRLF, comments, malformed data)

### Working Directory
```
/Users/kevin/src/tapwire/shadowcat
```

## Current Task: SSE Connection Management

### Objective
Implement persistent SSE connection management for the MCP Streamable HTTP transport, handling multiple concurrent streams, connection lifecycle, and proper resource cleanup.

### Task Details
**File**: `/Users/kevin/src/tapwire/plans/mcp-compliance/tasks/phase-1-task-002-sse-connection-management.md`
**Duration**: 4-5 hours
**Priority**: CRITICAL - Required for SSE-based communication
**Dependencies**: Task 1.1 ✅ (SSE Event Parser - Complete)

## Essential Context Files to Read

1. **Task Specification**: 
   - `/Users/kevin/src/tapwire/plans/mcp-compliance/tasks/phase-1-task-002-sse-connection-management.md`

2. **MCP SSE Specification**:
   - `/Users/kevin/src/tapwire/specs/mcp/docs/specification/2025-06-18/basic/transports.mdx` (Streamable HTTP section)
   - `/Users/kevin/src/tapwire/specs/mcp/docs/specification/2025-03-26/basic/transports.mdx` (for comparison)

3. **Existing SSE Parser** (from Task 1.1):
   - `/Users/kevin/src/tapwire/shadowcat/src/transport/sse/mod.rs`
   - `/Users/kevin/src/tapwire/shadowcat/src/transport/sse/parser.rs`
   - `/Users/kevin/src/tapwire/shadowcat/src/transport/sse/event.rs`
   - `/Users/kevin/src/tapwire/shadowcat/src/transport/sse/buffer.rs`

4. **Existing Transport Infrastructure**:
   - `/Users/kevin/src/tapwire/shadowcat/src/transport/mod.rs`
   - `/Users/kevin/src/tapwire/shadowcat/src/transport/http.rs`
   - `/Users/kevin/src/tapwire/shadowcat/src/transport/http_mcp.rs`

5. **Session Management** (for integration context):
   - `/Users/kevin/src/tapwire/shadowcat/src/session/mod.rs`

## Implementation Strategy

### Phase 1: Connection Structure (45 min)
1. Create `src/transport/sse/connection.rs`
2. Define `SseConnection` struct with state tracking
3. Implement connection lifecycle methods
4. Add connection state enum (Connecting, Connected, Reconnecting, Closed, Failed)
5. Export from SSE module

### Phase 2: Connection Manager (1.5 hours)
1. Create `src/transport/sse/manager.rs`
2. Implement `SseConnectionManager` with thread-safe storage
3. Add POST request handling with SSE/JSON detection
4. Add GET request support for server-initiated streams
5. Implement connection pool with limits
6. Add cleanup and resource management

### Phase 3: HTTP Client Integration (1.5 hours)
1. Create `src/transport/sse/client.rs`
2. Integrate with hyper for HTTP requests
3. Handle Content-Type detection (application/json vs text/event-stream)
4. Add proper header management (MCP-Protocol-Version, Mcp-Session-Id, Accept)
5. Implement response routing (immediate JSON vs SSE stream)
6. Handle 202 Accepted for notifications

### Phase 4: Stream Adapter (45 min)
1. Create async Stream implementation for SSE connections
2. Integrate with existing SseParser from Task 1.1
3. Add backpressure handling
4. Implement proper cleanup on drop
5. Track last-event-id for resumability

### Phase 5: Testing (45 min)
1. Unit tests for connection creation and lifecycle
2. Tests for multiple concurrent connections
3. Tests for POST vs GET request handling
4. Tests for Content-Type detection
5. Integration tests with mock HTTP server
6. Error handling and cleanup tests

## Success Criteria Checklist

- [ ] Can establish SSE connections via POST and GET
- [ ] Properly parse Content-Type headers for SSE detection
- [ ] Handle both single JSON responses and SSE streams
- [ ] Support multiple simultaneous connections per session
- [ ] Clean connection shutdown without resource leaks
- [ ] Error handling for network failures
- [ ] Connection state tracking and monitoring
- [ ] Integration with existing HTTP transport layer
- [ ] Comprehensive test coverage
- [ ] No clippy warnings
- [ ] All tests passing

## Commands to Use

```bash
# Navigate to shadowcat
cd /Users/kevin/src/tapwire/shadowcat

# Create new module files
touch src/transport/sse/connection.rs
touch src/transport/sse/manager.rs
touch src/transport/sse/client.rs

# Run tests for SSE module
cargo test sse

# Run specific connection tests
cargo test sse::connection
cargo test sse::manager

# Run all tests
cargo test

# Check compilation
cargo build

# Format code
cargo fmt

# Check for issues
cargo clippy --all-targets -- -D warnings

# When ready to commit (DO NOT commit unless explicitly asked)
git add -A
git status
```

## Implementation Details from Task File

### MCP Requirements to Implement
1. **POST Requests**:
   - Include `Accept: application/json, text/event-stream` header
   - Handle JSON response OR SSE stream response
   - SSE stream should remain open until response sent
   - Return 202 Accepted for notifications/responses with no body

2. **GET Requests**:
   - Include `Accept: text/event-stream` header
   - Used for server-initiated communication only
   - May remain open indefinitely
   - Handle 405 Method Not Allowed gracefully

3. **Multiple Connections**:
   - Support multiple SSE streams simultaneously
   - Each stream is independent (no broadcasting)
   - Track connections per session

### Core Types to Implement

```rust
pub struct SseConnection {
    id: Uuid,
    stream: Pin<Box<dyn Stream<Item = Result<SseEvent, SseError>> + Send>>,
    session_id: Option<String>,
    last_event_id: Option<String>,
    created_at: Instant,
    state: ConnectionState,
}

pub struct SseConnectionManager {
    connections: Arc<RwLock<HashMap<Uuid, SseConnection>>>,
    http_client: Arc<HttpClient>,
    max_connections: usize,
    session_id: Option<String>,
    protocol_version: String,
}

pub enum SseResponse {
    Json(serde_json::Value),
    Stream(Uuid),  // Connection ID for streaming response
}
```

## Important Notes

- **Always use TodoWrite tool** to track your progress through the task
- **Start with examining existing code** to understand current architecture
- **Follow established patterns** from previous implementations
- **Test incrementally** as you build each component
- **Run `cargo fmt`** after implementing new functionality
- **Run `cargo clippy --all-targets -- -D warnings`** before any commit
- **Update the refactor tracker** when the task is complete
- **Focus on the current phase objectives**

## Model Usage Guidelines

- **IMPORTANT** Be mindful of model capabilities. Assess whether Claude Opus or Claude Sonnet would be best for each step. When there's a benefit to a model change, pause and recommend it. Be mindful of the context window. When the context window has less than 15% availability, suggest creating a new Claude session and output a good prompt, referencing all available plans, tasks, and completion files that are relevant. Save the prompt into NEXT_SESSION_PROMPT.md.

## Development Workflow

1. Create todo list with TodoWrite tool to track progress
2. Examine existing codebase architecture and established patterns
3. Study current implementations related to the task
4. Design the solution approach and identify key components
5. Implement functionality incrementally with frequent testing
6. Add comprehensive error handling following project patterns
7. Create tests demonstrating functionality works correctly
8. Run tests after each significant change to catch issues early
9. Run `cargo fmt` to ensure consistent code formatting
10. Run `cargo clippy -- -D warnings` to catch potential issues
11. Update project documentation and tracker as needed
12. Commit changes with clear, descriptive messages (only when asked)

## Next Steps After This Task

Once Task 1.2 is complete:
- Update `/Users/kevin/src/tapwire/plans/mcp-compliance/compliance-tracker.md`
- Proceed to Task 1.3: SSE Reconnection Logic
- Build on the connection manager to add automatic reconnection

## Performance Targets

Remember the project performance requirements:
- **Latency overhead**: < 5% p95 for typical tool calls
- **Memory usage**: < 100MB for 1000 concurrent sessions
- **Connection limits**: 10 connections per session default
- **Buffer size**: 8KB default, configurable

## Integration Points

This task integrates with:
1. **SSE Parser** (Task 1.1): Use existing parser for event processing
2. **HTTP Transport**: Extend current HTTP transport capabilities
3. **Session Manager**: Associate connections with sessions
4. **Protocol Module**: Use version management from Phase 0
5. **Metrics**: Track connection statistics

Good luck with the SSE Connection Management implementation!