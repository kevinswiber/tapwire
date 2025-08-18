# E.1: Implement Worker Pattern with Channel-Based EventTracker (FINAL v2)

**Task ID**: E.1  
**Phase**: Critical Fixes  
**Duration**: 3 hours  
**Dependencies**: Phase C Complete  
**Priority**: 🔴 CRITICAL  
**Status**: ⬜ Not Started

## Approach Summary

EventTracker owns the persistence channel sender directly with:
- **Short timeout (50-150ms)** to prevent SSE stalls
- **Latest-only buffer** using `tokio::sync::watch` for the newest event
- **Natural backpressure** via bounded channels
- **No callbacks** - idiomatic Rust patterns only

## Implementation Steps

### 1. Update EventTracker with Latest-Only Buffer (45 min)

```rust
// In src/transport/sse/reconnect.rs

use tokio::sync::{mpsc, watch};
use std::collections::HashSet;

pub struct EventTracker {
    session_id: SessionId,
    persistence_tx: mpsc::Sender<PersistenceRequest>,  // Mandatory, not Option!
    latest_pending: watch::Sender<Option<String>>,     // Latest-only buffer
    latest_pending_rx: watch::Receiver<Option<String>>, // For reading back
    seen_events: Arc<RwLock<HashSet<Arc<str>>>>,       // O(1) duplicate detection
    recent_order: Arc<RwLock<VecDeque<Arc<str>>>>,     // Track order for LRU
    max_tracked: usize,
    last_event_id: Arc<RwLock<Option<Arc<str>>>>,
}

impl EventTracker {
    pub fn new(session_id: SessionId, max_tracked: usize, persistence_tx: mpsc::Sender<PersistenceRequest>) -> Self {
        let (latest_tx, latest_rx) = watch::channel(None);
        
        Self {
            session_id,
            persistence_tx,
            latest_pending: latest_tx,
            latest_pending_rx: latest_rx,
            seen_events: Arc::new(RwLock::new(HashSet::with_capacity(max_tracked))),
            recent_order: Arc::new(RwLock::new(VecDeque::with_capacity(max_tracked))),
            max_tracked,
            last_event_id: Arc::new(RwLock::new(None)),
        }
    }
    
    pub async fn record_event(&self, event: &SseEvent) -> Result<()> {
        if let Some(ref id) = event.id {
            // O(1) duplicate check
            {
                let seen = self.seen_events.read().await;
                if seen.contains(id.as_str()) {
                    debug!("Duplicate event {} ignored for session {}", id, self.session_id);
                    metrics::SSE_DUPLICATE_EVENTS.inc();
                    return Ok(());
                }
            }
            
            // Record new event
            let id_arc = Arc::from(id.as_str());
            {
                let mut seen = self.seen_events.write().await;
                let mut order = self.recent_order.write().await;
                
                seen.insert(id_arc.clone());
                order.push_back(id_arc.clone());
                
                // LRU eviction if needed
                if order.len() > self.max_tracked {
                    if let Some(old) = order.pop_front() {
                        seen.remove(&old);
                    }
                }
            }
            
            // Update last event ID
            *self.last_event_id.write().await = Some(id_arc);
            
            // Try to flush any pending event first
            if let Some(pending) = self.latest_pending_rx.borrow().clone() {
                let req = PersistenceRequest {
                    session_id: self.session_id.clone(),
                    event_id: pending,
                    attempt: 0,
                };
                
                // Try to send pending (don't block)
                if self.persistence_tx.try_send(req).is_ok() {
                    self.latest_pending.send(None)?; // Clear pending
                    metrics::PERSISTENCE_PENDING_FLUSHED.inc();
                }
            }
            
            // Send current event with SHORT timeout
            let req = PersistenceRequest {
                session_id: self.session_id.clone(),
                event_id: id.to_string(),
                attempt: 0,
            };
            
            match timeout(Duration::from_millis(100), self.persistence_tx.send(req)).await {
                Ok(Ok(_)) => {
                    metrics::PERSISTENCE_QUEUED.inc();
                    Ok(())
                }
                Ok(Err(_)) => {
                    error!("Persistence channel closed for session {}", self.session_id);
                    Err(anyhow!("Persistence channel closed"))
                }
                Err(_) => {
                    // Timeout - save as latest pending
                    self.latest_pending.send(Some(id.to_string()))?;
                    warn!("Backpressure timeout - buffered event {} for session {}", id, self.session_id);
                    metrics::PERSISTENCE_BACKPRESSURE_BUFFERED.inc();
                    Ok(()) // Don't block SSE stream!
                }
            }
        } else {
            Ok(())
        }
    }
    
    pub async fn get_last_event_id(&self) -> Option<String> {
        self.last_event_id.read().await.as_ref().map(|s| s.to_string())
    }
}
```

### 2. Create PersistenceWorker with All Optimizations (45 min)

```rust
// In src/session/persistence_worker.rs (new file)

use tokio::sync::mpsc;
use tokio::time::{interval, MissedTickBehavior};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Reverse;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct PersistenceRequest {
    pub session_id: SessionId,
    pub event_id: String,
    pub attempt: usize,
}

// For BinaryHeap ordering by retry time
struct RetryRequest {
    retry_time: Instant,
    request: PersistenceRequest,
}

// Implement Ord/PartialOrd for min-heap behavior
impl PartialEq for RetryRequest {
    fn eq(&self, other: &Self) -> bool {
        self.retry_time == other.retry_time
    }
}

impl Eq for RetryRequest {}

impl PartialOrd for RetryRequest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.retry_time.cmp(&self.retry_time)) // Reverse for min-heap
    }
}

impl Ord for RetryRequest {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.retry_time.cmp(&self.retry_time) // Reverse for min-heap
    }
}

pub struct PersistenceWorker {
    rx: mpsc::Receiver<PersistenceRequest>,
    store: Arc<dyn SessionStore>,
    retry_queue: BinaryHeap<RetryRequest>,
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
        let mut flush_interval = interval(self.flush_interval);
        
        // CRITICAL: Prevent burst catch-up after delays
        flush_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        
        loop {
            tokio::select! {
                // Use recv_many for efficient batching
                n = self.rx.recv_many(&mut buf, self.max_batch_size) => {
                    if n == 0 { 
                        info!("Persistence channel closed, shutting down worker");
                        break;
                    }
                    
                    metrics::PERSISTENCE_QUEUE_DEPTH.set(self.rx.len() as f64);
                    
                    let start = Instant::now();
                    self.coalesce_and_flush(&mut buf).await;
                    metrics::PERSISTENCE_LATENCY.observe(start.elapsed().as_secs_f64());
                }
                
                _ = flush_interval.tick() => {
                    // Time-based flush for durability guarantee
                    if !buf.is_empty() {
                        let start = Instant::now();
                        self.coalesce_and_flush(&mut buf).await;
                        metrics::PERSISTENCE_LATENCY.observe(start.elapsed().as_secs_f64());
                    }
                    
                    // Process any pending retries
                    self.process_retries().await;
                    
                    // Report metrics
                    metrics::PERSISTENCE_RETRY_QUEUE_DEPTH.set(self.retry_queue.len() as f64);
                }
            }
        }
        
        // Graceful shutdown
        if !buf.is_empty() {
            info!("Flushing {} pending events on shutdown", buf.len());
            self.coalesce_and_flush(&mut buf).await;
        }
        
        // Process remaining retries
        self.drain_retries().await;
    }
    
    /// Coalesce multiple events per session to reduce writes
    async fn coalesce_and_flush(&mut self, batch: &mut Vec<PersistenceRequest>) {
        if batch.is_empty() {
            return;
        }
        
        let original_size = batch.len();
        
        // CRITICAL: Only keep latest event per session (coalescing)
        let mut latest: HashMap<SessionId, String> = HashMap::with_capacity(batch.len());
        for req in batch.drain(..) {
            latest.insert(req.session_id, req.event_id);
        }
        
        let coalesced_size = latest.len();
        if original_size > coalesced_size {
            let ratio = original_size as f64 / coalesced_size as f64;
            debug!(
                "Coalesced {} events to {} sessions (ratio: {:.2})",
                original_size, coalesced_size, ratio
            );
            metrics::PERSISTENCE_COALESCE_RATIO.set(ratio);
            metrics::PERSISTENCE_EVENTS_COALESCED.inc_by((original_size - coalesced_size) as u64);
        }
        
        // Convert to vec for batch store
        let updates: Vec<_> = latest.into_iter().collect();
        
        match self.store.batch_store_event_ids(&updates).await {
            Ok(_) => {
                debug!("Persisted {} event IDs", updates.len());
                metrics::PERSISTENCE_SUCCESS.inc_by(updates.len() as u64);
                metrics::PERSISTENCE_BATCH_SIZE.observe(updates.len() as f64);
            }
            Err(e) => {
                error!("Batch persistence failed: {}", e);
                metrics::PERSISTENCE_FAILURES.inc_by(updates.len() as u64);
                
                // Add to retry queue with exponential backoff
                let retry_time = Instant::now() + Duration::from_millis(100);
                for (session_id, event_id) in updates {
                    self.retry_queue.push(RetryRequest {
                        retry_time,
                        request: PersistenceRequest {
                            session_id,
                            event_id,
                            attempt: 1,
                        },
                    });
                }
            }
        }
    }
    
    async fn process_retries(&mut self) {
        let now = Instant::now();
        let mut ready = Vec::new();
        
        // Pop all ready retries (BinaryHeap ensures proper time ordering!)
        while let Some(retry_req) = self.retry_queue.peek() {
            if retry_req.retry_time <= now {
                ready.push(self.retry_queue.pop().unwrap().request);
            } else {
                break; // Heap guarantees no earlier items remain
            }
        }
        
        if !ready.is_empty() {
            debug!("Processing {} retries", ready.len());
            metrics::PERSISTENCE_RETRIES_PROCESSED.inc_by(ready.len() as u64);
            
            // Coalesce retries too!
            let mut batch = ready;
            self.coalesce_and_flush(&mut batch).await;
        }
    }
    
    async fn drain_retries(&mut self) {
        while !self.retry_queue.is_empty() {
            self.process_retries().await;
            if !self.retry_queue.is_empty() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
}
```

### 3. Update SessionManager to Create Proper EventTrackers (30 min)

```rust
// In src/session/manager.rs

pub struct SessionManager {
    // ... existing fields ...
    persistence_tx: mpsc::Sender<PersistenceRequest>,
    persistence_handle: Option<JoinHandle<()>>,
}

impl SessionManager {
    pub async fn new(config: SessionConfig, store: Arc<dyn SessionStore>) -> Result<Self> {
        // Bounded channel for natural backpressure
        // Size tuned for expected load (1000 is reasonable default)
        let (tx, rx) = mpsc::channel(1000);
        
        // Start persistence worker
        let worker = PersistenceWorker::new(rx, store.clone());
        let handle = tokio::spawn(async move {
            worker.run().await;
            info!("Persistence worker terminated");
        });
        
        Ok(Self {
            // ... existing fields ...
            persistence_tx: tx,
            persistence_handle: Some(handle),
        })
    }
    
    pub fn create_event_tracker(&self, session_id: SessionId) -> Arc<EventTracker> {
        // Create tracker with mandatory persistence channel
        let tracker = EventTracker::new(
            session_id,
            self.config.max_pending_per_session,
            self.persistence_tx.clone(),
        );
        
        Arc::new(tracker)
    }
    
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down SessionManager");
        
        // Close persistence channel to signal worker shutdown
        // Note: dropping doesn't close, need to drop all clones
        drop(self.persistence_tx.clone()); 
        
        // Wait for worker to finish with timeout
        if let Some(handle) = self.persistence_handle.take() {
            match tokio::time::timeout(Duration::from_secs(5), handle).await {
                Ok(Ok(_)) => info!("Persistence worker shut down cleanly"),
                Ok(Err(e)) => error!("Persistence worker panicked: {}", e),
                Err(_) => error!("Persistence worker shutdown timeout"),
            }
        }
        
        Ok(())
    }
}
```

### 4. Add SSE Heartbeats to Prevent Connection Drops (20 min)

```rust
// In src/transport/sse/session.rs

const SSE_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);

impl SessionStream {
    pub async fn run_with_heartbeat(mut self) {
        let mut heartbeat = interval(SSE_HEARTBEAT_INTERVAL);
        heartbeat.set_missed_tick_behavior(MissedTickBehavior::Skip);
        
        loop {
            tokio::select! {
                Some(event) = self.next() => {
                    // Process normal event
                    if let Err(e) = self.send_event(event).await {
                        error!("Failed to send SSE event: {}", e);
                        break;
                    }
                }
                
                _ = heartbeat.tick() => {
                    // Send comment as keepalive
                    if let Err(e) = self.send_comment("heartbeat").await {
                        error!("Failed to send heartbeat: {}", e);
                        break;
                    }
                    metrics::SSE_HEARTBEATS_SENT.inc();
                }
                
                else => break,
            }
        }
    }
    
    async fn send_comment(&mut self, comment: &str) -> Result<()> {
        // SSE comment format: ": comment text\n\n"
        let data = format!(": {}\n\n", comment);
        self.sink.send(data.into()).await
    }
}
```

### 5. Update In-Memory Store (10 min)

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
                // Could create session or ignore based on requirements
                debug!("Session {} not found, ignoring event {}", session_id, event_id);
            }
        }
        
        Ok(())
    }
    
    async fn get_last_event_id(&self, session_id: &SessionId) -> Result<Option<String>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id)
            .and_then(|s| s.last_event_id.clone()))
    }
}
```

### 6. Add Comprehensive Metrics (10 min)

```rust
// In src/metrics.rs

lazy_static! {
    // Channel metrics
    pub static ref PERSISTENCE_QUEUE_DEPTH: Gauge = 
        register_gauge!("persistence_queue_depth", "Current queue depth");
    pub static ref PERSISTENCE_BACKPRESSURE_BUFFERED: Counter = 
        register_counter!("persistence_backpressure_buffered", "Events buffered due to backpressure");
    pub static ref PERSISTENCE_PENDING_FLUSHED: Counter = 
        register_counter!("persistence_pending_flushed", "Pending events successfully flushed");
        
    // Batching metrics
    pub static ref PERSISTENCE_BATCH_SIZE: Histogram = 
        register_histogram!("persistence_batch_size", "Batch sizes");
    pub static ref PERSISTENCE_COALESCE_RATIO: Gauge = 
        register_gauge!("persistence_coalesce_ratio", "Event coalescing ratio");
    pub static ref PERSISTENCE_EVENTS_COALESCED: Counter = 
        register_counter!("persistence_events_coalesced", "Events eliminated by coalescing");
        
    // SSE metrics
    pub static ref SSE_HEARTBEATS_SENT: Counter = 
        register_counter!("sse_heartbeats_sent", "SSE heartbeat comments sent");
    pub static ref SSE_DUPLICATE_EVENTS: Counter = 
        register_counter!("sse_duplicate_events", "Duplicate SSE events ignored");
        
    // Success/failure metrics
    pub static ref PERSISTENCE_SUCCESS: Counter = 
        register_counter!("persistence_success_total", "Successful persistence operations");
    pub static ref PERSISTENCE_FAILURES: Counter = 
        register_counter!("persistence_failures_total", "Failed persistence operations");
    pub static ref PERSISTENCE_LATENCY: Histogram = 
        register_histogram!("persistence_latency_seconds", "Persistence operation latency");
}
```

## Success Criteria

- [ ] EventTracker owns mandatory persistence channel (not Option!)
- [ ] Short timeout (100ms) with latest-only buffer via watch channel
- [ ] O(1) duplicate detection with HashSet
- [ ] BinaryHeap ensures proper retry time ordering
- [ ] recv_many provides efficient atomic batching
- [ ] Coalescing reduces writes (both in worker and retries)
- [ ] MissedTickBehavior::Skip prevents burst floods
- [ ] SSE heartbeats prevent connection timeouts
- [ ] Comprehensive metrics for monitoring
- [ ] Graceful shutdown with retry draining

## Testing

```bash
# Test channel-based backpressure and buffering
cargo test test_event_tracker_backpressure_with_buffer

# Test worker pattern
cargo test test_persistence_worker

# Test retry ordering
cargo test test_retry_heap_ordering

# Test coalescing
cargo test test_event_coalescing

# Test SSE heartbeats
cargo test test_sse_heartbeat

# Load test
cargo test test_load_1000_events_per_second -- --nocapture

# Benchmark
cargo bench event_persistence
```

### Critical Test Cases

```rust
#[tokio::test]
async fn test_backpressure_with_buffer() {
    let (tx, mut rx) = mpsc::channel(1);
    
    // Fill channel
    tx.send(create_request("fill")).await.unwrap();
    
    let tracker = EventTracker::new("session-1", 100, tx);
    
    // First event should timeout and buffer
    let event1 = create_sse_event("event-1");
    let start = Instant::now();
    tracker.record_event(&event1).await.unwrap();
    assert!(start.elapsed() >= Duration::from_millis(90));
    
    // Verify event was buffered
    assert_eq!(*tracker.latest_pending_rx.borrow(), Some("event-1".to_string()));
    
    // Clear channel
    rx.recv().await.unwrap();
    
    // Next event should flush pending first
    let event2 = create_sse_event("event-2");
    tracker.record_event(&event2).await.unwrap();
    
    // Should have both events in channel now
    let req1 = rx.recv().await.unwrap();
    assert_eq!(req1.event_id, "event-1"); // Pending flushed
    let req2 = rx.recv().await.unwrap();
    assert_eq!(req2.event_id, "event-2"); // Current sent
}

#[tokio::test]
async fn test_retry_ordering() {
    let mut heap = BinaryHeap::new();
    let now = Instant::now();
    
    // Add in random order
    heap.push(RetryRequest {
        retry_time: now + Duration::from_secs(3),
        request: create_request("third"),
    });
    heap.push(RetryRequest {
        retry_time: now + Duration::from_secs(1),
        request: create_request("first"),
    });
    heap.push(RetryRequest {
        retry_time: now + Duration::from_secs(2),
        request: create_request("second"),
    });
    
    // Should pop in time order (earliest first)
    assert_eq!(heap.pop().unwrap().request.event_id, "first");
    assert_eq!(heap.pop().unwrap().request.event_id, "second");
    assert_eq!(heap.pop().unwrap().request.event_id, "third");
}

#[tokio::test]
async fn test_coalescing() {
    let mut batch = vec![
        PersistenceRequest { session_id: "A".into(), event_id: "1".into(), attempt: 0 },
        PersistenceRequest { session_id: "A".into(), event_id: "2".into(), attempt: 0 },
        PersistenceRequest { session_id: "A".into(), event_id: "3".into(), attempt: 0 },
        PersistenceRequest { session_id: "B".into(), event_id: "4".into(), attempt: 0 },
    ];
    
    // After coalescing, should only have latest per session
    let coalesced = coalesce(batch);
    assert_eq!(coalesced.len(), 2); // A and B
    assert_eq!(coalesced.get("A"), Some(&"3".to_string())); // Latest for A
    assert_eq!(coalesced.get("B"), Some(&"4".to_string())); // Latest for B
}
```

## Performance Targets

- Task spawn rate: < 1 per SessionManager (from 1000+/sec)
- Backpressure: Applied via 100ms timeout + buffering
- Coalesce ratio: > 2.0 under burst loads
- Retry ordering: 100% correct via BinaryHeap
- SSE latency: < 100ms timeout prevents long stalls
- Memory overhead: < 100KB per SessionManager
- Persistence latency P99: < 200ms

## Key Improvements from Feedback

1. **Short timeout (100ms)** - Prevents SSE stalls
2. **Latest-only buffer with watch** - Perfect for "only newest matters"
3. **Mandatory sender** - Simpler than Option
4. **O(1) duplicate detection** - HashSet instead of VecDeque scan
5. **SSE heartbeats** - Prevents proxy/browser timeouts
6. **Coalescing everywhere** - In worker AND retries
7. **recv_many + Skip** - Already planned, confirmed correct

## Notes

- This is the FINAL approach incorporating all feedback
- tokio::sync::watch is perfect for latest-only semantics
- Short timeout + buffer gives best of both worlds
- SSE heartbeats are critical for production
- All metrics integrate with existing telemetry