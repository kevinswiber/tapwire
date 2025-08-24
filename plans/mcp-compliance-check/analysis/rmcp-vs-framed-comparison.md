# RMCP Transport vs. Framed/Sink/Stream Comparison

## Executive Summary

RMCP actually supports BOTH approaches! They have a flexible architecture that allows AsyncRead/AsyncWrite, Sink/Stream, and custom transports. Our proposed Framed/Sink/Stream approach is **complementary** to their design, not competing. However, we can simplify by focusing on Sink/Stream as the primary abstraction.

## RMCP's Transport Architecture

### Core Transport Trait
```rust
pub trait Transport<R: ServiceRole>: Send {
    type Error: std::error::Error + Send + Sync + 'static;
    
    fn send(&mut self, item: TxJsonRpcMessage<R>) 
        -> impl Future<Output = Result<(), Self::Error>> + Send + 'static;
    
    fn receive(&mut self) 
        -> impl Future<Output = Option<RxJsonRpcMessage<R>>> + Send;
    
    fn close(&mut self) 
        -> impl Future<Output = Result<(), Self::Error>> + Send;
}
```

### Multiple Transport Adapters

RMCP provides THREE different ways to create transports:

#### 1. AsyncRead/AsyncWrite Transport
```rust
pub struct AsyncRwTransport<Role, R: AsyncRead, W: AsyncWrite> {
    read: FramedRead<R, JsonRpcMessageCodec<RxJsonRpcMessage<Role>>>,
    write: Arc<Mutex<FramedWrite<W, JsonRpcMessageCodec<TxJsonRpcMessage<Role>>>>>,
}
```
- Uses `tokio_util::codec::{FramedRead, FramedWrite}`
- Wraps writer in `Arc<Mutex>` for concurrent sends
- Line-delimited JSON codec

#### 2. Sink/Stream Transport
```rust
pub struct SinkStreamTransport<Si, St> {
    stream: St,
    sink: Arc<Mutex<Si>>,
}
```
- Direct Sink + Stream wrapper
- Also uses `Arc<Mutex>` for sink
- Works with any Sink/Stream pair

#### 3. Worker Transport
```rust
pub struct WorkerTransport { /* ... */ }
```
- Runs processing in separate task
- Message passing via channels

### IntoTransport Trait
```rust
pub trait IntoTransport<R, E, A>: Send + 'static {
    fn into_transport(self) -> impl Transport<R, Error = E> + 'static;
}
```

Automatic conversions for:
- `(R: AsyncRead, W: AsyncWrite)` → AsyncRwTransport
- `(Si: Sink, St: Stream)` → SinkStreamTransport  
- Any type implementing both `Sink + Stream` → SinkStreamTransport
- Worker types → WorkerTransport

## Our Proposed Approach

### Direct Sink/Stream as Primary Interface
```rust
// Instead of custom Transport trait, use standard traits directly
pub trait Transport: 
    Sink<JsonRpcMessage, Error = TransportError> + 
    Stream<Item = Result<JsonRpcMessage, TransportError>> + 
    Unpin + Send {}

// All transports implement standard traits
impl Sink<JsonRpcMessage> for StdioTransport { /* ... */ }
impl Stream for StdioTransport { /* ... */ }
```

## Comparison Table

| Aspect | RMCP Approach | Our Framed/Sink/Stream | Winner |
|--------|---------------|------------------------|---------|
| **Flexibility** | Custom trait + adapters for everything | Direct Sink/Stream traits | RMCP ✅ |
| **Simplicity** | Custom trait adds indirection | Standard traits, no wrapper | Ours ✅ |
| **Type Safety** | Role-based typing (Client/Server) | Message-level typing | RMCP ✅ |
| **Composability** | Via custom adapters | Direct futures ecosystem | Ours ✅ |
| **Testing** | Need to mock Transport trait | Use channel Sink/Stream | Ours ✅ |
| **Ecosystem** | Custom ecosystem | Standard futures ecosystem | Ours ✅ |
| **Learning Curve** | Must learn Transport trait | Standard Rust async traits | Ours ✅ |
| **Performance** | Extra trait indirection | Direct trait calls | Ours ✅ (marginal) |

## When Each Approach is Better

### RMCP Approach Better When:
1. **Maximum Flexibility Needed**
   - Supporting multiple transport paradigms
   - Need Worker-based transports
   - Want automatic conversions from various types

2. **Role-Based Type Safety**
   - Separating Client vs Server at type level
   - Different message types for each role
   - Protocol version negotiation per role

3. **Custom Transport Semantics**
   - Need close() method for cleanup
   - Want transport-specific methods
   - Complex lifecycle management

### Our Approach Better When:
1. **Simplicity is Priority**
   - Want to use standard traits directly
   - Minimize abstraction layers
   - Quick to understand and implement

2. **Ecosystem Integration**
   - Want to use futures combinators directly
   - Easy testing with channels
   - Standard async patterns

3. **Proxy Use Case**
   - Don't need role separation (proxy is both)
   - Want uniform message handling
   - Focus on message transformation

## Hybrid Approach (Best of Both)

We could adopt RMCP's insight while keeping our simplicity:

```rust
// 1. Primary interface is Sink + Stream
pub trait Transport: 
    Sink<JsonRpcMessage> + Stream<Item = Result<JsonRpcMessage>> {}

// 2. But also provide IntoTransport for flexibility
pub trait IntoTransport {
    type Transport: Sink<JsonRpcMessage> + Stream<Item = Result<JsonRpcMessage>>;
    fn into_transport(self) -> Self::Transport;
}

// 3. Automatic implementations
impl<R, W> IntoTransport for (R, W) 
where 
    R: AsyncRead, 
    W: AsyncWrite 
{
    type Transport = Framed<Join<R, W>, JsonLineCodec>;
    fn into_transport(self) -> Self::Transport {
        Framed::new(tokio::io::join(self.0, self.1), JsonLineCodec)
    }
}

// 4. Direct usage still works
let transport = Framed::new(join(stdin, stdout), JsonLineCodec);
let client = Client::new(transport);

// 5. Or convenience
let client = Client::new((stdin, stdout).into_transport());
```

## Implementation Insights from RMCP

### 1. Arc<Mutex> Pattern Validated
RMCP uses `Arc<Mutex>` for writers/sinks in BOTH AsyncRw and SinkStream transports. This confirms our analysis about concurrent sends.

### 2. Framed is Standard
RMCP's AsyncRwTransport uses `FramedRead` and `FramedWrite`, validating our Framed approach.

### 3. Multiple Codec Support
RMCP has a custom `JsonRpcMessageCodec` that handles their role-based typing. We'd have simpler `JsonLineCodec`.

### 4. Transport Variety
RMCP supports:
- stdio (AsyncRead/Write)
- subprocess (AsyncRead/Write)
- HTTP streamable (custom)
- SSE (custom)
- WebSocket (planned, would use Sink/Stream)

## Recommendation

### Go with Framed/Sink/Stream, but Learn from RMCP

1. **Primary abstraction**: Sink + Stream traits directly
   - Simpler, more idiomatic, better ecosystem integration
   - This is what RMCP's SinkStreamTransport does internally anyway

2. **Add IntoTransport convenience** (optional)
   - For AsyncRead/AsyncWrite → Framed conversion
   - Makes migration easier
   - Provides flexibility without complexity

3. **Use Arc<Mutex> for Sink**
   - RMCP validates this pattern
   - Necessary for concurrent sends

4. **Consider role-based typing later**
   - Not needed for initial implementation
   - Can add if we need client/server separation

## Why Our Approach is Better for MCP Library

1. **Simpler Mental Model**
   - MCP is fundamentally message-oriented
   - Sink/Stream matches perfectly
   - No custom trait to learn

2. **Better Testing**
   ```rust
   // Super easy test transport
   let (tx, rx) = futures::channel::mpsc::channel(10);
   let transport = tx.fanout(rx); // That's it!
   ```

3. **Ecosystem Power**
   ```rust
   // All these work out of the box
   transport.buffered(10)
   transport.timeout(Duration::from_secs(30))
   transport.throttle(Duration::from_millis(100))
   ```

4. **Cleaner Client/Server**
   ```rust
   // No custom trait methods
   client.transport.send(msg).await?;  // Just Sink::send
   while let Some(msg) = client.transport.next().await {  // Just Stream::next
       // ...
   }
   ```

## Migration Path from Current Design

1. **Drop StreamTransport<R: AsyncRead, W: AsyncWrite>**
   - Don't implement at byte level
   
2. **Create Framed-based transports**
   - StdioTransport uses Framed with JsonLineCodec
   - SubprocessTransport uses Framed with JsonLineCodec
   
3. **Implement Sink + Stream for each**
   - Or just use Framed directly (it implements both!)
   
4. **Update Client/Server**
   - Accept anything that is Sink + Stream
   
5. **Optional: Add IntoTransport later**
   - For convenience conversions
   - Not required for MVP

## Conclusion

RMCP's architecture is more flexible but more complex. Our Framed/Sink/Stream approach is simpler and more idiomatic while still being powerful enough for all MCP use cases. 

Since RMCP itself supports Sink/Stream via SinkStreamTransport, we're not diverging from their approach - we're just choosing to make it the primary abstraction rather than hiding it behind a custom trait.

**Recommendation**: Proceed with Framed/Sink/Stream architecture. It's simpler, more testable, and leverages the Rust async ecosystem better.

---

*Document created: 2025-08-24*  
*Key finding: RMCP supports our approach via SinkStreamTransport*  
*Decision: Use Sink/Stream as primary abstraction for simplicity*