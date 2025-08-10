# Next Session: Phase 2 - Reverse Proxy Streamable HTTP

## Project Status Update

We are implementing SSE proxy integration with MCP message handling in Shadowcat, following the unified tracker at `plans/proxy-sse-message-tracker.md`.

**Current Status**: Ready to begin Phase 2  
**Phase 0**: 100% Complete ✅ (F.1-F.5 all done)  
**Phase 1**: 100% Complete ✅ (S.1-S.4 all done)  
**Phase 2**: 0% Complete - Ready to start

## Recent Accomplishments (This Session)

### Completed Phase 1! 🎉

#### S.3: Integrate with Forward Proxy ✅
- Implemented proper ForwardProxy integration with SSE transport
- Client uses stdio transport (spawned command)
- Server uses SSE transport (HTTP endpoint)
- Session manager integrated
- Rate limiting ready (through middleware)

#### S.4: Add MCP Parser Hooks to Transport ✅  
- Already completed in previous session
- Parser actively validates messages in send()
- Parser extracts info from received SSE events
- Debug logging for message types, IDs, and methods

## Phase 2: Reverse Proxy Streamable HTTP (Week 2)

Now we need to implement the `/mcp` endpoint with MCP message understanding for the reverse proxy.

### Tasks from Tracker

| ID | Task | Duration | Dependencies | Status | Notes |
|----|------|----------|--------------|--------|-------|
| R.1 | **Create MCP-Aware Dual-Method Endpoint** | 3h | F.1-F.3 | ⬜ Next | [Task 2.1](plans/sse-proxy-integration/tasks/task-2.1-dual-method-endpoint.md) |
| R.2 | Implement SSE Response Handler | 4h | R.1, F.4 | ⬜ Not Started | [Task 2.2](plans/sse-proxy-integration/tasks/task-2.2-sse-response-handler.md) |
| R.3 | Session-Aware SSE Streaming | 3h | R.2 | ⬜ Not Started | From SSE Task 2.3 |
| R.4 | **Add Early Message Correlation** | 2h | R.1, F.2 | ⬜ Not Started | Enhancement to reverse proxy |

**Phase 2 Total**: 12 hours

## Implementation Plan for R.1

### R.1: Create MCP-Aware Dual-Method Endpoint

The reverse proxy needs an `/mcp` endpoint that:
1. Accepts HTTP POST for client → server messages
2. Accepts HTTP GET with Accept: text/event-stream for SSE streaming
3. Manages sessions via headers
4. Validates MCP protocol version
5. Uses the MinimalMcpParser for early validation

Key files to modify:
- `src/proxy/reverse.rs` or create new `src/proxy/reverse/mcp_endpoint.rs`
- Integrate with existing ReverseProxyServer
- Use protocol version manager from F.1
- Use MinimalMcpParser from F.2
- Use batch handler from F.3 if needed

## Commands for Testing

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Test forward proxy with SSE (completed in Phase 1)
cargo run -- forward streamable-http --url http://localhost:8080/sse --enable-sse -- echo

# Build and run tests
cargo test --test sse_transport_test
cargo test reverse_proxy

# Check for warnings
cargo clippy --all-targets -- -D warnings
```

## Success Criteria for Phase 2

- ✅ `/mcp` endpoint handles both POST and GET requests
- ✅ SSE streaming works for server → client messages
- ✅ Sessions tracked via Mcp-Session-Id header
- ✅ Protocol version validated and negotiated
- ✅ Early message parsing with MinimalMcpParser
- ✅ Tests pass for reverse proxy scenarios

## Context from Tracker

From `plans/proxy-sse-message-tracker.md`:
- **Phase 0**: Foundation Components - 100% Complete ✅
- **Phase 1**: SSE Transport with MCP Awareness - 100% Complete ✅
- **Phase 2**: Reverse Proxy Streamable HTTP - 0% (Starting now)
- Foundation components (F.1-F.5) are all available for use
- SSE transport (S.1-S.4) is fully integrated

## Key Design Decisions from Phase 1

1. **Transport Architecture**: SSE transport implements the Transport trait cleanly
2. **Parser Integration**: MinimalMcpParser validates messages at transport layer
3. **Event ID Correlation**: UnifiedEventIdGenerator embeds correlation info
4. **Forward Proxy**: Successfully bridges stdio client to SSE server

## Notes for Next Session

- Start with R.1: Create the dual-method `/mcp` endpoint
- Review existing reverse proxy implementation first
- Ensure both forward and reverse proxy have feature parity
- Consider creating integration tests early in the phase
- Remember to handle both MCP protocol versions (2025-03-26 with batching, 2025-06-18)

---

**Next Goal**: Complete Phase 2 by implementing the reverse proxy Streamable HTTP support with the `/mcp` endpoint handling both POST and GET+SSE.