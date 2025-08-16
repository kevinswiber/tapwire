# Distributed Storage Considerations

**Created**: 2025-08-16  
**Purpose**: Document how transport architecture changes maintain compatibility with distributed session storage

## Overview

Shadowcat already has a well-designed `SessionStore` trait that supports distributed storage backends like Redis. This document ensures our transport architecture changes maintain compatibility with this abstraction.

## Existing SessionStore Abstraction

Located in `src/session/store.rs`:

```rust
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create_session(&self, session: Session) -> SessionResult<()>;
    async fn get_session(&self, id: &SessionId) -> SessionResult<Session>;
    async fn update_session(&self, session: Session) -> SessionResult<()>;
    async fn delete_session(&self, id: &SessionId) -> SessionResult<()>;
    async fn count_sessions(&self) -> SessionResult<usize>;
    async fn list_sessions(&self) -> SessionResult<Vec<Session>>;
    
    // SSE-specific operations
    async fn store_last_event_id(&self, session_id: &SessionId, event_id: String) -> SessionResult<()>;
    async fn get_last_event_id(&self, session_id: &SessionId) -> SessionResult<Option<String>>;
    
    // Batch operations for efficiency
    async fn get_sessions_batch(&self, ids: &[SessionId]) -> SessionResult<Vec<Session>>;
    async fn update_sessions_batch(&self, sessions: Vec<Session>) -> SessionResult<()>;
}
```

## Key Design Principles

### 1. Keep Session Serializable

The Session struct must implement `Serialize` and `Deserialize`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    // All fields must be serializable
    pub id: SessionId,
    pub response_mode: Option<ResponseMode>, // New field - must be serializable
    pub upstream_session_id: Option<SessionId>, // New field - also serializable
    // ...
}
```

### 2. Use Async Store Methods

All session operations must go through the async SessionStore trait:

```rust
// Good - uses async store
let session = store.get_session(&id).await?;

// Bad - direct field access
let session = sessions_map.get(&id); // Don't bypass store
```

### 3. Atomic Updates

Session updates should be atomic to prevent inconsistency:

```rust
// Good - atomic update
pub async fn set_response_mode(
    &self,
    session_id: &SessionId,
    mode: ResponseMode,
) -> Result<()> {
    let mut session = self.store.get_session(session_id).await?;
    session.set_response_mode(mode);
    self.store.update_session(session).await
}

// Bad - partial update
pub async fn set_response_mode(&self, id: &SessionId, mode: ResponseMode) {
    // Don't add methods that update individual fields
    self.store.update_field(id, "response_mode", mode).await
}
```

## Impact on Transport Architecture Changes

### ResponseMode Enum

Must be serializable for storage:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseMode {
    Json,
    SseStream,
    Passthrough,
}
```

### Session Updates in Proxy

When the proxy detects response mode:

```rust
// In proxy response handler
let response_mode = ResponseMode::from_content_type(content_type);

// Update session atomically
if let Some(mut session) = session_manager.get_session(&session_id).await? {
    session.set_response_mode(response_mode);
    session_manager.update_session(session).await?;
}
```

### Dual Session IDs

For reverse proxy session mapping:

```rust
// Both IDs must be serializable
pub struct Session {
    pub id: SessionId,                      // Proxy's session ID
    pub upstream_session_id: Option<SessionId>, // Upstream's session ID
}
```

## Future Redis Implementation

The Redis backend (documented in `plans/redis-session-storage/`) will:

1. **Implement SessionStore trait** - Drop-in replacement for InMemoryStore
2. **Use Redis data structures**:
   - Hash for session data
   - Sorted set for expiry
   - List for SSE events
3. **Support horizontal scaling** - Multiple proxies sharing sessions
4. **Handle failover** - Fallback to in-memory if Redis unavailable

## Testing Considerations

### Unit Tests
```rust
#[tokio::test]
async fn test_session_serialization() {
    let session = Session::new(SessionId::new(), TransportType::Stdio);
    
    // Verify serialization works
    let serialized = serde_json::to_string(&session).unwrap();
    let deserialized: Session = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(session.id, deserialized.id);
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_distributed_response_mode_update() {
    let store = create_test_store(); // Could be Redis or InMemory
    let session_id = SessionId::new();
    
    // Create session
    let session = Session::new(session_id.clone(), TransportType::Http);
    store.create_session(session).await.unwrap();
    
    // Update response mode
    let mut session = store.get_session(&session_id).await.unwrap();
    session.set_response_mode(ResponseMode::SseStream);
    store.update_session(session).await.unwrap();
    
    // Verify persistence
    let loaded = store.get_session(&session_id).await.unwrap();
    assert_eq!(loaded.response_mode, Some(ResponseMode::SseStream));
}
```

## Best Practices

1. **Always use SessionStore trait** - Don't bypass for direct access
2. **Keep Session lightweight** - Don't store large transient data
3. **Batch operations when possible** - Use batch methods for multiple updates
4. **Handle store failures gracefully** - Network stores can fail
5. **Test with multiple backends** - Ensure compatibility with both InMemory and Redis

## Migration Path

### Phase B (Current)
- Add ResponseMode to Session (serializable)
- Use existing SessionStore methods
- Test serialization

### Future Redis Phase
- Implement RedisStore
- Add configuration for store selection
- Deploy with feature flags
- Monitor performance

## Conclusion

The transport architecture changes are fully compatible with distributed session storage:
- ResponseMode enum is serializable
- Session updates use async SessionStore trait
- Atomic updates prevent inconsistency
- Design supports future Redis backend

No additional changes needed to maintain distributed storage compatibility.