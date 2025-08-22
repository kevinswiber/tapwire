# Port Allocation and Discovery System Design

## Overview
Dynamic port allocation is critical for reliable E2E testing, especially in CI/CD environments where multiple test suites run concurrently. The system must prevent port conflicts, support parallel execution, and provide service discovery.

## Core Requirements

1. **Zero Port Conflicts**: Guarantee unique ports across parallel tests
2. **OS Integration**: Leverage OS port assignment when possible  
3. **Service Discovery**: Tests can find dynamically assigned ports
4. **Fast Allocation**: Minimal overhead (<10ms per port)
5. **Cleanup Guarantee**: Released ports are immediately available
6. **CI/CD Compatible**: Work in containerized environments

## Architecture

### 1. Port Allocation Strategy

```rust
use std::net::{TcpListener, SocketAddr};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::{HashSet, HashMap};

pub struct PortAllocator {
    // Track allocated ports for cleanup
    allocated: Arc<RwLock<HashSet<Port>>>,
    
    // Service registry for discovery
    registry: Arc<RwLock<ServiceRegistry>>,
    
    // Configuration
    config: PortConfig,
    
    // Metrics
    metrics: AllocationMetrics,
}

#[derive(Debug, Clone)]
pub struct PortConfig {
    // Preferred allocation method
    pub strategy: AllocationStrategy,
    
    // Port range for manual allocation
    pub min_port: u16,
    pub max_port: u16,
    
    // Retry configuration
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    
    // Cleanup behavior
    pub release_delay_ms: u64,
}

#[derive(Debug, Clone)]
pub enum AllocationStrategy {
    // Let OS assign (preferred)
    OsAssigned,
    
    // Use specific range
    Range { start: u16, end: u16 },
    
    // Use ephemeral range (49152-65535)
    Ephemeral,
    
    // Docker/K8s compatible
    Container { base: u16, offset: u16 },
}
```

### 2. OS-Assigned Port Allocation (Primary Strategy)

```rust
impl PortAllocator {
    pub async fn allocate_os_assigned(&self) -> Result<Port> {
        // Method 1: Bind and immediately release
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;
        let port = addr.port();
        
        // Important: Drop listener to release the port
        drop(listener);
        
        // Small delay to ensure OS releases the port
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        // Verify port is available
        if !self.is_port_available(port).await? {
            // Retry if port was grabbed by another process
            return self.allocate_os_assigned().await;
        }
        
        // Register the allocation
        let port_handle = Port::new(port, self.allocated.clone());
        self.allocated.write().await.insert(port_handle.clone());
        
        Ok(port_handle)
    }
    
    pub async fn allocate_multiple(&self, count: usize) -> Result<Vec<Port>> {
        // Allocate multiple ports concurrently
        let futures = (0..count).map(|_| self.allocate_os_assigned());
        
        let results = futures::future::try_join_all(futures).await?;
        
        // Verify no duplicates (shouldn't happen with OS assignment)
        let mut seen = HashSet::new();
        for port in &results {
            if !seen.insert(port.number()) {
                return Err(anyhow!("Duplicate port allocated: {}", port.number()));
            }
        }
        
        Ok(results)
    }
}
```

### 3. Range-Based Allocation (Fallback Strategy)

```rust
impl PortAllocator {
    pub async fn allocate_from_range(&self, start: u16, end: u16) -> Result<Port> {
        let allocated = self.allocated.read().await;
        
        // Try random ports in range to reduce conflicts
        let mut rng = rand::thread_rng();
        let mut attempts = 0;
        
        while attempts < self.config.max_retries {
            let port_num = rng.gen_range(start..=end);
            
            // Skip if already allocated by this allocator
            if allocated.iter().any(|p| p.number() == port_num) {
                attempts += 1;
                continue;
            }
            
            // Check if port is available on the system
            if self.is_port_available(port_num).await? {
                let port_handle = Port::new(port_num, self.allocated.clone());
                drop(allocated); // Release read lock
                
                self.allocated.write().await.insert(port_handle.clone());
                return Ok(port_handle);
            }
            
            attempts += 1;
            tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
        }
        
        Err(anyhow!("Failed to allocate port from range {}..{}", start, end))
    }
}
```

### 4. Port Handle with Automatic Cleanup

```rust
#[derive(Debug, Clone)]
pub struct Port {
    number: u16,
    allocated_at: Instant,
    allocator: Weak<RwLock<HashSet<Port>>>,
}

impl Port {
    pub fn new(number: u16, allocator: Arc<RwLock<HashSet<Port>>>) -> Self {
        Port {
            number,
            allocated_at: Instant::now(),
            allocator: Arc::downgrade(&allocator),
        }
    }
    
    pub fn number(&self) -> u16 {
        self.number
    }
    
    pub fn address(&self, host: &str) -> String {
        format!("{}:{}", host, self.number)
    }
    
    pub fn url(&self, scheme: &str, path: &str) -> String {
        format!("{}://127.0.0.1:{}{}", scheme, self.number, path)
    }
}

impl Drop for Port {
    fn drop(&mut self) {
        // Automatically release port when handle is dropped
        if let Some(allocator) = self.allocator.upgrade() {
            let port = self.number;
            tokio::spawn(async move {
                // Small delay before releasing to avoid immediate reuse
                tokio::time::sleep(Duration::from_millis(100)).await;
                
                let mut allocated = allocator.write().await;
                allocated.retain(|p| p.number != port);
                
                debug!("Released port {}", port);
            });
        }
    }
}
```

### 5. Service Registry for Discovery

```rust
pub struct ServiceRegistry {
    services: HashMap<String, ServiceInfo>,
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub port: u16,
    pub protocol: Protocol,
    pub health_endpoint: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl PortAllocator {
    pub async fn allocate_for_service(&self, name: &str) -> Result<ServiceHandle> {
        let port = self.allocate_os_assigned().await?;
        
        let info = ServiceInfo {
            name: name.to_string(),
            port: port.number(),
            protocol: Protocol::Http,
            health_endpoint: Some("/health".to_string()),
            metadata: HashMap::new(),
        };
        
        self.registry.write().await.services.insert(name.to_string(), info.clone());
        
        Ok(ServiceHandle {
            port,
            info,
            registry: self.registry.clone(),
        })
    }
    
    pub async fn discover_service(&self, name: &str) -> Option<ServiceInfo> {
        self.registry.read().await.services.get(name).cloned()
    }
    
    pub async fn wait_for_service(&self, name: &str, timeout: Duration) -> Result<ServiceInfo> {
        let deadline = Instant::now() + timeout;
        
        while Instant::now() < deadline {
            if let Some(info) = self.discover_service(name).await {
                return Ok(info);
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        Err(anyhow!("Service {} not found within timeout", name))
    }
}
```

### 6. Port Availability Checking

```rust
impl PortAllocator {
    async fn is_port_available(&self, port: u16) -> Result<bool> {
        // Method 1: Try to bind
        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(listener) => {
                drop(listener);
                Ok(true)
            }
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                Ok(false)
            }
            Err(e) => Err(e.into()),
        }
    }
    
    async fn is_port_ready(&self, port: u16, timeout: Duration) -> Result<bool> {
        let deadline = Instant::now() + timeout;
        
        while Instant::now() < deadline {
            match TcpStream::connect(("127.0.0.1", port)).await {
                Ok(_) => return Ok(true),
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        }
        
        Ok(false)
    }
}
```

## CI/CD Considerations

### 1. Container-Aware Allocation

```rust
impl PortAllocator {
    pub fn new_for_container() -> Self {
        // In containers, use a specific range to avoid conflicts
        let base_port = std::env::var("TEST_PORT_BASE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30000);
        
        let worker_id = std::env::var("TEST_WORKER_ID")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(0);
        
        // Each worker gets 100 ports
        let min_port = base_port + (worker_id * 100);
        let max_port = min_port + 99;
        
        PortAllocator {
            config: PortConfig {
                strategy: AllocationStrategy::Range { 
                    start: min_port, 
                    end: max_port 
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
```

### 2. GitHub Actions Integration

```yaml
# .github/workflows/test.yml
jobs:
  test:
    strategy:
      matrix:
        worker: [0, 1, 2, 3]
    env:
      TEST_WORKER_ID: ${{ matrix.worker }}
      TEST_PORT_BASE: 40000
    steps:
      - run: cargo test --test e2e
```

## Test Patterns

### 1. Basic Test Setup

```rust
#[tokio::test]
async fn test_with_dynamic_ports() {
    let allocator = PortAllocator::new();
    
    // Allocate ports for all services
    let proxy_port = allocator.allocate_for_service("proxy").await?;
    let upstream_port = allocator.allocate_for_service("upstream").await?;
    
    // Start services with allocated ports
    let upstream = start_upstream_server(&upstream_port.address("127.0.0.1")).await?;
    let proxy = start_proxy(
        &proxy_port.address("127.0.0.1"),
        &upstream_port.url("http", "/")
    ).await?;
    
    // Ports are automatically released when test ends
}
```

### 2. Parallel Test Execution

```rust
#[tokio::test]
async fn test_parallel_allocation() {
    let allocator = Arc::new(PortAllocator::new());
    
    // Run 10 tests in parallel
    let handles: Vec<_> = (0..10).map(|i| {
        let allocator = allocator.clone();
        tokio::spawn(async move {
            let port = allocator.allocate_os_assigned().await?;
            
            // Each test gets a unique port
            run_test_with_port(i, port).await
        })
    }).collect();
    
    // Wait for all tests
    for handle in handles {
        handle.await??;
    }
}
```

### 3. Service Discovery Pattern

```rust
struct TestServices {
    allocator: Arc<PortAllocator>,
}

impl TestServices {
    async fn start_all(&self) -> Result<()> {
        // Start services in dependency order
        self.start_database().await?;
        self.start_auth_server().await?;
        self.start_upstream().await?;
        self.start_proxy().await?;
        
        Ok(())
    }
    
    async fn get_proxy_url(&self) -> Result<String> {
        let info = self.allocator
            .wait_for_service("proxy", Duration::from_secs(10))
            .await?;
        
        Ok(format!("http://127.0.0.1:{}", info.port))
    }
}
```

## Performance Optimizations

1. **Port Pool**: Pre-allocate ports for frequently created services
2. **Lazy Allocation**: Only allocate when service actually starts
3. **Batch Allocation**: Allocate multiple ports in single operation
4. **Cache Discovery**: Cache service lookups for duration of test

## Error Recovery

```rust
impl PortAllocator {
    pub async fn allocate_with_fallback(&self) -> Result<Port> {
        // Try OS assignment first
        match self.allocate_os_assigned().await {
            Ok(port) => Ok(port),
            Err(e) => {
                warn!("OS assignment failed: {}, trying range allocation", e);
                
                // Fall back to range allocation
                self.allocate_from_range(
                    self.config.min_port,
                    self.config.max_port
                ).await
            }
        }
    }
    
    pub async fn cleanup_stale_allocations(&self) {
        let mut allocated = self.allocated.write().await;
        let now = Instant::now();
        
        // Remove allocations older than 1 hour (likely leaked)
        allocated.retain(|port| {
            let age = now - port.allocated_at;
            age < Duration::from_secs(3600)
        });
    }
}
```

## Metrics and Monitoring

```rust
#[derive(Debug, Default)]
pub struct AllocationMetrics {
    pub total_allocated: AtomicU64,
    pub current_allocated: AtomicU64,
    pub allocation_failures: AtomicU64,
    pub average_allocation_time_us: AtomicU64,
}

impl PortAllocator {
    pub fn metrics(&self) -> AllocationMetrics {
        self.metrics.clone()
    }
    
    pub async fn health_check(&self) -> HealthStatus {
        let allocated = self.allocated.read().await;
        
        if allocated.len() > 1000 {
            HealthStatus::Degraded("Too many allocated ports".into())
        } else {
            HealthStatus::Healthy
        }
    }
}
```

## Integration with Shadowcat

Since Shadowcat already supports `127.0.0.1:0` for dynamic port binding, we can leverage this:

```rust
pub async fn start_shadowcat_with_dynamic_port() -> Result<(Port, String)> {
    // Let Shadowcat bind to 0
    let mut cmd = Command::new("./target/release/shadowcat");
    cmd.args(&["reverse", "--bind", "127.0.0.1:0"]);
    
    let output = cmd.output().await?;
    
    // Parse actual port from logs
    let port_regex = Regex::new(r"Listening on .*:(\d+)")?;
    let port_str = // extract from output
    
    let port = port_str.parse::<u16>()?;
    
    Ok((Port::new(port), format!("http://127.0.0.1:{}", port)))
}
```

## Summary

This port allocation system provides:
- **Reliability**: Zero port conflicts through OS assignment
- **Performance**: <10ms allocation time
- **Discovery**: Service registry for dynamic ports
- **Cleanup**: Automatic port release
- **CI/CD Ready**: Container and parallel execution support
- **Shadowcat Integration**: Leverages existing dynamic port support