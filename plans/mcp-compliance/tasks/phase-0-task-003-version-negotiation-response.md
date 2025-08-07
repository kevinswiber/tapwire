# Task 0.3: Implement Version Negotiation Response

## Task Metadata
- **Phase**: 0 (Critical Version Bug Fixes)
- **Priority**: 🔥 URGENT
- **Estimated Duration**: 3-4 hours
- **Dependencies**: Task 0.1 (Initialize Version Extraction)
- **Status**: ⏳ Not Started

## Problem Statement

The forward proxy doesn't implement version negotiation. When a client requests an unsupported version, the proxy should negotiate an alternative version in the initialize response rather than failing or silently accepting incompatible versions.

### Current Behavior
- Proxy forwards initialize requests/responses without version checking
- No modification of responses to negotiate versions
- No compatibility validation between client and server versions

### Expected Behavior
Per MCP specification:
1. Client sends initialize with `protocolVersion: "X"`
2. If server doesn't support X, it responds with supported version Y
3. Client must accept the negotiated version or disconnect
4. Proxy must facilitate this negotiation transparently

## Objectives

1. Intercept initialize responses in forward proxy
2. Check version compatibility between client and server
3. Negotiate common version when mismatch occurs
4. Update session with negotiated version
5. Handle incompatible version scenarios

## Implementation Plan

### Step 1: Create Version Negotiation Logic
Add negotiation algorithms:

```rust
// In shadowcat/src/protocol/negotiation.rs
use super::constants::*;

#[derive(Debug, Clone)]
pub struct VersionNegotiator {
    supported_versions: Vec<String>,
}

impl VersionNegotiator {
    pub fn new() -> Self {
        Self {
            supported_versions: SUPPORTED_VERSIONS
                .iter()
                .map(|&v| v.to_string())
                .collect(),
        }
    }
    
    /// Find best common version between client and server
    pub fn negotiate(
        &self,
        client_version: &str,
        server_version: &str,
    ) -> Result<String, NegotiationError> {
        // If versions match, no negotiation needed
        if client_version == server_version {
            return Ok(client_version.to_string());
        }
        
        // Check if both versions are supported by proxy
        if !self.is_supported(client_version) {
            return self.offer_alternative(client_version);
        }
        
        if !self.is_supported(server_version) {
            return self.offer_alternative(server_version);
        }
        
        // Both supported but different - check compatibility
        if self.are_compatible(client_version, server_version) {
            // Prefer newer version if compatible
            Ok(self.select_preferred(client_version, server_version))
        } else {
            // Find common compatible version
            self.find_common_version(client_version, server_version)
        }
    }
    
    /// Check if two versions are compatible
    fn are_compatible(&self, v1: &str, v2: &str) -> bool {
        // 2025-03-26 and 2025-06-18 have different negotiation methods
        match (v1, v2) {
            ("2025-03-26", "2025-06-18") | ("2025-06-18", "2025-03-26") => true,
            (a, b) if a == b => true,
            _ => false,
        }
    }
    
    fn offer_alternative(&self, requested: &str) -> Result<String, NegotiationError> {
        // Find closest supported version
        if requested < "2025-03-26" {
            Ok("2025-03-26".to_string())  // Minimum supported
        } else if requested > "2025-11-05" {
            Ok("2025-06-18".to_string())  // Latest stable
        } else {
            // Find nearest version
            self.find_nearest_version(requested)
        }
    }
}
```

### Step 2: Update Forward Proxy to Handle Initialize
Modify forward proxy to intercept initialize:

```rust
// In shadowcat/src/proxy/forward.rs
use crate::protocol::negotiation::VersionNegotiator;

impl ForwardProxy {
    async fn handle_initialize_request(
        &mut self,
        request: &TransportMessage,
        session_id: &SessionId,
    ) -> Result<()> {
        // Extract client requested version
        let client_version = if let TransportMessage::Request { params, .. } = request {
            params.as_ref()
                .and_then(|p| extract_protocol_version(p))
        } else {
            None
        };
        
        // Store in session for response handling
        if let Some(version) = client_version {
            self.sessions.set_client_requested_version(session_id, version).await?;
        }
        
        // Forward request as-is
        self.forward_to_upstream(request).await
    }
    
    async fn handle_initialize_response(
        &mut self,
        response: &mut TransportMessage,
        session_id: &SessionId,
    ) -> Result<()> {
        // Get client's requested version
        let client_version = self.sessions
            .get_client_requested_version(session_id)
            .await?
            .unwrap_or_else(|| DEFAULT_PROTOCOL_VERSION.to_string());
        
        // Extract server's version from response
        if let TransportMessage::Response { result, .. } = response {
            if let Some(server_version) = result.as_ref()
                .and_then(|r| extract_protocol_version(r)) {
                
                // Negotiate if versions differ
                let negotiator = VersionNegotiator::new();
                match negotiator.negotiate(&client_version, &server_version) {
                    Ok(negotiated) => {
                        if negotiated != server_version {
                            // Update response with negotiated version
                            info!("Negotiating version: client={}, server={}, negotiated={}",
                                client_version, server_version, negotiated);
                            
                            if let Some(result) = result.as_mut() {
                                result["protocolVersion"] = json!(negotiated);
                            }
                        }
                        
                        // Update session with final version
                        self.sessions.set_negotiated_version(
                            session_id,
                            negotiated.clone()
                        ).await?;
                    }
                    Err(e) => {
                        // No compatible version - convert to error response
                        warn!("Version negotiation failed: {}", e);
                        *response = create_version_error_response(request_id, e);
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

### Step 3: Add Response Interception
Hook into message flow:

```rust
// In shadowcat/src/proxy/forward.rs
impl ForwardProxy {
    async fn process_upstream_response(
        &mut self,
        response: TransportMessage,
        session_id: SessionId,
    ) -> Result<TransportMessage> {
        let mut response = response;
        
        // Check if this is an initialize response
        if self.is_initialize_response(&response).await? {
            self.handle_initialize_response(&mut response, &session_id).await?;
        }
        
        Ok(response)
    }
    
    async fn is_initialize_response(&self, msg: &TransportMessage) -> Result<bool> {
        if let TransportMessage::Response { id, .. } = msg {
            // Check if this ID corresponds to an initialize request
            if let Some(id) = id {
                return Ok(self.pending_requests.is_initialize(id).await);
            }
        }
        Ok(false)
    }
}
```

### Step 4: Create Error Response for Failed Negotiation
Handle negotiation failures gracefully:

```rust
// In shadowcat/src/protocol/negotiation.rs
pub fn create_version_error_response(
    request_id: Option<MessageId>,
    error: NegotiationError,
) -> TransportMessage {
    TransportMessage::Response {
        id: request_id,
        result: None,
        error: Some(json!({
            "code": -32600,  // Invalid Request per JSON-RPC
            "message": "Protocol version negotiation failed",
            "data": {
                "error": error.to_string(),
                "supported_versions": SUPPORTED_VERSIONS,
            }
        })),
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
    fn test_version_negotiation_same() {
        let negotiator = VersionNegotiator::new();
        let result = negotiator.negotiate("2025-06-18", "2025-06-18").unwrap();
        assert_eq!(result, "2025-06-18");
    }
    
    #[test]
    fn test_version_negotiation_compatible() {
        let negotiator = VersionNegotiator::new();
        
        // 2025-03-26 client with 2025-06-18 server
        let result = negotiator.negotiate("2025-03-26", "2025-06-18").unwrap();
        assert_eq!(result, "2025-03-26");  // Use client's version for compatibility
    }
    
    #[test]
    fn test_version_negotiation_unsupported() {
        let negotiator = VersionNegotiator::new();
        
        // Future version not supported
        let result = negotiator.negotiate("2099-01-01", "2025-06-18");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2025-06-18");  // Fallback to latest stable
    }
    
    #[test]
    fn test_initialize_response_modification() {
        let mut response = TransportMessage::Response {
            id: Some(MessageId::Number(1)),
            result: Some(json!({
                "protocolVersion": "2025-11-05",
                "capabilities": {}
            })),
            error: None,
        };
        
        // Simulate negotiation to different version
        if let TransportMessage::Response { result, .. } = &mut response {
            if let Some(result) = result.as_mut() {
                result["protocolVersion"] = json!("2025-06-18");
            }
        }
        
        // Verify modification
        if let TransportMessage::Response { result, .. } = response {
            assert_eq!(
                result.unwrap()["protocolVersion"].as_str().unwrap(),
                "2025-06-18"
            );
        }
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_forward_proxy_version_negotiation() {
    let mut proxy = ForwardProxy::new(config);
    
    // Client requests unsupported version
    let init_request = TransportMessage::Request {
        id: Some(MessageId::Number(1)),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocolVersion": "2099-01-01",
            "capabilities": {}
        })),
    };
    
    // Mock server responds with its version
    let server_response = TransportMessage::Response {
        id: Some(MessageId::Number(1)),
        result: Some(json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {}
        })),
        error: None,
    };
    
    // Process through proxy
    let negotiated_response = proxy
        .process_upstream_response(server_response, session_id)
        .await
        .unwrap();
    
    // Should have negotiated to compatible version
    if let TransportMessage::Response { result, .. } = negotiated_response {
        let version = result.unwrap()["protocolVersion"].as_str().unwrap();
        assert_eq!(version, "2025-06-18");
    }
}
```

## Files to Modify

1. **Create**: `shadowcat/src/protocol/negotiation.rs` - Negotiation logic
2. **Modify**: `shadowcat/src/proxy/forward.rs` - Add response interception
3. **Modify**: `shadowcat/src/session/manager.rs` - Track initialize requests
4. **Create**: `shadowcat/tests/version_negotiation_test.rs` - Integration tests
5. **Modify**: `shadowcat/src/protocol/mod.rs` - Export negotiation module

## Acceptance Criteria

- [ ] Initialize responses intercepted and processed
- [ ] Version compatibility checked
- [ ] Common version negotiated when possible
- [ ] Error response for incompatible versions
- [ ] Session updated with negotiated version
- [ ] All tests passing
- [ ] No clippy warnings
- [ ] Logging for negotiation decisions

## Edge Cases to Handle

1. Client requests future version → negotiate to latest supported
2. Server offers older version → check compatibility
3. No common version → return error
4. Missing version in response → use defaults
5. Network failure during negotiation → proper cleanup

## Performance Considerations

- Negotiation adds minimal latency (< 1ms)
- Cache negotiation results per session
- Avoid repeated version string parsing

## Rollback Plan

If issues arise:
1. Disable response interception with feature flag
2. Fall back to pass-through behavior
3. Log warnings for version mismatches

## Notes for Next Session

- Task 0.4 will build on this to add comprehensive version state management
- Consider adding metrics for negotiation success/failure rates
- May need to handle version-specific message transformations later

## References

- MCP Lifecycle: `/specs/mcp/docs/specification/2025-06-18/basic/lifecycle.mdx`
- Version Negotiation Spec: See initialize request/response sections
- Bug Report: `../006-critical-version-bugs.md`