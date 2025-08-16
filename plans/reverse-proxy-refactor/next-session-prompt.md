# Next Session Prompt - Reverse Proxy Final Cleanup

## Context
Continue the Shadowcat reverse proxy refactor after successfully implementing the JsonRpcId type system. The proxy now correctly preserves JSON-RPC ID types (numeric stays numeric, string stays string) and MCP Inspector can connect!

## Current Status
✅ **COMPLETE**: JsonRpcId type system fully implemented
✅ **WORKING**: ID type preservation (0 stays 0, not "0")
✅ **PROVEN**: MCP Inspector connects and communicates
✅ **IMPLEMENTED**: Hyper-based HTTP client for SSE streaming

## Key Achievements
### 2025-08-15: JsonRpcId Type System
Successfully refactored entire codebase to use type-safe JsonRpcId enum:
- Created `src/transport/jsonrpc_id.rs` with proper enum type
- Updated `ProtocolMessage` to use `JsonRpcId` instead of `serde_json::Value`
- Fixed 100+ compilation errors across library and tests
- Result: MCP Inspector works correctly with numeric IDs!

### 2025-08-16: SSE Interceptor Support & Analysis
- Implemented `hyper_sse_intercepted.rs` for SSE with interceptor support
- Fixed remaining JsonRpcId compilation errors
- Completed comprehensive SSE module analysis (see `analysis/sse-module-consolidation.md`)
- Identified 5-6 modules for removal (66% code reduction opportunity)

## Remaining Tasks for Next Session

### 1. Clean Up Unused SSE Modules (1-2 hours)
**Goal**: Remove deprecated SSE implementations per analysis

⚠️ **See `analysis/sse-module-consolidation.md` for detailed plan**

Phase 1 - Remove eventsource-client approach:
- [ ] Remove call to `stream_sse_with_eventsource` at legacy.rs:1356
- [ ] Delete `src/proxy/reverse/sse_streaming_v2.rs`
- [ ] Delete `src/proxy/reverse/sse_client.rs`
- [ ] Delete `src/proxy/reverse/process_via_http_sse_aware.rs`
- [ ] Remove eventsource-client from Cargo.toml

Phase 2 - Remove reqwest approaches:
- [ ] Delete `src/proxy/reverse/sse_streaming.rs`
- [ ] Delete `src/proxy/reverse/process_via_http_hyper.rs`
- [ ] Review and likely delete `src/proxy/reverse/hyper_streaming.rs`
- [ ] Update `src/proxy/reverse/mod.rs` exports

### 2. Legacy.rs Refactor - Break Up Monolith (4-6 hours)
**Goal**: Modularize the 876-line `handle_admin_request()` function

Current issues:
- Massive single function with mixed concerns
- Hard to test and maintain
- Poor separation of responsibilities

Refactor plan:
- [ ] Extract admin dashboard rendering to `admin/dashboard.rs`
- [ ] Move session management endpoints to `admin/sessions.rs`
- [ ] Extract metrics endpoints to `admin/metrics.rs`
- [ ] Create `admin/static.rs` for static asset serving
- [ ] Implement proper routing in `admin/mod.rs`

### 3. Test Suite Updates (2 hours)
**Goal**: Fix remaining test compilation errors

- [ ] Update tests to use `JsonRpcId` constructors
- [ ] Fix test assertions expecting string IDs
- [ ] Update mock implementations
- [ ] Ensure all integration tests pass

### 4. Documentation Updates (1 hour)
**Goal**: Document the new architecture

- [ ] Update API documentation for JsonRpcId
- [ ] Document SSE streaming architecture
- [ ] Update proxy configuration examples
- [ ] Add troubleshooting guide for ID type issues

## Key Files
- `src/transport/jsonrpc_id.rs` - New JsonRpcId type implementation
- `src/proxy/reverse/legacy.rs` - Main refactor target (876-line function)
- `src/proxy/reverse/hyper_raw_streaming.rs` - Working SSE implementation
- `tests/` - Various test files needing JsonRpcId updates

## Test Commands
```bash
# Quick library build test
cargo build --lib

# Run specific integration test
cargo test test_reverse_proxy_sse_streaming -- --ignored --nocapture

# Full test suite
cargo test

# Test with MCP Inspector
# Terminal 1: Start upstream MCP server
cd ~/src/modelcontextprotocol/servers && npx tsx src/everything/index.ts streamableHttp

# Terminal 2: Start Shadowcat proxy
cargo run --release -- reverse --bind 127.0.0.1:8081 --upstream http://localhost:3001/mcp

# Terminal 3: Launch Inspector
npx @modelcontextprotocol/inspector http://127.0.0.1:8081/mcp
```

## Success Criteria
1. All deprecated SSE modules removed
2. Legacy.rs refactored into manageable modules (<500 lines each)
3. All tests compiling and passing
4. Documentation updated with new architecture

## Important Notes
- JsonRpcId refactor is complete and working - don't break it!
- MCP Inspector connectivity is verified
- Focus on code organization and maintainability
- Keep performance characteristics intact