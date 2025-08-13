# Critical Version Negotiation Bugs in Shadowcat

## Severity: CRITICAL - Must Fix Before Production

## Executive Summary
Our analysis reveals fundamental flaws in Shadowcat's MCP version negotiation that violate the specification and will cause compatibility failures with standard MCP clients and servers. These bugs affect every MCP session and must be fixed immediately.

## Bug #1: No Version Extraction from Initialize Request
**Severity**: CRITICAL
**Location**: `shadowcat/src/session/manager.rs:783-786`

### Current Behavior
```rust
TransportMessage::Request { method, .. } if method == "initialize" => {
    session.transition(SessionEvent::InitializeRequest)?;
    // Ignores params field containing protocolVersion!
}
```

### Expected Behavior
Must extract and validate `protocolVersion` from the initialize request params:
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

### Fix Required
```rust
TransportMessage::Request { method, params, .. } if method == "initialize" => {
    // Extract requested version
    if let Some(params) = params {
        if let Some(version) = params.get("protocolVersion").and_then(|v| v.as_str()) {
            session.requested_version = Some(version.to_string());
            // Validate version is supported
            if !is_version_supported(version) {
                // Must negotiate alternative version in response
                session.negotiation_required = true;
            }
        }
    }
    session.transition(SessionEvent::InitializeRequest)?;
}
```

## Bug #2: Wrong Default Version for HTTP Transport
**Severity**: CRITICAL
**Location**: `shadowcat/src/transport/http_mcp.rs:75-80`

### Current Behavior
```rust
let protocol_version = headers
    .get("mcp-protocol-version")
    .and_then(|v| v.to_str().ok())
    .unwrap_or(MCP_PROTOCOL_VERSION); // WRONG: Defaults to "2025-11-05"
```

### Expected Behavior
Per MCP 2025-06-18 specification:
> "When no MCP-Protocol-Version header is present, the server MUST default to version 2025-03-26"

### Fix Required
```rust
const HTTP_DEFAULT_VERSION: &str = "2025-03-26"; // Per spec requirement

let protocol_version = headers
    .get("mcp-protocol-version")
    .and_then(|v| v.to_str().ok())
    .unwrap_or(HTTP_DEFAULT_VERSION); // Correct default
```

## Bug #3: No Version Negotiation in Initialize Response
**Severity**: CRITICAL
**Location**: `shadowcat/src/proxy/forward.rs` (missing implementation)

### Current Behavior
- Proxy forwards initialize response without checking version compatibility
- No logic to modify response with alternative version when requested version unsupported

### Expected Behavior
Per MCP specification:
1. Server receives initialize with `protocolVersion: "2025-11-05"`
2. If unsupported, server responds with supported version: `protocolVersion: "2025-06-18"`
3. Client must accept or terminate connection

### Fix Required
```rust
// In forward.rs, when handling initialize response
async fn handle_initialize_response(
    &mut self,
    client_requested: Option<String>,
    server_response: &mut Value,
) -> Result<()> {
    if let Some(requested) = client_requested {
        if let Some(server_version) = server_response.get("protocolVersion").and_then(|v| v.as_str()) {
            // Check if versions are compatible
            if !are_versions_compatible(&requested, server_version) {
                // Try to negotiate
                if let Some(common_version) = find_common_version(&requested, server_version) {
                    // Update response with negotiated version
                    server_response["protocolVersion"] = json!(common_version);
                    self.session.negotiated_version = Some(common_version);
                } else {
                    return Err(ProxyError::VersionNegotiationFailed);
                }
            }
        }
    }
    Ok(())
}
```

## Bug #4: Version State Not Tracked Properly
**Severity**: HIGH
**Location**: `shadowcat/src/session/store.rs:131`

### Current Behavior
```rust
protocol_version: crate::transport::MCP_PROTOCOL_VERSION.to_string(),
// Hardcoded, never updated with negotiated version
```

### Expected Behavior
Must track version negotiation state:
- Requested version (from client initialize)
- Offered versions (from server)
- Negotiated version (final agreed version)
- Transport version (from HTTP headers)

### Fix Required
```rust
#[derive(Debug, Clone)]
pub struct VersionState {
    pub requested: Option<String>,      // From initialize request
    pub negotiated: Option<String>,     // After negotiation
    pub transport_version: Option<String>, // From HTTP headers
    pub negotiation_method: NegotiationMethod,
}

impl Session {
    pub fn update_negotiated_version(&mut self, version: String) -> Result<()> {
        // Validate version transition is allowed
        if let Some(current) = &self.version_state.negotiated {
            if current != &version {
                return Err(SessionError::VersionRenegotiationNotAllowed);
            }
        }
        self.version_state.negotiated = Some(version);
        Ok(())
    }
}
```

## Bug #5: Dual-Channel Version Conflict Not Handled
**Severity**: HIGH
**Location**: `shadowcat/src/proxy/reverse.rs:801-805`

### Current Behavior
```rust
if session.protocol_version != mcp_headers.protocol_version {
    warn!(...); // Only warns, doesn't handle the conflict!
}
```

### Expected Behavior
For 2025-06-18+ versions using dual-channel negotiation:
1. Initialize negotiates version X
2. HTTP headers MUST include same version X
3. Mismatch is a protocol violation that must be rejected

### Fix Required
```rust
if session.version_state.negotiated.is_some() {
    let negotiated = session.version_state.negotiated.as_ref().unwrap();
    if &mcp_headers.protocol_version != negotiated {
        return Err(ReverseProxyError::VersionChannelMismatch {
            negotiated: negotiated.clone(),
            header: mcp_headers.protocol_version.clone(),
            details: "HTTP header version must match initialize negotiated version".into(),
        });
    }
}
```

## Impact Analysis

### What's Broken
1. **All MCP Clients**: Cannot properly negotiate versions with Shadowcat
2. **Version Mismatches**: Silent failures when versions incompatible
3. **Security Risk**: Potential version downgrade attacks
4. **Spec Compliance**: Violates MCP specification requirements
5. **Interoperability**: Cannot work with standard MCP implementations

### User-Visible Symptoms
- Connection failures with "protocol version mismatch" errors
- Silent message corruption when versions differ
- Inability to connect to newer/older MCP servers
- HTTP clients failing due to wrong default version

## Test Cases to Add

```rust
#[cfg(test)]
mod critical_version_tests {
    use super::*;
    
    #[test]
    fn test_extract_version_from_initialize() {
        let msg = json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18"
            },
            "id": 1
        });
        
        let version = extract_protocol_version(&msg["params"]);
        assert_eq!(version, Some("2025-06-18".to_string()));
    }
    
    #[test]
    fn test_http_default_version() {
        let headers = HeaderMap::new(); // No MCP-Protocol-Version
        let version = resolve_http_version(&headers);
        assert_eq!(version, "2025-03-26"); // Must be this per spec
    }
    
    #[test]
    fn test_version_negotiation_response() {
        let client_version = "2025-11-05";
        let supported = vec!["2025-03-26", "2025-06-18"];
        
        let negotiated = negotiate_version(client_version, &supported);
        assert_eq!(negotiated, Some("2025-06-18")); // Best compatible
    }
    
    #[test]
    fn test_dual_channel_consistency() {
        let mut session = Session::new();
        session.version_state.negotiated = Some("2025-06-18".into());
        
        let header_version = "2025-03-26";
        let result = validate_version_consistency(&session, header_version);
        assert!(result.is_err()); // Must reject mismatch
    }
}
```

## Remediation Plan

### Day 1: Critical Fixes
1. [ ] Implement version extraction from initialize params
2. [ ] Fix HTTP default version to "2025-03-26"
3. [ ] Add version negotiation logic to forward proxy
4. [ ] Update session to track version state properly

### Day 2: Testing & Validation
1. [ ] Add comprehensive version negotiation tests
2. [ ] Test with multiple MCP client/server versions
3. [ ] Validate dual-channel consistency
4. [ ] Security audit for version downgrade attacks

### Day 3: Documentation & Review
1. [ ] Update API documentation
2. [ ] Create version compatibility matrix
3. [ ] Code review with focus on edge cases
4. [ ] Performance testing of negotiation overhead

## Definition of Done
- [ ] All 5 critical bugs fixed
- [ ] Tests passing for all version scenarios
- [ ] Successfully tested with MCP 2025-03-26 clients
- [ ] Successfully tested with MCP 2025-06-18 clients  
- [ ] No version-related warnings in logs
- [ ] Documentation updated
- [ ] Security review completed

## References
- MCP 2025-03-26 Spec: `/specs/mcp/docs/specification/2025-03-26/`
- MCP 2025-06-18 Spec: `/specs/mcp/docs/specification/2025-06-18/`
- Current Implementation: `shadowcat/src/`
- Architecture Doc: `005-multi-version-architecture-design.md`