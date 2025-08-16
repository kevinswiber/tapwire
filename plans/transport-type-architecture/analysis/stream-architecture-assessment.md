# StdioCore Stream Architecture Assessment

**Created**: 2025-08-16  
**Purpose**: Assess the feasibility and impact of converting StdioCore to use async streams

## Current Architecture

### StdioCore Implementation
The current `StdioRawIncoming` and `StdioRawOutgoing` in `src/transport/raw/stdio.rs` already use async patterns:

```rust
// Current approach - channel-based async
async fn send_bytes(&mut self, data: &[u8]) -> TransportResult<()>
async fn receive_bytes(&mut self) -> TransportResult<Vec<u8>>
```

**Current Flow**:
1. Background tasks read from stdin/stdout into channels
2. `receive_bytes()` pulls from channel (async but blocking on single message)
3. `send_bytes()` pushes to channel (async)
4. Uses `tokio::io::stdin()` and `tokio::io::stdout()` internally

## Stream-Based Alternative

### What Would Change

Converting to a Stream-based approach would mean:

```rust
// Stream-based approach
impl Stream for StdioRawIncoming {
    type Item = Result<Bytes, TransportError>;
    
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Poll underlying stdin for data
    }
}

// Usage would change to:
let mut transport = StdioRawIncoming::new();
while let Some(chunk) = transport.next().await {
    match chunk {
        Ok(bytes) => process(bytes),
        Err(e) => handle_error(e),
    }
}
```

## Architectural Impact Analysis

### 1. Raw Transport Layer
**Impact**: MODERATE
- Would need to implement `Stream` trait for `StdioRawIncoming`
- Already has `StreamingRawTransport` trait defined but not used for stdio
- Would align with existing SSE stream patterns

### 2. Directional Transport Layer
**Impact**: HIGH
- `IncomingTransport` and `OutgoingTransport` traits expect:
  - `receive_request() -> MessageEnvelope` (single complete message)
  - `send_response(MessageEnvelope)` (single complete message)
- Would need new methods or trait redesign for streaming

### 3. Protocol Layer
**Impact**: HIGH
- MCP protocol expects complete JSON-RPC messages
- Would need buffering layer to accumulate stream chunks into complete messages
- Line-delimited JSON makes this feasible but adds complexity

### 4. Proxy Layer
**Impact**: VERY HIGH
- Forward and reverse proxies expect request/response pattern
- Would need major refactoring to handle streaming at proxy level
- Message interceptors expect complete messages

### 5. Session Management
**Impact**: LOW
- Sessions track metadata, not transport mechanics
- Would work with either approach

## Existing Stream Support

Shadowcat already has streaming support in specific areas:

1. **SSE Transport**: Uses `futures::Stream` for Server-Sent Events
2. **StreamingRawTransport trait**: Defined but not implemented for stdio
3. **SseStream**: Example of `Stream` implementation in `transport/sse/buffer.rs`

## Pros and Cons

### Pros of Stream-Based Approach
1. **Consistency**: Aligns with SSE streaming patterns
2. **Composability**: Can use Stream combinators (`.map()`, `.filter()`, etc.)
3. **Backpressure**: Automatic flow control
4. **Memory Efficiency**: Process data as it arrives without buffering entire messages

### Cons of Stream-Based Approach
1. **Major Refactor**: Would touch many layers of the architecture
2. **Protocol Mismatch**: MCP expects complete messages, not chunks
3. **Complexity**: Need buffering layer to reassemble messages
4. **Breaking Change**: Would change public API significantly

## Current Implementation Assessment

The current implementation is actually quite good:
- Already async (not blocking the runtime)
- Uses channels for decoupling
- Efficient buffer pooling
- Clear message boundaries

The `receive_bytes()` method waits for a complete line (message), which aligns with MCP's line-delimited JSON format.

## Recommendation

### Short Term: Keep Current Architecture
The current implementation is appropriate because:
1. **MCP is message-oriented**, not stream-oriented
2. **Line-delimited JSON** naturally maps to the current approach
3. **Already async** - not blocking the runtime
4. **Minimal overhead** - channels are efficient

### Long Term: Consider Hybrid Approach
For future extensibility (WebSocket, binary protocols):
1. Keep current `RawTransport` trait for message-oriented transports
2. Use `StreamingRawTransport` for truly streaming protocols
3. Add adapter layer to convert streams to messages where needed

### If We Were to Implement Streams

If we decide to implement streams later, here's the approach:

```rust
// 1. Add stream method to RawTransport (optional, default impl)
pub trait RawTransport {
    // Existing methods...
    
    // New optional stream method
    fn as_stream(&mut self) -> Option<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>> {
        None // Default: no streaming
    }
}

// 2. Implement for StdioRawIncoming
impl StdioRawIncoming {
    fn as_stream(&mut self) -> Option<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>> {
        Some(Box::pin(StdioStream::new(self.stdin_rx.clone())))
    }
}

// 3. Add buffering layer in directional transports
impl IncomingTransport for StdioIncoming {
    async fn receive_request(&mut self) -> TransportResult<MessageEnvelope> {
        if let Some(stream) = self.raw.as_stream() {
            // Use LineDelimitedCodec to get complete messages
            let framed = FramedRead::new(stream, LinesCodec::new());
            // ... handle streaming
        } else {
            // Fall back to current approach
            self.raw.receive_bytes().await
        }
    }
}
```

## Conclusion

**Current assessment**: The existing architecture is well-suited for MCP's message-oriented protocol. Converting to streams would add complexity without clear benefits for the stdio transport.

**Recommendation**: Document this assessment and revisit only if:
1. We need to support truly streaming protocols (beyond SSE)
2. We encounter performance issues with the current approach
3. We want to unify all transports under a streaming model

The current implementation is efficient, async, and appropriate for the use case.