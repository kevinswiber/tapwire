# Next Session: Continue Phase 3 - Message Builder and Correlation

## Project Status Update

We are implementing SSE proxy integration with MCP message handling in Shadowcat, following the unified tracker at `plans/proxy-sse-message-tracker.md`.

**Current Status**: Phase 3 in progress  
**Phase 0**: 100% Complete ✅ (F.1-F.5 all done)  
**Phase 1**: 100% Complete ✅ (S.1-S.4 all done)  
**Phase 2**: 100% Complete ✅ (R.1-R.4 all done)  
**Phase 3**: 31% Complete (M.1 ✅, M.2 ✅, M.3-M.5 pending)

## Accomplishments Previous Session

### Phase 3: Full MCP Parser ✅

Successfully implemented M.1 and M.2:
- M.1: Created complete MCP message types in `src/mcp/parser.rs`
- M.2: Implemented full message parser with:
  - `McpMessage` enum with Request, Response, and Notification variants
  - `McpParser` with support for both protocol versions (2025-03-26 and 2025-06-18)
  - Batch message handling for 2025-03-26
  - `MessageMetadata` for correlation tracking
  - Helper functions for streaming detection, session extraction, and correlation
  - 22+ comprehensive unit tests covering all edge cases
  - All tests passing, no clippy warnings

The parser provides:
- Full JSON-RPC 2.0 compliance
- Protocol version awareness
- Batch message support (2025-03-26 only)
- Error response handling with standard error codes
- Metadata extraction for correlation
- Helper functions for common operations

## Phase 3: Full MCP Parser and Correlation (Week 3)

Remaining tasks for Phase 3 (per tracker):

| ID | Task | Duration | Status |
|----|------|----------|--------|
| M.3 | **Message Builder API** | 2h | ⬜ Not Started |
| M.4 | **Correlation Engine** | 5h | ⬜ Not Started |
| M.5 | **Wire Correlation to SSE Transport** | 2h | ⬜ Not Started |

## Primary Task: M.3 - Message Builder API

### Objective
Create a fluent builder API for constructing MCP messages programmatically.

### Implementation Plan

1. **Create `src/mcp/builder.rs`**
   ```rust
   pub struct MessageBuilder {
       version: ProtocolVersion,
   }
   
   impl MessageBuilder {
       pub fn request(method: &str) -> RequestBuilder;
       pub fn response(id: JsonRpcId) -> ResponseBuilder;
       pub fn notification(method: &str) -> NotificationBuilder;
       pub fn error_response(id: JsonRpcId, code: i32, message: &str) -> McpMessage;
   }
   ```

2. **Fluent Builder Pattern**
   ```rust
   let msg = MessageBuilder::request("tools/call")
       .with_id("123")
       .with_param("name", "search")
       .with_param("arguments", json!({"query": "test"}))
       .build()?;
   ```

3. **Builder Types**
   - `RequestBuilder` for building request messages
   - `ResponseBuilder` for building response messages  
   - `NotificationBuilder` for building notifications
   - Support for all MCP methods with type-safe parameter building

4. **Integration Points**
   - Ensure built messages can be parsed by `McpParser`
   - Validate against protocol version
   - Support both 2025-03-26 and 2025-06-18 versions

### Success Criteria

1. ✅ Fluent builder API for all message types
2. ✅ Parameter validation
3. ✅ Protocol version awareness
4. ✅ 10+ unit tests
5. ✅ Documentation with examples

## Secondary Task: M.4 - Implement Correlation Engine

If time permits, begin work on the correlation engine:

1. **Create `src/mcp/correlation.rs`**
   ```rust
   pub struct CorrelationEngine {
       pending: HashMap<JsonRpcId, PendingRequest>,
       timeout: Duration,
   }
   ```

2. **Core Features**
   - Track request/response pairs by ID
   - Handle timeout and cleanup
   - Support concurrent correlations
   - Provide correlation statistics
   - Thread-safe with async support

## Task M.5: Wire Correlation to SSE Transport

The final task will integrate the correlation engine with SSE transport:
- Add correlation engine to `SseTransport`
- Track outgoing requests automatically
- Match incoming responses
- Generate correlation metrics

## Commands for Development

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Create the builder module
touch src/mcp/builder.rs

# Run tests as you develop
cargo test mcp::builder
cargo test mcp::correlation

# Check for warnings
cargo clippy --all-targets -- -D warnings

# Run all tests
cargo test
```

## Key Context

### Available Foundation
From M.1 and M.2 implementation:
- `McpMessage` enum with all message types
- `McpParser` for parsing messages
- `MessageMetadata` for tracking
- Helper functions for common operations
- Protocol version management

### MCP Methods to Support in Builder
1. **Lifecycle**: initialize, initialized, ping
2. **Tools**: tools/list, tools/call
3. **Prompts**: prompts/list, prompts/get
4. **Resources**: resources/list, resources/read, resources/subscribe, resources/unsubscribe
5. **Completion**: completion/complete
6. **Logging**: logging/setLevel
7. **Notifications**: error, progress, etc.

## Notes

- M.3 (Builder) is prerequisite for easier testing in M.4
- M.4 (Correlation) is the most complex task - allocate full 5 hours
- M.5 (Wiring) should be straightforward once M.4 is complete
- Keep thread safety in mind for correlation engine
- Consider using tokio::sync::RwLock for async access

---

**Next Goal**: Implement M.3 (Message Builder API) with fluent interface for all message types.