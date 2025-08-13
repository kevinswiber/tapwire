# Next Session: Phase 3 - Protocol Layer (3h)

## ✅ Phase 2 Complete with Improvements!

### Phase 2 Achievements
- ✅ Implemented all raw transports (StdioRawIncoming, StdioRawOutgoing, HttpRawClient, HttpRawServer)
- ✅ Implemented SSE transport (SseRawClient, SseRawServer)
- ✅ Implemented StreamableHttpRawTransport (KEY INNOVATION - unified HTTP+SSE)
- ✅ Fixed ALL compilation errors
- ✅ All 22 raw transport tests passing
- ✅ Code formatted with cargo fmt
- ✅ Zero clippy warnings

### Critical Fixes Applied (Post-Review)
- **Fixed duplicate process spawning bug** - StdioRawOutgoing was spawning TWO processes
- **Made ProcessManager fully async** - Removed all futures::executor::block_on usage
- **Improved Command API** - Added with_program() for direct string usage
- **Added type aliases** - HttpRequestChannel and RequestRouter for cleaner code
- **Fixed field visibility** - Used #[allow(dead_code)] for fields that will be used

## Context
You're implementing a layered transport architecture to solve issues with Shadowcat's MCP transport system. Phase 2 successfully created the raw transport layer that handles byte-level I/O without protocol knowledge.

## Current Status
- Phase 0: Prerequisites ✅ Complete
- Phase 1: Foundation Design ✅ Complete  
- Phase 2: Raw Transport Layer ✅ Complete (2025-08-13)
- Phase 3: Protocol Layer 📋 Ready to start

## Next Tasks (Phase 3 - 3h total)

### P.1: Implement JsonRpcProtocolHandler (1h)
From file: `plans/transport-refactor/tasks/P.1-json-rpc-protocol-handler.md`
- The foundation already exists in src/transport/protocol/mod.rs (McpProtocolHandler)
- Enhance with proper batch message support
- Add strict validation mode
- Implement proper error handling

### P.2: Implement McpProtocolValidator (30m)
From file: `plans/transport-refactor/tasks/P.2-mcp-protocol-validator.md`
- Validate MCP-specific requirements
- Check required fields and methods
- Enforce protocol version constraints
- Add known method validation

### P.3: Implement protocol negotiation (1h)
From file: `plans/transport-refactor/tasks/P.3-protocol-negotiation.md`
- Version negotiation for MCP
- Capability exchange
- Protocol upgrade paths
- Header-based negotiation

### P.4: Create protocol tests (30m)
From file: `plans/transport-refactor/tasks/P.4-protocol-tests.md`
- Unit tests for all handlers
- Protocol compliance tests
- Error handling tests
- Batch message tests

## Key Files
- Tracker: `plans/transport-refactor/transport-refactor-tracker.md`
- Protocol implementation: `src/transport/protocol/mod.rs` (McpProtocolHandler already started)
- Raw transports: `src/transport/raw/*.rs` (all implemented in Phase 2)
- Directional: `src/transport/directional/mod.rs` (uses protocol handler)

## Foundation Already in Place
From Phase 2, we have:
- `RawTransport` trait fully implemented
- All raw transports working and tested
- Process management extracted to `src/process/mod.rs`
- Error types extended with needed variants

## Success Criteria
- [ ] McpProtocolHandler enhanced with full functionality
- [ ] MCP validation working correctly
- [ ] Protocol negotiation functional
- [ ] All protocol tests passing
- [ ] No clippy warnings
- [ ] Regression tests still passing

## Implementation Notes
- The McpProtocolHandler already exists with basic serialize/deserialize
- Focus on enhancing it with proper validation and batch support
- Ensure compatibility with existing MCP flows
- The protocol layer bridges raw transports and directional transports

## Commands to Run First
```bash
# Verify Phase 2 tests still pass
cargo test --test raw_transport_tests

# Check protocol module
cargo check

# See current protocol implementation
cat src/transport/protocol/mod.rs
```

## Session Goal
Enhance the existing McpProtocolHandler with full JSON-RPC 2.0 and MCP compliance, including batch messages, validation, and protocol negotiation.

**Last Updated**: 2025-08-13 (End of Phase 2)
**Session Time**: Phase 3 estimated at 3 hours
**Next Phase**: Phase 4 - Direction-aware transports (5h)