# MCP Protocol Layer Analysis

## Protocol Layers

### 1. Transport Layer
**Purpose**: Provides the communication channel between client and server
**Responsibilities**: 
- Message delivery mechanism (stdio, HTTP, SSE, WebSocket)
- Connection management
- Byte stream handling
- Transport-specific headers (HTTP headers, SSE event IDs)

**Examples**: 
- stdio: Process spawning, stdin/stdout pipes
- HTTP: Request/response cycles, status codes
- SSE: Event streams, reconnection support

### 2. MCP Protocol Layer
**Purpose**: Defines the semantic protocol for Model Context Protocol communication
**Responsibilities**:
- Session management (initialization, lifecycle, shutdown)
- Capability negotiation
- Protocol version agreement
- Message routing and directionality

**Message types**: Request, Response, Notification (all bidirectional)

### 3. JSON-RPC Layer
**Purpose**: Provides the message structure and encoding
**Responsibilities**:
- Message format specification
- Request/response correlation via IDs
- Error structure standardization
- Method and parameter encoding

**Structure**: 
- Request: `{jsonrpc: "2.0", id: string|number, method: string, params?: object}`
- Response: `{jsonrpc: "2.0", id: string|number, result?: any, error?: object}`
- Notification: `{jsonrpc: "2.0", method: string, params?: object}`

## Notification Model

### Directionality: **BIDIRECTIONAL**

Based on the specification analysis, notifications are explicitly bidirectional:

#### Client→Server notifications:
- `notifications/initialized` - Sent after initialization completes
- `notifications/cancelled` - Cancel an in-flight request
- `notifications/roots/list_changed` - Roots list has changed (when client capability declared)

#### Server→Client notifications:
- `notifications/resources/list_changed` - Available resources changed
- `notifications/resources/updated` - Subscribed resource updated
- `notifications/prompts/list_changed` - Available prompts changed
- `notifications/tools/list_changed` - Available tools changed
- `notifications/message` - Log messages
- `notifications/progress` - Progress updates
- `notifications/cancelled` - Cancel an in-flight request

### Routing Requirements
**Critical Finding**: The current `TransportMessage::Notification` variant lacks direction information, making it impossible to properly route notifications in proxy scenarios. The direction is implicit based on:
- The session context (who sent it)
- The transport edge (where it arrived from)
- The method name (some methods are client-only or server-only)

## Metadata Requirements

### Transport Layer
**Required**:
- Connection state
- Transport type identifier

**Optional**:
- HTTP: Headers (Origin, User-Agent, etc.)
- SSE: Event IDs for resumability
- SSE: Retry-After headers for reconnection
- HTTP: Session cookies

### MCP Layer
**Required**:
- `protocolVersion` - During initialization only
- `Mcp-Session-Id` - For HTTP transport after initialization

**Optional**:
- `MCP-Protocol-Version` - HTTP header for version identification
- Client/Server info (name, version, title)
- Instructions field from server

### JSON-RPC Layer
**Required**:
- `jsonrpc: "2.0"` - Version identifier
- `id` - For requests and responses (must correlate)
- `method` - For requests and notifications

**Optional**:
- `params` - Method parameters
- `result` or `error` - In responses
- `_meta` - Extension field for additional metadata

## Transport Mapping

### HTTP Transport
- **Requests map to**: POST with request body → Response or SSE stream
- **Responses map to**: 
  - Single JSON response body
  - SSE event in stream (if streaming)
- **Notifications handled by**:
  - Client→Server: POST with 202 Accepted response
  - Server→Client: SSE events on GET stream

### SSE Transport
- **Events map to**: Any JSON-RPC message type
- **Notifications natural fit**: Yes, asynchronous by nature
- **Request/Response challenges**: 
  - Must maintain correlation across async streams
  - Resumability requires event ID tracking

### stdio Transport
- **Bidirectional stream**: Natural fit for all message types
- **Line-delimited JSON**: Each message on a single line
- **No built-in session management**: Must be handled at MCP layer

## Key Findings

1. **Notifications ARE bidirectional** - Both client and server can send notifications at any time
2. **Direction is implicit, not explicit** - Current `TransportMessage` has no direction field
3. **Session context determines routing** - The proxy must track which edge messages arrive from
4. **Transport metadata must be separated** - Headers, event IDs, etc. don't belong in the MCP message
5. **The protocol is truly layered** - Each layer has distinct responsibilities

## Refactor Implications

### Critical Changes Needed

1. **`TransportMessage` should be renamed to `McpMessage`** - It represents MCP protocol messages, not transport concerns

2. **Notifications need direction context** - Either:
   - Add a direction field to the notification variant
   - Wrap messages in an envelope with direction metadata
   - Track direction externally via session context

3. **Transport metadata must be separate** - Create a `TransportContext` or `MessageEnvelope` that contains:
   - The MCP message
   - Transport headers/metadata
   - Direction information
   - Session context

4. **Session context should be at MCP layer** - Not at transport layer, as it's part of the protocol semantics

5. **HTTP-specific concerns must be isolated** - Session IDs, headers, and SSE event IDs should not leak into the core message type

### Architectural Corrections

- The transport layer should only handle delivery, not message semantics
- The MCP layer should handle protocol concerns (sessions, capabilities, lifecycle)
- The JSON-RPC layer should handle message structure only
- Proxy logic needs explicit direction tracking for proper routing
- Interceptors need access to full context (message + metadata + direction)