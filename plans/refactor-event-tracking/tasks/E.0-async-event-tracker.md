# E.0: Make EventTracker Support Async Callbacks

**Task ID**: E.0  
**Phase**: Critical Fixes - Prerequisites  
**Duration**: 1 hour  
**Dependencies**: None  
**Priority**: 🔴 CRITICAL (Prerequisite for E.1)  
**Status**: ⬜ Not Started

## Problem Statement

Current EventTracker only supports synchronous callbacks, which prevents us from using `send().await` for proper backpressure in the worker pattern. Without async callbacks, we're forced to use `try_send` which drops messages when the channel is full - the opposite of backpressure.

## Objective

Refactor EventTracker to support async callbacks while maintaining backward compatibility for existing sync usage.

## Implementation Steps

### 1. Create AsyncEventCallback Trait (15 min)

```rust
// In src/transport/sse/reconnect.rs

use futures::future::BoxFuture;

/// Async callback for event notifications
pub trait AsyncEventCallback: Send + Sync {
    fn call(&self, event_id: &str) -> BoxFuture<'static, ()>;
}

// Implement for async closures
impl<F, Fut> AsyncEventCallback for F
where
    F: Fn(&str) -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send + 'static,
{
    fn call(&self, event_id: &str) -> BoxFuture<'static, ()> {
        Box::pin(self(event_id))
    }
}
```

### 2. Update EventTracker Structure (20 min)

```rust
pub struct EventTracker {
    last_event_id: Arc<RwLock<Option<Arc<str>>>>,
    seen_events: Arc<RwLock<VecDeque<Arc<str>>>>,
    max_tracked: usize,
    // Keep sync callback for backward compatibility
    on_new_event: Option<Arc<dyn Fn(&str) + Send + Sync>>,
    // Add async callback option
    on_new_event_async: Option<Arc<dyn AsyncEventCallback>>,
}

impl EventTracker {
    pub fn new(max_tracked: usize) -> Self {
        Self {
            last_event_id: Arc::new(RwLock::new(None)),
            seen_events: Arc::new(RwLock::new(VecDeque::with_capacity(max_tracked))),
            max_tracked,
            on_new_event: None,
            on_new_event_async: None,
        }
    }
    
    /// Set a sync callback (backward compatibility)
    pub fn with_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_new_event = Some(Arc::new(callback));
        self
    }
    
    /// Set an async callback (new functionality)
    pub fn with_async_callback<F, Fut>(mut self, callback: F) -> Self
    where
        F: Fn(&str) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_new_event_async = Some(Arc::new(callback));
        self
    }
}
```

### 3. Update record_event to Handle Async Callbacks (20 min)

```rust
impl EventTracker {
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
            let id_str = id.as_str();
            {
                let mut seen = self.seen_events.write().await;
                seen.push_back(Arc::from(id_str));
                if seen.len() > self.max_tracked {
                    seen.pop_front();
                }
            }
            
            // Update last event ID
            *self.last_event_id.write().await = Some(Arc::from(id_str));
            
            // Call sync callback if present (backward compatibility)
            if let Some(ref callback) = self.on_new_event {
                callback(id_str);
            }
            
            // Call async callback if present (new functionality)
            if let Some(ref async_callback) = self.on_new_event_async {
                async_callback.call(id_str).await;
            }
        }
        
        Ok(())
    }
}
```

### 4. Update SessionManager to Use Async Callbacks (5 min)

```rust
// In src/session/manager.rs

impl SessionManager {
    pub fn create_event_tracker(&self, session_id: SessionId) -> Arc<EventTracker> {
        let tx = self.persistence_tx.clone();
        let session_id_clone = session_id.clone();
        
        Arc::new(
            EventTracker::new(self.config.max_pending_per_session)
                .with_async_callback(move |event_id| {
                    let tx = tx.clone();
                    let session_id = session_id_clone.clone();
                    let event_id = event_id.to_string();
                    
                    async move {
                        let req = PersistenceRequest {
                            session_id,
                            event_id,
                            attempt: 0,
                            next_retry: None,
                        };
                        
                        // Apply real backpressure with timeout
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
                })
        )
    }
}
```

## Success Criteria

- [ ] EventTracker supports both sync and async callbacks
- [ ] Async callbacks can apply backpressure via `send().await`
- [ ] Backward compatibility maintained for existing sync usage
- [ ] No performance regression for sync callbacks
- [ ] All existing tests still pass
- [ ] New tests for async callback functionality

## Testing

```bash
# Unit tests for async callbacks
cargo test transport::sse::reconnect::test_async_callback

# Integration test with backpressure
cargo test test_event_tracker_backpressure

# Verify backward compatibility
cargo test --lib
```

### Test Cases

```rust
#[tokio::test]
async fn test_async_callback_called() {
    let (tx, mut rx) = mpsc::channel(10);
    
    let tracker = EventTracker::new(100)
        .with_async_callback(move |event_id| {
            let tx = tx.clone();
            let id = event_id.to_string();
            async move {
                tx.send(id).await.unwrap();
            }
        });
    
    let event = SseEvent {
        id: Some("test-123".to_string()),
        ..Default::default()
    };
    
    tracker.record_event(&event).await.unwrap();
    
    let received = rx.recv().await.unwrap();
    assert_eq!(received, "test-123");
}

#[tokio::test]
async fn test_backpressure_applied() {
    let (tx, _rx) = mpsc::channel(1);
    
    // Fill channel
    tx.send("fill".to_string()).await.unwrap();
    
    let tracker = EventTracker::new(100)
        .with_async_callback(move |event_id| {
            let tx = tx.clone();
            let id = event_id.to_string();
            async move {
                // This should timeout due to backpressure
                let result = timeout(Duration::from_millis(50), tx.send(id)).await;
                assert!(result.is_err());
            }
        });
    
    let event = SseEvent {
        id: Some("blocked".to_string()),
        ..Default::default()
    };
    
    tracker.record_event(&event).await.unwrap();
}
```

## Notes

- This is a prerequisite for E.1 to enable proper backpressure
- Consider using `futures::future::Either` to optimize the callback dispatch
- May need to handle callback panics gracefully
- Async callbacks will slightly increase latency but provide critical backpressure
- Monitor the impact on SSE event processing performance