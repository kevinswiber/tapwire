# Task 0.1: Fix Initialize Version Extraction

## Task Metadata
- **Phase**: 0 (Critical Version Bug Fixes)
- **Priority**: 🔥 URGENT
- **Estimated Duration**: 2-3 hours
- **Dependencies**: None
- **Status**: ⏳ Not Started

## Problem Statement

The Shadowcat proxy completely ignores the `protocolVersion` field in MCP initialize requests, violating the MCP specification and preventing proper version negotiation. This is a critical bug that affects every MCP session.

### Current Behavior
Location: `shadowcat/src/session/manager.rs:783-786`
```rust
TransportMessage::Request { method, .. } if method == "initialize" => {
    session.transition(SessionEvent::InitializeRequest)?;
    // CRITICAL BUG: Ignores params field containing protocolVersion!
}
```

### Expected Behavior
Must extract and validate `protocolVersion` from initialize request:
```json
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-06-18",
    "capabilities": { ... }
  },
  "id": 1
}
```

## Objectives

1. Extract `protocolVersion` from initialize request params
2. Store requested version in session state
3. Validate version is supported
4. Set up foundation for version negotiation

## Implementation Plan

### Step 1: Add Version Extraction Helper
Create a helper function to safely extract protocol version:

```rust
// In shadowcat/src/protocol/mod.rs (new module)
pub fn extract_protocol_version(params: &Value) -> Option<String> {
    params
        .as_object()?
        .get("protocolVersion")?
        .as_str()
        .map(|s| s.to_string())
}
```

### Step 2: Update Session Structure
Add version tracking to Session:

```rust
// In shadowcat/src/session/store.rs
#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub requested: Option<String>,     // From client initialize
    pub negotiated: Option<String>,    // After negotiation
    pub negotiation_required: bool,    // If requested version unsupported
}

impl Session {
    pub fn set_requested_version(&mut self, version: String) -> Result<()> {
        self.version_info.requested = Some(version.clone());
        
        // Check if version is supported
        if !is_version_supported(&version) {
            self.version_info.negotiation_required = true;
        }
        
        Ok(())
    }
}
```

### Step 3: Update Initialize Handler
Modify the session manager to extract version:

```rust
// In shadowcat/src/session/manager.rs
TransportMessage::Request { method, params, id, .. } if method == "initialize" => {
    // Extract protocol version if present
    if let Some(params) = params {
        if let Some(version) = extract_protocol_version(&params) {
            debug!("Client requested protocol version: {}", version);
            session.set_requested_version(version)?;
        } else {
            // No version specified, use default
            warn!("Initialize request missing protocolVersion, using default");
            session.set_requested_version(DEFAULT_PROTOCOL_VERSION.to_string())?;
        }
    }
    
    session.transition(SessionEvent::InitializeRequest)?;
    self.track_request(session_id.clone(), message).await?;
}
```

### Step 4: Add Version Validation
Create version support checking:

```rust
// In shadowcat/src/protocol/mod.rs
pub const SUPPORTED_VERSIONS: &[&str] = &[
    "2025-03-26",  // Minimum supported
    "2025-06-18",  // Current spec
    "2025-11-05",  // Our extension
];

pub fn is_version_supported(version: &str) -> bool {
    SUPPORTED_VERSIONS.contains(&version)
}

pub fn get_best_supported_version() -> &'static str {
    SUPPORTED_VERSIONS[1]  // Default to current spec version
}
```

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_extract_version_from_valid_params() {
        let params = json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {}
        });
        
        let version = extract_protocol_version(&params);
        assert_eq!(version, Some("2025-06-18".to_string()));
    }
    
    #[test]
    fn test_extract_version_missing_field() {
        let params = json!({
            "capabilities": {}
        });
        
        let version = extract_protocol_version(&params);
        assert_eq!(version, None);
    }
    
    #[test]
    fn test_version_support_check() {
        assert!(is_version_supported("2025-06-18"));
        assert!(is_version_supported("2025-03-26"));
        assert!(!is_version_supported("2024-01-01"));
        assert!(!is_version_supported("invalid"));
    }
    
    #[test]
    fn test_session_version_tracking() {
        let mut session = Session::new(SessionId::new(), TransportType::Stdio);
        
        session.set_requested_version("2025-06-18".to_string()).unwrap();
        assert_eq!(session.version_info.requested, Some("2025-06-18".to_string()));
        assert!(!session.version_info.negotiation_required);
        
        session.set_requested_version("2099-01-01".to_string()).unwrap();
        assert!(session.version_info.negotiation_required);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_initialize_version_extraction_flow() {
    let manager = SessionManager::new(store);
    
    let init_msg = TransportMessage::Request {
        id: Some(MessageId::Number(1)),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {
                "experimental": {}
            }
        })),
    };
    
    let session_id = SessionId::new();
    manager.process_message_for_session(session_id.clone(), init_msg).await.unwrap();
    
    let session = manager.get_session(&session_id).await.unwrap();
    assert_eq!(session.version_info.requested, Some("2025-06-18".to_string()));
}
```

## Files to Modify

1. **Create**: `shadowcat/src/protocol/mod.rs` - New module for protocol version handling
2. **Modify**: `shadowcat/src/session/store.rs` - Add VersionInfo to Session
3. **Modify**: `shadowcat/src/session/manager.rs:783-800` - Update initialize handler
4. **Modify**: `shadowcat/src/lib.rs` - Add protocol module
5. **Create**: `shadowcat/tests/version_extraction_test.rs` - Integration tests

## Acceptance Criteria

- [ ] Protocol version extracted from initialize params
- [ ] Version stored in session state
- [ ] Unsupported versions flagged for negotiation
- [ ] Missing version handled with appropriate default
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Debug logging added for version tracking

## Rollback Plan

If issues arise:
1. The changes are isolated to version extraction
2. Can be disabled with feature flag if needed
3. Existing behavior preserved for messages without version

## Notes for Next Session

- After this task, proceed to Task 0.2 (Fix HTTP Default Version)
- Version negotiation response (Task 0.3) depends on this foundation
- Consider adding metrics for version usage tracking

## References

- MCP Spec 2025-06-18: `/specs/mcp/docs/specification/2025-06-18/basic/lifecycle.mdx`
- MCP Spec 2025-03-26: `/specs/mcp/docs/specification/2025-03-26/basic/lifecycle.mdx`
- Critical Bug Report: `../006-critical-version-bugs.md`