# Multi-Version MCP Specification Support Architecture

## Executive Summary
Shadowcat must evolve to support multiple MCP specification versions simultaneously. This document outlines the architectural requirements and design patterns needed to handle version negotiation, feature compatibility, and graceful degradation across different MCP protocol versions.

## Current State & Problem
- **Hardcoded Version**: Currently using constant `MCP_PROTOCOL_VERSION = "2025-11-05"`
- **No Version Negotiation**: Cannot adapt to different client/server versions
- **Rigid Implementation**: Code directly implements single version's behavior
- **Breaking Changes Risk**: Future spec updates will break existing implementations
- **Critical Flaw**: No extraction of `protocolVersion` from initialize messages
- **Missing Logic**: No version negotiation response when requested version unsupported
- **Wrong Default**: Should default to `2025-03-26` when no HTTP header present (per spec)

## Version Negotiation Evolution

### 2025-03-26 Version (Minimum Supported)
- **Method**: Version negotiation via `initialize` request/response only
- **Flow**: Client sends `protocolVersion` in params → Server responds with same or alternative
- **Transport**: No transport-level version headers
- **Default**: No concept of default version

### 2025-06-18+ Version
- **Dual-Channel**: Version in both initialize AND HTTP headers
- **HTTP Header**: `MCP-Protocol-Version` header MUST be included after initialization
- **Backwards Compatible**: Defaults to `2025-03-26` when no header present
- **Consistency**: Header version must match negotiated initialize version

## Requirements for Multi-Version Support

### Core Requirements
1. **Simultaneous Support**: Handle multiple protocol versions in single binary
2. **Version Negotiation**: Dynamically select best compatible version
3. **Feature Detection**: Enable/disable features based on version
4. **Graceful Degradation**: Fall back to older versions when needed
5. **Forward Compatibility**: Design for unknown future versions
6. **Dual-Source Handling**: Reconcile version from initialize vs HTTP headers
7. **Transport-Aware**: Different negotiation for stdio vs HTTP transports

### Version Lifecycle Management
- Support at least 3 versions simultaneously (current, previous, next/experimental)
- Deprecation warnings for old versions
- Migration paths between versions
- Version-specific testing

## Proposed Architecture

### 1. Version Registry Pattern
```rust
// shadowcat/src/protocol/version_registry.rs
pub struct VersionRegistry {
    versions: HashMap<ProtocolVersion, Box<dyn ProtocolImplementation>>,
    default_version: ProtocolVersion,
    supported_range: VersionRange,
}

pub trait ProtocolImplementation: Send + Sync {
    fn version(&self) -> &ProtocolVersion;
    fn capabilities(&self) -> &VersionCapabilities;
    fn create_transport(&self, config: &TransportConfig) -> Box<dyn Transport>;
    fn create_session_manager(&self) -> Box<dyn SessionManager>;
    fn validate_message(&self, msg: &Value) -> Result<()>;
    fn transform_message(&self, msg: Value, target_version: &ProtocolVersion) -> Result<Value>;
}
```

### 2. Version-Specific Implementations
```rust
// shadowcat/src/protocol/versions/
mod v2025_03_26;  // Minimum supported version (initialize-only negotiation)
mod v2025_06_18;  // Dual-channel negotiation (initialize + HTTP headers)
mod v2025_11_05;  // Current implementation (needs fixing)
mod v2026_xx_xx;  // Future version

// Each version module contains:
pub struct Version2025_06_18 {
    // Version-specific configuration
}

impl ProtocolImplementation for Version2025_06_18 {
    // Version-specific behavior
}
```

### 3. Version Negotiation Protocol
```rust
// shadowcat/src/protocol/negotiation.rs
pub struct VersionNegotiator {
    registry: Arc<VersionRegistry>,
    minimum_version: ProtocolVersion, // "2025-03-26"
    default_fallback: ProtocolVersion, // "2025-03-26" per spec
}

impl VersionNegotiator {
    /// Extract version from initialize request params
    pub fn extract_initialize_version(&self, params: &Value) -> Option<String> {
        params.get("protocolVersion")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
    
    /// Negotiate version during initialize handshake
    pub async fn negotiate_initialize(
        &self,
        client_version: Option<String>,
        server_capabilities: &[String],
    ) -> Result<NegotiatedVersion> {
        let requested = client_version.unwrap_or_else(|| self.default_fallback.clone());
        
        if server_capabilities.contains(&requested) {
            return Ok(NegotiatedVersion::accepted(requested));
        }
        
        // Find best alternative version
        self.find_best_alternative(&requested, server_capabilities)
    }
    
    /// Validate HTTP header version matches negotiated version
    pub fn validate_http_version(
        &self,
        header_version: Option<&str>,
        negotiated_version: &str,
    ) -> Result<()> {
        let header_v = header_version.unwrap_or("2025-03-26"); // Spec default
        
        if header_v != negotiated_version {
            return Err(VersionError::Mismatch {
                negotiated: negotiated_version.to_string(),
                header: header_v.to_string(),
            });
        }
        Ok(())
    }
}

pub struct NegotiatedVersion {
    version: ProtocolVersion,
    capabilities: VersionCapabilities,
    limitations: Vec<Limitation>,
    upgrade_available: Option<ProtocolVersion>,
    negotiation_method: NegotiationMethod, // Initialize-only vs Dual-channel
}

#[derive(Debug, Clone)]
pub enum NegotiationMethod {
    InitializeOnly,  // 2025-03-26 style
    DualChannel,     // 2025-06-18+ style with HTTP headers
}
```

### 4. Feature Capability Matrix
```rust
// shadowcat/src/protocol/capabilities.rs
#[derive(Debug, Clone)]
pub struct VersionCapabilities {
    // Transport capabilities
    supports_stdio: bool,
    supports_http: bool,
    supports_sse: bool,
    supports_websocket: bool,  // Future
    
    // Protocol features
    supports_batch_requests: bool,
    supports_cancellation: bool,
    supports_progress: bool,
    supports_streaming: bool,
    
    // Message types
    supported_methods: HashSet<String>,
    supported_notifications: HashSet<String>,
    
    // Security features
    supports_oauth2: bool,
    supports_pkce: bool,
    requires_session_id: bool,
    
    // Limits
    max_message_size: usize,
    max_batch_size: usize,
    timeout_ms: u64,
}
```

### 5. Version-Aware Transport Layer
```rust
// shadowcat/src/transport/versioned.rs
pub struct VersionedTransport {
    version: ProtocolVersion,
    inner: Box<dyn Transport>,
    transformer: Box<dyn MessageTransformer>,
    negotiation_method: NegotiationMethod,
}

impl VersionedTransport {
    /// Handle version based on transport type
    pub fn resolve_version(&self, context: &TransportContext) -> Result<ProtocolVersion> {
        match (&self.inner.transport_type(), &self.negotiation_method) {
            (TransportType::Stdio, _) => {
                // stdio always uses initialize-only negotiation
                self.extract_from_initialize(context)
            }
            (TransportType::Http, NegotiationMethod::DualChannel) => {
                // HTTP with 2025-06-18+ uses both initialize and headers
                self.validate_dual_channel(context)
            }
            (TransportType::Http, NegotiationMethod::InitializeOnly) => {
                // HTTP with 2025-03-26 uses initialize only
                self.extract_from_initialize(context)
            }
        }
    }
}

impl Transport for VersionedTransport {
    async fn send(&mut self, msg: TransportMessage) -> Result<()> {
        // Transform message to version-specific format
        let versioned_msg = self.transformer.transform_outgoing(msg, &self.version)?;
        self.inner.send(versioned_msg).await
    }
    
    async fn receive(&mut self) -> Result<TransportMessage> {
        let msg = self.inner.receive().await?;
        // Transform to internal canonical format
        self.transformer.transform_incoming(msg, &self.version)
    }
}
```

### 6. Message Transformation Pipeline
```rust
// shadowcat/src/protocol/transform.rs
pub trait MessageTransformer {
    fn transform_outgoing(&self, msg: TransportMessage, target: &ProtocolVersion) -> Result<TransportMessage>;
    fn transform_incoming(&self, msg: TransportMessage, source: &ProtocolVersion) -> Result<TransportMessage>;
}

pub struct TransformationPipeline {
    transformers: Vec<Box<dyn MessageTransformer>>,
}

// Example transformers:
pub struct FieldRenameTransformer;  // Handle field name changes
pub struct MethodMappingTransformer;  // Map method names between versions
pub struct ParameterAdapter;  // Adapt parameter formats
pub struct ErrorCodeMapper;  // Map error codes between versions
```

### 7. Version-Specific Testing Framework
```rust
// shadowcat/tests/version_compliance/
mod test_framework {
    pub trait VersionTest {
        fn supported_versions(&self) -> Vec<ProtocolVersion>;
        fn run_for_version(&self, version: &ProtocolVersion) -> TestResult;
    }
    
    pub struct ComplianceTestSuite {
        tests: Vec<Box<dyn VersionTest>>,
    }
    
    impl ComplianceTestSuite {
        pub fn run_all_versions(&self) -> HashMap<ProtocolVersion, TestResults> {
            // Run tests against all supported versions
        }
    }
}
```

## Critical Fixes Required First

### Immediate Fixes (Before Multi-Version)
1. **Extract protocolVersion from initialize**: Parse params field
2. **Implement negotiation response**: Offer alternative versions
3. **Fix default version**: Use "2025-03-26" when no header
4. **Add version state tracking**: Track requested vs negotiated

## Implementation Strategy

### Phase 1: Fix Current Implementation (Days 1-2)
1. Fix version extraction from initialize messages
2. Add proper negotiation logic in forward proxy
3. Correct default version to "2025-03-26"
4. Implement version state machine

### Phase 2: Refactor for Multi-Version (Week 1)
1. Extract version-specific code into modules
2. Create ProtocolVersion enum and registry
3. Implement version detection for both channels
4. Add version to all relevant structures

### Phase 2: Add Version Negotiation (Week 2)
1. Implement negotiation protocol
2. Add capability detection
3. Create version selection logic
4. Add negotiation to connection establishment

### Phase 3: Multi-Version Support (Week 3-4)
1. Implement 2025-06-18 version module
2. Keep 2025-11-05 version module
3. Create transformation pipeline
4. Add version-aware routing

### Phase 4: Testing & Validation (Week 5)
1. Create version-specific test suites
2. Add cross-version compatibility tests
3. Performance benchmarks per version
4. Documentation for each version

## Configuration Schema
```toml
# shadowcat.toml
[protocol]
default_version = "2025-06-18"
supported_versions = ["2025-06-18", "2025-11-05"]
version_selection_policy = "prefer_latest"  # or "prefer_stable", "strict"

[protocol.versions."2025-06-18"]
enabled = true
priority = 100
features = ["sse", "batch_requests"]

[protocol.versions."2025-11-05"]
enabled = true
priority = 90
deprecated = false
deprecation_date = "2026-01-01"

[protocol.compatibility]
allow_version_downgrade = true
warn_on_version_mismatch = true
require_explicit_version = false
```

## Migration Patterns

### Client Version Upgrade
```rust
// Detect client version upgrade opportunity
if negotiated.upgrade_available.is_some() {
    session.send_notification(Notification::VersionUpgradeAvailable {
        current: negotiated.version.clone(),
        available: negotiated.upgrade_available.clone(),
        benefits: get_upgrade_benefits(&negotiated),
    }).await?;
}
```

### Gradual Feature Rollout
```rust
// Feature flags per version
pub struct FeatureFlags {
    version_gates: HashMap<Feature, MinimumVersion>,
}

impl FeatureFlags {
    pub fn is_enabled(&self, feature: Feature, version: &ProtocolVersion) -> bool {
        self.version_gates.get(&feature)
            .map(|min| version >= min)
            .unwrap_or(false)
    }
}
```

## Version Compatibility Matrix

| Client Version | Server Version | Negotiation Method | Compatibility | Notes |
|---------------|---------------|-------------------|---------------|-------|
| 2025-03-26 | 2025-03-26 | Initialize-only | ✅ Full | Original spec |
| 2025-03-26 | 2025-06-18 | Initialize-only | ⚠️ Partial | Server adapts |
| 2025-06-18 | 2025-03-26 | Dual-channel | ⚠️ Partial | Fallback mode |
| 2025-06-18 | 2025-06-18 | Dual-channel | ✅ Full | Full HTTP headers |
| 2025-06-18 | 2025-11-05 | Dual-channel | ⚠️ Partial | Via transformation |
| 2025-11-05 | 2025-06-18 | Dual-channel | ⚠️ Partial | Feature limitations |
| 2025-11-05 | 2025-11-05 | Dual-channel | ✅ Full | Native support |
| Future | Any | Negotiated | ⚠️ Negotiated | Fallback to common |

## Benefits of This Architecture

1. **Future-Proof**: New versions added without breaking existing code
2. **Backward Compatible**: Older clients continue working
3. **Testable**: Each version tested independently
4. **Maintainable**: Version-specific code isolated
5. **Performant**: No runtime overhead for single-version use
6. **Debuggable**: Clear version boundaries and transformations

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Complexity increase | High | Modular design, clear interfaces |
| Testing burden | Medium | Automated version matrix testing |
| Performance overhead | Low | Lazy loading, compile-time optimization |
| Version proliferation | Medium | Deprecation policy, version limits |
| Transformation errors | High | Comprehensive transformation tests |

## Success Metrics

- Support at least 3 protocol versions simultaneously
- < 1ms version negotiation time
- Zero version-related production incidents
- 100% backward compatibility maintained
- < 5% performance overhead for version handling

## Security Considerations

### Version Downgrade Prevention
```rust
pub struct VersionSecurity {
    minimum_allowed: ProtocolVersion,
    allow_downgrade: bool,
    require_explicit_version: bool,
}

impl VersionSecurity {
    pub fn validate_negotiation(
        &self,
        requested: &ProtocolVersion,
        negotiated: &ProtocolVersion,
    ) -> Result<()> {
        if negotiated < &self.minimum_allowed {
            return Err(SecurityError::VersionTooOld);
        }
        if !self.allow_downgrade && negotiated < requested {
            return Err(SecurityError::DowngradeAttempt);
        }
        Ok(())
    }
}
```

## Long-Term Considerations

### Version Sunset Policy
- Minimum support: 2025-03-26 (initialize-only negotiation)
- Minimum 6-month deprecation notice
- Telemetry on version usage
- Clear migration guides
- Automated migration tools where possible

### Standards Alignment
- Track MCP specification evolution
- Participate in specification discussions
- Maintain reference implementation status
- Contribute compatibility tests upstream

### Extension Points
- Plugin system for custom versions
- Runtime version loading (for development)
- Version-specific middlewares
- Custom transformation rules

## Next Steps

1. **URGENT**: Fix critical version negotiation bugs:
   - Extract protocolVersion from initialize params
   - Implement proper negotiation response
   - Fix default version to "2025-03-26"
   - Add dual-source version validation

2. **Immediate**: Refactor current code to extract version-specific logic
3. **Short-term**: Implement version registry and negotiation state machine
4. **Medium-term**: Add full multi-version support with transformations
5. **Long-term**: Build comprehensive version testing framework

## Testing Requirements

### Version Negotiation Tests
```rust
#[cfg(test)]
mod version_tests {
    #[test]
    fn test_initialize_version_extraction() {
        // Test extracting protocolVersion from params
    }
    
    #[test]
    fn test_version_negotiation_response() {
        // Test offering alternative versions
    }
    
    #[test]
    fn test_http_header_default() {
        // Test defaulting to 2025-03-26 when no header
    }
    
    #[test]
    fn test_dual_channel_consistency() {
        // Test initialize version matches HTTP header
    }
    
    #[test]
    fn test_version_downgrade_prevention() {
        // Test security against version downgrade attacks
    }
}
```

## Conclusion

Multi-version support is not just a nice-to-have but a critical requirement for Shadowcat's long-term viability as an MCP proxy. The proposed architecture provides a robust foundation for handling protocol evolution while maintaining backward compatibility and performance targets.

The investment in proper version handling infrastructure will pay dividends as the MCP specification evolves and as Shadowcat needs to bridge between different versions in production environments.