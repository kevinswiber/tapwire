# Process Management Design for E2E Testing

## Overview
A robust process management system is critical for E2E testing. It must handle multiple concurrent processes (proxy, upstream servers, clients), ensure proper cleanup, capture logs, and provide health monitoring.

## Core Components

### 1. Process Lifecycle Manager

```rust
use tokio::process::{Child, Command};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ProcessManager {
    processes: Arc<RwLock<HashMap<ProcessId, ManagedProcess>>>,
    shutdown_token: CancellationToken,
    cleanup_hooks: Vec<Box<dyn Fn() + Send + Sync>>,
}

pub struct ManagedProcess {
    id: ProcessId,
    name: String,
    child: Child,
    stdout_capture: JoinHandle<Vec<String>>,
    stderr_capture: JoinHandle<Vec<String>>,
    health_monitor: JoinHandle<()>,
    restart_policy: RestartPolicy,
    start_time: Instant,
    restart_count: u32,
}

#[derive(Debug, Clone)]
pub enum RestartPolicy {
    Never,
    OnFailure { max_attempts: u32, backoff_ms: u64 },
    Always { delay_ms: u64 },
}
```

### 2. Process Spawning Strategy

```rust
impl ProcessManager {
    pub async fn spawn_process(&self, config: ProcessConfig) -> Result<ProcessId> {
        let id = ProcessId::new();
        
        // Build command with environment and working directory
        let mut cmd = Command::new(&config.program);
        cmd.args(&config.args)
           .envs(&config.env_vars)
           .current_dir(&config.working_dir)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped())
           .kill_on_drop(true); // Critical for cleanup
        
        // Spawn the process
        let mut child = cmd.spawn()
            .context("Failed to spawn process")?;
        
        // Set up stdout/stderr capture
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        
        let stdout_capture = tokio::spawn(capture_stream(stdout));
        let stderr_capture = tokio::spawn(capture_stream(stderr));
        
        // Start health monitoring
        let health_monitor = tokio::spawn(
            monitor_health(id, config.health_check)
        );
        
        // Register the process
        let managed = ManagedProcess {
            id,
            name: config.name,
            child,
            stdout_capture,
            stderr_capture,
            health_monitor,
            restart_policy: config.restart_policy,
            start_time: Instant::now(),
            restart_count: 0,
        };
        
        self.processes.write().await.insert(id, managed);
        
        Ok(id)
    }
}
```

### 3. Health Monitoring

```rust
#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> Result<HealthStatus>;
    fn interval(&self) -> Duration;
    fn timeout(&self) -> Duration;
}

pub struct HttpHealthCheck {
    url: String,
    expected_status: StatusCode,
    interval: Duration,
    timeout: Duration,
}

pub struct StdioHealthCheck {
    ping_message: String,
    expected_response: String,
    interval: Duration,
    timeout: Duration,
}

async fn monitor_health(
    process_id: ProcessId,
    health_check: Box<dyn HealthCheck>
) {
    let mut interval = tokio::time::interval(health_check.interval());
    let mut consecutive_failures = 0;
    
    loop {
        interval.tick().await;
        
        match timeout(health_check.timeout(), health_check.check()).await {
            Ok(Ok(HealthStatus::Healthy)) => {
                consecutive_failures = 0;
                debug!("Process {} is healthy", process_id);
            }
            Ok(Ok(HealthStatus::Degraded)) => {
                warn!("Process {} is degraded", process_id);
            }
            _ => {
                consecutive_failures += 1;
                error!("Process {} health check failed ({} consecutive)", 
                       process_id, consecutive_failures);
                
                if consecutive_failures >= 3 {
                    // Trigger restart based on policy
                    handle_unhealthy_process(process_id).await;
                }
            }
        }
    }
}
```

### 4. Log Capture System

```rust
pub struct LogCapture {
    process_id: ProcessId,
    buffer: Arc<RwLock<CircularBuffer<LogEntry>>>,
    patterns: Vec<LogPattern>,
    alerts: Arc<RwLock<Vec<LogAlert>>>,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    message: String,
    source: LogSource,
}

async fn capture_stream(
    mut reader: impl AsyncBufRead + Unpin
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    
    while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
        // Parse and categorize log line
        let entry = parse_log_line(&line);
        
        // Check for critical patterns
        if entry.level == LogLevel::Error {
            error!("Process error: {}", entry.message);
        }
        
        lines.push(line.clone());
        line.clear();
        
        // Limit buffer size to prevent memory issues
        if lines.len() > 10000 {
            lines.drain(0..5000);
        }
    }
    
    lines
}
```

### 5. Graceful Shutdown

```rust
impl ProcessManager {
    pub async fn shutdown_all(&self, timeout: Duration) -> Result<()> {
        info!("Initiating graceful shutdown of all processes");
        
        let processes = self.processes.read().await;
        let shutdown_futures = processes.values().map(|p| {
            self.shutdown_process(p.id, timeout)
        });
        
        // Shutdown in parallel with timeout
        let results = timeout(
            timeout,
            futures::future::join_all(shutdown_futures)
        ).await;
        
        match results {
            Ok(results) => {
                for result in results {
                    if let Err(e) = result {
                        error!("Process shutdown error: {}", e);
                    }
                }
            }
            Err(_) => {
                warn!("Shutdown timeout exceeded, force killing remaining processes");
                self.force_kill_all().await?;
            }
        }
        
        // Run cleanup hooks
        for hook in &self.cleanup_hooks {
            hook();
        }
        
        Ok(())
    }
    
    async fn shutdown_process(&self, id: ProcessId, timeout: Duration) -> Result<()> {
        let mut processes = self.processes.write().await;
        
        if let Some(mut process) = processes.remove(&id) {
            // Try graceful shutdown first
            if cfg!(unix) {
                process.child.kill_with(Signal::SIGTERM)?;
            } else {
                process.child.kill()?;
            }
            
            // Wait for process to exit
            match timeout(timeout, process.child.wait()).await {
                Ok(Ok(status)) => {
                    info!("Process {} exited with status: {:?}", id, status);
                }
                Ok(Err(e)) => {
                    error!("Error waiting for process {}: {}", id, e);
                }
                Err(_) => {
                    warn!("Process {} didn't exit gracefully, force killing", id);
                    process.child.kill()?;
                }
            }
            
            // Collect logs
            let stdout = process.stdout_capture.await?;
            let stderr = process.stderr_capture.await?;
            
            // Save logs for debugging
            self.save_process_logs(id, stdout, stderr).await?;
        }
        
        Ok(())
    }
}
```

### 6. Process Types for E2E Testing

```rust
pub enum ProcessType {
    ShadowcatProxy {
        mode: ProxyMode,
        bind_address: String,
        upstream: String,
        auth: Option<String>,
    },
    McpValidator {
        port: u16,
        auth_token: Option<String>,
    },
    MockUpstream {
        port: u16,
        response_delay: Duration,
    },
    TestClient {
        target: String,
        scenario: String,
    },
}

impl ProcessType {
    pub fn to_process_config(&self) -> ProcessConfig {
        match self {
            ProcessType::ShadowcatProxy { mode, bind_address, upstream, auth } => {
                let mut args = vec![];
                
                match mode {
                    ProxyMode::Forward => {
                        args.push("forward".to_string());
                        args.push("http".to_string());
                        args.push("--port".to_string());
                        args.push(bind_address.to_string());
                    }
                    ProxyMode::Reverse => {
                        args.push("reverse".to_string());
                        args.push("--bind".to_string());
                        args.push(bind_address.to_string());
                        args.push("--upstream".to_string());
                        args.push(upstream.to_string());
                    }
                }
                
                ProcessConfig {
                    name: "shadowcat".to_string(),
                    program: "./target/release/shadowcat".to_string(),
                    args,
                    env_vars: auth.map(|a| {
                        vec![("SHADOWCAT_UPSTREAM_AUTH".to_string(), a)]
                    }).unwrap_or_default(),
                    working_dir: PathBuf::from("."),
                    health_check: Box::new(HttpHealthCheck {
                        url: format!("http://{}/health", bind_address),
                        expected_status: StatusCode::OK,
                        interval: Duration::from_secs(5),
                        timeout: Duration::from_secs(2),
                    }),
                    restart_policy: RestartPolicy::OnFailure {
                        max_attempts: 3,
                        backoff_ms: 1000,
                    },
                }
            }
            // ... other process types
        }
    }
}
```

## Resource Management

### 1. Resource Limits

```rust
#[cfg(unix)]
pub fn set_resource_limits(child: &mut Child) -> Result<()> {
    use nix::sys::resource::{setrlimit, Resource, Rlimit};
    
    // Limit memory to 500MB
    setrlimit(
        Resource::RLIMIT_AS,
        &Rlimit {
            rlim_cur: 500 * 1024 * 1024,
            rlim_max: 500 * 1024 * 1024,
        }
    )?;
    
    // Limit file descriptors
    setrlimit(
        Resource::RLIMIT_NOFILE,
        &Rlimit {
            rlim_cur: 1024,
            rlim_max: 1024,
        }
    )?;
    
    Ok(())
}
```

### 2. Cleanup Guards

```rust
pub struct ProcessGuard {
    manager: Arc<ProcessManager>,
    process_id: ProcessId,
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        // Ensure process is killed even if test panics
        let manager = self.manager.clone();
        let id = self.process_id;
        
        tokio::spawn(async move {
            if let Err(e) = manager.shutdown_process(id, Duration::from_secs(5)).await {
                error!("Failed to cleanup process {}: {}", id, e);
            }
        });
    }
}
```

## Integration with Test Framework

### 1. Test Harness Integration

```rust
pub struct E2ETestHarness {
    process_manager: Arc<ProcessManager>,
    port_allocator: PortAllocator,
    log_aggregator: LogAggregator,
}

impl E2ETestHarness {
    pub async fn start_shadowcat_proxy(
        &self,
        config: ProxyConfig
    ) -> Result<ProxyHandle> {
        // Allocate port
        let port = self.port_allocator.allocate().await?;
        
        // Start process
        let process_config = ProcessType::ShadowcatProxy {
            mode: config.mode,
            bind_address: format!("127.0.0.1:{}", port),
            upstream: config.upstream,
            auth: config.auth,
        }.to_process_config();
        
        let process_id = self.process_manager
            .spawn_process(process_config)
            .await?;
        
        // Wait for health check
        self.wait_for_ready(process_id, Duration::from_secs(10)).await?;
        
        Ok(ProxyHandle {
            process_id,
            port,
            url: format!("http://127.0.0.1:{}", port),
        })
    }
}
```

### 2. Test Pattern

```rust
#[tokio::test]
async fn test_basic_proxy_flow() {
    let harness = E2ETestHarness::new().await;
    
    // Start upstream server
    let upstream = harness.start_mcp_validator().await?;
    
    // Start proxy
    let proxy = harness.start_shadowcat_proxy(ProxyConfig {
        mode: ProxyMode::Reverse,
        upstream: upstream.url.clone(),
        auth: Some("Bearer test-token".to_string()),
    }).await?;
    
    // Run test scenario
    let client = TestClient::new(&proxy.url);
    let response = client.send_mcp_request("ping", None).await?;
    
    assert_eq!(response.status(), 200);
    
    // Logs are automatically captured and saved
    // Processes are automatically cleaned up
}
```

## Error Handling

### 1. Process Failure Detection

```rust
pub enum ProcessError {
    SpawnFailed(String),
    HealthCheckFailed(String),
    UnexpectedExit(ExitStatus),
    RestartLimitExceeded,
    ShutdownTimeout,
}

impl ProcessManager {
    async fn handle_process_failure(
        &self,
        id: ProcessId,
        error: ProcessError
    ) -> Result<()> {
        let processes = self.processes.read().await;
        
        if let Some(process) = processes.get(&id) {
            match process.restart_policy {
                RestartPolicy::Never => {
                    error!("Process {} failed and won't be restarted: {:?}", id, error);
                }
                RestartPolicy::OnFailure { max_attempts, backoff_ms } => {
                    if process.restart_count < max_attempts {
                        info!("Restarting process {} (attempt {}/{})", 
                              id, process.restart_count + 1, max_attempts);
                        
                        tokio::time::sleep(Duration::from_millis(
                            backoff_ms * 2_u64.pow(process.restart_count)
                        )).await;
                        
                        self.restart_process(id).await?;
                    } else {
                        error!("Process {} exceeded restart limit", id);
                    }
                }
                RestartPolicy::Always { delay_ms } => {
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    self.restart_process(id).await?;
                }
            }
        }
        
        Ok(())
    }
}
```

## Performance Considerations

1. **Process Pool**: Reuse processes across tests when possible
2. **Parallel Spawning**: Start processes concurrently
3. **Lazy Health Checks**: Only check when needed
4. **Log Buffering**: Use circular buffers to limit memory
5. **Cleanup Batching**: Shutdown processes in parallel

## CI/CD Considerations

1. **Docker Support**: Run processes in containers for isolation
2. **Resource Limits**: Enforce memory and CPU limits
3. **Timeout Enforcement**: Kill tests that exceed time limits
4. **Log Collection**: Archive logs for failed tests
5. **Process Leak Detection**: Verify all processes are cleaned up

## Next Steps

1. Implement core `ProcessManager` struct
2. Add health check implementations
3. Create log capture system
4. Build test harness integration
5. Add CI-specific optimizations