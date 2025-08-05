# Phase 5 Reverse Proxy - Continuation Prompt

## Context
I'm continuing implementation of the Shadowcat MCP proxy, specifically Phase 5 (Reverse Proxy & Authentication). The core reverse proxy functionality is 85% complete - it can accept HTTP requests and forward them to stdio-based MCP servers. Now I need to implement the remaining components.

## Current Status
**Completed (August 5, 2025):**
- ✅ HTTP server infrastructure (Axum-based)
- ✅ MCP-over-HTTP transport with header validation
- ✅ CLI integration (`shadowcat reverse --upstream "command"`)
- ✅ Actual proxy forwarding logic (replaced mocks with real implementation)
- ✅ Stdio upstream support (HTTP → stdio MCP server forwarding)
- ✅ Basic metrics and health endpoints

**Test the current implementation:**
```bash
cd /Users/kevin/src/tapwire/shadowcat
cargo run -- reverse --upstream "python3 test_mcp_echo.py"
# In another terminal:
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -H "MCP-Session-Id: test-123" \
  -H "MCP-Protocol-Version: 2025-11-05" \
  -d '{"jsonrpc":"2.0","id":"1","method":"test","params":{}}'
```

## Key Files to Review
1. **Implementation Status**: `/Users/kevin/src/tapwire/plans/tasks/reverse-proxy/001-implementation-status.md` - Shows 85% complete with detailed breakdown
2. **Session Notes**: `/Users/kevin/src/tapwire/plans/tasks/reverse-proxy/001-session-notes.md` - Has completed/remaining steps checklist
3. **Current Code**: 
   - `/Users/kevin/src/tapwire/shadowcat/src/proxy/reverse.rs` - Main reverse proxy implementation
   - `/Users/kevin/src/tapwire/shadowcat/src/main.rs` - CLI integration (see `run_reverse_proxy` function)

## Priority Tasks (in order)

### 1. Configuration Module (Priority 1)
Create `/Users/kevin/src/tapwire/shadowcat/src/config/reverse_proxy.rs` with:
- YAML configuration loading using serde_yaml
- Environment variable overrides
- Upstream server pool configuration
- TLS settings for HTTPS
- Update `src/config/mod.rs` exports

Example config structure to implement:
```yaml
reverse_proxy:
  bind_address: "127.0.0.1:8080"
  upstreams:
    - name: "primary"
      transport: "stdio"
      command: ["mcp-server", "--production"]
    - name: "secondary"
      transport: "http"
      url: "https://mcp.example.com"
  session:
    timeout_secs: 300
    max_sessions: 1000
  tls:
    enabled: false
    cert_path: "/path/to/cert.pem"
    key_path: "/path/to/key.pem"
```

### 2. HTTP Upstream Support (Priority 2)
Implement the `process_via_http` function in `src/proxy/reverse.rs` (currently returns "not implemented"):
- Use reqwest with connection pooling
- Handle MCP headers properly
- Support SSE for streaming responses
- Match the pattern used in `process_via_stdio`

### 3. Connection Pooling (Priority 3)
Optimize performance by reusing connections:
- Design generic pool interface for multiple transport types
- Implement stdio process pool (currently creates new process per request)
- Add health checks and retry logic
- Target: reduce latency from current ~26ms

### 4. Integration Tests (Priority 4)
Create `tests/integration/reverse_proxy_basic.rs`:
- Server lifecycle tests
- Concurrent request handling
- Error scenarios
- Reference the test strategy in `/Users/kevin/src/tapwire/plans/tasks/reverse-proxy/001-testing-strategy.md`

## Important Notes
- The reverse proxy is separate from the forward proxy (different use cases)
- Current performance: ~26ms average latency (needs optimization)
- All 133+ tests are passing
- Session management and metrics are already integrated
- Error handling uses the existing `ReverseProxyError` type

## Architecture Context
- **Forward Proxy** (Phases 1-4): Developer tool for intercepting MCP traffic
- **Reverse Proxy** (Phase 5): Production API gateway that will add OAuth 2.1 authentication
- Both use the same core infrastructure (Transport traits, SessionManager, etc.)

Please help me continue the implementation starting with the configuration module. The goal is to make the reverse proxy production-ready before adding authentication.