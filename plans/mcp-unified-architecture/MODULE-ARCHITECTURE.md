# MCP Module Architecture Design

## Overview

Based on analysis of the MCP specification structure, this document proposes a module organization that cleanly separates protocol, transport, and feature concerns while mirroring the spec's conceptual model.

## Key Insights from Specification

The MCP spec is organized into distinct layers:

1. **Architecture**: Core client-host-server model
2. **Basic Protocol**: Lifecycle, transports, utilities  
3. **Client Features**: Roots, sampling, elicitation
4. **Server Features**: Resources, tools, prompts
5. **Schema**: Type definitions and common structures

## Proposed Module Structure

```
crates/mcp/src/
├── protocol/                 # Core MCP protocol (transport-agnostic)
│   ├── mod.rs
│   ├── lifecycle.rs         # Initialize, shutdown sequences
│   ├── messages.rs          # Request, Response, Notification types
│   ├── request_id.rs        # RequestId type (renamed from JsonRpcId)
│   ├── error.rs             # Protocol-level errors
│   └── version.rs           # Protocol versioning
│
├── basic/                    # Basic protocol elements
│   ├── mod.rs
│   ├── capabilities.rs      # Client/Server capabilities
│   ├── utilities/           # Protocol utilities
│   │   ├── mod.rs
│   │   ├── ping.rs         # Ping/pong
│   │   ├── progress.rs     # Progress notifications
│   │   └── cancellation.rs # Request cancellation
│   └── authorization.rs     # Auth mechanisms
│
├── transport/               # Transport implementations
│   ├── mod.rs
│   ├── traits.rs           # Transport trait definition
│   │
│   ├── common/             # Shared transport concepts
│   │   ├── mod.rs
│   │   ├── session.rs     # SessionId, Session management
│   │   └── delivery.rs    # Delivery context
│   │
│   ├── stdio/              # stdio transport
│   │   ├── mod.rs
│   │   ├── incoming.rs
│   │   └── outgoing.rs
│   │
│   ├── http/               # HTTP + SSE transport
│   │   ├── mod.rs
│   │   ├── stream.rs       # StreamId, StreamManager
│   │   ├── event.rs        # EventId, EventStore, EventIdGenerator
│   │   ├── streamable_incoming.rs
│   │   └── streamable_outgoing.rs
│   │
│   └── websocket/          # Future WebSocket transport
│       ├── mod.rs
│       ├── stream.rs       # WS will likely reuse stream concepts
│       └── event.rs        # And event tracking
│
├── client/                  # Client-specific features
│   ├── mod.rs
│   ├── roots.rs            # Root directory management
│   ├── sampling.rs         # LLM sampling interface
│   └── elicitation.rs      # User input elicitation
│
├── server/                  # Server-specific features
│   ├── mod.rs
│   ├── resources.rs        # Resource management
│   ├── tools.rs            # Tool definitions
│   ├── prompts.rs          # Prompt templates
│   └── utilities/          # Server utilities
│       ├── logging.rs      # Structured logging
│       ├── completion.rs   # Completion support
│       └── pagination.rs   # Result pagination
│
├── session/                 # Session management (cross-cutting)
│   ├── mod.rs
│   ├── manager.rs          # SessionManager
│   ├── store.rs            # SessionStore trait
│   ├── memory.rs           # InMemorySessionStore
│   ├── persistence_worker.rs
│   └── builder.rs
│
├── types.rs                 # Core types used throughout
├── error.rs                 # Error types
└── lib.rs                   # Module exports
```

## ID Type Hierarchy

Based on the spec analysis, we have four distinct ID types serving different layers:

### Protocol Layer
- **RequestId** (aka JsonRpcId): Message correlation in JSON-RPC

### Session Layer (Transport-Common)
- **SessionId**: Unique session identifier across all transports

### HTTP/SSE Transport Layer
- **StreamId**: Individual SSE connection within a session
- **EventId**: Specific event for replay/resumption

## Key Design Principles

### 1. Clean Layering
- Protocol layer knows nothing about transports
- Transports implement protocol delivery
- Features (client/server) use protocol primitives

### 2. Transport Independence
- Core protocol types in `protocol/`
- Transport-specific types stay in their transport modules
- Common transport concepts in `transport/common/`

### 3. Spec Alignment
- Module structure mirrors specification structure
- Same terminology as spec (client/server, not producer/consumer)
- Features grouped by actor (client vs server)

### 4. Reusability
- Stream and Event concepts can be reused by WebSocket transport
- Session management is shared across all transports
- Utilities are grouped by their scope

## Migration Benefits

This structure provides:

1. **Clear Boundaries**: Protocol vs Transport vs Features
2. **Type Safety**: IDs can't be mixed across layers
3. **Extensibility**: Easy to add new transports or features
4. **Discoverability**: Developers can find code where spec says it should be
5. **Testability**: Each layer can be tested independently

## Implementation Order

1. Move types to appropriate modules
2. Reorganize transport implementations
3. Extract client/server features
4. Update imports throughout codebase
5. Update tests

## Backwards Compatibility

Since we haven't released yet:
- No need for compatibility shims
- Can make clean breaks in API
- Focus on correct architecture from the start

## Future Considerations

### WebSocket Transport
When WebSocket transport is finalized in the spec:
- Will likely reuse StreamId concept
- May share event replay mechanisms
- Can extend EventStore trait for WS-specific needs

### Additional Transports
The structure easily accommodates new transports:
- Add new directory under `transport/`
- Implement Transport trait
- Reuse common concepts from `transport/common/`

## Conclusion

This architecture provides a clean separation of concerns that:
- Mirrors the MCP specification structure
- Provides type safety at each layer
- Enables transport independence
- Supports future extensibility

The key insight is that **Sessions are transport-common, but Streams and Events are transport-specific** (primarily for HTTP/SSE and future WebSocket). This understanding drives the module organization and type placement.