# Next Session Prompt - Reverse Proxy Final Refactor

## Context
Continue the Shadowcat reverse proxy refactor after successfully fixing the JSON-RPC ID type preservation issue. The SSE streaming now works with MCP Inspector!

## Current Status
✅ **FIXED**: JSON-RPC ID type preservation (numeric IDs stay numeric)
✅ **WORKING**: SSE streaming with MCP Inspector 
✅ **PROVEN**: Integration test passes
✅ **IMPLEMENTED**: Hyper-based HTTP client for direct body control

## Key Achievement (2025-08-15)
Fixed the "No connection established for request ID: 0" error by preserving JSON-RPC ID types:
- Changed `ProtocolMessage` ID fields from `String` to `serde_json::Value`
- Request with `"id": 0` now returns response with `"id": 0` (not `"id": "0"`)
- MCP Inspector can now correctly correlate requests and responses

## Tasks for Next Session

### 1. JSON-RPC ID Type Refactor (2 hours)
**Current**: Using `serde_json::Value` for IDs (quick fix)
**Goal**: Create proper type-safe ID representation

- [ ] Create `JsonRpcId` enum:
  ```rust
  pub enum JsonRpcId {
      String(String),
      Number(i64),  // or serde_json::Number
  }
  ```
- [ ] Update `ProtocolMessage` to use `JsonRpcId` instead of `Value`
- [ ] Use `Option<JsonRpcId>` where null IDs are valid (notifications)
- [ ] Update all parsing/serialization to use the new type
- [ ] Fix all tests to work with new ID type

### 2. Complete Hyper Migration for SSE (2 hours)
**Goal**: Replace reqwest with hyper for upstream SSE connections

- [ ] Review `src/proxy/reverse/hyper_client.rs` implementation
- [ ] Ensure all SSE paths use hyper client
- [ ] Remove reqwest dependencies from SSE code paths
- [ ] Test with various SSE scenarios
- [ ] Verify connection pooling and keepalive work correctly

### 3. Legacy.rs Refactor - Phase C (3 hours)
**Goal**: Clean up the monolithic legacy.rs file

Current issues:
- `handle_admin_request()`: 876 lines (needs major refactor)
- Duplicate request anti-pattern for SSE
- Mixed concerns throughout

Tasks:
- [ ] Extract admin endpoints to separate module
- [ ] Clean up SSE detection and routing
- [ ] Remove duplicate request workaround
- [ ] Implement proper early SSE detection via Accept header
- [ ] Separate concerns into focused modules

### 4. Clean Up (1 hour)
- [ ] Remove unused modules: `sse_client.rs`, `sse_streaming.rs`, `sse_streaming_v2.rs`
- [ ] Clean up `hyper_streaming.rs` (replaced by `hyper_raw_streaming.rs`)
- [ ] Update module exports in `mod.rs`
- [ ] Update documentation with findings

## Key Files
- `src/transport/envelope.rs` - ProtocolMessage with Value IDs (needs refactor)
- `src/transport/http_utils.rs` - JSON-RPC parsing (needs ID type update)
- `src/proxy/reverse/legacy.rs` - Main refactor target
- `src/proxy/reverse/hyper_client.rs` - Hyper HTTP client
- `src/proxy/reverse/hyper_raw_streaming.rs` - Working SSE forwarder
- `plans/reverse-proxy-refactor/tracker.md` - Full history

## Test Commands
```bash
# Integration test
cargo test test_reverse_proxy_sse_streaming -- --ignored --nocapture

# Full test setup
# Terminal 1: Start upstream MCP server
cd ~/src/modelcontextprotocol/servers/everything && npm start

# Terminal 2: Start Shadowcat proxy
cd ~/src/tapwire/shadowcat
cargo run --release -- reverse --bind 127.0.0.1:8081 --upstream http://localhost:3001/mcp

# Terminal 3: Test with Inspector (now works!)
npx @modelcontextprotocol/inspector http://127.0.0.1:8081/mcp
```

## Success Criteria
1. Type-safe JSON-RPC ID handling with proper enum
2. Complete hyper migration for SSE connections
3. Clean, modular code structure (no more 800+ line functions)
4. All tests passing with new ID type
5. MCP Inspector continues to work

## Important Notes
- The ID type fix is working but needs proper type safety
- Don't break the working SSE streaming while refactoring
- Focus on maintainability and code organization
- Keep the tracker.md updated with progress