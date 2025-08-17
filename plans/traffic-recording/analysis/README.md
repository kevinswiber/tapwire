# Traffic Recording Analysis Documents

This directory contains analysis and design documents for the traffic recording refactor.

## Documents

### Planned Documents

- **recording-architecture.md** - Detailed design for the recording layer architecture
- **migration-plan.md** - Step-by-step plan for migrating from TransportContext::Sse
- **backward-compatibility.md** - How to handle existing tape formats
- **performance-analysis.md** - Memory and CPU impact of passing raw data

### Completed Analysis

The following analysis was completed during the initial conversation:

1. **TransportContext::Sse Usage Analysis**
   - Found it's only used in transport layer and recording
   - Not used by either forward or reverse proxy
   - Only created when receiving SSE responses, never for incoming

2. **SSE Metadata Requirements**
   - `event_id`: SSE reconnection ID (not JSON-RPC ID)
   - `event_type`: Always "message" for MCP
   - `retry_ms`: Reconnection delay hint
   - Only needed for recording/replay fidelity

3. **Type Duplication Analysis**
   - Found 3 different SSE-related types
   - Each serving slightly different purpose
   - Can be consolidated to single canonical type

4. **ResponseMode Discovery**
   - Already exists in `transport::core::response_mode`
   - Has Json/SseStream/Passthrough variants
   - Uses mime crate for proper content-type parsing
   - Perfect replacement for the SSE detection

## Key Findings

### Architectural Issues
1. SSE is not a transport type, it's an HTTP response format
2. Wire format details don't belong in TransportContext
3. Recording layer needs access to raw data it doesn't have
4. MessageContext.metadata HashMap is untyped and unused

### Correct Architecture
- Transport detects ResponseMode (Json/EventStream/Passthrough)
- Transport extracts JSON-RPC from SSE but doesn't carry SSE metadata
- Recording layer receives both MessageEnvelope and raw wire data
- Recording extracts SSE metadata when ResponseMode::EventStream
- SSE metadata stored in typed FrameMetadata for replay

## Design Principles

1. **Separation of Concerns**: Transport handles message delivery, not wire format details
2. **Type Safety**: Use proper types, not HashMap<String, Value>
3. **Performance**: Use Arc to share raw data without copies
4. **Compatibility**: Maintain backward compatibility with existing recordings
5. **Simplicity**: One canonical type for each concept

## References

- [MCP Specification](https://spec.modelcontextprotocol.io)
- [SSE Specification](https://html.spec.whatwg.org/multipage/server-sent-events.html)
- [Server-Sent Events on MDN](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events)