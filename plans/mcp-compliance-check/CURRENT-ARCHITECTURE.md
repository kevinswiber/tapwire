# Current MCP Transport Architecture (Single Source of Truth)

**Status**: FINAL - Ready for Implementation  
**Last Updated**: 2025-08-24  
**Phase**: C.5.4 Implementation

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

**ONE transport with THREE response modes:**

1. **JSON** - `200 OK` with `application/json`
2. **SSE** - `200 OK` with `text/event-stream`  
3. **WebSocket** - `101 Switching Protocols`

```rust
struct HttpTransport {
    // Handles all three modes internally
    single_responses: VecDeque<JsonRpcMessage>,
    sse_streams: HashMap<RequestId, SseStream>,
    ws_connection: Option<WebSocketStream>,
}
```

**Key insight**: Server chooses the mode based on operation. Client adapts transparently.

## Module Organization

```rust
mcp::transports::{
    stdio,          // Framed + JsonLineCodec
    subprocess,     // Framed + JsonLineCodec + process mgmt
    http,           // Custom Sink+Stream, three modes
}

// Internal (not public API):
mcp::transports::http::{
    transport,      // Main implementation
    sse,            // SSE parsing
    websocket,      // WebSocket upgrade handling
}
```

## Key Decisions

1. **Message-level unification** - Not byte-level (AsyncRead/AsyncWrite)
2. **Standard traits** - Sink + Stream, not custom Transport trait
3. **Framed for line protocols** - Only where it fits naturally
4. **HTTP adaptivity** - One transport, three modes, server-controlled
5. **Arc<Mutex> for concurrent sends** - Validated by RMCP

## Implementation Checklist

### Phase C.5.4 (Current - 3 hours)

- [ ] Create `JsonLineCodec` implementing Encoder/Decoder
- [ ] Implement `StdioTransport` using Framed
- [ ] Implement `SubprocessTransport` using Framed + Child
- [ ] Implement `HttpTransport` with three-mode handling
- [ ] Update `Client<T>` to use Sink + Stream directly
- [ ] Update `Server<T>` to use Sink + Stream directly

### Testing Strategy

```rust
// Simple test transport using channels
let (tx, rx) = futures::channel::mpsc::channel(10);
let transport = ChannelTransport::new(tx, rx);
let mut client = Client::new(transport);
```

## Documentation Map

### Current (Use These)
- **This document** - Single source of truth
- `analysis/transport-architecture-final-v2.md` - Detailed design
- `analysis/http-transport-unified-architecture.md` - HTTP's three modes

### Historical (For Context)
- `analysis/framed-sink-stream-architecture.md` - Why message-level
- `analysis/rmcp-vs-framed-comparison.md` - Validation from RMCP
- ~~`analysis/transport-architecture-final.md`~~ - Superseded

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
- **HTTP** = One transport, three modes (JSON, SSE, WebSocket)
- **Message-level** = Work with JsonRpcMessage, not bytes
- **Standard traits** = Sink + Stream, not custom

---

*This is the authoritative architecture document. All implementation should follow this design.*