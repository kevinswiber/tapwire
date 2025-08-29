# Next Session Prompt - Sprint 3: Advanced Features

## ✅ Sprint 2.5 COMPLETED: Stream Tracking

Successfully implemented typed IDs (SessionId, StreamId, EventId) for compile-time safety and proper MCP spec compliance. 

**What we built**:
- Typed ID system preventing parameter swapping bugs
- Proper stream isolation for SSE event replay
- DashMap-based high-performance concurrent access
- All 206 tests passing

See `analysis/stream-tracking-completion.md` for full implementation details.

## 🚀 Ready for Sprint 3: Advanced Features (38h)

### Next Priority: Task 3.0 - Interceptor Framework (8h)

**Objective**: Build middleware system for message interception and modification
3. Start analysis of current implementation

**Analysis Commands**:
```bash
cd /Users/kevin/src/tapwire/shadowcat-mcp-compliance

# Find EventIdGenerator usage
rg "EventIdGenerator" crates/mcp/src/

# Find EventStore usage  
rg "EventStore|store_event|get_events_after" crates/mcp/src/

# Find Last-Event-ID handling
rg -i "last-event-id" crates/mcp/src/

# Find SSE connection handling
rg "StreamableIncoming|SseEvent" crates/mcp/src/
```

## Deliverables for This Session

1. **Complete Analysis** (Task 2.5.0)
   - Map current event ID flow
   - Document all EventStore touchpoints
   - Understand SSE connection lifecycle
   - Create implementation checklist

2. **Begin Implementation** (if time permits)
   - Start Task 2.5.1: Stream Manager Implementation
   - Create StreamId type and StreamManager

## Context

- **Branch**: feat/mcpspec in shadowcat repo
- **Tests**: All 200 MCP tests currently passing (but with incorrect stream semantics)
- **Priority**: This MUST be fixed before any other work - it's a spec violation

## Success Criteria

By end of session:
- [ ] Complete understanding of changes needed
- [ ] Analysis document written to `analysis/stream-tracking-analysis.md`
- [ ] Implementation checklist created
- [ ] Ready to implement StreamManager (Task 2.5.1)

## Important Notes

- This is a breaking change to EventStore API
- Will affect PersistenceWorker, EventTracker, and StreamableIncomingConnection
- Must maintain backwards compatibility path
- Consider how non-SSE transports (stdio) handle this

Start by reading the design doc and task file, then begin the analysis!