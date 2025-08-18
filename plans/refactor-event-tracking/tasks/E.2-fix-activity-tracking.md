# E.2: Fix Activity Tracking Task Overhead

**Task ID**: E.2  
**Phase**: Critical Fixes  
**Duration**: 1.5 hours  
**Dependencies**: E.1  
**Priority**: 🟡 HIGH  
**Status**: ⬜ Not Started

## Problem Statement

Activity tracking in SessionStream spawns a separate task for each activity update, compounding the task explosion problem. While rate-limited, it still creates excessive overhead.

## Objective

Replace task spawning for activity updates with a more efficient pattern that:
- Reuses the worker pattern from E.1
- Batches activity updates
- Eliminates per-event task spawning

## Implementation Steps

### 1. Extend PersistenceWorker for Activity Tracking (30 min)

```rust
// In src/session/persistence_worker.rs

pub enum WorkerRequest {
    PersistEventId(PersistenceRequest),
    RecordActivity { session_id: SessionId },
}

impl PersistenceWorker {
    // Update run() to handle both types
    pub async fn run(mut self) {
        let mut event_batch = Vec::new();
        let mut activity_batch = HashSet::new();
        let mut interval = tokio::time::interval(self.flush_interval);
        
        loop {
            tokio::select! {
                Some(req) = self.rx.recv() => {
                    match req {
                        WorkerRequest::PersistEventId(req) => {
                            event_batch.push(req);
                            if event_batch.len() >= self.max_batch_size {
                                self.flush_event_batch(&mut event_batch).await;
                            }
                        }
                        WorkerRequest::RecordActivity { session_id } => {
                            activity_batch.insert(session_id);
                            if activity_batch.len() >= 100 {
                                self.flush_activity_batch(&mut activity_batch).await;
                            }
                        }
                    }
                }
                _ = interval.tick() => {
                    if !event_batch.is_empty() {
                        self.flush_event_batch(&mut event_batch).await;
                    }
                    if !activity_batch.is_empty() {
                        self.flush_activity_batch(&mut activity_batch).await;
                    }
                }
            }
        }
    }
    
    async fn flush_activity_batch(&mut self, batch: &mut HashSet<SessionId>) {
        let start = Instant::now();
        let batch_size = batch.len();
        
        for session_id in batch.drain() {
            // Update activity for each session
            if let Ok(sessions) = self.session_store.try_read() {
                if let Some(state) = sessions.get(&session_id) {
                    state.health_monitor.record_activity().await;
                }
            }
        }
        
        // Record metrics
        let duration = start.elapsed();
        metrics::ACTIVITY_BATCH_SIZE.observe(batch_size as f64);
        metrics::ACTIVITY_BATCH_LATENCY.observe(duration.as_secs_f64());
        metrics::ACTIVITY_UPDATES_PROCESSED.inc_by(batch_size as u64);
    }
}
```

### 2. Update SessionStream to Use Worker (30 min)

```rust
// In src/transport/sse/session.rs

pub struct SessionStream {
    // ... existing fields ...
    worker_tx: mpsc::Sender<WorkerRequest>, // Add this
}

impl Stream for SessionStream {
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // ... existing code ...
        
        // Replace task spawning with channel send
        if self.needs_activity_update {
            let req = WorkerRequest::RecordActivity {
                session_id: self.session_id.clone(),
            };
            
            // Try to send without blocking
            match self.worker_tx.try_send(req) {
                Ok(_) => {
                    self.needs_activity_update = false;
                    metrics::ACTIVITY_UPDATES_SENT.inc();
                }
                Err(TrySendError::Full(_)) => {
                    // Channel full, skip this update
                    debug!("Activity tracking queue full, skipping update");
                    metrics::ACTIVITY_UPDATES_SKIPPED.inc();
                    metrics::ACTIVITY_QUEUE_FULL.inc();
                }
                Err(TrySendError::Closed(_)) => {
                    error!("Worker channel closed");
                }
            }
        }
        
        // ... rest of poll_next ...
    }
}
```

### 3. Optimize Activity Update Batching (20 min)

```rust
// Add smarter batching logic
pub struct ActivityTracker {
    pending_updates: Arc<RwLock<HashMap<SessionId, Instant>>>,
    min_update_interval: Duration,
}

impl ActivityTracker {
    pub async fn should_update(&self, session_id: &SessionId) -> bool {
        let mut pending = self.pending_updates.write().await;
        
        match pending.get(session_id) {
            Some(last_update) => {
                if last_update.elapsed() > self.min_update_interval {
                    pending.insert(session_id.clone(), Instant::now());
                    true
                } else {
                    false // Too soon since last update
                }
            }
            None => {
                pending.insert(session_id.clone(), Instant::now());
                true
            }
        }
    }
}
```

### 4. Add Comprehensive Metrics and Monitoring (10 min)

```rust
pub struct ActivityMetrics {
    updates_sent: Counter,
    updates_skipped: Counter,
    updates_batched: Histogram,
    queue_depth: Gauge,
    batch_latency: Histogram,
    coalesce_ratio: Gauge,
}

impl ActivityTracker {
    fn record_metrics(&self, batch_size: usize, unique_sessions: usize, duration: Duration) {
        metrics::ACTIVITY_UPDATES_BATCHED.observe(batch_size as f64);
        metrics::ACTIVITY_QUEUE_DEPTH.set(self.pending_updates.read().len() as f64);
        metrics::ACTIVITY_BATCH_LATENCY.observe(duration.as_secs_f64());
        
        // Coalesce ratio shows efficiency (1.0 = no coalescing, >1.0 = good)
        let ratio = batch_size as f64 / unique_sessions.max(1) as f64;
        metrics::ACTIVITY_COALESCE_RATIO.set(ratio);
    }
}
```

## Success Criteria

- [ ] No task spawning for activity updates
- [ ] Activity updates batched efficiently
- [ ] Shared worker handles both persistence and activity
- [ ] Proper rate limiting prevents excessive updates
- [ ] Metrics track batching efficiency
- [ ] Monitoring metrics exposed:
  - `activity_updates_sent_total` counter
  - `activity_updates_skipped_total` counter
  - `activity_updates_batched` histogram
  - `activity_queue_depth` gauge
  - `activity_batch_latency_seconds` histogram
  - `activity_coalesce_ratio` gauge (efficiency metric)

## Testing

```bash
# Test activity batching
cargo test session::activity_batching

# Load test to verify no task explosion
cargo test test_activity_tracking_load -- --nocapture

# Verify metrics
cargo test test_activity_metrics
```

## Performance Targets

- Task spawn rate for activity: 0 (from N per update)
- Activity update latency: < 500ms (batched)
- Memory overhead: < 10KB for tracking
- CPU overhead: < 1% for activity tracking

## Notes

- Coordinate with E.1 to share worker infrastructure
- Consider coalescing updates for same session
- May need separate worker if load is high
- Monitor queue depth to prevent overflow
- All metrics should integrate with existing telemetry
- Consider adding alerts on:
  - `activity_queue_depth` > 800 (queue pressure)
  - `activity_coalesce_ratio` < 1.5 (poor batching)
  - `activity_batch_latency_seconds` P99 > 1 second
- Use OpenTelemetry spans to trace batch processing