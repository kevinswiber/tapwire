# SSE Implementation Comparison

## Reference Implementation Analysis

### TypeScript SDK SSE Client (typescript-sdk/src/client/sse.ts)

#### Key Design Patterns
1. **Separation of Concerns**
   - SSE connection for receiving messages
   - Separate POST requests for sending messages
   - No attempt to parse/buffer the SSE stream

2. **EventSource Usage**
   - Uses standard EventSource API for SSE
   - Handles `endpoint` event to get POST URL
   - Processes messages as they arrive

3. **Connection Flow**
   ```
   1. Open SSE connection to server URL
   2. Receive "endpoint" event with POST URL
   3. Send messages via POST to endpoint
   4. Receive responses via SSE stream
   ```

4. **Error Handling**
   - Custom SseError class with code and event
   - Auth retry on 401 errors
   - Graceful close with abort controller

### TypeScript SDK SSE Server (typescript-sdk/src/server/sse.ts)

#### Key Patterns
1. **Immediate Response**
   - Returns 200 with `text/event-stream` immediately
   - Sends `endpoint` event with session ID
   - Keeps connection open for streaming

2. **Message Handling**
   - POST requests handled separately
   - Messages sent via SSE write
   - No buffering of SSE stream

## Our Implementation Issues

### Problem 1: Attempting to Buffer SSE Stream
**Location**: `process_via_http()` lines 2312-2454

```rust
// BAD: Tries to consume entire response
if content_type.contains("text/event-stream") {
    // Can't parse SSE here - it's infinite!
    return Err(ReverseProxyError::SseStreamingRequired);
}
```

**Issue**: Function signature expects `ProtocolMessage` return, incompatible with streaming.

### Problem 2: Duplicate Requests
**Location**: Lines 1289-1311

```rust
Err(ReverseProxyError::SseStreamingRequired) => {
    // Makes ANOTHER request - wasteful!
    let response = client.post(url)...send().await?;
    return proxy_sse_response(response, ...);
}
```

**Issue**: Throws away first response, makes identical request again.

### Problem 3: Mixed Responsibilities
- `handle_mcp_request()` tries to handle both JSON and SSE
- No clear separation between streaming and buffered responses
- Interceptors assume complete messages, not streams

## Correct SSE Patterns from References

### 1. Response-Based Detection (CORRECTED)
```typescript
// Client sets Accept header to indicate preference
headers.set("Accept", "text/event-stream");
// But server determines actual response type
```

Our code's current approach is actually correct in concept:
```rust
// Check upstream response Content-Type AFTER making request
let content_type = response.headers()
    .get(reqwest::header::CONTENT_TYPE)
    .and_then(|v| v.to_str().ok())
    .unwrap_or("");

if content_type.contains("text/event-stream") {
    // Handle SSE streaming
}
```

The problem isn't WHERE we check, but WHAT we do after detecting SSE.

### 2. Streaming First Design
```typescript
// EventSource handles streaming automatically
this._eventSource.onmessage = (event) => {
    // Process each event as it arrives
    this.onmessage?.(JSON.parse(event.data));
};
```

Our code needs:
```rust
// Stream processor, not buffer consumer
async fn stream_sse_events(
    response: Response,
    tx: UnboundedSender<Event>
) {
    let mut stream = response.bytes_stream();
    // Process chunks as they arrive
}
```

### 3. Separate Send/Receive Channels
Reference pattern:
- SSE for server→client messages
- POST for client→server messages
- Never mix the two

Our code mixes:
- POST can return either JSON or SSE
- No clear separation of concerns

## Existing SSE Infrastructure We Can Reuse

From `src/transport/sse/`:

1. **SseParser** - Parses SSE events from byte streams
2. **SseStream** - Buffered SSE stream reader
3. **SseEvent** - Event structure with proper fields
4. **SseConnection** - Connection management
5. **ReconnectingStream** - Automatic reconnection logic

### Why We're Not Using Them
- Current architecture assumes request/response pattern
- SSE modules designed for client connections, not proxying
- Interceptor chain incompatible with streaming

## Content-Type Parsing Considerations

### Current Issue
```rust
// Current naive check - doesn't handle parameters
if content_type.contains("text/event-stream") {
    // This would match "text/event-stream; charset=utf-8"
    // But also "application/json; text/event-stream-not-really"
}
```

### Proper MIME Parsing
```rust
use mime::Mime;

// Parse Content-Type header properly
let content_type: Option<Mime> = response.headers()
    .get(reqwest::header::CONTENT_TYPE)
    .and_then(|v| v.to_str().ok())
    .and_then(|s| s.parse().ok());

// Check MIME type correctly
match content_type {
    Some(mime) if mime.type_() == mime::TEXT && 
                  mime.subtype() == "event-stream" => {
        // Handle SSE
    }
    Some(mime) if mime.type_() == mime::APPLICATION && 
                  mime.subtype() == mime::JSON => {
        // Handle JSON
    }
    _ => {
        // Handle other or missing Content-Type
    }
}
```

**Note**: The `mime` crate is already available through our dependencies (via reqwest/hyper).

## Recommended Solution

### Option 1: Return Response Object (Recommended)
```rust
async fn process_via_http(...) -> Result<reqwest::Response> {
    // Return the response object itself, not parsed content
    let response = client.post(url).send().await?;
    
    // Just validate and return - let caller decide how to handle
    validate_response_status(&response)?;
    Ok(response)
}

async fn handle_mcp_request(...) {
    let response = process_via_http(...).await?;
    
    // Parse Content-Type header properly
    let content_type = response.headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<mime::Mime>().ok());
    
    match content_type {
        Some(mime) if mime.type_() == "text" && mime.subtype() == "event-stream" => {
            // Stream SSE response directly
            proxy_sse_response(response, ...).await
        }
        _ => {
            // Parse JSON and process normally
            let json = response.json().await?;
            process_json_response(json, ...).await
        }
    }
}
```

### Option 2: Response Enum
```rust
enum ProxyResponse {
    Json(ProtocolMessage),
    Stream(Response),
}

async fn process_via_http(...) -> Result<ProxyResponse> {
    let response = client.post(url).send().await?;
    
    if is_sse_response(&response) {
        Ok(ProxyResponse::Stream(response))
    } else {
        let msg = parse_json_response(response).await?;
        Ok(ProxyResponse::Json(msg))
    }
}
```

### Option 3: Transport Trait
```rust
trait UpstreamTransport {
    async fn send(&self, msg: ProtocolMessage) -> Result<TransportResponse>;
}

enum TransportResponse {
    Message(ProtocolMessage),
    Stream(Box<dyn Stream<Item = Result<SseEvent>>>),
}
```

## Key Takeaways

1. **Never Buffer SSE Streams** - They're infinite by design
2. **Detect Early** - Check Accept header before making request
3. **Separate Paths** - JSON and SSE need different processing
4. **Stream Processing** - Process events as they arrive
5. **Reuse Infrastructure** - Leverage existing SSE modules
6. **Protocol Compliance** - Follow MCP patterns from reference

## Implementation Priority

1. **Fix Immediate Bug** - Stop duplicate requests
2. **Separate Handlers** - Split JSON and SSE paths
3. **Integrate SSE Modules** - Use existing parser/stream
4. **Update Interceptors** - Support streaming events
5. **Add Tests** - Validate against reference behavior