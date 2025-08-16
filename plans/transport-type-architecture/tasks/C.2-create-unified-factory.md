# Task C.2: Create Unified Factory

## Objective
Implement a unified TransportFactory that creates both incoming and outgoing transports with consistent configuration, buffer pooling, and connection management. This centralizes transport creation logic currently scattered across the codebase.

## Design Reference
This task implements **Phase 2, Task 2.3** from `analysis/implementation-roadmap.md` (lines 284-332).

From `analysis/architecture-proposal.md` (lines 215-271):
- TransportFactory handles all transport creation
- Includes buffer pool management
- Supports connection pooling for outgoing transports

## Prerequisites
- [x] C.0 complete (raw primitives created)
- [x] C.1 complete (directional transports refactored)
- [ ] Understanding of current transport creation patterns

## Implementation Steps

### Step 1: Create Factory Module Structure (15 min)

Following implementation-roadmap.md lines 285-290:

```rust
// src/transport/factory/mod.rs

mod builder;
mod config;

pub use builder::TransportBuilder;
pub use config::TransportConfig;

use crate::transport::directional::{IncomingTransport, OutgoingTransport};
use crate::transport::buffer_pool::{global_pools, BufferPools};
use crate::transport::pool::ConnectionPool;

pub struct TransportFactory {
    buffer_pools: Arc<BufferPools>,
    connection_pool: Arc<ConnectionPool>,
    config: TransportConfig,
}
```

### Step 2: Implement Factory Core (30 min)

Following the design from implementation-roadmap.md lines 292-324:

```rust
// src/transport/factory/mod.rs

impl TransportFactory {
    pub fn new(config: TransportConfig) -> Self {
        Self {
            buffer_pools: Arc::new(BufferPools::default()),
            connection_pool: Arc::new(ConnectionPool::new(
                config.pool_size,
                config.pool_timeout,
            )),
            config,
        }
    }
    
    pub async fn create_incoming(
        &self,
        transport_type: TransportType,
    ) -> Result<Box<dyn IncomingTransport>> {
        let transport = match transport_type {
            TransportType::Stdio => {
                Box::new(StdioIncoming::new(
                    self.buffer_pools.stdio_pool.clone()
                )) as Box<dyn IncomingTransport>
            }
            TransportType::StreamableHttp => {
                Box::new(HttpIncoming::new(
                    self.config.bind_addr,
                    self.buffer_pools.http_pool.clone(),
                )) as Box<dyn IncomingTransport>
            }
        };
        
        Ok(transport)
    }
    
    pub async fn create_outgoing(
        &self,
        transport_type: TransportType,
        target: &str,
    ) -> Result<Box<dyn OutgoingTransport>> {
        // Check pool first
        if let Some(pooled) = self.connection_pool.acquire(target).await? {
            return Ok(pooled);
        }
        
        // Create new connection
        let transport = match transport_type {
            TransportType::Stdio => {
                Box::new(SubprocessOutgoing::new(
                    target.split_whitespace().map(String::from).collect(),
                    self.buffer_pools.stdio_pool.clone(),
                )) as Box<dyn OutgoingTransport>
            }
            TransportType::StreamableHttp => {
                Box::new(HttpOutgoing::new(
                    target,
                    self.buffer_pools.http_pool.clone(),
                )) as Box<dyn OutgoingTransport>
            }
        };
        
        Ok(transport)
    }
}
```

### Step 3: Create Configuration (20 min)

Transport configuration types:

```rust
// src/transport/factory/config.rs

use std::time::Duration;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct TransportConfig {
    // Buffer pool settings
    pub stdio_buffer_size: usize,
    pub http_buffer_size: usize,
    
    // Connection pool settings  
    pub pool_size: usize,
    pub pool_timeout: Duration,
    pub pool_health_check_interval: Duration,
    
    // Network settings
    pub bind_addr: SocketAddr,
    pub connect_timeout: Duration,
    pub read_timeout: Duration,
    
    // Performance settings
    pub max_concurrent_requests: usize,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            stdio_buffer_size: 8192,  // From constants.rs
            http_buffer_size: 16384,   // From constants.rs
            pool_size: 10,
            pool_timeout: Duration::from_secs(30),
            pool_health_check_interval: Duration::from_secs(60),
            bind_addr: "127.0.0.1:0".parse().unwrap(),
            connect_timeout: Duration::from_secs(10),
            read_timeout: Duration::from_secs(30),
            max_concurrent_requests: 100,
        }
    }
}
```

### Step 4: Create Builder Pattern (25 min)

Fluent builder for configuration:

```rust
// src/transport/factory/builder.rs

pub struct TransportBuilder {
    config: TransportConfig,
}

impl TransportBuilder {
    pub fn new() -> Self {
        Self {
            config: TransportConfig::default(),
        }
    }
    
    pub fn stdio_buffer_size(mut self, size: usize) -> Self {
        self.config.stdio_buffer_size = size;
        self
    }
    
    pub fn pool_size(mut self, size: usize) -> Self {
        self.config.pool_size = size;
        self
    }
    
    pub fn bind_address(mut self, addr: SocketAddr) -> Self {
        self.config.bind_addr = addr;
        self
    }
    
    // ... other builder methods ...
    
    pub fn build(self) -> TransportFactory {
        TransportFactory::new(self.config)
    }
}
```

### Step 5: Update Usage Sites (30 min)

Update forward and reverse proxies to use factory:

```rust
// src/cli/forward.rs

let factory = TransportBuilder::new()
    .pool_size(20)
    .build();

let client_transport = factory.create_incoming(
    TransportType::Stdio
).await?;

let server_transport = factory.create_outgoing(
    TransportType::from_command(&command),
    &command.join(" "),
).await?;
```

```rust
// src/cli/reverse.rs

let factory = TransportBuilder::new()
    .bind_address(config.bind_addr)
    .pool_size(config.pool_size)
    .build();

// Use factory for all transport creation
```

## Validation Steps

1. **Factory creates all transports**:
   ```bash
   # No direct transport construction
   rg "StdioIncoming::new|HttpIncoming::new" src/ --glob '!factory.rs'
   ```

2. **Tests pass**:
   ```bash
   cargo test transport::factory
   ```

3. **Connection pooling works**:
   ```bash
   cargo test test_connection_pooling
   ```

## Success Criteria
- [ ] TransportFactory implemented
- [ ] All transports created via factory
- [ ] Buffer pools managed centrally
- [ ] Connection pooling functional
- [ ] Configuration centralized
- [ ] Tests passing

## Duration Estimate
**Total: 2 hours** (as per implementation-roadmap.md)
- Module structure: 15 min
- Factory implementation: 30 min
- Configuration: 20 min
- Builder pattern: 25 min
- Update usage sites: 30 min

## Integration Points

From implementation-roadmap.md lines 328-331:
- `src/cli/forward.rs` - Use factory for transport creation
- `src/cli/reverse.rs` - Use factory for transport creation

## Notes
- This centralizes all transport creation
- Enables consistent configuration
- Supports future enhancements (metrics, tracing)
- Reference implementation-roadmap.md lines 284-332

---

**Task Status**: Ready for implementation
**References**: analysis/implementation-roadmap.md (Task 2.3)
**Risk Level**: Medium - Changes transport creation patterns