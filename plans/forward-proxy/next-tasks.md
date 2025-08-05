# Forward Proxy Next Tasks & Roadmap

## Immediate Next Tasks (Priority: Medium)

### 1. HTTP-to-Stdio Bridge Implementation

**Goal:** Enable HTTP clients to connect to stdio-based MCP servers through the forward proxy.

**Usage Target:**
```bash
cargo run -- forward http --port 8080 --target stdio --command "my-mcp-server" "--args"
```

**Architecture:**
```
[HTTP MCP Client] --HTTP--> [Shadowcat Proxy :8080] --stdio--> [Stdio MCP Server Process]
```

**Implementation Tasks:**

#### A. CLI Extension (`src/main.rs`)
- Extend `ForwardTransport::Http` to support stdio targets
- Add `--command` argument for stdio process specification
- Update help text and validation

**Current CLI:**
```rust
#[derive(Subcommand)]
enum ForwardTransport {
    Http {
        #[arg(long, default_value = "8080")]
        port: u16,
        
        #[arg(long)]
        target: String,
    },
}
```

**Needed CLI:**
```rust
#[derive(Subcommand)]
enum ForwardTransport {
    Http {
        #[arg(long, default_value = "8080")]
        port: u16,
        
        #[arg(long)]
        target: String,
        
        // New: Optional command for stdio targets
        #[arg(long)]
        command: Option<Vec<String>>,
    },
}
```

#### B. HTTP-to-Stdio Handler (`src/main.rs`)
Create new request handler for stdio targets:

```rust
async fn handle_stdio_proxy_request(
    req: Request, 
    command: Arc<Vec<String>>
) -> Result<Response<Body>, StatusCode> {
    // 1. Parse HTTP request body as JSON-RPC
    // 2. Create/reuse stdio transport to process
    // 3. Send JSON-RPC message to stdio process
    // 4. Read response from stdio process
    // 5. Convert back to HTTP response
}
```

#### C. Process Management
- Decide on process lifecycle: per-request vs persistent
- Handle process failures and recovery
- Implement proper cleanup on server shutdown
- Consider connection pooling for multiple clients

#### D. Request/Response Conversion
- HTTP request body → JSON-RPC message
- Stdio response → HTTP response body
- Proper error handling for malformed messages
- Timeout handling for slow stdio processes

### 2. Enhanced Error Handling & Logging

**Goal:** Improve observability and debugging for the forward proxy.

**Tasks:**
- Add structured logging for request flows
- Implement proper error responses with MCP error format
- Add metrics collection (request count, latency, error rates)
- Improve error messages for common configuration issues

### 3. Configuration & Security

**Goal:** Make the forward proxy production-ready.

**Tasks:**
- Add configuration file support (YAML/TOML)
- Implement basic authentication for proxy access
- Add CORS headers for web client support
- TLS/HTTPS support for secure proxy connections
- Rate limiting to prevent abuse

## Future Enhancements (Priority: Low)

### 4. Multi-Target Support

**Goal:** Support multiple target servers with routing rules.

**Example:**
```bash
cargo run -- forward http --port 8080 --config proxy-config.yaml
```

**Config Example:**
```yaml
routes:
  - path: "/llm/*"
    target: "http://localhost:3001/mcp"
  - path: "/tools/*" 
    target: "stdio"
    command: ["python", "tools-server.py"]
  - path: "/files/*"
    target: "http://localhost:3002/mcp"
```

### 5. Load Balancing & High Availability

**Goal:** Support multiple backend servers for scaling.

**Features:**
- Round-robin load balancing
- Health checks for backend servers
- Failover support
- Connection pooling

### 6. WebSocket Support

**Goal:** Support WebSocket-based MCP servers.

**Architecture:**
```
[HTTP Client] --HTTP--> [Shadowcat] --WebSocket--> [WebSocket MCP Server]
```

## Implementation Guidance

### Current Codebase Understanding

**Key Files to Understand:**
- `src/transport/mod.rs` - Transport trait definition
- `src/transport/http.rs` - HTTP/SSE transport implementation
- `src/transport/stdio.rs` - Stdio transport implementation
- `src/main.rs` - CLI and forward proxy server
- `src/proxy/forward.rs` - Original transport-to-transport bridge (may be useful for reference)

**Key Patterns:**
- All transports implement the `Transport` trait
- Session management via `SessionId`
- Error handling with `ShadowcatError` types
- Async/await with Tokio runtime
- Axum for HTTP server functionality

### Testing Strategy

**Unit Tests:**
- Test HTTP-to-stdio message conversion
- Test process lifecycle management
- Test error handling paths

**Integration Tests:**
- End-to-end HTTP client → proxy → stdio server flows
- Test with real MCP servers and clients
- Performance testing under load

**Manual Testing:**
```bash
# Test HTTP-to-stdio bridge
cargo run -- forward http --port 8080 --target stdio --command "echo"
curl http://127.0.0.1:8080/ -d '{"jsonrpc":"2.0","method":"ping","id":1}' -H "Content-Type: application/json"

# Should see stdio process response converted to HTTP
```

### Key Challenges to Consider

1. **Process Lifecycle:** How long should stdio processes live? Per-request or persistent?
2. **Concurrency:** How to handle multiple HTTP clients connecting to single stdio process?
3. **Error Propagation:** How to translate stdio process errors to HTTP error responses?
4. **Performance:** Stdio processes may be slower than HTTP endpoints
5. **Resource Management:** Memory usage with many concurrent processes

### Development Tips

**Start Simple:**
1. Implement basic HTTP-to-stdio forwarding with per-request process spawning
2. Test with simple echo-style stdio servers
3. Add process reuse and connection pooling later

**Use Existing Code:**
- Reference `src/transport/stdio.rs` for process spawning patterns
- Use `src/main.rs:handle_proxy_request` as template for request handling
- Leverage existing error types and logging patterns

**Testing Commands:**
```bash
# Simple stdio echo server for testing
echo '{"jsonrpc":"2.0","id":"1","result":"pong"}' | cargo run -- forward http --port 8080 --target stdio --command "cat"

# Test with actual MCP server
cargo run -- forward http --port 8080 --target stdio --command "python" "my-mcp-server.py"
```

## Success Criteria for Next Phase

When HTTP-to-stdio bridge is complete:

- ✅ Can run: `cargo run -- forward http --port 8080 --target stdio --command "echo"`
- ✅ HTTP clients can connect to stdio MCP servers through proxy
- ✅ JSON-RPC messages properly converted between HTTP and stdio
- ✅ Process lifecycle managed appropriately
- ✅ Error handling works for process failures
- ✅ Performance remains acceptable (<10% overhead)
- ✅ All existing functionality (HTTP-to-HTTP, stdio-to-stdio) still works

This would make Shadowcat a complete forward proxy solution supporting all MCP transport combinations.