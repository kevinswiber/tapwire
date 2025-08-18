# H.6: Restore Buffer Pooling

**Priority**: 🟡 HIGH  
**Duration**: 2 hours  
**Status**: ⏳ Pending  

## Problem

Buffer pooling was removed in the SSE streaming path, causing:
- 2x memory usage per SSE connection
- >10,000 allocations/sec at 10K QPS (10x increase)
- Unnecessary GC pressure
- Performance degradation

## Current Issues

1. Every SSE frame allocates new buffer
2. No reuse of buffers between requests
3. Double buffering in SSE path (raw + intercepted)

## Solution

### Step 1: Restore Global Buffer Pools

```rust
// src/proxy/reverse/buffer_pool.rs
use bytes::{Bytes, BytesMut};
use std::sync::Arc;
use parking_lot::Mutex;

/// Global buffer pools for reverse proxy
pub mod global_pools {
    use super::*;
    
    lazy_static! {
        /// Pool for SSE event buffers (4KB default)
        pub static ref SSE_EVENT_POOL: BytesPool = BytesPool::new(
            4096,  // buffer size
            1000,  // max pooled buffers
        );
        
        /// Pool for HTTP response buffers (16KB default)
        pub static ref HTTP_RESPONSE_POOL: BytesPool = BytesPool::new(
            16384,
            500,
        );
        
        /// Pool for JSON serialization (8KB default)
        pub static ref JSON_POOL: BytesPool = BytesPool::new(
            8192,
            500,
        );
    }
}

pub struct BytesPool {
    pool: Arc<Mutex<Vec<BytesMut>>>,
    buffer_size: usize,
    max_pooled: usize,
    metrics: PoolMetrics,
}

impl BytesPool {
    pub fn new(buffer_size: usize, max_pooled: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(Vec::with_capacity(max_pooled))),
            buffer_size,
            max_pooled,
            metrics: PoolMetrics::default(),
        }
    }
    
    pub fn acquire(&self) -> BytesMut {
        let mut pool = self.pool.lock();
        
        if let Some(mut buf) = pool.pop() {
            buf.clear();
            self.metrics.reuse_count.fetch_add(1, Ordering::Relaxed);
            buf
        } else {
            self.metrics.alloc_count.fetch_add(1, Ordering::Relaxed);
            BytesMut::with_capacity(self.buffer_size)
        }
    }
    
    pub fn release(&self, mut buf: BytesMut) {
        // Don't pool oversized buffers
        if buf.capacity() > self.buffer_size * 2 {
            self.metrics.discard_count.fetch_add(1, Ordering::Relaxed);
            return;
        }
        
        let mut pool = self.pool.lock();
        if pool.len() < self.max_pooled {
            buf.clear();
            pool.push(buf);
        } else {
            self.metrics.discard_count.fetch_add(1, Ordering::Relaxed);
        }
    }
}
```

### Step 2: Update SSE Raw Stream to Use Pools

```rust
// src/proxy/reverse/upstream/http/streaming/raw.rs
use crate::proxy::reverse::buffer_pool::global_pools;

impl BodyForwarder {
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Use pooled buffer instead of allocating
        let mut buffer = global_pools::SSE_EVENT_POOL.acquire();
        
        match ready!(Pin::new(&mut self.body).poll_frame(cx)) {
            Some(Ok(frame)) => {
                if let Ok(data) = frame.into_data() {
                    // Copy data to pooled buffer
                    buffer.extend_from_slice(&data);
                    
                    let event = Event::default().data(
                        String::from_utf8_lossy(&buffer)
                    );
                    
                    // Return buffer to pool
                    global_pools::SSE_EVENT_POOL.release(buffer);
                    
                    Poll::Ready(Some(Ok(event)))
                } else {
                    // Trailers or non-data frame
                    global_pools::SSE_EVENT_POOL.release(buffer);
                    Poll::Pending
                }
            }
            Some(Err(e)) => {
                global_pools::SSE_EVENT_POOL.release(buffer);
                Poll::Ready(Some(Err(e.into())))
            }
            None => {
                global_pools::SSE_EVENT_POOL.release(buffer);
                Poll::Ready(None)
            }
        }
    }
}
```

### Step 3: Optimize Intercepted Stream Buffering

```rust
// src/proxy/reverse/upstream/http/streaming/intercepted.rs
impl InterceptedSseStream {
    pub fn new(...) -> Self {
        Self {
            // Use pooled buffer for pending events
            pending_buffer: global_pools::SSE_EVENT_POOL.acquire(),
            // ... other fields
        }
    }
    
    fn process_sse_event(&mut self, event: SseEvent) -> Result<()> {
        // Reuse buffer for JSON processing
        self.pending_buffer.clear();
        
        if let Some(data) = &event.data {
            // Parse JSON using pooled buffer
            let parsed: Value = serde_json::from_slice(data.as_bytes())?;
            
            // Process with interceptor...
            let processed = self.process_with_interceptor(parsed)?;
            
            // Serialize back to buffer
            serde_json::to_writer(&mut self.pending_buffer, &processed)?;
            
            // Create event with processed data
            let processed_event = SseEvent {
                id: event.id,
                event: event.event,
                data: String::from_utf8_lossy(&self.pending_buffer).to_string(),
                retry: event.retry,
            };
            
            self.pending_events.push(processed_event);
        }
        
        Ok(())
    }
}

impl Drop for InterceptedSseStream {
    fn drop(&mut self) {
        // Return buffer to pool
        let buffer = std::mem::replace(
            &mut self.pending_buffer,
            BytesMut::new()
        );
        global_pools::SSE_EVENT_POOL.release(buffer);
    }
}
```

### Step 4: Add Buffer Pool Metrics

```rust
#[derive(Default)]
pub struct PoolMetrics {
    pub alloc_count: AtomicU64,
    pub reuse_count: AtomicU64,
    pub discard_count: AtomicU64,
}

impl PoolMetrics {
    pub fn reuse_rate(&self) -> f64 {
        let reuse = self.reuse_count.load(Ordering::Relaxed) as f64;
        let total = reuse + self.alloc_count.load(Ordering::Relaxed) as f64;
        
        if total > 0.0 {
            reuse / total
        } else {
            0.0
        }
    }
    
    pub fn report(&self) {
        info!(
            "Buffer pool stats - Reuse rate: {:.2}%, Allocations: {}, Reuses: {}, Discards: {}",
            self.reuse_rate() * 100.0,
            self.alloc_count.load(Ordering::Relaxed),
            self.reuse_count.load(Ordering::Relaxed),
            self.discard_count.load(Ordering::Relaxed),
        );
    }
}
```

### Step 5: Optimize JSON Processing

```rust
// Use thread-local buffer for JSON serialization
thread_local! {
    static JSON_BUFFER: RefCell<BytesMut> = RefCell::new(BytesMut::with_capacity(8192));
}

pub fn serialize_json_pooled<T: Serialize>(value: &T) -> Result<Bytes> {
    JSON_BUFFER.with(|buf| {
        let mut buffer = buf.borrow_mut();
        buffer.clear();
        
        serde_json::to_writer(&mut *buffer, value)?;
        Ok(buffer.clone().freeze())
    })
}
```

## Testing

### Performance Test
```rust
#[bench]
fn bench_sse_processing_with_pooling(b: &mut Bencher) {
    b.iter(|| {
        let buffer = global_pools::SSE_EVENT_POOL.acquire();
        // Process event
        global_pools::SSE_EVENT_POOL.release(buffer);
    });
    // Should be 5-10x faster than allocation
}

#[test]
fn test_buffer_pool_reuse() {
    let pool = BytesPool::new(1024, 10);
    
    // Acquire and release buffers
    let buffers: Vec<_> = (0..100)
        .map(|_| pool.acquire())
        .collect();
    
    for buf in buffers {
        pool.release(buf);
    }
    
    // Check reuse rate
    assert!(pool.metrics.reuse_rate() > 0.8); // 80% reuse
}
```

### Memory Test
```rust
#[tokio::test]
async fn test_memory_usage_with_pooling() {
    let initial_memory = get_memory_usage();
    
    // Process 10000 SSE events
    for _ in 0..10000 {
        process_sse_event_with_pooling().await;
    }
    
    let final_memory = get_memory_usage();
    let increase = final_memory - initial_memory;
    
    // Should have minimal memory increase
    assert!(increase < 10 * 1024 * 1024); // Less than 10MB
}
```

## Success Criteria

- [ ] Buffer pools implemented for all hot paths
- [ ] >90% buffer reuse rate under load
- [ ] 50% reduction in memory allocations
- [ ] Memory usage reduced by 40%
- [ ] No memory leaks from pooled buffers
- [ ] Metrics show pool effectiveness

## Files to Modify

1. Create `src/proxy/reverse/buffer_pool.rs`
2. Update `src/proxy/reverse/upstream/http/streaming/raw.rs`
3. Update `src/proxy/reverse/upstream/http/streaming/intercepted.rs`
4. Update `src/proxy/reverse/handlers/mcp.rs` for JSON pooling
5. Add pool metrics to monitoring

## Configuration

```yaml
reverse_proxy:
  buffer_pools:
    sse_event:
      size: 4096
      max_pooled: 1000
    http_response:
      size: 16384
      max_pooled: 500
    json:
      size: 8192
      max_pooled: 500
```

## Monitoring

Track pool effectiveness:
```rust
// Export metrics
/metrics endpoint should include:
- buffer_pool_reuse_rate
- buffer_pool_allocations_total
- buffer_pool_discards_total
- buffer_pool_current_size
```