# WebSocket Transport Separation Decision

## Decision: WebSocket as Separate Transport ✅

After reviewing GPT-5's findings, the MCP WebSocket proposal (issue #1288), and our implementation, we confirm that **WebSocket should be a separate transport**, not a sub-mode of HTTP.

## Key Differences

| Aspect | HTTP Transport | WebSocket Transport |
|--------|---------------|---------------------|
| **Handshake** | POST request | GET + Upgrade |
| **Response** | 200 OK with body/SSE | 101 Switching Protocols |
| **Sessions** | Optional (via headers) | REQUIRED (in message body) |
| **Auth** | Headers/cookies | Subprotocol negotiation |
| **Connection** | Request/response or SSE stream | Persistent bidirectional |
| **Session Location** | Transport layer (headers) | Data layer (message field) |
| **Multiplexing** | Multiple sessions per connection | Single session per connection |

## Implementation Plan

### 1. HTTP Transport (`transport/http/`)
```rust
pub struct HttpTransport {
    // Handles POST requests
    // Adaptive: JSON response or SSE stream
    // Sessions via headers (optional)
}
```

### 2. WebSocket Transport (`transport/websocket/`)
```rust
pub struct WebSocketTransport {
    // Handles GET + Upgrade
    // Persistent bidirectional connection
    // Sessions in every message (required)
    // Single connection per session enforcement
}
```

## Shared Components

Both transports can share:
- Session management infrastructure (`src/session/`)
- Retry/backoff logic (`src/retry/`)
- Protocol message handling
- Error types

## Benefits of Separation

1. **Clarity**: Each transport has distinct lifecycle and requirements
2. **Maintainability**: Changes to one don't affect the other
3. **Testing**: Can test independently with appropriate mocks
4. **Feature Gating**: WebSocket can be optional feature
5. **Compliance**: Each can follow its specific protocol requirements

## Migration Path

For existing code that might expect WebSocket as HTTP sub-mode:
1. Current `HttpTransport` removes 101 status handling
2. New `WebSocketTransport` implements proper upgrade
3. Client code chooses transport based on URL scheme or config
4. Session management adapts based on transport type

## Conclusion

The separation is architecturally correct and aligns with:
- MCP WebSocket proposal requirements
- GPT-5's expert analysis
- Clean separation of concerns
- Protocol-specific requirements

This decision is **FINAL** and implementation should proceed with separate transports.