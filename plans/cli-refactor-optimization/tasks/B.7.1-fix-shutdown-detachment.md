# Task B.7.1: Fix Shutdown Task Detachment

**Status**: ⬜ Not Started  
**Estimated Duration**: 1 hour  
**Dependencies**: B.2 (Graceful Shutdown), B.3 (High-Level API)  
**Priority**: HIGH  

## Context

From the [Comprehensive Review](../../../reviews/cli-refactor-optimization/comprehensive-review.md#41-high-priority), the `ForwardProxyHandle::shutdown()` method spawns a detached task that could outlive the handle, leading to non-deterministic shutdown behavior.

## Problem

Current implementation in `src/api.rs:447-461`:
```rust
pub async fn shutdown(mut self) -> Result<()> {
    let controller = self.shutdown_controller.take();
    
    if let Some(controller) = controller {
        // PROBLEM: This spawns a detached task
        tokio::spawn(async move {
            if let Ok(ctrl) = Arc::try_unwrap(controller) {
                let _ = ctrl.shutdown(Duration::from_secs(30)).await;
            }
        });
    }
    self.wait().await
}
```

The detached task might:
- Continue running after the handle is dropped
- Not complete shutdown before the program exits
- Lead to resource leaks

## Solution

Await the shutdown directly instead of spawning a task:

```rust
pub async fn shutdown(mut self) -> Result<()> {
    if let Some(controller) = self.shutdown_controller.take() {
        // Await the shutdown directly
        if let Ok(ctrl) = Arc::try_unwrap(controller) {
            ctrl.shutdown(Duration::from_secs(30)).await?;
        }
        // If we can't unwrap, there are other references and shutdown is already happening
    }
    self.wait().await
}
```

## Implementation Steps

1. [ ] Open `src/api.rs` and locate `ForwardProxyHandle::shutdown()`
2. [ ] Remove the `tokio::spawn` wrapper
3. [ ] Await the shutdown directly
4. [ ] Ensure error propagation is correct
5. [ ] Run existing shutdown tests to verify behavior
6. [ ] Add a test for deterministic shutdown if missing

## Testing

- [ ] Run `cargo test shutdown` to verify existing tests pass
- [ ] Run `cargo test integration_api` to ensure API tests pass
- [ ] Manually test with Ctrl+C to verify graceful shutdown

## Success Criteria

- [ ] No detached tasks in shutdown path
- [ ] Shutdown completes deterministically
- [ ] All existing tests pass
- [ ] No clippy warnings