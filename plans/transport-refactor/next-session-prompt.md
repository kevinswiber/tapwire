# Next Session: Phase 4 - Direction-Aware Transports (5h)

## ✅ Phase 3 Complete!

### Phase 3 Achievements
- ✅ Enhanced McpProtocolHandler with full batch message support
- ✅ Implemented strict JSON-RPC 2.0 validation with error code checking
- ✅ Created protocol negotiation module with capability exchange
- ✅ Added EnvelopeProtocolHandler implementation
- ✅ 21 comprehensive protocol tests passing
- ✅ Zero clippy warnings, code formatted

### Key Additions
- **Batch message support**: serialize_batch/deserialize_batch methods
- **Protocol negotiation**: ProtocolNegotiator class with version/capability handling
- **Enhanced validation**: Error code ranges, method validation, strict mode
- **Metadata extraction**: Extract JSON-RPC metadata from messages
- **Protocol upgrader**: Future-proof upgrade path support

## Context
You're implementing a layered transport architecture for Shadowcat's MCP proxy. Phases 0-3 have successfully created the foundation, raw transport layer, and protocol handling layer.

## Current Status
- Phase 0: Prerequisites ✅ Complete
- Phase 1: Foundation Design ✅ Complete  
- Phase 2: Raw Transport Layer ✅ Complete
- Phase 3: Protocol Handler ✅ Complete (2025-08-13)
- Phase 4: Direction-Aware Transports 📋 Ready to start

## Next Tasks (Phase 4 - 5h total)

### D.1: Implement IncomingTransport types (2h)
- StdioIncoming using StdioRawIncoming
- HttpServerIncoming using HttpRawServer
- StreamableHttpIncoming using StreamableHttpRawTransport
- All should use McpProtocolHandler for serialization

### D.2: Implement OutgoingTransport types (2h)
- SubprocessOutgoing using StdioRawOutgoing
- HttpClientOutgoing using HttpRawClient  
- StreamableHttpOutgoing using StreamableHttpRawTransport
- Integrate with ProcessManager for subprocess handling

### D.3: Update proxy to use new transports (30m)
- Modify forward proxy to use new transport types
- Update transport factory to create appropriate types
- Ensure backward compatibility where needed

### D.4: Create direction-aware tests (30m)
- Test IncomingTransport implementations
- Test OutgoingTransport implementations
- Verify protocol handler integration
- Test end-to-end message flow

## Key Files
- Tracker: `plans/transport-refactor/transport-refactor-tracker.md`
- Directional base: `src/transport/directional/mod.rs`
- Raw transports: `src/transport/raw/*.rs` (all implemented)
- Protocol: `src/transport/protocol/mod.rs` (enhanced with batch/negotiation)
- Process manager: `src/process/mod.rs`

## Foundation Already in Place
From Phases 1-3:
- `RawTransport` trait with all implementations working
- `IncomingTransport` and `OutgoingTransport` traits defined
- `McpProtocolHandler` with batch support and validation
- `ProtocolNegotiator` for version/capability exchange
- Process management separated into ProcessManager
- 22 raw transport tests + 21 protocol tests passing

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

# Check raw transports still work
cargo test --test raw_transport_tests

# Verify no regressions
cargo test --test transport_regression_suite

# Check overall compilation
cargo check
```

## Session Goal
Implement the direction-aware transport layer that combines raw transports with protocol handling, providing clean IncomingTransport and OutgoingTransport abstractions for the proxy to use.

**Last Updated**: 2025-08-13 (End of Phase 3)
**Session Time**: Phase 4 estimated at 5 hours
**Next Phase**: Phase 5 - Migration and Cleanup (11h)