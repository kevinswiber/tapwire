# Current MCP Transport Architecture (Single Source of Truth)

**Status**: UPDATED - Critical bugs identified, WebSocket separated  
**Last Updated**: 2025-08-24  
**Phase**: C.6 Critical Bug Fixes

## The Final Architecture

All transports implement **`Sink<JsonRpcMessage> + Stream<Item = Result<JsonRpcMessage>>`**

This provides:
- Message-level abstraction (not byte-level)
- Standard async traits (not custom)
- Full ecosystem compatibility
- Type safety with JsonRpcMessage

## Transport Implementations

### 1. Line-Delimited JSON Transports

**Use `tokio_util::codec::Framed` with `JsonLineCodec`**

```rust
// Stdio
type StdioTransport = Framed<StdioStream, JsonLineCodec>;

// Subprocess (with process management)
struct SubprocessTransport {
    framed: Framed<ChildStdio, JsonLineCodec>,
    child: Child,  // For cleanup
}
```

**Applies to**: stdio, subprocess, future TCP/Unix sockets

### 2. HTTP Transport (Adaptive)

**ONE transport with TWO response modes:**

1. **JSON** - `200 OK` with `application/json`
2. **SSE** - `200 OK` with `text/event-stream`

```rust
struct HttpTransport {
    // Worker pattern with channels
    request_tx: mpsc::Sender<Value>,
    response_rx: mpsc::Receiver<io::Result<Value>>,
    worker_handle: JoinHandle<()>,
}
```

**Key insight**: Server chooses JSON or SSE based on operation. Client adapts transparently.

### 3. WebSocket Transport (Separate)

**Completely separate transport with different lifecycle:**

```rust
struct WebSocketTransport {
    // GET + Upgrade handshake
    // Sessions REQUIRED in every message
    // Single connection per session enforcement
}
```

**Key differences**: Different handshake, auth model, session requirements

## Module Organization

```rust
mcp::transports::{
    codec,          // JsonLineCodec for line-delimited
    stdio,          // Framed + JsonLineCodec
    subprocess,     // Framed + JsonLineCodec + process mgmt
    http,           // Worker pattern, JSON/SSE modes
    websocket,      // Separate transport (feature-gated)
}

// Internal (not public API):
mcp::transports::http::{
    mod,            // Main worker implementation
    streaming/sse,  // SSE parsing and reconnection
}
```

## Key Decisions

1. **Message-level unification** - Not byte-level (AsyncRead/AsyncWrite)
2. **Standard traits** - Sink + Stream, not custom Transport trait
3. **Framed for line protocols** - Only where it fits naturally
4. **HTTP adaptivity** - One transport, two modes (JSON/SSE), server-controlled
5. **WebSocket separation** - Different transport with different lifecycle
6. **Worker pattern for HTTP** - Handles async operations and SSE streams
7. **Background receiver for Client** - Prevents deadlock in concurrent ops

## Implementation Status

### Phase C.5.4 (Completed ✅)

- [x] Create `JsonLineCodec` implementing Encoder/Decoder
- [x] Implement `StdioTransport` using Framed
- [x] Implement `SubprocessTransport` using Framed + Child
- [x] Implement `HttpTransport` basic structure
- [x] Update `Client<T>` to use Sink + Stream directly
- [x] Update `Server<T>` to use Sink + Stream directly

### Phase C.6: Critical Bugs (Current 🔴)

- [ ] **Fix Client deadlock** - Background receiver needed
- [ ] **Fix HTTP transport** - Worker pattern required
- [ ] **Create WebSocket** - Separate transport module
- [ ] **Harden JsonLineCodec** - CRLF, overlong lines
- [ ] **Wire version negotiation** - Connect to version module

### Testing Strategy

```rust
// Simple test transport using channels
let (tx, rx) = futures::channel::mpsc::channel(10);
let transport = ChannelTransport::new(tx, rx);
let mut client = Client::new(transport);
```

## Documentation Map

### Current (Use These)
- **This document** - Architecture overview
- **`TRANSPORT-ARCHITECTURE-FINAL.md`** - Consolidated transport decision
- `analysis/gpt-findings-analysis.md` - Critical bugs to fix
- `analysis/websocket-separation-decision.md` - WebSocket rationale

### Deprecated (Historical Only)
- Multiple transport architecture docs consolidated into FINAL
- See `analysis/README.md` for complete deprecated list

## Quick Reference

```rust
// All transports implement:
trait Transport: 
    Sink<JsonRpcMessage, Error = TransportError> + 
    Stream<Item = Result<JsonRpcMessage, TransportError>> + 
    Unpin + Send {}

// Usage in Client/Server:
impl<T: Transport> Client<T> {
    async fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        self.transport.send(request).await?;  // Using Sink
        while let Some(msg) = self.transport.next().await {  // Using Stream
            // Handle response
        }
    }
}
```

## Remember

- **Framed** = Line-delimited JSON only (stdio, subprocess)
- **HTTP** = One transport, two modes (JSON, SSE)
- **WebSocket** = Separate transport (not HTTP sub-mode)
- **Message-level** = Work with Value, not bytes
- **Standard traits** = Sink + Stream, not custom
- **Critical bugs** = Client deadlock, HTTP not working

---

*This is the authoritative architecture document. All implementation should follow this design.*