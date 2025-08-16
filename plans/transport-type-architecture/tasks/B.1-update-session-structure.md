# Task B.1: Update Session Structure

## Objective
Update the Session struct to use ResponseMode and ClientCapabilities, completely removing the dead is_sse_session field and its associated methods. This is the core structural change that eliminates the code smell.

## Context from Phase A Analysis

### What We're Fixing (Reference: analysis/transport-usage-audit.md)
- **is_sse_session field**: Line 68 in src/session/store.rs
- **mark_as_sse_session() method**: Lines 249-252 (never called!)
- **is_sse() getter**: Lines 255-257 (barely used)
- **Default initialization**: Line 91 (always false)

### Design Context (Reference: analysis/architecture-proposal.md)
The updated Session structure must:
- Track response mode per request (not per session)
- Store client capabilities for content negotiation
- Support optional upstream session ID for reverse proxy
- Remain serializable for distributed storage (Redis future)

### Distributed Storage Requirements (Reference: analysis/distributed-storage-considerations.md)
- All fields must be Serialize/Deserialize
- Updates must be atomic (get-modify-update pattern)
- Compatible with existing SessionStore trait

## Prerequisites
- [x] B.0 complete (ResponseMode and ClientCapabilities exist)
- [ ] Types compile and pass tests
- [ ] On correct git branch

## Detailed Implementation Steps

### Step 1: Locate and Backup Current Session (5 min)

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Create backup of current Session implementation
cp src/session/store.rs src/session/store.rs.backup

# Review current structure
grep -n "pub struct Session" src/session/store.rs
grep -n "is_sse_session" src/session/store.rs
grep -n "mark_as_sse_session" src/session/store.rs
```

Document the line numbers for reference during editing.

### Step 2: Update Session Struct (15 min)

Edit `src/session/store.rs`:

```rust
use crate::transport::core::{ResponseMode, ClientCapabilities};  // Add imports

/// Session information tracked by the proxy
/// 
/// This struct must remain serializable for distributed storage backends.
/// All updates should be atomic through the SessionStore trait.
/// 
/// Major changes from previous version:
/// - Removed is_sse_session boolean (was dead code)
/// - Added response_mode for proper response format tracking
/// - Added client_capabilities using efficient bitflags
/// - Added upstream_session_id for reverse proxy dual session tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique identifier for this proxy session
    pub id: SessionId,
    
    /// Transport type for session categorization
    pub transport_type: TransportType,
    
    /// Current session status (Active, Closed, Error)
    pub status: SessionStatus,
    
    /// Session state machine position
    pub state: SessionState,
    
    /// Creation timestamp (milliseconds since UNIX epoch)
    pub created_at: u64,
    
    /// Last activity timestamp (milliseconds since UNIX epoch)
    pub last_activity: u64,
    
    /// Number of frames/messages processed
    pub frame_count: usize,
    
    /// Client information string (optional)
    pub client_info: Option<String>,
    
    /// Server information string (optional)
    pub server_info: Option<String>,
    
    /// Protocol version negotiation state
    pub version_state: VersionState,
    
    /// Tags for session classification
    pub tags: Vec<String>,
    
    /// SSE: Last event ID for reconnection (kept for SSE support)
    pub last_event_id: Option<String>,
    
    // REMOVED: pub is_sse_session: bool,
    
    /// NEW: Current response mode (detected from Content-Type)
    /// This replaces is_sse_session with proper type-safe tracking
    pub response_mode: Option<ResponseMode>,
    
    /// NEW: Client capabilities for content negotiation
    /// Replaces implicit assumptions with explicit capability tracking
    pub client_capabilities: ClientCapabilities,
    
    /// NEW: Upstream session ID for reverse proxy dual session tracking
    /// None for forward proxy, Some(id) for reverse proxy
    /// See: plans/reverse-proxy-session-mapping/
    pub upstream_session_id: Option<SessionId>,
}
```

### Step 3: Update Session Implementation (20 min)

Update the `impl Session` block:

```rust
impl Session {
    /// Create a new session with modern field initialization
    pub fn new(id: SessionId, transport_type: TransportType) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_millis(0))
            .as_millis() as u64;

        // Derive client capabilities from transport type
        let client_capabilities = ClientCapabilities::from_transport_type(transport_type);

        Self {
            id,
            transport_type,
            status: SessionStatus::Active,
            state: SessionState::Initializing,
            created_at: now,
            last_activity: now,
            frame_count: 0,
            client_info: None,
            server_info: None,
            version_state: VersionState::new(),
            tags: Vec::new(),
            last_event_id: None,
            // REMOVED: is_sse_session: false,
            response_mode: None,  // Detected at runtime from Content-Type
            client_capabilities,   // Derived from transport type
            upstream_session_id: None,  // Set by reverse proxy if needed
        }
    }

    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.last_activity = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_millis(0))
            .as_millis() as u64;
    }

    /// Increment frame count and update activity
    pub fn add_frame(&mut self) {
        self.frame_count += 1;
        self.update_activity();
    }

    /// Update session status
    pub fn set_status(&mut self, status: SessionStatus) {
        self.status = status;
        self.update_activity();
    }

    /// Update session state
    pub fn set_state(&mut self, state: SessionState) {
        self.state = state;
        self.update_activity();
    }

    // REMOVED: pub fn mark_as_sse_session(&mut self) - Dead code!
    // REMOVED: pub fn is_sse(&self) -> bool - Replaced by response_mode

    /// NEW: Set the response mode based on Content-Type detection
    /// 
    /// This replaces the old is_sse_session tracking with proper
    /// response format detection. Called when response headers are received.
    /// 
    /// Note: Caller must persist via SessionStore for distributed scenarios
    pub fn set_response_mode(&mut self, mode: ResponseMode) {
        self.response_mode = Some(mode);
        self.update_activity();
        
        // Log mode changes for debugging
        debug!(
            session_id = %self.id,
            mode = %mode,
            "Session response mode updated"
        );
    }

    /// NEW: Check if session is currently streaming
    /// 
    /// Replaces the old is_sse() method with ResponseMode-based check
    pub fn is_streaming(&self) -> bool {
        self.response_mode
            .map(|m| m.is_streaming())
            .unwrap_or(false)
    }

    /// NEW: Check if session supports streaming
    /// 
    /// Based on client capabilities, not current response mode
    pub fn supports_streaming(&self) -> bool {
        self.client_capabilities.supports_streaming()
    }

    /// NEW: Check if client accepts a specific response mode
    pub fn accepts_response_mode(&self, mode: ResponseMode) -> bool {
        self.client_capabilities.accepts_response_mode(mode)
    }

    /// NEW: Update client capabilities (e.g., from Accept header)
    /// 
    /// Used by reverse proxy when client sends explicit Accept header
    pub fn update_capabilities(&mut self, capabilities: ClientCapabilities) {
        self.client_capabilities = capabilities;
        self.update_activity();
    }

    /// NEW: Set upstream session ID for reverse proxy
    /// 
    /// Used for dual session tracking in reverse proxy scenarios
    pub fn set_upstream_session_id(&mut self, upstream_id: SessionId) {
        self.upstream_session_id = Some(upstream_id);
        self.update_activity();
    }

    /// Check if session has timed out
    pub fn is_timed_out(&self, timeout_ms: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_millis(0))
            .as_millis() as u64;
        
        now - self.last_activity > timeout_ms
    }

    /// Get session duration in milliseconds
    pub fn duration_ms(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_millis(0))
            .as_millis() as u64;
        
        now.saturating_sub(self.created_at)
    }
}
```

### Step 4: Add Migration Helper (10 min)

Add a temporary migration helper for any code that might still reference old methods:

```rust
#[cfg(test)]
impl Session {
    /// Test helper to verify is_sse_session is gone
    #[deprecated(note = "is_sse_session has been removed - use response_mode instead")]
    pub fn test_no_sse_session(&self) {
        // This method exists only to catch any test code still using old patterns
        panic!("is_sse_session has been removed! Use response_mode instead.");
    }
}
```

### Step 5: Update Session Tests (15 min)

Update tests in the same file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::core::{ResponseMode, ClientCapabilities};

    #[tokio::test]
    async fn test_session_creation() {
        let session_id = SessionId::new();
        let session = Session::new(session_id.clone(), TransportType::Stdio);
        
        assert_eq!(session.id, session_id);
        assert_eq!(session.transport_type, TransportType::Stdio);
        assert_eq!(session.status, SessionStatus::Active);
        assert_eq!(session.state, SessionState::Initializing);
        
        // New field checks
        assert_eq!(session.response_mode, None);
        assert_eq!(session.client_capabilities, ClientCapabilities::STANDARD);
        assert_eq!(session.upstream_session_id, None);
        
        // Verify is_sse_session is gone (won't compile if field exists)
        // assert!(!session.is_sse_session);  // This line should fail to compile
    }

    #[tokio::test]
    async fn test_response_mode_tracking() {
        let mut session = Session::new(SessionId::new(), TransportType::Http);
        
        // Initially no response mode
        assert_eq!(session.response_mode, None);
        assert!(!session.is_streaming());
        
        // Set JSON mode
        session.set_response_mode(ResponseMode::Json);
        assert_eq!(session.response_mode, Some(ResponseMode::Json));
        assert!(!session.is_streaming());
        
        // Set SSE mode
        session.set_response_mode(ResponseMode::SseStream);
        assert_eq!(session.response_mode, Some(ResponseMode::SseStream));
        assert!(session.is_streaming());
    }

    #[tokio::test]
    async fn test_client_capabilities() {
        let mut session = Session::new(SessionId::new(), TransportType::Stdio);
        
        // Stdio gets standard capabilities
        assert_eq!(session.client_capabilities, ClientCapabilities::STANDARD);
        assert!(session.accepts_response_mode(ResponseMode::Json));
        assert!(!session.accepts_response_mode(ResponseMode::SseStream));
        
        // Update to streaming capabilities
        session.update_capabilities(ClientCapabilities::STREAMING);
        assert!(session.accepts_response_mode(ResponseMode::Json));
        assert!(session.accepts_response_mode(ResponseMode::SseStream));
        assert!(session.supports_streaming());
    }

    #[tokio::test]
    async fn test_session_serialization() {
        // Verify session remains serializable for distributed storage
        let session = Session::new(SessionId::new(), TransportType::Http);
        
        let serialized = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(session.id, deserialized.id);
        assert_eq!(session.response_mode, deserialized.response_mode);
        assert_eq!(session.client_capabilities, deserialized.client_capabilities);
    }

    #[tokio::test]
    async fn test_upstream_session_id() {
        let mut session = Session::new(SessionId::new(), TransportType::Http);
        
        // Initially None (forward proxy)
        assert_eq!(session.upstream_session_id, None);
        
        // Set for reverse proxy
        let upstream_id = SessionId::new();
        session.set_upstream_session_id(upstream_id.clone());
        assert_eq!(session.upstream_session_id, Some(upstream_id));
    }
}
```

### Step 6: Update Memory Store Implementation (10 min)

If there's an InMemorySessionStore in `src/session/memory.rs`, update any references:

```rust
// In memory.rs or wherever InMemorySessionStore is implemented

// No changes needed to the store itself - it just stores Session objects
// But update any test fixtures that create sessions:

#[cfg(test)]
fn create_test_session() -> Session {
    let mut session = Session::new(SessionId::new(), TransportType::Http);
    // Don't set is_sse_session anymore!
    // session.is_sse_session = true;  // REMOVE THIS
    session.set_response_mode(ResponseMode::SseStream);  // Use this instead
    session
}
```

### Step 7: Compile and Fix Errors (10 min)

```bash
# Attempt compilation to find all usage sites
cargo build 2>&1 | tee build_errors.txt

# Common errors and fixes:
# "no field `is_sse_session`" - Good! The field is gone
# "method `mark_as_sse_session` not found" - Good! Dead code removed
# "method `is_sse` not found" - Replace with is_streaming()

# Search for any remaining references
grep -r "is_sse_session" src/
grep -r "mark_as_sse_session" src/
grep -r "\.is_sse()" src/
```

### Step 8: Run Tests (5 min)

```bash
# Run session-specific tests
cargo test session:: -- --nocapture

# Run all tests to catch integration issues
cargo test

# Check for any deprecation warnings
cargo build 2>&1 | grep -i deprecat
```

## Success Criteria Checklist

- [ ] Session struct updated with new fields
- [ ] is_sse_session field completely removed
- [ ] mark_as_sse_session() method removed
- [ ] is_sse() method removed
- [ ] response_mode field added (Option<ResponseMode>)
- [ ] client_capabilities field added (ClientCapabilities)
- [ ] upstream_session_id field added (Option<SessionId>)
- [ ] New helper methods implemented (set_response_mode, is_streaming, etc.)
- [ ] Session remains Serialize/Deserialize
- [ ] All session tests updated and passing
- [ ] No compilation errors
- [ ] No references to old fields/methods remain

## Common Issues and Solutions

1. **Compilation errors in other modules**
   - These are good! They show us where to update in B.2
   - Document each error location for systematic fixing
   - Don't fix them yet - that's B.2's job

2. **Test failures**
   - Update test fixtures to use new methods
   - Replace `session.is_sse_session = true` with `session.set_response_mode(ResponseMode::SseStream)`
   - Replace `session.is_sse()` with `session.is_streaming()`

3. **Serialization issues**
   - Ensure ResponseMode and ClientCapabilities are imported
   - Both types must derive Serialize/Deserialize (they do from B.0)

4. **Missing imports**
   - Add `use crate::transport::core::{ResponseMode, ClientCapabilities};`
   - May need `use crate::transport::{ResponseMode, ClientCapabilities};` depending on re-exports

## Design Rationale References

From our analysis phase:
- **Why remove is_sse_session**: It's completely dead code (analysis/transport-usage-audit.md)
- **Why add ResponseMode**: Proper type-safe response format tracking (analysis/response-mode-investigation.md)
- **Why ClientCapabilities**: Explicit capability negotiation (analysis/implementation-recommendations.md)
- **Why upstream_session_id**: Dual session tracking for reverse proxy (plans/reverse-proxy-session-mapping/)

## Duration Estimate
**Total: 60 minutes**
- Locate and backup: 5 min
- Update struct: 15 min
- Update implementation: 20 min
- Add migration helper: 10 min
- Update tests: 15 min
- Update memory store: 10 min
- Compile and fix: 10 min
- Run tests: 5 min

## Next Steps
After completing this task:
1. Document all compilation errors for B.2
2. Commit changes: `git commit -m "refactor(session): replace is_sse_session with ResponseMode and ClientCapabilities"`
3. Proceed to B.2 to fix all usage sites

## Notes
- This is the core structural change - expect compilation errors
- Don't try to fix everything - B.2 handles usage sites
- Focus on getting Session struct correct
- Remember: No backward compatibility - remove old code completely!

---

**Task Status**: Ready for implementation
**Prerequisites**: B.0 complete
**Blocks**: B.2 (usage site migration)
**Reviewer**: Verify all old fields/methods are gone