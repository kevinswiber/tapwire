# Next Session: Implement Framed/Sink/Stream Transport Architecture

## Session Goal
Implement Phase C.5.4 - the finalized Framed/Sink/Stream transport architecture for the MCP library.

## 🚨 IMPORTANT: Working in Git Worktree
**Work Directory**: `/Users/kevin/src/tapwire/shadowcat-mcp-compliance`
- This is a git worktree on branch `feat/mcpspec`
- Main shadowcat remains untouched
- All work happens in the worktree
- Commit to `feat/mcpspec` branch

## Current Status (2025-08-24)

### What's Complete
- **Phase B**: Core MCP library extraction ✅
- **Phase C.0-C.1**: HTTP/SSE transport and interceptors ✅
- **Phase C.5.0-C.5.3**: Transport architecture investigation ✅
- **Architecture Decision**: Framed/Sink/Stream at message level ✅

### Key Architecture Decisions
1. **Message-level unification** using `Sink<JsonRpcMessage> + Stream`
2. **Framed with JsonLineCodec** for line-delimited protocols only
3. **HTTP adaptive transport** handles JSON/SSE/WebSocket based on server response
4. **Standard traits** instead of custom Transport trait

## Primary Task: C.5.4 - Implement Transport Architecture (3 hours)

### Quick Reference
- **Architecture**: `/plans/mcp-compliance-check/CURRENT-ARCHITECTURE.md` (START HERE)
- **Task Details**: `/plans/mcp-compliance-check/tasks/C.5.4-implement-framed-sink-stream.md`
- **HTTP Design**: `/plans/mcp-compliance-check/analysis/http-transport-unified-architecture.md`

### Implementation Steps

#### 1. JsonLineCodec (30 min)
Create `crates/mcp/src/transport/codec.rs`:
```rust
use tokio_util::codec::{Decoder, Encoder};
use bytes::BytesMut;

pub struct JsonLineCodec;

impl Decoder for JsonLineCodec {
    type Item = JsonRpcMessage;
    type Error = TransportError;
    
    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Find newline, split, parse JSON
    }
}

impl Encoder<JsonRpcMessage> for JsonLineCodec {
    type Error = TransportError;
    
    fn encode(&mut self, msg: JsonRpcMessage, buf: &mut BytesMut) -> Result<(), Self::Error> {
        // Serialize to JSON, append newline
    }
}
```

#### 2. StdioTransport (30 min)
Update `crates/mcp/src/transport/stdio.rs`:
```rust
use tokio_util::codec::Framed;

// Simple type alias approach
pub type StdioTransport = Framed<StdioStream, JsonLineCodec>;

pub fn stdio() -> StdioTransport {
    Framed::new(StdioStream::default(), JsonLineCodec::new())
}
```

#### 3. SubprocessTransport (45 min)
Update `crates/mcp/src/transport/subprocess.rs`:
```rust
pub struct SubprocessTransport {
    framed: Framed<ChildStdio, JsonLineCodec>,
    child: Child,  // For cleanup
}

// Delegate Sink + Stream to framed
impl Sink<JsonRpcMessage> for SubprocessTransport { /* delegate */ }
impl Stream for SubprocessTransport { /* delegate */ }
```

#### 4. HttpTransport (60 min)
Update `crates/mcp/src/transport/http.rs`:
```rust
pub struct HttpTransport {
    // Three modes in one transport
    single_responses: VecDeque<JsonRpcMessage>,
    sse_streams: HashMap<RequestId, SseStream>,
    ws_connection: Option<WebSocketStream>,
}

// Custom Sink + Stream implementation
impl Sink<JsonRpcMessage> for HttpTransport { /* ... */ }
impl Stream for HttpTransport { /* ... */ }
```

#### 5. Update Client/Server (45 min)
```rust
pub struct Client<T> 
where T: Sink<JsonRpcMessage> + Stream<Item = Result<JsonRpcMessage>> + Unpin
{
    transport: T,
}

impl<T> Client<T> {
    pub async fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        self.transport.send(request).await?;  // Direct Sink usage
        while let Some(msg) = self.transport.next().await {  // Direct Stream usage
            // Handle response
        }
    }
}
```

## Testing Strategy
```rust
// Simple test transport
let (tx, rx) = futures::channel::mpsc::channel(10);
let transport = ChannelTransport::new(tx, rx);
let mut client = Client::new(transport);
```

## Commands to Run
```bash
cd /Users/kevin/src/tapwire/shadowcat-mcp-compliance
cargo build --package mcp
cargo test --package mcp transport::
cargo test --package mcp client::
```

## Success Criteria
- [ ] All transports implement Sink + Stream traits
- [ ] JsonLineCodec handles line-delimited JSON
- [ ] StdioTransport works with Framed
- [ ] SubprocessTransport manages child process
- [ ] HttpTransport handles three modes transparently
- [ ] Client/Server use standard traits directly
- [ ] Tests pass with channel transport

## Key Reminders
- **Framed** = Line-delimited JSON ONLY (stdio, subprocess)
- **HTTP** = ONE transport, THREE modes (JSON, SSE, WebSocket)
- **Message-level** = Work with JsonRpcMessage, not bytes
- **Server chooses** = HTTP response mode based on operation

## After This Session
- **Phase C.2**: Add batch support (2h)
- **Phase C.3**: Test MCP crate independently (2h)
- **Phase D**: Build compliance framework

---

**Duration**: 3 hours  
**Focus**: Implement Framed/Sink/Stream architecture  
**Deliverables**: All transports using Sink + Stream traits

*Last Updated: 2025-08-24*  
*Ready for: Transport implementation*