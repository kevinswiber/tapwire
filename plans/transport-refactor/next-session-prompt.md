# Next Session: Phase 4 Completion & Phase 5 Start (9h)

## ✅ Phase 4 Partial Progress (2025-08-14)!

### Phase 4 Achievements (Tasks D.1 & D.2 Complete)
- ✅ **D.1 IncomingTransport types**: Implemented StdioIncoming, HttpServerIncoming, StreamableHttpIncoming
- ✅ **D.2 OutgoingTransport types**: Implemented SubprocessOutgoing, HttpClientOutgoing, StreamableHttpOutgoing
- ✅ Created 12 unit tests for directional transports - all passing
- ✅ Zero clippy warnings, all 865 tests passing

### Key Implementation Details
- **IncomingTransport**: Accepts connections, receives requests, sends responses
- **OutgoingTransport**: Initiates connections, sends requests, receives responses
- **Session Management**: Using UUID-based SessionId (36 chars)
- **Protocol Integration**: McpProtocolHandler integrated with all transport types
- **Buffer Pools**: Raw transports continue using global pools

### Known TODOs (Non-blocking)
- Add public accessor for bind address in HttpRawServer/StreamableHttpRawServer
- Add header extraction support to server transports
- Add streaming state tracking to StreamableHttpRawClient
- Add SSE mode switching to StreamableHttpRawClient

## Context
You're implementing a layered transport architecture for Shadowcat's MCP proxy. Phases 0-3 are complete (foundation, raw transport, protocol handling), and Phase 4 is 50% complete with directional transport implementations done.

## Current Status
- Phase 0-3: ✅ Complete
- Phase 4: 50% Complete (D.1 & D.2 done, D.3 & D.4 remaining)
- Phase 5: Ready to start after D.3

## Next Tasks (9h total)

### D.3: Update proxy to use new transports (3h)
**File**: `src/proxy/forward.rs` and `src/proxy/reverse.rs`
- Modify `ForwardProxy` to use IncomingTransport and OutgoingTransport
- Update `ReverseProxy` similarly
- Update transport factory methods in `src/transport/factory.rs`
- Ensure backward compatibility during transition
- Integrate with existing SessionManager

### D.4: Complete direction-aware tests (3h)
**Files**: Create `tests/integration/directional_transport_test.rs`
- Integration tests for proxy with new transports
- End-to-end tests for complete message flow
- Performance benchmarks to verify no regression
- Add more comprehensive unit test coverage

### M.1: Begin Forward Proxy Migration (3h)
**File**: `src/proxy/forward.rs`
- Replace old Transport trait usage with directional transports
- Update initialization and connection logic
- Ensure session management works correctly
- Test with real MCP servers

## Key Files
- Implementations: `src/transport/directional/incoming.rs`, `src/transport/directional/outgoing.rs`
- Proxies: `src/proxy/forward.rs`, `src/proxy/reverse.rs`
- Factory: `src/transport/factory.rs`
- Tracker: `plans/transport-refactor/transport-refactor-tracker.md`

## Success Criteria
- [ ] ForwardProxy uses new directional transports
- [ ] ReverseProxy uses new directional transports
- [ ] All existing tests still passing
- [ ] New integration tests passing
- [ ] Performance benchmarks show no regression
- [ ] No clippy warnings

## Commands to Run First
```bash
# Verify current state
cargo test transport::directional --lib
cargo test --lib  # Should show 865+ tests passing

# Check for any regressions
cargo test --test transport_regression_suite

# Verify clean code
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Important Implementation Notes
1. **Use existing types**: SessionId is UUID-based (36 chars)
2. **Protocol**: Use McpProtocolHandler for all MCP messages
3. **Error handling**: Use TransportError::InvalidConfiguration for config errors
4. **Buffer pools**: Already integrated in raw layer
5. **Testing**: StdioRawIncoming requires tokio runtime for tests

## Architecture Reminder
```
Application (Proxy) → uses directional transports
    ↓
DirectionalTransport (IncomingTransport/OutgoingTransport) ← COMPLETE
    ↓
ProtocolHandler (McpProtocolHandler) - serialization/deserialization
    ↓
RawTransport (StdioRaw*, HttpRaw*, etc.) - bytes I/O with buffer pools
    ↓
Network/Process (actual I/O)
```

## Session Goal
Complete Phase 4 by integrating directional transports into the proxy layer and creating comprehensive tests. Begin Phase 5 migration if time permits.

**Last Updated**: 2025-08-14 (D.1 & D.2 complete)
**Session Time**: Estimated 9 hours
**Next Phase**: Phase 5 - Migration and Cleanup (11h)