# H.2: Add Server Drop Implementation

**Priority**: 🔴 CRITICAL  
**Duration**: 2 hours  
**Status**: ⏳ Pending  

## Problem

ReverseProxyServer has no Drop implementation, leading to resource leaks when the server shuts down or restarts.

**Issues**:
- Spawned tasks continue running (3+ TapeRecorder init tasks)
- Connection pools not properly closed
- Session manager not cleaned up
- Metrics not persisted
- No graceful connection draining

## Solution

### Step 1: Track Background Tasks

```rust
pub struct ReverseProxyServer {
    bind_address: SocketAddr,
    router: Router,
    session_manager: Arc<SessionManager>,
    config: ReverseProxyConfig,
    shutdown_token: ShutdownToken,
    // NEW: Track background tasks
    background_tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
    connection_pool: Option<Arc<ConnectionPool>>,
    tape_recorder: Option<Arc<TapeRecorder>>,
    metrics: Arc<ReverseProxyMetrics>,
}

impl ReverseProxyServerBuilder {
    pub fn build(self) -> Result<ReverseProxyServer> {
        // ... existing code ...
        
        let background_tasks = Arc::new(Mutex::new(Vec::new()));
        
        // Track TapeRecorder initialization
        if let Some(recorder) = &tape_recorder {
            let recorder_clone = recorder.clone();
            let handle = tokio::spawn(async move {
                if let Err(e) = recorder_clone.initialize().await {
                    warn!("Failed to initialize tape recorder: {}", e);
                }
            });
            background_tasks.lock().unwrap().push(handle);
        }
        
        // Track pool maintenance task
        if let Some(pool) = &connection_pool {
            let handle = pool.start_maintenance_task();
            background_tasks.lock().unwrap().push(handle);
        }
        
        // ... rest of build ...
    }
}
```

### Step 2: Implement Drop

```rust
impl Drop for ReverseProxyServer {
    fn drop(&mut self) {
        // Use blocking to ensure cleanup completes
        let rt = tokio::runtime::Handle::try_current();
        
        if let Ok(handle) = rt {
            // We're in async context
            handle.block_on(self.cleanup());
        } else {
            // Create a new runtime for cleanup
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(self.cleanup());
        }
    }
}

impl ReverseProxyServer {
    async fn cleanup(&mut self) {
        info!("Starting ReverseProxyServer cleanup");
        
        // 1. Signal shutdown to all components
        self.shutdown_token.signal();
        
        // 2. Abort background tasks
        if let Ok(mut tasks) = self.background_tasks.lock() {
            for handle in tasks.drain(..) {
                handle.abort();
            }
        }
        
        // 3. Flush tape recorder
        if let Some(recorder) = &self.tape_recorder {
            if let Err(e) = recorder.flush().await {
                error!("Failed to flush tape recorder: {}", e);
            }
        }
        
        // 4. Close connection pools
        if let Some(pool) = &self.connection_pool {
            pool.shutdown().await;
        }
        
        // 5. Persist metrics
        if let Err(e) = self.metrics.persist().await {
            error!("Failed to persist metrics: {}", e);
        }
        
        // 6. Close session manager
        if let Err(e) = self.session_manager.shutdown().await {
            error!("Failed to shutdown session manager: {}", e);
        }
        
        info!("ReverseProxyServer cleanup complete");
    }
}
```

### Step 3: Implement Graceful Shutdown

```rust
impl ReverseProxyServer {
    /// Gracefully shutdown the server with connection draining
    pub async fn shutdown_gracefully(&mut self, timeout: Duration) -> Result<()> {
        info!("Starting graceful shutdown with timeout: {:?}", timeout);
        
        let shutdown_start = Instant::now();
        
        // 1. Stop accepting new connections
        self.shutdown_token.signal();
        
        // 2. Wait for active connections to complete (with timeout)
        let drain_timeout = timeout.saturating_sub(Duration::from_secs(1));
        self.drain_connections(drain_timeout).await?;
        
        // 3. Perform cleanup
        self.cleanup().await;
        
        let elapsed = shutdown_start.elapsed();
        info!("Graceful shutdown completed in {:?}", elapsed);
        
        Ok(())
    }
    
    async fn drain_connections(&self, timeout: Duration) -> Result<()> {
        let start = Instant::now();
        
        while self.session_manager.active_session_count() > 0 {
            if start.elapsed() > timeout {
                warn!(
                    "Timeout draining connections, {} sessions still active",
                    self.session_manager.active_session_count()
                );
                break;
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        Ok(())
    }
}
```

### Step 4: Fix Duplicate State Creation

Remove duplicate AppState creation by extracting a single method:

```rust
impl ReverseProxyServerBuilder {
    fn create_app_state(&self) -> Result<AppState> {
        // Single source of truth for state creation
        let config = self.config.clone().unwrap_or_default();
        let session_manager = Arc::new(self.session_manager.clone().unwrap_or_else(|| {
            SessionManager::new(SessionConfig::default())
        }));
        
        let metrics = Arc::new(ReverseProxyMetrics::new());
        let pool = self.create_connection_pool(&config)?;
        let interceptor_chain = self.interceptor_chain.clone();
        let pause_controller = Arc::new(PauseController::new());
        let event_id_generator = Arc::new(EventIdGenerator::new());
        
        // Initialize recorder ONCE
        let recorder = if let Some(recorder_config) = &config.recorder {
            Some(Arc::new(TapeRecorder::new(recorder_config.clone())))
        } else {
            None
        };
        
        Ok(AppState {
            config,
            session_manager,
            metrics,
            connection_pool: pool,
            interceptor_chain,
            pause_controller,
            recorder,
            event_id_generator,
            upstream_selector: Arc::new(self.create_upstream_selector(&config)?),
        })
    }
}
```

## Testing

### Unit Test
```rust
#[tokio::test]
async fn test_server_cleanup_on_drop() {
    let server = ReverseProxyServer::builder()
        .bind_address("127.0.0.1:0")
        .build()
        .unwrap();
    
    // Get metrics before drop
    let initial_tasks = count_background_tasks();
    
    // Drop server (should trigger cleanup)
    drop(server);
    
    // Verify cleanup occurred
    tokio::time::sleep(Duration::from_millis(100)).await;
    let final_tasks = count_background_tasks();
    
    assert!(final_tasks < initial_tasks, "Background tasks should be cleaned up");
}

#[tokio::test]
async fn test_graceful_shutdown() {
    let mut server = create_test_server();
    
    // Create some active sessions
    for _ in 0..10 {
        create_test_session(&server).await;
    }
    
    // Graceful shutdown
    server.shutdown_gracefully(Duration::from_secs(5)).await.unwrap();
    
    // Verify all sessions closed
    assert_eq!(server.session_manager.active_session_count(), 0);
}
```

## Success Criteria

- [ ] All spawned tasks tracked and aborted on drop
- [ ] Connection pools properly closed
- [ ] Tape recorder flushed
- [ ] Metrics persisted
- [ ] Session manager shutdown cleanly
- [ ] No resource leaks after shutdown
- [ ] Graceful shutdown drains connections

## Files to Modify

1. `src/proxy/reverse/server.rs` - Add Drop impl and cleanup
2. `src/proxy/reverse/state.rs` - Add background task tracking
3. `src/session/manager.rs` - Add shutdown method if missing
4. `src/proxy/reverse/metrics.rs` - Add persist method

## Dependencies

- H.3 (Deduplicate AppState) should be done alongside this

## Notes

- Use `tokio::runtime::Handle::try_current()` to detect if in async context
- Consider using `futures::future::join_all` for parallel cleanup
- Add timeout to prevent hanging during cleanup
- Log all cleanup failures but don't panic