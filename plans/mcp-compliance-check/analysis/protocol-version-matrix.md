# MCP Protocol Version Comparison Matrix

## Overview

This document provides a comprehensive comparison of MCP protocol versions, highlighting key differences and migration considerations for compliance testing.

## Version Timeline

| Version | Release Date | Status | Key Focus |
|---------|-------------|--------|-----------|
| 2024-11-05 | November 2024 | Stable | Initial protocol release |
| 2025-03-26 | March 2025 | Current Stable | Async operations, streamable HTTP |
| 2025-06-18 | June 2025 | Latest | Enhanced validation, structured output |
| draft | Ongoing | Experimental | Future features |

## Feature Comparison Matrix

| Feature | 2024-11-05 | 2025-03-26 | 2025-06-18 |
|---------|------------|------------|------------|
| **Core Protocol** |
| JSON-RPC 2.0 | ✅ | ✅ | ✅ |
| Batch Requests | ✅ | ✅ | ❌ Removed |
| Protocol Negotiation | ✅ | ✅ Enhanced | ✅ Enhanced |
| **Transports** |
| stdio | ✅ | ✅ | ✅ |
| HTTP+SSE | ✅ | ❌ Deprecated | ❌ |
| Streamable HTTP | ❌ | ✅ New | ✅ |
| **Capabilities Format** |
| Boolean Values | ✅ | ❌ | ❌ |
| Object Values | ❌ | ✅ | ✅ |
| Experimental | ❌ | ✅ | ✅ |
| **Tools** |
| Basic Tools | ✅ | ✅ | ✅ |
| Async Tools | ❌ | ✅ New | ✅ |
| Tool Cancellation | ❌ | ✅ | ✅ Enhanced |
| Structured Output | ❌ | ❌ | ✅ New |
| Output Schema | ❌ | ❌ | ✅ New |
| **Resources** |
| Basic Resources | ✅ | ✅ | ✅ |
| Subscriptions | ✅ | ✅ | ✅ |
| Resource Metadata | ❌ | ❌ | ✅ New |
| URI Templates | ❌ | ❌ | ✅ New |
| **Prompts** |
| Basic Prompts | ✅ | ✅ | ✅ |
| Argument Schemas | ✅ | ✅ | ✅ Enhanced |
| Elicitation | ❌ | ❌ | ✅ New |
| **Error Handling** |
| Standard Codes | ✅ | ✅ | ✅ |
| Extended Codes | ❌ | ✅ | ✅ |
| Error Data Field | Optional | Recommended | Required |
| **Session Management** |
| Session IDs | ✅ | ✅ | ✅ |
| Session Persistence | Basic | Enhanced | Enhanced |
| Session Metadata | ❌ | ✅ | ✅ |

## Breaking Changes by Version

### 2024-11-05 → 2025-03-26

#### Transport Changes
- **HTTP+SSE deprecated** in favor of Streamable HTTP
- New endpoint structure for HTTP transport
- SSE stream management changes

#### Capability Format
```json
// 2024-11-05 (Old)
{
  "capabilities": {
    "tools": true,
    "resources": true
  }
}

// 2025-03-26 (New)
{
  "capabilities": {
    "tools": {
      "listChanged": true
    },
    "resources": {
      "subscribe": true,
      "listChanged": true
    }
  }
}
```

#### New Features
- Async tool operations with dedicated lifecycle
- Progress notifications for long-running operations
- Enhanced cancellation support

### 2025-03-26 → 2025-06-18

#### Batch Request Removal
- JSON-RPC batch requests no longer supported
- Each request must be sent individually
- Affects performance optimization strategies

#### Structured Tool Output
```json
// New in 2025-06-18
{
  "name": "analyze",
  "outputSchema": {
    "type": "object",
    "properties": {
      "summary": {"type": "string"},
      "score": {"type": "number"}
    }
  }
}
```

#### Elicitation Support
- New capability for requesting user information
- Interactive prompts during tool execution
- Context preservation across elicitation

## Migration Guidelines

### From 2024-11-05 to 2025-03-26

1. **Update Transport Layer**
   - Replace HTTP+SSE with Streamable HTTP
   - Update SSE event handling
   - Implement new session management

2. **Migrate Capabilities**
   - Convert boolean to object format
   - Add sub-capabilities (listChanged, subscribe)
   - Update capability negotiation logic

3. **Add Async Support**
   - Implement async tool handlers
   - Add progress notification support
   - Handle cancellation requests

### From 2025-03-26 to 2025-06-18

1. **Remove Batch Support**
   - Refactor batch request logic
   - Implement sequential request handling
   - Update performance optimizations

2. **Add Structured Output**
   - Implement outputSchema validation
   - Add structuredContent field
   - Update tool response handling

3. **Implement Elicitation**
   - Add elicitation capability
   - Handle user information requests
   - Maintain context across interactions

## Compliance Testing Implications

### Version Detection
```rust
pub enum ProtocolVersion {
    V2024_11_05,
    V2025_03_26,
    V2025_06_18,
    Draft,
}

impl ProtocolVersion {
    pub fn supports_async_tools(&self) -> bool {
        matches!(self, Self::V2025_03_26 | Self::V2025_06_18)
    }
    
    pub fn supports_batch_requests(&self) -> bool {
        matches!(self, Self::V2024_11_05 | Self::V2025_03_26)
    }
    
    pub fn supports_structured_output(&self) -> bool {
        matches!(self, Self::V2025_06_18)
    }
}
```

### Test Filtering
```rust
#[test]
#[version(min = "2025-03-26")]
fn test_async_tool_operation() {
    // Only runs for 2025-03-26 and later
}

#[test]
#[version(max = "2025-03-26")]
fn test_batch_requests() {
    // Only runs for 2025-03-26 and earlier
}

#[test]
#[version(exact = "2025-06-18")]
fn test_elicitation() {
    // Only runs for 2025-06-18
}
```

### Capability Adaptation
```rust
fn adapt_capabilities(version: ProtocolVersion, caps: Value) -> Value {
    match version {
        ProtocolVersion::V2024_11_05 => {
            // Convert object to boolean format
            convert_to_boolean_capabilities(caps)
        }
        ProtocolVersion::V2025_03_26 |
        ProtocolVersion::V2025_06_18 => {
            // Use object format as-is
            caps
        }
    }
}
```

## Backward Compatibility

### Principles
1. **Version Negotiation**: Always attempt highest common version
2. **Graceful Degradation**: Fall back to older features when needed
3. **Feature Detection**: Check capabilities before using features
4. **Clear Errors**: Provide version-specific error messages

### Implementation Strategy
```rust
pub struct ProtocolAdapter {
    version: ProtocolVersion,
    capabilities: Capabilities,
}

impl ProtocolAdapter {
    pub async fn call_tool(&self, name: &str, args: Value) -> Result<Value> {
        if self.version.supports_async_tools() {
            self.call_tool_async(name, args).await
        } else {
            self.call_tool_sync(name, args).await
        }
    }
}
```

## Test Coverage Requirements

### Version-Specific Tests

| Version | Required Tests | Optional Tests | Total |
|---------|---------------|----------------|-------|
| 2024-11-05 | 35 | 10 | 45 |
| 2025-03-26 | 42 | 12 | 54 |
| 2025-06-18 | 48 | 15 | 63 |

### Cross-Version Tests
- Protocol negotiation (all version pairs)
- Capability format conversion
- Error handling consistency
- Transport compatibility

## Recommendations

1. **Default to 2025-03-26** for new implementations
2. **Support 2024-11-05** for legacy compatibility
3. **Prepare for 2025-06-18** features
4. **Monitor draft** for upcoming changes
5. **Test against all versions** in CI/CD

## Summary

The MCP protocol has evolved significantly across versions, with each iteration adding new capabilities while maintaining core compatibility. Our compliance framework must:

1. Support all stable versions (2024-11-05, 2025-03-26, 2025-06-18)
2. Adapt tests based on version capabilities
3. Provide clear migration paths
4. Maintain backward compatibility
5. Prepare for future changes

---

*Generated: 2025-08-23*
*Source: MCP Specifications Analysis*
*Purpose: Compliance Framework Design*