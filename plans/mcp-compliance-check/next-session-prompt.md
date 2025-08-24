# Next Session: Critical Fixes from GPT-5 Review

## Session Goal
Fix two critical bugs identified by GPT-5 that block further MCP library development.

## 🚨 IMPORTANT: Working in Git Worktree
**Work Directory**: `/Users/kevin/src/tapwire/shadowcat-mcp-compliance`
- This is a git worktree on branch `feat/mcpspec`
- Main shadowcat remains untouched
- All work happens in the worktree
- Commit to `feat/mcpspec` branch

## Current Status (2025-08-24)

### What's Complete
- **Phase C.5.4**: Framed/Sink/Stream implementation ✅
  - JsonLineCodec ✅
  - StdioTransport ✅
  - SubprocessTransport ✅
  - HttpTransport (basic) ✅
  - Client/Server using Sink+Stream ✅

### What's Broken (GPT-5 Findings)
1. **Client Deadlock**: `request()` blocks forever unless `run()` is called, but `run()` consumes self
2. **HTTP Transport**: Doesn't actually send HTTP requests, just shuffles queues

## Critical Bug #1: Client Concurrency Deadlock

**Problem**: 
```rust
// Current broken pattern:
let mut client = Client::new(transport, handler);
let response = client.request("method", params).await; // Blocks forever!
// Can't call run() because request() is waiting
```

**Fix Required**:
1. Spawn background receiver task in `Client::new()`
2. Route responses to pending request channels
3. Route notifications to handler
4. Add shutdown mechanism
5. Make `request()/notify()` work standalone

**Implementation**:
```rust
pub struct Client<T, H = DefaultClientHandler> {
    transport: Arc<Mutex<T>>,  // Shared for concurrent access
    handler: Arc<H>,
    pending: Arc<Mutex<HashMap<JsonRpcId, oneshot::Sender<Result<Value>>>>>,
    shutdown: Option<oneshot::Sender<()>>,
    receiver_handle: Option<JoinHandle<()>>,
}

impl<T, H> Client<T, H> {
    pub fn new(transport: T, handler: H) -> Self {
        let client = Client { /* ... */ };
        client.spawn_receiver();
        client
    }
    
    fn spawn_receiver(&mut self) {
        let transport = self.transport.clone();
        let pending = self.pending.clone();
        let handler = self.handler.clone();
        
        self.receiver_handle = Some(tokio::spawn(async move {
            while let Some(msg) = transport.lock().await.next().await {
                // Route to pending requests or handler
            }
        }));
    }
}
```

## Critical Bug #2: HTTP Transport Worker Pattern

**Problem**:
```rust
// Current broken implementation:
fn poll_flush(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    while let Some(msg) = self.pending_requests.pop_front() {
        self.single_responses.push_back(msg); // Just moves to another queue!
    }
    Poll::Ready(Ok(()))
}
```

**Fix Required**:
1. Create worker task with HTTP client
2. Request channel (bounded mpsc)
3. Response channel for Stream
4. Actual HTTP request sending
5. SSE stream management

**Implementation**:
```rust
pub struct HttpTransport {
    request_tx: mpsc::Sender<Value>,
    response_rx: mpsc::Receiver<io::Result<Value>>,
    worker_handle: JoinHandle<()>,
}

impl HttpTransport {
    pub fn new(url: Url) -> Self {
        let (request_tx, request_rx) = mpsc::channel(100);
        let (response_tx, response_rx) = mpsc::channel(100);
        
        let worker_handle = tokio::spawn(async move {
            let client = Client::new();
            while let Some(request) = request_rx.recv().await {
                // Actually send HTTP request
                let response = client.request(request).await;
                response_tx.send(response).await;
            }
        });
        
        HttpTransport { request_tx, response_rx, worker_handle }
    }
}
```

## Testing Strategy

### Client Tests
```rust
#[tokio::test]
async fn test_client_request_without_run() {
    let (transport, mock_server) = create_mock_transport();
    let mut client = Client::new(transport, DefaultHandler);
    
    // Should NOT deadlock
    let response = client.request("ping", json!({})).await.unwrap();
    assert_eq!(response, json!({"pong": true}));
}
```

### HTTP Transport Tests
```rust
#[tokio::test]
async fn test_http_actually_sends_requests() {
    let mock_server = MockServer::start().await;
    let transport = HttpTransport::new(mock_server.url());
    
    transport.send(json!({"method": "test"})).await.unwrap();
    
    // Verify HTTP request was actually made
    assert_eq!(mock_server.received_requests(), 1);
}
```

## Commands to Run

```bash
cd /Users/kevin/src/tapwire/shadowcat-mcp-compliance/crates/mcp

# Test compilation
cargo test --lib --no-run

# Run specific tests
cargo test client::tests::test_client
cargo test transport::http::tests::test_http

# Check for deadlocks with timeout
timeout 10 cargo test test_client_request_without_run
```

## Success Criteria

- [ ] Client can make requests without calling `run()`
- [ ] HTTP transport sends actual HTTP requests  
- [ ] No deadlocks in concurrent operations
- [ ] Tests demonstrate both fixes work
- [ ] Clean `cargo clippy` output

## Files to Modify

1. `crates/mcp/src/client.rs` - Add background receiver
2. `crates/mcp/src/transport/http/mod.rs` - Add worker pattern
3. `crates/mcp/tests/client_integration.rs` - Add deadlock tests
4. `crates/mcp/tests/http_integration.rs` - Add HTTP request tests

## References

- **GPT-5 findings**: `/plans/mcp-compliance-check/gpt-findings/findings.md`
- **Our analysis**: `/plans/mcp-compliance-check/analysis/gpt-findings-analysis.md`
- **WebSocket decision**: `/plans/mcp-compliance-check/analysis/websocket-separation-decision.md`

## After This Session

Once these critical bugs are fixed:
1. Create WebSocket transport (separate module)
2. Harden JsonLineCodec (CRLF, overlong lines)
3. Wire version negotiation
4. Add comprehensive tests

---

**Duration**: 3-4 hours  
**Priority**: CRITICAL (blocks all further work)  
**Focus**: Fix deadlock and HTTP worker  

*Last Updated: 2025-08-24*  
*Based on: GPT-5 architecture review*