# H.0: Fix Connection Pool Resource Leak

**Priority**: 🔴 CRITICAL  
**Duration**: 2 hours  
**Status**: ✅ Completed  

## Outcome

The connection pool lifecycle and return path have been fixed.

Key changes (see `shadowcat/src/proxy/pool.rs`):
- Inner-Arc pattern with last-reference Drop gating (prevents premature shutdown on clone drop)
- Maintenance loop owns the return `mpsc::Receiver<T>` and consumes first tick to avoid select bias
- Drop-to-return error path: on `try_send` failure, extract the connection and `close().await` with a timeout, then decrement active
- Idle cleanup avoids awaits while holding locks (drain, process off-lock, repopulate)
- Last-reference Drop spawns async cleanup backstop: notify shutdown, await maintenance handle with timeout, and close idle connections

```rust
// CURRENT (BROKEN)
impl<T> Drop for PooledConnection<T> {
    fn drop(&mut self) {
        if let Some(connection) = self.connection.take() {
            let _ = self.pool.return_tx.send(connection); // Silent failure!
        }
    }
}
```

## Impact

- Connections lost if return channel is full or closed
- Semaphore count becomes incorrect
- Pool exhaustion under load (6.4MB/minute leak rate)
- Production outage risk: HIGH

## Verification

- Reuse test passes (persistent stdio servers reuse a single connection with `max_connections=1`)
- Backpressure path closes connections and decrements active; no leaks observed
- Maintenance loop continues to run and process returns; shutdown cleans up idle

## Testing

### Unit Test
```rust
#[tokio::test]
async fn test_connection_pool_leak_prevention() {
    let pool = create_test_pool(max_size: 10);
    
    // Acquire all connections
    let connections: Vec<_> = (0..10)
        .map(|_| pool.acquire().await.unwrap())
        .collect();
    
    // Drop the pool's return channel
    drop(pool.return_tx);
    
    // Drop all connections (should clean up properly)
    drop(connections);
    
    // Wait for cleanup
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify semaphore permits were restored
    assert_eq!(pool.semaphore.available_permits(), 10);
}
```

### Load Test
```bash
# Monitor for leaks under heavy load
for i in {1..10000}; do
    curl http://localhost:8080/test &
done
wait

# Check memory usage remains stable
```

## Success Criteria

- [ ] No connection leaks under channel failure
- [ ] Semaphore count remains accurate
- [ ] Memory usage stable under load
- [ ] Unit tests pass
- [ ] Load test shows no leak over 1 hour

## Artifacts

- Implementation: `shadowcat/src/proxy/pool.rs`
- Analysis: `reviews/stdio-connection-pool-analysis.md`

## Dependencies

None - this is the highest priority fix

## Notes

- Must handle both sync and async cleanup paths
- Consider using `Arc<AtomicUsize>` for active count instead of fetch_sub
- Monitor production metrics after deployment
