# Transport Architecture Decision Summary

## Investigation Results

We investigated the official Rust MCP SDK (rmcp) to understand their transport patterns and found:

### 1. RMCP Uses a Unified Transport Trait
Similar to our current design, they have a single bidirectional Transport trait with send/receive methods.

### 2. They Include Subprocess Management
Contrary to initial thoughts about separation of concerns, RMCP includes `TokioChildProcess` for spawning and managing stdio servers. This validates keeping our `SubprocessTransport`.

### 3. Push/Pull Resolution
The push/pull question resolves elegantly:
- **Transport Layer**: Provides bidirectional message passing (send/receive)
- **Protocol Layer**: Handles MCP semantics (requests, responses, notifications)
- Both client and server actively send (push) and receive (pull) messages

### 4. Key Pattern: Concurrent Sends
RMCP wraps the write side in `Arc<Mutex<>>` to allow concurrent sends from multiple tasks. This is important for MCP where both request/response and notifications can happen simultaneously.

### 5. Flexibility Through Adapters
RMCP provides multiple ways to create transports:
- From `(AsyncRead, AsyncWrite)` streams
- From `(Sink, Stream)` pairs  
- From combined types
- From child processes

## Recommended Actions

### Keep What We Have
1. ✅ **Unified Transport trait** - Validated by RMCP
2. ✅ **SubprocessTransport** - RMCP includes it, so should we
3. ✅ **StdioTransport** - For servers reading their own stdin/stdout
4. ✅ **HttpTransport** - For HTTP/SSE communication

### Consider Adding
1. **Concurrent sends**: Wrap write side in `Arc<Mutex<>>`
2. **Stream adapter**: Accept `(impl AsyncRead, impl AsyncWrite)`
3. **Return Option from receive**: `None` indicates stream closed (better than Error)

### Don't Need
1. **Incoming/Outgoing split**: Not necessary for MCP library
2. **Complex push/pull abstraction**: Transport handles bidirectional messages, that's enough

## Final Architecture

```rust
// Keep it simple
trait Transport {
    async fn send(&mut self, message: Value) -> Result<()>;
    async fn receive(&mut self) -> Result<Option<Value>>;
    async fn close(&mut self) -> Result<()>;
}

// Provide these implementations
StdioTransport        // For servers
SubprocessTransport   // For clients spawning servers
HttpTransport         // For HTTP/SSE
StreamTransport<R,W>  // For custom AsyncRead/AsyncWrite

// Optional: convenience constructors
impl Transport for (Box<dyn AsyncRead>, Box<dyn AsyncWrite>) { ... }
```

## Why This Works

1. **Simple API**: Easy to understand and use
2. **Flexible**: Covers all MCP communication patterns
3. **Precedent**: Aligns with official SDK patterns
4. **Subprocess included**: Common use case handled
5. **Escape hatch**: StreamTransport for custom needs

## Next Steps

1. Keep SubprocessTransport (don't remove it)
2. Consider adding Arc<Mutex> for concurrent sends
3. Add StreamTransport for AsyncRead/AsyncWrite flexibility
4. Continue with Phase C.2 (batch support)

The investigation confirms our general approach was correct. The main refinement is understanding that subprocess management IS appropriate for an MCP library, as validated by the official SDK.