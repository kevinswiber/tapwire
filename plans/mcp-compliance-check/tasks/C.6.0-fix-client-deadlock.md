# Task C.6.0: Fix Client Concurrency Deadlock

**Status**: 🔴 CRITICAL - Blocks all client usage  
**Duration**: 2 hours  
**Dependencies**: C.5.4 (Sink/Stream implementation)  

## Problem

The current `Client` implementation has a fundamental deadlock:
- `request()` waits for response on a oneshot channel
- Responses only arrive if someone calls `transport.next()`
- But `run()` is the only method that reads from transport
- And `run()` consumes self, preventing further `request()` calls

```rust
// Current broken usage:
let mut client = Client::new(transport, handler);
let response = client.request("ping", json!({})).await; // Blocks forever!
```

## Solution

Spawn a background task in `Client::new()` that continuously reads from the transport and routes messages to appropriate handlers.

## Implementation Steps

### 1. Update Client Structure
```rust
pub struct Client<T, H = DefaultClientHandler> {
    transport: Arc<Mutex<T>>,  // Shared for concurrent access
    handler: Arc<H>,
    pending_requests: Arc<Mutex<HashMap<JsonRpcId, oneshot::Sender<Result<Value>>>>>,
    next_id: Arc<Mutex<u64>>,
    version: ProtocolVersion,
    receiver_handle: Option<JoinHandle<()>>,  // Background task
    shutdown_tx: Option<oneshot::Sender<()>>,  // Shutdown signal
}
```

### 2. Spawn Background Receiver
```rust
impl<T, H> Client<T, H> {
    pub fn new(mut transport: T, handler: H) -> Self {
        let transport = Arc::new(Mutex::new(transport));
        let handler = Arc::new(handler);
        let pending = Arc::new(Mutex::new(HashMap::new()));
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        
        // Spawn background receiver
        let receiver_handle = {
            let transport = transport.clone();
            let pending = pending.clone();
            let handler = handler.clone();
            
            tokio::spawn(async move {
                tokio::select! {
                    _ = Self::receive_loop(transport, pending, handler) => {},
                    _ = shutdown_rx => {},
                }
            })
        };
        
        Client {
            transport,
            handler,
            pending_requests: pending,
            next_id: Arc::new(Mutex::new(1)),
            version: ProtocolVersion::V2025_06_18,
            receiver_handle: Some(receiver_handle),
            shutdown_tx: Some(shutdown_tx),
        }
    }
    
    async fn receive_loop(
        transport: Arc<Mutex<T>>,
        pending: Arc<Mutex<HashMap<JsonRpcId, oneshot::Sender<Result<Value>>>>>,
        handler: Arc<H>,
    ) {
        loop {
            let msg = {
                let mut transport = transport.lock().await;
                transport.next().await
            };
            
            match msg {
                Some(Ok(value)) => {
                    Self::route_message(value, &pending, &handler).await;
                }
                Some(Err(e)) => {
                    eprintln!("Transport error: {:?}", e);
                    break;
                }
                None => break,  // Stream ended
            }
        }
    }
}
```

### 3. Update request() Method
```rust
pub async fn request(&self, method: &str, params: Value) -> Result<Value, Error> {
    // Generate ID
    let id = {
        let mut next_id = self.next_id.lock().await;
        let id = JsonRpcId::Number(*next_id as i64);
        *next_id += 1;
        id
    };
    
    // Create response channel
    let (tx, rx) = oneshot::channel();
    
    // Store pending request
    {
        let mut pending = self.pending_requests.lock().await;
        pending.insert(id.clone(), tx);
    }
    
    // Send request
    let request = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });
    
    {
        let mut transport = self.transport.lock().await;
        transport.send(request).await
            .map_err(|e| Error::Transport(e.to_string()))?;
    }
    
    // Wait for response (background task will send it)
    rx.await.map_err(|_| Error::ResponseTimeout)?
}
```

### 4. Add Shutdown Method
```rust
impl<T, H> Drop for Client<T, H> {
    fn drop(&mut self) {
        // Signal shutdown
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        
        // Wait for receiver to finish
        if let Some(handle) = self.receiver_handle.take() {
            // Can't await in drop, so we detach
            handle.abort();
        }
    }
}
```

## Test Cases

### Test 1: Request Without Run
```rust
#[tokio::test]
async fn test_request_without_run() {
    let (client_transport, server_transport) = create_channel_pair();
    let client = Client::new(client_transport, DefaultClientHandler);
    
    // Spawn mock server
    tokio::spawn(async move {
        let mut server = Server::new(server_transport, DefaultServerHandler);
        server.serve().await.unwrap();
    });
    
    // Should NOT deadlock
    let response = client.request("ping", json!({})).await.unwrap();
    assert_eq!(response, json!({}));
}
```

### Test 2: Concurrent Requests
```rust
#[tokio::test]
async fn test_concurrent_requests() {
    let client = Arc::new(Client::new(transport, handler));
    
    let mut handles = vec![];
    for i in 0..10 {
        let client = client.clone();
        handles.push(tokio::spawn(async move {
            client.request("test", json!({"id": i})).await
        }));
    }
    
    // All should complete without deadlock
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}
```

## Success Criteria

- [ ] Client can make requests without calling `run()`
- [ ] Multiple concurrent requests work
- [ ] Background task properly routes responses
- [ ] Clean shutdown without leaking tasks
- [ ] No deadlocks in any usage pattern

## Files to Modify

- `crates/mcp/src/client.rs` - Main implementation
- `crates/mcp/tests/client_integration.rs` - Add tests

## References

- GPT-5 findings: Lines 64-67
- RMCP pattern: `rmcp::transports::stdio` uses similar Arc<Mutex> pattern