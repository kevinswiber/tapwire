# H.0: Fix Connection Pool Resource Leak

**Priority**: 🔴 CRITICAL  
**Duration**: 2 hours  
**Status**: ⏳ Pending  

## Problem

The connection pool Drop implementation silently fails to return connections, causing resource leaks under load.

**Location**: `src/proxy/reverse/upstream/pool.rs:56-60`

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

## Solution

### Step 1: Fix Drop Implementation

```rust
impl<T: Send + 'static> Drop for PooledConnection<T> {
    fn drop(&mut self) {
        if let Some(connection) = self.connection.take() {
            let pool = self.pool.clone();
            let return_tx = self.pool.return_tx.clone();
            
            // Ensure connection is returned or properly cleaned up
            tokio::spawn(async move {
                if return_tx.send(connection).await.is_err() {
                    // Connection couldn't be returned, decrement active count
                    pool.active_connections.fetch_sub(1, Ordering::Relaxed);
                    pool.semaphore.add_permits(1);
                    
                    // Log for monitoring
                    tracing::warn!("Connection pool return failed, cleaned up resources");
                }
            });
        }
    }
}
```

### Step 2: Add Pool Method for Safe Return

```rust
impl<T> ConnectionPool<T> {
    pub(crate) fn ensure_return_or_cleanup(&self, connection: T) {
        let pool = self.clone();
        let return_tx = self.return_tx.clone();
        
        tokio::spawn(async move {
            if return_tx.send(connection).await.is_err() {
                pool.decrement_active().await;
                tracing::warn!("Pool return channel closed, decremented active count");
            }
        });
    }
    
    async fn decrement_active(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
        self.semaphore.add_permits(1);
    }
}
```

### Step 3: Change to Bounded Channel

```rust
// In ConnectionPool::new()
// Change from unbounded to bounded with backpressure
let (return_tx, return_rx) = tokio::sync::mpsc::channel(max_size * 2);
```

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

## Files to Modify

1. `src/proxy/reverse/upstream/pool.rs` - Fix Drop implementation
2. `tests/integration/pool_tests.rs` - Add leak prevention tests

## Dependencies

None - this is the highest priority fix

## Notes

- Must handle both sync and async cleanup paths
- Consider using `Arc<AtomicUsize>` for active count instead of fetch_sub
- Monitor production metrics after deployment