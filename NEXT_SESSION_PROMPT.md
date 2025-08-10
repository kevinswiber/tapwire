# Next Session: Phase 1 - SSE Transport with MCP Awareness

## Project Status Update

We are implementing SSE proxy integration with MCP message handling capabilities in Shadowcat. The unified tracker (`plans/proxy-sse-message-tracker.md`) coordinates this work across 7 phases.

**Total Project**: 118-138 hours  
**Current Phase**: Phase 1 - SSE Transport with MCP Awareness (Week 1-2)  
**Phase 0 Complete**: ✅ All foundation tasks finished (11 hours)

### Phase 0 Completion Summary (✅ Complete)

All foundation components are now ready:

1. **F.1: Protocol Version Manager** (✅ 2025-08-08)
   - `src/mcp/protocol.rs` - Enum-based version management
   - 22 comprehensive tests

2. **F.2: Minimal MCP Parser** (✅ 2025-08-08)
   - `src/mcp/early_parser.rs` - Lightweight message parser
   - 37 comprehensive tests

3. **F.3: Batch Handler** (✅ 2025-08-08)
   - `src/mcp/batch.rs` - Complete batch handling
   - 18 comprehensive tests

4. **F.4: Event ID Generator** (✅ 2025-08-10)
   - `src/mcp/event_id.rs` - Thread-safe ID generation with correlation
   - 17 comprehensive tests

5. **F.5: Message Context** (✅ From Transport Refactor)
   - Already exists as `MessageContext` in `src/transport/envelope.rs`

## Phase 1 Tasks: SSE Transport with MCP Awareness

### Task Overview

| ID | Task | Duration | Dependencies | Priority |
|----|------|----------|--------------|----------|
| S.1 | Add SSE Transport CLI Option | 2h | None | 🔴 High |
| S.2 | Create MCP-Aware SSE Transport Wrapper | 4h | F.1-F.4, S.1 | 🔴 High |
| S.3 | Integrate with Forward Proxy | 3h | S.2 | 🟡 Medium |
| S.4 | Add MCP Parser Hooks to Transport | 2h | S.2, F.2 | 🟡 Medium |

**Total Phase 1**: 11 hours

### Recommended Session Plan

This phase can be completed in 2-3 focused sessions:

**Session 1 (3-4 hours)**: S.1 + Start S.2
- Add CLI option for SSE transport
- Begin SSE transport wrapper implementation

**Session 2 (4-5 hours)**: Complete S.2 + S.3
- Finish SSE transport wrapper
- Integrate with forward proxy

**Session 3 (2-3 hours)**: S.4 + Testing
- Add parser hooks
- Comprehensive testing

## Task S.1: Add SSE Transport CLI Option (2h)

### Objective
Add command-line option to enable SSE transport mode in the forward proxy.

### Essential Files
- `shadowcat/src/cli/mod.rs` - CLI argument parsing
- `shadowcat/src/config/mod.rs` - Configuration structures
- `shadowcat/src/main.rs` - Main entry point

### Implementation Requirements

1. **Update CLI arguments**:
   ```rust
   // In ForwardCommand
   #[arg(long, value_enum, default_value = "stdio")]
   transport: TransportType,
   
   #[derive(ValueEnum)]
   enum TransportType {
       Stdio,
       Sse,
   }
   ```

2. **Configuration updates**:
   - Add SSE-specific configuration options
   - Default SSE endpoint URL
   - Retry settings

3. **Tests**:
   - CLI parsing tests
   - Configuration validation

### Success Criteria
- [ ] CLI accepts `--transport sse` option
- [ ] Configuration properly stores transport type
- [ ] Help text documents new option
- [ ] Tests pass

## Task S.2: Create MCP-Aware SSE Transport Wrapper (4h)

### Objective
Implement SSE transport that understands MCP messages and uses the foundation components.

### Essential Files
- `shadowcat/src/transport/sse_transport.rs` (new)
- `shadowcat/src/transport/mod.rs` - Export new transport
- Foundation components:
  - `src/mcp/protocol.rs` - Version management
  - `src/mcp/early_parser.rs` - Message parsing
  - `src/mcp/event_id.rs` - Event ID generation
  - `src/transport/envelope.rs` - MessageEnvelope/Context

### Implementation Structure

```rust
pub struct SseTransport {
    url: Url,
    client: reqwest::Client,
    event_id_generator: UnifiedEventIdGenerator,
    parser: MinimalMcpParser,
    session_id: SessionId,
    // EventSource for receiving
}

impl Transport for SseTransport {
    async fn send(&mut self, envelope: MessageEnvelope) -> Result<()> {
        // 1. Parse message with MinimalMcpParser
        // 2. Generate event ID with correlation
        // 3. Format as SSE-compatible POST
        // 4. Send via HTTP client
    }
    
    async fn receive(&mut self) -> Result<MessageEnvelope> {
        // 1. Receive SSE event
        // 2. Parse with MinimalMcpParser
        // 3. Extract correlation from event ID
        // 4. Build MessageEnvelope with context
    }
}
```

### Key Integration Points

1. **Use MessageEnvelope** from transport refactor
2. **Parse with MinimalMcpParser** for early inspection
3. **Generate IDs with UnifiedEventIdGenerator**
4. **Track protocol version** with ProtocolVersion enum
5. **Set TransportContext::Sse** with proper fields

### Success Criteria
- [ ] Implements Transport trait
- [ ] Uses all foundation components
- [ ] Handles SSE event streams
- [ ] Proper error handling
- [ ] Comprehensive tests

## Task S.3: Integrate with Forward Proxy (3h)

### Objective
Wire the SSE transport into the forward proxy command flow.

### Essential Files
- `shadowcat/src/proxy/forward.rs`
- `shadowcat/src/transport/factory.rs` (may need creation)

### Implementation Steps

1. **Transport factory**:
   ```rust
   pub fn create_transport(config: &Config) -> Result<Box<dyn Transport>> {
       match config.transport_type {
           TransportType::Stdio => // existing
           TransportType::Sse => Box::new(SseTransport::new(config)?),
       }
   }
   ```

2. **Forward proxy integration**:
   - Use transport factory in forward proxy
   - Handle SSE-specific initialization
   - Manage SSE connection lifecycle

3. **Session management**:
   - Ensure session IDs flow properly
   - Track SSE connection state

### Success Criteria
- [ ] Forward proxy can use SSE transport
- [ ] Seamless switching between stdio and SSE
- [ ] Proper session management
- [ ] Integration tests pass

## Task S.4: Add MCP Parser Hooks to Transport (2h)

### Objective
Integrate the minimal parser more deeply into the SSE transport for message inspection.

### Essential Files
- `shadowcat/src/transport/sse_transport.rs`
- `shadowcat/src/mcp/early_parser.rs`

### Implementation Requirements

1. **Message inspection hooks**:
   - Pre-send parsing for correlation setup
   - Post-receive parsing for context enrichment
   - Batch detection and handling

2. **Correlation tracking**:
   - Track request IDs for response matching
   - Store pending correlations
   - Match responses to requests

3. **Metrics and logging**:
   - Log parsed message info
   - Track message types
   - Monitor correlation success

### Success Criteria
- [ ] Parser integrated at key points
- [ ] Correlation tracking works
- [ ] Batch messages handled correctly
- [ ] Logging provides visibility

## Important Context

### Available Foundation Components

All these are ready to use from Phase 0:

1. **MessageEnvelope/Context** (`src/transport/envelope.rs`)
   - Complete wrapper with transport metadata
   - Direction tracking (ClientToServer/ServerToClient)
   - TransportContext::Sse variant ready

2. **Protocol Version Manager** (`src/mcp/protocol.rs`)
   - ProtocolVersion enum with capability detection
   - Version negotiation helpers

3. **Minimal MCP Parser** (`src/mcp/early_parser.rs`)
   - Parse any MCP message
   - Extract type, method, ID
   - Batch detection

4. **Batch Handler** (`src/mcp/batch.rs`)
   - Split/combine batch messages
   - Version-aware batching

5. **Event ID Generator** (`src/mcp/event_id.rs`)
   - Thread-safe ID generation
   - Correlation embedding/extraction
   - SSE-compatible formatting

### Commands to Use

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Run tests as you implement
cargo test transport::sse
cargo test proxy::forward

# Check formatting and clippy
cargo fmt
cargo clippy --all-targets -- -D warnings

# Test the CLI
cargo run -- forward --transport sse -- echo-server

# Run all tests
cargo test
```

### Architecture Reminders

- SSE transport is just another Transport implementation
- Use MessageEnvelope for all message passing
- Parser is lightweight - full parsing comes in Phase 3
- Event IDs must work for both SSE and correlation
- Keep transport logic separate from MCP logic

## Next Steps After Phase 1

Once Phase 1 is complete, we'll move to **Phase 2: Reverse Proxy Streamable HTTP** (12 hours):

- R.1: Create MCP-Aware Dual-Method Endpoint (3h)
- R.2: Implement SSE Response Handler (4h)
- R.3: Session-Aware SSE Streaming (3h)
- R.4: Add Early Message Correlation (2h)

The reverse proxy work will reuse much of the SSE transport logic but in a server context.

## Success Metrics for Phase 1

- [ ] CLI accepts SSE transport option
- [ ] SSE transport fully implements Transport trait
- [ ] Forward proxy works with SSE transport
- [ ] All foundation components properly integrated
- [ ] Tests provide good coverage
- [ ] No clippy warnings
- [ ] Documentation updated

---

**Session Goal**: Complete Phase 1 SSE Transport implementation, enabling the forward proxy to use SSE as a transport mechanism with full MCP awareness using the foundation components from Phase 0.