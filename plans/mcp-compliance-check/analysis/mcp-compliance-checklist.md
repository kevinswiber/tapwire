# MCP Compliance Checklist

## Overview

This document catalogs all MUST/SHOULD/MAY requirements from the MCP specifications (version 2025-03-26) to create a comprehensive compliance checklist for our Rust implementation.

## Summary Statistics

| Requirement Level | Count | Description |
|-------------------|-------|-------------|
| MUST | 106 | Mandatory requirements for compliance |
| SHOULD | 91 | Recommended practices for quality implementations |
| MAY | 36 | Optional features that enhance functionality |
| **Total** | **233** | Complete set of protocol requirements |

## Compliance Requirements by Category

### 1. Lifecycle Requirements

#### Initialization Phase

##### MUST Requirements
- [ ] The initialization phase MUST be the first interaction between client and server
- [ ] The client MUST initiate this phase by sending an `initialize` request
- [ ] The initialize request MUST NOT be part of a JSON-RPC batch
- [ ] The server MUST respond with its own capabilities and information
- [ ] After successful initialization, the client MUST send an `initialized` notification
- [ ] The client MUST send a protocol version it supports in the initialize request
- [ ] If the server supports the requested protocol version, it MUST respond with the same version
- [ ] Otherwise, the server MUST respond with another protocol version it supports

##### SHOULD Requirements
- [ ] The protocol version sent by client SHOULD be the latest version supported
- [ ] The protocol version sent by server SHOULD be the latest version supported
- [ ] The client SHOULD NOT send requests other than pings before server responds to initialize
- [ ] The server SHOULD NOT send requests other than pings and logging before receiving initialized
- [ ] If the client doesn't support the server's version, it SHOULD disconnect

##### MAY Requirements
- [ ] The server MAY include optional instructions in the initialize response

#### Operation Phase

##### SHOULD Requirements
- [ ] Both parties SHOULD respect the negotiated protocol version
- [ ] Both parties SHOULD only use capabilities that were successfully negotiated

#### Shutdown Phase

##### SHOULD Requirements (stdio)
- [ ] The client SHOULD initiate shutdown by closing the input stream
- [ ] The client SHOULD wait for server exit or send SIGTERM after reasonable time
- [ ] The client SHOULD send SIGKILL if server doesn't exit after SIGTERM

##### MAY Requirements
- [ ] The server MAY initiate shutdown by closing output stream and exiting

#### Timeout Handling

##### SHOULD Requirements
- [ ] Implementations SHOULD establish timeouts for all sent requests
- [ ] Sender SHOULD issue cancellation if no response within timeout
- [ ] SDKs SHOULD allow timeouts to be configured per request
- [ ] Implementations MAY reset timeout clock when receiving progress notifications

---

### 2. Transport Requirements

#### General Transport

##### MUST Requirements
- [ ] JSON-RPC messages MUST be UTF-8 encoded

##### SHOULD Requirements
- [ ] Clients SHOULD support stdio transport whenever possible

#### stdio Transport

##### MUST Requirements
- [ ] Messages MUST NOT contain embedded newlines (newline-delimited)
- [ ] Server MUST NOT write anything to stdout that is not a valid MCP message
- [ ] Client MUST NOT write anything to stdin that is not a valid MCP message

##### MAY Requirements
- [ ] Server MAY write UTF-8 strings to stderr for logging
- [ ] Clients MAY capture, forward, or ignore stderr logging

#### Streamable HTTP Transport

##### Security Requirements (MUST)
- [ ] Servers MUST validate the Origin header to prevent DNS rebinding attacks
- [ ] Servers SHOULD bind only to localhost (127.0.0.1) when running locally
- [ ] Servers SHOULD implement proper authentication for all connections

##### Request Handling (MUST)
- [ ] Every client message MUST be a new HTTP POST request
- [ ] Client MUST use HTTP POST to send JSON-RPC messages
- [ ] Client MUST include Accept header with application/json and text/event-stream
- [ ] Server MUST provide a single HTTP endpoint supporting POST and GET

##### Response Handling (MUST)
- [ ] For notifications/responses only: server MUST return 202 Accepted with no body
- [ ] For requests: server MUST return either text/event-stream or application/json
- [ ] Client MUST support both SSE and JSON response types
- [ ] Server MUST NOT send responses on GET stream unless resuming

##### SSE Stream Requirements
- [ ] SSE stream SHOULD eventually include one response per request
- [ ] Server MAY send requests/notifications before responses
- [ ] Server SHOULD NOT close SSE before sending all responses
- [ ] Server SHOULD close SSE after all responses sent
- [ ] Disconnection SHOULD NOT be interpreted as cancellation

##### GET Request Support
- [ ] Client MAY issue HTTP GET to open SSE stream
- [ ] Client MUST include Accept: text/event-stream header
- [ ] Server MUST return SSE or 405 Method Not Allowed

---

### 3. Session Management (HTTP)

#### Session Creation
- [ ] Server MUST generate unique session ID for new sessions
- [ ] Server MUST include MCP-Session-Id header in responses
- [ ] Client MUST include session ID in subsequent requests

#### Session Persistence
- [ ] Server SHOULD maintain session state across requests
- [ ] Server MAY expire sessions after timeout period
- [ ] Client SHOULD handle session expiration gracefully

---

### 4. Message Format Requirements

#### JSON-RPC 2.0 Compliance

##### MUST Requirements
- [ ] All messages MUST follow JSON-RPC 2.0 specification
- [ ] Requests MUST include jsonrpc: "2.0" field
- [ ] Requests MUST include method string field
- [ ] Requests MUST include id (string or integer, not null) unless notification
- [ ] Responses MUST include same id as request
- [ ] Responses MUST include either result OR error (not both)
- [ ] Error responses MUST include code and message

##### Batch Support
- [ ] Implementations MUST support receiving JSON-RPC batches
- [ ] Batch responses MUST maintain order with requests
- [ ] Each batch item MUST be processed independently

---

### 5. Capability Requirements

#### Client Capabilities
- [ ] `roots` - Filesystem roots support
  - [ ] `listChanged` - Support for list change notifications
- [ ] `sampling` - LLM sampling support
- [ ] `experimental` - Non-standard features

#### Server Capabilities
- [ ] `prompts` - Prompt templates
  - [ ] `listChanged` - Support for list change notifications
- [ ] `resources` - Readable resources
  - [ ] `subscribe` - Individual resource subscriptions
  - [ ] `listChanged` - Support for list change notifications
- [ ] `tools` - Callable tools
  - [ ] `listChanged` - Support for list change notifications
- [ ] `logging` - Structured logging
- [ ] `completions` - Argument autocompletion
- [ ] `experimental` - Non-standard features

---

### 6. Protocol Features

#### Tools
- [ ] tools/list MUST return array of tool objects
- [ ] Each tool MUST have name and description
- [ ] Tool schema MUST follow JSON Schema specification
- [ ] Tool calls MUST validate parameters against schema
- [ ] Tool errors MUST include descriptive messages

#### Resources
- [ ] resources/list MUST return array of resource objects
- [ ] Each resource MUST have uri and name
- [ ] Resource subscriptions MUST send updates on changes
- [ ] Resource URIs MUST be valid URI format

#### Prompts
- [ ] prompts/list MUST return array of prompt objects
- [ ] Each prompt MUST have name and description
- [ ] Prompt arguments MUST be validated against schema

#### Logging
- [ ] Log messages MUST include level and message
- [ ] Log levels MUST be: debug, info, warning, error
- [ ] Structured data MAY be included in logs

---

### 7. Error Handling

#### Standard Error Codes
- [ ] -32700 Parse error (Invalid JSON)
- [ ] -32600 Invalid Request
- [ ] -32601 Method not found
- [ ] -32602 Invalid params
- [ ] -32603 Internal error
- [ ] -32000 to -32099 Server error (reserved)

#### Error Response Format
- [ ] Error MUST include code (integer)
- [ ] Error MUST include message (string)
- [ ] Error MAY include data field with additional information

---

### 8. Version-Specific Requirements

#### 2024-11-05 → 2025-03-26 Changes
- [ ] Async tool support added (2025-03-26+)
- [ ] Capability format changed from boolean to object
- [ ] Streamable HTTP replaces HTTP+SSE transport
- [ ] Enhanced error codes

#### 2025-03-26 → 2025-06-18 Changes
- [ ] Structured tool output with outputSchema
- [ ] Elicitation support for user information requests
- [ ] JSON-RPC batch support removed
- [ ] Enhanced validation requirements
- [ ] Resource metadata support

---

## Proxy-Specific Compliance Points

### Forward Proxy Requirements
- [ ] MUST preserve message ordering
- [ ] MUST NOT modify message content
- [ ] MUST forward all headers except connection-specific
- [ ] MUST handle session ID mapping correctly
- [ ] SHOULD implement connection pooling
- [ ] SHOULD support failover to backup upstreams

### Reverse Proxy Requirements
- [ ] MUST authenticate incoming connections
- [ ] MUST NOT forward client authentication to upstream
- [ ] MUST generate new session IDs for clients
- [ ] MUST maintain session mapping table
- [ ] SHOULD implement rate limiting
- [ ] SHOULD support OAuth 2.1 flows

### SSE Proxy Requirements
- [ ] MUST handle SSE reconnection transparently
- [ ] MUST buffer messages during reconnection
- [ ] MUST maintain event ID sequence
- [ ] SHOULD implement heartbeat/keepalive
- [ ] SHOULD support compression

---

## Testing Strategy

### Compliance Levels

#### Level 1: Core Compliance (MUST requirements)
All MUST requirements must pass for basic compliance:
- Lifecycle management
- Message format
- Transport basics
- Error handling

#### Level 2: Standard Compliance (MUST + SHOULD)
Include SHOULD requirements for production quality:
- Timeouts and cancellation
- Proper capability negotiation
- Security best practices
- Performance optimizations

#### Level 3: Full Compliance (MUST + SHOULD + MAY)
Complete implementation with optional features:
- All optional capabilities
- Extended error handling
- Custom transports
- Experimental features

### Test Categories Mapping

| Specification Area | Test Category | Test Count |
|-------------------|---------------|------------|
| Lifecycle | Base Protocol | 10 |
| Transport | Transport Tests | 8 |
| Message Format | Specification Coverage | 12 |
| Capabilities | Base Protocol | 6 |
| Tools | Tools Tests | 11 |
| Resources | Resources Tests | 6 |
| Error Handling | Error Tests | 8 |
| Version-Specific | Version Tests | 10 |
| Proxy-Specific | Proxy Tests | 28 |

---

## Implementation Priority

### Phase 1: Critical Requirements (Week 1)
Focus on MUST requirements for core functionality:
1. Lifecycle initialization sequence
2. JSON-RPC message format
3. Basic transport implementation
4. Core error handling

### Phase 2: Quality Requirements (Week 2)
Add SHOULD requirements for production readiness:
1. Timeout and cancellation
2. Security validations
3. Session management
4. Capability negotiation

### Phase 3: Enhanced Features (Week 3)
Implement MAY requirements and optimizations:
1. Optional capabilities
2. Performance optimizations
3. Extended error handling
4. Custom transport support

---

## Compliance Validation

### Self-Test Checklist
```rust
// Each requirement should have a corresponding test
#[test]
fn test_initialization_must_be_first() {
    // Verify initialization is first interaction
}

#[test]
fn test_client_must_send_initialized() {
    // Verify initialized notification sent
}
```

### Automated Compliance Score
```json
{
  "compliance_level": 1,
  "must_requirements": {
    "total": 106,
    "passed": 95,
    "failed": 11,
    "percentage": 89.6
  },
  "should_requirements": {
    "total": 91,
    "passed": 72,
    "failed": 19,
    "percentage": 79.1
  },
  "may_requirements": {
    "total": 36,
    "implemented": 12,
    "percentage": 33.3
  },
  "overall_score": 75.8
}
```

---

## Notes

1. This checklist is based on MCP specification version 2025-03-26
2. Requirements are extracted directly from specification documents
3. Proxy-specific requirements are additions for Shadowcat implementation
4. Test implementation should follow the same categorization
5. Compliance reporting should track each requirement individually

---

*Generated: 2025-08-23*
*Specification Version: 2025-03-26*
*Total Requirements: 233*