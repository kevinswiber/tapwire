# Next Session: Minimal Event Tracking Integration

## Project Context

We're consolidating 5 overlapping Last-Event-Id tracking systems into a single authoritative source. This unblocks SSE resilience in the reverse proxy and simplifies the codebase significantly.

**Project**: Event Tracking Refactor
**Tracker**: `plans/refactor-event-tracking/refactor-event-tracking-tracker.md`
**Status**: Phase B - Minimal Integration (0% Complete)

## Current Status

### What Has Been Completed
- **Phase A: Analysis & Planning** (✅ Completed 2025-08-17)
  - Identified 5 overlapping tracking systems
  - Mapped all dependencies and usage
  - Designed unified architecture with transport as authority

### What's In Progress
- **Phase B: Minimal Integration** (Not Started)
  - Duration: 2-3 hours
  - Dependencies: None
  - Goal: Quick fix to unblock SSE resilience

## Your Mission

Implement the minimal changes needed to consolidate event tracking and unblock the reverse proxy SSE resilience feature.

### Priority 1: Wire Transport Tracker (2 hours)

1. **Task B.1: Wire transport tracker to proxy** (1h)
   - Modify `src/proxy/reverse/sse_resilience.rs` to use transport's EventTracker
   - Remove duplicate tracker creation
   - Success: Proxy uses single tracker instance
   
2. **Task B.2: Connect session persistence** (1h)
   - Update session store from transport events
   - Ensure one-way flow: Transport → Session
   - Success: Event IDs persist correctly

### Priority 2: Test Integration (1 hour)

3. **Task B.3: Test SSE resilience** (1h)
   - Test with MCP Inspector
   - Verify deduplication works
   - Confirm reconnection with Last-Event-Id
   - Success: SSE resilience functional

## Essential Context Files to Read

1. **Analysis**: `plans/refactor-event-tracking/analysis/last-event-id-tracking-analysis.md` - Full problem analysis
2. **Transport Tracker**: `shadowcat/src/transport/sse/reconnect.rs` - Core EventTracker to use
3. **Proxy Resilience**: `shadowcat/src/proxy/reverse/sse_resilience.rs` - Needs refactoring
4. **Session Store**: `shadowcat/src/session/store.rs` - Persistence layer

## Working Directory

```bash
cd /Users/kevin/src/tapwire/shadowcat
```

## Commands to Run First

```bash
# Verify current state
cargo check

# Run existing SSE tests
cargo test sse

# Check for issues
cargo clippy --all-targets -- -D warnings
```

## Implementation Strategy

### Phase 1: Understand Current Usage (15 min)
1. Review how `ReverseProxySseManager` creates trackers
2. Trace transport `EventTracker` usage
3. Identify integration points

### Phase 2: Wire Transport Tracker (1 hour)
1. Modify `ReverseProxySseManager` to accept transport tracker
2. Remove `session_trackers` HashMap
3. Use transport's `ReconnectionManager` directly
4. Update all method calls

### Phase 3: Connect Persistence (45 min)
1. Add callback from transport to session store
2. Update `Session.last_event_id` on event receipt
3. Ensure atomic updates

### Phase 4: Test Integration (1 hour)
1. Build reverse proxy with changes
2. Test with MCP Inspector
3. Simulate disconnection/reconnection
4. Verify no duplicate events

## Success Criteria Checklist

- [ ] Single EventTracker instance per stream
- [ ] No duplicate tracker creation
- [ ] Session persistence updated from transport
- [ ] SSE deduplication working
- [ ] Reconnection with Last-Event-Id functional
- [ ] All tests passing
- [ ] No clippy warnings

## Key Commands

```bash
# Development
cd shadowcat
cargo build --release

# Testing
cargo test transport::sse
cargo test proxy::reverse::sse
cargo test session::sse

# Run reverse proxy
./target/release/shadowcat reverse --bind 127.0.0.1:8080 --upstream http://localhost:3000/mcp

# Validation
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Important Notes

- **Keep changes minimal** - We're doing Option A (quick fix) not full refactor
- **Transport is authority** - All other systems should reference it
- **One-way data flow** - Transport → Session, never reverse
- **Test incrementally** - Verify each change works
- **Document decisions** - Update analysis with implementation choices

## Key Design Considerations

1. **EventTracker Ownership**: Transport layer owns the canonical tracker
2. **Persistence Timing**: Update session store after successful deduplication
3. **Thread Safety**: Use Arc for shared tracker references
4. **Backward Compatibility**: Existing session store interface unchanged

## Risk Factors & Blockers

- **Risk**: Breaking existing SSE functionality
  - Mitigation: Test each change incrementally
- **Risk**: Thread safety issues with shared tracker
  - Mitigation: Use Arc<EventTracker> consistently

## Next Steps After This Task

Once Phase B is complete:
- **Resume Reverse Proxy Refactor**: SSE resilience will be unblocked
- **Phase C**: Remove redundant tracking systems (4 hours)
- **Phase D**: Documentation and comprehensive tests (2 hours)

After full consolidation:
- Deprecate unused tracking systems
- Consider Redis integration for distributed tracking

## Model Usage Guidelines

- **IMPORTANT**: This is a focused 2-3 hour task. If context grows large, complete Phase B and create new session for Phase C.

## Session Time Management

**Estimated Session Duration**: 2-3 hours
- Setup & Context: 15 min
- Implementation: 2 hours  
- Testing: 30-45 min
- Documentation: 15 min

---

**Session Goal**: Wire transport EventTracker to reverse proxy and verify SSE resilience works

**Last Updated**: 2025-08-17
**Next Review**: After Phase B completion