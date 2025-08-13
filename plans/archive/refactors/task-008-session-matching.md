# Task 008: Complete Session Matching

## Overview
Implement proper session matching logic to correctly track and associate MCP messages with their respective sessions.

## Context
From the [comprehensive review](../../reviews/shadowcat-comprehensive-review-2025-08-06.md), session matching is marked as CRITICAL with incomplete implementation. The TODO at `src/session/manager.rs:108` indicates core session tracking functionality is missing.

## Scope
- **Files to modify**: `src/session/manager.rs`, potentially `src/session/mod.rs`
- **Priority**: CRITICAL - Core functionality
- **Time estimate**: 1 day

## Current Problem

### Incomplete Implementation
**Location**: `src/session/manager.rs:108`
```rust
// TODO: Implement proper session matching logic
```

The session matching logic is critical for:
- Associating requests with responses
- Tracking session lifecycle
- Managing session state transitions
- Proper cleanup and resource management

## Implementation Plan

### Step 1: Analyze MCP Message Types
Understand all MCP message types that need session tracking:
- `initialize` - Creates new session
- `initialized` - Confirms session creation
- `ping`/`pong` - Heartbeat messages
- Tool calls and responses
- Resource operations
- Prompt operations
- `shutdown` - Ends session

### Step 2: Implement Session Matching Logic

#### Session ID Extraction
```rust
impl SessionManager {
    fn extract_session_id(&self, message: &Value) -> Option<String> {
        // Check for session ID in various locations:
        // 1. Headers (Mcp-Session-Id)
        // 2. Message metadata
        // 3. Context from transport
        
        // For initialize requests, generate new session ID
        if self.is_initialize_request(message) {
            return Some(self.generate_session_id());
        }
        
        // For responses, match with pending requests
        if let Some(id) = message.get("id") {
            return self.match_response_to_session(id);
        }
        
        // Extract from existing session context
        self.extract_from_context(message)
    }
    
    fn is_initialize_request(&self, message: &Value) -> bool {
        message.get("method")
            .and_then(|m| m.as_str())
            .map(|m| m == "initialize")
            .unwrap_or(false)
    }
}
```

#### Request-Response Correlation
```rust
#[derive(Debug, Clone)]
struct PendingRequest {
    session_id: String,
    request_id: Value,
    method: String,
    timestamp: SystemTime,
}

impl SessionManager {
    pending_requests: Arc<RwLock<HashMap<Value, PendingRequest>>>,
    
    async fn track_request(&self, session_id: String, message: &Value) -> Result<()> {
        if let Some(id) = message.get("id") {
            let pending = PendingRequest {
                session_id: session_id.clone(),
                request_id: id.clone(),
                method: message.get("method")
                    .and_then(|m| m.as_str())
                    .unwrap_or_default()
                    .to_string(),
                timestamp: SystemTime::now(),
            };
            
            let mut requests = self.pending_requests.write()
                .map_err(|e| SessionError::LockPoisoned(e.to_string()))?;
            requests.insert(id.clone(), pending);
        }
        Ok(())
    }
    
    fn match_response_to_session(&self, response_id: &Value) -> Option<String> {
        let requests = self.pending_requests.read().ok()?;
        requests.get(response_id)
            .map(|req| req.session_id.clone())
    }
}
```

### Step 3: Handle Session Lifecycle

#### Session State Machine
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Initializing,
    Active,
    ShuttingDown,
    Closed,
    Failed(String),
}

impl Session {
    pub fn transition(&mut self, event: SessionEvent) -> Result<(), SessionError> {
        match (&self.state, event) {
            (SessionState::Initializing, SessionEvent::Initialized) => {
                self.state = SessionState::Active;
                Ok(())
            }
            (SessionState::Active, SessionEvent::ShutdownRequest) => {
                self.state = SessionState::ShuttingDown;
                Ok(())
            }
            (SessionState::ShuttingDown, SessionEvent::ShutdownComplete) => {
                self.state = SessionState::Closed;
                Ok(())
            }
            (_, SessionEvent::Error(msg)) => {
                self.state = SessionState::Failed(msg);
                Ok(())
            }
            _ => Err(SessionError::InvalidTransition),
        }
    }
}
```

### Step 4: Implement Proper Cleanup
```rust
impl SessionManager {
    async fn cleanup_response(&self, response_id: &Value) -> Result<()> {
        let mut requests = self.pending_requests.write()
            .map_err(|e| SessionError::LockPoisoned(e.to_string()))?;
        
        if let Some(pending) = requests.remove(response_id) {
            // Log completion time
            let duration = SystemTime::now()
                .duration_since(pending.timestamp)
                .unwrap_or_default();
            
            tracing::debug!(
                session_id = %pending.session_id,
                method = %pending.method,
                duration_ms = %duration.as_millis(),
                "Request completed"
            );
        }
        
        Ok(())
    }
}
```

### Step 5: Add Timeout Handling
```rust
impl SessionManager {
    async fn cleanup_stale_requests(&self) {
        let timeout = Duration::from_secs(30);
        let now = SystemTime::now();
        
        let mut requests = match self.pending_requests.write() {
            Ok(r) => r,
            Err(_) => return,
        };
        
        let stale_ids: Vec<Value> = requests
            .iter()
            .filter_map(|(id, req)| {
                now.duration_since(req.timestamp)
                    .ok()
                    .and_then(|d| (d > timeout).then(|| id.clone()))
            })
            .collect();
        
        for id in stale_ids {
            if let Some(pending) = requests.remove(&id) {
                tracing::warn!(
                    session_id = %pending.session_id,
                    method = %pending.method,
                    "Request timed out"
                );
            }
        }
    }
}
```

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_initialize_creates_session() {
        let manager = SessionManager::new(Default::default());
        let message = json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {},
            "id": 1
        });
        
        let session_id = manager.extract_session_id(&message);
        assert!(session_id.is_some());
        
        let session = manager.get_session(&session_id.unwrap()).await;
        assert!(session.is_ok());
    }
    
    #[tokio::test]
    async fn test_response_matches_request() {
        let manager = SessionManager::new(Default::default());
        let session_id = "test-session";
        
        // Track request
        let request = json!({"id": 1, "method": "ping"});
        manager.track_request(session_id.to_string(), &request).await.unwrap();
        
        // Match response
        let response = json!({"id": 1, "result": "pong"});
        let matched = manager.match_response_to_session(&json!(1));
        assert_eq!(matched, Some(session_id.to_string()));
    }
    
    #[tokio::test]
    async fn test_session_state_transitions() {
        let mut session = Session::new("test");
        
        assert_eq!(session.state, SessionState::Initializing);
        
        session.transition(SessionEvent::Initialized).unwrap();
        assert_eq!(session.state, SessionState::Active);
        
        session.transition(SessionEvent::ShutdownRequest).unwrap();
        assert_eq!(session.state, SessionState::ShuttingDown);
        
        session.transition(SessionEvent::ShutdownComplete).unwrap();
        assert_eq!(session.state, SessionState::Closed);
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_full_session_lifecycle() {
    // Test complete flow from initialize to shutdown
    // Including request/response matching
}
```

## Validation

### Pre-check
```bash
# Find TODO comment
rg "TODO.*session matching" --type rust

# Check current implementation
rg "extract_session_id|match_response" --type rust
```

### Post-check
```bash
# Ensure TODO is removed
rg "TODO.*session matching" --type rust | wc -l  # Should be 0

# Run tests
cargo test session::manager

# Check session tracking works
cargo run -- forward stdio -- echo '{"jsonrpc":"2.0","method":"initialize","id":1}'
```

## Success Criteria

- [ ] Session matching logic handles all MCP message types
- [ ] Request-response correlation works correctly
- [ ] Session state transitions are properly managed
- [ ] Stale requests are cleaned up
- [ ] TODO comment removed
- [ ] Unit tests cover all scenarios
- [ ] Integration tests pass
- [ ] No performance degradation

## Implementation Order

1. Add session state enum and transitions
2. Implement session ID extraction
3. Add request-response correlation
4. Implement cleanup for completed requests
5. Add timeout handling for stale requests
6. Write comprehensive tests
7. Remove TODO comment

## Common Pitfalls to Avoid

1. **Race conditions** - Use proper locking for concurrent access
2. **Memory leaks** - Ensure pending requests are cleaned up
3. **Session orphaning** - Handle disconnections properly
4. **ID collisions** - Use UUIDs or similar for session IDs

## Dependencies

- May need to update `Session` struct in `src/session/mod.rs`
- Coordinate with transport layer for session context
- Consider impact on interceptor chain

## Notes

- Session matching is critical for proxy functionality
- Affects recording, replay, and interception features
- Must handle both stdio and HTTP transports consistently
- Consider adding metrics for session tracking