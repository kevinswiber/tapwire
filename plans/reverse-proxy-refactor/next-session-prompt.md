# Next Session: Phase D - Modularize the Reverse Proxy

## Context
We've successfully completed Phase C of the reverse proxy refactor. The SSE duplicate request bug has been **completely fixed**! The proxy now efficiently routes responses based on content type without unnecessary buffering or duplicate requests.

## Current Status
- **Phase A**: ✅ COMPLETE (5 hours) - Full analysis completed
- **Phase B**: ✅ COMPLETE (4 hours) - SessionStore abstraction implemented  
- **Phase C**: ✅ COMPLETE (6 hours) - SSE bug fixed, no more duplicate requests!
- **Phase D**: ⬜ Ready to start - Modularization (8-10 hours)
- **Phase E**: ⬜ Blocked on D - Integration & testing (4 hours)

## Key Architectural Achievements from Phase C

### 1. UpstreamResponse Pattern
Created `src/proxy/reverse/upstream_response.rs` that allows inspecting response headers WITHOUT consuming the body. This enables smart routing based on content type.

### 2. Content-Type Based Routing
The new `process_via_http_new()` function returns `UpstreamResponse` which routes to:
- **SSE**: `sse_streaming.rs` - Streams without buffering using existing `SseStream`
- **JSON**: `json_processing.rs` - Smart buffering based on Content-Length
- **202 Accepted**: Passthrough streaming without buffering
- **Unknown**: Attempts JSON parse with proper error handling

### 3. Module Structure Started
```
src/proxy/reverse/
├── mod.rs                    # Module coordinator with refactoring documentation
├── legacy.rs                 # The old 3,465-line file (to be eliminated)
├── upstream_response.rs      # ✅ Response wrapper for routing
├── sse_streaming.rs          # ✅ SSE streaming with interceptors
└── json_processing.rs        # ✅ JSON processing with smart buffering
```

## Important Discovery: Upstream SSE Behavior
During testing with MCP Inspector, we discovered the upstream server closes the SSE connection after sending a single event. This is **not a proxy bug** but an upstream configuration issue. The proxy correctly handles SSE streaming when the upstream keeps the connection open.

## Your Task: Phase D - Modularization

### Goal
Break up the 3,465-line `legacy.rs` file into manageable modules, each under 500 lines with single responsibility.

### Phase D Implementation Steps

#### D.0: Create Module Structure (2 hours)
1. Review the current `legacy.rs` structure
2. Create subdirectories for logical groupings:
   - `admin/` - Admin interface (876 lines to extract!)
   - `handlers/` - Request handlers
   - `upstream/` - Upstream management
3. Plan the extraction order to minimize disruption

#### D.1: Extract Admin Interface (3 hours)
**This is the biggest win - 876 lines!**

Create `admin/mod.rs` with:
- `handle_admin_request()` function
- HTML templates (currently inline strings)
- Admin-specific types and helpers
- Session management UI components

Files to create:
- `admin/mod.rs` - Main admin handler
- `admin/templates.rs` - HTML templates
- `admin/session_ui.rs` - Session management UI
- `admin/metrics_ui.rs` - Metrics display

#### D.2: Extract Request Handlers (2 hours)
Move core request handling logic:
- `handlers/mcp.rs` - Main MCP request handler (567 lines)
- `handlers/health.rs` - Health check endpoint
- `handlers/metrics.rs` - Metrics endpoint

#### D.3: Extract Upstream Management (2 hours)
Centralize upstream logic:
- `upstream/selection.rs` - Load balancing and selection
- `upstream/connection.rs` - Connection management
- `upstream/pool.rs` - Connection pooling

#### D.4: Extract Interceptor Integration (1 hour)
Move interceptor-specific code:
- `interceptors/integration.rs` - Chain integration
- `interceptors/context.rs` - Context creation

### Success Criteria for Phase D
- [ ] `legacy.rs` reduced to < 500 lines (or eliminated)
- [ ] Each new module < 500 lines
- [ ] Clear single responsibility per module
- [ ] All tests still passing
- [ ] No circular dependencies

### Testing Strategy
After each extraction:
1. Run `cargo build --release`
2. Run `cargo clippy --all-targets -- -D warnings`
3. Run `cargo test`
4. Test with a simple HTTP client to ensure functionality

### Key Files to Reference
- **Main tracker**: `plans/reverse-proxy-refactor/tracker.md`
- **Implementation guide**: `plans/reverse-proxy-refactor/analysis/unified-plan.md`
- **Current structure**: `src/proxy/reverse/legacy.rs` (3,465 lines)
- **New modules**: `src/proxy/reverse/*.rs`

### Important Notes
1. The refactoring uses an incremental approach - `legacy.rs` will shrink as we extract code
2. All exports currently come from `legacy.rs` via `mod.rs` re-exports
3. As we create new modules, we'll switch the exports in `mod.rs`
4. When `legacy.rs` has no more exports, we delete it

### Commands to Run
```bash
# Build and check
cargo build --release
cargo clippy --all-targets -- -D warnings

# Test
cargo test

# Run the proxy
./target/release/shadowcat reverse \
  --bind 127.0.0.1:8080 \
  --upstream http://localhost:3000
```

### Next Phase Preview (Phase E)
After Phase D is complete, Phase E will:
- Run comprehensive integration tests
- Benchmark performance improvements
- Update documentation
- Clean up any remaining technical debt

### Time Estimate
- Phase D: 8-10 hours
- Remaining work: 12-14 hours (D + E)
- Total project: ~27-29 hours (15 complete, 12-14 remaining)

## Questions?
The modularization strategy is clear and all architectural decisions have been made. Start with D.1 (Extract Admin Interface) as it provides the biggest immediate win by removing 876 lines from `legacy.rs`.