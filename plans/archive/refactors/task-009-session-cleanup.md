# Task 009: Implement Session Cleanup

## Overview
Implement automatic cleanup of old sessions to prevent memory leaks and resource exhaustion.

## Context
From the [comprehensive review](../../reviews/shadowcat-comprehensive-review-2025-08-06.md), the `SessionManager` has a `cleanup_interval` field that is set but never used. Sessions can accumulate indefinitely, leading to memory leaks.

## Scope
- **Files to modify**: `src/session/manager.rs`
- **Priority**: HIGH - Resource management
- **Time estimate**: 0.5 days

## Current Problem

### Unused Cleanup Field
**Location**: `src/session/manager.rs:41`
```rust
cleanup_interval: Duration,  // Set but never used
```

### Missing Cleanup Logic
- No background task to clean up old sessions
- No TTL (time-to-live) for sessions
- No maximum session limit
- Memory can grow unbounded

## Implementation Plan

### Step 1: Add Session Metadata

```rust
#[derive(Debug, Clone)]
pub struct SessionMetadata {
    pub created_at: SystemTime,
    pub last_activity: SystemTime,
    pub message_count: usize,
    pub state: SessionState,
}

impl Session {
    pub fn new(id: String) -> Self {
        Self {
            id,
            metadata: SessionMetadata {
                created_at: SystemTime::now(),
                last_activity: SystemTime::now(),
                message_count: 0,
                state: SessionState::Active,
            },
            // ... other fields
        }
    }
    
    pub fn touch(&mut self) {
        self.metadata.last_activity = SystemTime::now();
        self.metadata.message_count += 1;
    }
    
    pub fn is_stale(&self, max_idle: Duration) -> bool {
        SystemTime::now()
            .duration_since(self.metadata.last_activity)
            .map(|d| d > max_idle)
            .unwrap_or(true)
    }
}
```

### Step 2: Implement Cleanup Task

```rust
impl SessionManager {
    pub async fn start_cleanup_task(self: Arc<Self>) {
        let cleanup_interval = self.config.cleanup_interval;
        let max_idle_time = self.config.max_idle_time.unwrap_or(Duration::from_secs(3600));
        let max_session_age = self.config.max_session_age.unwrap_or(Duration::from_secs(86400));
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            
            loop {
                interval.tick().await;
                
                if let Err(e) = self.cleanup_stale_sessions(max_idle_time, max_session_age).await {
                    tracing::error!("Session cleanup failed: {}", e);
                }
            }
        });
    }
    
    async fn cleanup_stale_sessions(
        &self,
        max_idle: Duration,
        max_age: Duration,
    ) -> Result<(), SessionError> {
        let mut sessions = self.sessions.write()
            .map_err(|e| SessionError::LockPoisoned(e.to_string()))?;
        
        let now = SystemTime::now();
        let mut removed_count = 0;
        let mut removed_ids = Vec::new();
        
        // Find sessions to remove
        sessions.retain(|id, session| {
            // Check if session is stale
            let is_stale = session.is_stale(max_idle);
            
            // Check if session is too old
            let is_expired = now.duration_since(session.metadata.created_at)
                .map(|d| d > max_age)
                .unwrap_or(true);
            
            // Check if session is in terminal state
            let is_terminal = matches!(
                session.metadata.state,
                SessionState::Closed | SessionState::Failed(_)
            );
            
            let should_keep = !is_stale && !is_expired && !is_terminal;
            
            if !should_keep {
                removed_ids.push(id.clone());
                removed_count += 1;
            }
            
            should_keep
        });
        
        if removed_count > 0 {
            tracing::info!(
                "Cleaned up {} stale sessions: {:?}",
                removed_count,
                removed_ids
            );
            
            // Emit metrics
            self.metrics.sessions_cleaned.add(removed_count as u64);
        }
        
        // Log current session count
        tracing::debug!(
            "Active sessions after cleanup: {}",
            sessions.len()
        );
        
        Ok(())
    }
}
```

### Step 3: Add Configuration Options

```rust
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub cleanup_interval: Duration,
    pub max_idle_time: Option<Duration>,
    pub max_session_age: Option<Duration>,
    pub max_sessions: Option<usize>,
    pub cleanup_on_shutdown: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            cleanup_interval: Duration::from_secs(60),  // Clean every minute
            max_idle_time: Some(Duration::from_secs(3600)),  // 1 hour idle
            max_session_age: Some(Duration::from_secs(86400)),  // 24 hours max
            max_sessions: Some(1000),  // Maximum 1000 concurrent sessions
            cleanup_on_shutdown: true,
        }
    }
}
```

### Step 4: Implement Session Limits

```rust
impl SessionManager {
    pub async fn create_session(&self, id: String) -> Result<Arc<Session>, SessionError> {
        // Check session limit
        if let Some(max) = self.config.max_sessions {
            let count = self.sessions.read()
                .map_err(|e| SessionError::LockPoisoned(e.to_string()))?
                .len();
            
            if count >= max {
                // Try emergency cleanup
                self.emergency_cleanup().await?;
                
                // Check again
                let count = self.sessions.read()
                    .map_err(|e| SessionError::LockPoisoned(e.to_string()))?
                    .len();
                
                if count >= max {
                    return Err(SessionError::TooManySessions(max));
                }
            }
        }
        
        // Create new session
        let session = Arc::new(Session::new(id.clone()));
        
        let mut sessions = self.sessions.write()
            .map_err(|e| SessionError::LockPoisoned(e.to_string()))?;
        sessions.insert(id, session.clone());
        
        Ok(session)
    }
    
    async fn emergency_cleanup(&self) -> Result<(), SessionError> {
        tracing::warn!("Emergency cleanup triggered");
        
        // More aggressive cleanup
        let aggressive_idle = Duration::from_secs(300);  // 5 minutes
        let aggressive_age = Duration::from_secs(3600);  // 1 hour
        
        self.cleanup_stale_sessions(aggressive_idle, aggressive_age).await
    }
}
```

### Step 5: Add Graceful Shutdown

```rust
impl SessionManager {
    pub async fn shutdown(&self) -> Result<(), SessionError> {
        tracing::info!("Shutting down session manager");
        
        if self.config.cleanup_on_shutdown {
            // Close all active sessions
            let sessions = self.sessions.read()
                .map_err(|e| SessionError::LockPoisoned(e.to_string()))?;
            
            for (id, session) in sessions.iter() {
                tracing::debug!("Closing session: {}", id);
                // Send shutdown signal to session
                // This would depend on your session implementation
            }
        }
        
        // Final cleanup
        self.cleanup_stale_sessions(Duration::ZERO, Duration::ZERO).await?;
        
        Ok(())
    }
}
```

### Step 6: Add Metrics

```rust
#[derive(Debug, Default)]
struct SessionMetrics {
    sessions_created: AtomicU64,
    sessions_cleaned: AtomicU64,
    sessions_active: AtomicU64,
    cleanup_runs: AtomicU64,
    cleanup_errors: AtomicU64,
}

impl SessionManager {
    pub fn get_metrics(&self) -> SessionStats {
        let sessions = self.sessions.read().ok();
        
        SessionStats {
            total_created: self.metrics.sessions_created.load(Ordering::Relaxed),
            total_cleaned: self.metrics.sessions_cleaned.load(Ordering::Relaxed),
            currently_active: sessions.map(|s| s.len()).unwrap_or(0),
            cleanup_runs: self.metrics.cleanup_runs.load(Ordering::Relaxed),
            cleanup_errors: self.metrics.cleanup_errors.load(Ordering::Relaxed),
        }
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_session_cleanup() {
        let config = SessionConfig {
            cleanup_interval: Duration::from_millis(100),
            max_idle_time: Some(Duration::from_millis(200)),
            ..Default::default()
        };
        
        let manager = Arc::new(SessionManager::new(config));
        
        // Create sessions
        let session1 = manager.create_session("session1".into()).await.unwrap();
        let session2 = manager.create_session("session2".into()).await.unwrap();
        
        // Touch session1 to keep it alive
        session1.touch();
        
        // Wait for cleanup
        tokio::time::sleep(Duration::from_millis(300)).await;
        
        // Run cleanup
        manager.cleanup_stale_sessions(
            Duration::from_millis(200),
            Duration::from_secs(10)
        ).await.unwrap();
        
        // Check that session2 was removed
        let sessions = manager.sessions.read().unwrap();
        assert!(sessions.contains_key("session1"));
        assert!(!sessions.contains_key("session2"));
    }
    
    #[tokio::test]
    async fn test_max_sessions_limit() {
        let config = SessionConfig {
            max_sessions: Some(2),
            ..Default::default()
        };
        
        let manager = SessionManager::new(config);
        
        // Create max sessions
        manager.create_session("s1".into()).await.unwrap();
        manager.create_session("s2".into()).await.unwrap();
        
        // Third should trigger cleanup or fail
        let result = manager.create_session("s3".into()).await;
        
        // Either cleanup happened or we got an error
        assert!(result.is_ok() || matches!(
            result,
            Err(SessionError::TooManySessions(_))
        ));
    }
    
    #[tokio::test]
    async fn test_cleanup_task_runs() {
        let config = SessionConfig {
            cleanup_interval: Duration::from_millis(50),
            max_idle_time: Some(Duration::from_millis(100)),
            ..Default::default()
        };
        
        let manager = Arc::new(SessionManager::new(config));
        manager.clone().start_cleanup_task().await;
        
        // Create a session
        manager.create_session("test".into()).await.unwrap();
        
        // Wait for cleanup to run
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Check metrics
        let metrics = manager.get_metrics();
        assert!(metrics.cleanup_runs > 0);
    }
}
```

## Validation

### Pre-check
```bash
# Check for unused field warning
cargo build 2>&1 | grep "field.*cleanup_interval.*never read"

# Count current sessions in a long-running process
# (Would need to add debug endpoint or logging)
```

### Post-check
```bash
# No unused field warnings
cargo build 2>&1 | grep "field.*cleanup_interval.*never read" | wc -l  # Should be 0

# Run tests
cargo test session::cleanup

# Monitor memory usage during load test
cargo run --release -- forward stdio &
# Run load generator and monitor memory
```

## Success Criteria

- [ ] `cleanup_interval` field is actively used
- [ ] Background cleanup task runs periodically
- [ ] Stale sessions are automatically removed
- [ ] Session limits are enforced
- [ ] Graceful shutdown closes all sessions
- [ ] Memory usage remains bounded under load
- [ ] Metrics track cleanup operations
- [ ] All tests pass

## Configuration Example

```toml
[session]
cleanup_interval_secs = 60
max_idle_time_secs = 3600
max_session_age_secs = 86400
max_sessions = 1000
cleanup_on_shutdown = true
```

## Performance Considerations

1. **Lock contention** - Cleanup should use write lock briefly
2. **Cleanup frequency** - Balance between resource usage and cleanup
3. **Batch operations** - Clean multiple sessions in one pass
4. **Metrics overhead** - Use atomic operations for counters

## Integration Points

- Coordinate with session matching (Task 008)
- Update telemetry/metrics collection
- Consider impact on recording/replay features
- May affect rate limiting implementation

## Notes

- Essential for production deployment
- Prevents memory leaks in long-running processes
- Consider adding admin API to trigger manual cleanup
- Could add session export before cleanup for debugging