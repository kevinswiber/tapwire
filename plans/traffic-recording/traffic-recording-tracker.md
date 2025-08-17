# Traffic Recording & SSE Metadata Refactor Tracker

## Overview

This tracker coordinates fixing SSE metadata handling in the transport layer.

**FINAL REVELATION (2025-08-17)**: TransportContext::Sse is CORRECT! It's message-level context. The real problem is we're throwing away SSE metadata when buffering. See `analysis/the-real-problem.md`.

**Last Updated**: 2025-08-17  
**Total Estimated Duration**: ~~16-24 hours~~ ~~3 hours~~ **30 minutes!**  
**Status**: Ready to implement trivial fix

## Final Solution (2025-08-17)

1. **Keep TransportContext::Sse** - It's correct! Message-level delivery context
2. **Buffer full SseEvent** - Not just Vec<u8> (revert our "simplification")
3. **Populate SSE metadata** - When creating TransportContext::Sse
4. **Recording layer unchanged** - Already extracts from Sse correctly

The problem was we were throwing away SSE metadata during buffering!

## Architecture Vision

```
┌─────────────────────────────────────────────────────────┐
│                    Wire Format                          │
│  (HTTP Response with text/event-stream content-type)    │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│                  Transport Layer                        │
│  • Detects ResponseMode (Json/EventStream/Passthrough)  │
│  • Extracts JSON-RPC message from SSE data field       │
│  • Creates TransportContext::Http with ResponseMode     │
│  • Does NOT carry SSE protocol metadata                 │
└────────────────┬───────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────┐
│                 MessageEnvelope                         │
│  • Contains ProtocolMessage (Request/Response/Notif)    │
│  • Contains MessageContext with TransportContext        │
│  • No SSE-specific fields                               │
└────────────────┬───────────────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────────────┐
│                 Recording Layer                         │
│  • Receives MessageEnvelope AND raw wire data          │
│  • Extracts SseMetadata from raw SSE event if needed   │
│  • Stores in typed FrameMetadata.transport_metadata     │
│  • Preserves for faithful replay                        │
└─────────────────────────────────────────────────────────┘
```

## Current Problems (Analysis Summary)

### 1. TransportContext::Sse is a Code Smell
- SSE is NOT a transport type - it's an HTTP response format
- MCP spec defines only 2 transports: stdio and "Streamable HTTP" 
- SSE is part of Streamable HTTP, not separate
- We already consolidated TransportType::Sse → TransportType::Http
- TransportContext should follow the same pattern

### 2. Duplicate SSE Types
- `transport::sse::event::SseEvent` - Wire format representation
- `transport::outgoing::http::SseEvent` - Internal buffering struct
- `recorder::tape::SseMetadata` - Recording metadata
- Need single canonical SseEvent for wire format

### 3. SSE Metadata Doesn't Belong in TransportContext
- SSE fields (event_id, event_type, retry_ms) are wire format details
- They're SSE protocol fields, not MCP/JSON-RPC fields
- event_id is for SSE reconnection (Last-Event-ID), not JSON-RPC ID
- event_type is always "message" for MCP
- Only retry_ms has runtime behavior (reconnection delay)

### 4. Recording Layer Can't Access Wire Format
- Recorder needs SSE metadata for faithful replay
- Currently tries to extract from TransportContext (wrong layer)
- Should receive raw wire data alongside MessageEnvelope
- Can then parse SSE fields when ResponseMode::EventStream

### 5. MessageContext.metadata is Untyped
- HashMap<String, Value> loses type safety
- Never actually used in practice
- Not the right place for SSE metadata anyway

## Simplified Work Plan (Revised 2025-08-17)

### Quick Implementation (3 hours total)

| Step | Task | Duration | Status | Notes |
|------|------|----------|--------|-------|
| 1 | **Add SSE metadata to Http** | 1h | ⬜ Not Started | Add sse_metadata field and SseMetadata struct |
| 2 | **Update SSE transports** | 1h | ⬜ Not Started | Set sse_metadata when creating Http contexts |
| 3 | **Update recording layer** | 30m | ⬜ Not Started | Extract from Http instead of Sse |
| 4 | **Remove TransportContext::Sse** | 30m | ⬜ Not Started | Delete variant, fix compilation |

### Completed Work
- ✅ **Analysis** - Identified the issue and overly complex solution
- ✅ **Simplified SseEvent** - Removed BufferedSseEvent struct, using Vec<u8>
- ✅ **Created simplified plan** - Pragmatic 3-hour approach

### Original Complex Plan (Archived)
See `analysis/recording-architecture.md` and `analysis/migration-plan.md` for the original 17-hour RawWireData approach. We decided this was over-engineered for passing 3 optional fields.

### Status Legend
- ⬜ Not Started - Task not yet begun
- 🔄 In Progress - Currently being worked on
- ✅ Complete - Task finished and tested
- ❌ Blocked - Cannot proceed due to dependency or issue
- ⏸️ Paused - Temporarily halted

## Progress Tracking

### Completed Tasks (2025-08-17)
- [x] A.0: Analyzed TransportContext::Sse usage patterns
- [x] A.1: Mapped all SSE metadata usage in codebase
- [x] A.2: Designed detailed recording architecture (see analysis/recording-architecture.md)
- [x] A.3: Created migration plan (see analysis/migration-plan.md)
- [x] B.1: Consolidated SseEvent types - renamed internal buffer type to BufferedSseEvent
- [x] Identified that SSE metadata is only needed for recording/replay
- [x] Confirmed proxies don't use SSE context
- [x] Verified ResponseMode already exists in transport::core

### Next Session Tasks
- [ ] B.2: Remove TransportContext::Sse variant
- [ ] B.3: Add ResponseMode to Http Context
- [ ] C.1: Pass Raw Data to Recorder

## Success Criteria

### Functional Requirements
- ✅ TransportContext::Sse variant removed
- ✅ Single canonical SseEvent type
- ✅ SSE metadata properly recorded
- ✅ Recording/replay continues to work
- ✅ Both forward and reverse proxies work

### Code Quality Requirements
- ✅ No clippy warnings
- ✅ All tests passing
- ✅ Type safety maintained (no HashMap<String,Value>)
- ✅ Clean semantic boundaries

### Architecture Requirements
- ✅ Transport layer doesn't carry wire format details
- ✅ Recording layer has access to raw data
- ✅ SSE metadata only in recording/replay layer
- ✅ ResponseMode properly integrated

## Risk Mitigation

| Risk | Impact | Mitigation | Status |
|------|--------|------------|--------|
| Breaking existing recordings | HIGH | Implement backward compatibility for old tapes | Planned |
| Missing SSE metadata in recordings | MEDIUM | Comprehensive testing with real SSE streams | Planned |
| Performance impact of passing raw data | LOW | Use Arc/Rc to avoid copies | Planned |
| Proxy behavior changes | MEDIUM | Extensive integration testing | Planned |

## Key Design Decisions

### 1. ResponseMode vs ResponseFormat
- Use existing `ResponseMode` enum from `transport::core::response_mode`
- Already has Json/SseStream/Passthrough variants
- Already uses mime crate for proper parsing

### 2. SSE Event ID vs JSON-RPC ID
- SSE `id:` field is for reconnection (Last-Event-ID header)
- NOT the same as JSON-RPC message ID
- ProtocolMessage already has the JSON-RPC ID
- SSE event_id only needed for recording layer

### 3. Wire Data Passing
- Transport creates `MessageEnvelope` with parsed message
- Recording layer receives both envelope AND raw bytes
- Raw bytes wrapped in Arc to avoid copies
- Allows extraction of SSE metadata when needed

### 4. No sse_retry_ms on MessageContext
- Initially considered but rejected
- SSE retry is a wire format detail
- Only affects reconnection behavior
- Belongs in recording metadata, not message context

## Implementation Notes

### Consolidated SseEvent Structure
```rust
// In transport::sse::event (single canonical type)
pub struct SseEvent {
    pub id: Option<String>,        // SSE event ID (for reconnection)
    pub event_type: String,         // Usually "message" for MCP
    pub data: String,               // Contains JSON-RPC message
    pub retry: Option<u64>,         // Reconnection delay hint
}
```

### Updated TransportContext
```rust
pub enum TransportContext {
    Stdio { 
        process_id: Option<u32>,
        command: Option<String>,
    },
    Http {
        method: String,
        path: String,
        headers: HashMap<String, String>,
        status_code: Option<u16>,
        remote_addr: Option<String>,
        response_mode: Option<ResponseMode>, // NEW: Json/SseStream/Passthrough
    }
    // No Sse variant!
}
```

### Recording with Raw Data
```rust
pub struct RecordingFrame {
    pub envelope: MessageEnvelope,
    pub raw_wire_data: Option<Arc<Vec<u8>>>, // Raw bytes from transport
    pub timestamp: Instant,
}
```

## Related Documents

### Primary References
- [MCP Streamable HTTP Spec](~/src/modelcontextprotocol/modelcontextprotocol/specs/draft/basic/transports.mdx)
- [Current Transport Implementation](~/src/tapwire/shadowcat/src/transport/)
- [Recording Implementation](~/src/tapwire/shadowcat/src/recorder/)

### Task Files
- [Analysis Tasks](tasks/)
- [Design Documents](analysis/)

### Previous Analysis
- Transport type consolidation (completed 2025-08-17)
- HTTP/StreamableHttp consolidation (completed 2025-08-17)

## Next Actions

1. **Complete design document** for recording architecture
2. **Create detailed migration plan** with backward compatibility
3. **Begin SseEvent consolidation** in transport module

## Notes

- This refactor aligns with MCP spec's view of transports
- Maintains backward compatibility for existing recordings
- Improves type safety throughout the system
- Respects proper architectural boundaries
- Part of larger transport architecture cleanup

---

**Document Version**: 1.0  
**Created**: 2025-08-17  
**Last Modified**: 2025-08-17  
**Author**: Claude + Kevin

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-08-17 | 1.0 | Initial plan creation based on analysis | Claude + Kevin |