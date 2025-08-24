# Task C.5.4: Implement Framed/Sink/Stream Transport Architecture

**Duration**: 3 hours  
**Dependencies**: C.5.3 (Architecture Documentation) ✅ Complete  
**Status**: Ready to implement

## Objective

Implement the finalized Framed/Sink/Stream transport architecture for the MCP library, providing message-level unification across all transport types.

## Context

After extensive investigation including RMCP analysis, we've determined that:
- Message-level unification (Sink + Stream) is superior to byte-level (AsyncRead/AsyncWrite)
- Framed with JsonLineCodec handles line-delimited protocols perfectly
- HTTP is one adaptive transport with three modes (JSON, SSE, WebSocket)
- This approach provides better abstraction and ecosystem compatibility

## Deliverables

### 1. JsonLineCodec (30 min)

Create `crates/mcp/src/transport/codec.rs`:

```rust
use tokio_util::codec::{Decoder, Encoder};
use bytes::BytesMut;

pub struct JsonLineCodec;

impl Decoder for JsonLineCodec {
    type Item = JsonRpcMessage;
    type Error = TransportError;
    // Implement: Split on \n, parse JSON
}

impl Encoder<JsonRpcMessage> for JsonLineCodec {
    type Error = TransportError;
    // Implement: Serialize to JSON, append \n
}
```

### 2. StdioTransport (30 min)

Update `crates/mcp/src/transport/stdio.rs`:

```rust
use tokio_util::codec::Framed;

pub type StdioTransport = Framed<StdioStream, JsonLineCodec>;

pub fn stdio() -> StdioTransport {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    Framed::new(
        StdioStream::new(stdin, stdout),
        JsonLineCodec::new()
    )
}

// Helper to join stdin/stdout
struct StdioStream { /* ... */ }
impl AsyncRead for StdioStream { /* ... */ }
impl AsyncWrite for StdioStream { /* ... */ }
```

### 3. SubprocessTransport (45 min)

Update `crates/mcp/src/transport/subprocess.rs`:

```rust
pub struct SubprocessTransport {
    framed: Framed<ChildStdio, JsonLineCodec>,
    child: Child,  // Keep for cleanup
}

impl SubprocessTransport {
    pub fn spawn(cmd: &str, args: &[&str]) -> Result<Self> {
        // Spawn process, create Framed
    }
}

// Delegate Sink + Stream to framed
impl Sink<JsonRpcMessage> for SubprocessTransport { /* delegate */ }
impl Stream for SubprocessTransport { /* delegate */ }

impl Drop for SubprocessTransport {
    fn drop(&mut self) {
        // Clean up child process
    }
}
```

### 4. HttpTransport (60 min)

Update `crates/mcp/src/transport/http.rs`:

```rust
pub struct HttpTransport {
    client: hyper::Client<HttpsConnector>,
    url: Url,
    
    // Three modes of operation
    single_responses: VecDeque<JsonRpcMessage>,
    sse_streams: HashMap<RequestId, SseStream>,
    ws_connection: Option<WebSocketStream>,
    
    transport_mode: TransportMode,
}

enum TransportMode {
    RequestResponse,
    Upgraded(UpgradeType),
}

impl HttpTransport {
    async fn send_request(&mut self, msg: JsonRpcMessage) -> Result<()> {
        // Check if WebSocket active
        // Otherwise HTTP POST
        // Handle 200 JSON, 200 SSE, or 101 WebSocket
    }
}

impl Sink<JsonRpcMessage> for HttpTransport { /* ... */ }
impl Stream for HttpTransport { 
    // Poll single responses, then SSE, then WebSocket
}
```

### 5. Update Client/Server (45 min)

Update `crates/mcp/src/client.rs`:

```rust
use futures::{SinkExt, StreamExt};

pub struct Client<T> 
where 
    T: Sink<JsonRpcMessage, Error = TransportError> 
     + Stream<Item = Result<JsonRpcMessage, TransportError>>
     + Unpin
{
    transport: T,
    pending: HashMap<MessageId, oneshot::Sender<JsonRpcResponse>>,
}

impl<T> Client<T> 
where T: Sink<JsonRpcMessage> + Stream<Item = Result<JsonRpcMessage>> + Unpin
{
    pub async fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        // Use transport.send() directly (Sink trait)
        // Use transport.next() directly (Stream trait)
    }
}
```

Similar updates for `Server<T>`.

## Success Criteria

- [ ] All transports implement Sink + Stream traits
- [ ] JsonLineCodec correctly handles line-delimited JSON
- [ ] StdioTransport works with stdin/stdout
- [ ] SubprocessTransport manages child process lifecycle
- [ ] HttpTransport handles all three modes transparently
- [ ] Client/Server use standard traits directly
- [ ] Tests pass using channel-based test transport

## Testing Approach

Create `crates/mcp/src/transport/test.rs`:

```rust
pub struct ChannelTransport {
    tx: mpsc::Sender<JsonRpcMessage>,
    rx: mpsc::Receiver<JsonRpcMessage>,
}

impl Sink<JsonRpcMessage> for ChannelTransport { /* ... */ }
impl Stream for ChannelTransport { /* ... */ }

#[tokio::test]
async fn test_client_with_channel() {
    let (tx1, rx1) = mpsc::channel(10);
    let (tx2, rx2) = mpsc::channel(10);
    
    let client_transport = ChannelTransport::new(tx1, rx2);
    let server_transport = ChannelTransport::new(tx2, rx1);
    
    let mut client = Client::new(client_transport);
    let mut server = Server::new(server_transport, handler);
    
    // Test bidirectional communication
}
```

## Commands to Run

```bash
cd /Users/kevin/src/tapwire/shadowcat-mcp-compliance
cargo build --package mcp
cargo test --package mcp transport::
cargo test --package mcp client::
cargo test --package mcp server::
```

## References

- `/Users/kevin/src/tapwire/plans/mcp-compliance-check/CURRENT-ARCHITECTURE.md` - Single source of truth
- `/Users/kevin/src/tapwire/plans/mcp-compliance-check/analysis/transport-architecture-final-v2.md` - Detailed design
- `/Users/kevin/src/tapwire/plans/mcp-compliance-check/analysis/http-transport-unified-architecture.md` - HTTP three modes

## Notes

- Remember: Framed is ONLY for line-delimited JSON
- HTTP is ONE transport with THREE modes
- Use Arc<Mutex> for Sink when needed for concurrent sends
- WebSocket is part of HTTP transport, not separate

---

*Task created: 2025-08-24*  
*Ready for implementation in Phase C.5.4*