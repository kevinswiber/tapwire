# Architecture Clarification for Transport Context Refactor

## Metadata Ownership Model

### Transport Layer Owns
- **Connection Management**
  - Socket handles, process handles
  - Connection state (connected, disconnected, error)
  - Reconnection logic and backoff strategies
  
- **Wire Protocol Details**
  - HTTP status codes, headers, cookies
  - SSE event IDs, retry timing
  - stdio line delimiters, EOF handling
  - Buffer management, chunking

- **Transport-Specific Optimizations**
  - Keep-alive settings
  - Compression (if applicable)
  - TLS/SSL context

### MCP Protocol Layer Owns
- **Session Management**
  - Session ID generation and validation
  - Session lifecycle (init → active → shutdown)
  - Session-scoped state and capabilities
  
- **Protocol Semantics**
  - Message directionality (client→server vs server→client)
  - Capability negotiation results
  - Protocol version in use
  - Initialize/initialized handshake state

- **Routing Decisions**
  - Which endpoint receives a message
  - Request/response correlation
  - Notification fan-out rules

### JSON-RPC Layer Owns
- **Message Structure**
  - Request ID generation and tracking
  - Method names and parameter validation
  - Result/error formatting
  - JSON serialization/deserialization

- **Protocol Compliance**
  - JSON-RPC 2.0 specification adherence
  - Batch message support (if needed)
  - Error code standardization

## Layer Interaction Model

```
┌─────────────────────────────────────────────────────┐
│                   MessageEnvelope                    │
├───────────────────────────┬─────────────────────────┤
│   TransportContext        │     McpMessage          │
├───────────────────────────┼─────────────────────────┤
│ • Transport type          │ • Request               │
│ • HTTP headers            │   - id, method, params  │
│ • SSE event ID            │ • Response              │
│ • Connection ID           │   - id, result/error    │
│ • Timestamp               │ • Notification          │
│                           │   - method, params      │
└───────────────────────────┴─────────────────────────┘
                            │
                            ▼
                    ┌───────────────┐
                    │ SessionContext│
                    ├───────────────┤
                    │ • Session ID  │
                    │ • Direction   │
                    │ • Capabilities│
                    │ • Protocol ver│
                    └───────────────┘
```

## Abstraction Requirements

### 1. Clean Separation of Concerns
```rust
// Transport only knows about bytes and connections
trait Transport {
    async fn send_bytes(&mut self, data: &[u8]) -> Result<()>;
    async fn receive_bytes(&mut self) -> Result<Vec<u8>>;
}

// MCP layer handles protocol semantics
trait McpTransport {
    async fn send_message(&mut self, msg: McpMessage, ctx: SessionContext) -> Result<()>;
    async fn receive_message(&mut self) -> Result<(McpMessage, SessionContext)>;
}
```

### 2. Composable Context
```rust
struct MessageEnvelope {
    message: McpMessage,
    transport: TransportContext,
    session: SessionContext,
}

struct TransportContext {
    transport_type: TransportType,
    headers: Option<HashMap<String, String>>,
    event_id: Option<String>,
    retry_info: Option<RetryInfo>,
}

struct SessionContext {
    session_id: SessionId,
    direction: Direction,
    protocol_version: String,
    capabilities: Capabilities,
}
```

### 3. Direction Tracking
```rust
enum Direction {
    ClientToServer,
    ServerToClient,
}

// Direction is determined by:
// 1. Transport edge (where message arrived)
// 2. Session role (client or server)
// 3. Method semantics (some methods are directional)
```

## Protocol Compliance Requirements

### MCP Specification Alignment
- Messages must maintain JSON-RPC 2.0 format
- Session management must follow MCP lifecycle
- Capability negotiation must be preserved
- Transport switching must be transparent

### Security Boundaries
- Transport metadata must not leak into application layer
- Session tokens must be isolated from message content
- Authentication happens at transport, authorization at MCP layer
- Never forward client credentials upstream

## Migration Path Considerations

### Phase 1: Add Context Without Breaking Changes
```rust
// Extend existing types with optional context
impl TransportMessage {
    fn with_context(self, ctx: MessageContext) -> ContextualMessage {
        // Wrapper that preserves existing API
    }
}
```

### Phase 2: Introduce MessageEnvelope
```rust
// New abstraction that separates concerns
enum MessageEnvelope {
    V1(TransportMessage), // Backward compat
    V2 {
        message: McpMessage,
        transport: TransportContext,
        session: SessionContext,
    }
}
```

### Phase 3: Deprecate TransportMessage
- Mark old types as deprecated
- Provide migration tools
- Update all consumers to use new types

## SSE Integration Requirements

### Critical for SSE Proxy Support
1. **Event ID Tracking** - Must preserve SSE event IDs for resumability
2. **Direction Clarity** - SSE is inherently server→client for events
3. **Retry Semantics** - Must handle Retry-After headers properly
4. **Stream Multiplexing** - Multiple SSE connections per session

### New Abstractions Needed
```rust
struct SseContext {
    event_id: Option<String>,
    event_type: Option<String>,
    retry_ms: Option<u64>,
    last_event_id: Option<String>, // For resumption
}
```

## Testing Strategy

### Layer Isolation Tests
- Transport tests use mock bytes, no protocol knowledge
- MCP tests use mock transport, focus on protocol
- JSON-RPC tests validate structure only

### Integration Tests
- Full stack tests with all layers
- Protocol compliance validation
- Direction tracking verification
- Session management scenarios

## Documentation Requirements

### For Each Layer
- Clear responsibility boundaries
- Interaction patterns
- Extension points
- Migration examples

### For Implementers
- How to add new transports
- How to extend context
- How to maintain compatibility
- How to debug layer issues