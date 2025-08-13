# Next Session: Phase 1 - Foundation Design

## Project Context

We've completed Phase 0 analysis of the transport layer refactor. All current transport patterns have been documented, a comprehensive regression test suite has been created, and breaking change risks have been assessed.

**Project**: Transport Layer Refactor  
**Tracker**: `plans/transport-refactor/transport-refactor-tracker.md`  
**Status**: Phase 1 - Foundation (0% Complete)

## Current Status

### What Has Been Completed
- **Phase 0: Prerequisites and Analysis** (✅ Completed 2025-08-13)
  - A.1: Documented all transport patterns and architectural issues
  - A.2: Created 16 regression tests capturing current behavior
  - A.3: Comprehensive risk assessment with migration strategies
  - Performance baselines established

### Analysis Outputs Created
- `analysis/current-transport-architecture.md` - Complete architecture analysis
- `analysis/breaking-change-risk-assessment.md` - Risk assessment and mitigation
- `tests/transport_regression_suite.rs` - Regression test suite

### Key Findings
1. **Major Issues Confirmed**:
   - StdioTransport spawns processes (should be OutgoingTransport)
   - StdioClientTransport reads stdin (should be IncomingTransport)
   - No unified Streamable HTTP transport
   - Process management mixed with transport logic

2. **Performance Baselines**:
   - Message envelope creation: ~3.2µs
   - Transport lifecycle: ~2.4ms
   - Must maintain these targets

## Your Mission

Design the new transport trait hierarchy with clear separation of concerns.

### Priority 1: Foundation Tasks (11 hours total)

1. **F.1: Design RawTransport trait hierarchy** (2h)
   - Define bytes-only transport interface
   - No protocol knowledge at this layer
   - Support for both stream and message-based transports
   
2. **F.2: Design ProtocolHandler abstraction** (2h)
   - Extract MCP/JSON-RPC handling from transports
   - Define serialization/deserialization interface
   - Support for protocol version negotiation
   
3. **F.3: Design Incoming/Outgoing traits** (3h)
   - Clear directional interfaces
   - Compose RawTransport + ProtocolHandler
   - Define lifecycle methods
   
4. **F.4: Create ProcessManager trait** (2h)
   - Extract process spawning from StdioTransport
   - Define process lifecycle management
   - Support for process pools
   
5. **F.5: Design migration strategy** (2h)
   - Compatibility layer design
   - Deprecation approach
   - Type alias strategy

## Essential Context Files to Read

1. **Analysis Results**:
   - `plans/transport-refactor/analysis/current-transport-architecture.md`
   - `plans/transport-refactor/analysis/breaking-change-risk-assessment.md`
   
2. **Current Implementation**:
   - `shadowcat/src/transport/mod.rs` - Current Transport trait
   - `shadowcat/src/transport/envelope.rs` - MessageEnvelope (keep as-is)
   
3. **Test Suite**:
   - `shadowcat/tests/transport_regression_suite.rs` - Must continue passing

## Working Directory

```bash
cd /Users/kevin/src/tapwire/shadowcat
```

## Commands to Run First

```bash
# Verify regression tests still pass
cargo test --test transport_regression_suite

# Check current trait usage
rg "impl.*Transport" src/

# Review protocol coupling
rg "ProtocolMessage|JsonRpc" src/transport/
```

## Implementation Strategy

### Task F.1: RawTransport Design (2h)
Create `src/transport/raw/mod.rs`:
```rust
pub trait RawTransport: Send + Sync {
    async fn send_bytes(&mut self, data: &[u8]) -> Result<()>;
    async fn receive_bytes(&mut self) -> Result<Vec<u8>>;
    async fn close(&mut self) -> Result<()>;
}
```

### Task F.2: ProtocolHandler Design (2h)
Create `src/transport/protocol/mod.rs`:
```rust
pub trait ProtocolHandler {
    fn serialize(&self, msg: &ProtocolMessage) -> Result<Vec<u8>>;
    fn deserialize(&self, data: &[u8]) -> Result<ProtocolMessage>;
    fn version(&self) -> &ProtocolVersion;
}
```

### Task F.3: Direction-Aware Traits (3h)
Create `src/transport/directional/mod.rs`:
```rust
pub trait IncomingTransport {
    async fn accept(&mut self) -> Result<()>;
    async fn receive_request(&mut self) -> Result<MessageEnvelope>;
    async fn send_response(&mut self, response: MessageEnvelope) -> Result<()>;
}

pub trait OutgoingTransport {
    async fn connect(&mut self) -> Result<()>;
    async fn send_request(&mut self, request: MessageEnvelope) -> Result<()>;
    async fn receive_response(&mut self) -> Result<MessageEnvelope>;
}
```

### Task F.4: ProcessManager Design (2h)
Create `src/process/mod.rs`:
```rust
pub trait ProcessManager {
    async fn spawn(&mut self, command: Command) -> Result<ProcessHandle>;
    async fn terminate(&mut self, handle: ProcessHandle) -> Result<()>;
    fn is_alive(&self, handle: &ProcessHandle) -> bool;
}
```

### Task F.5: Migration Strategy (2h)
Create `src/transport/compat/mod.rs`:
- LegacyTransport wrapper implementing old Transport trait
- Type aliases for backward compatibility
- Deprecation attributes and warnings

## Success Criteria Checklist

- [ ] RawTransport trait defined and documented
- [ ] ProtocolHandler trait defined and documented
- [ ] IncomingTransport/OutgoingTransport traits defined
- [ ] ProcessManager trait extracted
- [ ] Migration strategy documented
- [ ] All designs reviewed against requirements
- [ ] Regression tests still pass
- [ ] Design document created for review

## Important Notes

- **DO NOT** modify existing transports yet - only design new traits
- **Keep MessageEnvelope** - it's a recent refactor that works well
- **Ensure backward compatibility** - design compatibility layer first
- **Document everything** - this is the foundation for all future work

## Key Design Decisions to Make

1. **Async vs Sync**: Which methods should be async?
2. **Error Types**: Unified error type or trait-specific?
3. **Lifecycle**: How to handle connection state?
4. **Buffering**: Where does buffering happen?
5. **Metadata**: How to pass transport-specific metadata?

## Next Steps After This Task

Once Phase 1 design is complete:
- **Phase 2**: Implement Raw Transport Layer (16h)
- **Phase 3**: Implement Protocol Handlers (7h)
- **Phase 4**: Implement Direction-Aware Transports (14h)

---

**Session Goal**: Complete foundation design with clear trait definitions that will guide the entire refactor.

**Last Updated**: 2025-08-13  
**Next Review**: After Phase 1 completion