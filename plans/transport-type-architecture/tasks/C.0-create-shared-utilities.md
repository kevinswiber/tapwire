# Task C.0: Create Shared Transport Utilities Module (REVISED)

## Status: Ready
## Estimated Duration: 1 hour
## Actual Duration: TBD

## Context

**REVISED APPROACH (2025-08-16)**: Based on implementation learnings, we're taking a different approach than originally planned. Instead of creating unified cores (StdioCore, HttpCore, SseCore) with mode flags, we're extracting shared utilities while keeping transport types separate. See [Phase C Revised Approach](../analysis/phase-c-revised-approach.md) for full rationale.

## Objective

Create a shared utilities module that extracts common logic from raw transport implementations, reducing code duplication by >50% while maintaining type safety and single responsibility principle.

## Problem Being Solved

The raw transport implementations (`StdioRawIncoming`, `StdioRawOutgoing`, `HttpRawClient`, `HttpRawServer`) have significant code duplication:
- Connection state validation (~50 lines duplicated)
- Buffer pool management (~100 lines duplicated)
- Message size validation (~50 lines duplicated)
- Timeout handling (~100 lines duplicated)
- Error wrapping patterns (~200 lines duplicated)

Total: ~500 lines of duplicated code across transport implementations.

## Implementation Steps

### Step 1: Create Common Module Structure (15 min)

Create the shared utilities module:

```rust
// src/transport/raw/common.rs
pub mod connection;
pub mod buffer;
pub mod timeout;
pub mod validation;
```

### Step 2: Extract Connection Utilities (15 min)

```rust
// src/transport/raw/common/connection.rs

use crate::error::{TransportError, TransportResult};

/// Validate that a transport is connected before operations
pub fn ensure_connected(connected: bool) -> TransportResult<()> {
    if !connected {
        return Err(TransportError::NotConnected);
    }
    Ok(())
}

/// Common connection state management
pub struct ConnectionState {
    connected: bool,
}

impl ConnectionState {
    pub fn new() -> Self {
        Self { connected: false }
    }
    
    pub fn connect(&mut self) -> TransportResult<()> {
        if self.connected {
            return Err(TransportError::ConnectionFailed(
                "Already connected".to_string()
            ));
        }
        self.connected = true;
        Ok(())
    }
    
    pub fn disconnect(&mut self) {
        self.connected = false;
    }
    
    pub fn ensure_connected(&self) -> TransportResult<()> {
        ensure_connected(self.connected)
    }
}
```

### Step 3: Extract Buffer Management (15 min)

```rust
// src/transport/raw/common/buffer.rs

use bytes::BytesMut;
use crate::transport::buffer_pool::{BytesPool, global_pools};
use std::sync::Arc;

/// Acquire buffer from pool and fill with data
pub fn acquire_and_fill(pool: &Arc<BytesPool>, data: &[u8]) -> BytesMut {
    let mut buffer = pool.acquire();
    buffer.clear();
    buffer.extend_from_slice(data);
    buffer
}

/// Convert buffer to Vec and release back to pool
pub fn to_vec_and_release(pool: &Arc<BytesPool>, buffer: BytesMut) -> Vec<u8> {
    let result = buffer.to_vec();
    pool.release(buffer);
    result
}

/// Get appropriate buffer pool for transport type
pub fn get_pool_for_transport(transport_type: &str) -> Arc<BytesPool> {
    match transport_type {
        "stdio" => Arc::new(global_pools::STDIO_POOL.clone()),
        "http" => Arc::new(global_pools::HTTP_POOL.clone()),
        _ => Arc::new(global_pools::STDIO_POOL.clone()),
    }
}
```

### Step 4: Extract Validation Utilities (10 min)

```rust
// src/transport/raw/common/validation.rs

use crate::error::{TransportError, TransportResult};

/// Validate message size against configured limit
pub fn validate_message_size(data: &[u8], max_size: usize) -> TransportResult<()> {
    if data.len() > max_size {
        return Err(TransportError::MessageTooLarge {
            size: data.len(),
            limit: max_size,
        });
    }
    Ok(())
}

/// Validate that data is not empty
pub fn validate_not_empty(data: &[u8]) -> TransportResult<()> {
    if data.is_empty() {
        return Err(TransportError::ProtocolError(
            "Empty message not allowed".to_string()
        ));
    }
    Ok(())
}
```

### Step 5: Extract Timeout Utilities (5 min)

```rust
// src/transport/raw/common/timeout.rs

use tokio::time::{timeout, Duration};
use crate::error::{TransportError, TransportResult};
use std::future::Future;

/// Execute an async operation with timeout
pub async fn with_timeout<F, T>(
    duration: Duration, 
    operation: F,
    timeout_msg: &str,
) -> TransportResult<T> 
where
    F: Future<Output = TransportResult<T>>,
{
    match timeout(duration, operation).await {
        Ok(result) => result,
        Err(_) => Err(TransportError::Timeout(timeout_msg.to_string())),
    }
}
```

### Step 6: Update Module Exports (5 min)

```rust
// src/transport/raw/common.rs
pub mod connection;
pub mod buffer;
pub mod validation;
pub mod timeout;

// Re-export commonly used items
pub use connection::{ensure_connected, ConnectionState};
pub use buffer::{acquire_and_fill, to_vec_and_release};
pub use validation::{validate_message_size, validate_not_empty};
pub use timeout::with_timeout;
```

Update raw module to include common:

```rust
// src/transport/raw/mod.rs
pub mod common;  // Add this
```

## Validation Steps

1. **Module compiles**: `cargo check`
2. **No circular dependencies**: Utilities don't depend on specific transport types
3. **All utilities have tests**: Each utility function should have unit tests
4. **Documentation complete**: All public functions have doc comments

## Success Criteria

- [ ] Common utilities module created
- [ ] Connection utilities extracted
- [ ] Buffer management utilities extracted  
- [ ] Validation utilities extracted
- [ ] Timeout utilities extracted
- [ ] Module exports properly configured
- [ ] No compilation errors
- [ ] Ready for use in transport refactoring (C.1)

## Notes

- This is a pure extraction - no new functionality
- Keep utilities generic and reusable
- Avoid transport-specific logic in utilities
- Focus on the patterns that are truly duplicated

## Risk Mitigation

- **Risk**: Breaking existing transports
- **Mitigation**: This task only creates new utilities, doesn't modify existing code

- **Risk**: Over-abstraction
- **Mitigation**: Only extract truly common patterns, keep specific logic in transports

---

**Task Status**: Ready for implementation
**Dependencies**: None (pure addition)
**Next Task**: C.1 - Refactor raw transports to use utilities