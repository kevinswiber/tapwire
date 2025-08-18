# Session Store Integration for Reverse Proxy

## Problem Statement

The reverse proxy needs to accept a `SessionStore` implementation for:
1. Persistent session tracking across restarts
2. SSE event ID tracking for reconnection resilience
3. Distributed deployment scenarios (e.g., Redis backend)

Currently, the `SessionManager` initializes its persistence worker in `with_config()`, which creates tight coupling and prevents runtime store injection.

## Current Architecture Issues

### 1. SessionManager Initialization
```rust
// In SessionManager::with_config()
let (persistence_tx, persistence_handle) = if config.max_pending_per_session > 0 {
    // Creates persistence worker immediately
    // Tightly coupled to store provided at construction
}
```

### 2. ReverseProxyServer Structure
- Currently creates its own `SessionManager` with in-memory store
- No way to inject external `SessionStore` implementation
- Can't share stores across proxy instances

### 3. Main API Limitations
- `shadowcat` library doesn't expose store configuration
- Binary hardcodes in-memory implementation
- No way for library users to provide custom stores

## Proposed Solution

### 1. Deferred Persistence Initialization

Make `SessionManager` initialization lazy:

```rust
impl SessionManager {
    /// Create without starting persistence worker
    pub fn with_config(store: Arc<dyn SessionStore>, config: SessionConfig) -> Self {
        // Don't start persistence worker here
        Self {
            store,
            config,
            persistence_tx: Arc::new(RwLock::new(None)),
            persistence_handle: Arc::new(RwLock::new(None)),
            // ...
        }
    }
    
    /// Start persistence worker (idempotent)
    pub async fn ensure_persistence_started(&self) {
        let mut tx = self.persistence_tx.write().await;
        if tx.is_none() && self.config.max_pending_per_session > 0 {
            // Start persistence worker
            let (new_tx, rx) = mpsc::channel(1000);
            let worker = PersistenceWorker::new(rx, self.store.clone(), ...);
            let handle = tokio::spawn(async move { worker.run().await });
            
            *tx = Some(new_tx);
            *self.persistence_handle.write().await = Some(handle);
        }
    }
    
    /// Call this in methods that need persistence
    async fn get_persistence_tx(&self) -> Option<mpsc::Sender<PersistenceCommand>> {
        self.ensure_persistence_started().await;
        self.persistence_tx.read().await.clone()
    }
}
```

### 2. ReverseProxyServer Builder Pattern

```rust
pub struct ReverseProxyServerBuilder {
    bind_addr: SocketAddr,
    upstream_url: String,
    session_store: Option<Arc<dyn SessionStore>>,
    // ... other options
}

impl ReverseProxyServerBuilder {
    pub fn with_session_store(mut self, store: Arc<dyn SessionStore>) -> Self {
        self.session_store = Some(store);
        self
    }
    
    pub async fn build(self) -> Result<ReverseProxyServer> {
        let store = self.session_store
            .unwrap_or_else(|| Arc::new(InMemoryStore::new()));
        
        let session_manager = Arc::new(
            SessionManager::with_config(store, SessionConfig::default())
        );
        
        // Start persistence if needed
        session_manager.ensure_persistence_started().await;
        
        // ... rest of initialization
    }
}
```

### 3. Main Library API

```rust
// In lib.rs
pub use session::{SessionStore, InMemoryStore};
pub use proxy::reverse::{ReverseProxyServer, ReverseProxyServerBuilder};

// Allow library users to provide stores
pub async fn run_reverse_proxy(
    bind: SocketAddr,
    upstream: String,
    store: Option<Arc<dyn SessionStore>>,
) -> Result<()> {
    let server = ReverseProxyServerBuilder::new(bind, upstream)
        .with_session_store(store.unwrap_or_else(|| Arc::new(InMemoryStore::new())))
        .build()
        .await?;
    
    server.run().await
}
```

### 4. Binary Integration

```rust
// In main.rs for reverse proxy command
let store = match args.store_type {
    Some(StoreType::Redis) => {
        Arc::new(RedisStore::new(&args.redis_url).await?) as Arc<dyn SessionStore>
    }
    _ => Arc::new(InMemoryStore::new()),
};

let server = ReverseProxyServerBuilder::new(bind_addr, upstream_url)
    .with_session_store(store)
    .build()
    .await?;
```

## Implementation Steps

1. **Refactor SessionManager** (30 min)
   - Make persistence initialization lazy
   - Add `ensure_persistence_started()` method
   - Update methods to call ensure_persistence as needed

2. **Add ReverseProxyServerBuilder** (20 min)
   - Create builder with `with_session_store()` method
   - Default to InMemoryStore if none provided

3. **Update Library API** (15 min)
   - Export necessary types
   - Add store parameter to public functions

4. **Update Binary** (15 min)
   - Add CLI flags for store selection
   - Wire up store creation

5. **Test Integration** (20 min)
   - Verify SSE reconnection with custom store
   - Test persistence across restarts

## Benefits

1. **Flexibility**: Library users can provide any `SessionStore` implementation
2. **Backwards Compatible**: Defaults to in-memory if no store provided
3. **Lazy Initialization**: Persistence only starts when needed
4. **Clean Separation**: Store choice separate from proxy logic
5. **Testability**: Can inject mock stores for testing

## Risks & Mitigations

- **Risk**: Race conditions in lazy initialization
  - **Mitigation**: Use RwLock and ensure idempotent initialization

- **Risk**: Breaking existing code
  - **Mitigation**: Keep existing APIs, add new builder pattern alongside

- **Risk**: Persistence worker lifecycle management
  - **Mitigation**: Add proper shutdown handling in Drop impl

## Testing Plan

1. Unit tests for lazy persistence initialization
2. Integration test with custom store implementation
3. E2E test of SSE reconnection with persistent store
4. Performance test to ensure no regression

## Next Steps

Before continuing with SSE integration:
1. Implement lazy persistence in SessionManager
2. Add builder pattern to ReverseProxyServer
3. Update library API to expose store configuration
4. Then proceed with SSE event tracking integration