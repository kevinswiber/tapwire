# TransportContext Revelation: It's Message-Level, Not Session-Level!

## The Key Insight

TransportContext is a field in MessageContext, which is attached to each individual MessageEnvelope. This means TransportContext describes **how THIS PARTICULAR MESSAGE was delivered**, not what type of connection we have.

## The Three Message Delivery Mechanisms

From a message's perspective, there are exactly three ways it can be delivered in MCP:

### 1. Stdio Delivery
- Message arrives via stdin as newline-delimited JSON
- Message sent via stdout as newline-delimited JSON
- Context needed: process_id, command

### 2. HTTP JSON Delivery  
- Message arrives as HTTP response body with `Content-Type: application/json`
- Single message per HTTP response
- Context needed: method, path, headers, status_code

### 3. SSE Event Delivery
- Message arrives as the `data` field of an SSE event
- Part of an HTTP response with `Content-Type: text/event-stream`
- Multiple messages can arrive in one HTTP response
- Context needed: HTTP info (method, path, headers) PLUS SSE event metadata (id, type, retry)

## Why TransportContext::Sse is NOT a Code Smell

We were wrong! TransportContext::Sse is completely valid because:

1. **Different delivery mechanism**: An SSE event IS different from a JSON response body
2. **Different metadata**: SSE events have id, type, retry fields that matter for replay
3. **Different semantics**: SSE is streaming, JSON is request/response
4. **Recording needs**: The recorder needs to know this was an SSE event to replay faithfully

## The Naming Confusion

The confusion comes from the name "TransportContext" which sounds like it's about the transport protocol (stdio vs HTTP). But it's actually about the message delivery context. Better names might be:

- `DeliveryContext`
- `MessageDeliveryContext`  
- `MessageTransportContext`

But changing the name would be a big refactor for little benefit.

## Implications for Our Refactor

### We Should Keep TransportContext::Sse!

The three variants make perfect sense:
```rust
pub enum TransportContext {
    Stdio { 
        process_id: Option<u32>,
        command: Option<String>,
    },
    Http {
        method: String,
        path: String,
        headers: HashMap<String, String>,
        status_code: Option<u16>,
        remote_addr: Option<String>,
    },
    Sse {
        event_type: Option<String>,
        event_id: Option<String>,
        retry_ms: Option<u64>,
        headers: HashMap<String, String>,
    },
}
```

### Why Our Original Instinct Was Wrong

We thought:
- "SSE is just HTTP, so it should be TransportContext::Http"
- "The MCP spec says there are only 2 transports: stdio and HTTP"

But we confused:
- **Transport protocols** (stdio, HTTP) - connection level
- **Message delivery** (stdio, JSON, SSE) - message level

### The Recording Layer is Already Correct!

The recording layer correctly extracts SSE metadata from TransportContext::Sse. This isn't a hack - it's the right design!

## What About ResponseMode?

ResponseMode is still useful at the transport layer to track what kind of HTTP response we're dealing with:
- `ResponseMode::Json` - buffered JSON response
- `ResponseMode::SseStream` - streaming SSE response
- `ResponseMode::Passthrough` - unknown format

But this is transport-layer concern, not message-level. Each SSE event that comes through an SseStream becomes a message with TransportContext::Sse.

## The Real Problem We Need to Solve

The problem isn't that TransportContext::Sse exists. The problem is that some transports might not be setting it correctly. We need to ensure:

1. When a message arrives via SSE, it gets TransportContext::Sse with the event metadata
2. When a message arrives via JSON response, it gets TransportContext::Http
3. When a message arrives via stdio, it gets TransportContext::Stdio

## Conclusion

**We should NOT remove TransportContext::Sse!** It's the correct abstraction for message-level delivery context. The variants accurately represent the three ways an MCP message can be delivered.

The only thing we might want to do is ensure all SSE-delivering transports are properly setting TransportContext::Sse with the event metadata (id, type, retry) so the recording layer can capture it for faithful replay.

## New Simplified Plan

Instead of removing TransportContext::Sse, we should:

1. **Audit SSE transports** - Ensure they're setting TransportContext::Sse correctly
2. **Add any missing metadata** - Make sure event_id, event_type, retry_ms are captured
3. **Leave recording layer alone** - It's already doing the right thing
4. **Maybe improve naming** - Consider renaming to DeliveryContext in future (post-release)

This is even simpler than our "simplified" plan - we don't need to change TransportContext at all!