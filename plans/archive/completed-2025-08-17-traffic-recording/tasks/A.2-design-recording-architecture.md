# Task A.2: Design Recording Architecture

## Objective
Design a clean architecture for the recording layer that properly handles SSE metadata extraction without violating semantic boundaries between transport and recording concerns.

## Background
The current implementation incorrectly passes SSE wire-format metadata through TransportContext, which is a transport-layer abstraction. SSE metadata (event_id, event_type, retry_ms) are wire format details needed only for faithful recording/replay, not for message transport.

## Key Questions to Answer

1. **Data Flow**
   - How does raw wire data flow from transport to recorder?
   - At what point is SSE metadata extracted?
   - How do we avoid unnecessary copies of large payloads?

2. **Interface Design**
   - What's the interface between transport and recording layers?
   - How does the recorder know when to extract SSE metadata?
   - What signals ResponseMode::EventStream to the recorder?

3. **Memory Management**
   - Should we use Arc or Rc for sharing raw data?
   - When can raw data be dropped?
   - How do we handle streaming vs buffered responses?

4. **Type Safety**
   - How do we maintain type safety without HashMap<String, Value>?
   - Where does SseMetadata live in the type hierarchy?
   - How do we handle different transport types cleanly?

## Deliverables

### 1. Architecture Diagram
Create a detailed flow diagram showing:
- Wire format → Transport → Recording pipeline
- Where SSE parsing happens
- Where metadata extraction occurs
- Data ownership and lifecycle

### 2. Interface Definitions
Define the key interfaces:
```rust
// Example structure (refine in design doc)
pub trait RecordingTransport {
    fn receive_with_raw(&mut self) -> Result<(MessageEnvelope, Option<RawData>)>;
}

pub struct RawData {
    pub bytes: Arc<Vec<u8>>,
    pub format: WireFormat,
}

pub enum WireFormat {
    Json,
    ServerSentEvent(SseEvent),
    Unknown,
}
```

### 3. Data Structures
Define the complete type hierarchy:
- Consolidated `SseEvent` structure
- Updated `TransportContext` without Sse variant  
- Recording layer types for metadata storage
- How `FrameMetadata` relates to transport metadata

### 4. Migration Strategy
- How to handle existing TransportContext::Sse usage
- Backward compatibility for recorded tapes
- Testing approach during migration

## Process

1. **Review Current Implementation** (30 min)
   - Study `src/recorder/session_recorder.rs`
   - Understand `src/transport/outgoing/http.rs` SSE handling
   - Review `src/replay/sse_support.rs`

2. **Design Core Architecture** (1 hour)
   - Draw data flow diagram
   - Define layer boundaries
   - Identify ownership model

3. **Define Interfaces** (45 min)
   - Transport → Recording interface
   - Recording → Storage interface
   - Replay → Transport interface

4. **Document Design** (45 min)
   - Write `analysis/recording-architecture.md`
   - Include code examples
   - Add rationale for decisions

## Success Criteria

- [ ] Clear separation of transport and recording concerns
- [ ] No SSE metadata in TransportContext
- [ ] Type-safe metadata handling
- [ ] Efficient memory usage (no unnecessary copies)
- [ ] Backward compatibility addressed
- [ ] Both forward and reverse proxy scenarios covered

## Dependencies
- Understanding of current SSE handling (complete)
- Analysis of TransportContext usage (complete)
- ResponseMode enum availability (confirmed)

## Notes
- SSE event_id is for reconnection, not JSON-RPC correlation
- event_type is typically "message" for MCP
- retry_ms only affects reconnection behavior
- Recording needs full wire format for replay fidelity

## References
- [MCP Streamable HTTP Spec](~/src/modelcontextprotocol/modelcontextprotocol/specs/draft/basic/transports.mdx)
- [SSE Specification](https://html.spec.whatwg.org/multipage/server-sent-events.html)
- Current implementation: `src/recorder/`, `src/transport/`

---

**Estimated Duration**: 3 hours  
**Output Location**: `plans/traffic-recording/analysis/recording-architecture.md`