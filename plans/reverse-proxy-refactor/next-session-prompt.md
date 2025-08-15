# Next Session Prompt - Reverse Proxy SSE Completion

## Context
Continue work on the Shadowcat reverse proxy SSE streaming feature. The core functionality is WORKING - we successfully forward SSE data from upstream to clients. However, the MCP Inspector client specifically fails due to its own proxy layer.

## Current Status
✅ **WORKING**: SSE streaming with standard HTTP clients
✅ **PROVEN**: Integration test at `tests/test_reverse_proxy_sse.rs` passes
✅ **FIXED**: Double-encoding issue (now forwarding raw bytes)
✅ **IMPLEMENTED**: Hyper-based HTTP client for direct body control
❌ **FAILING**: MCP Inspector can't correlate responses (Inspector proxy issue)

## Key Files
- `src/proxy/reverse/hyper_client.rs` - Hyper HTTP client implementation
- `src/proxy/reverse/hyper_raw_streaming.rs` - Raw SSE forwarding (WORKING)
- `src/proxy/reverse/legacy.rs:2536` - `process_via_http_hyper_sse()` entry point
- `tests/test_reverse_proxy_sse.rs` - Integration test (PASSES)
- `plans/reverse-proxy-refactor/tracker.md` - Full history and details

## Inspector Error
```
Error: No connection established for request ID: 0
at StreamableHTTPServerTransport.send
```
The Inspector's proxy expects request/response correlation that we're not providing.

## Tasks for This Session

### 1. Clean Up Implementation (1 hour)
- [ ] Re-enable interceptors for SSE streams (currently bypassed)
- [ ] Remove unused modules: `sse_client.rs`, `sse_streaming.rs`, `sse_streaming_v2.rs`
- [ ] Clean up `hyper_streaming.rs` (currently unused, replaced by `hyper_raw_streaming.rs`)
- [ ] Update module exports in `mod.rs`

### 2. Inspector Compatibility (2 hours)
**Note**: There's a separate debugging session happening in the Inspector codebase.
See `~/src/modelcontextprotocol/inspector/INSPECTOR_DEBUG_PROMPT.md`

Based on findings from Inspector debugging:
- [ ] Implement any necessary workarounds for Inspector proxy
- [ ] Consider adding request ID tracking if needed
- [ ] Test with updated Inspector

### 3. Performance & Polish (1 hour)
- [ ] Add metrics for SSE streaming performance
- [ ] Ensure proper error handling for partial writes
- [ ] Test with large SSE streams
- [ ] Update documentation

## Test Commands
```bash
# Run our working test
cargo test test_reverse_proxy_sse_streaming -- --ignored --nocapture

# Start proxy
cargo run --release -- reverse --bind 127.0.0.1:8081 --upstream http://localhost:3001/mcp

# Start upstream MCP server (everything example)
cd ~/src/modelcontextprotocol/servers/everything && npm start

# Test with Inspector
npx @modelcontextprotocol/inspector http://127.0.0.1:8081/mcp
```

## Success Criteria
1. Integration test continues to pass
2. MCP Inspector can successfully connect through proxy
3. No duplicate headers in responses
4. Clean, maintainable code with interceptor support

## Important Notes
- The proxy IS working correctly for standard clients
- The issue is specific to MCP Inspector's proxy correlation
- Don't break what's working while fixing Inspector compatibility