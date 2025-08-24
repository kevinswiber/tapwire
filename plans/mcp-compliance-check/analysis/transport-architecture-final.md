# Final Transport Architecture Design

## ⚠️ SUPERSEDED by transport-architecture-final-v2.md

**This document has been superseded by [transport-architecture-final-v2.md](transport-architecture-final-v2.md)**

The new architecture uses **Framed/Sink/Stream for message-level unification** instead of AsyncRead/AsyncWrite for byte-level unification. This provides:
- Better abstraction (messages not bytes)
- Simpler implementation (Framed does the work)
- Full ecosystem compatibility (futures combinators)
- Validated by RMCP's SinkStreamTransport

Please refer to the v2 document for the current architecture.

---

## [SUPERSEDED] Core Decision: Protocol-Specific Transports

After investigating RMCP and analyzing protocol requirements, we've reached a clear architectural decision:

### Two Transport Categories

#### 1. Stream-Based Transports (AsyncRead + AsyncWrite)
For protocols that work over continuous bidirectional byte streams:
- **Stdio**: Server's own stdin/stdout
- **Subprocess**: Spawned process stdin/stdout  
- **TCP/Unix sockets**: Network streams
- **In-memory**: Testing with duplex channels

These all share:
- Line-delimited JSON format
- Continuous bidirectional communication
- Natural AsyncRead/AsyncWrite mapping

#### 2. HTTP-Based Transports
For protocols that use HTTP semantics:
- **HTTP Request/Response**: Single message exchange
- **Server-Sent Events (SSE)**: Server→Client streaming
- **WebSocket** (future): Bidirectional after upgrade

These require:
- HTTP headers and status codes
- Content-Type negotiation
- Protocol upgrade handling

## Implemented Architecture

```rust
// Unified trait for all transports
pub trait Transport: Send + Sync {
    async fn send(&mut self, message: Value) -> Result<()>;
    async fn receive(&mut self) -> Result<Option<Value>>; 
    async fn close(&mut self) -> Result<()>;
}

// Stream-based transport for ANY AsyncRead + AsyncWrite
pub struct StreamTransport<R: AsyncRead, W: AsyncWrite> {
    reader: FramedRead<R, JsonLineCodec>,
    writer: Arc<Mutex<FramedWrite<W, JsonLineCodec>>>,
}

// HTTP-based transport with SSE support
pub struct HttpTransport {
    client: hyper::Client,
    base_url: Url,
    sse_receiver: Option<SseReceiver>, // Activated when server returns SSE
}
```

### Convenience Constructors

```rust
impl StreamTransport<Stdin, Stdout> {
    /// For MCP servers
    pub fn stdio() -> Self {
        Self::new(tokio::io::stdin(), tokio::io::stdout())
    }
}

impl StreamTransport<ChildStdout, ChildStdin> {
    /// For MCP clients spawning servers
    pub fn subprocess(cmd: &str, args: &[&str]) -> Result<Self> {
        // Spawn process, extract streams
    }
}

impl<R: AsyncRead, W: AsyncWrite> StreamTransport<R, W> {
    /// For custom streams (TCP, Unix sockets, etc.)
    pub fn new(reader: R, writer: W) -> Self {
        // Generic constructor
    }
}
```

## Why Not Unify Everything Under AsyncRead/AsyncWrite?

We investigated whether HTTP/SSE could use StreamTransport, but the protocols are fundamentally different:

| Protocol | Communication Model | Message Framing | State Management |
|----------|-------------------|-----------------|------------------|
| **Stdio** | Continuous bidirectional stream | Line-delimited JSON | Stateless |
| **HTTP** | Request/Response pairs | HTTP envelope + JSON body | Per-request |
| **SSE** | Server→Client stream | `data: {json}\n\n` format | Long-lived GET |
| **WebSocket** | Bidirectional messages | WebSocket frames + JSON | Stateful connection |

Forcing HTTP through AsyncRead/AsyncWrite would require:
- Buffering and parsing HTTP headers
- Managing request/response correlation
- Handling content-type negotiation
- Complex state machines

This would add complexity without benefit.

## Future WebSocket Support

Based on the [MCP WebSocket proposal](https://github.com/modelcontextprotocol/modelcontextprotocol/issues/1288):

### WebSocket Lifecycle
```
1. HTTP Request with Upgrade headers
   ↓
2. Server Decision Point:
   a. Reject → Return HTTP response (application/json)
   b. Accept → Switch to WebSocket protocol
   ↓
3. WebSocket: Bidirectional JSON-RPC messages
```

### Proposed WebSocket Transport Design

```rust
pub struct WebSocketTransport {
    state: WebSocketState,
}

enum WebSocketState {
    // Pre-upgrade: acts like HttpTransport
    Connecting {
        client: hyper::Client,
        url: Url,
    },
    
    // Post-upgrade: acts like StreamTransport  
    Connected {
        ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    },
    
    // Fallback: single HTTP response
    HttpFallback {
        response: Option<Value>,
    },
}

impl Transport for WebSocketTransport {
    async fn send(&mut self, message: Value) -> Result<()> {
        match &mut self.state {
            WebSocketState::Connecting { .. } => {
                // Send HTTP request with Upgrade headers
                // Transition to Connected or HttpFallback
            }
            WebSocketState::Connected { ws } => {
                // Send over WebSocket (like StreamTransport)
                ws.send(Message::Text(serde_json::to_string(&message)?)).await?;
            }
            WebSocketState::HttpFallback { .. } => {
                // Can't send after fallback
                Err(Error::NotConnected)
            }
        }
    }
    
    async fn receive(&mut self) -> Result<Option<Value>> {
        match &mut self.state {
            WebSocketState::Connected { ws } => {
                // Receive from WebSocket stream
                if let Some(msg) = ws.next().await {
                    // Parse WebSocket message as JSON
                }
            }
            WebSocketState::HttpFallback { response } => {
                // Return the single HTTP response
                Ok(response.take())
            }
            _ => Err(Error::NotConnected),
        }
    }
}
```

### Key Insights for WebSocket

1. **Dual Nature**: WebSocket starts as HTTP but becomes stream-like
2. **State Machine**: Need to handle pre-upgrade, post-upgrade, and fallback states
3. **Reuse Patterns**: Post-upgrade WebSocket is similar to StreamTransport
4. **Library Choice**: Use `tokio-tungstenite` which provides Sink/Stream interface

## Implementation Roadmap

### Current Status (Implemented)
- ✅ `Transport` trait
- ✅ `StdioTransport` (should refactor to StreamTransport)
- ✅ `SubprocessTransport` (should refactor to StreamTransport)
- ✅ `HttpTransport` with SSE support

### Immediate Refactor
1. Replace `StdioTransport` and `SubprocessTransport` with unified `StreamTransport<R, W>`
2. Add convenience constructors for common cases
3. Consider adding `Arc<Mutex>` for concurrent sends

### Future Addition
When WebSocket support is needed:
1. Add `WebSocketTransport` with state machine
2. Use `tokio-tungstenite` for WebSocket protocol
3. Handle upgrade negotiation and fallback
4. Reuse patterns from StreamTransport for post-upgrade phase

## Design Principles

1. **Protocol-Appropriate**: Don't force square pegs into round holes
2. **Type Safety**: Use Rust's type system to prevent misuse
3. **Flexibility**: Support custom streams via generic parameters
4. **Future-Proof**: Design allows WebSocket without major refactoring
5. **Simplicity**: Each transport handles its protocol idiomatically

## Summary

The final architecture uses:
- **StreamTransport<R, W>** for anything AsyncRead/AsyncWrite (stdio, subprocess, TCP)
- **HttpTransport** for HTTP request/response with SSE support
- **WebSocketTransport** (future) for WebSocket with HTTP upgrade

This design:
- Maximizes code reuse where protocols are similar
- Keeps implementations clean and protocol-specific
- Provides a unified `Transport` trait interface
- Easily extends to new protocols like WebSocket

The key insight: **Group transports by their underlying protocol model, not by their use case.**