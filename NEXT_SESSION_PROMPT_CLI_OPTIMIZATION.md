# Next Session: Complete Integration Tests (B.6)

## Session Context

We've made excellent progress on the CLI refactor optimization! In the previous session, we:

1. ✅ **Completed B.2.1**: Fixed all critical technical debt with real proxy implementations
2. ✅ **Completed B.3**: Created a comprehensive library facade with handle types
3. 🔄 **Started B.6**: Created integration test structure that needs mock servers

## Current State
- **Branch**: `shadowcat-cli-refactor` (git worktree at `/Users/kevin/src/tapwire/shadowcat-cli-refactor`)
- **Tests**: 681 passing, clippy clean
- **Tracker**: `plans/cli-refactor-optimization/cli-refactor-tracker.md`
- **Progress**: Phase B is 75% complete (18 of 24 hours)

## Next Task: Complete B.6 Integration Tests (1 hour)

### What's Already Done

Created `tests/integration_facade.rs` with 8 test cases:
- Simple forward proxy
- Shutdown handling  
- Builder configuration
- Development vs Production configs
- Recording functionality
- Handle management
- Reverse proxy

### What Needs to Be Done

The tests are currently failing because they try to create real proxies with real commands. We need to:

1. **Create mock MCP servers** for testing
2. **Fix the test implementations** to use mocks
3. **Add more comprehensive test coverage**

### Suggested Mock Server Implementation

Here's a starting point for mock servers you can use:

```rust
// tests/common/mod.rs - Shared test utilities
use tokio::process::Command;
use std::io::Write;

/// Create a mock MCP server that responds with predefined messages
pub async fn create_mock_mcp_server() -> Result<Command, Box<dyn std::error::Error>> {
    // Use a simple script that acts as an MCP server
    let mut cmd = Command::new("sh");
    cmd.arg("-c")
       .arg(r#"
           # Simple mock MCP server
           read -r line
           echo '{"jsonrpc":"2.0","result":{"protocolVersion":"2025-11-05"},"id":"1"}'
           read -r line
           echo '{"jsonrpc":"2.0","result":{"status":"ok"},"id":"2"}'
       "#);
    cmd.stdin(std::process::Stdio::piped())
       .stdout(std::process::Stdio::piped())
       .stderr(std::process::Stdio::null());
    Ok(cmd)
}

/// Create a test HTTP server for reverse proxy tests
pub async fn start_test_http_server(port: u16) -> tokio::task::JoinHandle<()> {
    use axum::{Router, routing::post, Json};
    use serde_json::{json, Value};
    
    let app = Router::new()
        .route("/mcp", post(|Json(payload): Json<Value>| async move {
            // Echo back with a result
            Json(json!({
                "jsonrpc": "2.0",
                "result": {"echo": payload},
                "id": payload.get("id").cloned().unwrap_or(json!(1))
            }))
        }));
    
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    })
}
```

### Test Implementation Fixes

Update the tests to use mock servers:

```rust
#[tokio::test]
async fn test_facade_simple_forward_proxy() {
    // Use the mock MCP server command
    let mock_cmd = create_mock_mcp_server().await.unwrap();
    
    let shadowcat = Shadowcat::new();
    
    // Create a proper shutdown mechanism
    let (controller, token) = ShutdownController::new();
    
    // Start proxy with mock
    let handle = tokio::spawn(async move {
        // Instead of using echo, use our mock server
        shadowcat.forward_stdio(vec!["mock_mcp".to_string()], Some(token)).await
    });
    
    // Give it time to initialize
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Trigger shutdown
    controller.shutdown(Duration::from_secs(1)).await.unwrap();
    
    // Wait for completion
    let result = timeout(Duration::from_secs(2), handle).await;
    assert!(result.is_ok());
}
```

## Additional Test Cases to Add

1. **Error Handling Tests**
   - Test proxy behavior when server fails
   - Test timeout handling
   - Test invalid configuration

2. **Concurrent Proxy Tests**
   - Multiple proxies running simultaneously
   - Resource cleanup verification

3. **Transport-Specific Tests**
   - HTTP transport with real HTTP mock server
   - SSE transport testing
   - Mixed transport scenarios

## Success Criteria

After completing B.6:
- [ ] All integration tests pass reliably
- [ ] Mock servers properly simulate MCP protocol
- [ ] Tests cover happy path and error cases
- [ ] Tests run in < 10 seconds total
- [ ] No test pollution between test cases

## Commands to Start With

```bash
# Navigate to the worktree
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Review existing test file
cat tests/integration_facade.rs

# Create common test utilities module
mkdir -p tests/common
echo "pub mod mock_servers;" > tests/common/mod.rs

# Run specific test with output
cargo test --test integration_facade test_facade_simple_forward_proxy -- --nocapture

# Run all integration tests
cargo test --test integration_facade
```

## After B.6: Next Tasks

- **B.4**: Extract Transport Factory (3 hours) - Refine TransportFactory
- **B.5**: Standardize Error Handling (2 hours) - Improve error context
- **C.1**: Documentation (4 hours) - Document the facade API
- **C.2**: Config Files (3 hours) - Add TOML/YAML support

## Files Changed in This Session

Key files modified:
- `src/facade.rs` - Enhanced with HTTP forward, reverse proxy, handle types
- `src/lib.rs` - Exported handle types
- `src/cli/forward.rs` - Updated to use ForwardProxyHandle
- `examples/*.rs` - 4 new example programs
- `tests/integration_facade.rs` - New integration test suite
- `plans/cli-refactor-optimization/cli-optimization-tracker.md` - Updated progress

## Duration Estimate: 1-2 hours

Focus on getting the mock servers working first, then fix each test case. The goal is to have a reliable integration test suite that validates the facade API works correctly.

Good luck!