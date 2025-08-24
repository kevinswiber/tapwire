# Transport Architecture Final v2: Framed/Sink/Stream Design

## Status: CURRENT ARCHITECTURE (Supersedes transport-architecture-final.md)

**Created**: 2025-08-24  
**Supersedes**: transport-architecture-final.md, transport-deviation-analysis.md  
**Validated by**: RMCP implementation analysis

## Executive Summary

After analyzing RMCP's implementation and reconsidering our abstraction level, we're adopting a **Framed/Sink/Stream architecture** that unifies transports at the message level rather than the byte level. This approach is simpler, more idiomatic, and leverages Rust's async ecosystem better.

## Final Architecture Decision

### Core Abstraction: Message-Level Unification
```rust
use futures::{Sink, Stream};
use tokio_util::codec::Framed;

// All transports implement these standard traits
pub trait Transport: 
    Sink<JsonRpcMessage, Error = TransportError> + 
    Stream<Item = Result<JsonRpcMessage, TransportError>> + 
    Unpin + Send 
{
    // Optional: transport-specific methods
    fn transport_type(&self) -> TransportType;
}

// Blanket implementation
impl<T> Transport for T 
where 
    T: Sink<JsonRpcMessage, Error = TransportError> 
     + Stream<Item = Result<JsonRpcMessage, TransportError>>
     + Unpin + Send {}
```

### Important Clarification (UPDATED)
- **Framed** is ONLY used for line-delimited JSON protocols (stdio, subprocess)
- **HTTP** is ONE transport with THREE response modes:
  - JSON response (200 OK + application/json)
  - SSE streaming (200 OK + text/event-stream)
  - WebSocket upgrade (101 Switching Protocols)
- All three HTTP modes are handled by a single HttpTransport implementation
- See [http-transport-unified-architecture.md](http-transport-unified-architecture.md) for details

### Transport Implementations

#### 1. Stdio Transport (Line-Delimited JSON)
```rust
pub type StdioTransport = Framed<StdioStream, JsonLineCodec>;

pub fn stdio() -> StdioTransport {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    Framed::new(
        StdioStream::new(stdin, stdout),
        JsonLineCodec::new()
    )
}
```

#### 2. Subprocess Transport (Line-Delimited JSON)
```rust
pub struct SubprocessTransport {
    framed: Framed<ChildStdio, JsonLineCodec>,
    child: Child,  // Keep handle for cleanup
}

impl SubprocessTransport {
    pub fn spawn(cmd: &str, args: &[&str]) -> Result<Self> {
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
            
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        
        Ok(Self {
            framed: Framed::new(
                ChildStdio::new(stdout, stdin),
                JsonLineCodec::new()
            ),
            child,
        })
    }
}

// Implement Sink + Stream by delegating to framed
impl Sink<JsonRpcMessage> for SubprocessTransport {
    type Error = TransportError;
    
    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.framed).poll_ready(cx)
    }
    
    fn start_send(mut self: Pin<&mut Self>, item: JsonRpcMessage) -> Result<(), Self::Error> {
        Pin::new(&mut self.framed).start_send(item)
    }
    
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.framed).poll_flush(cx)
    }
}

impl Stream for SubprocessTransport {
    type Item = Result<JsonRpcMessage, TransportError>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.framed).poll_next(cx)
    }
}
```

#### 3. HTTP Transport (Streamable HTTP - Handles JSON and SSE)
```rust
pub struct HttpTransport {
    client: hyper::Client<HttpsConnector>,
    url: Url,
    pending_requests: VecDeque<JsonRpcMessage>,
    single_responses: VecDeque<JsonRpcMessage>,
    sse_streams: HashMap<RequestId, SseStream>,
}

impl HttpTransport {
    async fn send_request(&mut self, msg: JsonRpcMessage) -> Result<()> {
        let request = Request::post(&self.url)
            .header("Accept", "application/json, text/event-stream")
            .body(Body::from(serde_json::to_vec(&msg)?))?;
        
        let response = self.client.request(request).await?;
        
        // MCP server chooses response type based on operation
        match response.headers().get("content-type").map(|v| v.to_str()) {
            Some(Ok("application/json")) => {
                // Single JSON response
                let body = hyper::body::to_bytes(response.into_body()).await?;
                let msg: JsonRpcMessage = serde_json::from_slice(&body)?;
                self.single_responses.push_back(msg);
            }
            Some(Ok("text/event-stream")) => {
                // SSE stream of responses
                let stream = SseStream::new(response.into_body());
                self.sse_streams.insert(msg.id, stream);
            }
            _ => return Err(TransportError::InvalidContentType),
        }
        Ok(())
    }
}

impl Sink<JsonRpcMessage> for HttpTransport {
    type Error = TransportError;
    
    fn start_send(mut self: Pin<&mut Self>, msg: JsonRpcMessage) -> Result<(), Self::Error> {
        self.pending_requests.push_back(msg);
        Ok(())
    }
    
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Send all pending requests
        while let Some(msg) = self.pending_requests.pop_front() {
            // This would need to be properly async, simplified here
            self.get_mut().send_request(msg);
        }
        Poll::Ready(Ok(()))
    }
}

impl Stream for HttpTransport {
    type Item = Result<JsonRpcMessage, TransportError>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // First check single responses
        if let Some(msg) = self.single_responses.pop_front() {
            return Poll::Ready(Some(Ok(msg)));
        }
        
        // Then poll SSE streams
        for stream in self.sse_streams.values_mut() {
            if let Poll::Ready(Some(event)) = stream.poll_next_unpin(cx) {
                match parse_sse_event(event) {
                    Ok(msg) => return Poll::Ready(Some(Ok(msg))),
                    Err(e) => return Poll::Ready(Some(Err(e))),
                }
            }
        }
        
        Poll::Pending
    }
}
```

Note: This is ONE transport that handles both response types based on the server's Content-Type header. The client doesn't choose between HTTP or SSE - the server decides based on the operation.

### JsonLineCodec Implementation
```rust
use tokio_util::codec::{Decoder, Encoder};
use bytes::BytesMut;

pub struct JsonLineCodec;

impl Decoder for JsonLineCodec {
    type Item = JsonRpcMessage;
    type Error = TransportError;
    
    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(i) = buf.iter().position(|&b| b == b'\n') {
            let line = buf.split_to(i);
            buf.advance(1); // Skip newline
            
            let msg: JsonRpcMessage = serde_json::from_slice(&line)?;
            Ok(Some(msg))
        } else {
            Ok(None)
        }
    }
}

impl Encoder<JsonRpcMessage> for JsonLineCodec {
    type Error = TransportError;
    
    fn encode(&mut self, msg: JsonRpcMessage, buf: &mut BytesMut) -> Result<(), Self::Error> {
        serde_json::to_writer(buf.writer(), &msg)?;
        buf.extend_from_slice(b"\n");
        Ok(())
    }
}
```

### Client/Server Implementation
```rust
pub struct Client<T> 
where 
    T: Sink<JsonRpcMessage, Error = TransportError> 
     + Stream<Item = Result<JsonRpcMessage, TransportError>>
     + Unpin
{
    transport: T,
    pending_requests: HashMap<MessageId, oneshot::Sender<JsonRpcResponse>>,
}

impl<T> Client<T> 
where 
    T: Sink<JsonRpcMessage, Error = TransportError> 
     + Stream<Item = Result<JsonRpcMessage, TransportError>>
     + Unpin
{
    pub async fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = MessageId::new();
        let request = JsonRpcMessage::request(id.clone(), method, params);
        
        // Send using Sink trait
        self.transport.send(request).await?;
        
        // Set up response channel
        let (tx, rx) = oneshot::channel();
        self.pending_requests.insert(id, tx);
        
        // Await response
        rx.await?
    }
    
    pub async fn run(&mut self) {
        use futures::StreamExt;
        
        while let Some(msg) = self.transport.next().await {
            match msg {
                Ok(JsonRpcMessage::Response(resp)) => {
                    if let Some(tx) = self.pending_requests.remove(&resp.id) {
                        let _ = tx.send(resp);
                    }
                }
                // Handle other message types...
                _ => {}
            }
        }
    }
}
```

## Key Architecture Decisions

### 1. Message-Level Unification
- Unify at JsonRpcMessage level, not bytes
- All transports speak messages, not raw data
- Natural fit for MCP's message-oriented protocol

### 2. Standard Trait Usage
- Use `futures::{Sink, Stream}` directly
- No custom Transport trait needed
- Full ecosystem compatibility

### 3. Framed for Line-Delimited Protocols ONLY
- `tokio_util::codec::Framed` for stdio/subprocess
- Creates JsonLineCodec for newline-delimited JSON
- Automatic Sink + Stream implementation
- NOT used for HTTP (different framing)

### 4. HTTP as Single Unified Transport
- **One transport** handling both JSON and SSE responses
- Server chooses response type via Content-Type
- Client handles both transparently
- SSE parsing is internal implementation detail
- Custom Sink/Stream implementation (no Framed)

### 5. Concurrent Sends via Arc<Mutex>
- Validated by RMCP implementation
- Wrap Sink in Arc<Mutex> when needed
- Allows multiple tasks to send concurrently

## Benefits Over Previous Design

### vs. StreamTransport<R: AsyncRead, W: AsyncWrite>
- ❌ Byte-level abstraction - too low level
- ❌ HTTP/SSE don't fit AsyncRead/Write model
- ✅ Message-level abstraction - perfect fit
- ✅ All transports unified under Sink/Stream

### vs. Custom Transport Trait (like RMCP)
- ❌ Extra indirection through custom trait
- ❌ Can't use ecosystem tools directly
- ✅ Direct Sink/Stream usage
- ✅ Full futures ecosystem compatibility

## Implementation Priority

1. **JsonLineCodec** - Core codec for line-delimited JSON (stdio/subprocess ONLY)
2. **StdioTransport** - Using Framed with JsonLineCodec
3. **SubprocessTransport** - Using Framed with JsonLineCodec + process management
4. **HttpTransport** - Custom Sink/Stream handling both JSON and SSE responses
5. **Future: WebSocketTransport** - Separate transport (not HTTP variant)

## Testing Strategy

Super simple with channels:
```rust
use futures::channel::mpsc;

#[tokio::test]
async fn test_client() {
    let (tx, rx) = mpsc::channel(10);
    let transport = ChannelTransport::new(tx, rx);
    let mut client = Client::new(transport);
    
    // Test away!
}
```

## Module Organization (UPDATED)

```rust
mcp::transports::{
    stdio,          // Line-delimited JSON over stdin/stdout
    subprocess,     // Line-delimited JSON over child process stdio
    http,           // HTTP transport with three modes: JSON, SSE, WebSocket
}

// Internal modules (not public API):
mcp::transports::http::{
    transport,      // Main HttpTransport implementation
    sse,            // SSE event parsing utilities
    websocket,      // WebSocket upgrade handling (part of HTTP transport)
    response,       // Response mode detection
}
```

## Future Extensibility

### WebSocket (UPDATED: Part of HTTP Transport)
WebSocket is now understood to be a mode of the HTTP transport, not a separate transport:
- Initiated via HTTP request with Upgrade headers
- Server responds with 101 Switching Protocols
- Connection upgrades to WebSocket for bidirectional streaming
- Handled internally by HttpTransport

### TCP/Unix Sockets
```rust
// Just use Framed with JsonLineCodec!
type TcpTransport = Framed<TcpStream, JsonLineCodec>;
type UnixTransport = Framed<UnixStream, JsonLineCodec>;
```

## Migration Notes

This architecture supersedes:
- `transport-architecture-final.md` - Used AsyncRead/AsyncWrite abstraction
- `transport-deviation-analysis.md` - Questioned subprocess inclusion
- Original Phase C.5.4 plan - Was going to implement StreamTransport<R,W>

## Conclusion

The Framed/Sink/Stream architecture provides the best balance of:
- **Simplicity** - Standard traits, no custom abstractions
- **Power** - Full async ecosystem compatibility
- **Flexibility** - Works for all transport types
- **Type Safety** - Message-level typing

This is our final transport architecture decision.

---

*Architecture validated by RMCP analysis showing they support the same approach via SinkStreamTransport*