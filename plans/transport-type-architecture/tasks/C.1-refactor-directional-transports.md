# Task C.1: Refactor Directional Transports

## Objective
Refactor the existing IncomingTransport and OutgoingTransport implementations to use the shared raw transport primitives created in C.0. This eliminates code duplication while maintaining the same public API.

## Design Reference
This task implements **Phase 2, Task 2.2** from `analysis/implementation-roadmap.md` (lines 253-282).

From `analysis/directional-transport-analysis.md`:
- Forward proxy already uses directional transports correctly
- These implementations contain duplicated I/O logic
- Goal is to delegate raw I/O to shared primitives

## Prerequisites
- [x] C.0 complete (raw transport primitives created)
- [ ] Understanding of existing directional implementations

## Implementation Steps

### Step 1: Refactor StdioIncoming (30 min)

Following the pattern in implementation-roadmap.md lines 255-281:

```rust
// src/transport/directional/incoming.rs

use crate::transport::raw::stdio::StdioCore;
use crate::protocol;  // For MCP serialization

pub struct StdioIncoming {
    core: StdioCore,  // Delegate raw I/O
    session_id: SessionId,
}

impl StdioIncoming {
    pub fn new(buffer_pool: Arc<BytesPool>) -> Self {
        Self {
            core: StdioCore::new(buffer_pool),
            session_id: SessionId::new(),
        }
    }
}

impl IncomingTransport for StdioIncoming {
    async fn receive_request(&mut self) -> TransportResult<MessageEnvelope> {
        // Use core for raw I/O
        let bytes = self.core.receive_bytes().await?;
        
        // Handle MCP protocol at this layer
        let message = protocol::deserialize(&bytes)?;
        
        Ok(MessageEnvelope {
            message,
            session_id: self.session_id.clone(),
        })
    }
    
    async fn send_response(&mut self, response: MessageEnvelope) -> TransportResult<()> {
        // Serialize at this layer
        let bytes = protocol::serialize(&response.message)?;
        
        // Use core for raw I/O
        self.core.send_bytes(&bytes).await?;
        Ok(())
    }
    
    // Other methods delegate to core
}
```

### Step 2: Refactor StdioOutgoing (30 min)

Similar pattern for outgoing:

```rust
// src/transport/directional/outgoing.rs

use crate::transport::raw::stdio::StdioCore;

pub struct SubprocessOutgoing {
    core: StdioCore,
    session_id: SessionId,
    command: Vec<String>,
}

impl SubprocessOutgoing {
    pub fn new(command: Vec<String>, buffer_pool: Arc<BytesPool>) -> Self {
        Self {
            core: StdioCore::new(buffer_pool),
            session_id: SessionId::new(),
            command,
        }
    }
}

impl OutgoingTransport for SubprocessOutgoing {
    async fn connect(&mut self) -> TransportResult<()> {
        // Use core to spawn process
        self.core.spawn_process(self.command.clone()).await
    }
    
    async fn send_request(&mut self, request: MessageEnvelope) -> TransportResult<()> {
        let bytes = protocol::serialize(&request.message)?;
        self.core.send_bytes(&bytes).await
    }
    
    async fn receive_response(&mut self) -> TransportResult<MessageEnvelope> {
        let bytes = self.core.receive_bytes().await?;
        let message = protocol::deserialize(&bytes)?;
        
        Ok(MessageEnvelope {
            message,
            session_id: self.session_id.clone(),
        })
    }
}
```

### Step 3: Refactor HTTP Transports (30 min)

Update HTTP implementations:

```rust
// src/transport/directional/http_incoming.rs

use crate::transport::raw::http::HttpCore;

pub struct HttpIncoming {
    core: HttpCore,
    session_id: SessionId,
    bind_addr: SocketAddr,
}

impl IncomingTransport for HttpIncoming {
    async fn accept(&mut self) -> TransportResult<()> {
        // Set up HTTP server using core
        // Handle request/response at protocol layer
    }
    
    // Similar delegation pattern
}
```

### Step 4: Refactor SSE Transports (30 min)

Update SSE implementations:

```rust
// src/transport/directional/sse_incoming.rs

use crate::transport::raw::sse::SseCore;

pub struct SseIncoming {
    core: SseCore,
    http_core: HttpCore,  // For HTTP transport
    session_id: SessionId,
}

impl IncomingTransport for SseIncoming {
    async fn receive_request(&mut self) -> TransportResult<MessageEnvelope> {
        // Use SseCore for event parsing
        // Use HttpCore for transport
        // Handle MCP deserialization here
    }
}
```

### Step 5: Verify No Duplication (20 min)

Ensure all raw I/O logic has been moved:

```bash
# Check for duplicate implementations
rg "AsyncWriteExt|AsyncBufReadExt" src/transport/directional/

# Should only see imports, not implementations
```

## Validation Steps

1. **API unchanged**:
   - Public trait methods identical
   - Same error types returned
   - Behavior preserved

2. **Tests pass**:
   ```bash
   cargo test transport::directional
   ```

3. **Code reduction**:
   - Measure LOC before/after
   - Should see ~50% reduction in directional modules

## Success Criteria
- [ ] All directional transports refactored
- [ ] Using shared raw primitives
- [ ] No duplicate I/O code
- [ ] All tests passing
- [ ] Same public API maintained

## Duration Estimate
**Total: 2 hours** (as per implementation-roadmap.md)
- StdioIncoming refactor: 30 min
- StdioOutgoing refactor: 30 min
- HTTP transports: 30 min
- SSE transports: 30 min
- Verification: 20 min

## Notes
- Pure refactoring - no behavior changes
- Maintain all optimizations
- Keep error handling consistent
- Reference implementation-roadmap.md lines 253-282

---

**Task Status**: Ready for implementation
**References**: analysis/implementation-roadmap.md (Task 2.2)
**Risk Level**: Low - Internal refactoring only