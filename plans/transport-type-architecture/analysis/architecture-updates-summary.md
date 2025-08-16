# Architecture Updates Summary

**Created**: 2025-08-16  
**Purpose**: Document refinements made to the architecture proposal based on feedback

## Key Changes Made

### 1. Module Naming Clarification

**Changed**: `transport/implementations/` → `transport/raw/`

**Rationale**: Better communicates the distinction between modules:
- **transport/raw/**: Low-level I/O primitives (no MCP knowledge)
- **transport/directional/**: High-level trait implementations (MCP-aware)

This makes it clear that `raw` contains shared primitives used by the directional implementations.

### 2. ResponseMode Enum Simplification

**Removed**:
- `Unknown` variant (ambiguous semantics)
- `Binary` variant (unnecessary specialization) 
- `Text` variant (unnecessary specialization)
- `WebSocket` variant (premature - MCP spec not ready)

**Changed to**:
- `Json` - Standard JSON-RPC responses
- `SseStream` - Server-Sent Events streaming
- `Passthrough` - Any other content type (stream without processing/buffering)

**Rationale**: Simpler, clearer semantics. `Passthrough` handles all unknown content types by streaming them directly without interceptors or buffering.

### 3. MIME Parsing for Content-Type

**Changed**: String contains checks → Proper MIME parsing using `mime` crate

```rust
// Before
if content_type.contains("application/json") { ... }

// After
match content_type.parse::<Mime>() {
    Ok(mime) => match (mime.type_(), mime.subtype()) {
        (mime::APPLICATION, mime::JSON) => Self::Json,
        // ...
    }
}
```

**Rationale**: More robust parsing, handles edge cases, already used in `upstream_response.rs`.

### 4. Stream Trait for Async Operations

**Changed**: Manual chunk methods → `futures::Stream` trait

```rust
// Before
async fn send_stream_start(&mut self) -> TransportResult<()>
async fn send_stream_chunk(&mut self, chunk: &[u8]) -> TransportResult<()>
async fn send_stream_end(&mut self) -> TransportResult<()>

// After
fn response_stream(&mut self) -> Option<Pin<Box<dyn Stream<Item = Result<Bytes>> + Send>>>
```

**Rationale**: 
- More idiomatic Rust
- Better async/await ergonomics
- Automatic backpressure handling
- Already used in `transport::sse::buffer::SseStream`

### 5. Session ID Mapping Consideration

**Added**: `upstream_session_id: Option<SessionId>` field to Session struct

**Rationale**: Prepare for reverse proxy session mapping (documented in separate plan). The reverse proxy needs dual session IDs:
- `id`: Proxy's session ID for client connection
- `upstream_session_id`: Upstream server's session ID

Forward proxy can leave this as `None`.

### 6. Future WebSocket Notes

**Added**: Comments indicating where WebSocket support will be added

```rust
pub fn is_streaming(&self) -> bool {
    matches!(self, Self::SseStream)
    // Note: Will include WebSocket when MCP spec supports it
}
```

**Rationale**: Provides hints for future implementation without premature abstraction.

## Impact on Implementation

These refinements:
1. **Clarify module responsibilities** - Easier to understand and maintain
2. **Simplify ResponseMode** - Less complexity, clearer semantics
3. **Improve robustness** - Proper MIME parsing prevents edge cases
4. **Better async ergonomics** - Stream trait is more idiomatic
5. **Prepare for session mapping** - Architecture accommodates future needs

## No Changes Needed For

- Overall three-phase migration plan remains the same
- Core architecture of separating ResponseMode from TransportType unchanged
- DirectionalTransport adoption strategy unchanged
- Timeline estimates remain valid

## Next Steps

1. Proceed with Phase B implementation using refined ResponseMode enum
2. Use `mime` crate for Content-Type parsing
3. Keep `upstream_session_id` as Option for now (implement in session mapping plan)
4. Consider Stream trait adoption in Phase C when refactoring transports

---

These refinements improve the architecture while maintaining the core design principles and migration strategy.