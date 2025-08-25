# Client/Server Consolidation Plan

**Date**: 2025-08-25  
**Status**: Planning Document  
**Context**: Path to consolidate multiple client/server implementations

## Current State

We have 6 implementations across 3 patterns:

```
Pattern 1: Sink/Stream (Legacy)
├── client.rs    - Original implementation
└── server.rs    - Original implementation

Pattern 2: Connection Trait (Target)
├── client2.rs   - Has concurrency issues (deadlock)
└── server2.rs   - Basic implementation

Pattern 3: Pooled (Enhanced)
├── client_pooled.rs - Factory pattern with pool
└── server_pooled.rs - Multi-client management
```

## Consolidation Strategy

### Phase 1: Complete Connection Implementations ✅ (Mostly Done)
- ✅ **StdioConnection** - Already implemented in `connection/stdio.rs`
- ✅ **Http2Connection** - Already implemented in `connection/http.rs`
- 🔄 **WebSocketConnection** - Placeholder created, can add later

### Phase 2: Fix client2/server2 Issues
1. **Fix client2 deadlock** - Background receiver task pattern
2. **Improve server2** - Better error handling and lifecycle
3. **Add tests** - Ensure Connection pattern works correctly

### Phase 3: Consolidation (C.7.4)
```bash
# Step 1: Ensure client2/server2 are stable
cargo test --lib

# Step 2: Remove legacy implementations
git rm src/client.rs src/server.rs

# Step 3: Rename new implementations
git mv src/client2.rs src/client.rs
git mv src/server2.rs src/server.rs

# Step 4: Update exports in lib.rs
# Change from:
#   pub use client::{Client, ...}
#   pub use client2::{Client as Client2, ...}
# To:
#   pub use client::{Client, ...}
#   # client2 module removed

# Step 5: Keep pooled variants as-is
# client_pooled.rs and server_pooled.rs remain separate
```

### Phase 4: Integration Points

After consolidation, we'll have a clean architecture:

```rust
// Standard usage (single connection)
let conn = StdioConnection::new();
let client = Client::new(conn, handler);

// Pooled usage (connection reuse)
let factory = Arc::new(|| async {
    Ok(StdioConnection::new())
});
let client = PooledClient::new(factory);

// Future WebSocket (drops in seamlessly)
let conn = WebSocketConnection::new("wss://api.example.com");
let client = Client::new(conn, handler);
```

## Why This Works

### 1. **Connection Trait is Protocol-Agnostic**
```rust
#[async_trait]
pub trait Connection: Send + Sync + Debug {
    async fn send(&mut self, message: Value) -> Result<()>;
    async fn receive(&mut self) -> Result<Value>;
    async fn close(&mut self) -> Result<()>;
    fn is_healthy(&self) -> bool;
    fn protocol(&self) -> Protocol;
}
```

Any transport that can send/receive JSON messages can implement this.

### 2. **Protocol Enum is Extensible**
```rust
pub enum Protocol {
    Http2,
    WebSocket,  // Ready for future
    Stdio,
    Unknown,
}
```

### 3. **Pooling Strategies are Protocol-Aware**
```rust
impl Protocol {
    pub fn default_pool_strategy(&self) -> PoolStrategy {
        match self {
            Protocol::Http2 => PoolStrategy::PerOrigin { max_per_origin: 10 },
            Protocol::WebSocket => PoolStrategy::PerSession,
            Protocol::Stdio => PoolStrategy::Singleton,
            Protocol::Unknown => PoolStrategy::None,
        }
    }
}
```

### 4. **WebSocket Can Be Added Later**

When we add WebSocket:

1. **Create `connection/websocket.rs`**
   - Implement Connection trait
   - Use tokio-tungstenite (WebSocketStream already is Sink+Stream!)
   - Handle session IDs in message payload

2. **Update transports if needed**
   - WebSocket module in transport/ for lower-level details
   - Connection wrapper for high-level API

3. **No changes needed to**:
   - Client/Server (work with any Connection)
   - PooledClient/PooledServer (work with any Connection)
   - Pool infrastructure (already protocol-aware)

## Testing the Architecture

```rust
#[test]
fn test_connection_abstraction() {
    // All these work the same way
    let stdio = StdioConnection::new();
    let http = Http2Connection::new("https://example.com");
    // Future: let ws = WebSocketConnection::new("wss://example.com");
    
    // Client works with any
    let client_stdio = Client::new(stdio, handler);
    let client_http = Client::new(http, handler);
    // Future: let client_ws = Client::new(ws, handler);
}
```

## Benefits of This Approach

1. **Clean Separation** - Transport details isolated in Connection implementations
2. **Type Safety** - Protocol enum ensures correct handling
3. **Future Proof** - New protocols just implement Connection trait
4. **Performance** - Zero overhead, no dynamic dispatch in hot path
5. **Flexibility** - Can use pooled or non-pooled as needed

## Timeline

- **Today**: Stdio complete, HTTP/2 complete
- **Next Session**: Fix client2/server2, do consolidation
- **Future**: Add WebSocket when needed (architecture ready)

## Validation Checklist

- [x] Stdio Connection works with Connection trait
- [x] HTTP/2 Connection works with Connection trait
- [x] Pool integration complete (alternative approach)
- [ ] Client2/Server2 fixed and stable
- [ ] Legacy implementations removed
- [ ] All tests passing with consolidated code
- [ ] WebSocket placeholder demonstrates extensibility