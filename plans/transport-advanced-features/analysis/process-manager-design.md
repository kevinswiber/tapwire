# ProcessManager Integration Design

## Overview

This document outlines the design for integrating ProcessManager with the transport layer, specifically focusing on SubprocessOutgoing and StdioRawOutgoing. The integration aims to improve subprocess lifecycle management, monitoring, and reliability without breaking existing functionality.

## Design Goals

1. **Backward Compatibility**: Existing code continues to work without modification
2. **Opt-in Enhancement**: ProcessManager is optional, with fallback to current behavior
3. **Minimal Performance Impact**: No overhead when ProcessManager is not used
4. **Clean Separation**: Transport logic remains separate from process management
5. **Future Extensibility**: Support for pooling and advanced features

## Architecture

### Component Relationships

```
SubprocessOutgoing
    ├── ProcessManager (optional)
    ├── StdioRawOutgoing
    │   ├── Child (when no ProcessManager)
    │   └── ProcessHandle (when ProcessManager present)
    └── Protocol Handler
```

### Integration Points

#### 1. SubprocessOutgoing Enhancement

```rust
pub struct SubprocessOutgoing {
    raw: StdioRawOutgoing,
    protocol: Arc<dyn ProtocolHandler>,
    session_id: SessionId,
    command: String,
    max_message_size: usize,
    process_manager: Option<Arc<dyn ProcessManager>>, // NEW
    process_handle: Option<ProcessHandle>,            // NEW
}
```

**Key Changes:**
- Optional ProcessManager injection
- Track ProcessHandle for managed processes
- Maintain backward compatibility when ProcessManager is None

#### 2. StdioRawOutgoing Modification

```rust
pub struct StdioRawOutgoing {
    // Existing fields...
    child: Option<Child>,
    
    // New fields for ProcessManager integration
    process_manager: Option<Arc<dyn ProcessManager>>,
    managed_handle: Option<ProcessHandle>,
}
```

**Spawning Logic:**
```rust
async fn connect(&mut self) -> TransportResult<()> {
    if let Some(manager) = &mut self.process_manager {
        // Managed spawning
        let handle = manager.spawn(command).await?;
        self.managed_handle = Some(handle);
        // Extract Child from manager for I/O setup
    } else {
        // Current direct spawning
        let child = command.spawn()?;
        self.child = Some(child);
    }
    // Setup I/O channels...
}
```

## Implementation Plan

### Phase 1: Core Integration (Immediate)

#### Step 1.1: Add ProcessManager Support to SubprocessOutgoing

```rust
impl SubprocessOutgoing {
    /// Create with ProcessManager for enhanced lifecycle management
    pub fn with_process_manager(
        command: String,
        manager: Arc<dyn ProcessManager>
    ) -> TransportResult<Self> {
        // Parse command
        let (program, args) = parse_command(&command)?;
        
        // Create raw transport with manager
        let raw = StdioRawOutgoing::with_process_manager(
            program,
            args,
            manager.clone(),
            RawTransportConfig::default()
        );
        
        Ok(Self {
            raw,
            protocol: Arc::new(McpProtocolHandler::new()),
            session_id: SessionId::new(),
            command,
            max_message_size: DEFAULT_MAX_MESSAGE_SIZE,
            process_manager: Some(manager),
            process_handle: None,
        })
    }
}
```

#### Step 1.2: Modify StdioRawOutgoing for Managed Spawning

```rust
impl StdioRawOutgoing {
    pub fn with_process_manager(
        program: String,
        args: Vec<String>,
        manager: Arc<dyn ProcessManager>,
        config: RawTransportConfig,
    ) -> Self {
        Self {
            process_manager: Some(manager),
            managed_handle: None,
            // ... other fields
        }
    }
    
    async fn spawn_process(&mut self) -> TransportResult<(ChildStdin, ChildStdout, ChildStderr)> {
        if let Some(manager) = &mut self.process_manager {
            // Managed spawning
            let mut command = Command::new(&self.command_program);
            command.args(&self.command_args);
            let handle = manager.spawn(command).await?;
            
            // Get I/O handles from managed process
            // Note: This requires ProcessManager to expose Child I/O
            let (stdin, stdout, stderr) = self.get_managed_io(&handle)?;
            self.managed_handle = Some(handle);
            
            Ok((stdin, stdout, stderr))
        } else {
            // Current direct spawning
            let mut child = self.create_command().spawn()?;
            let stdin = child.stdin.take().ok_or(...)?;
            let stdout = child.stdout.take().ok_or(...)?;
            let stderr = child.stderr.take().ok_or(...)?;
            self.child = Some(child);
            
            Ok((stdin, stdout, stderr))
        }
    }
}
```

#### Step 1.3: Implement Graceful Shutdown

```rust
impl StdioRawOutgoing {
    async fn close(&mut self) -> TransportResult<()> {
        self.connected = false;
        
        // Abort I/O tasks first
        self.abort_io_tasks();
        
        if let Some(manager) = &mut self.process_manager {
            if let Some(handle) = self.managed_handle.take() {
                // Graceful managed termination
                manager.terminate(handle).await?;
            }
        } else if let Some(mut child) = self.child.take() {
            // Enhanced termination for unmanaged processes
            #[cfg(unix)]
            {
                // Try SIGTERM first
                if let Some(pid) = child.id() {
                    let _ = send_sigterm(pid);
                    
                    // Wait with timeout
                    let timeout = Duration::from_secs(5);
                    match timeout(timeout, child.wait()).await {
                        Ok(Ok(_)) => {
                            info!("Process terminated gracefully");
                            return Ok(());
                        }
                        _ => {} // Fall through to force kill
                    }
                }
            }
            
            // Force kill as fallback
            let _ = child.kill().await;
        }
        
        Ok(())
    }
}
```

### Phase 2: Monitoring and Health Checks

#### Step 2.1: Add Health Monitoring Task

```rust
impl SubprocessOutgoing {
    async fn start_health_monitoring(&mut self) {
        if let Some(manager) = self.process_manager.clone() {
            if let Some(handle) = self.process_handle.clone() {
                let monitoring_handle = tokio::spawn(async move {
                    let mut interval = tokio::time::interval(Duration::from_secs(5));
                    
                    loop {
                        interval.tick().await;
                        let status = manager.status(&handle).await;
                        
                        match status {
                            ProcessStatus::Failed { reason } => {
                                error!("Process failed: {}", reason);
                                // Trigger recovery or notification
                                break;
                            }
                            ProcessStatus::Terminated { .. } => {
                                warn!("Process terminated unexpectedly");
                                break;
                            }
                            _ => {
                                debug!("Process health check: {:?}", status);
                            }
                        }
                    }
                });
                
                self.monitoring_handle = Some(monitoring_handle);
            }
        }
    }
}
```

#### Step 2.2: Expose Process Status

```rust
impl SubprocessOutgoing {
    /// Get current process status if managed
    pub async fn process_status(&self) -> Option<ProcessStatus> {
        if let (Some(manager), Some(handle)) = (&self.process_manager, &self.process_handle) {
            Some(manager.status(handle).await)
        } else {
            None
        }
    }
    
    /// Check if subprocess is healthy
    pub async fn is_healthy(&self) -> bool {
        if let Some(status) = self.process_status().await {
            matches!(status, ProcessStatus::Running | ProcessStatus::Starting)
        } else {
            // Fallback to connection check for unmanaged
            self.raw.is_connected()
        }
    }
}
```

### Phase 3: Recovery and Restart

#### Step 3.1: Auto-Recovery on Failure

```rust
impl SubprocessOutgoing {
    async fn handle_process_failure(&mut self) -> TransportResult<()> {
        if let Some(manager) = &mut self.process_manager {
            if let Some(handle) = self.process_handle.take() {
                warn!("Attempting to restart failed process");
                
                // Attempt restart with exponential backoff
                for attempt in 1..=3 {
                    let delay = Duration::from_secs(2_u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                    
                    match manager.restart(handle.clone()).await {
                        Ok(new_handle) => {
                            info!("Process restarted successfully");
                            self.process_handle = Some(new_handle);
                            
                            // Reconnect I/O channels
                            self.reconnect_io().await?;
                            return Ok(());
                        }
                        Err(e) => {
                            error!("Restart attempt {} failed: {}", attempt, e);
                        }
                    }
                }
                
                Err(TransportError::ProcessRecoveryFailed)
            } else {
                Err(TransportError::NoProcessHandle)
            }
        } else {
            // No recovery for unmanaged processes
            Err(TransportError::ProcessFailed)
        }
    }
}
```

## Configuration

### ProcessManager Configuration

```rust
pub struct ProcessManagerConfig {
    /// Enable process management
    pub enabled: bool,
    
    /// Graceful shutdown timeout
    pub graceful_timeout_ms: u64,
    
    /// Health check interval
    pub health_check_interval_ms: u64,
    
    /// Enable auto-recovery
    pub auto_recovery: bool,
    
    /// Max recovery attempts
    pub max_recovery_attempts: u32,
    
    /// Recovery backoff multiplier
    pub recovery_backoff_factor: f64,
}

impl Default for ProcessManagerConfig {
    fn default() -> Self {
        Self {
            enabled: false,  // Opt-in
            graceful_timeout_ms: 5000,
            health_check_interval_ms: 5000,
            auto_recovery: false,
            max_recovery_attempts: 3,
            recovery_backoff_factor: 2.0,
        }
    }
}
```

## Testing Strategy

### Unit Tests

1. **Backward Compatibility Tests**
   - Verify existing code works without ProcessManager
   - Test direct spawning path

2. **ProcessManager Integration Tests**
   - Mock ProcessManager for controlled testing
   - Verify spawning, termination, restart flows

3. **Health Monitoring Tests**
   - Test status reporting
   - Verify failure detection

### Integration Tests

1. **End-to-End Process Lifecycle**
   - Spawn real process with ProcessManager
   - Test communication and termination
   - Verify graceful shutdown

2. **Recovery Scenarios**
   - Simulate process crashes
   - Test auto-recovery mechanism
   - Verify backoff behavior

## Migration Path

### Step 1: Deploy with Opt-in
- Release with ProcessManager support disabled by default
- Allow early adopters to enable via configuration

### Step 2: Gradual Rollout
- Enable for specific use cases
- Monitor performance and stability
- Gather feedback

### Step 3: Default Enable
- Make ProcessManager default for new instances
- Maintain backward compatibility flag

## Success Metrics

1. **Reliability**
   - Reduced subprocess crashes
   - Successful recovery rate > 90%
   - Clean shutdown rate > 95%

2. **Performance**
   - No measurable latency increase
   - Memory overhead < 1MB per process
   - CPU overhead < 1%

3. **Observability**
   - Process status visibility
   - Lifecycle event tracking
   - Resource usage metrics

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking existing code | HIGH | Extensive backward compatibility testing |
| Performance regression | MEDIUM | Benchmarking and profiling |
| Complex error handling | MEDIUM | Clear error types and recovery strategies |
| ProcessManager bugs | LOW | Comprehensive testing, gradual rollout |

## Future Enhancements

1. **Process Pooling**
   - Pre-spawn processes for performance
   - Connection reuse patterns

2. **Resource Limits**
   - Memory and CPU limits
   - Automatic throttling

3. **Distributed Process Management**
   - Cross-node process tracking
   - Cluster-wide health monitoring

## Conclusion

The ProcessManager integration provides a robust foundation for subprocess lifecycle management while maintaining full backward compatibility. The phased approach allows incremental adoption with minimal risk. Initial implementation focuses on core integration and graceful shutdown, with monitoring and recovery features following in subsequent phases.