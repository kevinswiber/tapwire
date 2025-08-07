# Phase 1 - Task 1.5: SSE Performance Optimization

## Task Overview
Optimize SSE implementation for production performance, add comprehensive benchmarks, and ensure the system meets the < 5% latency overhead target specified in project requirements.

**Duration**: 4-5 hours
**Priority**: MEDIUM - Required for production readiness
**Dependencies**: Tasks 1.1-1.4 (Complete SSE implementation) must be done

## Objectives

### Primary Goals
1. Optimize parser for zero-copy operations
2. Implement efficient buffering strategies
3. Add performance benchmarks
4. Profile and eliminate bottlenecks
5. Ensure < 5% latency overhead target

### Success Criteria
- [ ] Parser operates with minimal allocations
- [ ] Buffering optimized for typical message sizes
- [ ] Benchmarks show < 5% latency overhead
- [ ] Memory usage < 100MB for 1000 concurrent sessions
- [ ] Throughput > 10,000 messages/second
- [ ] CPU usage remains low under load
- [ ] No memory leaks under stress testing
- [ ] Performance regression tests in place
- [ ] Profiling data documented

## Performance Requirements

### Project Targets (from requirements)
1. **Latency Overhead**: < 5% p95 for typical tool calls
2. **Memory Usage**: < 100MB for 1000 concurrent sessions
3. **Throughput**: > 10,000 requests/second
4. **Startup Time**: < 100ms (SSE subsystem)
5. **Recording Overhead**: < 10% additional latency

### SSE-Specific Targets
1. **Parse Speed**: > 1GB/s for event parsing
2. **Event Latency**: < 1ms from receipt to emission
3. **Connection Memory**: < 100KB per active connection
4. **Reconnection Time**: < 500ms average
5. **Buffer Efficiency**: < 2x data size in memory

## Implementation Plan

### Module Structure
```
src/transport/sse/
├── perf/
│   ├── mod.rs          # Performance utilities
│   ├── zero_copy.rs    # Zero-copy optimizations
│   ├── buffer_pool.rs  # Buffer pooling
│   └── metrics.rs      # Performance metrics
└── benches/
    ├── parser.rs       # Parser benchmarks
    ├── connection.rs   # Connection benchmarks
    └── end_to_end.rs   # Full flow benchmarks
```

### Core Optimizations

#### 1. Zero-Copy Parser (`perf/zero_copy.rs`)
```rust
use bytes::{Bytes, BytesMut};
use memchr::memchr;

pub struct ZeroCopyParser {
    buffer: BytesMut,
    position: usize,
    event_builder: EventBuilder,
}

impl ZeroCopyParser {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
            position: 0,
            event_builder: EventBuilder::new(),
        }
    }
    
    pub fn feed_bytes(&mut self, data: Bytes) -> Vec<SseEvent> {
        self.buffer.extend_from_slice(&data);
        let mut events = Vec::new();
        
        while let Some(line_end) = self.find_line_end() {
            let line = self.extract_line(line_end);
            if let Some(event) = self.process_line_zero_copy(&line) {
                events.push(event);
            }
        }
        
        self.compact_buffer();
        events
    }
    
    #[inline(always)]
    fn find_line_end(&self) -> Option<usize> {
        memchr(b'\n', &self.buffer[self.position..])
            .map(|pos| self.position + pos)
    }
    
    #[inline(always)]
    fn extract_line(&mut self, end: usize) -> Bytes {
        let line = self.buffer[self.position..end].to_vec();
        self.position = end + 1;
        Bytes::from(line)
    }
    
    fn process_line_zero_copy(&mut self, line: &Bytes) -> Option<SseEvent> {
        // Process without string allocation where possible
        if line.is_empty() {
            return self.event_builder.build_zero_copy();
        }
        
        // Use byte operations instead of string parsing
        if let Some(colon_pos) = memchr(b':', line) {
            let field = &line[..colon_pos];
            let value = if colon_pos + 1 < line.len() && line[colon_pos + 1] == b' ' {
                &line[colon_pos + 2..]
            } else {
                &line[colon_pos + 1..]
            };
            
            self.event_builder.add_field_bytes(field, value);
        }
        
        None
    }
    
    fn compact_buffer(&mut self) {
        if self.position > 0 {
            self.buffer.advance(self.position);
            self.position = 0;
        }
        
        // Shrink buffer if too large and mostly empty
        if self.buffer.capacity() > 65536 && self.buffer.len() < 8192 {
            self.buffer.shrink_to(16384);
        }
    }
}
```

#### 2. Buffer Pool (`perf/buffer_pool.rs`)
```rust
use parking_lot::Mutex;
use std::sync::Arc;

pub struct BufferPool {
    small_pool: Arc<Mutex<Vec<BytesMut>>>,  // 4KB buffers
    medium_pool: Arc<Mutex<Vec<BytesMut>>>, // 64KB buffers
    large_pool: Arc<Mutex<Vec<BytesMut>>>,  // 1MB buffers
    stats: Arc<PoolStats>,
}

#[derive(Default)]
struct PoolStats {
    allocations: AtomicU64,
    reuses: AtomicU64,
    current_size: AtomicUsize,
}

impl BufferPool {
    pub fn new() -> Self {
        Self {
            small_pool: Arc::new(Mutex::new(Vec::with_capacity(100))),
            medium_pool: Arc::new(Mutex::new(Vec::with_capacity(50))),
            large_pool: Arc::new(Mutex::new(Vec::with_capacity(10))),
            stats: Arc::new(PoolStats::default()),
        }
    }
    
    pub fn acquire(&self, size: usize) -> PooledBuffer {
        let buffer = if size <= 4096 {
            self.try_acquire_from(&self.small_pool, 4096)
        } else if size <= 65536 {
            self.try_acquire_from(&self.medium_pool, 65536)
        } else {
            self.try_acquire_from(&self.large_pool, 1048576)
        };
        
        let buffer = buffer.unwrap_or_else(|| {
            self.stats.allocations.fetch_add(1, Ordering::Relaxed);
            BytesMut::with_capacity(size)
        });
        
        PooledBuffer::new(buffer, self.clone())
    }
    
    fn try_acquire_from(&self, pool: &Mutex<Vec<BytesMut>>, capacity: usize) -> Option<BytesMut> {
        let mut guard = pool.lock();
        guard.pop().map(|mut buf| {
            buf.clear();
            buf.reserve(capacity);
            self.stats.reuses.fetch_add(1, Ordering::Relaxed);
            buf
        })
    }
    
    fn release(&self, mut buffer: BytesMut) {
        let capacity = buffer.capacity();
        buffer.clear();
        
        let pool = if capacity <= 4096 {
            &self.small_pool
        } else if capacity <= 65536 {
            &self.medium_pool
        } else {
            &self.large_pool
        };
        
        let mut guard = pool.lock();
        if guard.len() < guard.capacity() {
            guard.push(buffer);
        }
    }
}

pub struct PooledBuffer {
    buffer: Option<BytesMut>,
    pool: BufferPool,
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            self.pool.release(buffer);
        }
    }
}
```

#### 3. Performance Metrics (`perf/metrics.rs`)
```rust
use prometheus::{Histogram, IntCounter, IntGauge, register_histogram, register_int_counter};

pub struct SseMetrics {
    pub parse_duration: Histogram,
    pub events_parsed: IntCounter,
    pub bytes_processed: IntCounter,
    pub active_connections: IntGauge,
    pub buffer_allocations: IntCounter,
    pub buffer_reuses: IntCounter,
    pub reconnections: IntCounter,
    pub event_latency: Histogram,
}

impl SseMetrics {
    pub fn new() -> Self {
        Self {
            parse_duration: register_histogram!(
                "sse_parse_duration_seconds",
                "Time to parse SSE events"
            ).unwrap(),
            events_parsed: register_int_counter!(
                "sse_events_parsed_total",
                "Total SSE events parsed"
            ).unwrap(),
            bytes_processed: register_int_counter!(
                "sse_bytes_processed_total",
                "Total bytes processed"
            ).unwrap(),
            active_connections: register_int_gauge!(
                "sse_active_connections",
                "Number of active SSE connections"
            ).unwrap(),
            buffer_allocations: register_int_counter!(
                "sse_buffer_allocations_total",
                "Buffer allocations"
            ).unwrap(),
            buffer_reuses: register_int_counter!(
                "sse_buffer_reuses_total",
                "Buffer reuses from pool"
            ).unwrap(),
            reconnections: register_int_counter!(
                "sse_reconnections_total",
                "SSE reconnection attempts"
            ).unwrap(),
            event_latency: register_histogram!(
                "sse_event_latency_seconds",
                "Latency from receipt to emission"
            ).unwrap(),
        }
    }
    
    #[inline(always)]
    pub fn record_parse(&self, bytes: usize, events: usize, duration: Duration) {
        self.bytes_processed.inc_by(bytes as u64);
        self.events_parsed.inc_by(events as u64);
        self.parse_duration.observe(duration.as_secs_f64());
    }
}
```

### Benchmarks

#### 1. Parser Benchmark (`benches/parser.rs`)
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("sse_parser");
    
    // Small events (typical)
    group.bench_function("small_events", |b| {
        let data = generate_sse_data(100, 100);  // 100 events, 100 bytes each
        let mut parser = ZeroCopyParser::new(4096);
        
        b.iter(|| {
            parser.feed_bytes(black_box(data.clone()))
        });
    });
    
    // Large events
    group.bench_function("large_events", |b| {
        let data = generate_sse_data(10, 10000);  // 10 events, 10KB each
        let mut parser = ZeroCopyParser::new(65536);
        
        b.iter(|| {
            parser.feed_bytes(black_box(data.clone()))
        });
    });
    
    // Multi-line data fields
    group.bench_function("multiline_data", |b| {
        let data = generate_multiline_sse(50);
        let mut parser = ZeroCopyParser::new(8192);
        
        b.iter(|| {
            parser.feed_bytes(black_box(data.clone()))
        });
    });
    
    // Throughput test
    for size in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput::Bytes(*size * 1024));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}KB", size)),
            size,
            |b, &size| {
                let data = generate_sse_bytes(size * 1024);
                let mut parser = ZeroCopyParser::new(8192);
                
                b.iter(|| {
                    parser.feed_bytes(black_box(data.clone()))
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_parser);
criterion_main!(benches);
```

#### 2. End-to-End Benchmark (`benches/end_to_end.rs`)
```rust
fn bench_end_to_end(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("sse_roundtrip_latency", |b| {
        b.to_async(&runtime).iter(|| async {
            let server = MockSseServer::new();
            let client = create_sse_client();
            
            let start = Instant::now();
            let response = client.post_message(server.url(), test_message()).await.unwrap();
            let latency = start.elapsed();
            
            black_box(latency)
        });
    });
    
    c.bench_function("concurrent_connections", |b| {
        b.to_async(&runtime).iter(|| async {
            let manager = create_session_manager();
            let handles: Vec<_> = (0..100)
                .map(|_| {
                    let mgr = manager.clone();
                    tokio::spawn(async move {
                        mgr.open_stream("http://localhost:8080").await
                    })
                })
                .collect();
            
            futures::future::join_all(handles).await
        });
    });
}
```

### Profiling Strategy

1. **CPU Profiling**:
   ```bash
   cargo flamegraph --bench parser -- --bench
   perf record -g cargo bench
   perf report
   ```

2. **Memory Profiling**:
   ```bash
   valgrind --tool=massif cargo bench
   heaptrack cargo bench
   ```

3. **Allocation Tracking**:
   ```rust
   #[global_allocator]
   static ALLOC: dhat::Alloc = dhat::Alloc;
   
   fn profile_allocations() {
       let _profiler = dhat::Profiler::new_heap();
       // Run SSE operations
       // Profiler drops and prints stats
   }
   ```

### Optimization Techniques

1. **Parser Optimizations**:
   - Use SIMD for line detection (memchr)
   - Avoid UTF-8 validation until necessary
   - Inline hot functions
   - Minimize boundary checks

2. **Memory Optimizations**:
   - Buffer pooling for reuse
   - Compact buffers after use
   - Use Bytes for zero-copy slicing
   - Limit maximum buffer sizes

3. **Connection Optimizations**:
   - Connection pooling
   - HTTP/2 multiplexing
   - Compression where applicable
   - Batch small messages

4. **Concurrency Optimizations**:
   - Lock-free data structures where possible
   - Fine-grained locking
   - Async channel optimizations
   - Work stealing for load distribution

## Test Cases

### Performance Tests

1. **Latency Tests**:
   ```rust
   #[test]
   fn test_latency_target() {
       let baseline = measure_baseline_latency();
       let with_sse = measure_sse_latency();
       let overhead = (with_sse - baseline) / baseline;
       assert!(overhead < 0.05, "Latency overhead {}% exceeds 5% target", overhead * 100.0);
   }
   ```

2. **Memory Tests**:
   ```rust
   #[test]
   fn test_memory_usage() {
       let manager = create_manager();
       let initial = get_memory_usage();
       
       // Create 1000 sessions
       for _ in 0..1000 {
           manager.create_session().await;
       }
       
       let final_usage = get_memory_usage();
       let delta = final_usage - initial;
       assert!(delta < 100 * 1024 * 1024, "Memory usage {} exceeds 100MB", delta);
   }
   ```

3. **Throughput Tests**:
   ```rust
   #[test]
   fn test_throughput() {
       let parser = create_parser();
       let data = generate_events(10000);
       
       let start = Instant::now();
       let events = parser.parse_all(data);
       let duration = start.elapsed();
       
       let rate = events.len() as f64 / duration.as_secs_f64();
       assert!(rate > 10000.0, "Throughput {} below 10k/sec", rate);
   }
   ```

### Stress Tests

1. **Sustained Load**:
   - Run at 80% capacity for 1 hour
   - Monitor memory growth
   - Check for degradation

2. **Burst Load**:
   - Send 10x normal traffic
   - Verify recovery
   - Check buffer management

3. **Connection Churn**:
   - Rapidly open/close connections
   - Verify cleanup
   - Check for leaks

## Configuration

```rust
pub struct PerformanceConfig {
    pub parser_buffer_size: usize,         // Default: 8KB
    pub max_event_size: usize,             // Default: 1MB
    pub buffer_pool_enabled: bool,         // Default: true
    pub zero_copy_enabled: bool,           // Default: true
    pub metrics_enabled: bool,             // Default: true
    pub profiling_enabled: bool,           // Default: false (production)
}
```

## Metrics Dashboard

Create Grafana dashboard with:
- Parse rate (events/sec)
- Bytes processed (bytes/sec)
- Event latency (p50, p95, p99)
- Active connections
- Buffer pool efficiency
- Memory usage
- CPU usage
- Reconnection rate

## Next Steps

After completing Phase 1:
1. Update compliance tracker
2. Run full integration tests
3. Begin Phase 2: Multi-version architecture
4. Document performance characteristics

## Notes

- Profile in release mode for accurate results
- Consider platform-specific optimizations
- Document performance trade-offs
- Create performance regression tests
- Monitor production metrics continuously