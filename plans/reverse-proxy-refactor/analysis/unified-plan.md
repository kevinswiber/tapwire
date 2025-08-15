# Unified Implementation Plan - Reverse Proxy Refactor

## Overview
This document is the single source of truth for implementing the reverse proxy refactor. It consolidates all analysis findings and provides clear, actionable steps.

## Critical Problems We're Solving
1. **SSE Streaming Bug**: Proxy makes duplicate HTTP requests for SSE streams
2. **No Storage Abstraction**: Direct coupling to InMemorySessionStore blocks distributed sessions
3. **Monolithic Code**: 3,482 lines in single file, admin handler alone is 876 lines

## Implementation Strategy

### Core Principle: Build the Right Foundation First
- No quick patches that create technical debt
- Design for distributed systems from the start
- Implement only what we need now, but design for the future
- Enable library consumers to provide custom session stores

### Key Design Decisions
1. **SessionManager references store** - Enables dependency injection
2. **Store injectable via API** - Library consumers can provide implementations
3. **No backwards compatibility needed** - Shadowcat is unreleased
4. **Eager MIME parsing** - Almost always needed for routing
5. **Backpressure for all streams** - Client controls upstream reading pace

### Phase Execution Order
```
Phase B: SessionStore Abstraction (4-5 hours)
         ↓
Phase C: Fix SSE Bug Properly (5-6 hours)
         ├─ C.0-C.2: Core SSE fix (4h)
         └─ C.5: Backpressure handling (1h)
         ↓
Phase D: Modularization (8 hours total)
         ├─ D1: Admin Extract (3h) - can start after B
         └─ D2: Core Modules (5h) - must wait for C
         ↓
Phase E: Integration & Testing (4 hours)
```

## Phase B: SessionStore Abstraction (4-5 hours)

### B.1: Create Trait Definition (1 hour)

**File**: `src/session/store.rs`

```rust
use async_trait::async_trait;
use crate::session::{Session, SessionId, SessionResult, MessageEnvelope};

#[async_trait]
pub trait SessionStore: Send + Sync {
    // Core session operations
    async fn create_session(&self, session: Session) -> SessionResult<()>;
    async fn get_session(&self, id: &SessionId) -> SessionResult<Session>;
    async fn update_session(&self, session: Session) -> SessionResult<()>;
    async fn delete_session(&self, id: &SessionId) -> SessionResult<()>;
    async fn count_sessions(&self) -> SessionResult<usize>;
    async fn list_sessions(&self) -> SessionResult<Vec<Session>>;
    
    // Frame operations
    async fn add_frame(&self, frame: MessageEnvelope) -> SessionResult<()>;
    async fn get_frames(&self, session_id: &SessionId) -> SessionResult<Vec<MessageEnvelope>>;
    async fn delete_frames(&self, session_id: &SessionId) -> SessionResult<()>;
    
    // SSE-specific operations (for Phase C)
    async fn store_last_event_id(&self, session_id: &SessionId, event_id: String) -> SessionResult<()>;
    async fn get_last_event_id(&self, session_id: &SessionId) -> SessionResult<Option<String>>;
    
    // Batch operations (for future Redis efficiency)
    async fn get_sessions_batch(&self, ids: &[SessionId]) -> SessionResult<Vec<Session>>;
    async fn update_sessions_batch(&self, sessions: Vec<Session>) -> SessionResult<()>;
}
```

### B.2: Refactor InMemoryStore (2 hours)

**File**: `src/session/memory.rs`

1. Move current `InMemorySessionStore` from `store.rs` to `memory.rs`
2. Implement `SessionStore` trait for `InMemorySessionStore`
3. Add SSE-specific fields:
   ```rust
   pub struct InMemorySessionStore {
       sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
       frames: Arc<RwLock<HashMap<SessionId, Vec<MessageEnvelope>>>>,
       // New for SSE support
       last_event_ids: Arc<RwLock<HashMap<SessionId, String>>>,
   }
   ```

### B.3: Update SessionManager (1-2 hours)

**File**: `src/session/manager.rs`

Change from:
```rust
pub struct SessionManager {
    store: Arc<InMemorySessionStore>,
    // ...
}
```

To:
```rust
pub struct SessionManager {
    store: Arc<dyn SessionStore>,  // Reference, not ownership
    // ...
}
```

Update builder to accept trait (enables library consumers to inject custom stores):
```rust
impl SessionManagerBuilder {
    /// Allows library consumers to provide their own SessionStore implementation
    pub fn with_store(mut self, store: Arc<dyn SessionStore>) -> Self {
        self.store = store;
        self
    }
}

// In the API layer, expose store injection:
impl Shadowcat {
    pub fn with_session_store(mut self, store: Arc<dyn SessionStore>) -> Self {
        self.session_manager = SessionManager::builder()
            .with_store(store)
            .build();
        self
    }
}
```

### B.4: Fix Compilation Issues (1 hour)

Update all 13 files that reference SessionManager or InMemorySessionStore:
- Use `Arc<dyn SessionStore>` instead of concrete type
- Update imports to use new module structure
- Ensure all tests pass

## Phase C: Fix SSE Bug Properly (4-6 hours)

### C.1: Create UpstreamResponse Wrapper (1 hour)

**File**: `src/proxy/reverse/upstream_response.rs`

```rust
use reqwest::Response;
use mime::Mime;

pub struct UpstreamResponse {
    /// The unconsumed HTTP response from upstream
    pub response: Response,
    /// Parsed Content-Type header
    pub content_type: Option<Mime>,
    /// Content-Length if provided
    pub content_length: Option<usize>,
    /// Whether Transfer-Encoding is chunked
    pub is_chunked: bool,
    /// MCP Session ID from response headers
    pub session_id: Option<String>,
}

impl UpstreamResponse {
    pub async fn from_response(response: Response) -> Self {
        let headers = response.headers();
        
        let content_type = headers
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<Mime>().ok());
            
        let content_length = headers
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<usize>().ok());
            
        let is_chunked = headers
            .get(reqwest::header::TRANSFER_ENCODING)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.contains("chunked"))
            .unwrap_or(false);
            
        let session_id = headers
            .get("Mcp-Session-Id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        
        Self {
            response,
            content_type,
            content_length,
            is_chunked,
            session_id,
        }
    }
    
    pub fn is_sse(&self) -> bool {
        self.content_type
            .as_ref()
            .map(|mime| mime.type_() == "text" && mime.subtype() == "event-stream")
            .unwrap_or(false)
    }
    
    pub fn is_json(&self) -> bool {
        self.content_type
            .as_ref()
            .map(|mime| mime.type_() == "application" && mime.subtype() == "json")
            .unwrap_or(false)
    }
}
```

### C.2: Modify process_via_http (2 hours)

**File**: `src/proxy/reverse.rs` (lines 2312-2454)

Change signature:
```rust
async fn process_via_http(
    message: ProtocolMessage,
    session: &Session,
    url: &str,
) -> Result<UpstreamResponse, ReverseProxyError> {
    // ... make request ...
    let response = client.post(url)
        .headers(headers)
        .body(body)
        .send()
        .await?;
    
    Ok(UpstreamResponse::from_response(response).await)
}
```

### C.3: Implement SSE Streaming (2 hours)

**File**: `src/proxy/reverse/sse_handler.rs`

```rust
use crate::transport::sse::{SseParser, SseStream};
use futures::StreamExt;

pub async fn stream_sse_with_interceptors(
    upstream: UpstreamResponse,
    interceptor_chain: Arc<InterceptorChain>,
    session_store: Arc<dyn SessionStore>,
    session_id: SessionId,
) -> Result<Response, ReverseProxyError> {
    // Get or store Last-Event-Id
    let last_event_id = session_store.get_last_event_id(&session_id).await?;
    
    // Create SSE parser
    let mut parser = SseParser::new();
    let mut stream = upstream.response.bytes_stream();
    
    // Create response channel
    let (tx, body) = Body::channel();
    
    // Spawn streaming task
    tokio::spawn(async move {
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let events = parser.feed(&chunk)?;
            
            for event in events {
                // Store Last-Event-Id for reconnection
                if let Some(ref id) = event.id {
                    session_store.store_last_event_id(&session_id, id.clone()).await?;
                }
                
                // Process through interceptors
                let modified = interceptor_chain.process_sse_event(event).await?;
                
                // Stream to client immediately
                let formatted = format_sse_event(&modified);
                tx.send_data(formatted.into()).await?;
            }
        }
        Ok::<(), ReverseProxyError>(())
    });
    
    // Return SSE response
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .body(body)?)
}
```

### C.4: Update Caller Logic (1 hour)

**File**: `src/proxy/reverse.rs` (lines 1289-1311)

Replace the duplicate request hack:
```rust
let upstream = process_via_http(intercepted_msg.clone(), &session, url).await?;

let response = if upstream.is_sse() {
    // Stream SSE without buffering
    stream_sse_with_interceptors(
        upstream,
        interceptor_chain,
        session_manager.get_store(),
        session.id.clone()
    ).await?
} else if upstream.is_json() {
    // Buffer and process JSON
    process_json_response(upstream, interceptor_chain).await?
} else {
    // Pass through other content types with backpressure
    stream_passthrough_with_backpressure(upstream).await?
};
```

Remove `ReverseProxyError::SseStreamingRequired` - no longer needed!

### C.5: Implement Pass-through with Backpressure (1 hour)

**File**: `src/proxy/reverse/stream_handler.rs`

```rust
/// Stream non-JSON/SSE content with proper backpressure
pub async fn stream_passthrough_with_backpressure(
    upstream: UpstreamResponse,
) -> Result<Response, ReverseProxyError> {
    // Get the response body as a stream
    let upstream_stream = upstream.response.bytes_stream();
    
    // Create a channel with bounded capacity for backpressure
    let (tx, body) = Body::channel();
    
    // Spawn task to stream with backpressure
    tokio::spawn(async move {
        let mut stream = upstream_stream;
        
        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(bytes) => {
                    // This will apply backpressure - if client is slow,
                    // this will pause reading from upstream
                    if let Err(e) = tx.send_data(bytes).await {
                        debug!("Client disconnected: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    debug!("Upstream read error: {}", e);
                    break;
                }
            }
        }
    });
    
    // Return response with appropriate headers
    Ok(Response::builder()
        .status(upstream.response.status())
        .body(body)?)
}
```

## Phase D: Modularization (8 hours)

### D.1: Admin Extraction (3 hours) - CAN BE PARALLEL

**Start after Phase B completes**

Create structure:
```
src/proxy/reverse/admin/
├── mod.rs          # Public interface
├── handlers.rs     # 876 lines of handlers
├── templates.rs    # HTML generation
└── api.rs          # REST endpoints
```

Move `handle_admin_request` and all related functions.

### D.2: Core Modularization (5 hours) - MUST WAIT FOR C

Create structure:
```
src/proxy/reverse/
├── mod.rs              # Public API
├── config.rs           # Configuration (300 lines)
├── metrics.rs          # Metrics (330 lines)
├── handlers/
│   ├── mod.rs         # Handler traits
│   ├── json.rs        # JSON processing
│   └── sse.rs         # SSE streaming (from C.3)
├── upstream.rs         # Upstream management
└── upstream_response.rs # From C.1
```

## Phase E: Integration & Testing (4 hours)

### E.1: Integration Tests (2 hours)

Create `tests/reverse_proxy_sse.rs`:
- Test SSE streaming without duplicate requests
- Test Last-Event-Id storage and retrieval
- Test session mapping
- Test with MCP Inspector

### E.2: Performance Validation (1 hour)

- Benchmark SSE streaming latency
- Verify < 5% p95 overhead
- Test memory usage with long-lived streams

### E.3: Documentation (1 hour)

- Update architecture docs
- Document new module structure
- Add migration guide for SessionStore

## Testing Strategy

### Unit Test Fixtures

**SSE Test Data** (`tests/fixtures/sse_stream.txt`):
```
data: {"jsonrpc":"2.0","method":"initialize","id":1}

event: notification
data: {"status":"connecting"}

id: msg-001
data: {"jsonrpc":"2.0","result":{"status":"ok"},"id":1}

: heartbeat

data: multiline
data: message
data: here

```

### Integration Test Approach

1. Use `mockito` for unit tests
2. Use actual MCP servers for integration tests
3. Create test trait implementations:
   ```rust
   #[cfg(test)]
   mod tests {
       struct MockSessionStore {
           sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
       }
       
       impl SessionStore for MockSessionStore {
           // ... implement trait
       }
   }
   ```

## Risk Mitigation

### Phase B Risks
- **Breaking existing code**: Run full test suite after each change
- **Performance regression**: Arc<dyn Trait> has small overhead, benchmark it

### Phase C Risks  
- **SSE parsing errors**: Use existing, tested SseParser
- **Memory leaks**: Ensure streams are properly dropped
- **Infinite streams**: Add timeout/size limits

### Phase D Risks
- **Circular dependencies**: Keep clean module boundaries
- **Lost functionality**: Move code incrementally with tests

## Success Criteria Checklist

### Phase B Complete When:
- [ ] SessionStore trait defined and documented
- [ ] InMemoryStore implements trait
- [ ] SessionManager uses trait not concrete type
- [ ] All existing tests pass
- [ ] Can swap implementations at runtime

### Phase C Complete When:
- [ ] No duplicate HTTP requests for SSE
- [ ] SSE streams without buffering
- [ ] Last-Event-Id tracked properly
- [ ] Tested with MCP Inspector
- [ ] ReverseProxyError::SseStreamingRequired removed

### Phase D Complete When:
- [ ] No file > 500 lines
- [ ] Admin in separate module
- [ ] Clean module structure
- [ ] All tests still pass

### Phase E Complete When:
- [ ] Integration tests pass
- [ ] Performance targets met
- [ ] Documentation complete
- [ ] Ready for Redis implementation

## Commands for Testing

```bash
# After Phase B
cargo test --lib session::

# After Phase C  
cargo test reverse::sse
cargo run -- reverse --bind 127.0.0.1:8080 --upstream http://localhost:3000

# After Phase D
cargo test --all

# Integration test with Inspector
cd ~/src/modelcontextprotocol/inspector
npm start
# Then test proxy with Inspector as upstream
```

## Future Work (Not Part of This Refactor)

1. **Redis Implementation** (8-10 hours)
   - Implement SessionStore trait for Redis
   - Add connection pooling
   - Add configuration

2. **Connection Pooling** (4-6 hours)
   - Pool upstream connections
   - Share connections across sessions

3. **Distributed Session Mapping** (6-8 hours)
   - Full proxy-to-upstream mapping
   - Support for failover
   - Session migration

## Conclusion

This plan provides a clear, sequential path to fix the reverse proxy issues while building a solid foundation for future distributed session support. The key insight is that we must build the right abstractions first (SessionStore) before fixing the SSE bug, ensuring we don't create technical debt that needs immediate refactoring.