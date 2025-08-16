# Task B.2: Migrate Usage Sites

## Objective
Update all code that references the removed is_sse_session field and its associated methods to use the new ResponseMode and ClientCapabilities system. This task systematically fixes all compilation errors introduced by B.1's structural changes.

## Context from Phase A Analysis

### What We're Migrating (Reference: analysis/transport-usage-audit.md)
**Dead Code Removed**:
- `is_sse_session: bool` field (line 68 in src/session/store.rs)
- `mark_as_sse_session()` method (lines 249-252, never called!)
- `is_sse()` method (lines 255-257, barely used)

**Limited Usage Found**:
- `is_sse()` called only in src/proxy/forward.rs line 245
- Used to decide response handling strategy
- No other meaningful usage in codebase

### Migration Strategy (Reference: analysis/migration-strategy.md)
1. **Replace boolean checks with ResponseMode pattern matching**
2. **Use ClientCapabilities for feature detection**
3. **Update response routing logic to use response_mode**
4. **Remove all references to old methods**

### Design Context (Reference: analysis/architecture-proposal.md)
The migration maintains these principles:
- Response mode is detected at runtime from Content-Type
- Client capabilities determine what can be sent
- No assumptions about transport-response mode relationships
- Clean separation between transport type and response format

## Prerequisites
- [x] B.0 complete (ResponseMode and ClientCapabilities exist)
- [x] B.1 complete (Session struct updated)
- [ ] Compilation errors documented from B.1
- [ ] On correct git branch

## Detailed Implementation Steps

### Step 1: Document Compilation Errors (10 min)

```bash
cd /Users/kevin/src/tapwire/shadowcat

# Capture all compilation errors
cargo build 2>&1 | tee /tmp/b2_errors.txt

# Extract unique error locations
grep -E "error\[E[0-9]+\]|no (field|method)" /tmp/b2_errors.txt | \
  grep -v "^error:" | sort -u > /tmp/b2_locations.txt

# Common patterns to look for:
# - "no field `is_sse_session`"
# - "no method named `mark_as_sse_session`"
# - "no method named `is_sse`"

# Find all usage sites programmatically
rg "is_sse_session" src/ tests/ --type rust
rg "mark_as_sse_session" src/ tests/ --type rust
rg "\.is_sse\(\)" src/ tests/ --type rust
```

Document each error location for systematic fixing.

### Step 2: Update Forward Proxy (20 min)

The main usage is in `src/proxy/forward.rs`. Update the response handling logic:

```rust
// In src/proxy/forward.rs around line 245

// BEFORE (old code):
if session.is_sse() {
    // Handle SSE response
    self.handle_sse_response(response, &mut client_transport).await?;
} else {
    // Handle JSON response
    self.handle_json_response(response, &mut client_transport).await?;
}

// AFTER (new code):
use crate::transport::core::ResponseMode;

// Detect response mode from Content-Type header
let content_type = response.headers()
    .get("content-type")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("application/json");

let response_mode = ResponseMode::from_content_type(content_type);

// Update session with detected response mode
{
    let mut session = self.session_store.get_mut(&session_id).await?
        .ok_or_else(|| ProxyError::SessionNotFound(session_id.clone()))?;
    session.set_response_mode(response_mode);
    
    // For distributed storage, persist the update
    self.session_store.update(&session_id, session).await?;
}

// Route based on response mode
match response_mode {
    ResponseMode::SseStream => {
        // Check if client can handle SSE
        let session = self.session_store.get(&session_id).await?
            .ok_or_else(|| ProxyError::SessionNotFound(session_id.clone()))?;
        
        if !session.accepts_response_mode(ResponseMode::SseStream) {
            return Err(ProxyError::UnsupportedResponseFormat {
                mode: response_mode,
                capabilities: session.client_capabilities,
            });
        }
        
        self.handle_sse_response(response, &mut client_transport).await?;
    }
    ResponseMode::Json => {
        self.handle_json_response(response, &mut client_transport).await?;
    }
    ResponseMode::Passthrough => {
        // Stream without processing
        self.handle_passthrough_response(response, &mut client_transport).await?;
    }
}
```

Add new error types if needed:
```rust
// In src/error.rs or src/proxy/error.rs

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    // ... existing variants ...
    
    #[error("Client doesn't support response format {mode:?} (capabilities: {capabilities:?})")]
    UnsupportedResponseFormat {
        mode: ResponseMode,
        capabilities: ClientCapabilities,
    },
}
```

### Step 3: Update Reverse Proxy (15 min)

Update `src/proxy/reverse.rs` to use ResponseMode:

```rust
// In src/proxy/reverse.rs

use crate::transport::core::{ResponseMode, ClientCapabilities};

impl ReverseProxy {
    /// Handle incoming HTTP request with proper capability detection
    async fn handle_request(&mut self, req: Request<Body>) -> Result<Response<Body>> {
        // Extract client capabilities from Accept header
        let accept_header = req.headers()
            .get("accept")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("*/*");
        
        let client_capabilities = ClientCapabilities::from_accept_header(accept_header);
        
        // Create or update session with capabilities
        let session_id = self.extract_session_id(&req)?;
        let mut session = self.session_store.get_or_create(session_id).await?;
        session.update_capabilities(client_capabilities);
        
        // If this is a reverse proxy, track upstream session
        if let Some(upstream_id) = self.get_upstream_session_id(&req)? {
            session.set_upstream_session_id(upstream_id);
        }
        
        self.session_store.update(&session_id, session).await?;
        
        // Forward request and get response
        let upstream_response = self.forward_to_upstream(req).await?;
        
        // Detect response mode from upstream
        let content_type = upstream_response.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/json");
        
        let response_mode = ResponseMode::from_content_type(content_type);
        
        // Update session with response mode
        {
            let mut session = self.session_store.get_mut(&session_id).await?
                .ok_or_else(|| ProxyError::SessionNotFound(session_id.clone()))?;
            session.set_response_mode(response_mode);
            self.session_store.update(&session_id, session).await?;
        }
        
        // Check if client accepts this response mode
        let session = self.session_store.get(&session_id).await?
            .ok_or_else(|| ProxyError::SessionNotFound(session_id.clone()))?;
        
        if !session.accepts_response_mode(response_mode) {
            // Return 406 Not Acceptable
            return Ok(Response::builder()
                .status(406)
                .body(Body::from("Client doesn't accept response format"))
                .unwrap());
        }
        
        Ok(upstream_response)
    }
}
```

### Step 4: Update Transport Implementations (15 min)

Update any transport implementations that might reference session fields:

```rust
// In src/transport/directional/incoming.rs (if needed)

// Look for any code that might have accessed session.is_sse_session
// This is unlikely since transports shouldn't know about session internals

// However, StreamableHttpIncoming might need updating:
impl StreamableHttpIncoming {
    /// Determine if we should respond with SSE based on session
    async fn should_use_sse(&self, session_id: &SessionId) -> bool {
        // BEFORE:
        // let session = self.session_store.get(session_id).await?;
        // session.is_sse()
        
        // AFTER:
        if let Ok(Some(session)) = self.session_store.get(session_id).await {
            matches!(session.response_mode, Some(ResponseMode::SseStream))
        } else {
            false
        }
    }
}
```

### Step 5: Update Tests (20 min)

Fix all test compilation errors:

```rust
// In tests throughout the codebase

// Pattern 1: Tests that set is_sse_session
// BEFORE:
#[test]
fn test_sse_session() {
    let mut session = Session::new(id, TransportType::Http);
    session.is_sse_session = true;
    assert!(session.is_sse());
}

// AFTER:
#[test]
fn test_sse_session() {
    let mut session = Session::new(id, TransportType::Http);
    session.set_response_mode(ResponseMode::SseStream);
    assert!(session.is_streaming());
}

// Pattern 2: Tests that call mark_as_sse_session
// BEFORE:
#[test]
fn test_mark_sse() {
    let mut session = Session::new(id, TransportType::Http);
    session.mark_as_sse_session();
    assert!(session.is_sse());
}

// AFTER:
#[test]
fn test_mark_sse() {
    let mut session = Session::new(id, TransportType::Http);
    session.set_response_mode(ResponseMode::SseStream);
    assert!(session.is_streaming());
    assert_eq!(session.response_mode, Some(ResponseMode::SseStream));
}

// Pattern 3: Tests that check is_sse()
// BEFORE:
if session.is_sse() {
    // handle SSE
}

// AFTER:
if session.is_streaming() {
    // handle streaming response
}
// OR more precisely:
if matches!(session.response_mode, Some(ResponseMode::SseStream)) {
    // handle SSE specifically
}
```

### Step 6: Update Integration Tests (15 min)

Fix integration tests in `tests/` directory:

```rust
// In tests/integration/*.rs

// Look for session creation in test fixtures
fn create_test_session() -> Session {
    let mut session = Session::new(SessionId::new(), TransportType::Http);
    
    // REMOVE:
    // session.is_sse_session = true;
    
    // ADD:
    session.set_response_mode(ResponseMode::SseStream);
    session.update_capabilities(ClientCapabilities::STREAMING);
    
    session
}

// Update assertions
// BEFORE:
assert!(session.is_sse());

// AFTER:
assert!(session.is_streaming());
assert_eq!(session.response_mode, Some(ResponseMode::SseStream));
```

### Step 7: Add Migration Documentation (10 min)

Create a migration guide for future reference:

```rust
// In src/session/migration.md or similar

/// Migration Guide: is_sse_session to ResponseMode
/// 
/// The old boolean is_sse_session field has been replaced with a more
/// flexible ResponseMode enum that tracks response formats.
/// 
/// ## Quick Reference
/// 
/// | Old Code | New Code |
/// |----------|----------|
/// | `session.is_sse_session = true` | `session.set_response_mode(ResponseMode::SseStream)` |
/// | `session.mark_as_sse_session()` | `session.set_response_mode(ResponseMode::SseStream)` |
/// | `session.is_sse()` | `session.is_streaming()` |
/// | `if session.is_sse_session` | `if matches!(session.response_mode, Some(ResponseMode::SseStream))` |
/// 
/// ## Conceptual Changes
/// 
/// 1. **Response mode is detected, not configured**
///    - Old: Set is_sse_session manually
///    - New: Detect from Content-Type header
/// 
/// 2. **Client capabilities are explicit**
///    - Old: Assumed based on transport type
///    - New: Tracked via ClientCapabilities bitflags
/// 
/// 3. **Multiple response modes supported**
///    - Old: Binary choice (JSON or SSE)
///    - New: Json, SseStream, Passthrough (extensible)
```

### Step 8: Compile and Test (10 min)

```bash
# Clean build to ensure no cached artifacts
cargo clean

# Build with all features
cargo build --all-features

# Run all tests
cargo test

# Check for any remaining references
rg "is_sse_session|mark_as_sse_session|\.is_sse\(\)" src/ tests/ --type rust

# Should return no results! If it does, fix them.

# Run clippy to catch any style issues
cargo clippy --all-targets -- -D warnings

# Format code
cargo fmt
```

## Success Criteria Checklist

- [ ] All compilation errors from B.1 resolved
- [ ] No references to is_sse_session field remain
- [ ] No references to mark_as_sse_session() method remain  
- [ ] No references to is_sse() method remain
- [ ] Forward proxy uses ResponseMode for routing
- [ ] Reverse proxy uses ClientCapabilities for negotiation
- [ ] All tests updated to use new API
- [ ] Integration tests pass
- [ ] No clippy warnings
- [ ] Migration guide documented
- [ ] Code compiles without warnings
- [ ] All tests pass

## Common Issues and Solutions

1. **"cannot find field is_sse_session"**
   - Replace with response_mode field
   - Use set_response_mode() method

2. **"no method named is_sse"**
   - Replace with is_streaming() for general streaming check
   - Use matches!(response_mode, Some(ResponseMode::SseStream)) for SSE-specific

3. **Type mismatch errors**
   - Ensure ResponseMode and ClientCapabilities are imported
   - Check that you're using Option<ResponseMode> not ResponseMode

4. **Test fixture failures**
   - Update all test session creation to use new methods
   - Ensure test sessions have appropriate capabilities set

5. **Distributed storage issues**
   - Remember to call session_store.update() after modifying session
   - ResponseMode and ClientCapabilities must be Serialize/Deserialize

## Migration Patterns Reference

### Pattern 1: Response Detection
```rust
// Detect from HTTP response
let response_mode = ResponseMode::from_content_type(
    response.headers().get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json")
);
```

### Pattern 2: Capability Check
```rust
// Check if client can handle response
if !session.accepts_response_mode(response_mode) {
    return Err(ProxyError::UnsupportedFormat);
}
```

### Pattern 3: Session Update
```rust
// Update session with response mode
session.set_response_mode(response_mode);
// For distributed: session_store.update(&id, session).await?;
```

### Pattern 4: Streaming Check
```rust
// Check if currently streaming
if session.is_streaming() {
    // Handle streaming response
}
```

## Duration Estimate
**Total: 95 minutes**
- Document errors: 10 min
- Update forward proxy: 20 min
- Update reverse proxy: 15 min
- Update transports: 15 min
- Update tests: 20 min
- Update integration tests: 15 min
- Add documentation: 10 min
- Compile and test: 10 min

## Next Steps
After completing this task:
1. Verify no compilation errors remain
2. Run full test suite
3. Commit changes: `git commit -m "refactor(session): migrate all usage sites to ResponseMode"`
4. Proceed to B.3 for validation and cleanup

## Notes
- This is where we see the real impact of the refactor
- Expect to find dead code that was never executed
- The forward proxy is the main user of this functionality
- Take time to understand each usage before changing it
- Document any surprising findings for the team

---

**Task Status**: Ready for implementation
**Prerequisites**: B.0 and B.1 complete
**Blocks**: B.3 (test and validate)
**Reviewer**: Check that all old methods are completely gone