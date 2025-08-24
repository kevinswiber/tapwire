# HTTP+SSE as Unified Transport Clarification

## Executive Summary

MCP's "Streamable HTTP" is **ONE transport** with two response modes, not two separate transports. We cannot use Framed with it because hyper doesn't expose raw AsyncRead/AsyncWrite. Instead, we implement a single HttpTransport that handles both response types internally.

## Understanding MCP's Streamable HTTP

### The Protocol Flow

```
Client                          Server
  |                               |
  |-------- HTTP POST ----------->|
  |     Accept: application/json, |
  |              text/event-stream |
  |     Body: {"jsonrpc":"2.0"..} |
  |                               |
  |<--------- Response ------------|
  |   Either:                      |
  |   - Content-Type: application/json
  |     Body: {"jsonrpc":"2.0"...} |
  |   OR:                          |
  |   - Content-Type: text/event-stream
  |     data: {"jsonrpc":"2.0"...} |
  |     data: {"jsonrpc":"2.0"...} |
  |     ...                        |
```

Key insights:
1. **Always sends via HTTP POST** with JSON body
2. **Server chooses response type** based on operation
3. **Client must handle both** response types transparently
4. This is **one cohesive protocol**, not two

## Why We Can't Use Framed with HTTP

### Framed Requirements
```rust
// Framed needs AsyncRead + AsyncWrite
Framed<T, C> where T: AsyncRead + AsyncWrite
```

### What Hyper Provides
```rust
// Hyper gives us Request/Response objects, not raw streams
let response = client.request(request).await?;
let body = response.into_body(); // This is NOT AsyncRead!
```

### The Impedance Mismatch
- Framed expects **continuous bidirectional byte streams**
- HTTP is **discrete request/response pairs**
- Hyper abstracts away the raw socket
- We work at HTTP semantic level, not TCP level

## Correct Architecture: Single HttpTransport

### Implementation Structure
```rust
pub struct HttpTransport {
    client: hyper::Client<HttpsConnector>,
    url: Url,
    pending_requests: VecDeque<PendingRequest>,
    response_streams: HashMap<RequestId, SseStream>,
}

impl HttpTransport {
    async fn send_request(&mut self, msg: JsonRpcMessage) -> Result<()> {
        let request = Request::post(&self.url)
            .header("Accept", "application/json, text/event-stream")
            .body(Body::from(serde_json::to_vec(&msg)?))?;
        
        let response = self.client.request(request).await?;
        
        // Handle based on Content-Type
        match response.headers().get("content-type") {
            Some(ct) if ct == "application/json" => {
                // Parse single response
                let body = hyper::body::to_bytes(response.into_body()).await?;
                let msg: JsonRpcMessage = serde_json::from_slice(&body)?;
                self.pending_responses.push_back(msg);
            }
            Some(ct) if ct == "text/event-stream" => {
                // Create SSE stream
                let stream = SseStream::new(response.into_body());
                self.response_streams.insert(request_id, stream);
            }
            _ => return Err(TransportError::InvalidContentType),
        }
        Ok(())
    }
}

impl Stream for HttpTransport {
    type Item = Result<JsonRpcMessage>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        // Check pending single responses first
        if let Some(msg) = self.pending_responses.pop_front() {
            return Poll::Ready(Some(Ok(msg)));
        }
        
        // Then poll active SSE streams
        for stream in self.response_streams.values_mut() {
            if let Poll::Ready(Some(msg)) = stream.poll_next_unpin(cx) {
                return Poll::Ready(Some(msg));
            }
        }
        
        Poll::Pending
    }
}
```

## Module Organization Recommendation

### Current (Slightly Misleading)
```
mcp::transports::{
    http::{
        self,           // Main transport
        streaming::{
            sse,        // SSE is part of HTTP, not separate
            websockets  // WebSocket is actually separate
        }
    },
    stdio,
    subprocess
}
```

### Recommended (Clearer)
```
mcp::transports::{
    stdio,              // Line-delimited JSON over stdio
    subprocess,         // Line-delimited JSON over child process
    http,               // Streamable HTTP (handles both JSON and SSE)
    websocket,          // Future: WebSocket protocol (separate from HTTP)
}

// Internal organization (not public API):
mcp::transports::http::{
    transport,          // Main HttpTransport implementation
    sse_parser,         // Internal: SSE event parsing
    response_handler,   // Internal: Response type detection
}
```

### Why This Organization?
1. **Transport = User-visible protocol choice**
   - User chooses "http" transport
   - Implementation handles JSON vs SSE transparently

2. **SSE is implementation detail**
   - Not a separate transport
   - Just one possible response format

3. **WebSocket IS separate**
   - Different protocol (even though upgrade starts with HTTP)
   - Different connection lifecycle
   - Should be top-level transport

## Codec Usage Clarification

### Where Codecs Apply
```rust
// ✅ Line-delimited protocols use codec
type StdioTransport = Framed<StdioStream, JsonLineCodec>;
type SubprocessTransport = Framed<ChildStdio, JsonLineCodec>;

// ❌ HTTP doesn't use codec - wrong abstraction level
struct HttpTransport { /* custom Sink+Stream */ }

// ❌ WebSocket doesn't use our codec - has its own framing
type WebSocketTransport = WebSocketStream; // Already Sink+Stream!
```

### JsonLineCodec Scope
- **ONLY for line-delimited JSON** protocols
- Encodes: Append newline to JSON
- Decodes: Split on newline, parse JSON
- Used by: stdio, subprocess, future TCP/Unix sockets

### HTTP Response Handling
- **No codec needed** - HTTP has its own framing
- JSON response: Parse entire body as single JSON
- SSE response: Parse `data:` lines from event stream
- Both handled internally by HttpTransport

## Summary of Corrections

### What I Got Wrong
1. ❌ Suggested all transports use Framed with JsonLineCodec
2. ❌ Treated HTTP and SSE as separate transports
3. ❌ Implied we could use Framed with HTTP

### What's Actually Correct
1. ✅ Only line-delimited protocols use Framed
2. ✅ HTTP+SSE is ONE transport with adaptive response handling
3. ✅ HTTP needs custom Sink+Stream implementation
4. ✅ All transports expose same `Sink<JsonRpcMessage> + Stream` interface

## Implementation Priority

1. **JsonLineCodec** - For stdio/subprocess only
2. **StdioTransport** - Using Framed<_, JsonLineCodec>
3. **SubprocessTransport** - Using Framed<_, JsonLineCodec>
4. **HttpTransport** - Custom Sink+Stream with internal SSE handling
5. **Future: WebSocketTransport** - Separate transport (not HTTP variant)

## Key Takeaway

**HTTP+SSE is one transport with two response modes**, not two transports. The transport implementation internally handles both response types based on Content-Type header. This is invisible to the user - they just see a stream of JsonRpcMessages.

---

*Document created: 2025-08-24*  
*Clarification: HTTP+SSE unified, Framed only for line-delimited protocols*