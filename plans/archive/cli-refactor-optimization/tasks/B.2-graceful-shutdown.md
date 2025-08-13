# Task B.2: Add Graceful Shutdown

## Objective
Implement comprehensive graceful shutdown handling for all Shadowcat components, ensuring clean resource cleanup, session persistence, and proper termination of async tasks when receiving shutdown signals (Ctrl+C, SIGTERM).

## Background
Current issues:
- No cleanup on Ctrl+C (resources may leak)
- Sessions not saved on termination
- Async tasks may be abruptly killed
- No way to programmatically shutdown
- Recording may be incomplete

Graceful shutdown is critical for:
- Production reliability
- Data integrity (recordings, sessions)
- Resource cleanup (connections, files)
- Clean termination in containers/systemd

## Key Questions to Answer
1. How do we handle shutdown across different components?
2. Should shutdown be cooperative or forced after timeout?
3. How do we save session state on shutdown?
4. What signals should trigger shutdown?

## Step-by-Step Process

### 1. Create Shutdown Controller
```rust
// src/shutdown.rs (new file)
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Controls graceful shutdown of the application
pub struct ShutdownController {
    shutdown_tx: broadcast::Sender<()>,
    tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
    cleanup_hooks: Arc<Mutex<Vec<Box<dyn FnOnce() + Send>>>>,
}

impl ShutdownController {
    pub fn new() -> (Self, ShutdownToken) {
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        
        let controller = Self {
            shutdown_tx: shutdown_tx.clone(),
            tasks: Arc::new(Mutex::new(Vec::new())),
            cleanup_hooks: Arc::new(Mutex::new(Vec::new())),
        };
        
        let token = ShutdownToken {
            shutdown_rx,
            shutdown_tx,
        };
        
        (controller, token)
    }
    
    /// Register a task to be tracked for shutdown
    pub fn register_task(&self, task: JoinHandle<()>) {
        self.tasks.lock().unwrap().push(task);
    }
    
    /// Register a cleanup hook to run on shutdown
    pub fn register_cleanup<F>(&self, cleanup: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.cleanup_hooks.lock().unwrap().push(Box::new(cleanup));
    }
    
    /// Initiate graceful shutdown
    pub async fn shutdown(self, timeout: Duration) -> Result<()> {
        info!("Initiating graceful shutdown...");
        
        // Signal all components to shutdown
        let _ = self.shutdown_tx.send(());
        
        // Run cleanup hooks
        let hooks = self.cleanup_hooks.lock().unwrap().drain(..).collect::<Vec<_>>();
        for hook in hooks {
            hook();
        }
        
        // Wait for tasks with timeout
        let tasks = self.tasks.lock().unwrap().drain(..).collect::<Vec<_>>();
        
        if !tasks.is_empty() {
            info!("Waiting for {} tasks to complete...", tasks.len());
            
            let wait_future = async {
                for task in tasks {
                    let _ = task.await;
                }
            };
            
            match tokio::time::timeout(timeout, wait_future).await {
                Ok(_) => info!("All tasks completed successfully"),
                Err(_) => {
                    warn!("Shutdown timeout reached, forcing termination");
                    // Tasks will be dropped, causing cancellation
                }
            }
        }
        
        info!("Shutdown complete");
        Ok(())
    }
}

/// Token for receiving shutdown signals
#[derive(Clone)]
pub struct ShutdownToken {
    shutdown_rx: broadcast::Receiver<()>,
    shutdown_tx: broadcast::Sender<()>,
}

impl ShutdownToken {
    /// Wait for shutdown signal
    pub async fn wait(&mut self) {
        let _ = self.shutdown_rx.recv().await;
    }
    
    /// Check if shutdown has been signaled
    pub fn is_shutdown(&mut self) -> bool {
        self.shutdown_rx.try_recv().is_ok()
    }
    
    /// Create a child token
    pub fn child(&self) -> Self {
        Self {
            shutdown_rx: self.shutdown_tx.subscribe(),
            shutdown_tx: self.shutdown_tx.clone(),
        }
    }
}
```

### 2. Integrate with Session Manager
```rust
// src/session/manager.rs (modify)
impl SessionManager {
    pub async fn run_with_shutdown(
        self: Arc<Self>,
        mut shutdown: ShutdownToken,
    ) -> Result<()> {
        let cleanup_interval = Duration::from_secs(60);
        
        loop {
            tokio::select! {
                _ = tokio::time::sleep(cleanup_interval) => {
                    self.cleanup_expired_sessions().await?;
                }
                _ = shutdown.wait() => {
                    info!("Session manager shutting down...");
                    self.save_all_sessions().await?;
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn save_all_sessions(&self) -> Result<()> {
        let sessions = self.sessions.read().await;
        info!("Saving {} active sessions", sessions.len());
        
        for (id, session) in sessions.iter() {
            if let Err(e) = self.persist_session(id, session).await {
                error!("Failed to save session {}: {}", id, e);
            }
        }
        
        Ok(())
    }
}
```

### 3. Update Proxy Components
```rust
// src/proxy/forward.rs (modify)
impl ForwardProxy {
    pub async fn run_with_shutdown(
        self,
        client: impl Transport,
        server: impl Transport,
        mut shutdown: ShutdownToken,
    ) -> Result<()> {
        let (client_tx, mut client_rx) = mpsc::channel(100);
        let (server_tx, mut server_rx) = mpsc::channel(100);
        
        // Spawn forwarding tasks
        let client_task = tokio::spawn({
            let mut shutdown = shutdown.child();
            async move {
                loop {
                    tokio::select! {
                        msg = client.receive() => {
                            // Handle message
                        }
                        _ = shutdown.wait() => {
                            debug!("Client forwarding shutting down");
                            break;
                        }
                    }
                }
            }
        });
        
        // Similar for server task...
        
        // Wait for shutdown
        shutdown.wait().await;
        
        // Clean shutdown of transports
        info!("Closing client transport...");
        client.close().await?;
        
        info!("Closing server transport...");
        server.close().await?;
        
        // Wait for tasks
        client_task.await?;
        server_task.await?;
        
        Ok(())
    }
}
```

### 4. Add Signal Handlers in Main
```rust
// src/main.rs (modify)
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    init_logging(cli.log_level, cli.verbose);
    
    // Set up shutdown handling
    let (shutdown_controller, shutdown_token) = ShutdownController::new();
    
    // Spawn signal handler
    tokio::spawn(async move {
        match signal_handler().await {
            Ok(_) => info!("Received shutdown signal"),
            Err(e) => error!("Signal handler error: {}", e),
        }
    });
    
    // Execute command with shutdown support
    let result = match cli.command {
        Commands::Forward(cmd) => {
            run_with_shutdown(cmd.execute_with_shutdown(shutdown_token), shutdown_controller).await
        }
        Commands::Reverse(cmd) => {
            run_with_shutdown(cmd.execute_with_shutdown(shutdown_token), shutdown_controller).await
        }
        // ... other commands
    };
    
    result
}

async fn signal_handler() -> Result<()> {
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;
    
    tokio::select! {
        _ = sigint.recv() => {
            info!("Received SIGINT (Ctrl+C)");
        }
        _ = sigterm.recv() => {
            info!("Received SIGTERM");
        }
    }
    
    Ok(())
}

async fn run_with_shutdown<F>(
    command_future: F,
    shutdown_controller: ShutdownController,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    let command_handle = tokio::spawn(command_future);
    shutdown_controller.register_task(command_handle);
    
    // Wait for signal
    tokio::select! {
        result = command_handle => {
            result??
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received");
            shutdown_controller.shutdown(Duration::from_secs(30)).await?;
        }
    }
    
    Ok(())
}
```

### 5. Update Recorder for Clean Shutdown
```rust
// src/recorder/mod.rs (modify)
impl Recorder {
    pub async fn shutdown(&self) -> Result<()> {
        info!("Flushing recorder buffers...");
        
        // Flush any pending writes
        self.flush_buffers().await?;
        
        // Close tape file
        if let Some(mut file) = self.tape_file.lock().await.take() {
            file.flush().await?;
            file.sync_all().await?;
            info!("Tape file closed successfully");
        }
        
        Ok(())
    }
}
```

### 6. Add Shutdown Tests
```rust
// tests/shutdown.rs
#[tokio::test]
async fn test_graceful_shutdown() {
    let (controller, mut token) = ShutdownController::new();
    
    // Spawn a task that waits for shutdown
    let task = tokio::spawn(async move {
        token.wait().await;
        println!("Task received shutdown signal");
    });
    
    controller.register_task(task);
    
    // Trigger shutdown
    controller.shutdown(Duration::from_secs(5)).await.unwrap();
}

#[tokio::test]
async fn test_shutdown_timeout() {
    let (controller, mut token) = ShutdownController::new();
    
    // Spawn a task that doesn't respect shutdown
    let task = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });
    
    controller.register_task(task);
    
    // Should timeout and force termination
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        controller.shutdown(Duration::from_secs(1))
    ).await;
    
    assert!(result.is_ok());
}
```

## Expected Deliverables

### New Files
- `shadowcat/src/shutdown.rs` - Shutdown controller and token
- `shadowcat/tests/shutdown.rs` - Shutdown tests

### Modified Files
- `shadowcat/src/main.rs` - Signal handling and shutdown orchestration
- `shadowcat/src/proxy/forward.rs` - Shutdown support
- `shadowcat/src/proxy/reverse.rs` - Shutdown support
- `shadowcat/src/session/manager.rs` - Session persistence on shutdown
- `shadowcat/src/recorder/mod.rs` - Buffer flushing on shutdown
- `shadowcat/src/lib.rs` - Export shutdown module

### Verification Commands
```bash
# Test Ctrl+C handling
cargo run -- forward stdio -- sleep 30
# Press Ctrl+C and verify graceful shutdown

# Test with recording
cargo run -- record stdio --output test.tape -- sleep 30
# Press Ctrl+C and verify tape is properly closed

# Run shutdown tests
cargo test shutdown
```

## Success Criteria Checklist
- [ ] Ctrl+C triggers graceful shutdown
- [ ] All sessions saved on shutdown
- [ ] Recordings properly closed
- [ ] Async tasks terminated cleanly
- [ ] Shutdown completes within timeout
- [ ] Resources properly cleaned up
- [ ] No data loss on shutdown

## Risk Assessment
- **Risk**: Deadlock during shutdown
  - **Mitigation**: Use timeouts for all operations
  - **Mitigation**: Avoid holding locks during async operations

- **Risk**: Data loss if shutdown is too aggressive
  - **Mitigation**: Generous default timeout (30s)
  - **Mitigation**: Save critical data first

## Duration Estimate
**4 hours**
- 1 hour: Implement shutdown controller
- 1 hour: Integrate with components
- 1 hour: Add signal handling
- 30 min: Testing
- 30 min: Documentation

## Dependencies
- B.1: Builder patterns (for clean initialization)

## Notes
- Consider adding shutdown hooks for plugins later
- May want to add shutdown metrics
- Consider different shutdown strategies (graceful vs immediate)
- Important for container/Kubernetes deployments

## Commands Reference
```bash
cd /Users/kevin/src/tapwire/shadowcat-cli-refactor

# Test signal handling
cargo build --release
./target/release/shadowcat forward stdio -- sleep 100 &
PID=$!
sleep 2
kill -TERM $PID  # Should shutdown gracefully

# Test in Docker
docker run --rm -it shadowcat
# Ctrl+C should shutdown cleanly
```