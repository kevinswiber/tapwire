# Framed/Sink/Stream Architecture for MCP Transport Unification

## Executive Summary

**Key Insight**: Instead of unifying at the AsyncRead/AsyncWrite byte level, we should unify at the MCP message level using `tokio_util::codec::Framed` with `futures::{Sink, Stream}` traits. This provides better type safety, cleaner abstractions, and natural protocol semantics.

## Current Plan vs. Proposed Architecture

### Current Plan (Byte-Level Unification)
```rust
// Unify at byte level for some transports
pub struct StreamTransport<R: AsyncRead, W: AsyncWrite> {
    reader: FramedRead<R, JsonLineCodec>,
    writer: Arc<Mutex<FramedWrite<W, JsonLineCodec>>>,
}

// Separate for HTTP/SSE
pub struct HttpTransport { /* different implementation */ }

// Common trait they all implement
trait Transport {
    async fn send(&mut self, message: Value) -> Result<()>;
    async fn receive(&mut self) -> Result<Option<Value>>;
}
```

### Proposed Architecture (Message-Level Unification)
```rust
use futures::{Sink, Stream};
use tokio_util::codec::Framed;

// All transports unified at message level
pub trait Transport: 
    Sink<JsonRpcMessage, Error = TransportError> + 
    Stream<Item = Result<JsonRpcMessage, TransportError>> + 
    Unpin + Send 
{
    // Optional: transport-specific methods
    fn transport_type(&self) -> TransportType;
}

// Or even simpler - just use the trait bounds directly
type McpTransport = Box<dyn Sink<JsonRpcMessage, Error = TransportError> 
                        + Stream<Item = Result<JsonRpcMessage, TransportError>>
                        + Unpin + Send>;
```

## Why This Is Better

### 1. **Natural Protocol Semantics**
MCP is a message-oriented protocol, not a byte-stream protocol:
- Each JSON-RPC message is discrete
- Transports handle message framing differently
- Sink/Stream models this perfectly

### 2. **Type Safety at the Right Level**
```rust
// Current: Working with untyped JSON values
async fn send(&mut self, message: Value) -> Result<()>;

// Proposed: Strongly typed messages
async fn send(&mut self, message: JsonRpcMessage) -> Result<()>;
// But even better - just use Sink::send() directly!
```

### 3. **Backpressure and Flow Control**
Sink/Stream provide built-in:
- Backpressure handling
- Buffering strategies  
- Poll-based async that composes well
- Ready/not-ready signaling

### 4. **Simplified Client/Server Implementation**
```rust
pub struct Client<T: Sink<JsonRpcMessage> + Stream<Item = Result<JsonRpcMessage>>> {
    transport: T,
    // No need for separate send/receive methods
}

impl<T> Client<T> 
where 
    T: Sink<JsonRpcMessage> + Stream<Item = Result<JsonRpcMessage>> + Unpin
{
    pub async fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        let msg = JsonRpcMessage::request(method, params);
        
        // Using Sink trait directly
        self.transport.send(msg).await?;
        
        // Using Stream trait directly  
        while let Some(response) = self.transport.next().await {
            let response = response?;
            // Process response...
        }
    }
}
```

## Implementation for Each Transport

### 1. Stdio/Subprocess (Line-Delimited JSON)
```rust
use tokio_util::codec::{Framed, LinesCodec};

pub fn stdio_transport() -> impl Sink<JsonRpcMessage> + Stream<Item = Result<JsonRpcMessage>> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    
    // Framed automatically implements Sink + Stream!
    let transport = Framed::new(
        tokio::io::join(stdin, stdout),
        JsonLineCodec::new()
    );
    
    // That's it! Framed<_, JsonLineCodec> implements both traits
    transport
}

pub fn subprocess_transport(cmd: &str, args: &[&str]) -> Result<impl Transport> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
        
    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    
    Ok(Framed::new(
        tokio::io::join(stdout, stdin),
        JsonLineCodec::new()
    ))
}
```

### 2. HTTP (Request/Response Pairs)
```rust
pub struct HttpTransport {
    client: hyper::Client<HttpsConnector>,
    url: Url,
    pending_responses: VecDeque<JsonRpcMessage>,
}

impl Sink<JsonRpcMessage> for HttpTransport {
    type Error = TransportError;
    
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Always ready to send HTTP requests
        Poll::Ready(Ok(()))
    }
    
    fn start_send(self: Pin<&mut Self>, msg: JsonRpcMessage) -> Result<(), Self::Error> {
        // Queue the request
        self.get_mut().send_request(msg);
        Ok(())
    }
    
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Execute queued HTTP requests
        self.get_mut().flush_requests(cx)
    }
}

impl Stream for HttpTransport {
    type Item = Result<JsonRpcMessage, TransportError>;
    
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Return pending responses
        if let Some(response) = self.pending_responses.pop_front() {
            Poll::Ready(Some(Ok(response)))
        } else {
            Poll::Pending
        }
    }
}
```

### 3. SSE (Server-Sent Events) 
```rust
pub struct SseTransport {
    event_stream: SseStream,           // Implements Stream
    http_client: hyper::Client<...>,   // For sending requests
    send_url: Url,
}

impl Stream for SseTransport {
    type Item = Result<JsonRpcMessage, TransportError>;
    
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Delegate to SSE event stream
        self.event_stream.poll_next(cx)
            .map(|opt| opt.map(|event| parse_sse_event(event)))
    }
}

impl Sink<JsonRpcMessage> for SseTransport {
    // Send via HTTP POST to companion endpoint
    fn start_send(self: Pin<&mut Self>, msg: JsonRpcMessage) -> Result<(), Self::Error> {
        // POST to the send URL
        self.http_client.post(self.send_url)
            .json(&msg)
            .send();
        Ok(())
    }
}
```

### 4. Future: WebSocket
```rust
pub struct WebSocketTransport {
    ws: WebSocketStream<...>,  // Already implements Sink + Stream!
}

// WebSocketStream already implements Sink<Message> + Stream<Item=Message>
// We just need to adapt Message <-> JsonRpcMessage

impl Sink<JsonRpcMessage> for WebSocketTransport {
    fn start_send(self: Pin<&mut Self>, msg: JsonRpcMessage) -> Result<(), Self::Error> {
        let ws_msg = Message::text(serde_json::to_string(&msg)?);
        self.ws.start_send(ws_msg)
    }
}

impl Stream for WebSocketTransport {
    type Item = Result<JsonRpcMessage, TransportError>;
    
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.ws.poll_next(cx) {
            Poll::Ready(Some(Ok(Message::Text(text)))) => {
                let msg = serde_json::from_str(&text)?;
                Poll::Ready(Some(Ok(msg)))
            }
            // ... handle other cases
        }
    }
}
```

## Codec Design for JsonLineCodec

```rust
use tokio_util::codec::{Decoder, Encoder};
use bytes::{BytesMut, BufMut};

pub struct JsonLineCodec;

impl Decoder for JsonLineCodec {
    type Item = JsonRpcMessage;
    type Error = TransportError;
    
    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(idx) = buf.iter().position(|&b| b == b'\n') {
            let line = buf.split_to(idx + 1);
            let line = std::str::from_utf8(&line[..line.len()-1])?;
            let msg: JsonRpcMessage = serde_json::from_str(line)?;
            Ok(Some(msg))
        } else {
            Ok(None)
        }
    }
}

impl Encoder<JsonRpcMessage> for JsonLineCodec {
    type Error = TransportError;
    
    fn encode(&mut self, msg: JsonRpcMessage, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let json = serde_json::to_string(&msg)?;
        buf.put(json.as_bytes());
        buf.put_u8(b'\n');
        Ok(())
    }
}
```

## Benefits Summary

### 1. **Unified Interface**
All transports implement the same Sink + Stream traits, regardless of underlying protocol.

### 2. **Composability**
Can use all the futures/tokio ecosystem utilities:
```rust
use futures::stream::StreamExt;
use futures::sink::SinkExt;

// Rate limiting
let transport = transport.throttle(Duration::from_millis(100));

// Buffering
let transport = transport.buffer(100);

// Timeout
let transport = transport.timeout(Duration::from_secs(30));

// Retry logic
let transport = transport.retry(3);
```

### 3. **Type Safety**
Working with typed `JsonRpcMessage` instead of untyped `Value`.

### 4. **Simplified Testing**
```rust
// Easy to create test doubles
let (tx, rx) = futures::channel::mpsc::channel(100);
let transport: Box<dyn Transport> = Box::new(tx.fanout(rx));
```

### 5. **Natural Async/Await**
```rust
// Clean async/await usage
transport.send(request).await?;
while let Some(response) = transport.next().await {
    // Process response
}
```

## Migration Path

### Phase 1: Define Transport Trait
```rust
pub trait Transport: 
    Sink<JsonRpcMessage, Error = TransportError> + 
    Stream<Item = Result<JsonRpcMessage, TransportError>> + 
    Unpin + Send {}

// Blanket impl
impl<T> Transport for T 
where 
    T: Sink<JsonRpcMessage, Error = TransportError> 
     + Stream<Item = Result<JsonRpcMessage, TransportError>>
     + Unpin + Send {}
```

### Phase 2: Implement for Each Transport
1. StdioTransport → Framed with JsonLineCodec
2. SubprocessTransport → Framed with JsonLineCodec  
3. HttpTransport → Custom Sink/Stream impl
4. SseTransport → Custom Sink/Stream impl

### Phase 3: Update Client/Server
```rust
pub struct Client<T: Transport> {
    transport: T,
}

pub struct Server<T: Transport, H: Handler> {
    transport: T,
    handler: H,
}
```

## Decision Recommendation

**STRONG RECOMMENDATION**: Adopt the Framed/Sink/Stream architecture.

### Pros:
- ✅ Better abstraction (message-level not byte-level)
- ✅ Type safety with JsonRpcMessage
- ✅ Natural async/await patterns
- ✅ Composability with futures ecosystem
- ✅ Built-in backpressure handling
- ✅ Simpler Client/Server implementation
- ✅ Future-proof for WebSocket and other transports

### Cons:
- ❓ Slightly more complex initial setup for HTTP/SSE adapters
- ❓ Need to understand Sink/Stream traits (but they're standard)

### Risk Mitigation:
- Start with stdio/subprocess (Framed makes this trivial)
- Build HTTP/SSE adapters incrementally
- Extensive testing with test doubles

## Conclusion

Using `tokio_util::codec::Framed` with `futures::{Sink, Stream}` provides a superior architecture for MCP transport unification. It operates at the right abstraction level (messages not bytes), provides better type safety, and integrates naturally with the async ecosystem.

This approach will make our MCP library more robust, easier to test, and simpler to extend with new transports in the future.

---

*Document created: 2025-08-24*  
*Key insight: Unify at the message level, not the byte level*