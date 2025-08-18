# H.4: Implement SSE Reconnection

**Priority**: 🔴 CRITICAL  
**Duration**: 6 hours  
**Status**: ⏳ Pending  

## Problem

SSE connections drop permanently on any network issue, breaking real-time features. The reconnection logic is completely missing.

**Location**: `src/proxy/reverse/upstream/http/streaming/intercepted.rs:292-325`

```rust
// CURRENT - Just logs and gives up
if let Some(last_id) = &self.session.last_event_id {
    info!("Would reconnect with Last-Event-Id: {} (not implemented)", last_id);
}
// Connection drops permanently!
```

## Impact

- Real-time features break on any network hiccup
- No resilience to temporary network issues
- Poor user experience with dropped updates
- Production reliability issues

## Solution

### Step 1: Add Reconnection Configuration

```rust
#[derive(Debug, Clone)]
pub struct SseReconnectionConfig {
    /// Enable automatic reconnection
    pub enabled: bool,
    /// Maximum number of reconnection attempts
    pub max_retries: u32,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Backoff multiplier (e.g., 2.0 for exponential)
    pub backoff_multiplier: f64,
    /// Jitter to add to backoff (0.0 to 1.0)
    pub jitter_factor: f64,
}

impl Default for SseReconnectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_retries: 5,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}
```

### Step 2: Implement Reconnection Logic

```rust
impl InterceptedSseStream {
    async fn handle_upstream_disconnect(&mut self) -> Result<()> {
        if !self.config.reconnection.enabled {
            return Err(anyhow!("SSE reconnection disabled"));
        }
        
        let mut retry_count = 0;
        let mut backoff = self.config.reconnection.initial_backoff;
        
        while retry_count < self.config.reconnection.max_retries {
            retry_count += 1;
            
            info!(
                "Attempting SSE reconnection {}/{} for session {} after {:?}",
                retry_count,
                self.config.reconnection.max_retries,
                self.session.id,
                backoff
            );
            
            // Wait with backoff
            tokio::time::sleep(self.add_jitter(backoff)).await;
            
            // Attempt reconnection
            match self.reconnect_to_upstream().await {
                Ok(new_stream) => {
                    info!(
                        "Successfully reconnected SSE for session {} after {} attempts",
                        self.session.id, retry_count
                    );
                    
                    self.upstream_stream = new_stream;
                    self.notify_client_of_reconnection().await?;
                    
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        "SSE reconnection attempt {} failed for session {}: {}",
                        retry_count, self.session.id, e
                    );
                    
                    // Update backoff for next attempt
                    backoff = self.calculate_next_backoff(backoff);
                }
            }
        }
        
        error!(
            "Failed to reconnect SSE after {} attempts for session {}",
            retry_count, self.session.id
        );
        
        Err(anyhow!("SSE reconnection failed after {} attempts", retry_count))
    }
    
    async fn reconnect_to_upstream(&mut self) -> Result<Box<dyn Stream<Item = Result<SseEvent>>>> {
        // Build reconnection request with Last-Event-Id
        let mut headers = HeaderMap::new();
        
        if let Some(last_event_id) = &self.session.last_event_id {
            headers.insert(
                "Last-Event-Id",
                HeaderValue::from_str(last_event_id)?
            );
            
            debug!(
                "Reconnecting with Last-Event-Id: {} for session {}",
                last_event_id, self.session.id
            );
        }
        
        // Add session headers
        headers.insert(
            "MCP-Session-Id",
            HeaderValue::from_str(&self.session.id.to_string())?
        );
        
        // Create new upstream connection
        let client = self.create_http_client()?;
        let request = Request::get(&self.upstream_url)
            .headers(headers)
            .body(Body::empty())?;
        
        let response = client.request(request).await?;
        
        if !response.status().is_success() {
            return Err(anyhow!(
                "Upstream returned status {} on reconnection",
                response.status()
            ));
        }
        
        // Parse SSE stream from response
        let stream = SseStream::from_response(response)?;
        Ok(Box::new(stream))
    }
    
    async fn notify_client_of_reconnection(&mut self) -> Result<()> {
        // Send a special event to notify client of reconnection
        let reconnection_event = SseEvent {
            id: Some(Uuid::new_v4().to_string()),
            event: Some("mcp:reconnected".to_string()),
            data: json!({
                "type": "reconnection",
                "session_id": self.session.id.to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "last_event_id": self.session.last_event_id.clone(),
            }).to_string(),
            retry: None,
        };
        
        self.pending_events.push(reconnection_event);
        Ok(())
    }
    
    fn calculate_next_backoff(&self, current: Duration) -> Duration {
        let multiplier = self.config.reconnection.backoff_multiplier;
        let next = current.mul_f64(multiplier);
        next.min(self.config.reconnection.max_backoff)
    }
    
    fn add_jitter(&self, duration: Duration) -> Duration {
        let jitter_range = duration.mul_f64(self.config.reconnection.jitter_factor);
        let jitter = rand::random::<f64>() * jitter_range.as_secs_f64();
        duration + Duration::from_secs_f64(jitter)
    }
}
```

### Step 3: Update Session to Track Event IDs

```rust
impl InterceptedSseStream {
    fn update_session_last_event_id(&mut self, event_id: String) {
        // This requires making session mutable or using Arc<RwLock<Session>>
        // For now, track locally and sync periodically
        self.local_last_event_id = Some(event_id.clone());
        
        // Send update to session manager
        let session_id = self.session.id.clone();
        let manager = self.session_manager.clone();
        
        tokio::spawn(async move {
            if let Err(e) = manager.update_last_event_id(session_id, event_id).await {
                warn!("Failed to update session last_event_id: {}", e);
            }
        });
    }
}
```

### Step 4: Handle Reconnection in Stream Implementation

```rust
impl Stream for InterceptedSseStream {
    type Item = Result<Event>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // ... existing polling logic ...
        
        // Check upstream stream
        match self.upstream_stream.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(event))) => {
                // Update last event ID if present
                if let Some(id) = &event.id {
                    self.update_session_last_event_id(id.clone());
                }
                
                // Process event...
                Poll::Ready(Some(Ok(converted_event)))
            }
            Poll::Ready(None) => {
                // Stream ended - attempt reconnection
                let mut this = self.get_mut();
                let waker = cx.waker().clone();
                
                tokio::spawn(async move {
                    if let Err(e) = this.handle_upstream_disconnect().await {
                        error!("Failed to handle SSE disconnect: {}", e);
                    }
                    waker.wake();
                });
                
                Poll::Pending
            }
            Poll::Ready(Some(Err(e))) => {
                // Error - might need reconnection
                warn!("SSE upstream error: {}", e);
                
                if self.should_reconnect_on_error(&e) {
                    // Trigger reconnection
                    let mut this = self.get_mut();
                    let waker = cx.waker().clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = this.handle_upstream_disconnect().await {
                            error!("Failed to reconnect after error: {}", e);
                        }
                        waker.wake();
                    });
                    
                    Poll::Pending
                } else {
                    Poll::Ready(Some(Err(e.into())))
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
```

## Testing

### Unit Tests
```rust
#[tokio::test]
async fn test_sse_reconnection_with_backoff() {
    let mut stream = create_test_sse_stream();
    
    // Simulate disconnect
    stream.upstream_stream = Box::new(futures::stream::empty());
    
    // Should attempt reconnection
    let result = stream.handle_upstream_disconnect().await;
    assert!(result.is_ok());
    
    // Verify reconnection notification sent
    assert!(stream.pending_events.iter().any(|e| {
        e.event.as_deref() == Some("mcp:reconnected")
    }));
}

#[tokio::test]
async fn test_sse_reconnection_with_last_event_id() {
    let mut stream = create_test_sse_stream();
    stream.session.last_event_id = Some("event-123".to_string());
    
    // Mock upstream to verify Last-Event-Id header
    let mock_upstream = MockUpstream::new();
    mock_upstream.expect_header("Last-Event-Id", "event-123");
    
    stream.reconnect_to_upstream().await.unwrap();
}

#[tokio::test]
async fn test_exponential_backoff() {
    let config = SseReconnectionConfig {
        initial_backoff: Duration::from_millis(100),
        backoff_multiplier: 2.0,
        max_backoff: Duration::from_secs(1),
        ..Default::default()
    };
    
    // Test backoff progression
    let backoffs = calculate_backoff_sequence(&config, 5);
    assert_eq!(backoffs, vec![100ms, 200ms, 400ms, 800ms, 1000ms]);
}
```

### Integration Test
```rust
#[tokio::test]
async fn test_sse_resilience_to_network_issues() {
    let proxy = start_test_proxy().await;
    let upstream = start_flaky_upstream().await; // Drops connection every 10 events
    
    let mut client = SseClient::connect(&proxy.url()).await.unwrap();
    let mut events_received = 0;
    
    // Should receive events despite disconnections
    while events_received < 100 {
        match client.next_event().await {
            Ok(event) => {
                events_received += 1;
                assert!(event.id.is_some());
            }
            Err(e) if e.is_reconnection() => {
                // Expected during reconnection
                continue;
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }
    
    assert_eq!(events_received, 100);
}
```

## Success Criteria

- [ ] SSE automatically reconnects on disconnect
- [ ] Exponential backoff implemented
- [ ] Last-Event-Id header sent on reconnection
- [ ] Client notified of reconnection
- [ ] No duplicate events after reconnection
- [ ] Configurable retry limits
- [ ] Tests pass for various failure scenarios

## Files to Modify

1. `src/proxy/reverse/upstream/http/streaming/intercepted.rs` - Main implementation
2. `src/proxy/reverse/config.rs` - Add SseReconnectionConfig
3. `src/session/manager.rs` - Add update_last_event_id method
4. `tests/integration/sse_resilience_test.rs` - New test file

## Dependencies

- Session manager needs to support last_event_id updates
- HTTP client needs to support custom headers
- Consider using `tokio-retry` crate for backoff

## Risks

- Complexity of handling reconnection in Stream trait
- Potential for infinite reconnection loops
- Memory growth if events queue during reconnection
- Need to handle authentication on reconnection

## Alternative Approach

If full reconnection is too complex, implement a simpler version:
1. Detect disconnect
2. Send error event to client
3. Let client handle reconnection
4. Document as known limitation

This would at least prevent silent failures.