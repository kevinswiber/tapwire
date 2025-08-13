# Transport Layer Refactor: Foundation Design

## Phase 1 Design Deliverables (Completed 2025-08-13)

### 1. Raw Transport Layer

**Location**: `shadowcat/src/transport/raw/mod.rs`

```rust
pub trait RawTransport: Send + Sync {
    async fn connect(&mut self) -> TransportResult<()>;
    async fn send_bytes(&mut self, data: &[u8]) -> TransportResult<()>;
    async fn receive_bytes(&mut self) -> TransportResult<Vec<u8>>;
    async fn close(&mut self) -> TransportResult<()>;
    fn is_connected(&self) -> bool;
}
```

**Key Design Decisions**:
- Bytes-only interface, no protocol knowledge
- Async-first for all I/O operations
- Transport-specific framing handled internally
- Metadata support for debugging

### 2. Protocol Handler

**Location**: `shadowcat/src/transport/protocol/mod.rs`

```rust
pub trait ProtocolHandler: Send + Sync {
    fn serialize(&self, msg: &ProtocolMessage) -> TransportResult<Vec<u8>>;
    fn deserialize(&self, data: &[u8]) -> TransportResult<ProtocolMessage>;
    fn mcp_version(&self) -> &str;
    fn validate(&self, msg: &ProtocolMessage) -> TransportResult<()>;
}
```

**Key Design Decisions**:
- Single `McpProtocolHandler` since MCP uses JSON-RPC 2.0
- Uses existing `LATEST_SUPPORTED_VERSION` constant (2025-06-18)
- Validates MCP-specific requirements (e.g., initialize must have protocolVersion)
- Strict mode for production, lenient for testing

### 3. Direction-Aware Transports

**Location**: `shadowcat/src/transport/directional/mod.rs`

```rust
pub trait IncomingTransport: Send + Sync {
    async fn accept(&mut self) -> TransportResult<()>;
    async fn receive_request(&mut self) -> TransportResult<MessageEnvelope>;
    async fn send_response(&mut self, response: MessageEnvelope) -> TransportResult<()>;
    async fn close(&mut self) -> TransportResult<()>;
}

pub trait OutgoingTransport: Send + Sync {
    async fn connect(&mut self) -> TransportResult<()>;
    async fn send_request(&mut self, request: MessageEnvelope) -> TransportResult<()>;
    async fn receive_response(&mut self) -> TransportResult<MessageEnvelope>;
    async fn close(&mut self) -> TransportResult<()>;
}
```

**Key Design Decisions**:
- Clear request/response semantics
- Generic implementations compose RawTransport + ProtocolHandler
- Session ID tracking built-in
- MessageDirection set appropriately

### 4. Process Management

**Location**: `shadowcat/src/process/mod.rs`

```rust
pub trait ProcessManager: Send + Sync {
    async fn spawn(&mut self, command: Command) -> TransportResult<ProcessHandle>;
    async fn terminate(&mut self, handle: ProcessHandle) -> TransportResult<()>;
    fn is_alive(&self, handle: &ProcessHandle) -> bool;
}
```

**Key Design Decisions**:
- Process lifecycle completely separate from transport
- ProcessPool for expensive MCP servers
- Graceful shutdown with timeout fallback
- Better error messages for common failures

### 5. Error Types

**Location**: `shadowcat/src/error.rs`

Added new TransportError variants:
- `SerializationError(String)`
- `DeserializationError(String)`
- `ProcessSpawnFailed(String)`
- `ProcessTerminationFailed(String)`
- `ProcessNotFound(String)`
- `MultipleErrors(Vec<String>)`
- `UnsupportedOperation(String)`

## Migration Strategy

Since Shadowcat is pre-release, we can make breaking changes:

1. **No compatibility layer needed** - Direct replacement
2. **Parallel implementation** - Build new alongside old temporarily
3. **Atomic switch** - Replace all at once when ready
4. **Clean removal** - Delete old code completely

## Implementation Plan

### Clear Naming Convention
- **Old**: `StdioTransport` (spawns process) → **New**: `SubprocessOutgoing`
- **Old**: `StdioClientTransport` (reads stdin) → **New**: `StdioIncoming`
- **Old**: `HttpTransport` (client) → **New**: `HttpClientOutgoing`
- **Old**: `HttpMcpTransport` (server) → **New**: `HttpServerIncoming`
- **Old**: `SseTransport` (client) → **New**: Part of `StreamableHttpOutgoing`

### Unified Streamable HTTP
Instead of separate HTTP and SSE transports, create unified:
- `StreamableHttpIncoming`: HTTP server that can respond with SSE
- `StreamableHttpOutgoing`: HTTP client that can receive SSE responses

## Next Phase Requirements

### Phase 2: Raw Transport Implementation
- Implement `StdioRawTransport`
- Implement `HttpRawTransport`
- Implement `SseRawTransport`
- Implement `SubprocessRawTransport`
- Create comprehensive tests

### Phase 3: Integration
- Wire up Generic transports
- Update forward proxy
- Update reverse proxy
- Update CLI

### Phase 4: Cleanup
- Remove old Transport trait
- Remove old implementations
- Update all tests
- Update documentation