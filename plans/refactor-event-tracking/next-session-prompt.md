# Event Tracking Consolidation - Phase C Ready

## Quick Status Update

✅ **Phase B Complete!** We successfully:
1. Deleted ReverseProxySseManager (331 lines of dead code removed)
2. Added callback support to EventTracker
3. Wired EventTracker to SessionStore via SessionManager
4. All 775 unit tests pass

## Your Mission: Execute Phase C (2.5 hours)

The core infrastructure is done. Now integrate it with the reverse proxy and clean up redundancy.

### Task C.1: Update Reverse Proxy (1.5 hours)

Read the full task details: `plans/refactor-event-tracking/tasks/C.1-update-reverse-proxy.md`

Key steps:
1. Find SSE handling in reverse proxy (likely in `legacy.rs`)
2. Create EventTracker via `session_manager.create_event_tracker()`
3. Handle Last-Event-Id header from clients
4. Use `tracker.record_event_with_dedup()` for efficiency
5. Test with reverse proxy integration tests

### Task C.2: Remove Redundant Tracking (1 hour)

Read the full task details: `plans/refactor-event-tracking/tasks/C.2-remove-redundant-tracking.md`

Key steps:
1. Remove `last_event_id` from ConnectionInfo in `sse_integration.rs`
2. Remove related getter/setter methods
3. Update all call sites to use EventTracker
4. Keep Session.last_event_id (used by store)

## Testing Commands

```bash
# After C.1 - Test reverse proxy
cargo test proxy::reverse
cargo test --test integration_reverse_proxy

# After C.2 - Test cleanup
cargo test session::
cargo test sse_integration

# Full test suite
cargo test
```

## If Time Permits: Phase D (1 hour)

### D.1: Integration Testing

Read the full task details: `plans/refactor-event-tracking/tasks/D.1-integration-testing.md`

Create comprehensive integration tests:
- End-to-end SSE reconnection with deduplication
- Multi-connection session sharing
- Persistence recovery after crash
- Performance validation (<5% overhead)

## Architecture Reminder

```
Session Store (Persistence)
    ↑ callback on new event
Transport EventTracker (Authority)
    ↑ feeds events
SSE Connection Layer
```

The EventTracker is now the single source of truth, with automatic persistence via callbacks to SessionStore.

## Success Metrics

- ✅ Single tracking system (down from 5)
- ✅ Automatic persistence to any SessionStore
- ⬜ Reverse proxy SSE resilience enabled
- ⬜ No redundant tracking code
- ⬜ Full integration test coverage

## Key Files Modified So Far

- ✅ `src/proxy/reverse/sse_resilience.rs` - DELETED
- ✅ `src/transport/sse/reconnect.rs` - Added callbacks
- ✅ `src/session/manager.rs` - Added event tracker methods
- ⬜ `src/proxy/reverse/legacy.rs` - Needs update (C.1)
- ⬜ `src/session/sse_integration.rs` - Needs cleanup (C.2)

## Why This Matters

This consolidation:
1. **Unblocks reverse proxy SSE resilience** - Critical feature
2. **Enables Redis session storage** - Event IDs will persist automatically
3. **Reduces complexity** - From 5 systems to 1
4. **Future-proof** - Works with any SessionStore implementation

## Notes for Next Session

- Total work: 10.5 hours
- Complete: 7 hours (Phases A & B)
- Remaining: 3.5 hours (Phases C & D)
- This unblocks the reverse proxy refactor
- Clean, simple architecture with proper separation of concerns