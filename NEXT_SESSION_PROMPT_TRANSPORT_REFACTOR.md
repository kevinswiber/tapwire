# Transport Refactor Session Prompt

**Purpose**: This file is for the Transport Refactor project ONLY. Do not use for SSE/MCP work.

## Project Overview

We're refactoring Shadowcat's transport layer to introduce clearer `IncomingTransport` and `OutgoingTransport` abstractions. This addresses architectural confusion where:
- `StdioTransport` spawns subprocesses (actually outgoing)
- `StdioClientTransport` reads stdin (actually incoming)  
- HTTP and SSE are artificially separated when MCP uses both together
- Transport mechanics are mixed with protocol semantics

## Current Status

**Phase**: Planning
**Tracker**: `plans/transport-refactor/transport-refactor-tracker.md`
**Status**: Waiting for SSE/MCP Phase 3-7 completion

## Key Design Decisions

1. **IncomingTransport**: Transports the proxy exposes (accept connections)
   - StdioIncoming (read from stdin)
   - HttpServerIncoming (HTTP server)
   - StreamableHttpIncoming (HTTP server + SSE responses)

2. **OutgoingTransport**: Transports that connect to upstream targets
   - SubprocessOutgoing (spawn subprocess)
   - HttpClientOutgoing (HTTP client)
   - StreamableHttpOutgoing (HTTP POST + SSE client)

3. **Layer Separation**:
   - RawTransport: Handles bytes only
   - ProtocolHandler: Handles MCP/JSON-RPC
   - Direction-aware transports: Combine the above

## Next Session Tasks

Currently waiting for SSE/MCP work to complete Phase 3-7. Once ready:

1. Review current SSE/MCP implementation status
2. Document all existing transport usage patterns
3. Create comprehensive test suite for current behavior
4. Begin Phase 1: Foundation tasks (F.1-F.5)

## Important Notes

- **DO NOT** modify existing transports until migration strategy approved
- **DO NOT** break ongoing SSE/MCP work
- This refactor can either wait (safer) or run in parallel with careful coordination
- Use this file (`NEXT_SESSION_PROMPT_TRANSPORT_REFACTOR.md`) for transport refactor sessions
- Use `NEXT_SESSION_PROMPT.md` for SSE/MCP work sessions

## Success Criteria

- Clear separation between incoming/outgoing transports
- Unified Streamable HTTP support (HTTP POST + SSE)
- No protocol logic in transport layer
- Improved testability and maintainability

---

Last Updated: 2025-08-12
Next Review: After SSE/MCP Phase 3 completion