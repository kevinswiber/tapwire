# Task C.0: Create Raw Transport Primitives Module

## Objective
Extract the low-level I/O operations from existing transport implementations into a shared `transport/raw/` module. These primitives handle raw bytes without any MCP protocol knowledge, providing a foundation for both directional transport implementations.

## Design Reference
This task implements **Phase 2, Task 2.1** from `analysis/implementation-roadmap.md` (lines 207-251).

The architecture from `analysis/architecture-proposal.md` (lines 273-291) clarifies:
- **transport/raw/**: Low-level transport primitives (raw I/O operations)
- **transport/directional/**: High-level implementations using these primitives
- No knowledge of MCP protocol or message framing at this layer

## Prerequisites
- [x] Phase B complete (ResponseMode and ClientCapabilities added)
- [x] All 873 tests passing
- [ ] Understanding of existing stdio/HTTP/SSE implementations

## Implementation Steps

### Step 1: Create Raw Module Structure (15 min)

Create the module structure as designed in the implementation roadmap:

```rust
// src/transport/raw/mod.rs
pub mod stdio;
pub mod http;
pub mod sse;

// Re-export common types
pub use stdio::StdioCore;
pub use http::HttpCore;
pub use sse::SseCore;
```

### Step 2: Extract StdioCore (30 min)

Following the design in implementation-roadmap.md lines 220-250:

```rust
// src/transport/raw/stdio.rs

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use crate::transport::buffer_pool::{global_pools, BytesPool};
use std::sync::Arc;

/// Raw stdio I/O operations (no MCP knowledge)
/// Already well-implemented using tokio::io - extract existing logic
pub struct StdioCore {
    buffer_pool: Arc<BytesPool>,
    stdin: Option<BufReader<tokio::io::Stdin>>,
    stdout: Option<tokio::io::Stdout>,
    process: Option<Child>,  // For subprocess mode
}

impl StdioCore {
    pub fn new(buffer_pool: Arc<BytesPool>) -> Self {
        // Extract from existing StdioTransport
    }
    
    pub async fn send_bytes(&mut self, data: &[u8]) -> Result<()> {
        // Extract from StdioTransport::send_raw
        let stdout = self.stdout.as_mut().ok_or(Error::NotConnected)?;
        stdout.write_all(data).await?;
        stdout.flush().await?;
        Ok(())
    }
    
    pub async fn receive_bytes(&mut self) -> Result<Vec<u8>> {
        // Extract from StdioTransport::receive_raw
        // Already using buffer pooling - maintain this optimization
        let stdin = self.stdin.as_mut().ok_or(Error::NotConnected)?;
        // ... existing efficient implementation
    }
    
    pub async fn spawn_process(&mut self, command: Vec<String>) -> Result<()> {
        // Extract subprocess spawning logic
    }
}
```

### Step 3: Extract HttpCore (30 min)

Extract HTTP client/server primitives:

```rust
// src/transport/raw/http.rs

use hyper::{Body, Request, Response, Client};
use crate::transport::buffer_pool::BytesPool;

/// Raw HTTP operations
pub struct HttpCore {
    client: Option<Client<HttpsConnector<HttpConnector>>>,
    buffer_pool: Arc<BytesPool>,
}

impl HttpCore {
    pub async fn send_request(&self, request: Request<Body>) -> Result<Response<Body>> {
        // Extract from existing HTTP transport
    }
    
    pub async fn read_body(&self, body: Body) -> Result<Vec<u8>> {
        // Extract body reading with buffer pooling
    }
}
```

### Step 4: Extract SseCore (30 min)

Extract SSE-specific parsing:

```rust
// src/transport/raw/sse.rs

use tokio::io::AsyncBufReadExt;
use bytes::BytesMut;

/// Raw SSE event parsing (no MCP knowledge)
pub struct SseCore {
    buffer: BytesMut,
}

impl SseCore {
    pub fn parse_event(&mut self, line: &str) -> Option<SseEvent> {
        // Extract from existing SSE transport
        // Parse "data:", "event:", "id:" lines
    }
    
    pub async fn read_events<R: AsyncBufReadExt>(
        &mut self, 
        reader: &mut R
    ) -> Result<Vec<SseEvent>> {
        // Extract SSE event stream reading
    }
}

pub struct SseEvent {
    pub data: String,
    pub event: Option<String>,
    pub id: Option<String>,
}
```

### Step 5: Update Exports (15 min)

Update module exports:

```rust
// src/transport/mod.rs
pub mod raw;  // Add this

// Existing exports...
```

## Validation Steps

1. **Extraction preserves behavior**:
   - Move code, don't rewrite
   - Keep optimizations (buffer pooling)
   - Maintain error handling

2. **No protocol knowledge**:
   - Raw modules don't import `rmcp`
   - No JSON parsing at this layer
   - Just bytes in/out

3. **Tests still pass**:
   ```bash
   cargo test transport::
   ```

## Success Criteria
- [ ] Raw modules created with extracted logic
- [ ] No MCP/protocol imports in raw modules
- [ ] Buffer pooling preserved
- [ ] All existing transport tests still pass
- [ ] No performance regression

## Duration Estimate
**Total: 2 hours** (as per implementation-roadmap.md)
- Module structure: 15 min
- StdioCore extraction: 30 min
- HttpCore extraction: 30 min
- SseCore extraction: 30 min
- Integration and testing: 15 min

## Notes
- This is pure refactoring - no new functionality
- Preserve all existing optimizations
- Keep the same error types and handling
- Reference implementation-roadmap.md lines 207-251 for details

---

**Task Status**: Ready for implementation
**References**: analysis/implementation-roadmap.md (Task 2.1)
**Risk Level**: Low - Pure refactoring