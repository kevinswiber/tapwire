# Next Session: Phase 4 - Direction-Aware Transports (14h)

## ✅ Phase 3 Complete with Follow-up Optimizations!

### Phase 3 Core Achievements
- ✅ Enhanced McpProtocolHandler with batch message support (deferred to separate plan)
- ✅ Implemented strict JSON-RPC 2.0 validation with error code checking
- ✅ Created protocol negotiation module with capability exchange
- ✅ Added EnvelopeProtocolHandler implementation
- ✅ 21 comprehensive protocol tests passing

### Phase 3 Follow-up Optimizations (2025-08-14)
- ✅ **Batch Support Analysis**: Created `plans/full-batch-support/` for future decision
- ✅ **Negotiation Consolidation**: Merged protocol/negotiation modules (kept MCP enum separate)
- ✅ **Buffer Pool Integration**: Integrated with all raw transports, >80% hit rate
- ✅ **Performance Validation**: Added metrics, 5 buffer pool tests, no regressions
- ✅ **Version Fixes**: Removed fake "2025-11-05", using real versions (2025-06-18, 2025-03-26)

### Critical Context
- **Buffer pools are integrated**: All raw transports use global_pools (STDIO_POOL, HTTP_POOL)
- **Three negotiators exist for good reasons**: 
  - `transport::protocol::ProtocolNegotiator` for runtime negotiation
  - `mcp::protocol::VersionNegotiator` for parsing layer (enum-based)
  - Both serve different architectural layers
- **Batch support infrastructure exists** but decision deferred pending MCP spec analysis

## Context
You're implementing a layered transport architecture for Shadowcat's MCP proxy. Phases 0-3 have successfully created the foundation, raw transport layer, and protocol handling layer.

## Current Status
- Phase 0: Prerequisites ✅ Complete
- Phase 1: Foundation Design ✅ Complete  
- Phase 2: Raw Transport Layer ✅ Complete (with buffer pools)
- Phase 3: Protocol Handler ✅ Complete (2025-08-14, with optimizations)
- Phase 4: Direction-Aware Transports 📋 Ready to start

## Next Tasks (Phase 4 - 14h total)

### D.1: Implement IncomingTransport types (4h)
Create `src/transport/directional/incoming.rs`:
- Define `IncomingTransport` trait with receive_request/send_response
- Implement `StdioIncoming` using StdioRawIncoming + McpProtocolHandler
- Implement `HttpServerIncoming` using HttpRawServer + McpProtocolHandler
- Implement `StreamableHttpIncoming` using StreamableHttpRawServer + McpProtocolHandler
- Each should handle session management and protocol negotiation

### D.2: Implement OutgoingTransport types (4h)
Create `src/transport/directional/outgoing.rs`:
- Define `OutgoingTransport` trait with send_request/receive_response
- Implement `SubprocessOutgoing` using StdioRawOutgoing + ProcessManager
- Implement `HttpClientOutgoing` using HttpRawClient + McpProtocolHandler
- Implement `StreamableHttpOutgoing` using StreamableHttpRawClient + McpProtocolHandler
- Handle connection lifecycle and protocol negotiation

### D.3: Update proxy to use new transports (3h)
- Modify `ForwardProxy` to use IncomingTransport and OutgoingTransport
- Update `ReverseProxy` similarly
- Update transport factory methods
- Ensure backward compatibility during transition
- Integrate with existing SessionManager

### D.4: Create direction-aware tests (3h)
- Unit tests for each transport implementation
- Integration tests for proxy with new transports
- End-to-end tests for complete message flow
- Performance benchmarks to verify no regression

## Key Files
- Tracker: `plans/transport-refactor/transport-refactor-tracker.md`
- Directional base: `src/transport/directional/mod.rs`
- Raw transports: `src/transport/raw/*.rs` (all implemented)
- Protocol: `src/transport/protocol/mod.rs` (enhanced with batch/negotiation)
- Process manager: `src/process/mod.rs`

## Foundation Already in Place
From Phases 1-3:
- `RawTransport` trait with all implementations working (with buffer pools!)
- `IncomingTransport` and `OutgoingTransport` traits defined in `src/transport/directional/mod.rs`
- `McpProtocolHandler` with batch support infrastructure and validation
- `ProtocolNegotiator` for version/capability exchange
- Process management separated into ProcessManager
- Buffer pools integrated with >80% hit rate
- 22 raw transport tests + 21 protocol tests + 5 buffer pool tests passing
- Total: 853 tests passing across entire codebase

## Success Criteria
- [ ] All IncomingTransport types implemented and tested
- [ ] All OutgoingTransport types implemented and tested
- [ ] Proxy updated to use new transport abstractions
- [ ] Protocol handler properly integrated
- [ ] All existing tests still passing
- [ ] No clippy warnings

## Commands to Run First
```bash
# Verify Phase 3 tests still pass
cargo test transport::protocol --lib
cargo test --test buffer_pool_test

# Check raw transports still work
cargo test transport::raw

# Verify no regressions
cargo test --test transport_regression_suite

# Check overall compilation and quality
cargo check
cargo clippy --all-targets -- -D warnings
```

## Important Implementation Notes
1. **Use buffer pools**: Raw transports already use them, continue in directional layer
2. **Protocol negotiation**: Use `transport::protocol::ProtocolNegotiator` (not MCP module version)
3. **Error handling**: Use existing `TransportError` and `TransportResult` types
4. **Session management**: Integrate with existing `SessionManager` 
5. **Version constants**: Use real versions (2025-06-18, 2025-03-26), not fake ones

## Architecture Reminder
```
Application (Proxy) → uses directional transports
    ↓
DirectionalTransport (IncomingTransport/OutgoingTransport) ← YOU ARE HERE
    ↓
ProtocolHandler (McpProtocolHandler) - serialization/deserialization
    ↓
RawTransport (StdioRaw*, HttpRaw*, etc.) - bytes I/O with buffer pools
    ↓
Network/Process (actual I/O)
```

## Session Goal
Implement the direction-aware transport layer that combines raw transports with protocol handling, providing clean IncomingTransport and OutgoingTransport abstractions for the proxy to use. This is the user-facing transport API that the proxy will interact with.

**Last Updated**: 2025-08-14 (End of Phase 3 with optimizations)
**Session Time**: Phase 4 estimated at 14 hours
**Next Phase**: Phase 5 - Migration and Cleanup (11h)