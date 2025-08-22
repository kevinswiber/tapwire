# H.1: Fix Stdio Subprocess Spawning

**Priority**: 🔴 CRITICAL  
**Duration**: 4 hours estimated, 12 hours actual  
**Status**: ✅ COMPLETE (Session 9)  

## Problem

The stdio transport creates a new subprocess for EVERY request instead of reusing connections, defeating the purpose of connection pooling entirely.

**Location**: `src/proxy/reverse/upstream/stdio.rs:87-106`

```rust
// CURRENT (BROKEN)
let factory = move || {
    let mut transport = Subprocess::new(command_string)?;
    transport.connect().await?;  // New process spawned per request!
    Ok(transport)
}
```

## Impact

- 10ms overhead per request (process spawn time)
- 90% throughput reduction for stdio
- File descriptor exhaustion risk
- PID exhaustion on busy systems
- System resource exhaustion under load

## Solution Options

### Option 1: Implement True Connection Reuse (Recommended)

Create a persistent subprocess that handles multiple requests:

```rust
pub struct ReusableStdioConnection {
    process: Arc<Mutex<Child>>,
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
    health_check_interval: Duration,
    last_health_check: Instant,
}

impl ReusableStdioConnection {
    pub async fn new(command: Vec<String>) -> Result<Self> {
        let mut cmd = Command::new(&command[0]);
        cmd.args(&command[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        let mut child = cmd.spawn()?;
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        
        Ok(Self {
            process: Arc::new(Mutex::new(child)),
            stdin: Arc::new(Mutex::new(stdin)),
            stdout: Arc::new(Mutex::new(stdout)),
            health_check_interval: Duration::from_secs(30),
            last_health_check: Instant::now(),
        })
    }
    
    pub async fn send_request(&self, request: Value) -> Result<Value> {
        // Check process health
        if self.last_health_check.elapsed() > self.health_check_interval {
            self.health_check().await?;
        }
        
        // Send request
        let mut stdin = self.stdin.lock().await;
        serde_json::to_writer(&mut *stdin, &request)?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        drop(stdin);
        
        // Read response
        let mut stdout = self.stdout.lock().await;
        let mut line = String::new();
        stdout.read_line(&mut line).await?;
        
        Ok(serde_json::from_str(&line)?)
    }
    
    async fn health_check(&self) -> Result<()> {
        let mut process = self.process.lock().await;
        match process.try_wait()? {
            Some(status) => {
                return Err(anyhow!("Process exited with status: {}", status));
            }
            None => {
                // Process still running
                Ok(())
            }
        }
    }
}
```

### Option 2: Document as Known Limitation

If true pooling is too complex, document the limitation and optimize spawning:

```rust
/// StdioUpstream - Creates new subprocess per request
/// 
/// LIMITATION: Due to the nature of stdio communication, each request
/// spawns a new subprocess. This is inefficient for high-throughput
/// scenarios. Consider using HTTP transport for better performance.
/// 
/// Performance characteristics:
/// - Overhead: ~10ms per request
/// - Max throughput: ~100 requests/second
/// - Resource usage: High (process per request)
pub struct StdioUpstream {
    // ... existing fields ...
    spawn_timeout: Duration,
    max_concurrent_processes: usize,
    process_semaphore: Arc<Semaphore>,
}
```

### Option 3: Implement Process Pool

Maintain a pool of pre-spawned processes:

```rust
pub struct StdioProcessPool {
    command: Vec<String>,
    pool_size: usize,
    processes: Arc<Mutex<Vec<StdioProcess>>>,
    available: Arc<Semaphore>,
}

impl StdioProcessPool {
    pub async fn new(command: Vec<String>, pool_size: usize) -> Result<Self> {
        let mut processes = Vec::with_capacity(pool_size);
        
        // Pre-spawn processes
        for _ in 0..pool_size {
            let process = StdioProcess::spawn(command.clone()).await?;
            processes.push(process);
        }
        
        Ok(Self {
            command,
            pool_size,
            processes: Arc::new(Mutex::new(processes)),
            available: Arc::new(Semaphore::new(pool_size)),
        })
    }
    
    pub async fn execute(&self, request: Value) -> Result<Value> {
        let _permit = self.available.acquire().await?;
        
        let process = {
            let mut pool = self.processes.lock().await;
            pool.pop().ok_or_else(|| anyhow!("No processes available"))?
        };
        
        let result = process.send_request(request).await;
        
        // Return process to pool or spawn new one if it died
        let mut pool = self.processes.lock().await;
        if process.is_alive().await {
            pool.push(process);
        } else {
            // Spawn replacement
            let new_process = StdioProcess::spawn(self.command.clone()).await?;
            pool.push(new_process);
        }
        
        result
    }
}
```

## Implementation Steps

1. **Choose approach** (recommend Option 1 for best performance)
2. **Implement connection reuse logic**
3. **Add health checking for processes**
4. **Handle process crashes gracefully**
5. **Add metrics for process lifecycle**
6. **Update factory closure in StdioUpstream**
7. **Add comprehensive tests**

## Testing

### Unit Tests
```rust
#[tokio::test]
async fn test_stdio_connection_reuse() {
    let connection = ReusableStdioConnection::new(vec!["echo".into()]).await.unwrap();
    
    // Send multiple requests through same connection
    for i in 0..100 {
        let request = json!({"id": i, "method": "test"});
        let response = connection.send_request(request).await.unwrap();
        assert_eq!(response["id"], i);
    }
    
    // Verify only one process was spawned
    // (check process metrics)
}

#[tokio::test]
async fn test_stdio_process_crash_recovery() {
    // Test that crashed processes are detected and replaced
}
```

### Performance Test
```bash
# Before fix: ~10 seconds for 100 requests
time for i in {1..100}; do
    curl http://localhost:8080/stdio-endpoint
done

# After fix: <1 second for 100 requests
```

## Success Criteria

- [ ] Single process handles multiple requests
- [ ] 10x throughput improvement
- [ ] Process health monitoring works
- [ ] Crash recovery implemented
- [ ] No file descriptor leaks
- [ ] Performance tests pass

## Files to Modify

1. `src/proxy/reverse/upstream/stdio.rs` - Implement reusable connections
2. `src/transport/subprocess.rs` - May need modifications for reuse
3. `tests/integration/stdio_tests.rs` - Add reuse tests

## Dependencies

- May need to modify the underlying Subprocess transport
- Consider impact on existing stdio forward proxy

## Risks

- Process communication protocol must support multiple requests
- Need to handle partial reads/writes
- Process crash detection and recovery complexity

## Alternative

If full reuse is not feasible, at minimum:
1. Pre-validate command before spawning
2. Use process pool with pre-spawned processes
3. Add spawn rate limiting
4. Document performance limitations clearly

## Resolution (Session 9)

### Root Cause Discovered
The connection pool's Drop implementation was calling `shutdown.notify_one()` on ANY clone drop, not just the last reference. This caused the maintenance loop to shut down immediately after construction, preventing connection reuse.

### Fix Evolution
1. **Initial**: Removed Drop entirely (worked but no cleanup)
2. **Attempted**: Check `Arc::strong_count(&self.shutdown)` (wrong Arc)
3. **Final**: Inner Arc pattern per GPT-5 recommendation

### Implementation
Restructured ConnectionPool to use inner Arc pattern:
```rust
pub struct ConnectionPool<T> {
    inner: Arc<ConnectionPoolInner<T>>,
}

impl<T> Drop for ConnectionPool<T> {
    fn drop(&mut self) {
        if Arc::strong_count(&self.inner) == 1 {
            // Last reference - do cleanup
        }
    }
}
```

### Results
- ✅ Pool correctly reuses connections (1 subprocess for N requests)
- ✅ Drop only triggers on last ConnectionPool reference
- ✅ Automatic cleanup as safety net
- ✅ All tests pass including new last-reference tests
- ✅ 90% throughput loss RESOLVED
- ✅ No more subprocess spawning overhead

### Tests Added
- `test_simple_pool_reuse` - Unit test for basic reuse
- `test_stdio_subprocess_pool_reuse` - Integration test with real subprocesses
- `test_last_reference_drop_cleanup` - Verifies Drop semantics
- `test_pool_returns_connections` - Verifies return mechanism

### Files Modified
- `src/proxy/pool.rs` - Complete restructure with inner Arc
- `src/transport/outgoing/subprocess.rs` - Added disconnection detection
- `tests/test_stdio_pool_reuse.rs` - New integration test