# Next Session: Fix SSE Metadata in Transport Layer

## Major Revelation (2025-08-17)
TransportContext is MESSAGE-LEVEL, not session-level! The three variants (Stdio, Http, Sse) correctly represent the three ways a message can be delivered. TransportContext::Sse is NOT a code smell!

## The Real Problem
We're throwing away SSE metadata when buffering events! See `analysis/the-real-problem.md`

## The Simple Fix (30 minutes total!)

### Primary Tasks
1. **Revert buffer simplification** (10 min) - Buffer full `SseEvent` instead of just `Vec<u8>`
2. **Update create_sse_envelope** (10 min) - Use SSE metadata when creating TransportContext::Sse
3. **Test** (10 min) - Verify metadata flows to recording layer

## No Complex Refactoring Needed!
- Keep TransportContext::Sse ✓
- No RawWireData needed ✓
- No 17-hour refactor ✓
- Recording layer already correct ✓

## Key Context from Analysis

### What We Know
- `TransportContext::Sse` should be removed in favor of `Http` with `ResponseMode`
- We have duplicate `SseEvent` structs that need consolidation
- SSE metadata (event_id, event_type, retry_ms) are wire format details, not transport properties
- Recording layer needs raw wire data to extract SSE metadata
- `ResponseMode` already exists in `transport::core` with proper mime parsing
- Neither forward nor reverse proxy uses SSE context directly

### Design Constraints
- Must maintain backward compatibility with existing recordings
- Must preserve type safety (no HashMap<String, Value>)
- Must respect semantic boundaries between layers
- SSE metadata only needed for recording/replay

## Deliverables

### Task A.2: Design Recording Architecture
Create `analysis/recording-architecture.md` with:
- [ ] Detailed data flow from wire to recording
- [ ] Interface between transport and recording layers
- [ ] How raw data will be passed alongside MessageEnvelope
- [ ] Memory management strategy (Arc/Rc for efficiency)
- [ ] Type definitions for new structures

### Task A.3: Migration Plan
Create `analysis/migration-plan.md` with:
- [ ] Step-by-step refactoring sequence
- [ ] Backward compatibility for old tape format
- [ ] Testing strategy at each step
- [ ] Rollback plan if issues found
- [ ] Timeline and risk assessment

### Task B.1: Consolidate SseEvent (if time permits)
- [ ] Identify all SseEvent definitions
- [ ] Create single canonical type in `transport::sse::event`
- [ ] Update all references
- [ ] Ensure tests still pass

## Reference Materials

### Key Files to Review
- `src/transport/sse/event.rs` - Current SseEvent definition
- `src/transport/outgoing/http.rs` - Duplicate SseEvent (lines 52-60)
- `src/recorder/tape.rs` - SseMetadata structure (lines 186-200)
- `src/recorder/session_recorder.rs` - How metadata is extracted (lines 390-420)
- `src/transport/core/response_mode.rs` - ResponseMode enum

### Tracker
- [Traffic Recording Tracker](traffic-recording-tracker.md) - Full context and plan

## Success Criteria
- [ ] Clear architectural design documented
- [ ] Migration plan addresses all risks
- [ ] No breaking changes to existing functionality
- [ ] Type safety maintained throughout

## Notes for Next Session
- Focus on design first before implementation
- Consider performance implications of passing raw data
- Think about how this affects both forward and reverse proxies
- Remember that SSE event_id ≠ JSON-RPC id

---

**Remember**: Update this file at the end of the session with progress and next steps.