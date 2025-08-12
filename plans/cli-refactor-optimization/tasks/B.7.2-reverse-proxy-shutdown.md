# Task B.7.2: Implement Reverse Proxy Shutdown

**Status**: ⬜ Not Started  
**Estimated Duration**: 2 hours  
**Dependencies**: B.3 (High-Level API)  
**Priority**: HIGH  

## Context

From the [Comprehensive Review](../../../reviews/cli-refactor-optimization/comprehensive-review.md#41-high-priority), the `ReverseProxyHandle::shutdown()` method has a TODO and doesn't implement graceful shutdown.

## Problem

Current implementation in `src/api.rs:492-497`:
```rust
/// Shutdown the server gracefully
pub async fn shutdown(self) -> Result<()> {
    // TODO: Implement graceful shutdown
    // For now, just wait for completion
    self.wait().await
}
```

This means:
- No graceful connection draining
- Abrupt termination of active requests
- Poor user experience on shutdown

## Solution

Implement proper shutdown signaling for the Axum server:

```rust
pub async fn shutdown(mut self) -> Result<()> {
    // Send shutdown signal to the server
    if let Some(shutdown_task) = self.shutdown_task.take() {
        // Trigger shutdown
        shutdown_task.abort();
        
        // Wait for graceful termination with timeout
        match tokio::time::timeout(
            Duration::from_secs(30),
            self.server_task
        ).await {
            Ok(result) => result.map_err(|e| {
                ShadowcatError::Config(ConfigError::Invalid(
                    format!("Server task failed: {e}")
                ))
            })?,
            Err(_) => {
                warn!("Server shutdown timeout, forcing termination");
                self.server_task.abort();
                return Err(ShadowcatError::Config(ConfigError::Invalid(
                    "Server shutdown timeout".to_string()
                )));
            }
        }
    } else {
        self.wait().await
    }
}
```

## Implementation Steps

1. [ ] Review how the reverse proxy server is started in `src/proxy/reverse.rs`
2. [ ] Check if we're using `axum::Server::with_graceful_shutdown`
3. [ ] Implement proper shutdown signaling mechanism
4. [ ] Update `ReverseProxyHandle` to store shutdown signal
5. [ ] Implement the `shutdown()` method with:
   - [ ] Shutdown signal triggering
   - [ ] Graceful connection draining
   - [ ] Timeout handling
   - [ ] Force shutdown fallback
6. [ ] Update the builder to wire up shutdown signals

## Testing

- [ ] Create a test that starts a reverse proxy and shuts it down
- [ ] Verify active connections are drained gracefully
- [ ] Test timeout behavior with long-running requests
- [ ] Ensure no panics on multiple shutdown calls

## Success Criteria

- [ ] Graceful shutdown implemented
- [ ] Active connections drain properly
- [ ] Timeout enforced for shutdown
- [ ] No TODO comment remaining
- [ ] Integration test for shutdown scenario
- [ ] All existing tests pass