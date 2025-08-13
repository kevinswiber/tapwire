# MCP Specification Compliance Checklist

Based on MCP Protocol Revision: 2025-06-18

This document provides a comprehensive checklist of all MCP specification requirements organized by category. Each requirement is marked as MUST, SHOULD, or MAY per the specification, with checkboxes to track implementation status.

## 1. JSON-RPC Base Protocol (REQUIRED)

All implementations **MUST** support the base protocol.

### 1.1 Message Format Requirements
- [ ] **MUST**: Follow JSON-RPC 2.0 specification
- [ ] **MUST**: Include `jsonrpc: "2.0"` field in all messages
- [ ] **MUST**: Use UTF-8 encoding for all JSON-RPC messages

### 1.2 Request Requirements
- [ ] **MUST**: Include string or integer ID (not null)
- [ ] **MUST**: Include method field as string
- [ ] **MUST**: ID must not have been previously used in same session
- [ ] **MAY**: Include params field

### 1.3 Response Requirements  
- [ ] **MUST**: Include same ID as corresponding request
- [ ] **MUST**: Include either result OR error (not both)
- [ ] **MUST**: Error codes must be integers
- [ ] **MUST**: Error must include code and message at minimum

### 1.4 Notification Requirements
- [ ] **MUST**: NOT include ID field
- [ ] **MUST**: Include method field as string
- [ ] **MUST**: Receiver must NOT send response to notifications

## 2. Transport Layer

### 2.1 stdio Transport (SHOULD support)
- [x] **SHOULD**: Support stdio transport whenever possible
- [x] **MUST**: Read JSON-RPC from stdin, write to stdout  
- [x] **MUST**: Delimit messages by newlines
- [x] **MUST**: NOT contain embedded newlines in messages
- [x] **MAY**: Write UTF-8 logs to stderr
- [x] **MUST**: NOT write non-MCP content to stdout
- [x] **MUST**: NOT write non-MCP content to stdin

### 2.2 Streamable HTTP Transport
- [ ] **MUST**: Provide single HTTP endpoint supporting POST and GET
- [ ] **MUST**: Validate Origin header to prevent DNS rebinding
- [ ] **SHOULD**: Bind only to localhost for local servers  
- [ ] **SHOULD**: Implement proper authentication

#### 2.2.1 Sending Messages to Server (POST)
- [ ] **MUST**: Use HTTP POST for client-to-server messages
- [ ] **MUST**: Include Accept header with "application/json" and "text/event-stream"
- [ ] **MUST**: Single JSON-RPC message per POST body
- [ ] **MUST**: Return 202 Accepted for notifications/responses
- [ ] **MUST**: Support both SSE stream and JSON responses for requests
- [ ] **SHOULD**: Eventually include JSON-RPC response in SSE stream
- [ ] **MAY**: Send requests/notifications before response in SSE
- [ ] **SHOULD**: NOT close SSE before sending response
- [ ] **SHOULD**: Close SSE after sending response

#### 2.2.2 Listening for Messages from Server (GET)
- [ ] **MAY**: Support HTTP GET to open SSE stream
- [ ] **MUST**: Include Accept header with "text/event-stream"  
- [ ] **MUST**: Return SSE stream or 405 Method Not Allowed
- [ ] **MAY**: Send requests/notifications on stream
- [ ] **MUST**: NOT send responses unless resuming stream
- [ ] **MAY**: Close SSE stream at any time

#### 2.2.3 Multiple Connections
- [ ] **MAY**: Support multiple simultaneous SSE streams
- [ ] **MUST**: Send each message on only one stream (no broadcasting)

#### 2.2.4 Resumability and Redelivery
- [ ] **MAY**: Attach ID field to SSE events
- [ ] **MUST**: ID must be globally unique within session if present
- [ ] **MAY**: Support Last-Event-ID header for resume
- [ ] **MAY**: Replay messages after last event ID
- [ ] **MUST**: NOT replay messages from different streams

#### 2.2.5 Session Management
- [ ] **MAY**: Assign session ID during initialization
- [ ] **SHOULD**: Use globally unique, cryptographically secure session ID
- [ ] **MUST**: Session ID only contain visible ASCII (0x21-0x7E)
- [ ] **MUST**: Include Mcp-Session-Id header if server provided one
- [ ] **SHOULD**: Return 400 Bad Request for missing session ID
- [ ] **MAY**: Terminate session, respond 404 Not Found after
- [ ] **MUST**: Start new session on 404 response
- [ ] **SHOULD**: Send DELETE to explicitly terminate session
- [ ] **MAY**: Respond 405 Method Not Allowed to DELETE

#### 2.2.6 Protocol Version Header
- [ ] **MUST**: Include MCP-Protocol-Version header on all HTTP requests
- [ ] **SHOULD**: Use negotiated version from initialization
- [ ] **SHOULD**: Assume 2025-03-26 if no header received
- [ ] **MUST**: Return 400 Bad Request for invalid/unsupported version

### 2.3 Custom Transports
- [ ] **MAY**: Implement additional custom transports
- [ ] **MUST**: Preserve JSON-RPC format for custom transports
- [ ] **MUST**: Preserve lifecycle requirements for custom transports
- [ ] **SHOULD**: Document custom transport patterns

## 3. Lifecycle Management (REQUIRED)

All implementations **MUST** support lifecycle management.

### 3.1 Initialization Phase
- [ ] **MUST**: Be first interaction between client and server
- [ ] **MUST**: Client sends initialize request with protocol version
- [ ] **MUST**: Client includes capabilities and client info
- [ ] **MUST**: Server responds with capabilities and server info
- [ ] **MUST**: Client sends initialized notification after response
- [ ] **SHOULD**: NOT send requests before initialize response (except ping)
- [ ] **SHOULD**: Server not send requests before initialized (except ping/logging)

### 3.2 Version Negotiation  
- [ ] **MUST**: Client sends latest supported version in initialize
- [ ] **MUST**: Server responds with same version if supported
- [ ] **MUST**: Server responds with different supported version if not
- [ ] **SHOULD**: Server responds with latest supported version
- [ ] **SHOULD**: Client disconnect if version not supported

### 3.3 Capability Negotiation
- [ ] **MUST**: Exchange capabilities during initialization
- [ ] **MUST**: Only use successfully negotiated capabilities
- [ ] **MUST**: Respect negotiated protocol version during operation

### 3.4 Shutdown
- [ ] **SHOULD**: Use transport mechanism for shutdown signaling
- [ ] **SHOULD**: Client close stdin then wait for server exit (stdio)
- [ ] **MAY**: Server initiate shutdown by closing stdout (stdio)
- [ ] **SHOULD**: Use HTTP connection closure for HTTP transports

## 4. Error Handling and Timeouts

### 4.1 Timeout Management
- [ ] **SHOULD**: Establish timeouts for all sent requests  
- [ ] **SHOULD**: Issue cancellation notification on timeout
- [ ] **SHOULD**: Allow per-request timeout configuration
- [ ] **MAY**: Reset timeout on progress notifications
- [ ] **SHOULD**: Always enforce maximum timeout

### 4.2 Error Cases
- [ ] **SHOULD**: Handle protocol version mismatch
- [ ] **SHOULD**: Handle capability negotiation failure
- [ ] **SHOULD**: Handle request timeouts

## 5. Server-Sent Events (SSE) - CRITICAL GAP

### 5.1 SSE Stream Management
- [ ] **MUST**: Support text/event-stream content type
- [ ] **MUST**: Implement proper SSE event format
- [ ] **MUST**: Handle SSE connection lifecycle
- [ ] **MUST**: Support SSE event IDs for resumability
- [ ] **MUST**: Parse Last-Event-ID header correctly
- [ ] **MUST**: Implement SSE error handling and reconnection

### 5.2 SSE Security
- [ ] **MUST**: Validate Origin header for SSE connections
- [ ] **MUST**: Implement CORS policies for SSE if needed
- [ ] **MUST**: Handle SSE connection limits properly

## 6. Authorization Framework (OPTIONAL)

### 6.1 OAuth 2.1 Compliance
- [ ] **SHOULD**: Conform to OAuth 2.1 for HTTP transports
- [ ] **SHOULD**: NOT use OAuth for stdio (use environment instead)
- [ ] **MUST**: Support OAuth 2.1 for authorization servers
- [ ] **SHOULD**: Support OAuth 2.0 Dynamic Client Registration (RFC7591)
- [ ] **MUST**: Implement OAuth 2.0 Protected Resource Metadata (RFC9728)
- [ ] **MUST**: Provide OAuth 2.0 Authorization Server Metadata (RFC8414)

### 6.2 Authorization Server Discovery
- [ ] **MUST**: Implement Protected Resource Metadata for server discovery
- [ ] **MUST**: Include authorization_servers field with ≥1 server
- [ ] **MUST**: Use WWW-Authenticate header on 401 responses
- [ ] **MUST**: Parse WWW-Authenticate headers (clients)
- [ ] **MUST**: Follow Authorization Server Metadata spec

### 6.3 Security Requirements
- [ ] **MUST**: Use PKCE (Proof Key for Code Exchange)
- [ ] **MUST**: Validate JWT audience claims properly
- [ ] **MUST**: NOT forward client tokens to upstream servers
- [ ] **MUST**: Implement proper token validation

## 7. Advanced Features (OPTIONAL)

### 7.1 Resources
- [ ] **MAY**: Implement resource capability
- [ ] **MAY**: Support resource subscription
- [ ] **MAY**: Send resource list change notifications

### 7.2 Tools  
- [ ] **MAY**: Implement tools capability
- [ ] **MAY**: Send tools list change notifications

### 7.3 Prompts
- [ ] **MAY**: Implement prompts capability  
- [ ] **MAY**: Send prompts list change notifications

### 7.4 Sampling
- [ ] **MAY**: Support LLM sampling requests (clients)

### 7.5 Utilities
- [ ] **MAY**: Implement ping utility
- [ ] **MAY**: Implement logging utility
- [ ] **MAY**: Implement progress notifications
- [ ] **MAY**: Implement cancellation notifications
- [ ] **MAY**: Implement argument completion

## 8. Reserved Fields and Metadata

### 8.1 _meta Property
- [ ] **MUST**: NOT make assumptions about reserved _meta values
- [ ] **MUST**: Follow _meta key naming conventions  
- [ ] **MUST**: NOT use MCP reserved prefixes for custom metadata

## Summary Status

**Critical Missing Features:**
- [ ] Complete SSE (Server-Sent Events) implementation
- [ ] Streamable HTTP transport compliance  
- [ ] Session management for HTTP transport
- [ ] OAuth 2.1 authorization framework

**Implemented Features:**
- [x] stdio transport (mostly compliant)
- [x] Basic JSON-RPC handling
- [x] Basic HTTP transport (partial)
- [x] Session ID generation

**Implementation Priority:**
1. **CRITICAL**: SSE support for Streamable HTTP
2. **HIGH**: Complete Streamable HTTP transport
3. **HIGH**: Session management for HTTP
4. **MEDIUM**: OAuth 2.1 authorization
5. **LOW**: Advanced optional features