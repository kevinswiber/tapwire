# Task 001: Session Notes & Quick Reference

**Created:** August 5, 2025  
**Purpose:** Quick reference for continuing implementation across Claude sessions

## Current Status

- ✅ Detailed implementation plan created
- ✅ Comprehensive testing strategy documented
- ✅ Complete code examples provided
- ⏳ Ready to begin implementation

## Quick Start Commands

```bash
# Navigate to shadowcat directory
cd /Users/kevin/src/tapwire/shadowcat

# Verify clean state
git status
cargo test

# Start implementation
cargo add hex ring  # Step 1
```

## Implementation Checklist

### Step 1: Dependencies (30 min)
- [ ] Add `hex = "0.4"` to Cargo.toml
- [ ] Add `ring = "0.17"` to Cargo.toml
- [ ] Run `cargo check` to verify
- [ ] Commit: "feat(deps): add hex and ring for secure session IDs"

### Step 2: Error Handling (30 min)
- [ ] Add `ReverseProxyError` enum to error.rs
- [ ] Add HTTP status mapping functions
- [ ] Add unit tests for error mapping
- [ ] Run `cargo test error::reverse_proxy_error_tests`
- [ ] Commit: "feat(error): add reverse proxy error types and HTTP mapping"

### Step 3: HTTP MCP Transport (1.5 hrs)
- [ ] Create `src/transport/http_mcp.rs`
- [ ] Implement `generate_secure_session_id()`
- [ ] Implement `extract_mcp_headers()`
- [ ] Implement `HttpMcpTransport` with Transport trait
- [ ] Update `src/transport/mod.rs` exports
- [ ] Run `cargo test transport::http_mcp`
- [ ] Commit: "feat(transport): add MCP Streamable HTTP server transport"

### Step 4: Reverse Proxy Core (2 hrs)
- [ ] Implement `src/proxy/reverse.rs`
- [ ] Create `ReverseProxyServer` struct
- [ ] Create `ReverseProxyConfig` struct
- [ ] Implement router creation with endpoints
- [ ] Add request handlers (mcp, health, metrics)
- [ ] Run `cargo test proxy::reverse`
- [ ] Commit: "feat(proxy): implement reverse proxy server core"

### Step 5: Router & Handlers (integrated with Step 4)
- [ ] Implement `handle_mcp_request`
- [ ] Implement `handle_health`
- [ ] Implement `handle_metrics`
- [ ] Add JSON-RPC parsing functions
- [ ] Add session integration

### Step 6: Configuration & CLI (1 hr)
- [ ] Create `src/config/reverse_proxy.rs`
- [ ] Implement configuration structures
- [ ] Update `src/config/mod.rs`
- [ ] Update `src/main.rs` with reverse proxy command
- [ ] Run `cargo run -- reverse-proxy --help`
- [ ] Commit: "feat(cli): add reverse proxy command and configuration"

### Step 7: Integration Tests (1.5 hrs)
- [ ] Create `tests/integration/reverse_proxy_basic.rs`
- [ ] Write server lifecycle tests
- [ ] Write endpoint tests
- [ ] Write error scenario tests
- [ ] Run `cargo test --test reverse_proxy_basic`
- [ ] Commit: "test: add comprehensive integration tests for reverse proxy"

### Step 8: Benchmarks (1 hr)
- [ ] Create `benches/reverse_proxy_bench.rs`
- [ ] Update Cargo.toml with bench configuration
- [ ] Run `cargo bench --bench reverse_proxy_bench`
- [ ] Verify < 1ms HTTP overhead
- [ ] Commit: "test: add performance benchmarks for reverse proxy"

## Key Files to Reference

1. **Implementation Plan**: `/plans/tasks/reverse-proxy/001-implementation-plan.md`
2. **Testing Strategy**: `/plans/tasks/reverse-proxy/001-testing-strategy.md`
3. **Code Examples**: `/plans/tasks/reverse-proxy/001-code-examples.md`
4. **Task Specification**: `/plans/tasks/reverse-proxy/001-axum-http-server-setup.md`

## Architecture Reminders

### Session ID Format
- Use `ring::rand::SystemRandom` for cryptographic security
- Generate 256 bits of entropy
- Convert to UUID v4 for compatibility with existing `SessionId` type

### MCP Headers
- **Required**: `MCP-Session-Id`, `MCP-Protocol-Version`
- **Optional**: `MCP-Client-Info`
- Support versions: `2025-11-05` (current), `2025-06-18` (Streamable HTTP)

### Error Handling
- All errors implement proper HTTP status mapping
- JSON-RPC error responses follow MCP spec
- Use structured logging with `tracing`

### Integration Points
- Use existing `SessionManager` from Phase 4
- Use existing `TransportMessage` types
- Maintain backward compatibility

## Testing Reminders

### Unit Test Pattern
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature() {
        // Test implementation
    }
}
```

### Integration Test Pattern
```rust
#[tokio::test]
async fn test_async_feature() {
    let (addr, _handle) = start_test_server().await;
    // Test implementation
}
```

### Manual Testing
```bash
# Start server
cargo run -- reverse-proxy --bind 127.0.0.1:8080 --debug

# Test endpoints
curl http://localhost:8080/health
curl -X POST http://localhost:8080/mcp -H "MCP-Session-Id: test" -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":"1","method":"test"}'
```

## Common Issues & Solutions

1. **Port Already in Use**
   - Use dynamic port allocation in tests
   - Kill existing process: `lsof -i :8080 | grep LISTEN`

2. **Missing Dependencies**
   - Run `cargo update`
   - Check for typos in Cargo.toml

3. **Test Timeouts**
   - Increase timeout in async tests
   - Check for deadlocks in session manager

## Next Session Quick Start

```bash
# Restore context
cd /Users/kevin/src/tapwire/shadowcat
git status
cargo test

# Check todo list
# Review this file for current step
# Continue from last completed checkbox
```

## Performance Targets
- HTTP overhead: < 1ms
- Memory per connection: < 2KB
- Concurrent connections: 1000+
- Session lookup: < 100μs

## Commit Message Format
```
feat(component): description
test(component): description
docs(component): description
fix(component): description
```

This quick reference ensures seamless continuation of Task 001 implementation across Claude sessions.