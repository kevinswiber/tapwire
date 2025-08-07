# MCP Version Negotiation Analysis: 2025-03-26 vs 2025-06-18

## Executive Summary

This document analyzes the critical differences in version negotiation mechanisms between MCP specification versions 2025-03-26 and 2025-06-18, examining how these changes impact our multi-version Shadowcat architecture. **Critical design flaws have been identified** that require immediate attention.

## Version Negotiation Mechanisms

### 2025-03-26 Specification: Initialize-Only Negotiation

The 2025-03-26 specification handles version negotiation **exclusively through the JSON-RPC initialize handshake**:

#### Initialize Request (Client → Server):
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-03-26",
    "capabilities": {
      "roots": { "listChanged": true },
      "sampling": {}
    },
    "clientInfo": {
      "name": "ExampleClient",
      "version": "1.0.0"
    }
  }
}
```

#### Initialize Response (Server → Client):
```json
{
  "jsonrpc": "2.0", 
  "id": 1,
  "result": {
    "protocolVersion": "2025-03-26",
    "capabilities": {
      "logging": {},
      "prompts": { "listChanged": true },
      "resources": { "subscribe": true, "listChanged": true },
      "tools": { "listChanged": true }
    },
    "serverInfo": {
      "name": "ExampleServer",
      "version": "1.0.0"
    }
  }
}
```

#### Version Negotiation Rules (2025-03-26):
> "In the `initialize` request, the client **MUST** send a protocol version it supports. This **SHOULD** be the _latest_ version supported by the client.
>
> If the server supports the requested protocol version, it **MUST** respond with the same version. Otherwise, the server **MUST** respond with another protocol version it supports. This **SHOULD** be the _latest_ version supported by the server.
>
> If the client does not support the version in the server's response, it **SHOULD** disconnect."

**Key Point**: No transport-level version headers exist in 2025-03-26.

### 2025-06-18 Specification: Initialize + HTTP Header System

The 2025-06-18 specification introduces **dual-layer version negotiation**:

#### 1. JSON-RPC Initialize Handshake (Same as 2025-03-26)
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize", 
  "params": {
    "protocolVersion": "2024-11-05",  // NOTE: Example uses older version
    "capabilities": {
      "roots": { "listChanged": true },
      "sampling": {},
      "elicitation": {}  // NEW capability
    },
    "clientInfo": {
      "name": "ExampleClient",
      "title": "Example Client Display Name",  // NEW field
      "version": "1.0.0"
    }
  }
}
```

#### 2. HTTP Protocol Version Header (NEW)
> "If using HTTP, the client **MUST** include the `MCP-Protocol-Version: <protocol-version>` HTTP header on all subsequent requests to the MCP server."

**Example**: `MCP-Protocol-Version: 2025-06-18`

#### Critical Implementation Note from 2025-06-18:
> **Note**  
> If using HTTP, the client **MUST** include the `MCP-Protocol-Version: <protocol-version>` HTTP header on all subsequent requests to the MCP server.  
> For details, see [the Protocol Version Header section in Transports](/specification/2025-06-18/basic/transports#protocol-version-header).

#### Protocol Version Header Rules (2025-06-18):
> "The protocol version sent by the client **SHOULD** be the one [negotiated during initialization](/specification/2025-06-18/basic/lifecycle#version-negotiation).
>
> For backwards compatibility, if the server does _not_ receive an `MCP-Protocol-Version` header, and has no other way to identify the version - for example, by relying on the protocol version negotiated during initialization - the server **SHOULD** assume protocol version `2025-03-26`.
>
> If the server receives a request with an invalid or unsupported `MCP-Protocol-Version`, it **MUST** respond with `400 Bad Request`."

## Critical Design Flaws Identified

### 1. **Default Version Assumption Mismatch**
The 2025-06-18 specification states that servers should assume `2025-03-26` when no header is present, but our Shadowcat implementation defaults to `2025-11-05`:

```rust
// In /Users/kevin/src/tapwire/shadowcat/src/transport/mod.rs
pub const MCP_PROTOCOL_VERSION: &str = "2025-11-05";

// In /Users/kevin/src/tapwire/shadowcat/src/transport/http_mcp.rs  
let protocol_version = headers
    .get("mcp-protocol-version")
    .and_then(|v| v.to_str().ok())
    .unwrap_or(MCP_PROTOCOL_VERSION);  // ❌ WRONG: Should be "2025-03-26" per spec
```

**Impact**: Non-compliant behavior that breaks compatibility with 2025-06-18 spec clients.

### 2. **Missing Initialize Message Version Extraction**
Our current implementation only checks HTTP headers but **completely ignores version information in initialize messages**:

```rust
// Current session manager only looks at method name, not params
fn is_initialize_request(&self, message: &TransportMessage) -> bool {
    matches!(message, TransportMessage::Request { method, .. } if method == "initialize")
}
```

**Critical Gap**: No parsing of `protocolVersion` field from initialize requests/responses.

### 3. **Version Negotiation Logic Missing**
The specification requires active version negotiation:
- Server should respond with different version if requested version unsupported
- Client should disconnect if server's version is unsupported  

**Current State**: We only validate supported versions but don't implement proper negotiation.

### 4. **Inconsistent Version Handling Across Transports**
```rust
// HTTP Transport sends headers
.header("MCP-Protocol-Version", MCP_PROTOCOL_VERSION)

// But stdio transport has no version communication mechanism
// Relies entirely on initialize message exchange
```

**Issue**: Different version handling behavior across transport types.

## Current Shadowcat Implementation Analysis

### Version Constants
```rust
// /Users/kevin/src/tapwire/shadowcat/src/transport/mod.rs:25
pub const MCP_PROTOCOL_VERSION: &str = "2025-11-05";

// /Users/kevin/src/tapwire/shadowcat/src/transport/http_mcp.rs:15-18
const SUPPORTED_VERSIONS: &[&str] = &[
    "2025-11-05", // Current shadowcat version  
    "2025-06-18", // Streamable HTTP version
];
```

### Header Processing
```rust
// /Users/kevin/src/tapwire/shadowcat/src/transport/http_mcp.rs:75-79
let protocol_version = headers
    .get("mcp-protocol-version") 
    .and_then(|v| v.to_str().ok())
    .unwrap_or(MCP_PROTOCOL_VERSION); // ❌ Should default to "2025-03-26"
```

### Version Validation
```rust
// /Users/kevin/src/tapwire/shadowcat/src/transport/http_mcp.rs:81-87
if !is_version_supported(protocol_version) {
    warn!("Unsupported protocol version: {}", protocol_version);
    return Err(ReverseProxyError::ProtocolVersionMismatch {
        expected: SUPPORTED_VERSIONS[0].to_string(),
        actual: protocol_version.to_string(),
    });
}
```

**Problem**: Rejects instead of negotiating alternative version.

## Migration Challenges

### 1. **Dual Version Sources**
2025-06-18 introduces two sources of version information:
- Initialize message `protocolVersion` field
- HTTP `MCP-Protocol-Version` header

**Challenge**: Which takes precedence? What if they conflict?

### 2. **Backwards Compatibility Requirements**  
Must support:
- Pure 2025-03-26 clients (initialize-only)
- Pure 2025-06-18 clients (initialize + headers)
- Mixed scenarios during transition

### 3. **Transport-Specific Behavior**
- **stdio**: Only initialize message version (both specs)
- **HTTP**: Initialize + headers in 2025-06-18, initialize-only in 2025-03-26

### 4. **Version Negotiation State Machine**
Need to track:
- Requested version (from initialize)
- Negotiated version (agreed upon)  
- Transport version (from headers)
- Session consistency

## Specific Quotes from Specifications

### 2025-03-26 Version Negotiation:
> "In the `initialize` request, the client **MUST** send a protocol version it supports. This **SHOULD** be the _latest_ version supported by the client. If the server supports the requested protocol version, it **MUST** respond with the same version. Otherwise, the server **MUST** respond with another protocol version it supports."

### 2025-06-18 Additional HTTP Requirements:
> "If using HTTP, the client **MUST** include the `MCP-Protocol-Version: <protocol-version>` HTTP header on all subsequent requests to the MCP server, allowing the MCP server to respond based on the MCP protocol version."

> "For backwards compatibility, if the server does _not_ receive an `MCP-Protocol-Version` header, and has no other way to identify the version - for example, by relying on the protocol version negotiated during initialization - the server **SHOULD** assume protocol version `2025-03-26`."

## Recommendations for Multi-Version Architecture

### 1. **Fix Default Version Assumption**
```rust
// Change default from 2025-11-05 to 2025-03-26 per spec
let protocol_version = headers
    .get("mcp-protocol-version")
    .and_then(|v| v.to_str().ok()) 
    .unwrap_or("2025-03-26"); // ✅ Spec-compliant default
```

### 2. **Implement Initialize Message Version Parsing**
```rust
fn extract_protocol_version_from_initialize(message: &TransportMessage) -> Option<String> {
    if let TransportMessage::Request { method: "initialize", params } = message {
        params.get("protocolVersion")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    } else {
        None
    }
}
```

### 3. **Add Version Negotiation Logic**
```rust
fn negotiate_version(requested: &str, supported: &[&str]) -> Result<String, VersionError> {
    if supported.contains(&requested) {
        Ok(requested.to_string())
    } else {
        // Return highest supported version
        Ok(supported[0].to_string()) 
    }
}
```

### 4. **Track Version State Per Session**
```rust
struct SessionVersionInfo {
    initialize_version: Option<String>,    // From JSON-RPC initialize
    transport_version: Option<String>,     // From HTTP headers  
    negotiated_version: String,            // Final agreed version
    transport_type: TransportType,
}
```

### 5. **Version Consistency Validation**
Ensure initialize and HTTP header versions match (when both present).

## Edge Cases and Compatibility Issues

### 1. **Version Mismatch Between Initialize and Headers**
Client sends:
- Initialize: `"protocolVersion": "2025-03-26"`  
- HTTP Header: `MCP-Protocol-Version: 2025-06-18`

**Resolution**: Headers should match negotiated version from initialize.

### 2. **Legacy Client Support**  
2025-03-26 clients won't send HTTP version headers.
**Must**: Default to 2025-03-26 and rely on initialize version only.

### 3. **Version Upgrade During Session**
**Forbidden**: Version cannot change after initialize handshake completes.

### 4. **Unsupported Version Handling**
Current: Immediate rejection  
**Should**: Offer alternative version in initialize response

## Impact on Shadowcat Multi-Version Support

### Current Architecture Issues
1. **Hardcoded version assumptions**
2. **Missing initialize message parsing**  
3. **No version negotiation logic**
4. **Non-spec-compliant defaults**

### Required Changes
1. **Version parsing from both sources**
2. **Proper negotiation state machine**
3. **Spec-compliant backwards compatibility**
4. **Transport-aware version handling**

## Conclusion

The introduction of HTTP version headers in 2025-06-18 creates a **dual-channel version system** that significantly complicates our multi-version architecture. The current Shadowcat implementation has **critical compliance gaps** that must be addressed:

1. **Wrong default version** (2025-11-05 instead of 2025-03-26)
2. **Missing initialize message version parsing**  
3. **No version negotiation logic**
4. **Inconsistent cross-transport behavior**

These issues will cause **compatibility failures** with standard MCP clients and servers. Immediate remediation is required before production deployment.

The complexity introduced by having version information in both initialize messages AND HTTP headers necessitates careful state management and adds significant implementation burden to proxy systems like Shadowcat that must handle multiple protocol versions simultaneously.