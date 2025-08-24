# Transport Architecture Investigation Summary

## ⚠️ UPDATED: See transport-architecture-final-v2.md for CURRENT architecture

## Investigation Completed: 2025-08-24 (Updated with v2 decision)

### What We Investigated
1. Official Rust MCP SDK (rmcp) transport patterns
2. Push vs Pull communication models
3. AsyncRead/AsyncWrite abstraction possibilities
4. Future WebSocket support requirements

## Key Findings

### 1. RMCP Validates Our Approach
- ✅ They use a unified bidirectional Transport trait (like ours)
- ✅ They include subprocess management (TokioChildProcess)
- ✅ They keep HTTP/SSE separate from stream transports
- 📝 They use Arc<Mutex> for concurrent sends (we should consider)

### 2. Transport Architecture Decision

#### Refactor to Two Core Types

**StreamTransport<R: AsyncRead, W: AsyncWrite>**
- Replaces both StdioTransport and SubprocessTransport
- Works with ANY AsyncRead + AsyncWrite pair
- Convenience constructors:
  - `StreamTransport::stdio()` - for servers
  - `StreamTransport::subprocess(cmd, args)` - for clients
  - `StreamTransport::new(r, w)` - for custom streams

**HttpTransport** (keep as is)
- HTTP request/response
- SSE support
- Cannot be unified with streams (different protocol semantics)

### 3. WebSocket Future-Proofing
- WebSocket will be a third transport type
- Three-phase protocol: HTTP → Upgrade → WebSocket (or fallback)
- Design documented in `websocket-transport-design.md`
- Current architecture supports adding it without breaking changes

## Architectural Insights

### Why Not Everything as AsyncRead/AsyncWrite?

| Protocol | Model | Why It Can/Can't Use Streams |
|----------|-------|------------------------------|
| Stdio | Continuous bidirectional | ✅ Natural fit |
| Subprocess | Continuous bidirectional | ✅ Natural fit |
| TCP/Unix | Continuous bidirectional | ✅ Natural fit |
| HTTP | Request/Response pairs | ❌ Needs headers, status codes |
| SSE | Server→Client events | ❌ Special event syntax |
| WebSocket | Bidirectional after upgrade | ❌ Needs upgrade negotiation |

### The Transport Trait Remains Simple
```rust
trait Transport {
    async fn send(&mut self, message: Value) -> Result<()>;
    async fn receive(&mut self) -> Result<Option<Value>>;
    async fn close(&mut self) -> Result<()>;
}
```

## Action Items

### Immediate (Phase C.5.4)
1. **Refactor StdioTransport + SubprocessTransport → StreamTransport<R,W>**
   - One implementation for all stream-based protocols
   - Type parameters for flexibility
   - Convenience constructors for common cases

2. **Add Concurrent Send Support**
   - Wrap writer in Arc<Mutex<>> like RMCP
   - Allows multiple tasks to send simultaneously

### Next (Continue Phase C)
3. **Phase C.2**: Add batch support for v2025-03-26
4. **Phase C.3**: Test MCP crate independently

### Future
5. **WebSocket Transport**: When spec is finalized
   - Already designed and documented
   - Will slot in cleanly as third transport type

## Validation Summary

Our original architecture was fundamentally correct:
- ✅ Unified Transport trait - validated by RMCP
- ✅ Include subprocess management - standard practice
- ✅ Keep HTTP separate - different protocol semantics
- 🔄 Refactor stdio/subprocess into StreamTransport - better abstraction
- 🔄 Add concurrent sends - important for MCP

## References
- `transport-patterns-analysis.md` - RMCP investigation details
- `transport-architecture-final.md` - Final architecture design
- `websocket-transport-design.md` - Future WebSocket support
- `transport-decision-summary.md` - Decision rationale

## Conclusion

The investigation confirmed our approach while identifying two improvements:
1. ~~Unify stdio/subprocess into generic StreamTransport<R,W>~~ **UPDATED**: Use Framed/Sink/Stream instead
2. Add Arc<Mutex> for concurrent sends ✅ Still valid

**UPDATE**: After further analysis comparing with RMCP's SinkStreamTransport, we decided to use **Framed/Sink/Stream architecture** for message-level unification instead of AsyncRead/AsyncWrite for byte-level. See [transport-architecture-final-v2.md](transport-architecture-final-v2.md) for the current design.

These changes will make the transport layer more flexible and powerful while maintaining simplicity.