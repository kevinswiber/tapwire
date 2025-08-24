# Shadowcat Transport, Session & Interceptor Inventory

## Purpose
High-level inventory of existing transport, session, and interceptor code for MCP library extraction. Keep lightweight - discover details during implementation.

## Transport Layer (`src/transport/`)

### Core Transport Abstractions
- `traits.rs` - Transport trait definition
- `factory.rs` - Transport creation and configuration
- `core/` - Capabilities, response modes
- `buffer_pool.rs` - Performance optimization (consider extracting)

### What Can Be Extracted
**Incoming Transports** (for MCP Server):
- `incoming/stdio.rs` - stdio server transport
- `incoming/http.rs` - HTTP server transport

**Outgoing Transports** (for MCP Client):
- `outgoing/subprocess.rs` - stdio client via subprocess
- `outgoing/http.rs` - HTTP client transport

**SSE Module** (`sse/`):
- Complete SSE implementation (client, parser, reconnect)
- Already handles streaming, reconnection, session management
- **Action**: Extract as `http::streaming::sse` module

### What Stays Proxy-Specific
- `pause_controller.rs` - Proxy-specific pause/resume
- `replay.rs` - Tape replay functionality
- Transport factory with proxy configuration

## Session Layer (`src/session/`)

### What to Consider
- `manager.rs` - Session lifecycle management
- `store.rs` - Session persistence interface
- `memory.rs` - In-memory session store

### Extraction Strategy
- **Client/Server need**: Basic session tracking
- **Compliance needs**: Session validation
- **Extract**: Core session types and interfaces
- **Leave**: Proxy-specific dual-session tracking

### SSE Integration
- `sse_integration.rs` - How SSE reconnection works with sessions
- **Important**: Study this for SSE transport implementation

## Interceptor Layer (`src/interceptor/`)

### Core Interceptor Pattern
- `mod.rs` - Interceptor trait definition
- `engine.rs` - Interceptor chain execution
- `builder.rs` - Interceptor configuration

### MCP-Specific Interceptors
- `mcp_interceptor.rs` - MCP message interception
- `mcp_rules_engine.rs` - Rule-based message filtering
- `rules.rs` - Rule definitions

### Extraction Approach
- **Extract**: Core interceptor trait and chain
- **Extract**: Basic MCP interceptor for compliance testing
- **Leave**: Complex rules engine (proxy-specific)
- **Leave**: HTTP policy enforcement

## Key Insights for Extraction

### Transport Layer
1. **Well-abstracted** - Transport trait is clean
2. **SSE is complete** - Full implementation ready to extract
3. **Incoming/Outgoing split** - Maps to Server/Client needs
4. **Buffer pooling** - Performance optimization worth keeping

### Session Layer
1. **Minimal needs** - Client/Server need basic session tracking
2. **Store interface** - Clean abstraction for different backends
3. **SSE integration** - Critical for reconnection support

### Interceptor Layer  
1. **Clean trait** - Interceptor pattern is well-defined
2. **Chainable** - Already supports multiple interceptors
3. **MCP-aware** - Has MCP-specific message handling
4. **Testable** - Good test coverage to guide extraction

## Extraction Priority

### High Priority (Week 1-2)
1. Transport trait + stdio implementation
2. Basic session types
3. Core interceptor trait

### Medium Priority (Week 3)
1. HTTP transport with SSE module
2. Session manager basics
3. MCP interceptor

### Low Priority (Week 4+)
1. Buffer pooling optimization
2. Advanced session persistence
3. Rules engine

## What NOT to Extract
- Pause/resume control (proxy-specific)
- Replay functionality (tape-specific)
- Complex policy enforcement
- Dual-session tracking
- Persistence worker (background jobs)

## Dependencies to Watch
- `tokio` - Async runtime (already used)
- `hyper` - HTTP implementation (not reqwest!)
- `bytes` - Buffer management
- `futures` - Stream traits

---

*Created: 2025-08-24*
*Purpose: High-level guide for transport/session/interceptor extraction*
*Key: These are mature, well-tested components ready for extraction*