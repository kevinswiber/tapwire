# Next Session: Shadowcat Recorder Consolidation

## Current Status
**Date**: 2025-01-13  
**Phase 5**: MCP-Aware Recorder - 56% Complete (C.1 & C.2 done)  
**Phase 5.5**: Recorder Consolidation - Ready to Start (Critical Priority)

## Context Files
1. **Main Tracker**: `/Users/kevin/src/tapwire/plans/proxy-sse-message-tracker.md`
2. **Shadowcat Guide**: `/Users/kevin/src/tapwire/shadowcat/CLAUDE.md`

## What Was Just Completed
- ✅ Created `McpTape` format with full MCP semantics (759 lines)
- ✅ Implemented `SessionRecorder` with async buffering (658 lines)
- ✅ Added 12 comprehensive tests, all passing
- ✅ Kept both implementations temporarily to avoid breaking changes

## Critical Issue: Dual Implementation
We now have TWO parallel recorder systems:
1. **Old**: `Tape`, `TapeRecorder` (simple, working, integrated)
2. **New**: `McpTape`, `SessionRecorder` (advanced, MCP-aware, not integrated)

This duplication is technical debt that needs immediate consolidation.

## Phase 5.5: Consolidation Tasks (16 hours)

### Task D.1: Migrate Tape to McpTape (3 hours)
**Challenge**: Old Tape has `frames: Vec<MessageEnvelope>`, new has `frames: Vec<TapeFrame>`

**Approach Options**:
1. **Option A**: Make TapeFrame wrap MessageEnvelope initially
2. **Option B**: Convert all code to use TapeFrame structure
3. **Option C**: Create adapter layer for compatibility

**Deliverables**:
- Single tape format used everywhere
- Migration utility for existing tape files
- All tape tests passing

### Task D.2: Update Storage Layer (2 hours)
- Update `storage.rs` to handle McpTape
- Update `format.rs` for new structure
- Add compression support
- Ensure backward compatibility or migration path

### Task D.3: Migrate TapeRecorder (4 hours)
**Challenge**: Different APIs between TapeRecorder and SessionRecorder

**Approach Options**:
1. **Option A**: Make TapeRecorder a wrapper around SessionRecorder
2. **Option B**: Replace TapeRecorder entirely, update all call sites
3. **Option C**: Merge best of both into single implementation

**Key Differences to Resolve**:
- `start_recording(&Session, String)` vs `start_recording(SessionId, ProtocolVersion, String, Option<CorrelationEngine>)`
- `record_frame(MessageEnvelope)` vs `record_message(MessageEnvelope, Option<InterceptAction>)`

### Task D.4: Update All Call Sites (2 hours)
- Forward proxy: `src/proxy/forward.rs`
- Reverse proxy: `src/proxy/reverse.rs`
- CLI commands: `src/cli/record.rs`, `src/cli/replay.rs`, `src/cli/tape.rs`
- API layer: `src/api.rs`

### Task D.5: Update Replay System (3 hours)
- Update `replay.rs` to handle TapeFrame
- Support correlation-aware playback
- Handle interceptor action replay
- Update CLI replay commands

### Task D.6: Migration Testing (2 hours)
- Test all transport types
- Test replay functionality
- Performance benchmarks
- Migration of existing tapes

## Key Decisions Needed

1. **Migration Strategy**: Gradual adapter layer or big-bang replacement?
2. **API Design**: Keep old API, adopt new API, or hybrid?
3. **Backward Compatibility**: Support old tape files or require migration?
4. **Feature Scope**: Include all McpTape features or start minimal?

## Technical Constraints

- Must maintain working state throughout migration
- 811 existing tests must continue passing
- Zero clippy warnings policy
- Performance target: < 10% recording overhead

## Recommended Approach

1. **Start with D.1**: Create adapter to make TapeFrame work with existing code
2. **Incremental Migration**: Update one component at a time
3. **Test Continuously**: Run full test suite after each change
4. **Feature Flag**: Consider using feature flag for gradual rollout

## Files to Review First

```bash
# Current implementations
src/recorder/tape.rs           # Old implementation (529 lines)
src/recorder/mcp_tape.rs       # New implementation (759 lines)
src/recorder/session_recorder.rs # New recorder (658 lines)

# Key integration points
src/proxy/forward.rs:143-153   # Recording initialization
src/proxy/forward.rs:561       # Frame recording
src/api.rs:445                 # Recorder creation
```

## Success Criteria

- [ ] Single tape format (McpTape)
- [ ] Single recorder implementation
- [ ] All tests passing (811+)
- [ ] Zero clippy warnings
- [ ] No performance regression
- [ ] Clean module structure

## Next Session Focus

Begin with Task D.1 - Migrate Tape to McpTape. This is the foundation for all other consolidation work. The key challenge is converting `Vec<MessageEnvelope>` to `Vec<TapeFrame>` without breaking existing functionality.

Good luck!