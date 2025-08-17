# Reqwest vs Hyper Decision Analysis

**Created**: 2025-08-17  
**Purpose**: Decide whether to use reqwest or hyper for unified HTTP transport

## Executive Summary

**Decision: Use Hyper Everywhere**

After careful analysis, we should standardize on hyper for all HTTP transports in both forward and reverse proxies. The SSE streaming issues that forced the reverse proxy to use hyper will also affect the forward proxy if we use reqwest there.

## The SSE Problem with Reqwest

### What Happened
The reverse proxy originally used reqwest but had to switch to hyper because:
1. Reqwest buffers responses, making SSE streaming impossible
2. Reqwest's `Response::bytes_stream()` still chunks data unpredictably
3. No fine control over when data is available to the client
4. SSE requires immediate forwarding of each event as it arrives

### Why It Matters
If we use reqwest in the forward proxy's `HttpOutgoing`:
- **Same SSE issues will occur** when proxying to SSE-capable servers
- We'd need hyper for SSE anyway, defeating the purpose
- Mixed implementations increase complexity

## Reqwest vs Hyper Comparison

| Aspect | Reqwest | Hyper |
|--------|---------|-------|
| **Ease of Use** | ✅ High-level, simple | ❌ Lower-level, more code |
| **SSE Support** | ❌ Poor, buffering issues | ✅ Excellent, full control |
| **Streaming Control** | ❌ Limited | ✅ Complete |
| **Connection Pooling** | ✅ Built-in | ⚠️ Manual but doable |
| **HTTP/2** | ✅ Automatic | ✅ Available |
| **Error Handling** | ✅ Simplified | ❌ More verbose |
| **Body Ownership** | ❌ Consumes body | ✅ Can stream incrementally |
| **Proxy Transparency** | ❌ Opinionated | ✅ Full control |

## The Proxy Transparency Principle

A good proxy should be transparent - if it doesn't understand the content, it should forward it unchanged:

```rust
// Good proxy behavior with hyper
match content_type {
    "application/json" => // Parse and possibly intercept
    "text/event-stream" => // Stream with SSE handling
    _ => // Forward unchanged (passthrough)
}
```

Reqwest makes this harder because:
- It tries to be "helpful" (decompression, charset handling)
- Less control over raw bytes
- Can't easily do true passthrough

Hyper gives us:
- Raw access to response body
- Complete control over forwarding
- True proxy transparency

## MCP Spec Considerations

The MCP spec only defines:
- `application/json` for requests/responses
- `text/event-stream` for SSE

But a production proxy might see:
- `application/octet-stream` (binary data)
- `text/plain` (logs, debug info)
- `multipart/form-data` (file uploads)
- Custom content types

**With hyper**: We can handle MCP types specially and pass through everything else
**With reqwest**: Harder to avoid interfering with non-MCP content

## Code Complexity Analysis

### Reqwest Implementation (Simpler but Limited)
```rust
// Looks simple but breaks on SSE
let response = client.post(url)
    .json(&message)
    .send().await?;
    
let body = response.bytes().await?; // Buffers entire response!
```

### Hyper Implementation (More Code but Works)
```rust
// More verbose but handles all cases
let request = Request::post(url)
    .header(CONTENT_TYPE, "application/json")
    .body(Body::from(json_bytes))?;
    
let response = client.request(request).await?;
let body = response.into_body(); // Can stream!
```

The extra complexity is worth it for:
- Working SSE support
- True streaming
- Proxy transparency

## Migration Path

Since we're consolidating anyway, switching everything to hyper is actually easier:

1. **Delete reqwest-based code**:
   - `transport::http_client.rs` (unused anyway)
   - Current `HttpRawClient` using reqwest

2. **Create single hyper-based implementation**:
   - One HTTP transport that handles everything
   - Proven pattern from reverse proxy

3. **Consistency across codebase**:
   - Same HTTP library everywhere
   - Same patterns and error handling
   - Knowledge transfer between proxy types

## Performance Considerations

Hyper actually performs better for proxy use cases:
- Less memory usage (no buffering)
- Lower latency (immediate forwarding)
- Better for high-throughput scenarios

Reqwest's conveniences come with overhead we don't need in a proxy.

## Risk Assessment

### Risks of Using Reqwest
- ❌ **HIGH**: SSE will break in forward proxy
- ❌ **HIGH**: Can't do true passthrough proxying
- ❌ **MEDIUM**: Mixed implementations increase complexity

### Risks of Using Hyper
- ⚠️ **LOW**: More code to write initially
- ⚠️ **LOW**: Lower-level error handling
- ✅ **MITIGATED**: We already have working hyper code to reference

## Recommendation

**Go all-in on Hyper** for these reasons:

1. **SSE Support**: We know hyper works, reqwest doesn't
2. **Consistency**: Same library everywhere reduces cognitive load
3. **Proxy Transparency**: Better for unknown content types
4. **Future Proof**: If MCP adds WebSockets or other streaming protocols
5. **Performance**: Better for proxy workloads
6. **Proven**: Reverse proxy already validates the approach

The extra initial complexity of hyper is a small price for:
- Actually working SSE
- True proxy transparency  
- Consistent codebase
- Better performance

## Implementation Strategy

1. **Start with hyper** in the unified HTTP transport
2. **Port useful patterns** from reverse proxy's `HyperHttpClient`
3. **Add helper functions** to reduce boilerplate
4. **Create good abstractions** to hide hyper's complexity where possible

## Conclusion

While reqwest is easier to use, its limitations (especially with SSE) make it unsuitable for proxy implementations. Hyper's additional complexity is manageable and necessary for proper proxy behavior.

The reverse proxy's switch from reqwest to hyper already proved this decision correct. Let's learn from that experience and use hyper everywhere from the start.