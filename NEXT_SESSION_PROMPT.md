# Next Session: Begin Phase 3 - Full MCP Parser and Correlation

## Project Status Update

We are implementing SSE proxy integration with MCP message handling in Shadowcat, following the unified tracker at `plans/proxy-sse-message-tracker.md`.

**Current Status**: Phase 2 Complete ✅, Ready to begin Phase 3
**Phase 0**: 100% Complete ✅ (F.1-F.5 all done)  
**Phase 1**: 100% Complete ✅ (S.1-S.4 all done)  
**Phase 2**: 100% Complete ✅ (R.1-R.4 all done)
**Phase 3**: 0% (Starting now)

## Accomplishments Previous Session

### Phase 2: Reverse Proxy Streamable HTTP ✅

#### R.4: Add Early Message Correlation ✅
- Integrated UnifiedEventIdGenerator into reverse proxy
- Enhanced SSE proxy to generate correlation IDs for all events
- Parse MCP messages from SSE data to extract JSON-RPC IDs
- Generate session-aware event IDs: `{session}-{node}-{json_rpc_id}-{counter}`
- Preserve upstream event IDs while adding correlation info
- All tests passing, no clippy warnings

## Phase 3: Full MCP Parser and Correlation (Week 3)

The next phase will implement comprehensive MCP message parsing and correlation:

| ID | Task | Duration | Status |
|----|------|----------|--------|
| P.1 | **Create Full MCP Parser** | 6h | ⬜ Not Started |
| P.2 | Add Schema Validation | 4h | ⬜ Not Started |
| P.3 | Implement Correlation Store | 5h | ⬜ Not Started |
| P.4 | Add Request/Response Matching | 4h | ⬜ Not Started |
| P.5 | **Integrate with Proxy** | 5h | ⬜ Not Started |

## Primary Task: P.1 - Create Full MCP Parser

### Objective
Build a comprehensive MCP message parser that can:
1. Parse all MCP message types (requests, responses, notifications)
2. Handle both protocol versions (2025-03-26 with batching, 2025-06-18)
3. Extract metadata for correlation and interception
4. Provide structured error handling

### Implementation Plan

1. **Create `src/mcp/parser.rs`**
   - Define comprehensive message structures
   - Implement parsing logic for all message types
   - Handle protocol version differences
   - Extract method names, IDs, parameters

2. **Message Type Definitions**
   ```rust
   pub enum McpMessage {
       Request { id: Value, method: String, params: Option<Value> },
       Response { id: Value, result: Option<Value>, error: Option<McpError> },
       Notification { method: String, params: Option<Value> },
   }
   ```

3. **Parser Interface**
   ```rust
   pub struct McpParser {
       protocol_version: ProtocolVersion,
   }
   
   impl McpParser {
       pub fn parse(&self, data: &Value) -> Result<McpMessage>;
       pub fn parse_batch(&self, data: &Value) -> Result<Vec<McpMessage>>;
       pub fn extract_metadata(&self, msg: &McpMessage) -> MessageMetadata;
   }
   ```

4. **Integration Points**
   - Use existing `ProtocolVersion` from `src/mcp/protocol.rs`
   - Build on `EarlyMcpParser` from `src/mcp/early_parser.rs`
   - Support batch handling from `src/mcp/batch.rs`

### Success Criteria

1. ✅ Parser handles all MCP message types
2. ✅ Supports both protocol versions
3. ✅ Comprehensive error handling
4. ✅ 20+ unit tests covering edge cases
5. ✅ Documentation with examples
6. ✅ Integration ready for proxies

## Commands for Development

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Create the parser module
touch src/mcp/parser.rs

# Run tests as you develop
cargo test mcp::parser

# Check for warnings
cargo clippy --all-targets -- -D warnings

# Run all tests
cargo test
```

## Key Context

### Available Foundation
From previous phases, we have:
- `ProtocolVersion` enum with capability checks
- `EarlyMcpParser` for basic parsing
- `BatchHandler` for batch message support
- `UnifiedEventIdGenerator` for correlation IDs
- `MessageEnvelope` and `TransportContext` from refactor

### MCP Protocol Requirements
- JSON-RPC 2.0 base protocol
- Version 2025-03-26: Supports batching
- Version 2025-06-18: No batching, requires version header
- Methods: initialize, initialized, tools/*, prompts/*, resources/*
- Error codes: -32700 (parse), -32600 (invalid), -32601 (method not found)

### Parser Design Decisions
1. **Stateful vs Stateless**: Make parser stateless for simplicity
2. **Validation Level**: Basic structural validation, leave semantic validation to higher layers
3. **Error Recovery**: Return errors for invalid messages, don't try to fix them
4. **Performance**: Optimize for common case (single messages), batch is rare

## Notes

- The parser is a critical component that will be used by interceptors, recorders, and correlation
- Focus on correctness and comprehensive testing
- Keep performance in mind but prioritize clarity
- This parser will replace the early parser once complete
- Consider streaming parsing for large messages in future iteration

---

**Next Goal**: Implement P.1 (Full MCP Parser) with comprehensive message type support and testing.