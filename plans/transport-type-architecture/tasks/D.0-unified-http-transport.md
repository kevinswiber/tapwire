# Task D.0 (Final): Create Unified HTTP Transport with Hyper

**Duration**: 4 hours  
**Dependencies**: Phase C complete  
**Priority**: HIGH  
**Status**: 80% Complete (2025-08-17)

## Major Architecture Decisions

After comprehensive analysis (see `analysis/` directory):
1. **Single HTTP implementation** handles JSON, SSE, and passthrough based on Content-Type
2. **Use Hyper everywhere** - reqwest has SSE issues we must avoid
3. **True proxy transparency** - pass through unknown content types
4. **Consistent naming** - use `snake_case.rs` for all multi-word files

## Objective

1. Delete unused `transport::http_client.rs` 
2. Create unified hyper-based HTTP transport
3. Consolidate http.rs, sse.rs, and streamable_http.rs
4. Enable both proxies to use same implementation
5. Maintain true proxy transparency for unknown content

## Key Design Points

1. **Content Negotiation**: Response type from Content-Type header, not modes
2. **Streaming First**: Design for SSE, buffer only for JSON
3. **Proxy Transparency**: Forward unknown content unchanged
4. **Connection Pooling**: Hyper's built-in pooling properly configured

## Process

### Step 1: Clean Up and Rename (20 min)

```bash
# Delete unused global HTTP client
rm src/transport/http_client.rs

# Rename for consistency (if needed)
mv src/transport/raw/streamable_http.rs src/transport/raw/streamable_http.rs.backup

# Update imports in mod.rs files
```

### Step 2: Create Unified HTTP Transport (1 hour)

Create enhanced `src/transport/raw/http.rs`:

```rust
use hyper::{Client, Response, Body};
use hyper::header::CONTENT_TYPE;

/// Unified HTTP transport that handles JSON and SSE responses
pub struct HttpTransport {
    client: Client<HttpConnector, Body>,
    url: Url,
    pending_response: Option<Response<Incoming>>,
}

impl HttpTransport {
    pub async fn send_request(&mut self, data: Vec<u8>) -> Result<()> {
        let request = Request::post(&self.url)
            .header("Accept", "application/json, text/event-stream")
            .body(data)?;
        
        let response = self.client.request(request).await?;
        self.pending_response = Some(response);
        Ok(())
    }
    
    pub async fn receive_response(&mut self) -> Result<HttpResponse> {
        let response = self.pending_response.take()
            .ok_or(Error::NoResponse)?;
        
        // Auto-detect response type
        let content_type = response.headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream");
        
        match content_type {
            ct if ct.contains("application/json") => {
                // Buffer and parse JSON
                let body = response.into_body().collect().await?;
                Ok(HttpResponse::Json(body))
            }
            ct if ct.contains("text/event-stream") => {
                // Return SSE stream
                Ok(HttpResponse::SseStream(response.into_body()))
            }
            _ => {
                // Return raw body
                Ok(HttpResponse::Other(response.into_body()))
            }
        }
    }
}

pub enum HttpResponse {
    Json(Vec<u8>),
    SseStream(Body),
    Other(Body),
}
```

### Step 3: Create Unified Directional Transport (45 min)

Update `src/transport/directional/outgoing/http.rs`:

```rust
/// Unified HTTP outgoing transport
pub struct HttpOutgoing {
    transport: HttpTransport,
    protocol: Arc<dyn ProtocolHandler>,
    session_id: SessionId,
    sse_buffer: Option<SseEventBuffer>, // For streaming responses
}

impl OutgoingTransport for HttpOutgoing {
    async fn send_request(&mut self, envelope: MessageEnvelope) -> Result<()> {
        let bytes = self.protocol.serialize(&envelope.message)?;
        self.transport.send_request(bytes).await
    }
    
    async fn receive_response(&mut self) -> Result<MessageEnvelope> {
        match self.transport.receive_response().await? {
            HttpResponse::Json(data) => {
                // Parse JSON response
                let message = self.protocol.deserialize(&data)?;
                Ok(MessageEnvelope::new(message, ...))
            }
            HttpResponse::SseStream(stream) => {
                // Buffer SSE events and return them one by one
                if self.sse_buffer.is_none() {
                    self.sse_buffer = Some(SseEventBuffer::new(stream));
                }
                self.sse_buffer.as_mut().unwrap().next_event().await
            }
            HttpResponse::Other(_) => {
                Err(Error::UnsupportedContentType)
            }
        }
    }
}
```

### Step 4: Update Reverse Proxy (30 min)

Modify reverse proxy to use unified transport:

```rust
// Instead of separate HyperHttpClient
use crate::transport::directional::outgoing::HttpOutgoing;

// Works for both JSON and SSE responses!
let mut transport = HttpOutgoing::new(url)?;
```

### Step 5: Delete Redundant Code (25 min)

Remove no-longer-needed files:
```bash
# These are now redundant
rm src/transport/raw/sse.rs              # Absorbed into http.rs
rm src/transport/raw/streamable_http.rs  # No longer needed

# Remove StreamableHttpOutgoing (or make it an alias)
# Update factory.rs to only create HttpOutgoing
```

### Step 6: Update Tests and Documentation (30 min)

- Update tests to use unified transport
- Update documentation to explain content negotiation
- Ensure all existing tests pass

## Deliverables

1. **Deleted Files**:
   - `src/transport/http_client.rs` - Unused global client
   - `src/transport/raw/sse.rs` - Absorbed into http.rs
   - `src/transport/raw/streamable_http.rs` - No longer needed

2. **Enhanced Files**:
   - `src/transport/raw/http.rs` - Unified HTTP with SSE support
   - `src/transport/directional/outgoing/http.rs` - Single HTTP outgoing

3. **Updated Files**:
   - `src/transport/mod.rs` - Remove deleted modules
   - `src/transport/directional/factory.rs` - Single HTTP factory method
   - `src/proxy/reverse/legacy.rs` - Use unified transport

## Success Criteria

- [x] Single HTTP implementation handles both JSON and SSE
- [x] Content-Type auto-detection works correctly
- [ ] SSE streaming works through OutgoingTransport trait (needs SSE buffer)
- [ ] Reverse proxy uses unified transport (not yet migrated)
- [x] ~500 lines of redundant code removed
- [x] All tests pass (237 transport tests passing)
- [x] Consistent file naming (snake_case.rs)

## Benefits

1. **Massive Simplification**: 3 implementations → 1
2. **Natural HTTP Behavior**: Content negotiation, not modes
3. **Better Maintenance**: Single code path
4. **More Flexible**: Can handle any content type
5. **Cleaner Architecture**: Removes artificial distinctions

## Risks and Mitigations

**Risk**: Breaking existing code that expects separate transports
- **Mitigation**: Keep TransportType enum, provide compatibility aliases

**Risk**: SSE buffering complexity
- **Mitigation**: Use proven SseEventBuffer pattern from existing code

**Risk**: MCP spec compliance
- **Mitigation**: Transport types are about capabilities, not implementation

## Alternative If Time Constrained

If 3 hours isn't enough:
1. Just delete `http_client.rs` (5 min)
2. Create `HyperOutgoing` wrapper for existing HyperHttpClient (1 hour)
3. Leave consolidation for future refactor

But the full consolidation is strongly recommended - it will save much more time in the long run.

## Completion Status (2025-08-17)

### ✅ Completed:
1. **Deleted redundant files**:
   - `transport/http_client.rs` (unused global client)
   - `transport/raw/sse.rs` (basic SSE without advanced features)  
   - `transport/raw/streamable_http.rs` (redundant orchestration)

2. **Created unified transport**:
   - New `HyperHttpTransport` in `transport/raw/http.rs`
   - Handles JSON, SSE, and passthrough based on Content-Type
   - Uses hyper for true streaming control
   - Integrates with mature `transport::sse::parser::SseParser`

3. **Simplified naming**:
   - `HttpClientOutgoing` → `HttpOutgoing`
   - `HttpServerIncoming` → `HttpIncoming`
   - Removed `StreamableHttpOutgoing` (uses `HttpOutgoing` now)

4. **Preserved advanced SSE**:
   - Kept `transport/sse/` modules with reconnection logic
   - Session management, event deduplication, health monitoring intact

### 🔄 Remaining Work (20%):

1. **SSE Buffering in HttpOutgoing** (30 min):
   - Add `SseEventBuffer` to handle streaming through `OutgoingTransport` trait
   - Currently just uses basic `HttpRawClient`

2. **Reverse Proxy Integration** (30 min):
   - Migrate from `proxy/reverse/hyper_client.rs` to unified transport
   - Requires careful testing of existing functionality

3. **Documentation Updates** (30 min):
   - Document content negotiation approach
   - Update architecture diagrams
   - Add migration guide for future transports

## Notes

- Core unification complete and working
- This is a bigger change than originally planned but much better
- Eliminates entire categories of bugs (mode switching, etc.)
- Makes the codebase significantly simpler
- Aligns with how HTTP actually works