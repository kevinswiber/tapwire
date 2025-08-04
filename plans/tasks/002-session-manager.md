# Task: Implement Session Management

**Status:** Not Started  
**Priority:** High  
**Estimated Time:** 1.5 days  
**Dependencies:** Error types, Transport types

---

## Objective

Implement SessionManager and related types to track MCP session lifecycle, store frames, and provide session querying capabilities. Start with in-memory storage but design for future SQLite backend.

---

## Design

### Core Types
```rust
pub struct Session {
    pub id: SessionId,
    pub client_session_id: Option<String>,
    pub server_session_id: Option<String>,
    pub transport_type: TransportType,
    pub protocol_version: String,
    pub created_at: Instant,
    pub ended_at: Option<Instant>,
    pub state: SessionState,
    pub metadata: SessionMetadata,
}

pub enum SessionState {
    Connecting,
    Initialized,
    Active,
    Closing,
    Closed,
    Failed(String),
}

pub struct Frame {
    pub id: FrameId,
    pub session_id: SessionId,
    pub timestamp: Instant,
    pub direction: Direction,
    pub edge: TransportEdge,
    pub message: TransportMessage,
    pub metadata: FrameMetadata,
}

pub struct SessionMetadata {
    pub client_info: Option<ClientInfo>,
    pub server_info: Option<ServerInfo>,
    pub capabilities: Option<Capabilities>,
    pub error_count: u32,
    pub frame_count: u32,
}
```

---

## Implementation Steps

### 1. Define Storage Trait
```rust
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create(&self, session: &Session) -> StorageResult<()>;
    async fn update(&self, session: &Session) -> StorageResult<()>;
    async fn get(&self, id: &SessionId) -> StorageResult<Option<Session>>;
    async fn list(&self, filter: SessionFilter) -> StorageResult<Vec<Session>>;
    async fn add_frame(&self, frame: &Frame) -> StorageResult<()>;
    async fn get_frames(&self, session_id: &SessionId) -> StorageResult<Vec<Frame>>;
    async fn delete_old(&self, before: Instant) -> StorageResult<u64>;
}
```

### 2. Implement In-Memory Store
```rust
pub struct MemoryStore {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    frames: Arc<RwLock<HashMap<SessionId, Vec<Frame>>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            frames: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SessionStore for MemoryStore {
    async fn create(&self, session: &Session) -> StorageResult<()> {
        let mut sessions = self.sessions.write().await;
        if sessions.contains_key(&session.id) {
            return Err(StorageError::AlreadyExists(session.id.to_string()));
        }
        sessions.insert(session.id.clone(), session.clone());
        
        let mut frames = self.frames.write().await;
        frames.insert(session.id.clone(), Vec::new());
        
        Ok(())
    }
    
    // ... implement other methods
}
```

### 3. Implement SessionManager
```rust
pub struct SessionManager {
    store: Arc<dyn SessionStore>,
    config: SessionConfig,
    active_sessions: Arc<RwLock<HashSet<SessionId>>>,
}

pub struct SessionConfig {
    pub max_sessions: usize,
    pub session_timeout: Duration,
    pub max_frames_per_session: usize,
    pub cleanup_interval: Duration,
}

impl SessionManager {
    pub fn new(store: Arc<dyn SessionStore>, config: SessionConfig) -> Self {
        let manager = Self {
            store,
            config,
            active_sessions: Arc::new(RwLock::new(HashSet::new())),
        };
        
        // Start cleanup task
        manager.start_cleanup_task();
        
        manager
    }
    
    pub async fn create_session(&self, transport_type: TransportType) -> SessionResult<Arc<Session>> {
        // Check session limit
        let active_count = self.active_sessions.read().await.len();
        if active_count >= self.config.max_sessions {
            return Err(SessionError::TooManySessions(self.config.max_sessions));
        }
        
        let session = Session {
            id: SessionId::new(),
            client_session_id: None,
            server_session_id: None,
            transport_type,
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            created_at: Instant::now(),
            ended_at: None,
            state: SessionState::Connecting,
            metadata: SessionMetadata::default(),
        };
        
        self.store.create(&session).await?;
        self.active_sessions.write().await.insert(session.id.clone());
        
        Ok(Arc::new(session))
    }
}
```

### 4. Implement Frame Management
```rust
impl SessionManager {
    pub async fn add_frame(
        &self,
        session_id: &SessionId,
        direction: Direction,
        edge: TransportEdge,
        message: &TransportMessage,
    ) -> SessionResult<()> {
        let frame = Frame {
            id: FrameId::new(),
            session_id: session_id.clone(),
            timestamp: Instant::now(),
            direction,
            edge,
            message: message.clone(),
            metadata: FrameMetadata::default(),
        };
        
        self.store.add_frame(&frame).await?;
        
        // Update session metadata
        if let Some(mut session) = self.store.get(session_id).await? {
            session.metadata.frame_count += 1;
            
            // Extract protocol info from initialize messages
            if let TransportMessage::Request { method, params, .. } = message {
                if method == "initialize" {
                    self.update_session_from_initialize(&mut session, params).await?;
                }
            }
            
            self.store.update(&session).await?;
        }
        
        Ok(())
    }
}
```

### 5. Implement Session State Transitions
```rust
impl SessionManager {
    pub async fn update_state(
        &self,
        session_id: &SessionId,
        new_state: SessionState,
    ) -> SessionResult<()> {
        let mut session = self.store.get(session_id).await?
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))?;
        
        // Validate state transition
        match (&session.state, &new_state) {
            (SessionState::Connecting, SessionState::Initialized) => {},
            (SessionState::Initialized, SessionState::Active) => {},
            (SessionState::Active, SessionState::Closing) => {},
            (SessionState::Closing, SessionState::Closed) => {},
            (_, SessionState::Failed(_)) => {}, // Can fail from any state
            _ => return Err(SessionError::InvalidStateTransition(
                format!("{:?} -> {:?}", session.state, new_state)
            )),
        }
        
        session.state = new_state;
        
        if matches!(session.state, SessionState::Closed | SessionState::Failed(_)) {
            session.ended_at = Some(Instant::now());
            self.active_sessions.write().await.remove(session_id);
        }
        
        self.store.update(&session).await?;
        Ok(())
    }
}
```

### 6. Implement Cleanup Task
```rust
impl SessionManager {
    fn start_cleanup_task(&self) {
        let store = self.store.clone();
        let config = self.config.clone();
        let active_sessions = self.active_sessions.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.cleanup_interval);
            
            loop {
                interval.tick().await;
                
                let cutoff = Instant::now() - config.session_timeout;
                
                // Find timed out sessions
                if let Ok(sessions) = store.list(SessionFilter::All).await {
                    for session in sessions {
                        if session.created_at < cutoff && 
                           matches!(session.state, SessionState::Active | SessionState::Connecting) {
                            // Mark as failed
                            let mut updated = session.clone();
                            updated.state = SessionState::Failed("Timeout".to_string());
                            updated.ended_at = Some(Instant::now());
                            
                            let _ = store.update(&updated).await;
                            active_sessions.write().await.remove(&session.id);
                        }
                    }
                }
                
                // Clean up old sessions
                let _ = store.delete_old(cutoff).await;
            }
        });
    }
}
```

---

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_session_lifecycle() {
    let store = Arc::new(MemoryStore::new());
    let manager = SessionManager::new(store, SessionConfig::default());
    
    // Create session
    let session = manager.create_session(TransportType::Stdio).await.unwrap();
    assert_eq!(session.state, SessionState::Connecting);
    
    // Add frames
    manager.add_frame(
        &session.id,
        Direction::ClientToServer,
        TransportEdge::TransportIn,
        &TransportMessage::new_request("1", "initialize", json!({}))
    ).await.unwrap();
    
    // Update state
    manager.update_state(&session.id, SessionState::Initialized).await.unwrap();
    
    // Verify
    let updated = manager.get_session(&session.id).await.unwrap().unwrap();
    assert_eq!(updated.state, SessionState::Initialized);
    assert_eq!(updated.metadata.frame_count, 1);
}

#[tokio::test]
async fn test_concurrent_frame_addition() {
    let store = Arc::new(MemoryStore::new());
    let manager = Arc::new(SessionManager::new(store, SessionConfig::default()));
    let session = manager.create_session(TransportType::Http).await.unwrap();
    
    // Add frames concurrently
    let handles: Vec<_> = (0..10).map(|i| {
        let manager = manager.clone();
        let session_id = session.id.clone();
        
        tokio::spawn(async move {
            manager.add_frame(
                &session_id,
                Direction::ClientToServer,
                TransportEdge::TransportIn,
                &TransportMessage::new_notification("test", json!({"i": i}))
            ).await
        })
    }).collect();
    
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
    
    let frames = manager.get_frames(&session.id).await.unwrap();
    assert_eq!(frames.len(), 10);
}
```

---

## Performance Considerations

- Use RwLock for read-heavy operations
- Batch frame insertions when possible
- Index sessions by multiple fields for fast lookup
- Consider using dashmap for better concurrent performance
- Implement frame pagination for large sessions

---

## Future Enhancements

- SQLite backend implementation
- Session export/import
- Real-time session monitoring
- Session replay from stored frames
- Compression for archived sessions
- Search functionality across sessions