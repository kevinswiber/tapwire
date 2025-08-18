# E.1: Implement Worker Pattern for Persistence (REVISED)

**Task ID**: E.1  
**Phase**: Critical Fixes  
**Duration**: 3 hours  
**Dependencies**: E.0 (async EventTracker)  
**Priority**: 🔴 CRITICAL  
**Status**: ⬜ Not Started

## Problem Statement

The current implementation spawns a new async task for EVERY event recorded, leading to:
- Task explosion (1000+ tasks/second under load)
- Memory exhaustion from task stacks
- No backpressure mechanism
- Potential system instability

## Critical Fixes from GPT-5 Review

1. ✅ **Real backpressure** via async callbacks and `send().await`
2. ✅ **BinaryHeap** for proper retry time ordering
3. ✅ **recv_many** for efficient batching
4. ✅ **Coalescing** to reduce write amplification
5. ✅ **Skip interval bursts** to prevent catch-up floods

## Implementation Steps

### 1. Create PersistenceWorker with BinaryHeap (45 min)

```rust
// In src/session/persistence_worker.rs (new file)

use tokio::sync::mpsc;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;
use std::time::{Duration, Instant};
use tokio::time::MissedTickBehavior;

#[derive(Clone)]
pub struct PersistenceRequest {
    pub session_id: SessionId,
    pub event_id: String,
    pub attempt: usize,
}

// Wrapper for heap ordering by retry time
struct RetryRequest {
    retry_time: Instant,
    request: PersistenceRequest,
}

impl PartialEq for RetryRequest {
    fn eq(&self, other: &Self) -> bool {
        self.retry_time == other.retry_time
    }
}

impl Eq for RetryRequest {}

impl PartialOrd for RetryRequest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Reverse for min-heap behavior (earliest first)
        Some(other.retry_time.cmp(&self.retry_time))
    }
}

impl Ord for RetryRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse for min-heap behavior
        other.retry_time.cmp(&self.retry_time)
    }
}

pub struct PersistenceWorker {
    rx: mpsc::Receiver<PersistenceRequest>,
    store: Arc<dyn SessionStore>,
    retry_queue: BinaryHeap<RetryRequest>,  // Properly ordered by time!
    max_batch_size: usize,
    flush_interval: Duration,
    max_retries: usize,
}

impl PersistenceWorker {
    pub fn new(
        rx: mpsc::Receiver<PersistenceRequest>,
        store: Arc<dyn SessionStore>,
    ) -> Self {
        Self {
            rx,
            store,
            retry_queue: BinaryHeap::new(),
            max_batch_size: 50,
            flush_interval: Duration::from_millis(100),
            max_retries: 3,
        }
    }
    
    pub async fn run(mut self) {
        let mut buf = Vec::with_capacity(self.max_batch_size);
        let mut interval = tokio::time::interval(self.flush_interval);
        
        // Prevent burst catch-up after delays
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        
        loop {
            tokio::select! {
                // Use recv_many for efficient batching
                n = self.rx.recv_many(&mut buf, self.max_batch_size) => {
                    if n == 0 { 
                        break; // Channel closed
                    }
                    
                    metrics::PERSISTENCE_QUEUE_DEPTH.set(self.rx.len() as f64);
                    
                    let start = Instant::now();
                    self.coalesce_and_flush(&mut buf).await;
                    metrics::PERSISTENCE_LATENCY.observe(start.elapsed().as_secs_f64());
                }
                _ = interval.tick() => {
                    // Process any pending retries
                    self.process_retries().await;
                    
                    // Report metrics
                    metrics::PERSISTENCE_RETRY_QUEUE_DEPTH.set(self.retry_queue.len() as f64);
                }
            }
        }
        
        // Flush remaining on shutdown
        if !buf.is_empty() {
            self.coalesce_and_flush(&mut buf).await;
        }
        
        // Process final retries
        self.drain_retries().await;
    }
}
```

### 2. Implement Coalescing for Write Reduction (30 min)

```rust
impl PersistenceWorker {
    /// Coalesce multiple events per session to reduce writes
    async fn coalesce_and_flush(&mut self, batch: &mut Vec<PersistenceRequest>) {
        if batch.is_empty() {
            return;
        }
        
        let original_size = batch.len();
        
        // Coalesce: only keep latest event per session
        let mut latest: HashMap<SessionId, String> = HashMap::with_capacity(batch.len());
        for req in batch.drain(..) {
            latest.insert(req.session_id, req.event_id);
        }
        
        let coalesced_size = latest.len();
        let coalesce_ratio = original_size as f64 / coalesced_size as f64;
        
        info!(
            "Coalesced {} events to {} unique sessions (ratio: {:.2})",
            original_size, coalesced_size, coalesce_ratio
        );
        
        metrics::PERSISTENCE_COALESCE_RATIO.set(coalesce_ratio);
        metrics::PERSISTENCE_EVENTS_COALESCED.inc_by((original_size - coalesced_size) as u64);
        
        // Convert to vec for batch store
        let updates: Vec<_> = latest.into_iter().collect();
        
        match self.store.batch_store_event_ids(&updates).await {
            Ok(_) => {
                info!("Successfully persisted {} event IDs", updates.len());
                metrics::PERSISTENCE_SUCCESS.inc_by(updates.len() as u64);
                metrics::PERSISTENCE_BATCH_SIZE.observe(updates.len() as f64);
            }
            Err(e) => {
                error!("Batch persistence failed: {}", e);
                metrics::PERSISTENCE_FAILURES.inc_by(updates.len() as u64);
                
                // Add to retry queue with proper ordering
                let retry_time = Instant::now() + Duration::from_millis(100);
                for (session_id, event_id) in updates {
                    let req = PersistenceRequest {
                        session_id,
                        event_id,
                        attempt: 1,
                    };
                    self.retry_queue.push(RetryRequest { retry_time, request: req });
                }
            }
        }
    }
}
```

### 3. Implement Proper Retry Processing with BinaryHeap (30 min)

```rust
impl PersistenceWorker {
    async fn process_retries(&mut self) {
        let now = Instant::now();
        let mut ready = Vec::new();
        
        // Pop all ready retries (properly ordered by time!)
        while let Some(retry_req) = self.retry_queue.peek() {
            if retry_req.retry_time <= now {
                let retry_req = self.retry_queue.pop().unwrap();
                ready.push(retry_req.request);
            } else {
                break; // Heap ensures earliest is at top
            }
        }
        
        if ready.is_empty() {
            return;
        }
        
        info!("Processing {} retries", ready.len());
        metrics::PERSISTENCE_RETRIES_PROCESSED.inc_by(ready.len() as u64);
        
        // Group by session for coalescing
        let mut by_session: HashMap<SessionId, String> = HashMap::new();
        for req in ready {
            by_session.insert(req.session_id, req.event_id);
        }
        
        let updates: Vec<_> = by_session.into_iter().collect();
        
        match self.store.batch_store_event_ids(&updates).await {
            Ok(_) => {
                info!("Successfully persisted {} retried events", updates.len());
                metrics::PERSISTENCE_RETRY_SUCCESS.inc_by(updates.len() as u64);
            }
            Err(e) => {
                error!("Retry batch failed: {}", e);
                
                // Re-add with exponential backoff
                for (session_id, event_id) in updates {
                    let req = PersistenceRequest {
                        session_id: session_id.clone(),
                        event_id: event_id.clone(),
                        attempt: 2, // Already attempted once
                    };
                    
                    if req.attempt < self.max_retries {
                        let delay = Duration::from_millis(100 * 2_u64.pow(req.attempt as u32));
                        let retry_time = Instant::now() + delay;
                        self.retry_queue.push(RetryRequest { 
                            retry_time, 
                            request: req 
                        });
                    } else {
                        error!("Dropping event after {} attempts: {}:{}", 
                               req.attempt, session_id, event_id);
                        metrics::PERSISTENCE_DROPPED.inc();
                        metrics::PERSISTENCE_RETRY_EXHAUSTED.inc();
                    }
                }
            }
        }
    }
    
    async fn drain_retries(&mut self) {
        // Process all remaining retries on shutdown
        while !self.retry_queue.is_empty() {
            self.process_retries().await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}
```

### 4. Update SessionManager with Async Callbacks (45 min)

```rust
// In src/session/manager.rs

use tokio::time::timeout;

pub struct SessionManager {
    // ... existing fields ...
    persistence_tx: mpsc::Sender<PersistenceRequest>,
    persistence_handle: Option<JoinHandle<()>>,
}

impl SessionManager {
    pub async fn new(config: SessionConfig, store: Arc<dyn SessionStore>) -> Result<Self> {
        // Bounded channel for natural backpressure
        let (tx, rx) = mpsc::channel(1000);
        
        // Start persistence worker
        let worker = PersistenceWorker::new(rx, store.clone());
        let handle = tokio::spawn(worker.run());
        
        Ok(Self {
            // ... existing fields ...
            persistence_tx: tx,
            persistence_handle: Some(handle),
        })
    }
    
    pub fn create_event_tracker(&self, session_id: SessionId) -> Arc<EventTracker> {
        let tx = self.persistence_tx.clone();
        let session_id_clone = session_id.clone();
        
        Arc::new(
            EventTracker::new(self.config.max_pending_per_session)
                // Use async callback for real backpressure!
                .with_async_callback(move |event_id| {
                    let tx = tx.clone();
                    let session_id = session_id_clone.clone();
                    let event_id = event_id.to_string();
                    
                    async move {
                        let req = PersistenceRequest {
                            session_id,
                            event_id,
                            attempt: 0,
                        };
                        
                        // Apply backpressure with timeout
                        match timeout(Duration::from_millis(100), tx.send(req)).await {
                            Ok(Ok(_)) => {
                                metrics::PERSISTENCE_QUEUED.inc();
                            }
                            Ok(Err(_)) => {
                                error!("Persistence channel closed");
                                metrics::PERSISTENCE_CHANNEL_CLOSED.inc();
                            }
                            Err(_) => {
                                // Timeout - backpressure is working!
                                warn!("Persistence send timeout - backpressure applied");
                                metrics::PERSISTENCE_BACKPRESSURE_TIMEOUT.inc();
                            }
                        }
                    }
                })
        )
    }
}
```

### 5. Ensure In-Memory Store Supports Batching (15 min)

```rust
// In src/session/memory.rs

impl SessionStore for MemoryStore {
    async fn batch_store_event_ids(
        &self,
        updates: &[(SessionId, String)]
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        for (session_id, event_id) in updates {
            if let Some(session) = sessions.get_mut(session_id) {
                session.last_event_id = Some(event_id.clone());
                session.updated_at = Instant::now();
            } else {
                // Session doesn't exist - could create or ignore
                debug!("Ignoring event for non-existent session: {}", session_id);
            }
        }
        
        Ok(())
    }
}
```

### 6. Add Comprehensive Metrics (15 min)

```rust
// In src/metrics.rs

lazy_static! {
    // Queue metrics
    pub static ref PERSISTENCE_QUEUE_DEPTH: Gauge = 
        register_gauge!("persistence_queue_depth", "Current queue depth");
    pub static ref PERSISTENCE_BACKPRESSURE_TIMEOUT: Counter = 
        register_counter!("persistence_backpressure_timeout", "Backpressure timeouts");
        
    // Batching metrics    
    pub static ref PERSISTENCE_BATCH_SIZE: Histogram = 
        register_histogram!("persistence_batch_size", "Batch sizes");
    pub static ref PERSISTENCE_COALESCE_RATIO: Gauge = 
        register_gauge!("persistence_coalesce_ratio", "Event coalescing ratio");
    pub static ref PERSISTENCE_EVENTS_COALESCED: Counter = 
        register_counter!("persistence_events_coalesced", "Events eliminated by coalescing");
        
    // Retry metrics
    pub static ref PERSISTENCE_RETRY_QUEUE_DEPTH: Gauge = 
        register_gauge!("persistence_retry_queue_depth", "Retry queue depth");
    pub static ref PERSISTENCE_RETRIES_PROCESSED: Counter = 
        register_counter!("persistence_retries_processed", "Retries processed");
    pub static ref PERSISTENCE_RETRY_SUCCESS: Counter = 
        register_counter!("persistence_retry_success", "Successful retries");
    pub static ref PERSISTENCE_RETRY_EXHAUSTED: Counter = 
        register_counter!("persistence_retry_exhausted", "Retries exhausted");
}
```

## Success Criteria

- [ ] Async callbacks enable real backpressure via `send().await`
- [ ] BinaryHeap ensures retry ordering by time
- [ ] recv_many provides efficient batching
- [ ] Coalescing reduces write amplification
- [ ] Interval skips prevent burst floods
- [ ] Single worker task per SessionManager
- [ ] Bounded channel prevents unbounded growth
- [ ] Comprehensive metrics for monitoring
- [ ] Graceful shutdown with retry draining

## Testing

```bash
# Test backpressure
cargo test test_worker_backpressure

# Test retry ordering
cargo test test_retry_heap_ordering

# Test coalescing
cargo test test_event_coalescing

# Load test
cargo test test_load_1000_events_per_second -- --nocapture

# Benchmark
cargo bench event_persistence
```

### Critical Test Cases

```rust
#[tokio::test]
async fn test_backpressure_blocks_producer() {
    let (tx, mut rx) = mpsc::channel(10);
    
    // Fill channel
    for i in 0..10 {
        tx.send(i).await.unwrap();
    }
    
    // This should timeout (backpressure working)
    let result = timeout(Duration::from_millis(50), tx.send(11)).await;
    assert!(result.is_err(), "Should block on full channel");
}

#[tokio::test]
async fn test_retry_ordering() {
    let mut queue = BinaryHeap::new();
    let now = Instant::now();
    
    // Add in random order
    queue.push(RetryRequest {
        retry_time: now + Duration::from_secs(3),
        request: create_request("third"),
    });
    queue.push(RetryRequest {
        retry_time: now + Duration::from_secs(1),
        request: create_request("first"),
    });
    queue.push(RetryRequest {
        retry_time: now + Duration::from_secs(2),
        request: create_request("second"),
    });
    
    // Should pop in time order
    assert_eq!(queue.pop().unwrap().request.event_id, "first");
    assert_eq!(queue.pop().unwrap().request.event_id, "second");
    assert_eq!(queue.pop().unwrap().request.event_id, "third");
}

#[tokio::test]
async fn test_coalescing() {
    let mut batch = vec![
        PersistenceRequest { session_id: "A", event_id: "1" },
        PersistenceRequest { session_id: "A", event_id: "2" },
        PersistenceRequest { session_id: "A", event_id: "3" },
        PersistenceRequest { session_id: "B", event_id: "4" },
    ];
    
    let coalesced = coalesce(batch);
    assert_eq!(coalesced.len(), 2); // A and B
    assert_eq!(coalesced.get("A"), Some("3")); // Latest for A
}
```

## Performance Targets

- Task spawn rate: < 1 per SessionManager (from 1000+/sec)
- Backpressure: Producers block when queue 80% full
- Coalesce ratio: > 2.0 under burst loads
- Retry ordering: 100% correct time ordering
- Memory overhead: < 100KB per SessionManager
- Persistence latency P99: < 200ms

## Notes

- This is the most critical fix - system is unstable without it
- Requires E.0 (async EventTracker) to be completed first
- Channel size (1000) is tunable based on load testing
- Coalescing is especially valuable for SSE burst patterns
- BinaryHeap ensures retries are always processed in correct order
- Skip behavior prevents interval floods after processing delays
- All metrics integrate with existing telemetry infrastructure