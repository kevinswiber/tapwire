# Phase 1, Task 1.3: SSE Reconnection Logic

## Context

You are working on the Shadowcat MCP proxy implementation in the Tapwire project. Phase 0 (Critical Version Bug Fixes) has been COMPLETED with all 5 tasks finished successfully. Phase 1 Tasks 1.1 (SSE Event Parser) and 1.2 (SSE Connection Management) have also been completed. The project is now at 24.1% overall completion (7 of 29 tasks).

### Phase 0 Achievements
- ✅ All critical version bugs fixed
- ✅ Dual-channel validation fully enforced
- ✅ Version downgrade prevention implemented
- ✅ Both proxy modes have version state parity
- ✅ Performance optimized after thorough code review

### Phase 1 Achievements So Far
- ✅ Task 1.1: Comprehensive SSE Event Parser (48 tests passing)
- ✅ Task 1.2: SSE Connection Management with thread-safe pooling (62 total SSE tests)
  - Fixed critical race conditions and performance issues
  - Integrated with Phase 0 protocol module
  - Added health_check() method for proactive cleanup
  - Optimized Stream polling implementation

### Working Directory
```
/Users/kevin/src/tapwire/shadowcat
```

## Current Task: SSE Reconnection Logic

### Objective
Implement automatic reconnection for SSE connections with exponential backoff, Last-Event-ID support, and proper error recovery for the MCP Streamable HTTP transport.

### Task Details
**File**: `/Users/kevin/src/tapwire/plans/mcp-compliance/tasks/phase-1-task-003-sse-reconnection.md`
**Duration**: 3-4 hours
**Priority**: CRITICAL - Required for production-ready SSE
**Dependencies**: Tasks 1.1 ✅, 1.2 ✅ (Complete)

## Essential Context Files to Read

1. **Task Specification**: 
   - `/Users/kevin/src/tapwire/plans/mcp-compliance/tasks/phase-1-task-003-sse-reconnection.md`

2. **MCP SSE Specification**:
   - `/Users/kevin/src/tapwire/specs/mcp/docs/specification/2025-06-18/basic/transports.mdx` (Streamable HTTP section)

3. **Existing SSE Implementation** (from Tasks 1.1 & 1.2):
   - `/Users/kevin/src/tapwire/shadowcat/src/transport/sse/connection.rs` (has Reconnecting state)
   - `/Users/kevin/src/tapwire/shadowcat/src/transport/sse/manager.rs` (has health_check method)
   - `/Users/kevin/src/tapwire/shadowcat/src/transport/sse/client.rs` (HTTP client integration)
   - `/Users/kevin/src/tapwire/shadowcat/src/transport/sse/event.rs` (has retry field support)

4. **Protocol Module** (for version management):
   - `/Users/kevin/src/tapwire/shadowcat/src/protocol/mod.rs`

## Foundation Already in Place

From Task 1.2, you have:
- ✅ SseConnectionManager with `health_check()` method ready for integration
- ✅ Last-Event-ID tracking already in SseConnection
- ✅ ConnectionState enum includes Reconnecting state
- ✅ Proper error handling with context for retry decisions
- ✅ Thread-safe connection pool with limits
- ✅ Optimized Stream implementation

## Implementation Strategy

### Phase 1: Reconnection Policy (45 min)
1. Create `src/transport/sse/reconnect.rs`
2. Define ReconnectPolicy struct with exponential backoff
3. Implement jitter calculation for thundering herd prevention
4. Add retry budget to prevent infinite retries
5. Honor server `retry` hints from SSE events

### Phase 2: Connection Monitor (1 hour)
1. Add connection health monitoring to manager
2. Detect disconnections and trigger reconnection
3. Track Last-Event-ID for resumption
4. Implement reconnection state machine
5. Handle different error types (4xx vs 5xx)

### Phase 3: Reconnection Implementation (1 hour)
1. Integrate reconnection logic into SseConnectionStream
2. Automatic reconnection on connection drop
3. Pass Last-Event-ID header on reconnection
4. Update connection state during reconnection
5. Emit reconnection events for observability

### Phase 4: Event Deduplication (45 min)
1. Track recent event IDs in circular buffer
2. Filter duplicate events after reconnection
3. Handle edge cases (buffer overflow, ID reuse)
4. Add metrics for duplicate detection

### Phase 5: Testing (30 min)
1. Unit tests for reconnection policy
2. Tests for connection failure scenarios
3. Tests for Last-Event-ID resumption
4. Integration tests with mock server
5. Chaos tests for network failures

## Success Criteria Checklist

- [ ] Automatic reconnection with exponential backoff
- [ ] Jitter to prevent thundering herd
- [ ] Last-Event-ID header for resumption
- [ ] Honor server retry hints
- [ ] Different handling for 4xx vs 5xx errors
- [ ] Event deduplication after reconnection
- [ ] Connection health monitoring
- [ ] Retry budget to prevent infinite loops
- [ ] Comprehensive test coverage
- [ ] No clippy warnings
- [ ] All tests passing

## Key Considerations

1. **SSE Retry Field**: The parser already supports the `retry` field - use it to update reconnection timing
2. **Connection Pool Limits**: Ensure reconnections respect the max connections limit
3. **Backpressure**: Consider what happens if events accumulate during reconnection
4. **Observability**: Add tracing for reconnection attempts and outcomes
5. **Error Categories**: 
   - 4xx errors (client errors) - don't retry or use longer backoff
   - 5xx errors (server errors) - retry with exponential backoff
   - Network errors - retry with exponential backoff

## Commands to Use

```bash
# Navigate to shadowcat
cd /Users/kevin/src/tapwire/shadowcat

# Create new module file
touch src/transport/sse/reconnect.rs

# Run tests for SSE module
cargo test sse

# Run specific reconnection tests
cargo test sse::reconnect

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

## Implementation Notes

### Exponential Backoff Formula
```
delay = min(initial_delay * (2 ^ attempt), max_delay)
jitter = random(0, delay * jitter_factor)
final_delay = delay + jitter
```

### Typical Values
- Initial delay: 1 second
- Max delay: 30 seconds
- Jitter factor: 0.3 (30%)
- Max retries: 10

### Error Handling Strategy
- Network errors: Immediate retry with backoff
- 5xx errors: Retry with backoff
- 429 (Too Many Requests): Honor Retry-After header
- 4xx errors (except 429): Don't retry or use extended backoff
- Parse errors: Don't retry

## Important Notes

- **Always use TodoWrite tool** to track your progress through the task
- **Test incrementally** as you build each component
- **Consider edge cases** like rapid disconnection/reconnection cycles
- **Run `cargo fmt`** after implementing new functionality
- **Run `cargo clippy --all-targets -- -D warnings`** before any commit
- **Update the compliance tracker** when the task is complete

## Next Steps After This Task

Once Task 1.3 is complete:
- Update `/Users/kevin/src/tapwire/plans/mcp-compliance/compliance-tracker.md`
- Proceed to Task 1.4: SSE Session Integration
- Build on reconnection to integrate with session management

## Performance Targets

Remember the project performance requirements:
- **Latency overhead**: < 5% p95 for typical tool calls
- **Memory usage**: < 100MB for 1000 concurrent sessions
- **Reconnection delay**: Minimal impact on message delivery
- **CPU usage**: Efficient backoff calculations

Good luck with the SSE Reconnection Logic implementation!