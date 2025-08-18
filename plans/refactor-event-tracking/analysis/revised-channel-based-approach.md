# Revised Implementation: Channel-Based EventTracker (No Callbacks)

**Date**: 2025-08-18  
**Status**: Final approach - removing callback pattern

## Problem with Callbacks

The callback pattern (both sync and async) is:
- Not idiomatic Rust
- An outlier in the Shadowcat codebase  
- Unnecessarily complex
- Hard to test

## Better Solution: EventTracker Owns Channel Sender

### Option 1: EventTracker with Session Context (RECOMMENDED)

```rust
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
                    metrics::SSE_DUPLICATE_EVENTS.inc();
                    return Ok(());
                }
            }
            
            // Record new event
            {
                let mut seen = self.seen_events.write().await;
                seen.push_back(Arc::from(id.as_str()));
                if seen.len() > self.max_tracked {
                    seen.pop_front();
                }
            }
            
            // Update last event ID
            *self.last_event_id.write().await = Some(Arc::from(id.as_str()));
            
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
                        warn!("Persistence send timeout - backpressure applied");
                        metrics::PERSISTENCE_BACKPRESSURE_TIMEOUT.inc();
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

### Option 2: Return Events for External Handling

```rust
pub struct NewEventInfo {
    pub session_id: SessionId,
    pub event_id: String,
}

impl EventTracker {
    pub async fn record_event(&self, event: &SseEvent) -> Result<Option<NewEventInfo>> {
        if let Some(ref id) = event.id {
            // Check for duplicate
            if is_duplicate {
                return Ok(None);
            }
            
            // Record event...
            
            // Return info for caller to handle
            Ok(Some(NewEventInfo {
                session_id: self.session_id.clone(),
                event_id: id.to_string(),
            }))
        } else {
            Ok(None)
        }
    }
}

// Caller handles persistence
if let Some(info) = tracker.record_event(&event).await? {
    persistence_tx.send(PersistenceRequest {
        session_id: info.session_id,
        event_id: info.event_id,
        attempt: 0,
    }).await?;
}
```

## Why Option 1 is Better

1. **Encapsulation** - EventTracker handles its own persistence
2. **Single Responsibility** - Tracking and persistence are cohesive
3. **Less Coupling** - Callers don't need to know about persistence
4. **Backpressure** - Built into the EventTracker
5. **Cleaner API** - Just call `record_event()`

## Implementation Changes

### SessionManager Creates EventTracker with Channel

```rust
impl SessionManager {
    pub fn create_event_tracker(&self, session_id: SessionId) -> Arc<EventTracker> {
        let tracker = EventTracker::new(session_id.clone(), self.config.max_pending_per_session)
            .with_persistence(self.persistence_tx.clone());
        
        Arc::new(tracker)
    }
}
```

### No Need for E.0 Task

We can **completely remove E.0** because:
- No async callbacks needed
- Channels provide natural async backpressure
- Much simpler implementation
- Consistent with rest of codebase

## Revised Phase E Structure

- **E.0: REMOVED** - No callbacks needed
- **E.1: Worker Pattern** (3 hours) - Now simpler!
  - EventTracker uses channels directly
  - Worker with BinaryHeap, recv_many, coalescing
- **E.2: Fix Activity Tracking** (1.5 hours) - Unchanged
- **E.3: Memory Optimization** (2 hours) - Unchanged

**Total: 6.5 hours** (down from 7.5)

## Benefits

1. **Simpler** - No callback traits or async closures
2. **Idiomatic** - Channels are standard Rust async pattern
3. **Consistent** - Matches existing Shadowcat patterns
4. **Testable** - Easy to test with mock channels
5. **Less Work** - Saves 1 hour by removing E.0

## Testing

```rust
#[tokio::test]
async fn test_event_tracker_applies_backpressure() {
    let (tx, mut rx) = mpsc::channel(1);
    
    // Fill channel
    rx.try_recv(); // Make space
    tx.send(create_request("fill")).await.unwrap();
    
    let tracker = EventTracker::new("session-1", 100)
        .with_persistence(tx);
    
    // First event should succeed quickly
    let event1 = create_sse_event("event-1");
    tracker.record_event(&event1).await.unwrap();
    
    // Second event should timeout (backpressure)
    let event2 = create_sse_event("event-2");
    let start = Instant::now();
    tracker.record_event(&event2).await.unwrap();
    
    // Should have taken ~100ms due to timeout
    assert!(start.elapsed() >= Duration::from_millis(90));
}
```

## Migration Path

1. Update EventTracker to include session_id field
2. Add persistence_tx as optional field
3. Remove all callback-related code
4. Update SessionManager to pass channel when creating EventTracker
5. Implement E.1 worker pattern as planned

This approach is cleaner, simpler, and more maintainable than callbacks!