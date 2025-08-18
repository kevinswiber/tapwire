# E.1: Implement Worker Pattern for Persistence

**Task ID**: E.1  
**Phase**: Critical Fixes  
**Duration**: 3 hours  
**Dependencies**: Phase C Complete  
**Priority**: 🔴 CRITICAL  
**Status**: ⬜ Not Started

## Problem Statement

The current implementation spawns a new async task for EVERY event recorded, leading to:
- Task explosion (1000+ tasks/second under load)
- Memory exhaustion from task stacks
- No backpressure mechanism
- Potential system instability

## Objective

Replace the fire-and-forget task spawning pattern with a bounded worker pattern that:
- Uses a single worker task per SessionManager
- Batches persistence operations
- Provides backpressure via bounded channels
- Handles errors properly

## Implementation Steps

### 1. Create PersistenceWorker Struct (45 min)

```rust
// In src/session/persistence_worker.rs (new file)

use tokio::sync::mpsc;
use std::collections::VecDeque;
use std::time::Duration;

pub struct PersistenceRequest {
    pub session_id: SessionId,
    pub event_id: String,
    pub attempt: usize,
    pub next_retry: Option<Instant>,
}

pub struct PersistenceWorker {
    rx: mpsc::Receiver<PersistenceRequest>,
    store: Arc<dyn SessionStore>,
    retry_queue: VecDeque<PersistenceRequest>,
    max_batch_size: usize,
    flush_interval: Duration,
}

impl PersistenceWorker {
    pub fn new(
        rx: mpsc::Receiver<PersistenceRequest>,
        store: Arc<dyn SessionStore>,
    ) -> Self {
        Self {
            rx,
            store,
            retry_queue: VecDeque::new(),
            max_batch_size: 50,
            flush_interval: Duration::from_millis(100),
        }
    }
    
    pub async fn run(mut self) {
        let mut batch = Vec::with_capacity(self.max_batch_size);
        let mut interval = tokio::time::interval(self.flush_interval);
        
        loop {
            tokio::select! {
                Some(req) = self.rx.recv() => {
                    batch.push(req);
                    metrics::PERSISTENCE_QUEUE_DEPTH.set(self.rx.len() as f64);
                    if batch.len() >= self.max_batch_size {
                        let start = Instant::now();
                        self.flush_batch(&mut batch).await;
                        metrics::PERSISTENCE_LATENCY.observe(start.elapsed().as_secs_f64());
                    }
                }
                _ = interval.tick() => {
                    if !batch.is_empty() {
                        let start = Instant::now();
                        self.flush_batch(&mut batch).await;
                        metrics::PERSISTENCE_LATENCY.observe(start.elapsed().as_secs_f64());
                    }
                    self.process_retries().await;
                    metrics::PERSISTENCE_RETRY_QUEUE_DEPTH.set(self.retry_queue.len() as f64);
                else => break, // Channel closed
            }
        }
        
        // Flush remaining on shutdown
        if !batch.is_empty() {
            self.flush_batch(&mut batch).await;
        }
    }
}
```

### 2. Add Batch Persistence to SessionStore Trait (30 min)

```rust
// In src/session/store.rs

#[async_trait]
pub trait SessionStore: Send + Sync {
    // ... existing methods ...
    
    /// Store multiple event IDs in a single operation
    async fn batch_store_event_ids(
        &self,
        updates: &[(SessionId, String)]
    ) -> Result<()> {
        // Default implementation: fall back to individual stores
        for (session_id, event_id) in updates {
            self.store_last_event_id(session_id, event_id.clone()).await?;
        }
        Ok(())
    }
}
```

### 3. Update SessionManager to Use Worker (45 min)

```rust
// In src/session/manager.rs

pub struct SessionManager {
    // ... existing fields ...
    persistence_tx: mpsc::Sender<PersistenceRequest>,
    persistence_handle: Option<JoinHandle<()>>,
}

impl SessionManager {
    pub async fn new(config: SessionConfig, store: Arc<dyn SessionStore>) -> Result<Self> {
        let (tx, rx) = mpsc::channel(1000); // Bounded channel
        
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
                .with_callback(move |event_id| {
                    // Send to channel instead of spawning task
                    let req = PersistenceRequest {
                        session_id: session_id_clone.clone(),
                        event_id: event_id.to_string(),
                        attempt: 0,
                        next_retry: None,
                    };
                    
                    // Non-blocking send with error handling
                    match tx.try_send(req) {
                        Ok(_) => {},
                        Err(TrySendError::Full(_)) => {
                            warn!("Persistence queue full, applying backpressure");
                            metrics::PERSISTENCE_QUEUE_FULL.inc();
                            metrics::PERSISTENCE_QUEUE_DEPTH.set(1000.0); // At capacity
                        }
                        Err(TrySendError::Closed(_)) => {
                            error!("Persistence worker stopped!");
                            metrics::PERSISTENCE_WORKER_STOPPED.inc();
                        }
                    }
                })
        )
    }
}
```

### 4. Implement Error Handling, Retries & Monitoring (45 min)

```rust
impl PersistenceWorker {
    async fn flush_batch(&mut self, batch: &mut Vec<PersistenceRequest>) {
        if batch.is_empty() {
            return;
        }
        
        let updates: Vec<_> = batch.iter()
            .map(|req| (req.session_id.clone(), req.event_id.clone()))
            .collect();
        
        match self.store.batch_store_event_ids(&updates).await {
            Ok(_) => {
                let batch_size = batch.len();
                info!("Successfully persisted {} event IDs", batch_size);
                metrics::PERSISTENCE_SUCCESS.inc_by(batch_size as u64);
                metrics::PERSISTENCE_BATCH_SIZE.observe(batch_size as f64);
                batch.clear();
            }
            Err(e) => {
                error!("Failed to persist {} event IDs: {}", batch.len(), e);
                metrics::PERSISTENCE_FAILURES.inc_by(batch.len() as u64);
                
                // Add to retry queue with exponential backoff
                for mut req in batch.drain(..) {
                    req.attempt += 1;
                    if req.attempt < 3 {  // Max 3 attempts
                        let delay = Duration::from_millis(100 * 2_u64.pow(req.attempt as u32));
                        req.next_retry = Some(Instant::now() + delay);
                        self.retry_queue.push_back(req);
                    } else {
                        error!("Dropping event after {} attempts: {}", req.attempt, req.event_id);
                        metrics::PERSISTENCE_DROPPED.inc();
                        metrics::PERSISTENCE_RETRY_EXHAUSTED.inc();
                    }
                }
            }
        }
    }
    
    async fn process_retries(&mut self) {
        let now = Instant::now();
        let mut ready_for_retry = Vec::new();
        
        // Find requests ready for retry
        while let Some(req) = self.retry_queue.front() {
            if let Some(retry_time) = req.next_retry {
                if retry_time <= now {
                    ready_for_retry.push(self.retry_queue.pop_front().unwrap());
                } else {
                    break; // Queue is sorted by retry time
                }
            } else {
                break;
            }
        }
        
        if !ready_for_retry.is_empty() {
            self.flush_batch(&mut ready_for_retry).await;
        }
    }
}
```

### 5. Add Graceful Shutdown (15 min)

```rust
impl SessionManager {
    pub async fn shutdown(&mut self) -> Result<()> {
        // Close persistence channel
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

## Success Criteria

- [ ] Single worker task per SessionManager (not per event)
- [ ] Bounded channel prevents unbounded task growth
- [ ] Batch persistence reduces database load
- [ ] Proper error handling with retries
- [ ] Graceful shutdown flushes pending events
- [ ] Metrics track success/failure/retry rates
- [ ] No task explosion under load
- [ ] Monitoring metrics exposed:
  - `persistence_queue_depth` gauge
  - `persistence_batch_size` histogram
  - `persistence_success_total` counter
  - `persistence_failure_total` counter
  - `persistence_retry_total` counter
  - `persistence_dropped_total` counter
  - `persistence_latency_seconds` histogram

## Testing

```bash
# Unit tests
cargo test session::persistence_worker

# Integration test with load
cargo test test_persistence_under_load -- --nocapture

# Benchmark before/after
cargo bench event_persistence
```

## Performance Targets

- Task spawn rate: < 1 per SessionManager (from 1000+/sec)
- Memory overhead: < 100KB per SessionManager (from unbounded)
- Persistence latency P99: < 200ms (batched)
- Error recovery: 99.9% success after retries

## Notes

- This is the most critical fix - system is unstable without it
- Consider using `tokio::sync::batch_semaphore` for additional backpressure
- Monitor channel depth in production via metrics
- May need to tune batch size and flush interval based on load
- All metrics should be exported via existing telemetry infrastructure
- Consider adding alerts on:
  - `persistence_queue_depth` > 800 (80% full)
  - `persistence_dropped_total` increasing
  - `persistence_latency_seconds` P99 > 1 second