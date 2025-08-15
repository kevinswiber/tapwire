# Architectural Decision: Frames vs MessageEnvelopes

## Discovery
During Phase B implementation, we discovered that the SessionStore is conflating two distinct concepts:
- **Recording Frames**: Used for tape recording/playback 
- **MessageEnvelopes**: Live protocol messages in transit

## The Problem
The current `InMemorySessionStore` stores `MessageEnvelope` objects in a `frames` field, treating them as if they were recording frames. This is problematic because:

1. **Different Purposes**:
   - Recording frames capture traffic for replay with timing metadata
   - MessageEnvelopes are live messages that should be forwarded and forgotten
   
2. **Memory Waste**: 
   - Storing every message that passes through the proxy is unnecessary
   - Live proxy operation doesn't need message history
   
3. **Semantic Confusion**:
   - Frames belong in the recording/tape domain
   - MessageEnvelopes belong in the transport/protocol domain

## Decision
**Remove frame storage from SessionStore entirely**. The SessionStore should focus on:
- Session state and metadata
- SSE Last-Event-Id for reconnection  
- Active connection tracking
- Session lifecycle management

Recording/playback should have its own separate storage mechanism for frames.

## Impact on Implementation

### SessionStore Trait Changes
Remove these methods:
```rust
// REMOVE - these don't belong in session management
async fn add_frame(&self, frame: MessageEnvelope) -> SessionResult<()>;
async fn get_frames(&self, session_id: &SessionId) -> SessionResult<Vec<MessageEnvelope>>;
async fn delete_frames(&self, session_id: &SessionId) -> SessionResult<()>;
```

Keep only session-focused methods:
```rust
// Core session operations
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
```

### Recording Module
The recording functionality should:
- Have its own frame storage (separate from SessionStore)
- Define proper Frame types with timing metadata
- Handle tape file I/O independently
- Not interfere with live proxy operations

### Benefits
1. **Clear separation of concerns**: Session management vs recording
2. **Better memory usage**: Only store what's needed for operation
3. **Cleaner architecture**: Each domain has its own storage
4. **Future flexibility**: Recording can evolve independently

## Implementation Order
1. Update SessionStore trait to remove frame methods
2. Update InMemorySessionStore to remove frame storage
3. Ensure recording module has its own frame management
4. Update any code that was incorrectly using frames for message history