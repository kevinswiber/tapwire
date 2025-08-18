# E.1: Implement Worker Pattern with Channel-Based EventTracker (FINAL)

**Task ID**: E.1  
**Phase**: Critical Fixes  
**Duration**: 3 hours  
**Dependencies**: Phase C Complete  
**Priority**: 🔴 CRITICAL  
**Status**: ⬜ Not Started

## Approach Change

**No callbacks!** EventTracker will own the persistence channel sender directly, providing natural backpressure through standard channel semantics.

## Implementation Steps

### 1. Update EventTracker to Own Channel (30 min)

```rust
// In src/transport/sse/reconnect.rs

use tokio::sync::mpsc;
use crate::session::{SessionId, PersistenceRequest};

pub struct EventTracker {
    session_id: SessionId,  // Track which session this is for
    last_event_id: Arc<RwLock<Option<Arc<str>>>>,
    seen_events: Arc<RwLock<VecDeque<Arc<str>>>>,
    max_tracked: usize,
    persistence_tx: Option<mpsc::Sender<PersistenceRequest>>,  // Direct ownership!
}

impl EventTracker {
    pub fn new(session_id: SessionId, max_tracked: usize) -> Self {
        Self {
            session_id,
            last_event_id: Arc::new(RwLock::new(None)),
            seen_events: Arc::new(RwLock::new(VecDeque::with_capacity(max_tracked))),
            max_tracked,
            persistence_tx: None,
        }
    }
    
    /// Configure persistence channel
    pub fn with_persistence(mut self, tx: mpsc::Sender<PersistenceRequest>) -> Self {
        self.persistence_tx = Some(tx);
        self
    }
    
    pub async fn record_event(&self, event: &SseEvent) -> Result<()> {
        if let Some(ref id) = event.id {
            // Check for duplicate
            {
                let seen = self.seen_events.read().await;
                if seen.iter().any(|s| **s == *id) {
                    debug!("Duplicate event {} ignored", id);
                    metrics::SSE_DUPLICATE_EVENTS.inc();
                    return Ok(());
                }
            }
            
            // Record new event
            let id_arc = Arc::from(id.as_str());
            {
                let mut seen = self.seen_events.write().await;
                seen.push_back(id_arc.clone());
                if seen.len() > self.max_tracked {
                    seen.pop_front();
                }
            }
            
            // Update last event ID
            *self.last_event_id.write().await = Some(id_arc);
            
            // Send to persistence if configured
            if let Some(ref tx) = self.persistence_tx {
                let req = PersistenceRequest {
                    session_id: self.session_id.clone(),
                    event_id: id.to_string(),
                    attempt: 0,
                };
                
                // Natural backpressure via channel!
                match timeout(Duration::from_millis(100), tx.send(req)).await {
                    Ok(Ok(_)) => {
                        metrics::PERSISTENCE_QUEUED.inc();
                    }
                    Ok(Err(_)) => {
                        error!("Persistence channel closed");
                        metrics::PERSISTENCE_CHANNEL_CLOSED.inc();
                    }
                    Err(_) => {
                        warn!("Persistence backpressure timeout for session {}", self.session_id);
                        metrics::PERSISTENCE_BACKPRESSURE_TIMEOUT.inc();
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

### 2. Create PersistenceWorker with BinaryHeap (45 min)

```rust
// In src/session/persistence_worker.rs (new file)

use tokio::sync::mpsc;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;
use std::time::{Duration, Instant};
use tokio::time::MissedTickBehavior;

#[derive(Clone, Debug)]
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
        other.retry_time.cmp(&self.retry_time)
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
        let mut interval = tokio::time::interval(self.flush_interval);
        
        // Prevent burst catch-up after delays
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        
        loop {
            tokio::select! {
                // Use recv_many for efficient batching (GPT-5 recommendation)
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
                _ = interval.tick() => {
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
}
```

### 3. Implement Coalescing for Write Reduction (30 min)

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
        
        debug!(
            "Coalesced {} events to {} unique sessions (ratio: {:.2})",
            original_size, coalesced_size, coalesce_ratio
        );
        
        metrics::PERSISTENCE_COALESCE_RATIO.set(coalesce_ratio);
        metrics::PERSISTENCE_EVENTS_COALESCED.inc_by((original_size - coalesced_size) as u64);
        
        // Convert to vec for batch store
        let updates: Vec<_> = latest.into_iter().collect();
        
        match self.store.batch_store_event_ids(&updates).await {
            Ok(_) => {
                debug!("Successfully persisted {} event IDs", updates.len());
                metrics::PERSISTENCE_SUCCESS.inc_by(updates.len() as u64);
                metrics::PERSISTENCE_BATCH_SIZE.observe(updates.len() as f64);
            }
            Err(e) => {
                error!("Batch persistence failed: {}", e);
                metrics::PERSISTENCE_FAILURES.inc_by(updates.len() as u64);
                
                // Add to retry queue with proper time ordering
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

### 4. Implement Proper Retry Processing (30 min)

```rust
impl PersistenceWorker {
    async fn process_retries(&mut self) {
        let now = Instant::now();
        let mut ready = Vec::new();
        
        // Pop all ready retries (BinaryHeap ensures proper ordering!)
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
        
        debug!("Processing {} retries", ready.len());
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
                    let mut req = PersistenceRequest {
                        session_id: session_id.clone(),
                        event_id: event_id.clone(),
                        attempt: 2,
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
        while !self.retry_queue.is_empty() {
            self.process_retries().await;
            if !self.retry_queue.is_empty() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
}
```

### 5. Update SessionManager (45 min)

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
        // Create tracker with persistence channel
        let tracker = EventTracker::new(session_id, self.config.max_pending_per_session)
            .with_persistence(self.persistence_tx.clone());
        
        Arc::new(tracker)
    }
    
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down SessionManager");
        
        // Close persistence channel to signal worker shutdown
        drop(self.persistence_tx.clone());
        
        // Wait for worker to finish
        if let Some(handle) = self.persistence_handle.take() {
            match tokio::time::timeout(Duration::from_secs(5), handle).await {
                Ok(Ok(_)) => info!("Persistence worker shut down cleanly"),
                Ok(Err(e)) => error!("Persistence worker panicked: {}", e),
                Err(_) => error!("Persistence worker shutdown timeout"),
            }
        }
        
        // ... rest of shutdown
        Ok(())
    }
}
```

### 6. Ensure In-Memory Store Supports Batching (10 min)

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
                
                debug!("Updated session {} with event {}", session_id, event_id);
            } else {
                // Session doesn't exist - could create or ignore based on requirements
                debug!("Ignoring event {} for non-existent session {}", event_id, session_id);
            }
        }
        
        Ok(())
    }
}
```

## Success Criteria

- [ ] EventTracker owns persistence channel (no callbacks!)
- [ ] Natural backpressure via channel send
- [ ] BinaryHeap ensures proper retry ordering
- [ ] recv_many provides efficient batching
- [ ] Coalescing reduces write amplification
- [ ] Interval skips prevent burst floods
- [ ] Single worker task per SessionManager
- [ ] Comprehensive metrics for monitoring
- [ ] Graceful shutdown with retry draining

## Testing

```bash
# Test channel-based backpressure
cargo test test_event_tracker_channel_backpressure

# Test worker pattern
cargo test test_persistence_worker

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
async fn test_event_tracker_channel_backpressure() {
    let (tx, mut rx) = mpsc::channel(1);
    
    // Fill channel
    tx.send(create_request("fill")).await.unwrap();
    
    let tracker = EventTracker::new("session-1", 100)
        .with_persistence(tx);
    
    // This should timeout due to backpressure
    let event = create_sse_event("test-event");
    let start = Instant::now();
    tracker.record_event(&event).await.unwrap();
    
    // Should have taken ~100ms due to timeout
    assert!(start.elapsed() >= Duration::from_millis(90));
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
    
    // Should pop in time order
    assert_eq!(queue.pop().unwrap().request.event_id, "first");
}

#[tokio::test]
async fn test_coalescing() {
    let mut batch = vec![
        PersistenceRequest { session_id: "A".into(), event_id: "1".into(), attempt: 0 },
        PersistenceRequest { session_id: "A".into(), event_id: "2".into(), attempt: 0 },
        PersistenceRequest { session_id: "A".into(), event_id: "3".into(), attempt: 0 },
    ];
    
    // After coalescing, should only have latest
    let coalesced = coalesce(batch);
    assert_eq!(coalesced.len(), 1);
    assert_eq!(coalesced.get("A"), Some(&"3".to_string()));
}
```

## Performance Targets

- Task spawn rate: < 1 per SessionManager (from 1000+/sec)
- Backpressure: Natural via channel blocking
- Coalesce ratio: > 2.0 under burst loads
- Retry ordering: 100% correct via BinaryHeap
- Memory overhead: < 100KB per SessionManager
- Persistence latency P99: < 200ms

## Notes

- This is the FINAL approach - no callbacks, just channels
- EventTracker knows its session_id for proper scoping
- Channel ownership provides natural backpressure
- All GPT-5 recommendations incorporated
- Consistent with existing Shadowcat patterns
- Simpler and more testable than callback approach