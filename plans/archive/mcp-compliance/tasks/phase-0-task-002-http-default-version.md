# Task 0.2: Fix HTTP Default Version

## Task Metadata
- **Phase**: 0 (Critical Version Bug Fixes)
- **Priority**: 🔥 URGENT
- **Estimated Duration**: 1-2 hours
- **Dependencies**: None (can run parallel with Task 0.1)
- **Status**: ⏳ Not Started

## Problem Statement

The HTTP transport incorrectly defaults to version "2025-11-05" when no `MCP-Protocol-Version` header is present. Per MCP 2025-06-18 specification, it MUST default to "2025-03-26" for backward compatibility.

### Current Behavior
Location: `shadowcat/src/transport/http_mcp.rs:75-80`
```rust
let protocol_version = headers
    .get("mcp-protocol-version")
    .and_then(|v| v.to_str().ok())
    .unwrap_or(MCP_PROTOCOL_VERSION); // WRONG: Uses "2025-11-05"
```

### Expected Behavior
Per MCP specification:
> "When no MCP-Protocol-Version header is present, the server MUST default to version 2025-03-26"

## Objectives

1. Change HTTP default version to "2025-03-26"
2. Add proper constants for version defaults
3. Update all HTTP header extraction code
4. Ensure consistency across codebase

## Implementation Plan

### Step 1: Define Proper Constants
Create version constants module:

```rust
// In shadowcat/src/protocol/constants.rs
/// Default protocol version when none specified (per MCP spec)
pub const DEFAULT_PROTOCOL_VERSION: &str = "2025-03-26";

/// HTTP-specific default when no header present (per MCP 2025-06-18 spec)
pub const HTTP_DEFAULT_PROTOCOL_VERSION: &str = "2025-03-26";

/// Version where dual-channel negotiation was introduced
pub const DUAL_CHANNEL_VERSION: &str = "2025-06-18";

/// Our current implementation version
pub const IMPLEMENTATION_VERSION: &str = "2025-11-05";
```

### Step 2: Fix HTTP Header Extraction
Update the HTTP MCP module:

```rust
// In shadowcat/src/transport/http_mcp.rs
use crate::protocol::constants::HTTP_DEFAULT_PROTOCOL_VERSION;

pub fn extract_mcp_headers(headers: &HeaderMap) -> ReverseProxyResult<McpHeaders> {
    // Extract protocol version with correct default
    let protocol_version = headers
        .get("mcp-protocol-version")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
        .unwrap_or_else(|| {
            debug!("No MCP-Protocol-Version header, defaulting to {}", HTTP_DEFAULT_PROTOCOL_VERSION);
            HTTP_DEFAULT_PROTOCOL_VERSION.to_string()
        });
    
    // Validate the version
    if !is_version_supported(&protocol_version) {
        warn!("Client using unsupported version: {}", protocol_version);
        // Don't reject immediately - allow negotiation
    }
    
    Ok(McpHeaders {
        protocol_version,
        session_id,
    })
}
```

### Step 3: Update HTTP Transport
Fix the HTTP transport module:

```rust
// In shadowcat/src/transport/http.rs
use crate::protocol::constants::HTTP_DEFAULT_PROTOCOL_VERSION;

impl HttpTransport {
    fn get_protocol_version(&self, headers: &HeaderMap) -> String {
        headers
            .get("mcp-protocol-version")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string())
            .unwrap_or_else(|| HTTP_DEFAULT_PROTOCOL_VERSION.to_string())
    }
}
```

### Step 4: Remove Hardcoded Version Usage
Search and replace hardcoded version references:

```rust
// In shadowcat/src/session/store.rs
use crate::protocol::constants::DEFAULT_PROTOCOL_VERSION;

impl Session {
    pub fn new(id: SessionId, transport_type: TransportType) -> Self {
        Self {
            id: id.clone(),
            transport_type,
            // Use proper default, not hardcoded
            protocol_version: DEFAULT_PROTOCOL_VERSION.to_string(),
            // ...
        }
    }
}
```

### Step 5: Add Version Helpers
Create helper functions for version checking:

```rust
// In shadowcat/src/protocol/mod.rs
use self::constants::*;

/// Check if this version uses dual-channel negotiation
pub fn uses_dual_channel_negotiation(version: &str) -> bool {
    version >= DUAL_CHANNEL_VERSION
}

/// Get the appropriate default version for a transport type
pub fn get_transport_default_version(transport_type: TransportType) -> &'static str {
    match transport_type {
        TransportType::Http | TransportType::HttpSse => HTTP_DEFAULT_PROTOCOL_VERSION,
        TransportType::Stdio => DEFAULT_PROTOCOL_VERSION,
    }
}
```

## Testing Requirements

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_http_default_version() {
        let headers = HeaderMap::new();  // No MCP-Protocol-Version
        let mcp_headers = extract_mcp_headers(&headers).unwrap();
        
        assert_eq!(mcp_headers.protocol_version, "2025-03-26");
    }
    
    #[test]
    fn test_explicit_version_preserved() {
        let mut headers = HeaderMap::new();
        headers.insert("mcp-protocol-version", "2025-06-18".parse().unwrap());
        
        let mcp_headers = extract_mcp_headers(&headers).unwrap();
        assert_eq!(mcp_headers.protocol_version, "2025-06-18");
    }
    
    #[test]
    fn test_transport_default_versions() {
        assert_eq!(get_transport_default_version(TransportType::Http), "2025-03-26");
        assert_eq!(get_transport_default_version(TransportType::HttpSse), "2025-03-26");
        assert_eq!(get_transport_default_version(TransportType::Stdio), "2025-03-26");
    }
    
    #[test]
    fn test_dual_channel_detection() {
        assert!(!uses_dual_channel_negotiation("2025-03-26"));
        assert!(uses_dual_channel_negotiation("2025-06-18"));
        assert!(uses_dual_channel_negotiation("2025-11-05"));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_http_request_without_version_header() {
    let client = TestClient::new();
    
    // Send request without MCP-Protocol-Version header
    let response = client
        .post("/mcp")
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "ping",
            "id": 1
        }))
        .send()
        .await
        .unwrap();
    
    // Should work with default version 2025-03-26
    assert_eq!(response.status(), 200);
}
```

## Files to Modify

1. **Create**: `shadowcat/src/protocol/constants.rs` - Version constants
2. **Modify**: `shadowcat/src/transport/http_mcp.rs:75-80` - Fix default version
3. **Modify**: `shadowcat/src/transport/http.rs` - Update version handling
4. **Modify**: `shadowcat/src/session/store.rs:131` - Use proper default
5. **Modify**: `shadowcat/src/transport/mod.rs:25` - Update constant definition
6. **Create**: `shadowcat/tests/http_version_default_test.rs` - Integration tests

## Acceptance Criteria

- [ ] HTTP defaults to "2025-03-26" when no header present
- [ ] Explicit version headers are preserved
- [ ] All hardcoded versions replaced with constants
- [ ] Transport-specific defaults implemented
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Debug logging for default version usage

## Performance Considerations

- No performance impact expected
- String allocations minimized by using &'static str where possible
- Consider caching version strings to avoid repeated allocations

## Rollback Plan

If issues arise:
1. Changes are isolated to version defaulting logic
2. Can temporarily revert to old default with single constant change
3. No data migration required

## Notes for Next Session

- This task can be done in parallel with Task 0.1
- Task 0.3 (Version Negotiation) will build on both 0.1 and 0.2
- Consider adding metrics for version usage patterns

## References

- MCP Spec 2025-06-18: Section on HTTP Transport headers
- Critical Bug Report: `../006-critical-version-bugs.md`
- Original constant location: `shadowcat/src/transport/mod.rs:25`