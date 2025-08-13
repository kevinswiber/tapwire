# MCP Specification Gap Analysis

Based on MCP Protocol Revision: 2025-06-18  
Analysis Date: 2025-01-07  
Shadowcat Version: Current implementation in `/Users/kevin/src/tapwire/shadowcat/src/`

## Executive Summary

Shadowcat has strong compliance with the stdio transport and basic JSON-RPC requirements, but has **critical gaps** in Streamable HTTP transport support, particularly missing complete Server-Sent Events (SSE) implementation. The implementation appears to target an older protocol version (2025-11-05) rather than the current spec (2025-06-18).

## Critical Gaps (MUST Fix)

### 1. Server-Sent Events (SSE) Implementation - CRITICAL
**Severity: Critical**  
**Spec References**: Transport Layer, Section 2.2.1-2.2.6

**Current State**: Partial implementation  
**Code References**:
- `/Users/kevin/src/tapwire/shadowcat/src/transport/http.rs:148-217` - Basic SSE connection attempt
- `/Users/kevin/src/tapwire/shadowcat/src/transport/http.rs:123` - SSE content-type detection

**Missing Components**:
- Complete SSE event parsing and formatting
- SSE event ID support for resumability  
- Last-Event-ID header processing
- SSE connection lifecycle management
- SSE error handling and reconnection logic
- Multiple concurrent SSE stream support

**Impact**: Cannot comply with Streamable HTTP transport specification

### 2. Protocol Version Mismatch - CRITICAL  
**Severity: Critical**
**Spec References**: All sections

**Current State**: Targeting wrong version  
**Code References**:
- `/Users/kevin/src/tapwire/shadowcat/src/transport/mod.rs:25` - `MCP_PROTOCOL_VERSION: "2025-11-05"`
- `/Users/kevin/src/tapwire/shadowcat/src/transport/http_mcp.rs:17` - Version in supported list

**Gap**: Implementation targets `2025-11-05` but current spec is `2025-06-18`

**Impact**: Version negotiation will fail with spec-compliant clients

### 3. HTTP Transport Session Management - CRITICAL
**Severity: Critical**  
**Spec References**: Transport Layer, Section 2.2.5

**Current State**: Basic session ID support but incomplete HTTP session lifecycle  
**Code References**:
- `/Users/kevin/src/tapwire/shadowcat/src/transport/http_mcp.rs:28-47` - Session ID generation
- `/Users/kevin/src/tapwire/shadowcat/src/transport/http_mcp.rs:57-100` - Header extraction

**Missing Components**:
- Session ID assignment during initialization response
- Mcp-Session-Id header requirement enforcement
- 400 Bad Request for missing session IDs
- 404 Not Found for expired sessions
- DELETE method support for session termination

**Impact**: HTTP clients cannot maintain stateful sessions

## Major Gaps (SHOULD Fix)

### 4. Complete Streamable HTTP Compliance - MAJOR
**Severity: Major**  
**Spec References**: Transport Layer, Section 2.2

**Current State**: Basic HTTP support with partial SSE  
**Code References**:
- `/Users/kevin/src/tapwire/shadowcat/src/transport/http.rs:74-145` - Streamable HTTP request handling
- `/Users/kevin/src/tapwire/shadowcat/src/transport/http.rs:366-409` - HTTP transport connection

**Missing Components**:
- Single endpoint supporting both POST and GET methods
- Proper 202 Accepted responses for notifications
- GET method support for opening SSE streams  
- 405 Method Not Allowed responses when appropriate
- Multiple connection management

**Impact**: Cannot serve as compliant HTTP MCP server

### 5. Security Requirements - MAJOR
**Severity: Major**  
**Spec References**: Transport Layer, Section 2.2 Security Warning

**Current State**: Missing security validations  
**Code References**: No Origin header validation found in transport layer

**Missing Components**:
- Origin header validation for DNS rebinding protection
- Localhost binding enforcement for local servers
- CORS policy implementation for SSE

**Impact**: Vulnerable to DNS rebinding and other web-based attacks

### 6. OAuth 2.1 Authorization Framework - MAJOR  
**Severity: Major**  
**Spec References**: Authorization section

**Current State**: Some OAuth infrastructure exists  
**Code References**:
- `/Users/kevin/src/tapwire/shadowcat/src/auth/oauth.rs` - OAuth implementation
- `/Users/kevin/src/tapwire/shadowcat/src/auth/pkce.rs` - PKCE support

**Missing Analysis**: Need detailed review of auth module compliance with:
- OAuth 2.1 (draft-ietf-oauth-v2-1-13)  
- RFC8414 (Authorization Server Metadata)
- RFC7591 (Dynamic Client Registration)
- RFC9728 (Protected Resource Metadata)

**Impact**: Cannot support protected MCP servers

## Moderate Gaps (MAY Fix)

### 7. Advanced SSE Features - MODERATE
**Severity: Moderate**  
**Spec References**: Transport Layer, Section 2.2.4

**Missing Components**:
- SSE resumability with event IDs
- Last-Event-ID header support for reconnection
- Message replay after disconnection
- Per-stream event ID uniqueness

### 8. Timeout and Error Handling - MODERATE  
**Severity: Moderate**  
**Spec References**: Lifecycle section

**Current State**: Basic timeout support  
**Code References**:
- `/Users/kevin/src/tapwire/shadowcat/src/transport/stdio.rs:310-314` - Send timeout
- `/Users/kevin/src/tapwire/shadowcat/src/transport/stdio.rs:329-334` - Receive timeout

**Missing Components**:
- Per-request timeout configuration  
- Cancellation notification on timeout
- Progress notification timeout reset
- Maximum timeout enforcement

## What's Currently Implemented ✅

### 1. stdio Transport Compliance - GOOD
**Code References**:
- `/Users/kevin/src/tapwire/shadowcat/src/transport/stdio.rs` - Complete implementation

**Implemented Features**:
- ✅ JSON-RPC message parsing and serialization
- ✅ Newline delimited message format
- ✅ stdin/stdout communication
- ✅ stderr logging support
- ✅ Process lifecycle management
- ✅ Message size limits
- ✅ Timeout handling

### 2. JSON-RPC Base Protocol - GOOD
**Code References**:
- `/Users/kevin/src/tapwire/shadowcat/src/transport/stdio.rs:158-257` - Message parsing
- `/Users/kevin/src/tapwire/shadowcat/src/transport/http_mcp.rs:249-441` - JSON-RPC handling

**Implemented Features**:
- ✅ JSON-RPC 2.0 format compliance
- ✅ Request/Response/Notification message types
- ✅ ID validation and handling
- ✅ Error response formatting
- ✅ Parameter handling

### 3. Basic Transport Abstraction - GOOD  
**Code References**:
- `/Users/kevin/src/tapwire/shadowcat/src/transport/mod.rs:154-173` - Transport trait

**Implemented Features**:
- ✅ Unified Transport trait
- ✅ Session ID management
- ✅ Multiple transport type support
- ✅ Connection lifecycle methods

### 4. Session Infrastructure - PARTIAL
**Code References**:
- `/Users/kevin/src/tapwire/shadowcat/src/session/` - Session management
- `/Users/kevin/src/tapwire/shadowcat/src/transport/http_mcp.rs:28-47` - Secure session ID generation

**Implemented Features**:
- ✅ Cryptographically secure session ID generation
- ✅ Session storage and management
- ✅ Session state tracking
- 🔶 Missing HTTP session lifecycle integration

## Implementation Recommendations

### Phase 1: Critical Fixes (Required for Compliance)
1. **Update Protocol Version** - Change to `2025-06-18` throughout codebase
2. **Complete SSE Implementation** - Full Server-Sent Events support
3. **HTTP Session Management** - Complete Streamable HTTP session lifecycle
4. **Security Headers** - Origin validation and DNS rebinding protection

### Phase 2: Major Compliance (High Priority)  
1. **Single Endpoint HTTP** - Unified POST/GET endpoint design
2. **Response Code Compliance** - Proper 202, 404, 405 responses
3. **Multiple Connection Support** - Concurrent SSE stream management

### Phase 3: Enhanced Features (Medium Priority)
1. **OAuth 2.1 Review** - Audit existing auth module for spec compliance  
2. **SSE Resumability** - Event ID and replay support
3. **Advanced Error Handling** - Comprehensive timeout and cancellation

## Files Requiring Changes

### High Priority:
- `/Users/kevin/src/tapwire/shadowcat/src/transport/mod.rs` - Protocol version
- `/Users/kevin/src/tapwire/shadowcat/src/transport/http.rs` - SSE implementation
- `/Users/kevin/src/tapwire/shadowcat/src/transport/http_mcp.rs` - Session management

### Medium Priority:
- `/Users/kevin/src/tapwire/shadowcat/src/proxy/reverse.rs` - HTTP proxy integration
- `/Users/kevin/src/tapwire/shadowcat/src/session/manager.rs` - Session lifecycle
- `/Users/kevin/src/tapwire/shadowcat/src/auth/*.rs` - OAuth compliance review

## Testing Requirements

**Critical Tests Needed**:
- SSE connection lifecycle
- Session management over HTTP
- Protocol version negotiation
- Security header validation
- Multi-stream SSE handling

**Existing Test Coverage**:
- ✅ stdio transport tests
- ✅ JSON-RPC parsing tests  
- ✅ Basic HTTP transport tests
- ✅ Session ID generation tests

## Risk Assessment

**High Risk** - Current implementation cannot support spec-compliant HTTP clients due to missing SSE and session management features. This limits Shadowcat's utility as an MCP proxy for HTTP-based servers.

**Medium Risk** - Security vulnerabilities from missing Origin validation could allow DNS rebinding attacks.

**Low Risk** - stdio transport compliance is strong, so file-based MCP servers should work well.

## Success Metrics

1. **SSE Compliance**: Pass all Server-Sent Events conformance tests
2. **Session Management**: Successful HTTP session lifecycle with major MCP clients
3. **Protocol Version**: Successful negotiation with `2025-06-18` spec clients
4. **Security**: Pass security audit for DNS rebinding protection
5. **Performance**: <5% latency overhead target maintained with full HTTP compliance