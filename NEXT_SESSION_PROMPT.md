# Next Session: Complete Phase 2 and Begin Phase 3

## Project Status Update

We are implementing SSE proxy integration with MCP message handling in Shadowcat, following the unified tracker at `plans/proxy-sse-message-tracker.md`.

**Current Status**: Phase 2 nearly complete, ready to finish R.4 and begin Phase 3  
**Phase 0**: 100% Complete ✅ (F.1-F.5 all done)  
**Phase 1**: 100% Complete ✅ (S.1-S.4 all done)  
**Phase 2**: 75% Complete (R.1-R.3 done, R.4 remaining)

## Accomplishments This Session

### Phase 2: Reverse Proxy Streamable HTTP ✨

#### R.1: Create MCP-Aware Dual-Method Endpoint ✅
- Enhanced `/mcp` endpoint to support both POST and GET methods
- POST continues to handle JSON-RPC messages
- GET with Accept: text/event-stream opens SSE connection
- Added proper header validation and error responses

#### R.2: Implement SSE Response Handler ✅
- Created `handle_mcp_sse_request` function for SSE GET requests
- Implemented `proxy_sse_from_upstream` to proxy SSE from HTTP upstreams
- Handles SSE event parsing and forwarding
- Includes keepalive mechanism for connection health

#### R.3: Session-Aware SSE Streaming ✅
- Integrated session management with SSE streams
- Sessions validated via Mcp-Session-Id header
- Protocol version tracked and validated
- Proper cleanup on disconnect

### Tests Added
- Created `tests/test_reverse_proxy_sse.rs` with integration tests
- Tests verify dual-method endpoint behavior
- Tests confirm SSE response headers
- Tests validate Accept header requirements

## Remaining Work

### Phase 2: Complete R.4 (2 hours)

**R.4: Add Early Message Correlation**
- Integrate UnifiedEventIdGenerator with reverse proxy SSE
- Add correlation info to SSE event IDs
- Track request/response pairs through SSE streams
- Files to modify:
  - `src/proxy/reverse.rs` - Add event ID generation to SSE responses

### Phase 3: Full MCP Parser and Correlation (Week 3)

After completing R.4, begin Phase 3:

| ID | Task | Duration | Status |
|----|------|----------|--------|
| P.1 | **Create Full MCP Parser** | 6h | ⬜ Not Started |
| P.2 | Add Schema Validation | 4h | ⬜ Not Started |
| P.3 | Implement Correlation Store | 5h | ⬜ Not Started |
| P.4 | Add Request/Response Matching | 4h | ⬜ Not Started |
| P.5 | **Integrate with Proxy** | 5h | ⬜ Not Started |

## Commands for Testing

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Run SSE tests
cargo test test_reverse_proxy_sse
cargo test test_sse_get

# Test the reverse proxy manually
cargo run -- reverse --bind 127.0.0.1:8080

# In another terminal, test SSE endpoint
curl -H "Accept: text/event-stream" \
     -H "Mcp-Session-Id: test-123" \
     -H "MCP-Protocol-Version: 2025-06-18" \
     http://localhost:8080/mcp

# Check for warnings
cargo clippy --all-targets -- -D warnings
```

## Key Implementation Details

### Dual-Method `/mcp` Endpoint
- Located in `src/proxy/reverse.rs`
- Router configuration: `.route("/mcp", post(handle_mcp_request).get(handle_mcp_sse_request))`
- POST handler: `handle_mcp_request` - Returns JSON responses
- GET handler: `handle_mcp_sse_request` - Returns SSE stream

### SSE Proxy Implementation
- Function: `proxy_sse_from_upstream` in `src/proxy/reverse.rs`
- Uses `reqwest` streaming response parsing
- Parses SSE format: event:, data:, id: fields
- Forwards events through tokio channels
- **Important**: Uses upstream URLs as-is (no path assumptions)

### Session Integration
- Sessions created/retrieved for both POST and GET
- Protocol version validation enforced
- Session ID required in headers
- Proper cleanup on stream termination

## Success Criteria for Next Session

1. ✅ Complete R.4: Early Message Correlation
2. ✅ All Phase 2 tests passing
3. ✅ Begin Phase 3: Full MCP Parser (P.1)
4. ✅ Document parser design decisions
5. ✅ Update tracker with progress

## Context from Tracker

From `plans/proxy-sse-message-tracker.md`:
- **Phase 0**: Foundation Components - 100% Complete ✅
- **Phase 1**: SSE Transport with MCP Awareness - 100% Complete ✅
- **Phase 2**: Reverse Proxy Streamable HTTP - 75% Complete (R.4 remaining)
- **Phase 3**: Full MCP Parser and Correlation - 0% (Starting next)

## Notes

- The reverse proxy now fully supports the MCP Streamable HTTP transport
- Both stdio and HTTP upstreams are supported
- SSE streaming works with keepalive and proper connection management
- **Important**: Upstream HTTP URLs must be complete (including path) - MCP doesn't mandate any specific path
- Our reverse proxy exposes `/mcp`, but upstream URLs are used exactly as configured
- Early message correlation (R.4) will enhance debugging and tracing
- Phase 3 will add full MCP message parsing and validation

---

**Next Goal**: Complete R.4 to finish Phase 2, then begin Phase 3 with the full MCP parser implementation (P.1).