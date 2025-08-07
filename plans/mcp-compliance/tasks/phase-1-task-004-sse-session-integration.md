# Phase 1 - Task 1.4: SSE Session Integration

## Task Overview
Integrate SSE connections with the existing session management system, ensuring proper session tracking, state management, and lifecycle coordination between SSE streams and MCP sessions.

**Duration**: 3-4 hours
**Priority**: HIGH - Required for proper session-aware SSE communication
**Dependencies**: Tasks 1.1-1.3 (SSE infrastructure) and Phase 0 (session management) complete

## Objectives

### Primary Goals
1. Link SSE connections to MCP sessions
2. Track SSE streams per session
3. Handle session-scoped event IDs
4. Coordinate session lifecycle with connections
5. Implement session-aware reconnection

### Success Criteria
- [ ] SSE connections properly associated with sessions
- [ ] Session ID headers included in all SSE requests
- [ ] Event IDs scoped to sessions as per spec
- [ ] Clean connection cleanup on session termination
- [ ] Session state preserved across reconnections
- [ ] Multiple streams per session properly managed
- [ ] Session expiry handled gracefully
- [ ] Proper isolation between sessions
- [ ] Test coverage for session scenarios

## Technical Requirements

### MCP Session Requirements
From the MCP Streamable HTTP specification:

1. **Session Headers**:
   - `Mcp-Session-Id`: Required after initialization
   - `MCP-Protocol-Version`: Include negotiated version
   - Sessions assigned during initialization

2. **Event ID Scope**:
   - Event IDs globally unique within session
   - Independent ID spaces per session
   - Resumption scoped to session

3. **Session Lifecycle**:
   - HTTP 404 indicates expired session
   - DELETE request for explicit termination
   - All streams closed on session end

4. **Multiple Streams**:
   - Client MAY maintain multiple streams per session
   - Server MUST NOT duplicate messages across streams
   - Each stream independent within session

## Implementation Plan

### Module Structure
```
src/transport/sse/
├── session.rs         # Session-SSE integration
├── session_manager.rs # Extended session manager
└── tests/
    └── session.rs     # Session integration tests

src/session/
├── sse_integration.rs # SSE-specific session extensions
```

### Core Components

#### 1. SSE Session Extension (`session/sse_integration.rs`)
```rust
use crate::session::{Session, SessionId};
use crate::transport::sse::{SseConnection, EventTracker};

pub struct SseSessionState {
    session_id: SessionId,
    mcp_session_id: Option<String>,  // From Mcp-Session-Id header
    protocol_version: String,
    connections: HashMap<Uuid, ConnectionInfo>,
    event_tracker: Arc<EventTracker>,
    created_at: Instant,
    last_activity: RwLock<Instant>,
}

#[derive(Debug)]
struct ConnectionInfo {
    id: Uuid,
    url: String,
    state: ConnectionState,
    last_event_id: Option<String>,
    created_at: Instant,
}

impl SseSessionState {
    pub fn new(session_id: SessionId, protocol_version: String) -> Self;
    
    pub fn set_mcp_session_id(&mut self, id: String);
    
    pub fn add_connection(&mut self, conn: SseConnection) -> Result<Uuid, SessionError>;
    
    pub fn remove_connection(&mut self, conn_id: Uuid);
    
    pub fn get_connection(&self, conn_id: Uuid) -> Option<&ConnectionInfo>;
    
    pub fn active_connections(&self) -> Vec<Uuid>;
    
    pub fn close_all_connections(&mut self);
    
    pub fn is_expired(&self, timeout: Duration) -> bool;
    
    pub fn record_activity(&self);
}
```

#### 2. Session-Aware Connection Manager (`sse/session.rs`)
```rust
pub struct SessionAwareSseManager {
    base_manager: Arc<SseConnectionManager>,
    session_store: Arc<RwLock<HashMap<SessionId, SseSessionState>>>,
    config: SessionSseConfig,
}

impl SessionAwareSseManager {
    pub fn new(base_manager: Arc<SseConnectionManager>) -> Self;
    
    pub async fn create_session(
        &self,
        protocol_version: String,
    ) -> Result<SessionId, SessionError>;
    
    pub async fn post_with_session(
        &self,
        session_id: SessionId,
        url: &str,
        message: JsonRpcMessage,
    ) -> Result<SseResponse, SessionError>;
    
    pub async fn open_stream_with_session(
        &self,
        session_id: SessionId,
        url: &str,
    ) -> Result<SessionStream, SessionError>;
    
    pub fn terminate_session(&self, session_id: SessionId);
    
    pub fn get_session_state(&self, session_id: SessionId) -> Option<SseSessionState>;
    
    async fn handle_session_expiry(&self, session_id: SessionId);
    
    fn build_session_headers(
        &self,
        session: &SseSessionState,
        last_event_id: Option<&str>,
    ) -> HeaderMap;
}

pub struct SessionStream {
    inner: ReconnectingStream,
    session_id: SessionId,
    manager: Arc<SessionAwareSseManager>,
}

impl Stream for SessionStream {
    type Item = Result<SseEvent, SessionError>;
    
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        // Check session validity
        // Forward to inner stream
        // Update session activity
    }
}
```

#### 3. Session Lifecycle Hooks (`session/sse_integration.rs`)
```rust
pub trait SseSessionLifecycle {
    fn on_session_created(&self, session_id: SessionId);
    fn on_session_initialized(&self, session_id: SessionId, mcp_id: String);
    fn on_connection_added(&self, session_id: SessionId, conn_id: Uuid);
    fn on_connection_removed(&self, session_id: SessionId, conn_id: Uuid);
    fn on_session_terminated(&self, session_id: SessionId);
    fn on_session_expired(&self, session_id: SessionId);
}

pub struct SseSessionCoordinator {
    lifecycle_hooks: Vec<Box<dyn SseSessionLifecycle>>,
    expiry_monitor: JoinHandle<()>,
}

impl SseSessionCoordinator {
    pub fn new(config: SessionConfig) -> Self;
    
    pub fn register_hook(&mut self, hook: Box<dyn SseSessionLifecycle>);
    
    pub async fn start_monitoring(&self, manager: Arc<SessionAwareSseManager>);
    
    async fn check_expired_sessions(&self, manager: &SessionAwareSseManager);
    
    fn notify_lifecycle_event(&self, event: SessionEvent);
}
```

#### 4. Event ID Management (`sse/session.rs`)
```rust
pub struct SessionEventIdGenerator {
    session_id: SessionId,
    counter: AtomicU64,
    prefix: String,
}

impl SessionEventIdGenerator {
    pub fn new(session_id: SessionId) -> Self;
    
    pub fn generate_id(&self) -> String {
        let count = self.counter.fetch_add(1, Ordering::SeqCst);
        format!("{}-{}-{}", self.prefix, self.session_id, count)
    }
    
    pub fn parse_id(&self, id: &str) -> Option<(SessionId, u64)>;
    
    pub fn is_session_event(&self, id: &str) -> bool;
}
```

### Integration Flow

1. **Session Creation**:
   ```
   Initialize request → Create session → 
   Store protocol version → Generate session ID →
   Return with Mcp-Session-Id header
   ```

2. **Connection Establishment**:
   ```
   Request with session ID → Validate session →
   Create SSE connection → Register with session →
   Track in session state
   ```

3. **Event Processing**:
   ```
   Receive event → Extract session context →
   Update session activity → Check event ID uniqueness →
   Process event
   ```

4. **Session Termination**:
   ```
   Termination trigger → Close all connections →
   Clean up resources → Notify hooks →
   Remove from store
   ```

### Configuration

```rust
pub struct SessionSseConfig {
    pub max_connections_per_session: usize,     // Default: 10
    pub session_timeout: Duration,              // Default: 30 minutes
    pub idle_timeout: Duration,                 // Default: 5 minutes
    pub expiry_check_interval: Duration,        // Default: 60 seconds
    pub require_session_id: bool,               // Default: true
    pub auto_terminate_on_expiry: bool,         // Default: true
}

impl Default for SessionSseConfig {
    fn default() -> Self {
        Self {
            max_connections_per_session: 10,
            session_timeout: Duration::from_secs(1800),
            idle_timeout: Duration::from_secs(300),
            expiry_check_interval: Duration::from_secs(60),
            require_session_id: true,
            auto_terminate_on_expiry: true,
        }
    }
}
```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session not found: {0}")]
    SessionNotFound(SessionId),
    
    #[error("Session expired: {0}")]
    SessionExpired(SessionId),
    
    #[error("Maximum connections exceeded for session: {session_id}")]
    TooManyConnections { session_id: SessionId },
    
    #[error("Invalid session ID in header: {0}")]
    InvalidSessionId(String),
    
    #[error("Session required but not provided")]
    SessionRequired,
    
    #[error("Connection {conn_id} not found in session {session_id}")]
    ConnectionNotFound { session_id: SessionId, conn_id: Uuid },
    
    #[error("SSE error: {0}")]
    Sse(#[from] SseError),
}
```

## Test Cases

### Unit Tests

1. **Session Creation**:
   - Create new session
   - Set MCP session ID
   - Verify initial state

2. **Connection Management**:
   - Add connections to session
   - Remove connections
   - Enforce connection limits
   - List active connections

3. **Event ID Scoping**:
   - Generate unique IDs per session
   - Verify ID format
   - Parse session from ID

4. **Session Expiry**:
   - Detect idle sessions
   - Detect expired sessions
   - Activity updates

### Integration Tests

1. **Full Session Flow**:
   ```rust
   #[tokio::test]
   async fn test_session_lifecycle() {
       let manager = create_session_manager();
       
       // Create session
       let session_id = manager.create_session("2025-06-18").await?;
       
       // Open multiple streams
       let stream1 = manager.open_stream_with_session(session_id, url1).await?;
       let stream2 = manager.open_stream_with_session(session_id, url2).await?;
       
       // Verify isolation
       let other_session = manager.create_session("2025-06-18").await?;
       assert_ne!(session_id, other_session);
       
       // Terminate session
       manager.terminate_session(session_id);
       
       // Verify connections closed
       assert!(stream1.next().await.is_none());
       assert!(stream2.next().await.is_none());
   }
   ```

2. **Session Expiry Handling**:
   - Create session with short timeout
   - Wait for expiry
   - Verify automatic cleanup
   - Check 404 responses

3. **Reconnection with Session**:
   - Disconnect connection
   - Reconnect with same session
   - Verify state preserved

4. **Cross-Session Isolation**:
   - Create multiple sessions
   - Verify event ID independence
   - Check connection isolation

## Performance Considerations

1. **Session Lookup**: Use efficient HashMap for O(1) lookups
2. **Connection Tracking**: Limit connections per session
3. **Memory Management**: Clean up expired sessions promptly
4. **Lock Contention**: Use RwLock for read-heavy operations
5. **Event ID Generation**: Atomic operations for thread safety

## Metrics to Track

- Active sessions count
- Connections per session
- Session duration statistics
- Session expiry rate
- Event IDs per session
- Memory usage per session

## Integration Points

1. **Session Manager**: Extend existing session system
2. **SSE Connections**: Wrap with session context
3. **Protocol Module**: Use version from session
4. **Metrics**: Report session-level statistics
5. **Audit**: Log session lifecycle events

## Next Steps

After completing this task:
1. Task 1.5: Performance optimization and benchmarks
2. Begin Phase 2: Multi-version architecture

## Notes

- Consider session persistence for recovery
- Implement session migration for load balancing
- Document session timeout behavior
- Add session inspection/debugging tools
- Consider rate limiting per session